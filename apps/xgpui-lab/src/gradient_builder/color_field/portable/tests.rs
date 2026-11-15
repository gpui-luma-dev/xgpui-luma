use super::*;
use crate::gradient_builder::color_field::{Mixing, tests::fixture};
use std::time::Duration;
#[cfg(target_os = "macos")]
use std::time::Instant;

#[test]
fn wgsl_validates_and_upload_layout_matches() {
    let module = naga::front::wgsl::parse_str(SHADER).unwrap();
    naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
        .validate(&module)
        .unwrap();
    assert_eq!(std::mem::size_of::<Parameters>(), 64 + 16 * 64);
    let mut field = fixture();
    field.points = vec![field.points[0].clone(); 17];
    assert!(Parameters::new(&field, false, [32, 24]).is_err());
}

fn readback(engine: &Engine, field: &ColorField, wide: bool, extent: [u32; 2]) -> Vec<u8> {
    let [width, height] = extent;
    let bpp = if wide { 8 } else { 4 };
    let row = (width * bpp).div_ceil(256) * 256;
    let texture = engine.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Freeform readback"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: if wide {
            wgpu::TextureFormat::Rgba16Float
        } else {
            wgpu::TextureFormat::Bgra8Unorm
        },
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let buffer = engine.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Freeform readback"),
        size: u64::from(row * height),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let render = engine.encode(&texture, field, wide).unwrap();
    let mut encoder = engine.device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: Some(height) },
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
fn expected(field: &ColorField, uv: [f32; 2], wide: bool) -> [f32; 4] {
    let mut color = field.sample(uv).map(|v| v.clamp(0., 1.));
    if wide {
        for i in 0..3 {
            let v = color[i];
            color[i] = if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            } * color[3];
        }
    }
    color
}
fn verify(pixels: &[u8], field: &ColorField, wide: bool, extent: [u32; 2]) {
    let [width, height] = extent;
    let bpp = if wide { 8 } else { 4 };
    let row = (width * bpp).div_ceil(256) * 256;
    for y in 0..height {
        for x in 0..width {
            let offset = (y * row + x * bpp) as usize;
            let actual = if wide {
                std::array::from_fn(|i| half(u16::from_le_bytes([pixels[offset + i * 2], pixels[offset + i * 2 + 1]])))
            } else {
                [
                    pixels[offset + 2] as f32 / 255.,
                    pixels[offset + 1] as f32 / 255.,
                    pixels[offset] as f32 / 255.,
                    pixels[offset + 3] as f32 / 255.,
                ]
            };
            let reference = expected(field, [(x as f32 + 0.5) / width as f32, (y as f32 + 0.5) / height as f32], wide);
            for (a, b) in actual.into_iter().zip(reference) {
                assert!(
                    (a - b).abs() <= if wide { 0.0015 } else { 1.1 / 255. },
                    "{:?} hsl={} wide={wide} ({x},{y}): {actual:?} != {reference:?}",
                    field.mixing,
                    field.hsl
                );
            }
        }
    }
}

#[test]
#[ignore = "requires a GPU, no GUI or window"]
fn wgpu_field_matches_cpu_for_all_modes() {
    let engine = Engine::new().unwrap();
    for mixing in [Mixing::Ordered, Mixing::Normalized] {
        for hsl in [false, true] {
            for wide in [false, true] {
                for count in [0, 2, 16] {
                    let mut field = fixture();
                    field.mixing = mixing;
                    field.hsl = hsl;
                    field.softness = if hsl { 0.25 } else { 2. };
                    let base = field.points.clone();
                    field.points = (0..count)
                        .map(|i| {
                            let mut p = base[i % 2].clone();
                            p.hsl[0] -= 1.;
                            p.color[3] *= 0.2;
                            p.angle += i as f32 * 7.;
                            p
                        })
                        .collect();
                    verify(&readback(&engine, &field, wide, [33, 25]), &field, wide, [33, 25]);
                }
                let mut transparent = fixture();
                transparent.mixing = mixing;
                transparent.hsl = hsl;
                transparent.background[3] = 0.;
                for p in &mut transparent.points {
                    p.color[3] = 0.;
                }
                verify(&readback(&engine, &transparent, wide, [33, 25]), &transparent, wide, [33, 25]);
            }
        }
    }
}

