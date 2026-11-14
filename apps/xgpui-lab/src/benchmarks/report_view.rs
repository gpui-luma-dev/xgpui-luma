//! Shared presentation for one benchmark report: outcome, passed checks, metrics,
//! timing distributions, parity limits and labelled captures.
use super::*;
use crate::{
    screen::{TextRole, readout, text},
    theme::ColorVizLookExt,
};
use radix::{BadgeSize, BadgeVariant, SemanticRole, Tone};

/// One timing distribution in milliseconds.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Timing {
    pub label: SharedString,
    pub frames: usize,
    pub median: f64,
    pub p95: Option<f64>,
    pub max: f64,
}
impl Timing {
    /// Nearest-rank summary of raw samples; `None` for no finite samples.
    pub fn from_samples(label: impl Into<SharedString>, samples: &[f64]) -> Option<Self> {
        let mut values: Vec<f64> = samples.iter().copied().filter(|v| v.is_finite()).collect();
        if values.is_empty() {
            return None;
        }
        values.sort_by(f64::total_cmp);
        let rank = |q: f64| values[((values.len() as f64 * q).ceil() as usize).clamp(1, values.len()) - 1];
        Some(Self {
            label: label.into(),
            frames: values.len(),
            median: rank(0.5),
            p95: (values.len() >= 20).then(|| rank(0.95)),
            max: values[values.len() - 1],
        })
    }
    /// A summary the workload already computed (`median_ms`, optional `p95_ms`, `max_ms`, `frames`).
    pub fn from_summary(label: impl Into<SharedString>, summary: &Value) -> Option<Self> {
        Some(Self {
            label: label.into(),
            frames: summary["frames"].as_u64().unwrap_or(0) as usize,
            median: summary["median_ms"].as_f64()?,
            p95: summary["p95_ms"].as_f64(),
            max: summary["max_ms"].as_f64()?,
        })
    }
}

/// A measured value and its acceptance limit, if it has one.
pub(super) struct Metric {
    pub name: SharedString,
    pub value: f64,
    pub limit: Option<f64>,
}
impl Metric {
    pub fn passes(&self) -> bool {
        self.limit.is_none_or(|limit| self.value <= limit)
    }
}
/// One parity comparison between two renders.
pub(super) struct Comparison {
    pub label: SharedString,
    pub metrics: Vec<Metric>,
}

/// Captures shown together, all in the same display box.
pub(super) struct CaptureGroup {
    pub title: SharedString,
    pub size: Size<Pixels>,
    pub captures: Vec<(SharedString, PathBuf)>,
}

/// A saved preview whose file name is `name`, from the report's `previews`.
pub(super) fn preview(report: &Value, name: &str) -> Option<PathBuf> {
    report["previews"]
        .as_array()?
        .iter()
        .filter_map(Value::as_str)
        .map(PathBuf::from)
        .find(|path| path.file_name().is_some_and(|file| file == name))
}

pub(super) fn ms(value: f64) -> String {
    format!("{value:.2}")
}

/// One report as a card. Workload views add tiles, timings, comparisons and captures.
pub(super) struct ReportCard<'a> {
    look: &'a Look,
    report: &'a Value,
    tiles: Vec<(SharedString, SharedString)>,
    timings: Option<(SharedString, Vec<Timing>)>,
    comparisons: Vec<Comparison>,
    groups: Vec<CaptureGroup>,
}
impl<'a> ReportCard<'a> {
    pub fn new(look: &'a Look, report: &'a Value) -> Self {
        Self { look, report, tiles: vec![], timings: None, comparisons: vec![], groups: vec![] }
    }
    pub fn tile(mut self, key: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        self.tiles.push((key.into(), value.into()));
        self
    }
    pub fn timings(mut self, title: impl Into<SharedString>, timings: impl IntoIterator<Item = Timing>) -> Self {
        self.timings = Some((title.into(), timings.into_iter().collect()));
        self
    }
    pub fn comparisons(mut self, comparisons: impl IntoIterator<Item = Comparison>) -> Self {
        self.comparisons.extend(comparisons);
        self
    }
    pub fn captures(mut self, group: CaptureGroup) -> Self {
        if !group.captures.is_empty() {
            self.groups.push(group);
        }
        self
    }

