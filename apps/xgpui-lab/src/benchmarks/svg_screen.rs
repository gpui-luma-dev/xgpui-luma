//! SVG Benchmarks page: artwork cards, run progress and Graph / Table tabs on `LabScreen`.
use super::*;
use super::svg_model::{self, Artwork, Case, Finding};
use crate::screen::{LabCard, LabScreen, PanelShell, TextRole, data, labeled_control, panel_rows, text};
use crate::theme::ColorVizLookExt;
use super::svg_badge::{Status, status_badge};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Tab {
    Graph,
    Table,
}
impl Tab {
    pub const ALL: [(Self, &'static str, &'static str); 2] =
        [(Self::Graph, "graph", "Graph"), (Self::Table, "table", "Table")];
    pub fn from_id(id: &str) -> Self {
        Self::ALL.iter().find(|(_, tab_id, _)| *tab_id == id).map_or(Self::Graph, |(tab, _, _)| *tab)
    }
}

/// Budget choices: one budget or both, in MiB order.
pub(super) const BUDGET_LABELS: [&str; 3] = ["16 MiB", "32 MiB", "Both"];
pub(super) fn budgets(choice: usize) -> Vec<usize> {
    let [low, high] = crate::svg_lab::benchmarks::COMPARISON_BUDGETS;
    match choice {
        0 => vec![low],
        1 => vec![high],
        _ => vec![low, high],
    }
}
/// Zoom choices: the full sweep or one zoom.
pub(super) const ZOOM_LABELS: [&str; 4] = ["Sweep 1–16×", "1×", "4×", "16×"];
pub(super) fn zooms(choice: usize) -> Vec<f32> {
    match choice {
        1 => vec![1.],
        2 => vec![4.],
        3 => vec![16.],
        _ => crate::svg_lab::benchmarks::SWEEP_ZOOMS.to_vec(),
    }
}

fn badge(look: &Look, art: &Artwork) -> impl IntoElement {
    let (label, status) = if art.failed > 0 {
        (format!("{} failed", art.failed), Status::Failed)
    } else if art.settled == art.total {
        ("Done".into(), Status::Done)
    } else if art.settled > 0 || art.running {
        (format!("{} / {}", art.settled, art.total), Status::Running)
    } else {
        ("Pending".into(), Status::Neutral)
    };
    status_badge(look, label, status)
}

fn card(look: &Look, index: usize, art: &Artwork, image: Option<Arc<RenderImage>>) -> LabCard {
    let thumbnail = div()
        .size_full()
        .when(image.is_some(), |cell| cell.bg(gpui::rgb(0xffffff)))
        .when_some(image, |cell, image| cell.child(img(image).size_full().object_fit(ObjectFit::Contain)));
    let mut subtitle = art.coverage.clone().unwrap_or_else(|| "Added artwork".into());
    if let Some(first) = subtitle.get(..1) {
        subtitle = first.to_uppercase() + &subtitle[1..];
    }
    LabCard::new(art.name.clone())
        .swatch(thumbnail)
        .mark(data::series(index))
        .subtitle(subtitle)
        .badge(badge(look, art))
        .values([
            ("size", art.input_bytes.map_or_else(|| "—".into(), svg_model::file_size)),
            ("parse", art.parse_ms.map_or_else(|| "—".into(), |ms| format!("{ms:.1} ms"))),
            ("ms/Mpx", art.cost.map_or_else(|| "—".into(), |cost| format!("{cost:.1}"))),
        ])
        .debug_selector(move || format!("svg-artwork-card-{index}"))
}

fn findings(look: &Look, findings: Vec<Finding>) -> AnyElement {
    if findings.is_empty() {
        return text(look, TextRole::Caption, "Findings appear here as cases finish.").into_any_element();
    }
    let border = look.chrome().border;
    div()
        .debug_selector(|| "svg-findings".into())
        .flex()
        .flex_wrap()
        .gap_4()
        .children(findings.into_iter().map(|finding| {
            div()
                .flex_1()
                .min_w(px(220.))
                .flex()
                .flex_col()
                .gap_1()
                .pl_3()
                .border_l_2()
                .border_color(border)
                .child(text(look, TextRole::Section, finding.headline).font_family(crate::screen::mono_family(look)))
                .child(text(look, TextRole::Caption, finding.detail))
        }))
        .into_any_element()
}

