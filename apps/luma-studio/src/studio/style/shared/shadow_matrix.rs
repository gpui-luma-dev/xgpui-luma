use gpui::{AnyElement, IntoElement, SharedString, div, prelude::*, px};
use xgpui_luma_look_shadcn::{ShadcnLook, ShadcnShadow};

#[derive(Clone, Copy)]
pub(crate) enum ShadowPreviewShape {
    Choice,
    Selector,
    TextControl,
}

pub(crate) fn render_shadow_token_matrix(look: &ShadcnLook, shape: ShadowPreviewShape) -> AnyElement {
    const SHADOWS: [(ShadcnShadow, &str); 9] = [
        (ShadcnShadow::None, "none"),
        (ShadcnShadow::TwoXs, "2xs"),
        (ShadcnShadow::Xs, "xs"),
        (ShadcnShadow::Sm, "sm"),
        (ShadcnShadow::Default, "default"),
        (ShadcnShadow::Md, "md"),
        (ShadcnShadow::Lg, "lg"),
        (ShadcnShadow::Xl, "xl"),
        (ShadcnShadow::TwoXl, "2xl"),
    ];
    let chrome = look.chrome();

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(12.0))
        .children(SHADOWS.into_iter().map(|(shadow, label)| {
            let mut preview = div()
                .id(SharedString::from(format!("shadow-preview-{label}")))
                .bg(chrome.panel_background)
                .border_1()
                .border_color(chrome.border)
                .shadow(look.shadow(shadow));
            preview = match shape {
                ShadowPreviewShape::Choice => preview.size(px(24.0)).rounded(px(4.0)),
                ShadowPreviewShape::Selector | ShadowPreviewShape::TextControl => {
                    preview.w(px(180.0)).h(px(36.0)).rounded(px(6.0))
                }
            };

            div()
                .w_full()
                .flex()
                .items_center()
                .gap(px(24.0))
                .child(div().w(px(100.0)).text_sm().text_color(chrome.muted_text).child(format!("shadow-{label}")))
                .child(preview)
                .into_any_element()
        }))
        .into_any_element()
}
