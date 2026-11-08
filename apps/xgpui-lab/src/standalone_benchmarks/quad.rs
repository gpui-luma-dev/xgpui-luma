use anyhow::{Context as _, Result, ensure};
use clap::{Parser, ValueEnum};
use gpui::{Bounds, Hsla, Pixels, Quad, Scene, Size, point, prelude::*, px, rgb, size};
use serde::Serialize;
use std::{fs, path::PathBuf, time::Instant};
#[derive(Clone, Copy, Debug, ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Pattern {
    Grid,
    Overlap,
}

#[derive(Clone, Parser)]
#[command(
    args_override_self = true,
    about = "One workload: GPUI quad composition and drawing. UI by default; --headless for automation."
)]
#[allow(dead_code)] // Original validated CLI fields; windowed launch options are not used by the embedded runner.
struct Options {
    #[arg(long)]
    headless: bool,
    /// Keep the UI drawing until its window is closed.
    #[arg(long, conflicts_with_all = ["headless", "json"])]
    continuous: bool,
    /// Seconds to leave the completed UI visible before quitting (default: immediate).
    #[arg(long, default_value_t = 0, value_parser = clap::value_parser!(u64).range(0..=3600), conflicts_with_all = ["headless", "continuous"])]
    exit_delay_secs: u64,
    #[arg(long, requires = "headless", conflicts_with = "png")]
    cpu_only: bool,
    #[arg(long, default_value_t = 1000)]
    count: usize,
    #[arg(long, value_enum, default_value_t = Pattern::Grid)]
    pattern: Pattern,
    #[arg(long, default_value_t = 0.65)]
    opacity: f32,
    #[arg(long, default_value_t = 960)]
    width: u32,
    #[arg(long, default_value_t = 640)]
    height: u32,
    #[arg(long, default_value_t = 60)]
    frames: usize,
    #[arg(long, default_value_t = 10)]
    warmup: usize,
    #[arg(long)]
    json: Option<PathBuf>,
    #[arg(long, requires = "headless")]
    png: Option<PathBuf>,
}
impl Options {
    fn validate(&self) -> Result<()> {
        ensure!((1..=20_000).contains(&self.count), "count must be 1..=20000");
        ensure!(
            (64..=4096).contains(&self.width) && (64..=4096).contains(&self.height),
            "width and height must be 64..=4096"
        );
        ensure!(
            self.opacity.is_finite() && self.opacity > 0.0 && self.opacity <= 1.0,
            "opacity must be finite and in (0, 1]"
        );
        ensure!(
            (1..=10_000).contains(&self.frames) && self.warmup <= 1000,
            "frames must be 1..=10000; warmup at most 1000"
        );
        Ok(())
    }
}

// Both modes use exactly this geometry and color generator. UI uses logical
// pixels; headless fixes the scale factor at 1 for controlled comparisons.
fn workload(options: &Options, viewport: Size<Pixels>) -> Vec<(Bounds<Pixels>, Hsla)> {
    let columns = (options.count as f32).sqrt().ceil() as usize;
    let rows = options.count.div_ceil(columns);
    let cell_w = f32::from(viewport.width) / columns as f32;
    let cell_h = f32::from(viewport.height) / rows as f32;
    (0..options.count)
        .map(|i| {
            let (x, y, w, h) = match options.pattern {
                Pattern::Grid => {
                    ((i % columns) as f32 * cell_w, (i / columns) as f32 * cell_h, cell_w * 0.85, cell_h * 0.85)
                }
                Pattern::Overlap => (
                    f32::from(viewport.width) * 0.3,
                    f32::from(viewport.height) * 0.3,
                    f32::from(viewport.width) * 0.4,
                    f32::from(viewport.height) * 0.4,
                ),
            };
            let colors = [0x286b8a, 0xb85543, 0x4c8858, 0x8664a0, 0xba8d37, 0x3b8381];
            (
                Bounds::new(point(px(x), px(y)), size(px(w), px(h))),
                rgb(colors[i % colors.len()]).alpha(options.opacity).into(),
            )
        })
        .collect()
}

fn make_scene(options: &Options) -> Scene {
    let viewport = size(px(options.width as f32), px(options.height as f32));
    let mask = Bounds::new(point(px(0.), px(0.)), viewport).scale(1.);
    let mut scene = Scene::default();
    scene.insert_primitive(Quad {
        bounds: mask,
        content_mask: gpui::ContentMask { bounds: mask },
        background: rgb(0xf1f0e8).into(),
        ..Default::default()
    });
    for (bounds, color) in workload(options, viewport) {
        scene.insert_primitive(Quad {
            bounds: bounds.scale(1.),
            content_mask: gpui::ContentMask { bounds: mask },
            background: color.into(),
            ..Default::default()
        });
    }
    scene
}

