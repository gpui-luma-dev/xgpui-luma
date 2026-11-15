//! Wave producer ported from xgpui-ext-lab/gpu-interop-lab.
use anyhow::{Context as _, Result};
use std::time::Instant;
use xgpui_gpu_interop::{AcquireError, SurfaceDesc, SurfaceFormat, WgpuSurfaceTarget, wgpu};
#[cfg(target_os = "macos")]
const BACKEND: (wgpu::Backends, &str) = (wgpu::Backends::METAL, "Metal");
#[cfg(target_os = "linux")]
const BACKEND: (wgpu::Backends, &str) = (wgpu::Backends::VULKAN, "Vulkan");
#[cfg(target_os = "windows")]
const BACKEND: (wgpu::Backends, &str) = (wgpu::Backends::DX12, "D3D12");
const SHADER: &str = r#"
struct Uniforms { time: f32, wide: f32, width: f32, height: f32, work: f32, p0: f32, p1: f32, p2: f32 };
@group(0) @binding(0) var<uniform> u: Uniforms;

@vertex
fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let p = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    return vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0);
}

fn encode_srgb(c: vec3<f32>) -> vec3<f32> {
    let lo = c * 12.92;
    let hi = 1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(hi, lo, c <= vec3<f32>(0.0031308));
}

@fragment
fn fs(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = pos.xy / vec2<f32>(u.width, u.height);
    let t = u.time;
    let v = sin(uv.x * 9.0 + t) + sin(uv.y * 7.0 - t * 1.3) + sin((uv.x + uv.y) * 5.0 + t * 0.7);
    let hue = v * 0.25 + t * 0.1;
    // Saturated primaries: in a wide window these are linear Display P3.
    var color = 0.5 + 0.5 * cos(6.2831853 * (hue + vec3<f32>(0.0, 0.33, 0.67)));
    color = color * color;
    // A moving bar makes dropped or repeated frames visible.
    if abs(uv.x - fract(t * 0.2)) < 0.004 {
        color = vec3<f32>(1.0);
    }
    // Optional extra per-pixel work (stress); contributes invisibly.
    var acc = vec3<f32>(0.0);
    for (var i = 0u; i < u32(u.work); i++) {
        acc = sin(acc * 1.0001 + color + vec3<f32>(f32(i) * 1e-4));
    }
    color = color + acc * 1e-7;
    if u.wide > 0.5 {
        return vec4<f32>(color, 1.0);
    }
    return vec4<f32>(encode_srgb(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0))), 1.0);
}
"#;

/// The independent engine: its own wgpu device, queue and pipeline.
pub(super) struct WaveEngine {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    uniforms: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    target: WgpuSurfaceTarget,
    wide: bool,
    start: Instant,
    size: (u32, u32),
    work: u32,
    bands: u32,
}

impl WaveEngine {
    pub fn new(desc: SurfaceDesc, work: u32) -> Result<Self> {
        Self::with_bands(desc, work, 1)
    }
    fn with_bands(desc: SurfaceDesc, work: u32, bands: u32) -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: BACKEND.0,
            flags: wgpu::InstanceFlags::default(),
            backend_options: wgpu::BackendOptions::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            display: None,
        });
        let adapter = gpui::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .with_context(|| format!("no wgpu {} adapter", BACKEND.1))?;
        // Linux engines also need semaphore export for GPU-side handoff.
        #[cfg(target_os = "linux")]
        let (device, queue) = WgpuSurfaceTarget::request_device(&adapter)?;
        #[cfg(not(target_os = "linux"))]
        let (device, queue) = gpui::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("engine"),
            required_features: WgpuSurfaceTarget::required_features(),
            ..Default::default()
        }))?;
        let target = WgpuSurfaceTarget::new(desc, &device, &queue)?;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("engine_waves"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("engine_uniforms"),
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
            label: Some("engine_waves"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("engine_waves"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(target.format().into())],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("engine_uniforms"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("engine_uniforms"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniforms.as_entire_binding() }],
        });
        Ok(Self {
            device,
            queue,
            pipeline,
            uniforms,
            bind_group,
            wide: desc.format == SurfaceFormat::Rgba16FloatLinearP3,
            target,
            start: Instant::now(),
            size: (desc.width, desc.height),
            work,
            bands: bands.clamp(1, desc.height),
        })
    }

    pub fn surface(&self) -> &xgpui_gpu_interop::EngineSurface {
        self.target.surface()
    }
    pub fn set_work(&mut self, work: u32) {
        self.work = work;
    }
    pub fn set_bands(&mut self, bands: u32) {
        self.bands = bands.clamp(1, self.size.1);
    }
    /// Render and present one frame; `false` when the ring was full.
    pub fn frame(&mut self) -> Result<bool> {
        let slot = match self.target.surface().acquire() {
            Ok(slot) => slot,
            Err(AcquireError::WouldBlock) => return Ok(false),
            Err(error @ AcquireError::Invalidated) => return Err(error.into()),
        };
        let texture = self.target.texture(&slot)?;
        let view = texture.create_view(&Default::default());
        let uniforms = [
            self.start.elapsed().as_secs_f32(),
            if self.wide { 1. } else { 0. },
            self.size.0 as f32,
            self.size.1 as f32,
            self.work as f32,
            0.,
            0.,
            0.,
        ];
        let bytes: Vec<u8> = uniforms.iter().flat_map(|v| v.to_ne_bytes()).collect();
        self.queue.write_buffer(&self.uniforms, 0, &bytes);
        // Each band is its own submission, so the GPU can schedule the
        // compositor's work between them; the last goes with `present`.
        let (width, height) = self.size;
        let mut last = None;
        for band in 0..self.bands {
            let y0 = height * band / self.bands;
            let y1 = height * (band + 1) / self.bands;
            let mut encoder = self.device.create_command_encoder(&Default::default());
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("engine_waves"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                    })],
                    ..Default::default()
                });
                pass.set_scissor_rect(0, y0, width, y1 - y0);
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.draw(0..3, 0..1);
            }
            if let Some(previous) = last.replace(encoder.finish()) {
                self.queue.submit([previous]);
            }
        }
        self.target.present(slot, last)?;
        Ok(true)
    }
}
