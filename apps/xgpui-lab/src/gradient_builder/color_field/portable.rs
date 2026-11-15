//! Retained wgpu producer. All methods run on the background executor, never in prepaint.
use super::ColorField;
use anyhow::{Context as _, Result, ensure};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use xgpui_gpu_interop::{AcquireError, EngineSurface, SurfaceDesc, SurfaceFormat, WgpuSurfaceTarget, wgpu};

pub(super) const SHADER: &str = include_str!("field.wgsl");
#[cfg(target_os = "macos")]
const BACKEND: wgpu::Backends = wgpu::Backends::METAL;
#[cfg(target_os = "linux")]
const BACKEND: wgpu::Backends = wgpu::Backends::VULKAN;
#[cfg(target_os = "windows")]
const BACKEND: wgpu::Backends = wgpu::Backends::DX12;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Point {
    geometry: [f32; 4],
    color: [f32; 4],
    hsl: [f32; 4],
    shape: [f32; 4],
}
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Parameters {
    background: [f32; 4],
    background_hsl: [f32; 4],
    options: [f32; 4],
    shape: [f32; 4],
    points: [Point; 16],
}
impl Parameters {
    fn new(field: &ColorField, wide: bool, extent: [u32; 2]) -> Result<Self> {
        use bytemuck::Zeroable;
        field.validate()?;
        let mut p = Self::zeroed();
        p.background = field.background;
        p.background_hsl[..3].copy_from_slice(&field.background_hsl);
        p.options = [
            field.points.len() as f32,
            f32::from(field.mixing == super::Mixing::Normalized),
            f32::from(field.hsl),
            field.background_weight,
        ];
        p.shape = [1. / field.softness, f32::from(wide), extent[0] as f32, extent[1] as f32];
        for (out, point) in p.points.iter_mut().zip(&field.points) {
            let (sin, cos) = point.angle.to_radians().sin_cos();
            *out = Point {
                geometry: [point.position[0], point.position[1], cos, sin],
                color: point.color,
                hsl: [point.hsl[0], point.hsl[1], point.hsl[2], 0.],
                shape: [point.radius, point.stretch, 0., 0.],
            };
        }
        Ok(p)
    }
}

pub(crate) struct Engine {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipelines: [wgpu::RenderPipeline; 2],
    uniforms: wgpu::Buffer,
    bind: wgpu::BindGroup,
    target: Option<WgpuSurfaceTarget>,
    lost: Arc<AtomicBool>,
}
impl Engine {
    pub(crate) fn new() -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: BACKEND,
            flags: Default::default(),
            backend_options: Default::default(),
            memory_budget_thresholds: Default::default(),
            display: None,
        });
        let adapter = gpui::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .context("No compatible wgpu adapter for Freeform")?;
        #[cfg(test)]
        eprintln!("Freeform wgpu adapter: {:?}", adapter.get_info());
        #[cfg(target_os = "linux")]
        let (device, queue) = WgpuSurfaceTarget::request_device(&adapter)?;
        #[cfg(not(target_os = "linux"))]
        let (device, queue) = gpui::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Freeform interop"),
            required_features: WgpuSurfaceTarget::required_features(),
            ..Default::default()
        }))?;
        let lost = Arc::new(AtomicBool::new(false));
        let flag = lost.clone();
        device.set_device_lost_callback(move |_, _| {
            flag.store(true, Ordering::Release);
        });
        let errors = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Freeform"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Freeform"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Freeform"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipelines = [wgpu::TextureFormat::Bgra8Unorm, wgpu::TextureFormat::Rgba16Float].map(|format| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Freeform"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vertex"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some("fragment"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            })
        });
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Freeform"),
            size: std::mem::size_of::<Parameters>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Freeform"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniforms.as_entire_binding() }],
        });
        if let Some(error) = gpui::block_on(errors.pop()) {
            anyhow::bail!("Freeform pipeline: {error}");
        }
        Ok(Self { device, queue, pipelines, uniforms, bind, target: None, lost })
    }

    fn encode(&self, texture: &wgpu::Texture, field: &ColorField, wide: bool) -> Result<wgpu::CommandBuffer> {
        ensure!(
            !self.lost.load(Ordering::Acquire),
            "Freeform GPU device lost; switch away and back to wgpu to retry"
        );
        let p = Parameters::new(field, wide, [texture.width(), texture.height()])?;
        self.queue.write_buffer(&self.uniforms, 0, bytemuck::bytes_of(&p));
        let view = texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Freeform"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            pass.set_pipeline(&self.pipelines[usize::from(wide)]);
            pass.set_bind_group(0, &self.bind, &[]);
            pass.draw(0..3, 0..1);
        }
        Ok(encoder.finish())
    }

    /// None means bounded backpressure: keep the last frame and retry later.
    pub(crate) fn render(&mut self, field: &ColorField, wide: bool, extent: [u32; 2]) -> Result<Option<EngineSurface>> {
        let desc = SurfaceDesc {
            width: extent[0],
            height: extent[1],
            slots: 3,
            format: if wide {
                SurfaceFormat::Rgba16FloatLinearP3
            } else {
                SurfaceFormat::Bgra8Srgb
            },
        };
        desc.validate()?;
        // Keep the prototype's 64 MiB per-image limit (three retained slots).
        ensure!(
            u64::from(desc.width) * u64::from(desc.height) * u64::from(desc.format.bytes_per_pixel())
                <= 64 * 1024 * 1024,
            "Freeform image exceeds 64 MiB"
        );
        if self.target.as_ref().is_none_or(|t| t.surface().desc() != desc) {
            self.target = Some(WgpuSurfaceTarget::new(desc, &self.device, &self.queue)?);
        }
        let target = self.target.as_mut().context("Freeform target unavailable")?;
        if let Some(error) = target.surface().unsupported() {
            anyhow::bail!("{error}");
        }
        let slot = match target.surface().acquire() {
            Ok(slot) => slot,
            Err(AcquireError::WouldBlock) => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        let texture = target.texture(&slot)?;
        let command = self.encode(&texture, field, wide)?;
        let target = self.target.as_mut().context("Freeform target unavailable")?;
        target.present(slot, [command])?;
        Ok(Some(target.surface().clone()))
    }
}

#[cfg(test)]
mod tests;
