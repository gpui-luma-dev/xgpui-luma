//! Variant / comparison card.
use gpui::{
    AnyElement, Div, Hsla, InteractiveElement, IntoElement, ParentElement, SharedString, Styled, div,
    prelude::FluentBuilder,
};
use lucide_svg_static::Icon;
use xgpui_luma::{hstack, infra::icon::lucide_icon, vstack};
use xgpui_luma_look_radix::Look;

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
    body: Option<AnyElement>,
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
            body: None,
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

    /// A line icon in place of the swatch, for status cards: plain, `card_icon` sized, foreground.
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

    /// Free content between the heading and the values row, e.g. a sample line.
    pub fn body(mut self, element: impl IntoElement) -> Self {
        self.body = Some(element.into_any_element());
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
        let disabled = self.disabled;
        let selector = self.selector.clone();
        card_surface(self.contents(look), look)
            .flex_1()
            .min_w(metrics.card_min_width)
            .flex_basis(metrics.card_min_width)
            .when(disabled, |card| card.opacity(0.45))
            .when_some(selector, |card, selector| card.debug_selector(|| selector))
    }

    /// The card's content without its surface, for hosts that draw the surface themselves
    /// (e.g. `card_button`, where the card is a Luma button).
    pub fn contents(self, look: &Look) -> Div {
        let metrics = LabMetrics::STANDARD;
        // A status icon is a plain line icon, not a box, so it never reads as a button.
        let icon = self.icon.map(|icon| {
            div()
                .size(metrics.card_icon)
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .child(lucide_icon(icon, look.chrome().body_text, metrics.card_icon.as_f32()))
        });
        let swatch = icon.or_else(|| {
            self.swatch.map(|swatch| div().size(metrics.card_swatch).flex_shrink_0().rounded_sm().child(swatch))
        });
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
        div()
            .flex()
            .flex_col()
            .gap_3()
            .min_w_0()
            .child(hstack! {}.children(swatch).child(heading).items_center().gap_3())
            .children(self.body)
            .children(values)
    }
}