#[test]
#[ignore = "requires GPU interop, no GUI or window"]
fn interop_preserves_snapshots_and_recreates_target_on_resize() {
    let mut engine = Engine::new().unwrap();
    let mut field = fixture();
    let first = engine.render(&field, false, [33, 25]).unwrap().unwrap();
    let snapshot = first.preview_content().unwrap();
    assert_eq!(first.stats().presents, 1);
    field.points[0].position[0] += 0.1;
    engine.render(&field, false, [33, 25]).unwrap().unwrap();
    assert_eq!(first.stats().presents, 2);
    let second_snapshot = first.preview_content().unwrap();
    engine.render(&field, false, [33, 25]).unwrap().unwrap();
    let third_snapshot = first.preview_content().unwrap();
    assert!(engine.render(&field, false, [33, 25]).unwrap().is_none());
    drop(second_snapshot);
    drop(third_snapshot);
    let resized = engine.render(&field, true, [65, 27]).unwrap().unwrap();
    assert_eq!(resized.desc().width, 65);
    assert_eq!(resized.desc().format, SurfaceFormat::Rgba16FloatLinearP3);
    assert_eq!(first.desc().width, 33);
    assert_eq!(first.stats().presents, 3);
    drop(snapshot);
    assert!(engine.render(&field, false, [0, 25]).is_err());
    engine.lost.store(true, Ordering::Release);
    assert!(engine.render(&field, true, [65, 27]).is_err());
}

#[cfg(target_os = "macos")]
#[test]
#[ignore = "native Metal/wgpu performance comparison; headless, no GUI"]
fn compare_metal_wgpu_field_performance() {
    use crate::gradient_builder::color_field::{Output, Paint, PipelineCache, metal::tests::readback as metal_readback};
    use gpui_apple::{custom_draw::MetalCustomContext, metal::*};
    objc2::rc::autoreleasepool(|_| {
        let start = Instant::now();
        let mut engine = Engine::new().unwrap();
        eprintln!("wgpu device + two pipelines cold setup: {:?}", start.elapsed());
        let device = MTLCreateSystemDefaultDevice().unwrap();
        let ctx = MetalCustomContext::new(device.clone(), device.newCommandQueue().unwrap());
        let pipeline = Arc::new(PipelineCache::default());
        let extent = [640u32, 400u32];
        let mut field = fixture();
        let base = field.points.clone();
        field.points = (0..16).map(|i| base[i % 2].clone()).collect();
        for wide in [false, true] {
            let output = if wide { Output::DisplayP3 } else { Output::Srgb };
            // Cold first render includes native lazy compilation and allocation.
            for cold in [true, false] {
                let count = if cold { 1 } else { 20 };
                let mut native_time = Duration::ZERO;
                let mut wgpu_time = Duration::ZERO;
                for i in 0..count {
                    field.points[0].position[0] = 0.25 + i as f32 * 0.001;
                    let paint = Paint::new(field.clone(), output, pipeline.clone()).unwrap();
                    let start = Instant::now();
                    let native = metal_readback(&paint, &ctx, extent[0] as usize, extent[1] as usize);
                    native_time += start.elapsed();
                    let start = Instant::now();
                    let portable = readback(&engine, &field, wide, extent);
                    wgpu_time += start.elapsed();
                    if i == 0 {
                        verify(&native, &field, wide, extent);
                        verify(&portable, &field, wide, extent);
                    }
                }
                eprintln!(
                    "640x400 16 points {output:?} cold={cold}: Metal {:?}, wgpu {:?} per edit (render + allocation + synchronized readback; excludes compositor)",
                    native_time / count,
                    wgpu_time / count
                );
            }
        }
        use gpui::{DevicePixels, PlatformHeadlessRenderer, size};
        use gpui_apple::metal_renderer::MetalHeadlessRenderer;
        let mut renderer = MetalHeadlessRenderer::new().unwrap();
        for wide in [false, true] {
            renderer
                .set_color_mode(if wide {
                    gpui::WindowColorMode::WideGamutSdr
                } else {
                    gpui::WindowColorMode::Default
                })
                .unwrap();
            let output = if wide { Output::DisplayP3 } else { Output::Srgb };
            let viewport = size(DevicePixels(640), DevicePixels(400));
            let mut native_time = Duration::ZERO;
            let mut interop_time = Duration::ZERO;
            // First iteration warms the actual interop/compositor path, outside the measured samples.
            for i in 0..21 {
                field.points[0].position[0] = 0.25 + i as f32 * 0.001;
                let paint = Arc::new(Paint::new(field.clone(), output, pipeline.clone()).unwrap());
                let start = Instant::now();
                renderer.render_scene_to_image(&scene(paint, 640., 400.), viewport).unwrap();
                let native_elapsed = start.elapsed();
                let start = Instant::now();
                let surface = engine.render(&field, wide, extent).unwrap().unwrap();
                let content = if wide {
                    surface.content().unwrap()
                } else {
                    surface.preview_content().unwrap()
                };
                renderer.render_scene_to_image(&scene(content, 640., 400.), viewport).unwrap();
                if i > 0 {
                    native_time += native_elapsed;
                    interop_time += start.elapsed();
                }
            }
            eprintln!(
                "640x400 16 points {output:?} warm headless presentation: Metal {:?}, wgpu interop {:?} per edit (includes compositor + synchronized readback; excludes window/display scheduling)",
                native_time / 20,
                interop_time / 20
            );
        }
    });
}

