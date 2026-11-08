use std::sync::Arc;

use gpui::{AnyElement, IntoElement, RenderOnce, Window, div, prelude::*, px, relative};
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::control_exposition::template::controls_mono_font;

#[derive(IntoElement)]
pub struct InspectorSection {
    look: Arc<ShadcnLook>,
    rows: Vec<AnyElement>,
}

impl InspectorSection {
    pub fn new(look: &Arc<ShadcnLook>, rows: Vec<AnyElement>) -> Self {
        Self { look: look.clone(), rows }
    }
}

impl RenderOnce for InspectorSection {
    fn render(self, _window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        let chrome = self.look.chrome();
        div()
            .w_full()
            .min_w(px(0.0))
            .border_1()
            .border_color(chrome.border)
            .rounded(px(6.0))
            .overflow_hidden()
            .children(self.rows)
    }
}

#[derive(IntoElement)]
pub struct InspectorRow {
    look: Arc<ShadcnLook>,
    label: AnyElement,
    value: AnyElement,
    source: String,
    value_width: f32,
    show_separator: bool,
}

impl InspectorRow {
    pub fn new(
        look: &Arc<ShadcnLook>,
        label: AnyElement,
        value: AnyElement,
        source: String,
        value_width: f32,
        show_separator: bool,
    ) -> Self {
        Self { look: look.clone(), label, value, source, value_width, show_separator }
    }
}

impl RenderOnce for InspectorRow {
    fn render(self, _window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        let chrome = self.look.chrome();
        let caption = &self.look.mode_tokens().typography.text.caption;
        let mono_font = controls_mono_font();

        div()
            .min_h(px(34.0))
            .min_w(px(0.0))
            .flex()
            .flex_wrap()
            .items_start()
            .gap(px(4.0))
            .when(self.show_separator, |row| row.border_t_1().border_color(chrome.border))
            .px(px(9.0))
            .py(px(6.0))
            .child(div().w(relative(0.28)).min_w(px(180.0)).flex_shrink_0().mr(px(6.0)).child(self.label))
            .child(
                div()
                    .w(px(self.value_width))
                    .flex_shrink_0()
                    .mr(px(6.0))
                    .font_family(mono_font.clone())
                    .text_size(px(caption.size))
                    .line_height(px(caption.line_height))
                    .text_color(chrome.muted_text)
                    .child(self.value),
            )
            .child(
                div()
                    .flex_none()
                    .whitespace_nowrap()
                    .font_family(mono_font)
                    .text_size(px(caption.size))
                    .line_height(px(caption.line_height))
                    .text_color(chrome.muted_text)
                    .child(self.source),
            )
    }
}