    fn outcome(&self) -> AnyElement {
        let outcome = self.report["outcome"].as_str().unwrap_or("unknown");
        let badge = radix::Badge::new(self.look, outcome).size(BadgeSize::One);
        match outcome {
            "completed" => badge.variant(BadgeVariant::Soft),
            "failed" => badge.variant(BadgeVariant::Solid).high_contrast(true),
            _ => badge.variant(BadgeVariant::Outline).tone(Tone::Gray),
        }
        .into_any_element()
    }
    fn header(&self) -> Div {
        let platform = [self.report["profile"].as_str(), self.report["os"].as_str(), self.report["arch"].as_str()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" · ");
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_2()
            .child(text(self.look, TextRole::Name, self.report["label"].as_str().unwrap_or("Workload").to_owned()))
            .child(self.outcome())
            .child(div().flex_1())
            .child(text(self.look, TextRole::Fine, platform))
    }
    fn errors(&self) -> Option<Div> {
        let errors: Vec<String> = self.report["error"]
            .as_str()
            .map(str::to_owned)
            .into_iter()
            .chain(
                self.report["errors"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|e| e.as_str().map_or_else(|| e.to_string(), str::to_owned)),
            )
            .collect();
        let danger = self.look.resolve_role(SemanticRole::Destructive).hsla();
        (!errors.is_empty()).then(|| {
            div()
                .flex()
                .flex_col()
                .gap_1()
                .children(errors.into_iter().map(|error| text(self.look, TextRole::Caption, error).text_color(danger)))
        })
    }
    fn checks(&self) -> Option<Div> {
        let checks: Vec<&str> = self.report["checks"].as_array()?.iter().filter_map(Value::as_str).collect();
        (!checks.is_empty()).then(|| {
            section(self.look, format!("Checks passed · {}", checks.len())).child(
                div().flex().flex_wrap().gap_1().children(checks.into_iter().map(|check| {
                    radix::Badge::new(self.look, format!("✓ {check}"))
                        .size(BadgeSize::One)
                        .variant(BadgeVariant::Surface)
                        .tone(Tone::Gray)
                })),
            )
        })
    }
    fn timing_rows(&self, title: &SharedString, timings: &[Timing]) -> Div {
        let scale = timings.iter().map(|t| t.max).fold(0., f64::max).max(f64::EPSILON);
        let fraction = |v: f64| relative((v / scale).clamp(0., 1.) as f32);
        let chrome = self.look.chrome();
        let primary = self.look.resolve_role(SemanticRole::Primary).hsla();
        let rows = timings.iter().map(|t| {
            let marker =
                |at: f64| div().absolute().top(px(-3.)).left(fraction(at)).w(px(2.)).h(px(12.)).bg(chrome.body_text);
            let bar = div()
                .relative()
                .flex_1()
                .min_w(px(120.))
                .h(px(6.))
                .rounded_full()
                .bg(chrome.border)
                .child(div().absolute().left_0().top_0().h_full().rounded_full().w(fraction(t.median)).bg(primary))
                .children(t.p95.map(marker))
                .child(marker(t.max).opacity(0.4));
            let values = [("median", Some(t.median)), ("p95", t.p95), ("max", Some(t.max))]
                .into_iter()
                .filter_map(|(key, value)| value.map(|v| (key, ms(v))));
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap_3()
                .child(div().w(px(150.)).child(text(
                    self.look,
                    TextRole::Detail,
                    format!("{} · {} frames", t.label, t.frames),
                )))
                .child(bar)
                .child(readout(self.look, values).w(px(260.)))
        });
        section(self.look, format!("{title} · ms"))
            .child(div().flex().flex_col().gap_2().children(rows))
            .child(text(
                self.look,
                TextRole::Fine,
                "Bar fills to the median; dark mark p95, faint mark max. Shared scale per card.",
            ))
    }
    fn comparison_rows(&self) -> Div {
        let danger = self.look.resolve_role(SemanticRole::Destructive).hsla();
        let rows = self.comparisons.iter().map(|comparison| {
            let pass = comparison.metrics.iter().all(Metric::passes);
            div()
                .flex()
                .flex_wrap()
                .items_baseline()
                .gap_3()
                .child(
                    text(self.look, TextRole::Value, if pass { "✓" } else { "✗" })
                        .when(!pass, |t| t.text_color(danger)),
                )
                .child(div().w(px(280.)).child(text(self.look, TextRole::Detail, comparison.label.clone())))
                .children(comparison.metrics.iter().map(|metric| {
                    let value = match metric.limit {
                        Some(limit) => format!("{:.3} ≤ {limit}", metric.value),
                        None => format!("{:.3}", metric.value),
                    };
                    readout(self.look, [(metric.name.clone(), value)]).when(!metric.passes(), |r| r.text_color(danger))
                }))
        });
        section(self.look, "Pixel parity").child(div().flex().flex_col().gap_1().children(rows))
    }
    fn capture_groups(&self) -> Div {
        div().flex().flex_col().gap_3().children(self.groups.iter().map(|group| {
            section(self.look, group.title.clone()).child(div().flex().flex_wrap().gap_3().children(
                group.captures.iter().map(|(caption, path)| {
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .w(group.size.width)
                                .h(group.size.height)
                                .border_1()
                                .border_color(self.look.chrome().border)
                                .child(img(path.clone()).size_full().object_fit(ObjectFit::Contain)),
                        )
                        .child(text(self.look, TextRole::Fine, caption.clone()))
                }),
            ))
        }))
    }

    pub fn into_element(self) -> AnyElement {
        let failed = self.report["outcome"] == "failed";
        let mut body = div().flex().flex_col().gap_4().child(self.header()).children(self.errors());
        if !failed {
            body = body
                .when(!self.tiles.is_empty(), |b| b.child(readout(self.look, self.tiles.clone())))
                .children(self.checks())
                .children(self.timings.as_ref().map(|(title, timings)| self.timing_rows(title, timings)))
                .when(!self.comparisons.is_empty(), |b| b.child(self.comparison_rows()));
        }
        body = body
            .when(!self.groups.is_empty(), |b| b.child(self.capture_groups()))
            .children(self.report["measurement"].as_str().map(|m| text(self.look, TextRole::Fine, m.to_owned())));
        radix::Card::new(self.look).child(body).into_any_element()
    }
}

