//! Typed view of SVG benchmark rows, shared by the cards, charts, table and findings.
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Skipped,
}
impl Outcome {
    fn of(case: &Value) -> Self {
        match case["outcome"].as_str() {
            Some("pending") => Self::Pending,
            Some("running") => Self::Running,
            Some("failed") => Self::Failed,
            Some("cancelled") => Self::Cancelled,
            Some("skipped") => Self::Skipped,
            Some(_) => Self::Completed,
            None if case.get("error").is_some() => Self::Failed,
            None => Self::Completed,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "done",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Skipped => "skipped",
        }
    }
    /// The case has finished one way or another.
    pub fn settled(self) -> bool {
        !matches!(self, Self::Pending | Self::Running)
    }
}

/// One finished measurement. Pending and failed cases have none, so they never plot as zero.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Measure {
    pub parse_ms: f64,
    pub raster_ms: f64,
    pub raster_max_ms: Option<f64>,
    pub width: u64,
    pub height: u64,
    /// Output size before the budget cap, when the worker reported it.
    pub requested: Option<(u64, u64)>,
    pub capped: bool,
    pub rss_mib: Option<f64>,
}
impl Measure {
    pub fn megapixels(&self) -> f64 {
        (self.width * self.height) as f64 / 1e6
    }
    pub fn ms_per_megapixel(&self) -> Option<f64> {
        let mp = self.megapixels();
        (mp > 0.).then(|| self.raster_ms / mp)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Case {
    /// Source id; also the thumbnail key.
    pub id: String,
    pub name: String,
    pub coverage: Option<String>,
    pub input_bytes: Option<u64>,
    pub zoom: f64,
    pub budget_mib: u64,
    pub outcome: Outcome,
    pub error: Option<String>,
    pub measure: Option<Measure>,
}

fn finite(value: &Value) -> Option<f64> {
    value.as_f64().filter(|v| v.is_finite() && *v >= 0.)
}

fn measure(case: &Value) -> Option<Measure> {
    if case.get("error").is_some() {
        return None;
    }
    let parse_ms = finite(&case["parse"]["median_ms"]).or_else(|| finite(&case["parse_ms"]))?;
    let raster_ms = finite(&case["raster"]["median_ms"]).or_else(|| finite(&case["raster_ms"]))?;
    let requested = case["requested_width"].as_u64().zip(case["requested_height"].as_u64());
    Some(Measure {
        parse_ms,
        raster_ms,
        raster_max_ms: finite(&case["raster"]["max_ms"]),
        width: case["pixel_width"].as_u64()?,
        height: case["pixel_height"].as_u64()?,
        requested,
        capped: case["resolution_limited"].as_bool().unwrap_or(false),
        rss_mib: finite(&case["sampled_process_rss_peak_bytes"]).map(|bytes| bytes / 1048576.),
    })
}

pub(crate) fn case(value: &Value) -> Option<Case> {
    let id = value["id"].as_str()?.to_owned();
    let outcome = Outcome::of(value);
    Some(Case {
        name: value["display_name"].as_str().unwrap_or(&id).to_owned(),
        id,
        coverage: value["coverage"].as_str().map(str::to_owned),
        input_bytes: value["input_bytes"].as_u64(),
        zoom: value["zoom"].as_f64().unwrap_or(f64::NAN),
        budget_mib: value["budget_bytes"].as_u64().unwrap_or(0) / xgpui_svg::MIB as u64,
        outcome,
        error: value["error"].as_str().map(str::to_owned),
        measure: (outcome == Outcome::Completed).then(|| measure(value)).flatten(),
    })
}

pub(crate) fn cases(values: &[Value]) -> Vec<Case> {
    values.iter().filter_map(case).collect()
}

/// Everything a card needs for one artwork.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Artwork {
    pub id: String,
    pub name: String,
    pub coverage: Option<String>,
    pub input_bytes: Option<u64>,
    pub parse_ms: Option<f64>,
    /// Raster ms per megapixel from the largest uncapped measurement.
    pub cost: Option<f64>,
    pub total: usize,
    pub settled: usize,
    pub failed: usize,
    pub running: bool,
}

fn median(mut values: Vec<f64>) -> Option<f64> {
    values.sort_by(f64::total_cmp);
    values.get(values.len() / 2).copied()
}

/// The measurement whose ms/Mpx represents an artwork: its largest output that the budget did not cap.
pub(crate) fn cost_case<'a>(cases: impl IntoIterator<Item = &'a Case>) -> Option<&'a Case> {
    cases
        .into_iter()
        .filter(|case| case.measure.is_some_and(|m| !m.capped && m.ms_per_megapixel().is_some()))
        .max_by(|a, b| {
            let pixels = |case: &Case| case.measure.map_or(0., |m| m.megapixels());
            pixels(a).total_cmp(&pixels(b))
        })
}

