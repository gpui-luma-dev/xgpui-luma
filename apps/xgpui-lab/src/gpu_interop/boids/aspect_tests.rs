//! GPU readback acceptance for the wider world and undistorted triangles.
use super::*;
use super::super::engine::{WIDTH, HEIGHT, ASPECT};

fn read(device: &wgpu::Device, buffer: &wgpu::Buffer) -> Vec<u8> {
    let (tx, rx) = std::sync::mpsc::channel();
    buffer.slice(..).map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).unwrap();
    });
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    rx.recv().unwrap().unwrap();
    let data = buffer.slice(..).get_mapped_range().unwrap().to_vec();
    buffer.unmap();
    data
}

#[test]
#[ignore = "requires a compatible GPU, no GUI or window"]
fn wide_world_wraps_at_two_and_draws_without_stretching() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = gpui::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = gpui::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Boids 2:1 acceptance"),
        size: wgpu::Extent3d { width: WIDTH, height: HEIGHT, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    let row = (WIDTH * 4).div_ceil(256) * 256;
    let pixels = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: u64::from(row * HEIGHT),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    // The glyph lives beyond the former square world's +1 boundary.
    let mut glyph = Boids::new(&device, format, 1);
    queue.write_buffer(&glyph.particle_buffers[0], 0, &f32_bytes(&[1.5, 0., 0., 0.1]));
    let mut encoder = device.create_command_encoder(&Default::default());
    glyph.render(&mut encoder, &view, false);
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &pixels,
            layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: None },
        },
        texture.size(),
    );
    queue.submit([encoder.finish()]);
    let data = read(&device, &pixels);
    let (mut left, mut right, mut top, mut bottom) = (WIDTH, 0, HEIGHT, 0);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if data[(y * row + x * 4) as usize] > 128 {
                left = left.min(x);
                right = right.max(x);
                top = top.min(y);
                bottom = bottom.max(y);
            }
        }
    }
    assert!(left < right && top < bottom, "the outer half of the wider world is visible");
    assert!((1048..=1052).contains(&((left + right) / 2)), "world X=1.5 projects near pixel 1050");
    assert!((5..=7).contains(&(right - left + 1)), "triangle width is corrected for aspect ratio");
    assert!((10..=13).contains(&(bottom - top + 1)), "triangle height retains isotropic world scale");

    let mut flock = Boids::new(&device, format, 5);
    let initial = [1.5, 0., 0.1, 0., 1.99, 0.5, 0.1, 0., -1.99, -0.5, -0.1, 0., 0., 0.99, 0., 0.1, 0., -0.99, 0., -0.1];
    queue.write_buffer(&flock.particle_buffers[0], 0, &f32_bytes(&initial));
    // Disable flock forces to isolate velocity and boundary handling.
    queue.write_buffer(&flock.sim_param_buffer, 0, &f32_bytes(&[1., 0.1, 0.025, 0.025, 0., 0., 0., ASPECT]));
    let particles = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 80,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    flock.render(&mut encoder, &view, true);
    encoder.copy_buffer_to_buffer(&flock.particle_buffers[1], 0, &particles, 0, 80);
    queue.submit([encoder.finish()]);
    let data = read(&device, &particles);
    let values: Vec<f32> = data.chunks_exact(4).map(|bytes| f32::from_ne_bytes(bytes.try_into().unwrap())).collect();
    for (index, expected) in [(0, 1.6), (4, -2.), (8, 2.), (13, -1.), (17, 1.)] {
        assert!((values[index] - expected).abs() < 1e-5, "particle scalar {index}: {} != {expected}", values[index]);
    }
}