fn section(look: &Look, title: impl Into<SharedString>) -> Div {
    div().flex().flex_col().gap_2().child(text(look, TextRole::Label, title))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[::core::prelude::v1::test]
    fn sample_summary_uses_nearest_rank_and_skips_p95_for_small_runs() {
        let five = Timing::from_samples("a", &[5., 1., 3., f64::NAN, 2., 4.]).unwrap();
        assert_eq!((five.frames, five.median, five.p95, five.max), (5, 3., None, 5.));
        let many: Vec<f64> = (1..=20).map(f64::from).collect();
        assert_eq!(Timing::from_samples("b", &many).unwrap().p95, Some(19.));
        assert!(Timing::from_samples("c", &[]).is_none());
    }
    #[::core::prelude::v1::test]
    fn previews_match_whole_file_names() {
        let report = json!({"previews": ["/out/sea-0-instanced.png", "/out/sea-0-wgpu-instanced.png"]});
        assert_eq!(preview(&report, "sea-0-instanced.png"), Some(PathBuf::from("/out/sea-0-instanced.png")));
        assert_eq!(preview(&report, "instanced.png"), None);
    }
    #[::core::prelude::v1::test]
    fn metrics_without_limits_always_pass() {
        assert!(Metric { name: "max".into(), value: 90., limit: None }.passes());
        assert!(!Metric { name: "mae".into(), value: 1.5, limit: Some(1.) }.passes());
    }
}
