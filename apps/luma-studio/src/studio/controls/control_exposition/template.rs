use gpui::{AnyElement, FontWeight, IntoElement, div, prelude::*, px};
use xgpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextSize};

use super::model::ControlExpositionLayout;
use crate::studio::controls::catalog::ControlDocEntry;
use crate::studio::doc_shell::render_section_heading_anchor_with_order_options;

const PREVIEW_PAD: f32 = 20.0;
const SNIPPET_PAD: f32 = 14.0;

pub(crate) fn controls_mono_font() -> gpui::SharedString {
    #[cfg(target_os = "macos")]
    {
        "Menlo".into()
    }
    #[cfg(target_os = "windows")]
    {
        "Consolas".into()
    }
    #[cfg(target_os = "linux")]
    {
        "DejaVu Sans Mono".into()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        "monospace".into()
    }
}

pub(crate) fn render_control_exposition_card(
    look: &ShadcnLook,
    entry: ControlDocEntry,
    preview: AnyElement,
    between_preview_and_snippet: Option<AnyElement>,
    layout: ControlExpositionLayout,
) -> AnyElement {
    let chrome = look.chrome();
    let card_bg = look.token_color("card").unwrap_or(chrome.panel_background);
    let card_id = entry.id;
    let borderless = layout.borderless;

    let mut card = div().id(format!("controls-doc-card-{card_id}")).w_full().min_w(px(0.0)).flex().flex_col();

    if borderless {
        card = card.overflow_hidden();
    } else {
        card = card.rounded(px(12.0)).border_1().border_color(chrome.border).bg(card_bg).overflow_hidden();
    }

    if !layout.skip_heading {
        card = card.child(div().w_full().flex().flex_col().gap(px(10.0)).p(px(16.0)).child(
            render_section_heading_anchor_with_order_options(
                entry.title,
                entry.description,
                entry.section_order,
                chrome.title_text,
                chrome.muted_text,
                chrome.border,
                false,
            ),
        ));
    }

    card.child(render_card_section(preview, borderless, chrome, PREVIEW_PAD))
        .when_some(between_preview_and_snippet, |card, section| {
            card.child(render_card_section(section, borderless, chrome, PREVIEW_PAD))
        })
        .when(!layout.skip_snippet, |card| {
            card.child(render_card_section(render_snippet_block(look, entry.snippet), borderless, chrome, SNIPPET_PAD))
        })
        .into_any_element()
}

fn render_card_section(
    content: AnyElement,
    borderless: bool,
    chrome: xgpui_luma::theme::LumaChrome,
    pad: f32,
) -> gpui::Div {
    if borderless {
        div()
            .w_full()
            .min_w(px(0.0))
            .px(px(16.0))
            .pb(px(12.0))
            .flex()
            .flex_col()
            .items_stretch()
            .child(content)
    } else {
        div()
            .w_full()
            .mx(px(16.0))
            .mb(px(12.0))
            .rounded(px(10.0))
            .border_1()
            .border_color(chrome.border)
            .bg(chrome.content_background)
            .p(px(pad))
            .child(content)
    }
}

fn render_snippet_block(look: &ShadcnLook, snippet: &str) -> AnyElement {
    let chrome = look.chrome();
    let title_style = look.typography_scale(ShadcnTextSize::Sm);
    let mono = controls_mono_font();

    div()
        .w_full()
        .flex()
        .flex_col()
        .items_start()
        .gap(px(6.0))
        .child(
            div()
                .typography_style(title_style)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(chrome.title_text)
                .child("Code Sample"),
        )
        .child(
            div()
                .w_full()
                .text_size(px(12.0))
                .line_height(px(18.0))
                .font_family(mono)
                .text_color(chrome.muted_text)
                .child(snippet.to_string()),
        )
        .into_any_element()
}
