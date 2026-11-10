//! Legacy presentation helpers, kept for Italy and Shell Lab until they migrate.
//! New screens use [`crate::screen::LabScreen`]; every helper forwards there.
use gpui::{Div, ParentElement, SharedString, Styled, div};
use xgpui_luma_look_radix::Look;

use crate::screen::{self, TextRole};

pub(crate) use crate::screen::{card_surface as comparison_surface, container, control_row, page};

pub(crate) fn header(look: &Look, title: &'static str, subtitle: &'static str) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(screen::LabMetrics::STANDARD.header_gap)
        .child(screen::text(look, TextRole::Title, title))
        .child(caption(look, subtitle))
}

pub(crate) fn caption(look: &Look, text: impl Into<SharedString>) -> Div {
    screen::text(look, TextRole::Caption, text)
}

/// Apply the reference switch label typography to a labeled control.
pub(crate) fn control(look: &Look, element: Div) -> Div {
    screen::labeled_control(look, element)
}

pub(crate) fn panel(look: &Look) -> Div {
    screen::panel_surface(look)
}