#[derive(Serialize)]
struct Distribution {
    median: f64,
    p95: f64,
    min: f64,
    max: f64,
}
fn distribution(mut samples: Vec<f64>) -> Distribution {
    samples.sort_by(f64::total_cmp);
    let len = samples.len();
    Distribution {
        median: (samples[(len - 1) / 2] + samples[len / 2]) / 2.0,
        p95: samples[(len * 95).div_ceil(100).saturating_sub(1)],
        min: samples[0],
        max: samples[len - 1],
    }
}
#[derive(Serialize)]
struct Report {
    schema_version: u32,
    app: &'static str,
    upstream_commit: String,
    os: &'static str,
    arch: &'static str,
    optimized_build: bool,
    renderer: &'static str,
    pattern: Pattern,
    requested_quads: usize,
    scene_quads_including_background: usize,
    batches: usize,
    opacity: f32,
    width: u32,
    height: u32,
    measured_frames: usize,
    warmup_frames: usize,
    compose_us: Distribution,
    finish_us: Distribution,
    batch_traversal_us: Distribution,
    render_and_readback_ms: Option<Distribution>,
    timing_scope: &'static str,
}
fn headless(options: &Options, cancel: &std::sync::atomic::AtomicBool) -> Result<()> {
    let mut renderer = if options.cpu_only {
        None
    } else {
        Some(
            gpui_platform::current_headless_renderer()?
                .context("No headless renderer for this target; use --cpu-only to measure composition")?,
        )
    };
    let mut compose = Vec::new();
    let mut finish = Vec::new();
    let mut batches_time = Vec::new();
    let mut rendering = Vec::new();
    let mut final_counts = (0, 0);
    let target = size(gpui::DevicePixels(options.width as i32), gpui::DevicePixels(options.height as i32));
    for frame in 0..options.warmup + options.frames {
        crate::benchmarks::check_cancel(cancel)?;
        let started = Instant::now();
        let mut scene = make_scene(options);
        let composed = started.elapsed().as_secs_f64() * 1e6;
        let started = Instant::now();
        scene.finish();
        let finished = started.elapsed().as_secs_f64() * 1e6;
        let started = Instant::now();
        let batches = std::hint::black_box(scene.batches().count());
        let batched = started.elapsed().as_secs_f64() * 1e6;
        final_counts = (scene.quads.len(), batches);
        if let Some(renderer) = renderer.as_mut() {
            let started = Instant::now();
            let image = renderer.render_scene_to_image(&scene, target)?;
            let elapsed = started.elapsed().as_secs_f64() * 1000.;
            ensure!(image.width() == options.width && image.height() == options.height, "Unexpected image dimensions");
            ensure!(image.pixels().any(|p| p.0[3] == 255), "Renderer returned no opaque pixels");
            let sample = match options.pattern {
                Pattern::Grid => image.get_pixel(0, 0),
                Pattern::Overlap => image.get_pixel(options.width / 2, options.height / 2),
            };
            ensure!(
                options.opacity < 0.1
                    || sample.0[..3]
                        .iter()
                        .zip([241u8, 240, 232])
                        .any(|(actual, background)| actual.abs_diff(background) > 2),
                "Renderer returned background where a workload quad was expected"
            );
            if frame >= options.warmup {
                rendering.push(elapsed);
            }
            if frame + 1 == options.warmup + options.frames
                && let Some(path) = &options.png
            {
                image.save(path).with_context(|| format!("Save {}", path.display()))?;
            }
        }
        if frame >= options.warmup {
            compose.push(composed);
            finish.push(finished);
            batches_time.push(batched);
        }
    }
    let provenance: serde_json::Value = serde_json::json!({"commit":"74b8e4e6c40bff94d12af94e71ce5014c7d8d49c"});
    let report = Report {
        schema_version: 1,
        app: "quad-bench",
        upstream_commit: provenance["commit"].as_str().unwrap_or("unknown").into(),
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        optimized_build: !cfg!(debug_assertions),
        renderer: if options.cpu_only {
            "none (CPU only)"
        } else if cfg!(target_os = "macos") {
            "Metal headless"
        } else {
            "platform headless"
        },
        pattern: options.pattern,
        requested_quads: options.count,
        scene_quads_including_background: final_counts.0,
        batches: final_counts.1,
        opacity: options.opacity,
        width: options.width,
        height: options.height,
        measured_frames: options.frames,
        warmup_frames: options.warmup,
        compose_us: distribution(compose),
        finish_us: distribution(finish),
        batch_traversal_us: distribution(batches_time),
        render_and_readback_ms: if rendering.is_empty() {
            None
        } else {
            Some(distribution(rendering))
        },
        timing_scope: "CPU composition includes geometry generation, allocation and Scene insertion; finish is sorting; batch traversal counts batches. Offscreen render/readback includes CPU encoding, target allocation, GPU completion and image copy, not isolated GPU time. No window presentation is measured.",
    };
    let json = serde_json::to_string_pretty(&report)?;
    if let Some(path) = &options.json {
        fs::write(path, &json)?;
    }

    Ok(())
}

pub(super) fn run(
    args: &[String],
    output: &std::path::Path,
    cancel: &std::sync::atomic::AtomicBool,
) -> Result<serde_json::Value> {
    let mut argv = vec!["embedded-benchmark".to_owned(), "--headless".to_owned()];
    argv.extend_from_slice(args);
    argv.extend(["--json".into(), output.join("report.json").to_string_lossy().into_owned()]);
    if !args.iter().any(|a| a == "--cpu-only") {
        argv.extend(["--png".into(), output.join("capture.png").to_string_lossy().into_owned()]);
    }
    let options = Options::try_parse_from(argv)?;
    options.validate()?;
    headless(&options, cancel)?;
    Ok(serde_json::from_slice(&std::fs::read(output.join("report.json"))?)?)
}

pub(super) fn preview(count: usize, overlap: bool, opacity: f32) -> impl gpui::IntoElement {
    let options = Options::try_parse_from([
        "preview",
        "--count",
        &count.to_string(),
        "--pattern",
        if overlap { "overlap" } else { "grid" },
        "--opacity",
        &opacity.to_string(),
    ])
    .expect("bounded preview options");
    gpui::canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            for (mut rect, color) in workload(&options, bounds.size) {
                rect.origin += bounds.origin;
                window.paint_quad(gpui::fill(rect, color));
            }
        },
    )
    .size_full()
}
