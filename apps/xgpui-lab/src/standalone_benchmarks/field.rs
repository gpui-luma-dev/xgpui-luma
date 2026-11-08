#[path = "workload.rs"]
mod workload;

use anyhow::{Context as _, Result, ensure};
use clap::Parser;
use gpui::{Bounds, DevicePixels, Pixels, Quad, Scene, Size, point, px, size};
use serde::Serialize;
use std::{collections::BTreeMap, fs, path::PathBuf, time::Instant};
use workload::Method;

const SOURCE: &str =
    "gpui-luma/crates/luma-color/src/color_field/field/paint_vector.rs @ b5a4a23ce6ed98731c1e3aef0abec02da4879274";

#[derive(Clone, Parser)]
#[command(
    about = "SV color-field quad rendering, adapted from gpui-luma-color",
    args_override_self = true
)]
#[allow(dead_code)] // Original validated CLI fields; windowed launch options are not used by the embedded runner.
struct Options {
    #[arg(long)]
    headless: bool,
    #[arg(long, conflicts_with_all = ["headless", "json"])]
    continuous: bool,
    #[arg(long, requires = "headless", conflicts_with = "png")]
    cpu_only: bool,
    #[arg(long, value_enum, default_value_t = Method::Layered)]
    method: Method,
    /// Sampling target along the longest axis; the source algorithm enforces a minimum 0.25 scale.
    #[arg(long, default_value_t = 128)]
    samples: usize,
    #[arg(long, default_value_t = 210.)]
    hue: f32,
    /// Advance hue by one degree per rendered frame, including warmup.
    #[arg(long)]
    animate: bool,
    #[arg(long, default_value_t = 512)]
    width: u32,
    #[arg(long, default_value_t = 384)]
    height: u32,
    #[arg(long, default_value_t = 600)]
    frames: usize,
    #[arg(long, default_value_t = 120)]
    warmup: usize,
    #[arg(long)]
    json: Option<PathBuf>,
    #[arg(long, requires = "headless")]
    png: Option<PathBuf>,
    #[arg(long, default_value_t = 0, value_parser = clap::value_parser!(u64).range(0..=3600), conflicts_with_all = ["headless", "continuous"])]
    exit_delay_secs: u64,
    /// Fail a bounded UI run if frame delivery stops; zero disables the watchdog.
    #[arg(long, default_value_t = 60, value_parser = clap::value_parser!(u64).range(0..=3600))]
    timeout_secs: u64,
}
impl Options {
    fn validate(&self) -> Result<()> {
        ensure!(
            (64..=1024).contains(&self.width) && (64..=1024).contains(&self.height),
            "width and height must be 64..=1024"
        );
        ensure!((16..=512).contains(&self.samples), "samples must be 16..=512");
        ensure!(self.hue.is_finite() && (0. ..360.).contains(&self.hue), "hue must be finite and in [0, 360)");
        ensure!(
            (1..=10000).contains(&self.frames) && self.warmup <= 1000,
            "frames must be 1..=10000; warmup at most 1000"
        );
        Ok(())
    }
    fn hue_at(&self, frame: usize) -> f32 {
        (self.hue + if self.animate { (frame % 360) as f32 } else { 0. }).rem_euclid(360.)
    }
}

