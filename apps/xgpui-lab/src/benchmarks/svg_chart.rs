//! SVG benchmark charts on xgpui-luma-plot: raster time by zoom, output size and cost per megapixel.
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use gpui::{
    AnyElement, Hsla, InteractiveElement, IntoElement, ParentElement, SharedString, Styled, div,
    prelude::FluentBuilder, px,
};
use xgpui_luma::controls::tooltip::Tooltip;
use xgpui_luma_look_radix::Look;
use xgpui_luma_plot::{
    Annotation, AnnotationPresentation, Axis, AxisPresentation, AxisSide, BarGrouping, BarLabel, BarLayer, BarLayout,
    BarOrientation, BarPresentation, BarRenderModel, CartesianChartBuilder, CartesianStyle, CategoryScale,
    ChartAnnotation, Dataset, EmptyStatePresentation, GridPresentation, LineLayer, LinePresentation,
    MarkerPresentation, NumericScale, Sample, Series, Stroke, TextPresentation,
};

use super::svg_model::{self, Artwork, Case};
use crate::{
    screen::{TextRole, data, mono_family, text},
    theme::ColorVizLookExt,
};

pub(super) const FACET_HEIGHT: f32 = 200.;
pub(super) const MERGED_HEIGHT: f32 = 340.;
pub(super) const SMALL_HEIGHT: f32 = 180.;
/// Faded series for the second budget; the plot crate has no dashed strokes.
const SECOND_BUDGET_OPACITY: f32 = 0.45;

fn text_style(look: &Look, role: TextRole, family: SharedString) -> TextPresentation {
    let style = role.style(look);
    TextPresentation {
        color: look.chrome().muted_text,
        font_family: family,
        font_size: px(style.size),
        line_height: px(style.line_height),
    }
}

fn sans(look: &Look) -> SharedString {
    look.common_stylesheet().tokens.typography.font.sans.family.clone().into()
}

/// Chrome for every chart on the page: look roles for axes, grid and text; legends are drawn by the page.
fn style(look: &Look, x_title: &str, y_title: &str) -> CartesianStyle {
    let chrome = look.chrome();
    let mut axis =
        AxisPresentation::new(chrome.border).tick_labels(text_style(look, TextRole::Label, mono_family(look)));
    axis.title = text_style(look, TextRole::Label, sans(look));
    let mut style = CartesianStyle::new(axis.clone(), LinePresentation::new(chrome.muted_text));
    let mut bottom = Axis::new(AxisSide::Bottom, axis.clone());
    let mut left = Axis::new(AxisSide::Left, axis);
    if !x_title.is_empty() {
        bottom = bottom.title(x_title.to_owned());
    }
    if !y_title.is_empty() {
        left = left.title(y_title.to_owned());
    }
    style.axes = vec![bottom, left];
    style.grid = GridPresentation { vertical: None, ..GridPresentation::new(chrome.border) };
    style.legend.visible = false;
    style.empty_state = Some(EmptyStatePresentation::new("Waiting for measurements", chrome.muted_text));
    style
}

fn line(color: Hsla, primary: bool) -> LinePresentation {
    let (color, width, radius) = if primary {
        (color, 2., 3.5)
    } else {
        (color.opacity(SECOND_BUDGET_OPACITY), 1.5, 2.5)
    };
    LinePresentation {
        stroke: Stroke { color, width: px(width) },
        markers: Some(MarkerPresentation::new(color, px(radius))),
    }
}

/// Zooms that appear in the plan, in order.
fn zooms(cases: &[Case]) -> Vec<f64> {
    let mut zooms: Vec<f64> = cases.iter().map(|c| c.zoom).filter(|z| z.is_finite() && *z > 0.).collect();
    zooms.sort_by(f64::total_cmp);
    zooms.dedup();
    zooms
}

/// Log-2 zoom axis with a tick at each planned zoom.
/// Zoom axis padding past the first and last zoom (log scale), so end markers are not clipped.
const ZOOM_PAD: f64 = 1.15;

fn zoom_scale(zooms: &[f64]) -> NumericScale {
    let (low, high) = match (zooms.first(), zooms.last()) {
        // A little room past the first and last zoom, so end markers and labels are not clipped.
        (Some(&low), Some(&high)) if low < high => (low / ZOOM_PAD, high * ZOOM_PAD),
        (Some(&zoom), _) => (zoom / 2., zoom * 2.),
        _ => (1., 16.),
    };
    NumericScale::new()
        .logarithmic(2.)
        .domain(low..=high)
        .tick_values(zooms.iter().copied())
        .tick_labels(zooms.iter().map(|&z| (z, svg_model::zoom_label(z))))
}