fn graph(
    look: &Look,
    cases: &[Case],
    artworks: &[Artwork],
    budgets: &[u64],
    merged: bool,
    view: crate::lab_controls::Segmented,
) -> AnyElement {
    let raster = PanelShell::new("Raster time by zoom")
        .subtitle("Median of five samples on a logarithmic axis, one chart per artwork or merged. Bars stop growing where the budget caps the output. Hover a bar for its whole zoom group.")
        .child(svg_chart::legend(look, budgets))
        .action(view)
        .child(svg_chart::raster(look, cases, artworks, budgets, merged));
    let output = PanelShell::new("Output size")
        .subtitle("Megapixels rendered at each zoom, averaged over the artwork. Orange points are capped.")
        .child(svg_chart::output(look, cases, budgets));
    let cost = PanelShell::new("Cost per megapixel")
        .subtitle("Raster ms per output megapixel, from each artwork’s largest uncapped case. Higher means more complex artwork.")
        .child(svg_chart::cost(look, artworks));
    div()
        .flex()
        .flex_col()
        .gap(crate::screen::LabMetrics::STANDARD.section_gap)
        .child(panel_rows("svg-graph", look, vec![vec![raster.into()], vec![output.into(), cost.into()]]))
        .child(findings(look, svg_model::findings(cases)))
        .into_any_element()
}

fn plan_summary(cases: &[Case], artworks: usize, zooms: usize, budgets: usize) -> String {
    let measured = cases.iter().filter(|c| c.measure.is_some()).count();
    let mut summary = format!(
        "{} cases · {artworks} artworks × {zooms} {} × {budgets} {}",
        cases.len(),
        if zooms == 1 { "zoom" } else { "zooms" },
        if budgets == 1 { "budget" } else { "budgets" }
    );
    if measured > 0 {
        summary.push_str(&format!(" · {measured} measured"));
    }
    summary
}

impl Page {
    /// Fraction of planned cases that have finished, and a time estimate from the average case so far.
    fn svg_progress(&self, cases: &[Case]) -> (f64, Option<String>) {
        let settled = cases.iter().filter(|c| c.outcome.settled()).count();
        let fraction = if cases.is_empty() {
            0.
        } else {
            settled as f64 / cases.len() as f64
        };
        let estimate = self.svg_started.filter(|_| settled > 0).map(|started| {
            let per_case = started.elapsed().as_secs_f64() / settled as f64;
            let left = per_case * (cases.len() - settled) as f64;
            if left >= 90. {
                format!("~{:.0} min left", (left / 60.).round())
            } else {
                format!("~{:.0} s left", left.ceil())
            }
        });
        (fraction, estimate)
    }

    /// The run panel under the controls: always shown, so the page keeps its shape between runs.
    fn run_panel(&self, look: &Look, cases: &[Case], estimate: Option<String>) -> impl IntoElement {
        let settled = cases.iter().filter(|c| c.outcome.settled()).count();
        let (label, status) = if self.busy {
            ("Running", Status::Running)
        } else if self.status.starts_with("Cancelled") {
            ("Stopped", Status::Neutral)
        } else if settled > 0 && settled == cases.len() {
            ("Done", Status::Done)
        } else {
            ("Ready", Status::Neutral)
        };
        let message = if !self.status.is_empty() {
            self.status.clone()
        } else {
            format!("{} cases planned. Run benchmark measures them one at a time.", cases.len())
        };
        let right = estimate.filter(|_| self.busy).unwrap_or_else(|| format!("{settled} / {}", cases.len()));
        let muted = look.chrome().muted_text;
        crate::screen::card_surface(div(), look)
            .debug_selector(|| "svg-run-progress".into())
            .px_4()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_x_4()
            .gap_y_2()
            .child(status_badge(look, label, status))
            .child(text(look, TextRole::Value, message).text_color(muted).min_w_0())
            .child(div().flex_1().min_w(px(160.)).child(self.svg_progress_bar.clone()))
            .child(text(look, TextRole::Value, right).text_color(muted))
    }