/// Artworks in first-seen order.
pub(crate) fn artworks(cases: &[Case]) -> Vec<Artwork> {
    let mut ids: Vec<&str> = Vec::new();
    for case in cases {
        if !ids.contains(&case.id.as_str()) {
            ids.push(&case.id);
        }
    }
    ids.into_iter()
        .map(|id| {
            let own: Vec<&Case> = cases.iter().filter(|case| case.id == id).collect();
            let first = own[0];
            Artwork {
                id: id.to_owned(),
                name: first.name.clone(),
                coverage: first.coverage.clone(),
                input_bytes: first.input_bytes,
                parse_ms: median(own.iter().filter_map(|case| case.measure.map(|m| m.parse_ms)).collect()),
                cost: cost_case(own.iter().copied()).and_then(|case| case.measure?.ms_per_megapixel()),
                total: own.len(),
                settled: own.iter().filter(|case| case.outcome.settled()).count(),
                failed: own.iter().filter(|case| case.outcome == Outcome::Failed).count(),
                running: own.iter().any(|case| case.outcome == Outcome::Running),
            }
        })
        .collect()
}

pub(crate) fn file_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024. * 1024.))
    } else {
        format!("{} KB", (bytes as f64 / 1024.).round().max(1.))
    }
}

pub(crate) fn zoom_label(zoom: f64) -> String {
    if zoom.fract() == 0. {
        format!("{zoom:.0}×")
    } else {
        format!("{zoom}×")
    }
}

pub(crate) struct Finding {
    pub headline: String,
    pub detail: String,
}

/// Short results computed from finished measurements; empty until something has been measured.
pub(crate) fn findings(cases: &[Case]) -> Vec<Finding> {
    let artworks = artworks(cases);
    let mut out = Vec::new();
    let costed: Vec<&Artwork> = artworks.iter().filter(|a| a.cost.is_some()).collect();
    let by_cost = |a: &&&Artwork, b: &&&Artwork| a.cost.unwrap_or(0.).total_cmp(&b.cost.unwrap_or(0.));
    if let (Some(high), Some(low)) = (costed.iter().max_by(by_cost), costed.iter().min_by(by_cost))
        && high.id != low.id
    {
        let ratio = high.cost.unwrap_or(0.) / low.cost.unwrap_or(1.);
        let smaller = matches!((high.input_bytes, low.input_bytes), (Some(h), Some(l)) if h < l);
        out.push(Finding {
            headline: format!("{ratio:.1}× the cost"),
            detail: format!(
                "{} costs {ratio:.1}× as much per megapixel as {}{}.",
                high.name,
                low.name,
                if smaller { " despite a smaller file" } else { "" }
            ),
        });
    }
    let mut budgets: Vec<u64> = cases.iter().filter(|c| c.measure.is_some()).map(|c| c.budget_mib).collect();
    budgets.sort_unstable();
    budgets.dedup();
    let caps: Vec<(u64, Option<f64>, f64)> = budgets
        .iter()
        .map(|&budget| {
            let measured = cases.iter().filter(|c| c.budget_mib == budget && c.measure.is_some());
            let first_capped = measured
                .clone()
                .filter(|c| c.measure.is_some_and(|m| m.capped))
                .map(|c| c.zoom)
                .min_by(f64::total_cmp);
            let highest = measured.map(|c| c.zoom).fold(0., f64::max);
            (budget, first_capped, highest)
        })
        .collect();
    if let Some(&(_, first, highest)) = caps.first() {
        let headline = first.map_or_else(
            || format!("No cap to {}", zoom_label(highest)),
            |zoom| format!("Caps from {}", zoom_label(zoom)),
        );
        let detail = caps
            .iter()
            .map(|(budget, first, highest)| match first {
                Some(zoom) => format!("{budget} MiB caps output from {} up", zoom_label(*zoom)),
                None => format!("{budget} MiB never caps up to {}", zoom_label(*highest)),
            })
            .collect::<Vec<_>>()
            .join("; ");
        out.push(Finding { headline, detail: format!("{detail}.") });
    }
    if let Some(slow) = artworks
        .iter()
        .filter(|a| a.parse_ms.is_some())
        .max_by(|a, b| a.parse_ms.unwrap_or(0.).total_cmp(&b.parse_ms.unwrap_or(0.)))
    {
        out.push(Finding {
            headline: format!("{:.1} ms", slow.parse_ms.unwrap_or(0.)),
            detail: format!(
                "Slowest parse: {}{}. Parse time does not depend on zoom or budget.",
                slow.name,
                slow.input_bytes.map(|b| format!(" ({})", file_size(b))).unwrap_or_default()
            ),
        });
    }
    out
}

