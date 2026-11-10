//! Variant / comparison card.
use gpui::{
    AnyElement, Div, Hsla, InteractiveElement, IntoElement, ParentElement, SharedString, Styled, div,
    prelude::FluentBuilder,
};
use lucide_svg_static::Icon;
use xgpui_luma::{hstack, infra::icon::lucide_icon, vstack};
use xgpui_luma_look_radix::{Look, ScaleFamily};

use crate::theme::ColorVizLookExt;

use super::{LabMetrics, TextRole, card_surface, readout, text};

/// One card in the screen's card row. Styling is fixed; the screen supplies content.
pub(crate) struct LabCard {
    name: SharedString,
    swatch: Option<AnyElement>,
    icon: Option<Icon>,
    mark: Option<Hsla>,
    subtitle: Option<SharedString>,
    badge: Option<AnyElement>,
    values: Vec<(SharedString, SharedString)>,
    disabled: bool,
    selector: Option<String>,
}

impl FluentBuilder for LabCard {}

impl LabCard {
    pub fn new(name: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            swatch: None,
            icon: None,
            mark: None,
            subtitle: None,
            badge: None,
            values: Vec::new(),
            disabled: false,
            selector: None,
        }
    }

    /// Swatch content, sized and clipped by the card. Usually a data-color paint.
    pub fn swatch(mut self, swatch: impl IntoElement) -> Self {
        self.swatch = Some(swatch.into_any_element());
        self
    }

    /// Swatch filled with one data color.
    pub fn swatch_color(self, color: impl Into<Hsla>) -> Self {
        self.swatch(div().size_full().rounded_sm().bg(color.into()))
    }

    /// A line icon in place of the swatch, for status cards: gray 3 box, gray 6 border.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Small square in a data color before the name, tying the card to a series in the hero.
    pub fn mark(mut self, color: impl Into<Hsla>) -> Self {
        self.mark = Some(color.into());
        self
    }

    pub fn subtitle(mut self, text: impl Into<SharedString>) -> Self {
        self.subtitle = Some(text.into());
        self
    }

    /// Any element, placed right of the name. The screen decides what a badge is.
    pub fn badge(mut self, badge: impl IntoElement) -> Self {
        self.badge = Some(badge.into_any_element());
        self
    }

    /// Label/value pairs in the values row.
    pub fn values<L: Into<SharedString>, V: Into<SharedString>>(
        mut self,
        pairs: impl IntoIterator<Item = (L, V)>,
    ) -> Self {
        self.values = pairs.into_iter().map(|(label, value)| (label.into(), value.into())).collect();
        self
    }

    /// Dims the card, e.g. a color outside this space's range.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn debug_selector(mut self, selector: impl FnOnce() -> String) -> Self {
        self.selector = Some(selector());
        self
    }

    pub(super) fn render(self, look: &Look, metrics: &LabMetrics) -> Div {
        let gray = |step| look.resolve_step(ScaleFamily::Gray, step).hsla();
        let icon = self.icon.map(|icon| {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .rounded_sm()
                .bg(gray(3))
                .border_1()
                .border_color(gray(6))
                .child(lucide_icon(icon, look.chrome().body_text, 16.))
                .into_any_element()
        });
        let swatch = icon
            .or(self.swatch)
            .map(|swatch| div().size(metrics.card_swatch).flex_shrink_0().rounded_sm().child(swatch));
        // Names and subtitles stay on one line and truncate, so a badge never wraps or overflows.
        let name = text(look, TextRole::Name, self.name).min_w_0().truncate();
        let name = match self.mark {
            Some(color) => hstack! {
                div().size(metrics.card_mark).flex_shrink_0().rounded_sm().bg(color),
                name,
            }
            .min_w_0()
            .items_center()
            .gap_1(),
            None => name,
        };
        let badge = self.badge.map(|badge| div().flex().flex_shrink_0().child(badge));
        let name_row = hstack! { name }.min_w_0().items_center().gap_2().children(badge);
        let heading = vstack! {}
            .min_w_0()
            .child(name_row)
            .children(self.subtitle.map(|subtitle| text(look, TextRole::Label, subtitle).truncate()));
        let values = (!self.values.is_empty()).then(|| readout(look, self.values));
        let selector = self.selector;
        card_surface(div().flex().flex_col(), look)
            .flex_1()
            .min_w(metrics.card_min_width)
            .flex_basis(metrics.card_min_width)
            .child(hstack! {}.children(swatch).child(heading).items_center().gap_3())
            .children(values)
            .when(self.disabled, |card| card.opacity(0.45))
            .when_some(selector, |card, selector| card.debug_selector(|| selector))
    }
}
