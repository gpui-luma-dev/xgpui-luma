//! The screen's main visualization.
//!
//! `Fixed` (Italy) is added with the screen that needs it; see `docs/lab-screen-api.md`.
use gpui::{AnyElement, Div, Hsla, IntoElement, ParentElement, SharedString, Styled, div, px};
use xgpui_luma_look_radix::Look;

use super::{LabMetrics, TextRole, text};
use crate::theme::ColorVizLookExt;

pub(crate) enum Hero {
    /// Full content width; the visual owns its height. Color Spaces field.
    Full(AnyElement),
    /// Visual with a legend column to its right; the legend wraps below when narrow. Chromaticity.
    WithLegend { visual: AnyElement, legend: Legend },
    /// Two panes side by side, each with its own heading; they wrap below each other when narrow.
    Compare(ComparePane, ComparePane),
}

impl Hero {
    pub fn full(visual: impl IntoElement) -> Self {
        Self::Full(visual.into_any_element())
    }

    pub fn with_legend(visual: impl IntoElement, legend: Legend) -> Self {
        Self::WithLegend { visual: visual.into_any_element(), legend }
    }

    pub fn compare(left: ComparePane, right: ComparePane) -> Self {
        Self::Compare(left, right)
    }

    pub(super) fn render(self, look: &Look, metrics: &LabMetrics) -> Div {
        match self {
            Self::Full(visual) => div().flex().flex_col().w_full().flex_shrink_0().child(visual),
            Self::WithLegend { visual, legend } => div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap(metrics.part_gap)
                .child(div().flex_1().min_w(metrics.hero_min_width).child(visual))
                .child(legend.render(look, metrics)),
            Self::Compare(left, right) => div()
                .flex()
                .flex_wrap()
                .gap(metrics.section_gap)
                .child(left.render(look, metrics))
                .child(right.render(look, metrics)),
        }
    }
}

/// One side of a comparison: a one-line heading (title, subtitle, badge) over the visual.
pub(crate) struct ComparePane {
    title: SharedString,
    subtitle: Option<SharedString>,
    badge: Option<AnyElement>,
    visual: AnyElement,
}

impl gpui::prelude::FluentBuilder for ComparePane {}

impl ComparePane {
    pub fn new(title: impl Into<SharedString>, visual: impl IntoElement) -> Self {
        Self { title: title.into(), subtitle: None, badge: None, visual: visual.into_any_element() }
    }

    pub fn subtitle(mut self, text: impl Into<SharedString>) -> Self {
        self.subtitle = Some(text.into());
        self
    }

    /// Any element at the right of the heading. The screen decides what a badge is.
    pub fn badge(mut self, badge: impl IntoElement) -> Self {
        self.badge = Some(badge.into_any_element());
        self
    }

    fn render(self, look: &Look, metrics: &LabMetrics) -> Div {
        // Title and subtitle share one line, so the heading costs a single row of height.
        let heading = div()
            .flex()
            .items_baseline()
            .gap_2()
            .flex_1()
            .min_w_0()
            .child(text(look, TextRole::Name, self.title).flex_shrink_0())
            .children(self.subtitle.map(|subtitle| text(look, TextRole::Caption, subtitle).truncate()));
        let header = div()
            .flex()
            .items_center()
            .gap_2()
            .child(heading)
            .children(self.badge.map(|badge| div().flex().flex_shrink_0().child(badge)));
        div()
            .flex()
            .flex_col()
            .gap_2()
            .flex_1()
            .flex_basis(px(0.))
            .min_w(metrics.hero_min_width)
            .child(header)
            .child(super::card_surface(div(), look).child(self.visual))
    }
}

/// How a legend row is marked. Colors are data colors (see `screen::data`) or `None` for foreground.
pub(crate) enum Mark {
    /// A short line in an outline color.
    Line(Hsla),
    /// A point marker drawn like the diagram's: a 2 px cross over a 4 px halo.
    /// `None` uses the look's foreground (color) or background (halo).
    Cross { color: Option<Hsla>, halo: Option<Hsla> },
    /// Hatched region.
    Hatch,
}

struct LegendRow {
    mark: Mark,
    label: SharedString,
    value: Option<SharedString>,
}

/// A 12 px cross: 10 px arms, 2 px stroke, over a 1 px halo on every side.
fn cross(color: Hsla, halo: Hsla) -> Div {
    let bar = |left: f32, top: f32, width: f32, height: f32, fill: Hsla| {
        div().absolute().left(px(left)).top(px(top)).w(px(width)).h(px(height)).bg(fill)
    };
    div()
        .relative()
        .size(px(12.))
        .child(bar(0., 4., 12., 4., halo))
        .child(bar(4., 0., 4., 12., halo))
        .child(bar(1., 5., 10., 2., color))
        .child(bar(5., 1., 2., 10., color))
}

/// Series rows, an optional note on what the values mean, then marker rows.
#[derive(Default)]
pub(crate) struct Legend {
    items: Vec<LegendRow>,
    note: Option<SharedString>,
    markers: Vec<LegendRow>,
}

impl gpui::prelude::FluentBuilder for Legend {}

impl Legend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn item(mut self, mark: Mark, label: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        self.items.push(LegendRow { mark, label: label.into(), value: Some(value.into()) });
        self
    }

    pub fn note(mut self, text: impl Into<SharedString>) -> Self {
        self.note = Some(text.into());
        self
    }

    pub fn marker(mut self, mark: Mark, label: impl Into<SharedString>) -> Self {
        self.markers.push(LegendRow { mark, label: label.into(), value: None });
        self
    }

    fn render(self, look: &Look, metrics: &LabMetrics) -> Div {
        let chrome = look.chrome();
        let mark_width = metrics.legend_mark_width;
        let row = |row: LegendRow| {
            let mark = match row.mark {
                Mark::Line(color) => div().w(px(16.)).h(px(2.)).bg(color),
                Mark::Cross { color, halo } => {
                    cross(color.unwrap_or(chrome.body_text), halo.unwrap_or(chrome.content_background))
                }
                Mark::Hatch => text(look, TextRole::Label, "////"),
            };
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().w(mark_width).flex_shrink_0().flex().items_center().child(mark))
                .child(text(look, TextRole::Detail, row.label).flex_1())
                .children(row.value.map(|value| text(look, TextRole::Value, value)))
        };
        let has_markers = !self.markers.is_empty();
        div()
            .w(metrics.legend_width)
            .flex_shrink_0()
            .flex()
            .flex_col()
            // Compact technical type: rows use the card detail roles, not body text.
            .gap_2()
            .children(self.items.into_iter().map(&row))
            .children(self.note.map(|note| text(look, TextRole::Label, note)))
            .children(has_markers.then(|| div().h(px(1.)).bg(chrome.border)))
            .children(self.markers.into_iter().map(&row))
    }
}
