//! Lab sample layout: color-gallery card spacing with Radix colors, radii, and typography.
use gpui::{AnyElement, FontWeight, div, prelude::*, px};
use crate::theme::{ColorVizLookExt, Look, TextSize, TypographyExt};

pub(super) fn page(id: &'static str, content: impl IntoElement) -> AnyElement {
    div()
        .id(id)
        .size_full()
        .min_w_0()
        .min_h_0()
        .overflow_y_scroll()
        .p(px(16.))
        .child(content)
        .into_any_element()
}

pub(super) fn card(
    look: &Look,
    id: &'static str,
    title: &'static str,
    description: &'static str,
    width: f32,
    content: impl IntoElement,
) -> AnyElement {
    let chrome = look.chrome();
    div()
        .id(id)
        .debug_selector(move || id.into())
        .w(px(width))
        .max_w_full()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(14.))
        .rounded(px(look.metrics().radius.xl))
        .border_1()
        .border_color(chrome.border)
        .bg(chrome.panel_background)
        .p(px(18.))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(3.))
                .child(
                    div()
                        .typography_style(look.typography_scale(TextSize::Sm))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(chrome.title_text)
                        .child(title),
                )
                .child(
                    div()
                        .typography_style(look.typography_scale(TextSize::Xs))
                        .text_color(chrome.muted_text)
                        .child(description),
                ),
        )
        .child(div().w_full().min_w_0().child(content))
        .into_any_element()
}