#[cfg(target_os = "macos")]
#[test]
#[ignore = "headless Metal compositor, no GUI or window"]
fn interop_composites_same_pixels_as_native_metal() {
    use crate::gradient_builder::color_field::{Output, Paint, PipelineCache};
    use gpui::{DevicePixels, PlatformHeadlessRenderer, size};
    use gpui_apple::metal_renderer::MetalHeadlessRenderer;
    objc2::rc::autoreleasepool(|_| {
        let mut engine = Engine::new().unwrap();
        let mut renderer = MetalHeadlessRenderer::new().unwrap();
        let pipeline = Arc::new(PipelineCache::default());
        let extent = size(DevicePixels(33), DevicePixels(25));
        for mixing in [Mixing::Ordered, Mixing::Normalized] {
            for wide in [false, true] {
                let mut field = fixture();
                field.mixing = mixing;
                let surface = engine.render(&field, wide, [33, 25]).unwrap().unwrap();
                renderer
                    .set_color_mode(if wide {
                        gpui::WindowColorMode::WideGamutSdr
                    } else {
                        gpui::WindowColorMode::Default
                    })
                    .unwrap();
                let portable = renderer
                    .render_scene_to_image(
                        &scene(
                            if wide {
                                surface.content().unwrap()
                            } else {
                                surface.preview_content().unwrap()
                            },
                            33.,
                            25.,
                        ),
                        extent,
                    )
                    .unwrap();
                let paint = Arc::new(
                    Paint::new(field, if wide { Output::DisplayP3 } else { Output::Srgb }, pipeline.clone()).unwrap(),
                );
                let native = renderer.render_scene_to_image(&scene(paint, 33., 25.), extent).unwrap();
                let max = portable.as_raw().iter().zip(native.as_raw()).map(|(a, b)| a.abs_diff(*b)).max().unwrap();
                assert!(max <= 2, "interop/native compositor mismatch {mixing:?} wide={wide}: max delta {max}");
                assert!(surface.unsupported().is_none());
                assert!(surface.stats().composited > 0);
            }
        }
    });
}

#[cfg(target_os = "macos")]
fn scene(content: Arc<dyn gpui::CustomContent>, width: f32, height: f32) -> gpui::Scene {
    use gpui::{Bounds, ContentMask, CustomDraw, ScaledPixels, Scene, point, size};
    let bounds =
        Bounds::new(point(ScaledPixels(0.), ScaledPixels(0.)), size(ScaledPixels(width), ScaledPixels(height)));
    let mut scene = Scene::default();
    scene.insert_primitive(CustomDraw {
        order: 0,
        bounds,
        content_mask: ContentMask { bounds },
        corner_radii: Default::default(),
        opacity: 0.7,
        content,
        coverage: None,
    });
    scene.finish();
    scene
}