fn wrap_dynamic(selector: String, height: f32, chart: impl IntoElement) -> AnyElement {
    div()
        .debug_selector(move || selector.clone())
        .w_full()
        .h(px(height))
        .child(chart)
        .into_any_element()
}

fn wrap(selector: &'static str, height: f32, chart: impl IntoElement) -> AnyElement {
    div().debug_selector(move || selector.into()).w_full().h(px(height)).child(chart).into_any_element()
}

#[cfg(test)]
thread_local! {
    /// The most recent raster tooltip text, so tests can hover a chart and read what it showed.
    static LAST_TOOLTIP: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

fn plain(look: &Look) -> AnnotationPresentation {
    AnnotationPresentation {
        stroke: None,
        fill: None,
        marker_radius: px(0.),
        text: None,
        text_style: text_style(look, TextRole::Label, mono_family(look)),
    }
}

/// Log-10 axis from the decade below the fastest case to the decade above the slowest.
fn decade_scale(cases: &[Case]) -> (f64, NumericScale) {
    let times = cases.iter().filter_map(|c| c.measure).map(|m| m.raster_ms.max(1e-3));
    let (low, high) = times.fold((f64::INFINITY, 0_f64), |(lo, hi), ms| (lo.min(ms), hi.max(ms)));
    let (floor, ceil) = if high > 0. {
        let bottom = low.log10().floor();
        (10_f64.powf(bottom), 10_f64.powf(high.log10().ceil().max(bottom + 1.)))
    } else {
        (1., 1000.)
    };
    let decades: Vec<f64> = (floor.log10().round() as i32..=ceil.log10().round() as i32)
        .map(|exponent| 10_f64.powi(exponent))
        .collect();
    let scale = NumericScale::new()
        .logarithmic(10.)
        .domain(floor..=ceil)
        .tick_values(decades.iter().copied())
        .tick_labels(decades.iter().map(|&v| (v, format!("{v}"))));
    (floor, scale)
}

/// Bars with an orange top edge where the budget capped the output; the plot crate has no outlined bars.
/// `capped` holds (series id, x) pairs.
fn capped_bars(
    capped: HashSet<(String, u64)>,
) -> impl Fn(&BarRenderModel, &mut gpui::Window, &mut gpui::App) -> gpui::Div + 'static {
    let edge = data::color(data::CAPPED);
    move |model, _, _| {
        let is_capped = capped.contains(&(model.series.as_str().to_owned(), model.sample.x.to_bits()));
        div()
            .w(model.bounds.size.width)
            .h(model.bounds.size.height)
            .rounded_t(px(2.))
            .bg(model.presentation.fill)
            .when(is_capped, |bar| bar.border_t_2().border_color(edge))
    }
}

/// Tooltip for one zoom group in the merged chart: every artwork and budget, slowest first.
fn merged_text(zoom: f64, mut rows: Vec<(String, f64, bool)>) -> String {
    rows.sort_by(|a, b| b.1.total_cmp(&a.1));
    let mut text = format!("{} · raster ms", svg_model::zoom_label(zoom));
    for (name, ms, capped) in rows {
        text.push_str(&format!("\n{name}  {ms:.1}{}", if capped { "  capped" } else { "" }));
    }
    text
}

/// Tooltip for one zoom group in one facet: both budgets, with capped cases marked.
fn group_text(name: &str, zoom: f64, rows: &[(u64, f64, bool)]) -> String {
    let mut text = format!("{name} · {} · raster ms", svg_model::zoom_label(zoom));
    for (budget, ms, capped) in rows {
        text.push_str(&format!("\n{budget} MiB  {ms:.1}{}", if *capped { "  capped" } else { "" }));
    }
    text
}

