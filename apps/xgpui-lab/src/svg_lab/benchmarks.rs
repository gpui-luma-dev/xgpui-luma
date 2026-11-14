//! Embedded CPU SVG benchmarks; corpus cases use fresh copies of this executable, not Python/source workers.
use anyhow::{Context as _, Result, ensure};
use gpui::{SvgRenderer, RenderImage};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    path::Path,
    sync::{Arc, atomic::AtomicBool},
    time::Instant,
};
use xgpui_svg::{SvgDocument, RasterPolicy, MIB};
use crate::benchmarks::progress::Progress;

// Original steady-policy matrix: apps/xgpui-ext-lab/svg-lab/bench/policy_run.py
// in xgpui 91b8629cf39cd52eca82e33a3835edee24454e8e (16/32 MiB; zoom 1/4/16).
pub(crate) const COMPARISON_BUDGETS: [usize; 2] = [16 * MIB, 32 * MIB];
/// Opt-in uncapped baseline: the worker's largest budget (about 67 Mpx). The 8192 px edge limit still applies.
pub(crate) const BASELINE_BUDGET: usize = 256 * MIB;
/// Zoom sweep for the in-app comparison, at display density 2.
pub(crate) const SWEEP_ZOOMS: [f32; 5] = [1., 2., 4., 8., 16.];
pub const MANIFEST: &str = include_str!("corpus.json");
fn stats(samples: &[f64]) -> Value {
    let mut sorted = samples.to_vec();
    sorted.sort_by(f64::total_cmp);
    json!({"samples_ms": samples, "median_ms": sorted[sorted.len()/2], "max_ms": sorted.last()})
}
fn rss() -> Option<u64> {
    let mut system = sysinfo::System::new();
    let pid = sysinfo::get_current_pid().ok()?;
    system.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::Some(&[pid]),
        sysinfo::ProcessRefreshKind::new().with_memory(),
    );
    system.process(pid).map(|process| process.memory())
}
fn read(path: &Path) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?.take((32 * MIB + 1) as u64).read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= 32 * MIB, "SVG exceeds 32 MiB input limit");
    Ok(bytes)
}
fn save(image: &RenderImage, output: &Path) -> Result<()> {
    let size = image.size(0);
    let mut pixels = image.as_bytes(0).context("missing SVG pixels")?.to_vec();
    for pixel in pixels.as_chunks_mut::<4>().0 {
        pixel.swap(0, 2);
    }
    image::save_buffer(output, &pixels, size.width.0 as u32, size.height.0 as u32, image::ColorType::Rgba8)?;
    Ok(())
}
pub fn measure(bytes: &[u8], zoom: f32, budget: usize, output: &Path, cancel: &AtomicBool) -> Result<Value> {
    measure_report(bytes, zoom, budget, Some(output), cancel, None)
}
fn measure_report(
    bytes: &[u8],
    zoom: f32,
    budget: usize,
    output: Option<&Path>,
    cancel: &AtomicBool,
    progress: Option<&Progress>,
) -> Result<Value> {
    ensure!(zoom.is_finite() && (0.25..=16.).contains(&zoom), "zoom must be 0.25..=16");
    crate::benchmarks::check_cancel(cancel)?;
    let rss_before = rss();
    let renderer = SvgRenderer::new(Arc::new(()));
    if let Some(p) = progress {
        p.stage("Parsing artwork / first use");
    }
    let start = Instant::now();
    let document = SvgDocument::parse(&renderer, bytes)?;
    let first_parse = start.elapsed().as_secs_f64() * 1000.;
    let logical = document.fit_size(320.)?;
    let policy = RasterPolicy { max_output_bytes: budget, ..Default::default() };
    let resolution = policy.resolution(logical, zoom, 2.)?;
    // Original policy benchmark: time parsing separately, then raster the retained tree.
    let mut parse = Vec::new();
    for sample in 0..5 {
        crate::benchmarks::check_cancel(cancel)?;
        if let Some(p) = progress {
            p.stage(format!("Parsing sample {} of 5", sample + 1));
        }
        let start = Instant::now();
        let fresh = SvgDocument::parse(&renderer, bytes)?;
        parse.push(start.elapsed().as_secs_f64() * 1000.);
        std::hint::black_box(fresh);
    }
    let base_size = policy.resolution(logical, 1., 1.)?.output;
    if let Some(p) = progress {
        p.stage("Rasterizing baseline / warm-up 1 of 2");
    }
    let base = document.rasterize(&renderer, base_size, budget)?;
    if let Some(p) = progress {
        p.stage(format!(
            "Rasterizing {}×{} px / warm-up 2 of 2 · {} MiB budget{}",
            i32::from(resolution.output.width),
            i32::from(resolution.output.height),
            budget / MIB,
            if resolution.limited() { " · capped" } else { "" }
        ));
    }
    let mut previous = document.rasterize(&renderer, resolution.output, budget)?;
    let first_raster = previous.ms;
    let mut raster = Vec::new();
    let mut sampled_rss_peak = rss_before;
    for sample in 0..5 {
        crate::benchmarks::check_cancel(cancel)?;
        if let Some(p) = progress {
            p.stage(format!(
                "Rasterizing sample {} of 5 · {}×{} px",
                sample + 1,
                i32::from(resolution.output.width),
                i32::from(resolution.output.height)
            ));
        }
        let next = document.rasterize(&renderer, resolution.output, budget)?;
        raster.push(next.ms);
        sampled_rss_peak = sampled_rss_peak.into_iter().chain(rss()).max();
        previous = next;
    }
    if let Some(p) = progress {
        p.stage("Finishing pixel coverage / report");
    }
    let pixels = previous.image.as_bytes(0).context("missing SVG pixels")?;
    let visible = pixels.as_chunks::<4>().0.iter().filter(|pixel| pixel[3] != 0).count();
    if let Some(output) = output {
        save(&previous.image, &output.join("capture.png"))?;
    }
    Ok(json!({"input_bytes": bytes.len(), "zoom": zoom, "density": 2, "budget_bytes": budget,
        "parse_first_ms": first_parse, "parse": stats(&parse), "raster_first_ms": first_raster, "raster": stats(&raster),
        "pixel_width": resolution.output.width.0, "pixel_height": resolution.output.height.0, "resolution_limited": resolution.limited(),
        "requested_width": resolution.requested.width.0, "requested_height": resolution.requested.height.0,
        "visible_pixels": visible, "output_bytes": pixels.len(), "base_bytes": base.image.as_bytes(0).context("base pixels")?.len(),
        "rss_start_bytes": rss_before, "sampled_process_rss_peak_bytes": sampled_rss_peak,
        "measurement": "CPU parse/raster samples; base and previous zoom retained during replacement. PNG encoding excluded. RSS covers the whole process; not GPU memory or per-document allocation.", "errors": []}))
}
pub fn builtin(output: &Path, cancel: &AtomicBool) -> Result<Value> {
    measure(super::scene::svg(false, true).as_bytes(), 4., 16 * MIB, output, cancel)
}
pub fn selected(path: &Path, output: &Path, cancel: &AtomicBool) -> Result<Value> {
    measure(&read(path)?, 4., 16 * MIB, output, cancel)
}
const SAMPLES: &[(&str, &[u8])] = &[
    ("camera", include_bytes!("../../assets/samples/svg/camera.svg").as_slice()),
    ("grapes", include_bytes!("../../assets/samples/svg/grapes.svg").as_slice()),
    ("metal", include_bytes!("../../assets/samples/svg/metal.svg").as_slice()),
    ("dragon", include_bytes!("../../assets/samples/svg/dragon.svg").as_slice()),
];
fn verify_hash(bytes: &[u8], expected: &str) -> Result<()> {
    ensure!(format!("{:x}", Sha256::digest(bytes)) == expected, "corpus SHA-256 mismatch");
    Ok(())
}
/// Reference previews prepared outside benchmark timing and retained in memory.
pub(crate) fn thumbnails(selected: Option<&Path>, folder: Option<&Path>) -> crate::benchmarks::svg_table::Thumbnails {
    let renderer = SvgRenderer::new(Arc::new(()));
    let mut images = crate::benchmarks::svg_table::Thumbnails::new();
    let mut preview = |name: &str, bytes: Result<Vec<u8>>| {
        let image = bytes.and_then(|bytes| {
            let doc = SvgDocument::parse(&renderer, &bytes)?;
            let size = doc.fit_size(360.)?;
            let resolution = RasterPolicy::default().resolution(size, 1., 1.)?;
            Ok(doc.rasterize(&renderer, resolution.output, MIB)?.image)
        });
        if let Ok(image) = image {
            images.insert(name.into(), image);
        }
    };
    preview("Built-in SVG", Ok(super::scene::svg(false, true).into_bytes()));
    if let Some(path) = selected {
        preview("Selected SVG", read(path));
    }
    for (id, bytes) in SAMPLES {
        preview(id, folder.map_or_else(|| Ok(bytes.to_vec()), |folder| read(&folder.join(format!("{id}.svg")))));
    }
    images
}
/// One pending row per zoom and budget for one artwork, in run order.
pub(crate) fn planned_rows(base: Value, zooms: &[f32], budgets: &[usize]) -> Vec<Value> {
    zooms
        .iter()
        .flat_map(|zoom| budgets.iter().map(move |budget| (zoom, budget)))
        .map(|(zoom, budget)| {
            let mut row = base.clone();
            row["zoom"] = json!(zoom);
            row["budget_bytes"] = json!(budget);
            row["outcome"] = json!("pending");
            row
        })
        .collect()
}
/// The complete corpus schedule, available before any workers start: artwork, then zoom, then budget.
pub(crate) fn planned_cases(zooms: &[f32], budgets: &[usize]) -> Vec<Value> {
    let manifest: Vec<Value> = serde_json::from_str(MANIFEST).unwrap_or_default();
    SAMPLES
        .iter()
        .flat_map(|(id, bytes)| {
            let coverage = manifest.iter().find(|item| item["id"] == *id).and_then(|item| item["coverage"].as_str());
            planned_rows(json!({"id":id,"job":"corpus","input_bytes":bytes.len(),"coverage":coverage}), zooms, budgets)
        })
        .collect()
}
/// File-free interactive reports; corpus cases retain process isolation.
pub(crate) fn isolated_selected(
    path: Option<&Path>,
    budget: usize,
    zoom: f32,
    cancel: &AtomicBool,
    progress: &Progress,
) -> Result<Value> {
    let mut args = vec![
        "--svg-case".into(),
        path.map_or_else(|| "builtin".into(), |p| p.display().to_string()),
        "--zoom".into(),
        zoom.to_string(),
        "--svg-budget-mib".into(),
        (budget / MIB).to_string(),
        "--svg-report-stdout".into(),
    ];
    if path.is_none() {
        args.push("--svg-builtin".into());
    }
    crate::benchmarks::memory::run(&args, cancel, progress)
}
pub(crate) fn memory_corpus(
    folder: Option<&Path>,
    zooms: &[f32],
    budgets: &[usize],
    cancel: &AtomicBool,
    progress: &Progress,
) -> Result<Value> {
    let manifest: Vec<Value> = serde_json::from_str(MANIFEST)?;
    progress.stage("Validating corpus hashes");
    let total = manifest.len() * zooms.len() * budgets.len();
    let mut sizes = std::collections::HashMap::new();
    for item in &manifest {
        crate::benchmarks::check_cancel(cancel)?;
        let id = item["id"].as_str().context("manifest id")?;
        let bytes = if let Some(folder) = folder {
            read(&folder.join(format!("{id}.svg")))?
        } else {
            SAMPLES.iter().find(|(name, _)| *name == id).context("missing bundled SVG")?.1.to_vec()
        };
        verify_hash(&bytes, item["sha256"].as_str().context("manifest hash")?)?;
        sizes.insert(id.to_owned(), bytes.len());
    }
    let tasks: Vec<_> = manifest
        .into_iter()
        .flat_map(|item| {
            zooms.iter().flat_map(move |zoom| {
                let item = item.clone();
                budgets.iter().map(move |budget| (item.clone(), *zoom, *budget))
            })
        })
        .collect();
    let mut cases = Vec::new();
    for (index, (item, zoom, budget)) in tasks.iter().enumerate() {
        let zoom = *zoom;
        if cancel.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }
        let id = item["id"].as_str().unwrap_or("SVG");
        let worker_progress = progress;
        worker_progress.case(format!("Case {} of {total} · {id} · {zoom}× · {} MiB", index + 1, budget / MIB));
        worker_progress.running_budget(id, zoom as f64, *budget);
        let mut args = vec![
            "--svg-budget-mib".into(),
            (budget / MIB).to_string(),
            "--svg-case".into(),
            folder.map_or_else(|| id.to_owned(), |f| f.join(format!("{id}.svg")).display().to_string()),
            "--zoom".into(),
            zoom.to_string(),
            "--svg-report-stdout".into(),
        ];
        if folder.is_none() {
            args.push("--svg-bundled".into());
        }
        let result = crate::benchmarks::memory::run(&args, cancel, worker_progress);
        let mut report = result.unwrap_or_else(|e| json!({"error":format!("{e:#}")}));
        report["id"] = json!(id);
        report["zoom"] = json!(zoom);
        report["budget_bytes"] = json!(budget);
        report["sha256"] = item["sha256"].clone();
        report["source"] = item["url"].clone();
        report["coverage"] = item["coverage"].clone();
        if let Some(bytes) = sizes.get(id) {
            report["input_bytes"] = json!(bytes);
        }
        report["execution"] = json!("sequential");
        report["workers"] = json!(1);
        if cancel.load(std::sync::atomic::Ordering::Relaxed) {
            report["outcome"] = json!("cancelled");
        }
        worker_progress.completed(report.clone());
        cases.push(report);
    }
    let failed = cases.iter().filter(|c| c.get("error").is_some() && c["outcome"] != "cancelled").count();
    Ok(
        json!({"cases":cases,"workers":1,"execution":"sequential","errors":if failed==0 {vec![]} else {vec![format!("{failed} corpus cases failed")]},
        "measurement":"Fresh process per artwork, zoom and budget; five CPU parse/raster repeats at density 2. Reports travel over pipes; no temporary files or captures are written. RSS is process-wide; timings exclude PNG encoding and GPU work."}),
    )
}
pub fn run_case_cli(args: &[String]) -> Result<()> {
    let source = crate::benchmarks::argument(args, "--svg-case").context("SVG path required")?;
    let zoom = crate::benchmarks::argument(args, "--zoom").unwrap_or("4").parse()?;
    let bytes = if args.iter().any(|a| a == "--svg-builtin") {
        super::scene::svg(false, true).into_bytes()
    } else if args.iter().any(|a| a == "--svg-bundled") {
        SAMPLES.iter().find(|(id, _)| *id == source).context("unknown bundled SVG")?.1.to_vec()
    } else {
        read(Path::new(source))?
    };
    if args.iter().any(|a| a == "--svg-report-stdout") {
        let budget: usize = crate::benchmarks::argument(args, "--svg-budget-mib").unwrap_or("256").parse()?;
        ensure!((1..=256).contains(&budget), "SVG budget must be 1..=256 MiB");
        let report =
            measure_report(&bytes, zoom, budget * MIB, None, &AtomicBool::new(false), Some(&Progress::worker()))?;
        use std::io::Write;
        serde_json::to_writer(std::io::stdout().lock(), &report)?;
        std::io::stdout().flush()?;
    } else {
        let output = Path::new(crate::benchmarks::argument(args, "--output").context("output directory required")?);
        std::fs::create_dir_all(output)?;
        let report = measure(&bytes, zoom, 256 * MIB, output, &AtomicBool::new(false))?;
        std::fs::write(output.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[::core::prelude::v1::test]
    fn bundled_samples_match_the_manifest() {
        let manifest: Vec<Value> = serde_json::from_str(MANIFEST).unwrap();
        assert_eq!(manifest.len(), SAMPLES.len());
        for (id, bytes) in SAMPLES {
            let entry = manifest.iter().find(|item| item["id"] == *id).unwrap();
            verify_hash(bytes, entry["sha256"].as_str().unwrap()).unwrap();
            assert!(entry["local"].as_str().unwrap().starts_with("assets/samples/"));
        }
    }
    #[::core::prelude::v1::test]
    fn corpus_hash_rejects_changed_bytes() {
        let hash = format!("{:x}", Sha256::digest(b"fixture"));
        assert!(verify_hash(b"fixture", &hash).is_ok());
        assert!(verify_hash(b"changed", &hash).is_err());
    }
    #[::core::prelude::v1::test]
    fn cancelled_memory_corpus_does_not_start_workers_or_materialize_inputs() {
        assert!(memory_corpus(None, &[4.], &COMPARISON_BUDGETS, &AtomicBool::new(true), &Progress::default()).is_err());
    }
    #[::core::prelude::v1::test]
    fn cancelled_svg_does_not_parse_or_write_a_preview() {
        let output = tempfile::tempdir().unwrap();
        assert!(builtin(output.path(), &AtomicBool::new(true)).is_err());
        assert!(!output.path().join("capture.png").exists());
    }
}

/// One-line artwork credit for the benchmark page, from the manifest: "camera · AJ Ashton (public domain); …".
pub(crate) fn artwork_credits() -> String {
    let manifest: Vec<Value> = serde_json::from_str(MANIFEST).unwrap_or_default();
    let credits: Vec<String> = manifest
        .iter()
        .filter_map(|item| {
            let license = match item["license_url"].as_str()? {
                url if url.contains("publicdomain/zero") => "CC0 1.0",
                url if url.contains("by-sa/3.0") => "CC BY-SA 3.0",
                url if url.contains("PublicDomain") => "public domain",
                _ => item["license"].as_str()?,
            };
            Some(format!("{} · {} ({license})", item["id"].as_str()?, item["author"].as_str()?))
        })
        .collect();
    format!("Artwork: {}. Unmodified; sources and licenses in assets/samples/README.md.", credits.join("; "))
}

#[cfg(test)]
mod credit_tests {
    #[::core::prelude::v1::test]
    fn every_bundled_artwork_is_credited_with_its_license() {
        let credits = super::artwork_credits();
        assert!(credits.contains("camera · AJ Ashton (public domain)"), "{credits}");
        assert!(credits.contains("grapes · rg1024 (CC0 1.0)"), "{credits}");
        assert!(credits.contains("dragon · Ebaychatter0 (CC BY-SA 3.0)"), "{credits}");
    }
}
