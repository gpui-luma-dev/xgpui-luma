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
    let base_size = policy.resolution(logical, 1., 1.)?.output;
    if let Some(p) = progress {
        p.stage("Rasterizing baseline / warm-up 1 of 2");
    }
    let base = document.rasterize(&renderer, base_size, budget)?;
    if let Some(p) = progress {
        p.stage(format!(
            "Rasterizing {}×{} px / warm-up 2 of 2 · {:.1} MiB{}",
            i32::from(resolution.output.width),
            i32::from(resolution.output.height),
            xgpui_svg::image_bytes(resolution.output) as f64 / MIB as f64,
            if resolution.limited() { " · capped" } else { "" }
        ));
    }
    let mut previous = document.rasterize(&renderer, resolution.output, budget)?;
    let first_raster = previous.ms;
    let mut parse = Vec::new();
    let mut raster = Vec::new();
    let mut sampled_rss_peak = rss_before;
    for sample in 0..5 {
        crate::benchmarks::check_cancel(cancel)?;
        if let Some(p) = progress {
            p.stage(format!("Parsing sample {} of 5", sample + 1));
        }
        let start = Instant::now();
        let fresh = SvgDocument::parse(&renderer, bytes)?;
        parse.push(start.elapsed().as_secs_f64() * 1000.);
        if let Some(p) = progress {
            p.stage(format!(
                "Rasterizing sample {} of 5 · {}×{} px",
                sample + 1,
                i32::from(resolution.output.width),
                i32::from(resolution.output.height)
            ));
        }
        let next = fresh.rasterize(&renderer, resolution.output, budget)?;
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
    ("AJ_Digital_Camera", include_bytes!("../../assets/samples/svg/AJ_Digital_Camera.svg").as_slice()),
    ("rg1024_green_grapes", include_bytes!("../../assets/samples/svg/rg1024_green_grapes.svg").as_slice()),
    ("rg1024_metal_effect", include_bytes!("../../assets/samples/svg/rg1024_metal_effect.svg").as_slice()),
    ("dragon", include_bytes!("../../assets/samples/svg/dragon.svg").as_slice()),
];
/// Materialize embedded repository fixtures for isolated case processes.
pub fn bundled_corpus(output: &Path, cancel: &AtomicBool) -> Result<Value> {
    let inputs = output.join("inputs");
    std::fs::create_dir_all(&inputs)?;
    for (id, bytes) in SAMPLES {
        crate::benchmarks::check_cancel(cancel)?;
        std::fs::write(inputs.join(format!("{id}.svg")), bytes)?;
    }
    corpus(&inputs, output, cancel)
}
fn verify_hash(bytes: &[u8], expected: &str) -> Result<()> {
    ensure!(format!("{:x}", Sha256::digest(bytes)) == expected, "corpus SHA-256 mismatch");
    Ok(())
}
pub fn corpus(folder: &Path, output: &Path, cancel: &AtomicBool) -> Result<Value> {
    let manifest: Vec<Value> = serde_json::from_str(MANIFEST)?;
    // Validate all supplied documents before starting any timed case.
    for item in &manifest {
        crate::benchmarks::check_cancel(cancel)?;
        let path = folder.join(format!("{}.svg", item["id"].as_str().context("manifest id")?));
        verify_hash(
            &read(&path).with_context(|| format!("Corpus file {}", path.display()))?,
            item["sha256"].as_str().context("manifest hash")?,
        )
        .with_context(|| format!("Corpus file {}", path.display()))?;
    }
    let mut cases = Vec::new();
    for item in manifest {
        let id = item["id"].as_str().context("manifest id")?;
        let source = folder.join(format!("{id}.svg"));
        for zoom in [1, 4, 8, 16] {
            crate::benchmarks::check_cancel(cancel)?;
            let directory = output.join(format!("{id}-{zoom}x"));
            std::fs::create_dir_all(&directory)?;
            let args = vec![
                "--svg-case".to_owned(),
                source.display().to_string(),
                "--zoom".into(),
                zoom.to_string(),
                "--output".into(),
                directory.display().to_string(),
            ];
            let result = crate::benchmarks::run_self(&args, &directory, cancel);
            cases.push(match result {
                Ok(mut report) => {
                    report["id"] = json!(id);
                    report["sha256"] = item["sha256"].clone();
                    report["source"] = item["url"].clone();
                    report
                }
                Err(error) => json!({"id": id, "zoom": zoom, "error": format!("{error:#}")}),
            });
            std::fs::write(output.join("raw.json"), serde_json::to_vec_pretty(&json!({"cases": cases}))?)?;
        }
    }
    let failures = cases.iter().filter(|case| case.get("error").is_some()).count();
    Ok(
        json!({"cases": cases, "errors": if failures == 0 { vec![] } else { vec![format!("{failures} corpus cases failed")] },
        "measurement": "Hash-verified source manifest, one fresh copy of this executable per document/zoom; five parse/raster repeats, zooms 1/4/8/16 at density 2, 256 MiB output budget. No runtime compilation or Python."}),
    )
}
/// Small reference images prepared outside benchmark timing and retained in memory.
pub(crate) fn thumbnails(selected: Option<&Path>, folder: Option<&Path>) -> crate::benchmarks::svg_table::Thumbnails {
    let renderer = SvgRenderer::new(Arc::new(()));
    let mut images = crate::benchmarks::svg_table::Thumbnails::new();
    let mut preview = |name: &str, bytes: Result<Vec<u8>>| {
        let image = bytes.and_then(|bytes| {
            let doc = SvgDocument::parse(&renderer, &bytes)?;
            let size = doc.fit_size(96.)?;
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
/// The complete corpus schedule, available before any workers start.
pub(crate) fn planned_cases() -> Vec<Value> {
    SAMPLES
        .iter()
        .flat_map(|(id, _)| [1, 4, 8, 16].map(|zoom| json!({"id":id,"job":"corpus","zoom":zoom,"outcome":"pending"})))
        .collect()
}
/// File-free interactive reports; corpus cases retain process isolation.
pub(crate) fn memory_selected(
    path: Option<&Path>,
    budget: usize,
    cancel: &AtomicBool,
    progress: &Progress,
) -> Result<Value> {
    crate::benchmarks::check_cancel(cancel)?;
    let bytes = match path {
        Some(path) => read(path)?,
        None => super::scene::svg(false, true).into_bytes(),
    };
    measure_report(&bytes, 4., budget, None, cancel, Some(progress))
}
pub(crate) fn memory_corpus(
    folder: Option<&Path>,
    budget: usize,
    cancel: &AtomicBool,
    progress: &Progress,
) -> Result<Value> {
    let manifest: Vec<Value> = serde_json::from_str(MANIFEST)?;
    progress.stage("Validating corpus hashes");
    let total = manifest.len() * 4;
    for item in &manifest {
        crate::benchmarks::check_cancel(cancel)?;
        let id = item["id"].as_str().context("manifest id")?;
        let bytes = if let Some(folder) = folder {
            read(&folder.join(format!("{id}.svg")))?
        } else {
            SAMPLES.iter().find(|(name, _)| *name == id).context("missing bundled SVG")?.1.to_vec()
        };
        verify_hash(&bytes, item["sha256"].as_str().context("manifest hash")?)?;
    }
    let mut cases = Vec::new();
    'cases: for item in manifest {
        let id = item["id"].as_str().context("manifest id")?;
        for zoom in [1, 4, 8, 16] {
            if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                break 'cases;
            }
            progress.case(format!("Case {} of {total} · {id} · {zoom}×", cases.len() + 1));
            progress.running(id, zoom as f64);
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
            let result = crate::benchmarks::memory::run(&args, cancel, progress);
            let mut report = result.unwrap_or_else(|e| json!({"error":format!("{e:#}")}));
            report["id"] = json!(id);
            report["zoom"] = json!(zoom);
            report["sha256"] = item["sha256"].clone();
            report["source"] = item["url"].clone();
            if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                report["outcome"] = json!("cancelled");
            }
            progress.completed(report.clone());
            cases.push(report);
        }
    }
    let failed = cases.iter().filter(|c| c.get("error").is_some()).count();
    Ok(
        json!({"cases":cases,"errors":if failed==0 {vec![]} else {vec![format!("{failed} corpus cases failed")]},
        "measurement":"Fresh process per artwork/zoom, five CPU parse/raster repeats at density 2 and the selected output budget. Reports travel over pipes; no temporary files or captures are written. RSS is process-wide; timings exclude PNG encoding and GPU work."}),
    )
}
pub fn run_case_cli(args: &[String]) -> Result<()> {
    let source = crate::benchmarks::argument(args, "--svg-case").context("SVG path required")?;
    let zoom = crate::benchmarks::argument(args, "--zoom").unwrap_or("4").parse()?;
    let bytes = if args.iter().any(|a| a == "--svg-bundled") {
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
        assert!(memory_corpus(None, 16 * MIB, &AtomicBool::new(true), &Progress::default()).is_err());
    }
    #[::core::prelude::v1::test]
    fn cancelled_svg_does_not_parse_or_write_a_preview() {
        let output = tempfile::tempdir().unwrap();
        assert!(builtin(output.path(), &AtomicBool::new(true)).is_err());
        assert!(!output.path().join("capture.png").exists());
    }
}
