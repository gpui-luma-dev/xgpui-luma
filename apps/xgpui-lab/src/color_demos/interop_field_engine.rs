//! Retained wgpu producer. All methods run on the background executor, never in prepaint.
use super::interop_field::Request;
use anyhow::{Context as _, Result, ensure};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use xgpui_gpu_interop::{AcquireError, EngineSurface, SurfaceDesc, SurfaceFormat, WgpuSurfaceTarget, wgpu};

#[cfg(target_os = "macos")]
const BACKEND: wgpu::Backends = wgpu::Backends::METAL;
#[cfg(target_os = "linux")]
const BACKEND: wgpu::Backends = wgpu::Backends::VULKAN;
#[cfg(target_os = "windows")]
const BACKEND: wgpu::Backends = wgpu::Backends::DX12;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    parameters: super::hv_field_shader::Parameters,
    viewport: [f32; 4],
}

pub(super) fn shader() -> String {
    format!(
        "{}\n{}\n{}",
        xgpui_color::WGSL_COLOR_MATH,
        include_str!("hv_field.wgsl"),
        include_str!("interop_field.wgsl")
    )
}

pub(crate) struct Engine {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
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
            .context("No compatible wgpu adapter for Color Spaces")?;
        #[cfg(test)]
        eprintln!("Color Spaces wgpu adapter: {:?}", adapter.get_info());
        #[cfg(target_os = "linux")]
        let (device, queue) = WgpuSurfaceTarget::request_device(&adapter)?;
        #[cfg(not(target_os = "linux"))]
        let (device, queue) = gpui::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Color Spaces interop"),
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
            label: Some("Color Spaces"),
            source: wgpu::ShaderSource::Wgsl(shader().into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Color Spaces"),
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
            label: Some("Color Spaces"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = {
            let format = wgpu::TextureFormat::Rgba16Float;
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Color Spaces"),
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
        };
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Color Spaces"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Color Spaces"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniforms.as_entire_binding() }],
        });
        if let Some(error) = gpui::block_on(errors.pop()) {
            anyhow::bail!("Color Spaces pipeline: {error}");
        }
        Ok(Self { device, queue, pipeline, uniforms, bind, target: None, lost })
    }

    fn encode(&self, texture: &wgpu::Texture, request: &Request) -> Result<wgpu::CommandBuffer> {
        ensure!(
            !self.lost.load(Ordering::Acquire),
            "Color Spaces GPU device lost; switch away and back to wgpu to retry"
        );
        let p = Uniforms {
            parameters: super::hv_field_shader::Parameters::field(request.source, request.saturation)
                .with_contour(request.contour),
            viewport: [
                request.extent[0] as f32 / request.scale,
                request.extent[1] as f32 / request.scale,
                request.scale,
                0.,
            ],
        };
        self.queue.write_buffer(&self.uniforms, 0, bytemuck::bytes_of(&p));
        let view = texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Color Spaces"),
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
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind, &[]);
            pass.draw(0..3, 0..1);
        }
        Ok(encoder.finish())
    }

    /// None means bounded backpressure: keep the last frame and retry later.
    pub(crate) fn render(&mut self, request: &Request) -> Result<Option<EngineSurface>> {
        let desc = SurfaceDesc {
            width: request.extent[0],
            height: request.extent[1],
            slots: 3,
            format: SurfaceFormat::Rgba16FloatLinearP3,
        };
        desc.validate()?;
        // Keep the prototype's 64 MiB per-image limit (three retained slots).
        ensure!(
            u64::from(desc.width) * u64::from(desc.height) * u64::from(desc.format.bytes_per_pixel())
                <= 64 * 1024 * 1024,
            "Color Spaces image exceeds 64 MiB"
        );
        if self.target.as_ref().is_none_or(|t| t.surface().desc() != desc) {
            self.target = Some(WgpuSurfaceTarget::new(desc, &self.device, &self.queue)?);
        }
        let target = self.target.as_mut().context("Color Spaces target unavailable")?;
        if let Some(error) = target.surface().unsupported() {
            anyhow::bail!("{error}");
        }
        let slot = match target.surface().acquire() {
            Ok(slot) => slot,
            Err(AcquireError::WouldBlock) => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        let texture = target.texture(&slot)?;
        let command = self.encode(&texture, request)?;
        let target = self.target.as_mut().context("Color Spaces target unavailable")?;
        target.present(slot, [command])?;
        Ok(Some(target.surface().clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    #[test]
    fn shared_shader_and_uniform_layout_validate() {
        let module = naga::front::wgsl::parse_str(&shader()).unwrap();
        naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
            .validate(&module)
            .unwrap();
        assert_eq!(std::mem::size_of::<Uniforms>(), 64);
    }
    fn readback(engine: &Engine, request: &Request) -> Vec<u8> {
        let [width, height] = request.extent;
        let bpp = 8;
        let row = (width * bpp).div_ceil(256) * 256;
        let texture = engine.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Freeform readback"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let buffer = engine.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Freeform readback"),
            size: u64::from(row * height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let render = engine.encode(&texture, request).unwrap();
        let mut encoder = engine.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(row),
                    rows_per_image: Some(height),
                },
            },
            texture.size(),
        );
        let submission = engine.queue.submit([render, encoder.finish()]);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer.slice(..).map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        engine
            .device
            .poll(wgpu::PollType::Wait { submission_index: Some(submission), timeout: Some(Duration::from_secs(10)) })
            .unwrap();
        receiver.recv_timeout(Duration::from_secs(10)).unwrap().unwrap();
        buffer.slice(..).get_mapped_range().unwrap().to_vec()
    }
    fn half(bits: u16) -> f32 {
        let sign = if bits & 0x8000 == 0 { 1. } else { -1. };
        let exponent = ((bits >> 10) & 31) as i32;
        let fraction = (bits & 1023) as f32;
        sign * if exponent == 0 {
            fraction * 2f32.powi(-24)
        } else {
            (1. + fraction / 1024.) * 2f32.powi(exponent - 15)
        }
    }

    #[test]
    #[ignore = "requires a GPU, no GUI or window"]
    fn interop_field_matches_source_colors_without_p3_clipping() {
        use xgpui_color::{LinearRgb, RgbSpace};
        let engine = Engine::new().unwrap();
        let mut saw_extended = false;
        for source in 0..4 {
            for scale in [1., 2.] {
                for saturation in [0., 0.62, 1.] {
                    let extent = [(64. * scale) as u32, (32. * scale) as u32];
                    let request = Request { source, saturation, contour: false, extent, scale };
                    let actual = readback(&engine, &request);
                    let reference =
                        super::super::native_cpu_field::pixels(source, extent[0], extent[1], scale, saturation)
                            .unwrap();
                    let row = (extent[0] * 8).div_ceil(256) * 256;
                    for y in 0..extent[1] {
                        for x in 0..extent[0] {
                            let rgb = reference[(y * extent[0] + x) as usize];
                            let expected = LinearRgb::new(
                                super::super::source_field::SPACES[source],
                                [rgb[0] as f64, rgb[1] as f64, rgb[2] as f64],
                            )
                            .unwrap()
                            .convert(RgbSpace::DisplayP3)
                            .unwrap()
                            .components();
                            saw_extended |= expected.iter().any(|v| *v < -0.01 || *v > 1.01);
                            let offset = (y * row + x * 8) as usize;
                            for (i, expected) in expected.into_iter().enumerate() {
                                let value =
                                    half(u16::from_le_bytes([actual[offset + i * 2], actual[offset + i * 2 + 1]]))
                                        as f64;
                                assert!(
                                    (value - expected).abs() < 0.004,
                                    "source={source} scale={scale} saturation={saturation} ({x},{y}) channel={i}: {value} != {expected}"
                                );
                            }
                        }
                    }
                }
            }
        }
        assert!(saw_extended, "must exercise colors beyond P3");
    }
    #[cfg(target_os = "macos")]
    #[test]
    #[ignore = "requires headless Metal compositor, no GUI or window"]
    fn shared_surface_composites_extended_linear_p3() {
        use gpui::{Bounds, ContentMask, CustomDraw, DevicePixels, ScaledPixels, Scene, point, size};
        let mut engine = Engine::new().unwrap();
        let mut renderer = gpui_apple::metal_renderer::MetalHeadlessRenderer::new().unwrap();
        renderer.set_color_mode(gpui::WindowColorMode::WideGamutSdr).unwrap();
        for source in 0..4 {
            let request = Request { source, saturation: 1., contour: true, extent: [64, 32], scale: 1. };
            let reference = readback(&engine, &request);
            let surface = engine.render(&request).unwrap().unwrap();
            let bounds =
                Bounds::new(point(ScaledPixels(0.), ScaledPixels(0.)), size(ScaledPixels(64.), ScaledPixels(32.)));
            let mut scene = Scene::default();
            scene.insert_primitive(CustomDraw {
                order: 0,
                bounds,
                content_mask: ContentMask { bounds },
                corner_radii: Default::default(),
                opacity: 1.,
                content: surface.content().unwrap(),
                coverage: None,
            });
            scene.finish();
            let image = renderer.render_scene_to_linear_p3(&scene, size(DevicePixels(64), DevicePixels(32))).unwrap();
            for y in 2..30 {
                for x in 2..62 {
                    let index = y * 64 + x;
                    for channel in 0..3 {
                        let offset = index * 8 + channel * 2;
                        let expected = half(u16::from_le_bytes([reference[offset], reference[offset + 1]]));
                        let actual = half(image.pixels[index][channel]);
                        assert!(
                            (actual - expected).abs() < 0.004,
                            "source={source} ({x},{y}) channel={channel}: {actual} != {expected}"
                        );
                    }
                }
            }
            assert!(surface.unsupported().is_none());
            assert!(surface.stats().composited > 0);
        }
    }
}