/// Raster time for one artwork: a group of budget bars at each zoom, on the shared log axis.
fn raster_facet(look: &Look, index: usize, art: &Artwork, cases: &[Case], budgets: &[u64]) -> AnyElement {
    let zooms = zooms(cases);
    let (floor, y_scale) = decade_scale(cases);
    let color = data::series(index);
    let mut data = Dataset::new();
    let mut bars = Vec::new();
    let mut capped: HashSet<(String, u64)> = HashSet::new();
    let mut groups: HashMap<u64, Vec<(u64, f64, bool)>> = HashMap::new();
    for (rank, &budget) in budgets.iter().enumerate() {
        let id = format!("{budget}");
        let samples: Vec<Sample> = zooms
            .iter()
            .enumerate()
            .filter_map(|(slot, &zoom)| {
                let case = cases.iter().find(|c| c.id == art.id && c.budget_mib == budget && c.zoom == zoom)?;
                let m = case.measure?;
                let x = slot as f64;
                if m.capped {
                    capped.insert((id.clone(), x.to_bits()));
                }
                groups.entry(x.to_bits()).or_default().push((budget, m.raster_ms, m.capped));
                Some(Sample::new(x, m.raster_ms.max(floor)))
            })
            .collect();
        if let Ok(next) = data.clone().with_series(Series::new(id.clone(), format!("{budget} MiB"), samples)) {
            data = next;
            let fill = if rank == 0 {
                color
            } else {
                color.opacity(SECOND_BUDGET_OPACITY)
            };
            bars.push((id, fill));
        }
    }
    let mut chart = CartesianChartBuilder::new(data)
        .id(SharedString::from(format!("svg-raster-facet-{index}")))
        .x_scale(CategoryScale::new(zooms.iter().map(|&z| svg_model::zoom_label(z))))
        .y_scale(y_scale)
        .bar_layout(BarLayout::new().grouping(BarGrouping::Grouped).width(0.7).gap(0.08))
        .style(style(look, "", "ms"));
    for (id, fill) in bars {
        chart = chart.bar(BarLayer::new(id.clone()).baseline(floor)).bar_style(id, BarPresentation::new(fill));
    }
    // Capped bars get an orange top edge; the plot crate has no outlined bars.
    let chart = chart.bar_template(capped_bars(capped));
    // One tooltip per zoom group with both budgets, whichever bar is hovered.
    let (name, groups) = (art.name.clone(), Arc::new(groups));
    let zooms = Arc::new(zooms);
    let chart = chart
        .bar_tooltip(move |point| {
            let slot = point.sample.x;
            let zoom = zooms.get(slot as usize).copied().unwrap_or(f64::NAN);
            let text = group_text(&name, zoom, groups.get(&slot.to_bits()).map_or(&[][..], Vec::as_slice));
            #[cfg(test)]
            LAST_TOOLTIP.with(|last| *last.borrow_mut() = Some(text.clone()));
            Tooltip::new(text)
        })
        .build();
    wrap_dynamic(format!("svg-raster-facet-{index}"), FACET_HEIGHT, chart)
}

/// All artworks in one bar chart: at each zoom, a bar per artwork and budget in run order.
fn raster_merged(look: &Look, cases: &[Case], artworks: &[Artwork], budgets: &[u64]) -> AnyElement {
    let zooms = zooms(cases);
    let (floor, y_scale) = decade_scale(cases);
    let mut data = Dataset::new();
    let mut bars = Vec::new();
    let mut capped: HashSet<(String, u64)> = HashSet::new();
    let mut groups: HashMap<u64, Vec<(String, f64, bool)>> = HashMap::new();
    for (index, art) in artworks.iter().enumerate() {
        for (rank, &budget) in budgets.iter().enumerate() {
            let id = format!("{index}@{budget}");
            let label = format!("{} · {budget} MiB", art.name);
            let samples: Vec<Sample> = zooms
                .iter()
                .enumerate()
                .filter_map(|(slot, &zoom)| {
                    let case = cases.iter().find(|c| c.id == art.id && c.budget_mib == budget && c.zoom == zoom)?;
                    let m = case.measure?;
                    let x = slot as f64;
                    if m.capped {
                        capped.insert((id.clone(), x.to_bits()));
                    }
                    groups.entry(x.to_bits()).or_default().push((label.clone(), m.raster_ms, m.capped));
                    Some(Sample::new(x, m.raster_ms.max(floor)))
                })
                .collect();
            if let Ok(next) = data.clone().with_series(Series::new(id.clone(), label, samples)) {
                data = next;
                let color = data::series(index);
                bars.push((
                    id,
                    if rank == 0 {
                        color
                    } else {
                        color.opacity(SECOND_BUDGET_OPACITY)
                    },
                ));
            }
        }
    }
    let mut chart = CartesianChartBuilder::new(data)
        .id("svg-raster-merged")
        .x_scale(CategoryScale::new(zooms.iter().map(|&z| svg_model::zoom_label(z))))
        .y_scale(y_scale)
        .bar_layout(BarLayout::new().grouping(BarGrouping::Grouped).width(0.85).gap(0.04))
        .style(style(look, "Zoom (density 2)", "Raster ms"));
    for (id, fill) in bars {
        chart = chart.bar(BarLayer::new(id.clone()).baseline(floor)).bar_style(id, BarPresentation::new(fill));
    }
    let (groups, zooms) = (Arc::new(groups), Arc::new(zooms));
    let chart = chart
        .bar_template(capped_bars(capped))
        .bar_tooltip(move |point| {
            let slot = point.sample.x;
            let zoom = zooms.get(slot as usize).copied().unwrap_or(f64::NAN);
            let text = merged_text(zoom, groups.get(&slot.to_bits()).cloned().unwrap_or_default());
            #[cfg(test)]
            LAST_TOOLTIP.with(|last| *last.borrow_mut() = Some(text.clone()));
            Tooltip::new(text)
        })
        .build();
    let legend =
        div()
            .flex()
            .flex_wrap()
            .gap_x_4()
            .gap_y_1()
            .children(artworks.iter().enumerate().map(|(index, art)| {
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().size(px(10.)).rounded_sm().bg(data::series(index)))
                    .child(text(look, TextRole::Detail, art.name.clone()))
            }));
    div()
        .debug_selector(|| "svg-raster-chart".into())
        .flex()
        .flex_col()
        .gap_2()
        .child(legend)
        .child(wrap("svg-raster-merged", MERGED_HEIGHT, chart))
        .into_any_element()
}

