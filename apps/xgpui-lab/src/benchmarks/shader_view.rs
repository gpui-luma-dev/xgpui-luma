//! Shader suite reports: paint coverage, mesh, WGSL, Grid and Seascape.
use super::report_view::{CaptureGroup, Comparison, Metric, ReportCard, Timing, preview};
use super::*;

pub(super) fn card(look: &Look, report: &Value) -> AnyElement {
    let card = ReportCard::new(look, report);
    match report["job"].as_str() {
        Some("grid") => grid(card, report),
        Some("seascape") => seascape(card, report),
        _ => paint(card, report),
    }
    .into_element()
}

fn count(value: &Value) -> String {
    value.as_u64().map_or_else(|| "—".into(), |n| n.to_string())
}
fn captures(report: &Value, files: &[(&str, &str)]) -> Vec<(SharedString, PathBuf)> {
    files
        .iter()
        .filter_map(|(caption, file)| preview(report, file).map(|path| (SharedString::from(*caption), path)))
        .collect()
}
/// `render_readback_timings` rows, labelled by API.
fn api_timings(report: &Value) -> Vec<Timing> {
    report["render_readback_timings"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| {
            let label = if row["api"] == "instanced" {
                "Instanced"
            } else {
                "Shader paint"
            };
            Timing::from_summary(label, row)
        })
        .collect()
}

fn paint<'a>(card: ReportCard<'a>, report: &Value) -> ReportCard<'a> {
    let samples: Vec<f64> =
        report["render_readback_ms"].as_array().into_iter().flatten().filter_map(Value::as_f64).collect();
    card.tile("Source preparations", count(&report["source_generations"]))
        .tile("Pipeline compilations", count(&report["pipeline_compilations"]))
        .timings("Render + readback · 1024×320", Timing::from_samples("Demo scene", &samples))
        .captures(CaptureGroup {
            title: "Capture".into(),
            size: size(px(512.), px(160.)),
            captures: captures(report, &[("Plain · dashed border · path coverage", "capture.png")]),
        })
}

fn grid<'a>(card: ReportCard<'a>, report: &Value) -> ReportCard<'a> {
    let thumb = size(px(264.), px(168.));
    card.tile("Pixel tolerance", format!("≤ {}/255", count(&report["pixel_tolerance"])))
        .tile("Native / wgpu parity", report["native_wgpu_parity"].as_str().unwrap_or("—").to_owned())
        .timings("Render + readback · 176×112", api_timings(report))
        .captures(CaptureGroup {
            title: "Native".into(),
            size: thumb,
            captures: captures(
                report,
                &[("Shader paint", "grid-shader-paint.png"), ("Instanced", "grid-instanced.png")],
            ),
        })
        .captures(CaptureGroup {
            title: "wgpu".into(),
            size: thumb,
            captures: captures(
                report,
                &[("Shader paint", "grid-wgpu-shader-paint.png"), ("Instanced", "grid-wgpu-instanced.png")],
            ),
        })
}

const SEA_CASES: [&str; 3] = [
    "Case 0 · 320×180 · defaults",
    "Case 1 · 192×108 · time 2.5",
    "Case 2 · 160×160 · rough sea, no detail",
];

fn seascape<'a>(card: ReportCard<'a>, report: &Value) -> ReportCard<'a> {
    let limits = &report["similarity_limits"];
    let comparisons = report["comparisons"].as_array().into_iter().flatten().map(|row| Comparison {
        label: row["comparison"].as_str().unwrap_or("comparison").to_owned().into(),
        metrics: [
            ("MAE", "mean_absolute_rgb_error", limits["mean_absolute_rgb_error"].as_f64()),
            (">24/255", "pixels_over_24_fraction", limits["pixels_over_24_fraction"].as_f64()),
            ("max", "max_channel_error", None),
        ]
        .into_iter()
        .filter_map(|(name, key, limit)| row[key].as_f64().map(|value| Metric { name: name.into(), value, limit }))
        .collect(),
    });
    let mut card = card
        .tile("Native / wgpu parity", report["native_wgpu_parity"].as_str().unwrap_or("—").to_owned())
        .timings("Render + readback · 320×180", api_timings(report))
        .comparisons(comparisons.collect::<Vec<_>>());
    for (case, title) in SEA_CASES.into_iter().enumerate() {
        let files = [
            ("Shader paint", format!("sea-{case}-shader-paint.png")),
            ("Instanced", format!("sea-{case}-instanced.png")),
            ("wgpu shader paint", format!("sea-{case}-wgpu-shader-paint.png")),
            ("wgpu instanced", format!("sea-{case}-wgpu-instanced.png")),
        ];
        let files: Vec<(&str, &str)> = files.iter().map(|(caption, file)| (*caption, file.as_str())).collect();
        card = card.captures(CaptureGroup {
            title: title.into(),
            size: size(px(240.), px(160.)),
            captures: captures(report, &files),
        });
    }
    card
}

#[cfg(test)]
mod tests {
    use super::*;
    #[::core::prelude::v1::test]
    fn grid_timings_keep_api_order_and_p95() {
        let report = json!({"render_readback_timings": [
            {"api":"shader_paint","frames":120,"median_ms":1.0,"p95_ms":1.5,"max_ms":2.0},
            {"api":"instanced","frames":120,"median_ms":0.8,"p95_ms":1.1,"max_ms":1.9},
        ]});
        let timings = api_timings(&report);
        assert_eq!(timings.iter().map(|t| t.label.as_ref()).collect::<Vec<_>>(), ["Shader paint", "Instanced"]);
        assert_eq!(timings[0].p95, Some(1.5));
    }
    #[::core::prelude::v1::test]
    fn missing_captures_are_left_out() {
        let report = json!({"previews": ["/out/grid-instanced.png"]});
        let found =
            captures(&report, &[("Shader paint", "grid-shader-paint.png"), ("Instanced", "grid-instanced.png")]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0.as_ref(), "Instanced");
    }
}