#[derive(Serialize)]
struct Distribution {
    median: f64,
    p95: f64,
    min: f64,
    max: f64,
}
fn distribution(mut values: Vec<f64>) -> Option<Distribution> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    let n = values.len();
    Some(Distribution {
        median: (values[(n - 1) / 2] + values[n / 2]) / 2.,
        p95: values[(n * 95).div_ceil(100) - 1],
        min: values[0],
        max: values[n - 1],
    })
}
#[derive(Default)]
struct Samples {
    generate_us: Vec<f64>,
    compose_us: Vec<f64>,
    finish_us: Vec<f64>,
    render_readback_ms: Vec<f64>,
    cpu_paint_ms: Vec<f64>,
    callback_interval_ms: Vec<f64>,
}
#[derive(Serialize)]
struct Report {
    schema_version: u32,
    app: &'static str,
    workload_revision: u32,
    source_evidence: &'static str,
    upstream_commit: String,
    mode: &'static str,
    renderer: &'static str,
    optimized_build: bool,
    os: &'static str,
    arch: &'static str,
    method: Method,
    samples_target: usize,
    sampling_grid: Option<[u32; 2]>,
    quads_per_frame: usize,
    primitive_batches: Option<usize>,
    width: f32,
    height: f32,
    scale_factor: f32,
    measured_frames: usize,
    warmup_frames: usize,
    animated: bool,
    first_measured_hue: f32,
    last_measured_hue: f32,
    focused_frames: Option<usize>,
    unfocused_frames: Option<usize>,
    inactive_throttling_disabled: bool,
    metal_hud_enabled: bool,
    callback_fps: Option<f64>,
    timings: BTreeMap<&'static str, Option<Distribution>>,
    timing_scope: &'static str,
}
fn report(
    options: &Options,
    samples: Samples,
    viewport: Size<Pixels>,
    scale: f32,
    quad_count: usize,
    batches: Option<usize>,
    focus: Option<usize>,
) -> Report {
    let width = f32::from(viewport.width);
    let height = f32::from(viewport.height);
    let (cols, rows) = workload::sampling(width, height, options.samples);
    let fps = if samples.callback_interval_ms.is_empty() {
        None
    } else {
        Some(samples.callback_interval_ms.len() as f64 * 1000. / samples.callback_interval_ms.iter().sum::<f64>())
    };
    let provenance: serde_json::Value = serde_json::json!({"commit":"74b8e4e6c40bff94d12af94e71ce5014c7d8d49c"});
    Report {
        schema_version: 1,
        app: "color-field-bench",
        workload_revision: 1,
        source_evidence: SOURCE,
        upstream_commit: provenance["commit"].as_str().unwrap_or("unknown").into(),
        mode: if options.headless {
            "headless"
        } else {
            "windowed-callback-cadence"
        },
        renderer: if options.cpu_only {
            "none (CPU only)"
        } else if cfg!(target_os = "macos") {
            "Metal"
        } else {
            "platform renderer"
        },
        optimized_build: !cfg!(debug_assertions),
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        method: options.method,
        samples_target: options.samples,
        sampling_grid: (options.method == Method::Sampled).then_some([cols, rows]),
        quads_per_frame: quad_count,
        primitive_batches: batches,
        width,
        height,
        scale_factor: scale,
        measured_frames: options.frames,
        warmup_frames: options.warmup,
        animated: options.animate,
        first_measured_hue: options.hue_at(options.warmup),
        last_measured_hue: options.hue_at(options.warmup + options.frames - 1),
        focused_frames: focus,
        unfocused_frames: focus.map(|f| options.frames - f),
        inactive_throttling_disabled: !options.headless,
        metal_hud_enabled: std::env::var("MTL_HUD_ENABLED").is_ok_and(|v| v == "1"),
        callback_fps: fps,
        timings: BTreeMap::from([
            ("generate_us", distribution(samples.generate_us)),
            ("compose_us", distribution(samples.compose_us)),
            ("finish_us", distribution(samples.finish_us)),
            ("render_readback_ms", distribution(samples.render_readback_ms)),
            ("cpu_paint_ms", distribution(samples.cpu_paint_ms)),
            ("callback_interval_ms", distribution(samples.callback_interval_ms)),
        ]),
        timing_scope: "generate_us includes allocation, color calculation and PaintQuad generation. Headless compose_us converts/inserts generated quads into Scene; finish_us sorts it. render_readback_ms includes target allocation, CPU encoding, GPU completion and image copy, not isolated GPU time. Image checks and file writes are outside measured intervals. UI cpu_paint_ms is Window::paint_quad enqueue time, excluding generation; callback FPS is scheduling cadence, not confirmed presented FPS. UI uses logical pixels/native scale; headless uses physical pixels at scale 1. No control layout, image caching or wide-gamut output is measured.",
    }
}
fn save(options: &Options, report: &Report) -> Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    if let Some(path) = &options.json {
        fs::write(path, &json).with_context(|| format!("Save {}", path.display()))?;
    }

    Ok(())
}
fn scene(quads: Vec<gpui::PaintQuad>, viewport: Size<Pixels>) -> Scene {
    let mask = gpui::ContentMask { bounds: Bounds::new(point(px(0.), px(0.)), viewport).scale(1.) };
    let mut scene = Scene::default();
    for q in quads {
        scene.insert_primitive(Quad {
            bounds: q.bounds.scale(1.),
            content_mask: mask,
            background: q.background,
            corner_radii: q.corner_radii.scale(1.),
            border_widths: q.border_widths.scale(1.),
            border_color: q.border_color,
            border_style: q.border_style,
            ..Default::default()
        });
    }
    scene
}
fn headless(options: &Options, cancel: &std::sync::atomic::AtomicBool) -> Result<()> {
    let mut renderer = if options.cpu_only {
        None
    } else {
        Some(
            gpui_platform::current_headless_renderer()?
                .context("No headless renderer for this target; use --cpu-only")?,
        )
    };
    let viewport = size(px(options.width as f32), px(options.height as f32));
    let bounds = Bounds::new(point(px(0.), px(0.)), viewport);
    let mut samples = Samples::default();
    let mut counts = (0, 0);
    for frame in 0..options.warmup + options.frames {
        crate::benchmarks::check_cancel(cancel)?;
        let hue = options.hue_at(frame);
        let start = Instant::now();
        let quads = workload::quads(options.method, bounds, hue, options.samples);
        let generated = start.elapsed().as_secs_f64() * 1e6;
        let start = Instant::now();
        let mut scene = scene(quads, viewport);
        let composed = start.elapsed().as_secs_f64() * 1e6;
        let start = Instant::now();
        scene.finish();
        let finished = start.elapsed().as_secs_f64() * 1e6;
        counts = (scene.quads.len(), std::hint::black_box(scene.batches().count()));
        if let Some(renderer) = renderer.as_mut() {
            let start = Instant::now();
            let image = renderer.render_scene_to_image(
                &scene,
                size(DevicePixels(options.width as i32), DevicePixels(options.height as i32)),
            )?;
            let elapsed = start.elapsed().as_secs_f64() * 1000.;
            ensure!(image.width() == options.width && image.height() == options.height, "Unexpected image dimensions");
            // Check nine interior points against the analytical SV plane. The
            // fallback samples cell midpoints; allow its bounded sampling error
            // plus 8-bit quantization/dithering. This is not gamut validation.
            let (cols, rows) = workload::sampling(options.width as f32, options.height as f32, options.samples);
            let tolerance = if options.method == Method::Layered {
                6.
            } else {
                255. * (1. / cols as f32 + 1. / rows as f32) + 6.
            };
            for u in [0.1, 0.5, 0.9] {
                for v in [0.1, 0.5, 0.9] {
                    let x = (u * options.width as f32) as u32;
                    let y = (v * options.height as f32) as u32;
                    let expected = workload::hsv(
                        hue,
                        (x as f32 + 0.5) / options.width as f32,
                        1. - (y as f32 + 0.5) / options.height as f32,
                    );
                    let pixel = image.get_pixel(x, y).0;
                    ensure!(pixel[3] == 255, "Non-opaque field at ({x},{y})");
                    for (actual, expected) in pixel[..3].iter().zip([expected.r, expected.g, expected.b]) {
                        ensure!(
                            (*actual as f32 - expected * 255.).abs() <= tolerance,
                            "SV color mismatch at ({x},{y}): actual {actual}, expected {}, tolerance {tolerance}",
                            expected * 255.
                        );
                    }
                }
            }
            if frame >= options.warmup {
                samples.render_readback_ms.push(elapsed);
            }
            if frame + 1 == options.warmup + options.frames
                && let Some(path) = &options.png
            {
                image.save(path).with_context(|| format!("Save {}", path.display()))?;
            }
        }
        if frame >= options.warmup {
            samples.generate_us.push(generated);
            samples.compose_us.push(composed);
            samples.finish_us.push(finished);
        }
    }
    save(options, &report(options, samples, viewport, 1., counts.0, Some(counts.1), None))
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

pub(super) fn preview(sampled: bool, samples: usize, hue: f32) -> impl gpui::IntoElement {
    use gpui::prelude::*;
    gpui::canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            for quad in workload::quads(if sampled { Method::Sampled } else { Method::Layered }, bounds, hue, samples) {
                window.paint_quad(quad);
            }
        },
    )
    .size_full()
}
