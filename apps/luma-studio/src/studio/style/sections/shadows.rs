use gpui::{AnyElement, IntoElement, div, prelude::*, px};
use xgpui_luma::vstack;
use xgpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextSize};

use crate::studio::style::shared::shell::section_shell_with_width;

const SHADOW_TOKEN_KEYS: [&str; 8] = [
    "shadow",
    "shadow-2xs",
    "shadow-xs",
    "shadow-sm",
    "shadow-md",
    "shadow-lg",
    "shadow-xl",
    "shadow-2xl",
];

pub(crate) fn render_shadow_tokens_section(look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();
    let caption_style = look.typography_scale(ShadcnTextSize::Xs);
    let detail_style = look.typography_scale(ShadcnTextSize::Sm);
    let mode_tokens = look.mode_tokens();
    let catalog = &mode_tokens.catalog;

    section_shell_with_width(
        860.0,
        "Shadow Tokens",
        "Resolved shadow ladder values from the active theme.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        vstack! {
            gap=8;
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .children(SHADOW_TOKEN_KEYS.into_iter().map(|token| {
                    let raw = catalog.get(token).unwrap_or("unresolved");
                    let parsed = look.parse_shadow_token(token);
                    let detail = match &parsed {
                        Ok(layers) => format!("{} layer{}", layers.len(), if layers.len() == 1 { "" } else { "s" }),
                        Err(error) => format!("parse error: {error}"),
                    };
                    let mut preview = div()
                        .flex_none()
                        .size(px(72.0))
                        .bg(chrome.panel_background)
                        .border_1()
                        .border_color(chrome.border)
                        .rounded(px(4.0));
                    if let Ok(layers) = &parsed
                        && !layers.is_empty()
                    {
                        preview = preview.shadow(layers.clone());
                    }
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.0))
                        .border_1()
                        .border_color(chrome.border)
                        .rounded(px(6.0))
                        .p(px(8.0))
                        .child(preview)
                        .child(
                            div()
                                .min_w(px(0.0))
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .child(
                                    div()
                                        .font_family("Monaco")
                                        .typography_style(detail_style)
                                        .text_color(chrome.title_text)
                                        .child(format!("--{token}")),
                                )
                                .child(
                                    div()
                                        .font_family("Monaco")
                                        .typography_style(caption_style)
                                        .text_color(chrome.body_text)
                                        .child(detail),
                                )
                                .child(
                                    div()
                                        .font_family("Monaco")
                                        .typography_style(caption_style)
                                        .text_color(chrome.muted_text)
                                        .child(raw.to_string()),
                                ),
                        )
                        .into_any_element()
                }))
        }
        .w_full()
        .into_any_element(),
    )
}
