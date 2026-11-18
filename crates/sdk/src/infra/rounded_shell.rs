//! Rounded bordered panel helpers for GPUI.
//!
//! GPUI applies `corner_radii` when painting an element's own background and border, but
//! [`overflow_hidden`](gpui::Styled::overflow_hidden) clips descendants with a **rectangular**
//! mask only. A child with a different [`bg`](gpui::Styled::bg) must use matching radii on
//! the same node (e.g. [`rounded_top`] for a header strip inside a rounded shell).

use gpui::{Div, Hsla, Stateful, div, px, prelude::*};

/// Rounds the top-left and top-right corners on the same node as `bg()` (table headers, toolbars).
///
/// Chain before `.bg(...)` / `.child(...)` on a `div()` builder.
pub fn rounded_top(element: Div, radius: f32) -> Div {
    element.rounded_tl(px(radius)).rounded_tr(px(radius))
}

/// Rounds the bottom-left and bottom-right corners on the same node as `bg()` (panel footers).
pub fn rounded_bottom(element: Div, radius: f32) -> Div {
    element.rounded_bl(px(radius)).rounded_br(px(radius))
}

/// Bordered panel frame: `rounded` + `border` + rectangular child clip (no fill).
///
/// Use when child sections own their own `bg()` (e.g. table header + body).
pub fn rounded_bordered_frame(id: impl Into<gpui::ElementId>, radius: f32, border: Hsla) -> Stateful<Div> {
    div()
        .id(id)
        .relative()
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .rounded(px(radius))
        .border(px(crate::theme::MetricTokens::default().border_width))
        .border_color(border)
        .overflow_hidden()
}

/// Bordered panel shell: `rounded` + `bg` + `border` on one element, then rectangular child clip.
pub fn rounded_bordered_panel(
    id: impl Into<gpui::ElementId>,
    radius: f32,
    background: Hsla,
    border: Hsla,
) -> Stateful<Div> {
    rounded_bordered_frame(id, radius, border).bg(background)
}
