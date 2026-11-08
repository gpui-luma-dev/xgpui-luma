use gpui::{AnyElement, FontWeight, IntoElement, div, prelude::*, px};
use xgpui_luma_look_shadcn::ShadcnLook;
use xgpui_luma_look_shadcn::prelude::*;

pub(crate) fn render_sparse_catalog_callout(look: &ShadcnLook) -> Option<AnyElement> {
    if look.has_css_catalog() {
        return None;
    }

    let chrome = look.chrome();
    let title = "Native default theme — sparse CSS catalog";
    let body = "This view uses the active look typography and token mappings. The native default theme has no CSS catalog, so cross-reference data is limited. SDK controls still resolve colors from the embedded palette. Pick a tweakcn theme in the sidebar for full catalog-backed typography context.";

    Some(render_callout(
        title,
        body,
        chrome.border,
        chrome.panel_background,
        chrome.title_text,
        chrome.body_text,
    ))
}

pub(crate) fn render_callout(
    title: &str,
    body: &str,
    border: gpui::Hsla,
    background: gpui::Hsla,
    title_color: gpui::Hsla,
    body_color: gpui::Hsla,
) -> AnyElement {
    div()
        .w_full()
        .max_w(px(860.0))
        .flex()
        .flex_col()
        .gap(px(6.0))
        .border_1()
        .border_color(border)
        .rounded(px(10.0))
        .bg(background)
        .p(px(14.0))
        .child(
            div()
                .typography_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(title_color)
                .child(title.to_string()),
        )
        .child(div().typography_xs().text_color(body_color).child(body.to_string()))
        .into_any_element()
}
