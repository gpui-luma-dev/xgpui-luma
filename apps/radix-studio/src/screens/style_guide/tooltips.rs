use gpui::{IntoElement, div, prelude::*, px};
use xgpui_luma::controls::tooltip::{TooltipRenderModel, bubble};
use xgpui_luma_look_radix::{self as radix, Look, SemanticRole};

const SAMPLES: [(&str, &str, Option<&str>); 3] = [
    ("Basic help", "Create a new item", None),
    ("Shortcut hint", "Save changes", Some("⌘S")),
    (
        "Wrapping",
        "Use a longer explanation when a short label does not provide enough context for this action.",
        None,
    ),
];

#[derive(Clone)]
pub(crate) struct TooltipExamples;

impl TooltipExamples {
    pub(crate) fn new<M: 'static>(_: &Look, _: &mut gpui::Context<M>) -> Self {
        Self
    }

    pub(crate) fn render(&self, look: &Look) -> gpui::AnyElement {
        let foreground = look.resolve_role(SemanticRole::Foreground).hsla();
        let border = look.resolve_role(SemanticRole::Border).hsla();
        let muted = look.resolve_role(SemanticRole::MutedForeground).hsla();
        let style = radix::tooltip_theme(look).resolve();
        let samples = SAMPLES.iter().map(|(title, text, hint)| {
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
                .border_color(border)
                .rounded(px(8.0))
                .child(div().text_size(px(14.0)).font_weight(gpui::FontWeight::SEMIBOLD).child(*title))
                .child(div().min_h(px(110.0)).flex().items_center().justify_center().child(bubble(&model)))
        });
        div().flex().flex_col().gap(px(16.0)).text_color(foreground)
            .child(div().flex().flex_wrap().gap(px(16.0)).children(samples))
            .child(div().text_size(px(12.0)).text_color(muted)
                .child("Foreground surface · background text · 12 px type · 8 × 4 px padding · 260 px maximum width. Colors follow the current theme; hint text is display only."))
            .into_any_element()
    }
}