/// Raster time by zoom: one bar chart per artwork in a 2 × 2 grid, sharing the log axis so
/// artworks compare at a glance. Rows wrap to one chart per row in narrow windows.
pub(super) fn raster(look: &Look, cases: &[Case], artworks: &[Artwork], budgets: &[u64], merged: bool) -> AnyElement {
    if merged {
        return raster_merged(look, cases, artworks, budgets);
    }
    let facet = |index: usize, art: &Artwork| {
        let header = div()
            .flex()
            .items_center()
            .gap_2()
            .child(div().size(px(10.)).rounded_sm().bg(data::series(index)))
            .child(text(look, TextRole::Detail, art.name.clone()));
        div()
            .flex_1()
            .min_w(px(300.))
            .flex()
            .flex_col()
            .gap_1()
            .child(header)
            .child(raster_facet(look, index, art, cases, budgets))
    };
    let items: Vec<(usize, &Artwork)> = artworks.iter().enumerate().collect();
    div()
        .debug_selector(|| "svg-raster-chart".into())
        .flex()
        .flex_col()
        .gap_4()
        .children(items.chunks(2).map(|pair| {
            div()
                .flex()
                .flex_wrap()
                .gap_x_6()
                .gap_y_4()
                .children(pair.iter().map(|(index, art)| facet(*index, art)))
                // An odd last chart keeps half the width instead of stretching across the row.
                .when(pair.len() == 1, |row| row.child(div().flex_1().min_w(px(300.))))
        }))
        .into_any_element()
}

