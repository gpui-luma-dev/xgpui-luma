use std::sync::Arc;
use xgpui_luma::prelude::HasPresenter;

use gpui::{Context, Entity, IntoElement, Render, Window, div, prelude::*, px};
use xgpui_luma::{
    controls::{
        button::Button,
        tooltip::{Tooltip, TooltipRenderModel, bubble},
    },
    infra::attachments::TooltipEntityExt,
};
use xgpui_luma_look_shadcn::{self as shadcn, ShadcnLook};
use crate::studio::style::shared::shell::section_shell_with_width;

const SAMPLES: [(&str, &str, Option<&str>); 3] = [
    ("Basic help", "Create a new item", None),
    ("Shortcut hint", "Save changes", Some("⌘S")),
    (
        "Wrapping",
        "Use a longer explanation when a short label does not provide enough context for this action.",
        None,
    ),
];

pub(crate) struct TooltipPreview {
    look: Arc<ShadcnLook>,
    buttons: Vec<Entity<Button>>,
}

impl TooltipPreview {
    pub(crate) fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let buttons = SAMPLES
            .iter()
            .enumerate()
            .map(|(index, (_, text, hint))| {
                let mut tooltip = Tooltip::new(*text);
                if let Some(hint) = hint {
                    tooltip = tooltip.shortcut(*hint);
                }
                shadcn::Button::new(format!("style-tooltip-{index}"))
                    .look(&look)
                    .outline()
                    .label("Hover or focus")
                    .with_template_modifier(move |root, _| {
                        root.debug_selector(move || format!("style-tooltip-{index}"))
                    })
                    .spawn(cx)
                    .tooltip(tooltip, cx)
            })
            .collect();
        Self { look, buttons }
    }

    pub(crate) fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        *self = Self::new(cx, look);
        cx.notify();
    }
}

impl Render for TooltipPreview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let style = shadcn::tooltip_theme(&self.look).resolve();
        let samples = SAMPLES.iter().zip(&self.buttons).map(|((title, text, hint), button)| {
            let model = TooltipRenderModel {
                placement: style.placement,
                shortcut_gap: style.shortcut_gap,
                shortcut_font_size: style.shortcut_font_size,
                shortcut_opacity: style.shortcut_opacity,
                text: (*text).into(),
                shortcut: hint.map(Into::into),
                background: style.background,
                foreground: style.foreground,
                padding: style.padding,
                padding_y: style.padding_y,
                radius: style.radius,
                max_width: style.max_width,
                text_size: style.text_size,
                line_height: style.line_height,
                shadow: style.shadow.clone(),
            };
            div()
                .w(px(280.0))
                .max_w_full()
                .flex_none()
                .flex()
                .flex_col()
                .gap(px(16.0))
                .p(px(16.0))
                .border_1()
                .border_color(chrome.border)
                .rounded(px(8.0))
                .child(div().text_size(px(14.0)).font_weight(gpui::FontWeight::SEMIBOLD).child(*title))
                .child(div().min_h(px(110.0)).flex().items_center().justify_center().child(bubble(&model)))
                .child(div().flex().justify_center().child(button.clone()))
        });
        div().flex().flex_col().gap(px(16.0)).text_color(chrome.title_text)
            .child(div().flex().flex_wrap().gap(px(16.0)).children(samples))
            .child(div().text_size(px(12.0)).text_color(chrome.muted_text)
                .child("Foreground surface · background text · 12 px type · 8 × 4 px padding · 260 px maximum width. Colors follow the current theme; hint text is display only."))
            .into_any_element()
    }
}

pub(crate) fn render_tooltip_section(look: &ShadcnLook, preview: Entity<TooltipPreview>) -> gpui::AnyElement {
    let chrome = look.chrome();
    section_shell_with_width(
        960.0,
        "Tooltips",
        "Standard Shadcn help, shortcut hints and wrapping. Hover a button or Tab to it to open the live tooltip.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.content_background,
        div().child(preview).into_any_element(),
    )
}
