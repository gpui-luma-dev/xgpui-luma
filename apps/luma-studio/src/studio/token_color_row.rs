use gpui::{IntoElement, SharedString, div, prelude::*, px};
use xgpui_luma::theme::{LumaChrome, LumaTextStyle};

pub(crate) const TOKEN_SWATCH_SIZE: f32 = 30.0;
const TOKEN_SWATCH_RADIUS: f32 = 4.0;
/// Wide enough for the compact color labels used in the Colors sidebar.
const TOKEN_LABEL_WIDTH: f32 = 88.0;
/// Standard HSLA field width, capped so token rows still fit in a narrow sidebar.
const TOKEN_EDITOR_WIDTH: f32 = 320.0;

// future: consider hstack_align! macro — row-level gap/align plus per-slot flex (shrink-0 swatch,
// fixed-width wrapping label, flex-1 min_w(0) control) without hand-built wrapper divs.

/// Single-row token editor: swatch, label, and HSLA field.
pub(crate) fn token_color_row(
    label: impl Into<SharedString>,
    swatch: impl IntoElement,
    control: impl IntoElement,
    chrome: &LumaChrome,
    label_typography: &LumaTextStyle,
) -> impl IntoElement {
    let label = label.into();
    div()
        .flex()
        .w_full()
        .gap(px(8.0))
        .items_center()
        .child(div().size(px(TOKEN_SWATCH_SIZE)).flex_shrink_0().rounded(px(TOKEN_SWATCH_RADIUS)).child(swatch))
        .child(
            div()
                .flex_shrink_0()
                .w(px(TOKEN_LABEL_WIDTH))
                .whitespace_normal()
                .text_size(px(label_typography.size))
                .line_height(px(label_typography.line_height))
                .font_weight(label_typography.weight)
                .text_color(chrome.muted_text)
                .child(label),
        )
        .child(
            div()
                .flex()
                .items_center()
                .flex_1()
                .min_w(px(0.0))
                .max_w(px(TOKEN_EDITOR_WIDTH))
                .child(div().w_full().min_w(px(0.0)).child(control)),
        )
}