/// Average output megapixels at each zoom, per budget, with capped averages marked.
pub(super) fn output(look: &Look, cases: &[Case], budgets: &[u64]) -> AnyElement {
    let zooms = zooms(cases);
    let mut data = Dataset::new();
    let mut lines = Vec::new();
    let mut capped = Vec::new();
    let mut ends = Vec::new();
    for (rank, &budget) in budgets.iter().enumerate() {
        let samples: Vec<Sample> = zooms
            .iter()
            .filter_map(|&zoom| {
                let measured: Vec<_> = cases
                    .iter()
                    .filter(|c| c.zoom == zoom && c.budget_mib == budget)
                    .filter_map(|c| c.measure)
                    .collect();
                if measured.is_empty() {
                    return None;
                }
                let mean = measured.iter().map(|m| m.megapixels()).sum::<f64>() / measured.len() as f64;
                if measured.iter().any(|m| m.capped) {
                    capped.push(Sample::new(zoom, mean));
                }
                Some(Sample::new(zoom, mean))
            })
            .collect();
        if samples.is_empty() {
            continue;
        }
        let id = format!("{budget}");
        if let Some(last) = samples.last() {
            ends.push((budget, rank, *last));
        }
        if let Ok(next) = data.clone().with_series(Series::new(id.clone(), format!("{budget} MiB"), samples)) {
            data = next;
            lines.push((id, line(look.chrome().muted_text, rank == 0)));
        }
    }
    let top = max_y(&data);
    let mut chart = CartesianChartBuilder::new(data)
        .id("svg-output-chart")
        .x_scale(zoom_scale(&zooms))
        .y_scale(linear_scale(top))
        .style(style(look, "", "Mpx"));
    for (id, presentation) in lines {
        chart = chart.line(LineLayer::new(id.clone())).line_style(id, presentation);
    }
    for sample in capped {
        chart = chart.annotation(ChartAnnotation::new(
            Annotation::Point(sample),
            AnnotationPresentation {
                fill: Some(data::color(data::CAPPED).into()),
                marker_radius: px(4.),
                ..plain(look)
            },
        ));
    }
    // Line labels as in the mockup ("16 MiB · 4.2 Mpx"). Plot text paints left-aligned from its
    // top-left corner, so each label starts past the next-to-last zoom and hangs just below its line
    let muted = look.chrome().muted_text;
    let label_x = zooms.iter().rev().nth(1).copied();
    for (budget, rank, end) in ends {
        let Some(x) = label_x else { break };
        let color = if rank == 0 {
            muted
        } else {
            muted.opacity(SECOND_BUDGET_OPACITY)
        };
        chart = chart.annotation(ChartAnnotation::new(
            Annotation::Text(Sample::new(x * 1.08, end.y)),
            AnnotationPresentation {
                text: Some(format!("{budget} MiB · {:.1} Mpx", end.y).into()),
                text_style: TextPresentation { color, ..text_style(look, TextRole::Label, mono_family(look)) },
                ..plain(look)
            },
        ));
    }
    wrap("svg-output-chart", SMALL_HEIGHT, chart.build())
}

/// Largest y value in the data, for a zero-based axis.
fn max_y(data: &Dataset) -> f64 {
    data.series()
        .iter()
        .flat_map(|s| s.samples().iter().map(|p| p.y))
        .filter(|y| y.is_finite())
        .fold(0., f64::max)
}

/// Zero-based linear axis with four or five round ticks (steps of 1, 2, 2.5 or 5 × 10ⁿ).
fn linear_scale(max: f64) -> NumericScale {
    if max <= 0. {
        return NumericScale::new().domain(0.0..=1.0).tick_values([0., 0.5, 1.]);
    }
    let rough = max / 4.;
    let power = 10f64.powf(rough.log10().floor());
    let step = [1., 2., 2.5, 5., 10.].into_iter().map(|m| m * power).find(|s| *s >= rough).unwrap_or(10. * power);
    let top = (max / step).ceil() * step;
    let ticks: Vec<f64> = (0..=((top / step).round() as usize)).map(|i| i as f64 * step).collect();
    // Fewest decimals that show the step exactly (0.25 → 2, 0.5 → 1, 20 → 0).
    let digits = (0..4usize)
        .find(|d| {
            let s = step * 10f64.powi(*d as i32);
            (s - s.round()).abs() < 1e-6
        })
        .unwrap_or(3);
    NumericScale::new()
        .domain(0.0..=top)
        .tick_values(ticks.clone())
        .tick_labels(ticks.into_iter().map(move |t| (t, format!("{t:.digits$}"))))
}

/// Raster ms per output megapixel for each artwork, from its largest uncapped measurement.
pub(super) fn cost(look: &Look, artworks: &[Artwork]) -> AnyElement {
    let mut data = Dataset::new();
    let mut bars = Vec::new();
    for (index, art) in artworks.iter().enumerate() {
        let Some(cost) = art.cost else {
            continue;
        };
        let id = format!("cost-{index}");
        if let Ok(next) =
            data.clone()
                .with_series(Series::new(id.clone(), art.name.clone(), [Sample::new(index as f64, cost)]))
        {
            data = next;
            bars.push((id, data::series(index)));
        }
    }
    let names: Vec<String> = artworks.iter().map(|a| a.name.clone()).collect();
    let top = max_y(&data);
    let mut chart = CartesianChartBuilder::new(data)
        .id("svg-cost-chart")
        .x_scale(CategoryScale::new(names))
        .y_scale(linear_scale(top))
        .bar_orientation(BarOrientation::Right)
        .bar_layout(BarLayout::new().grouping(BarGrouping::Stacked).width(0.55))
        .style(style(look, "", "ms/Mpx"));
    for (id, color) in bars {
        chart = chart.bar(BarLayer::new(id.clone())).bar_style(id, BarPresentation::new(color));
    }
    let label = text_style(look, TextRole::Label, mono_family(look));
    let chart = chart
        .bar_labels(move |bar| vec![BarLabel::new(format!("{:.0}", bar.sample.y), label.clone())])
        .build();
    wrap("svg-cost-chart", SMALL_HEIGHT, chart)
}