    pub(super) fn render_svg(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let look = self.look.clone();
        let cases = svg_model::cases(&self.svg_cases);
        let artworks = svg_model::artworks(&cases);
        let budgets: Vec<u64> = self.svg_budgets.iter().map(|b| (*b / xgpui_svg::MIB) as u64).collect();
        let (fraction, estimate) = self.svg_progress(&cases);
        self.svg_progress_bar.update(cx, |bar, cx| bar.set_value(fraction * 100., cx));

        let cards: Vec<LabCard> = artworks
            .iter()
            .enumerate()
            .map(|(index, art)| card(&look, index, art, self.svg_thumbnails.get(&art.id).cloned()))
            .collect();
        let build = status_badge(&look, format!("{} build", profile()), Status::Neutral);
        let mut screen = LabScreen::new("svg-benchmarks", look.clone())
            .crumb("xgpui-svg", "Benchmarks")
            .title("SVG Benchmarks")
            .lede(
                "Measure how parse and raster time grow with zoom for artwork of different size and complexity, \
                 and where the raster budget caps the output.",
            )
            .control(labeled_control(&look, self.svg_budget_choice.clone().into_element()))
            .control(labeled_control(&look, self.svg_zoom_choice.clone().into_element()))
            .control(build)
            .action(self.buttons[0].clone())
            .action(self.cancel_button.clone());
        screen = screen.notice_element(self.run_panel(&look, &cases, estimate));
        for result in &self.results {
            if let Some(error) = result["error"].as_str() {
                screen = screen.notice(Some(format!("{}: {error}", result["label"].as_str().unwrap_or("Run"))));
            }
        }
        let tab_bar = div()
            .flex()
            .flex_wrap()
            .items_end()
            .justify_between()
            .gap_3()
            .border_b_1()
            .border_color(look.chrome().border)
            .child(self.svg_tabs.clone())
            .child(
                text(&look, TextRole::Label, plan_summary(&cases, artworks.len(), self.svg_zooms.len(), budgets.len()))
                    .pb_2(),
            );
        let body = match self.svg_tab {
            Tab::Graph => {
                graph(&look, &cases, &artworks, &budgets, self.svg_raster_merged, self.svg_raster_view.clone())
            }
            Tab::Table => svg_table::view(self.svg_table.clone()).into_any_element(),
        };
        screen
            .cards(cards)
            .section(
                div()
                    .id("svg-benchmark-results")
                    .debug_selector(|| "svg-benchmark-results".into())
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(tab_bar)
                    .child(body),
            )
            .note(
                "Each case runs in a fresh copy of this app, one at a time: five parse samples, then five raster \
                 samples of the parsed artwork. Stop keeps finished results.",
            )
            .credits(format!(
                "Timings are CPU only and exclude PNG encoding and GPU upload. Debug builds run much slower than \
                 release; compare runs from the same build. RSS is for the whole process. Budgets limit the output \
                 image, not filter intermediates. {}",
                crate::svg_lab::benchmarks::artwork_credits()
            ))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[::core::prelude::v1::test]
    fn choices_map_to_budgets_and_zooms() {
        assert_eq!(budgets(0), vec![16 * xgpui_svg::MIB]);
        assert_eq!(budgets(2).len(), 2);
        assert_eq!(zooms(0), vec![1., 2., 4., 8., 16.]);
        assert_eq!(zooms(3), vec![16.]);
        assert_eq!(Tab::from_id("table"), Tab::Table);
        assert_eq!(Tab::from_id("other"), Tab::Graph);
    }
}
