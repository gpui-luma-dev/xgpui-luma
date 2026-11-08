//! Native video pixel/range/matrix, mask, opacity and import reuse checks from media-lab/video_lab.rs.
#[cfg(target_os = "macos")]
use {
    anyhow::Result,
    gpui::*,
    std::{path::Path as FilePath, time::Duration},
    xgpui_media::VideoFrame,
};
#[cfg(target_os = "macos")]
use super::super::video_source::SourceFormat;
#[cfg(target_os = "macos")]
pub fn run(png: &FilePath, cancel: &std::sync::atomic::AtomicBool) -> Result<()> {
    super::surface::run(png.parent().unwrap())?;
    let mut renderer =
        gpui_platform::current_headless_renderer()?.ok_or_else(|| anyhow::anyhow!("native renderer unavailable"))?;
    for (index, format) in SourceFormat::ALL.iter().copied().enumerate() {
        super::check_cancel(cancel)?;
        let path = if index == 0 {
            png.to_path_buf()
        } else {
            png.with_file_name(format!("video-{index}.png"))
        };
        capture_format(&mut *renderer, &path, format)?;
    }
    Ok(())
}
#[cfg(target_os = "macos")]
fn capture_format(renderer: &mut dyn PlatformHeadlessRenderer, png: &FilePath, format: SourceFormat) -> Result<()> {
    use anyhow::ensure;
    let frame = format.frame(0)?;
    let video = VideoFrame::core_video(frame.clone(), Duration::ZERO);
    let bounds = Bounds::new(point(ScaledPixels(4.), ScaledPixels(4.)), size(ScaledPixels(320.), ScaledPixels(180.)));
    let mut scene = Scene::default();
    scene.insert_primitive(CustomDraw {
        bounds,
        content_mask: ContentMask { bounds },
        corner_radii: Corners::all(ScaledPixels(12.)),
        content: video.content(),
        coverage: None,
        opacity: 1.,
        order: Default::default(),
    });
    scene.finish();
    let extent = size(DevicePixels(328), DevicePixels(188));
    let pixels = renderer.render_scene_to_image(&scene, extent)?;
    for (x, expected) in [(24, [224u8, 64, 32]), (104, [64, 208, 64]), (184, [32, 64, 224]), (264, [224, 192, 96])] {
        let pixel = pixels.get_pixel(x, 24).0;
        ensure!(
            (0..3).all(|i| pixel[i].abs_diff(expected[i]) <= 3),
            "{} RGB/range/matrix at {x}: {pixel:?}",
            format.label()
        );
    }
    ensure!(pixels.get_pixel(4, 4).0[..3] == [0, 0, 0], "rounded corner coverage");
    let reference = pixels.get_pixel(264, 124).0;
    let expected = if format == SourceFormat::Bgra {
        [112u8, 96, 48]
    } else {
        [224, 192, 96]
    };
    ensure!(
        (0..3).all(|i| reference[i].abs_diff(expected[i]) <= 3),
        "{} alpha/opaque reference: {reference:?}",
        format.label()
    );
    let next = format.frame(8)?;
    scene.custom_draws[0].content = VideoFrame::core_video(next, Duration::from_millis(264)).content();
    let moving = renderer.render_scene_to_image(&scene, extent)?;
    let changed = pixels.pixels().zip(moving.pixels()).filter(|(a, b)| a != b).count();
    ensure!(
        changed > 320 * 180 / 4,
        "{} playback did not update the animated region: {changed} pixels",
        format.label()
    );
    scene.custom_draws[0].content = video.content();
    scene.custom_draws[0].opacity = 0.5;
    let alpha = renderer.render_scene_to_image(&scene, extent)?;
    ensure!(alpha.get_pixel(24, 24).0[0].abs_diff(112) <= 2, "inherited video opacity");
    scene.custom_draws[0].content_mask.bounds.origin.x = ScaledPixels(20.);
    scene.custom_draws[0].content_mask.bounds.size.width = ScaledPixels(304.);
    let clipped = renderer.render_scene_to_image(&scene, extent)?;
    ensure!(clipped.get_pixel(10, 50).0[..3] == [0, 0, 0], "native video content mask");
    scene.custom_draws[0].bounds.size = size(ScaledPixels(160.), ScaledPixels(90.));
    renderer.render_scene_to_image(&scene, extent)?;
    ensure!(frame.import_count() == 1, "resize/replay must reuse imported texture");
    pixels.save(png)?;

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn run(png: &std::path::Path, cancel: &std::sync::atomic::AtomicBool) -> anyhow::Result<()> {
    super::check_cancel(cancel)?;
    super::super::video_source::windows::capture(png)?;
    super::check_cancel(cancel)
}