fn swatch(color: Hsla, height: f32) -> gpui::Div {
    div().w(px(16.)).h(px(height)).flex_shrink_0().bg(color)
}

/// Budget and cap key for the raster facets, drawn by the page so it wraps like other lab legends.
pub(super) fn legend(look: &Look, budgets: &[u64]) -> gpui::Div {
    let item = |mark: gpui::Div, label: String| {
        div().flex().items_center().gap_2().child(mark).child(text(look, TextRole::Detail, label))
    };
    let muted = look.chrome().muted_text;
    let capped = data::color(data::CAPPED);
    div()
        .flex()
        .flex_wrap()
        .gap_x_4()
        .gap_y_1()
        .children(budgets.iter().enumerate().map(|(rank, budget)| {
            let color = if rank == 0 {
                muted
            } else {
                muted.opacity(SECOND_BUDGET_OPACITY)
            };
            item(swatch(color, 10.), format!("{budget} MiB"))
        }))
        .child(item(swatch(muted, 10.).border_t_2().border_color(capped), "capped".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[::core::prelude::v1::test]
    fn zoom_axis_ticks_follow_the_plan_and_single_zooms_get_a_domain() {
        let cases = svg_model::cases(&[
            json!({"id":"a","zoom":4,"budget_bytes":16*xgpui_svg::MIB,"outcome":"pending"}),
            json!({"id":"a","zoom":1,"budget_bytes":16*xgpui_svg::MIB,"outcome":"pending"}),
        ]);
        assert_eq!(zooms(&cases), vec![1., 4.]);
        assert_eq!(zoom_scale(&[4.]).domain_bounds(), Some(&(2.0..=8.0)));
        assert_eq!(zoom_scale(&[1., 16.]).explicit_ticks(), Some(&[1., 16.][..]));
    }
    #[::core::prelude::v1::test]
    fn merged_tooltips_list_every_bar_in_the_group_slowest_first() {
        let text =
            merged_text(4., vec![("dragon · 16 MiB".into(), 78.4, false), ("metal · 16 MiB".into(), 583.2, true)]);
        assert_eq!(text, "4× · raster ms\nmetal · 16 MiB  583.2  capped\ndragon · 16 MiB  78.4");
    }
    #[::core::prelude::v1::test]
    fn group_tooltips_show_both_budgets_and_caps() {
        let text = group_text("metal", 4., &[(16, 583.2, true), (32, 760.3, false)]);
        assert_eq!(text, "metal · 4× · raster ms\n16 MiB  583.2  capped\n32 MiB  760.3");
    }
    #[cfg(feature = "test-support")]
    #[::core::prelude::v1::test]
    fn hovering_a_bar_shows_its_zoom_group() {
        use gpui::{Render, TestAppContext, point};
        struct Host(Vec<Case>);
        impl Render for Host {
            fn render(&mut self, _: &mut gpui::Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                let look = crate::theme::default_look();
                let artworks = svg_model::artworks(&self.0);
                div().w(px(1000.)).child(raster(&look, &self.0, &artworks, &[16, 32], false))
            }
        }
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let cases = svg_model::cases(&svg_model::sample_cases());
        let (_, cx) = app.add_window_view(|_, _| Host(cases));
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        let facet = cx.debug_bounds("svg-raster-facet-0").expect("first facet");
        // 4× is the middle of five zoom groups; aim at its left (16 MiB) bar, low enough to be inside it.
        let plot_left = facet.left() + px(50.);
        let hover = point(plot_left + (facet.right() - plot_left) * 0.5 - px(12.), facet.bottom() - px(40.));
        LAST_TOOLTIP.with(|last| last.borrow_mut().take());
        for _ in 0..3 {
            cx.executor().advance_clock(std::time::Duration::from_millis(1));
            cx.run_until_parked();
            cx.update(|window, cx| {
                window.simulate_mouse_move(hover, cx);
                window.draw(cx).clear(cx);
            });
        }
        let text = LAST_TOOLTIP.with(|last| last.borrow().clone()).expect("hovering a bar shows a tooltip");
        assert_eq!(text.lines().next(), Some("camera · 4× · raster ms"), "{text}");
        assert!(text.contains("16 MiB") && text.contains("32 MiB") && text.contains("capped"), "{text}");
    }
}