/// Made-up but plausible results for the full default plan, for layout work without running workers.
/// Output grows with zoom² until the budget caps it; raster time follows output size × a per-artwork cost.
pub(crate) fn sample_cases() -> Vec<Value> {
    use crate::svg_lab::benchmarks::{COMPARISON_BUDGETS, SWEEP_ZOOMS, planned_cases};
    // Output at 1× (density 2 included), parse ms and raster ms per megapixel, per corpus artwork.
    let profile = |id: &str| match id {
        "camera" => ((640., 480.), 9.5, 38.),
        "grapes" => ((540., 620.), 4.2, 96.),
        "metal" => ((600., 600.), 3.1, 141.),
        _ => ((720., 640.), 1.4, 17.),
    };
    planned_cases(&SWEEP_ZOOMS, &COMPARISON_BUDGETS)
        .into_iter()
        .enumerate()
        .map(|(index, mut row)| {
            let jitter = 1. + 0.06 * (index as f64 * 1.7).sin();
            let ((w, h), parse, cost) = profile(row["id"].as_str().unwrap_or_default());
            let zoom = row["zoom"].as_f64().unwrap_or(1.);
            let budget = row["budget_bytes"].as_f64().unwrap_or(0.);
            let (rw, rh) = (w * zoom, h * zoom);
            let scale = ((budget / 4.) / (rw * rh)).sqrt().min(1.);
            let (pw, ph) = ((rw * scale).floor(), (rh * scale).floor());
            let mpx = pw * ph / 1e6;
            let raster = (1.5 + cost * mpx) * jitter;
            row["outcome"] = json!("completed");
            row["requested_width"] = json!(rw as u64);
            row["requested_height"] = json!(rh as u64);
            row["pixel_width"] = json!(pw as u64);
            row["pixel_height"] = json!(ph as u64);
            row["resolution_limited"] = json!(scale < 1.);
            row["parse"] = json!({"median_ms": parse * jitter});
            row["raster"] = json!({"median_ms": raster, "max_ms": raster * 1.12});
            row["sampled_process_rss_peak_bytes"] = json!(((48. + 4.4 * mpx) * 1048576.) as u64);
            row
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    const MIB: u64 = xgpui_svg::MIB as u64;
    fn done(id: &str, zoom: f64, budget: u64, ms: f64, size: (u64, u64), requested: (u64, u64)) -> Value {
        json!({"id":id,"zoom":zoom,"budget_bytes":budget*MIB,"outcome":"completed","input_bytes":1000,
            "parse":{"median_ms":2.0},"raster":{"median_ms":ms,"max_ms":ms*1.1},
            "pixel_width":size.0,"pixel_height":size.1,"requested_width":requested.0,"requested_height":requested.1,
            "resolution_limited":size!=requested})
    }
    #[::core::prelude::v1::test]
    fn pending_and_failed_cases_have_no_measurement() {
        let rows = cases(&[
            json!({"id":"a","zoom":1,"budget_bytes":16*MIB,"outcome":"pending"}),
            json!({"id":"a","zoom":2,"budget_bytes":16*MIB,"error":"worker failed"}),
            json!({"id":"a","zoom":4,"budget_bytes":16*MIB,"outcome":"failed","raster":{"median_ms":1.0}}),
            done("a", 8., 16, 4., (100, 100), (100, 100)),
        ]);
        assert_eq!(rows[0].outcome, Outcome::Pending);
        assert_eq!(rows[1].outcome, Outcome::Failed);
        assert!(rows[..3].iter().all(|c| c.measure.is_none()));
        assert_eq!(rows[3].measure.unwrap().ms_per_megapixel(), Some(400.));
    }
    #[::core::prelude::v1::test]
    fn cost_uses_the_largest_uncapped_case_and_caps_are_detected() {
        let rows = cases(&[
            done("a", 1., 16, 1., (1000, 1000), (1000, 1000)),
            done("a", 2., 16, 4., (2000, 2000), (2000, 2000)),
            done("a", 4., 16, 4.2, (2048, 2048), (4000, 4000)),
            done("b", 2., 16, 40., (2000, 2000), (2000, 2000)),
        ]);
        let arts = artworks(&rows);
        assert_eq!(arts[0].cost, Some(1.));
        assert_eq!(arts[1].cost, Some(10.));
        let found = findings(&rows);
        assert_eq!(found[0].headline, "10.0× the cost");
        assert_eq!(found[1].headline, "Caps from 4×");
        assert_eq!(found[2].headline, "2.0 ms");
    }
    #[::core::prelude::v1::test]
    fn findings_wait_for_measurements() {
        assert!(findings(&cases(&[json!({"id":"a","zoom":1,"budget_bytes":16*MIB,"outcome":"pending"})])).is_empty());
    }
    #[::core::prelude::v1::test]
    fn sample_results_cover_the_default_plan_and_cap_at_high_zoom() {
        let cases = cases(&sample_cases());
        assert_eq!(cases.len(), 40);
        assert!(cases.iter().all(|c| c.measure.is_some()));
        let capped = |zoom: f64, mib: u64| {
            cases.iter().filter(|c| c.zoom == zoom && c.budget_mib == mib).all(|c| c.measure.unwrap().capped)
        };
        assert!(capped(16., 16) && capped(16., 32));
        assert!(cases.iter().filter(|c| c.zoom == 1.).all(|c| !c.measure.unwrap().capped));
    }
}
