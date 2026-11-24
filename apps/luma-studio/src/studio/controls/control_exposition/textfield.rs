//! TextField exposition — live preview and event stream.

use xgpui_luma::controls::textfield::TextFieldEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, FontWeight, Render, SharedString, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::button::Button;
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma::controls::textfield::{TextField, TextFieldEvent};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::model::{ControlExpositionLayout};
use super::template::render_control_exposition_card;

pub struct TextFieldControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    preview: TextField,
    event_stream: Entity<ControlEventStream>,
    left_pane: Entity<TextFieldExpositionLeftPane>,
    _subscriptions: Vec<Subscription>,
}

struct TextFieldExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    preview: TextField,
    required_preview: TextField,
    event_stream: Entity<ControlEventStream>,
    set_sample_button: Entity<Button>,
}

impl TextFieldExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.preview.update(cx, |_, cx| cx.notify());
        self.required_preview.update(cx, |_, cx| cx.notify());
        self.set_sample_button.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for TextFieldExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let foreground = self.look.token_color("foreground").unwrap_or_else(|_| self.look.chrome().body_text);
            let section_heading = self.look.typography_scale(ShadcnTextSize::Sm);
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(16.0))
                .text_color(foreground)
                .child(
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(
                            div()
                                .typography_style(section_heading)
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Default / Focusable"),
                        )
                        .child(self.preview.clone()),
                )
                .child(
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(
                            div()
                                .typography_style(section_heading)
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Required / Invalid"),
                        )
                        .child(self.required_preview.clone()),
                )
                .child(self.set_sample_button.clone())
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-textfield-left-pane")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .overflow_y_scroll()
                .child(render_control_exposition_card(
                    &self.look,
                    self.entry,
                    preview.into_any_element(),
                    None,
                    ControlExpositionLayout::BORDERLESS,
                ))
        })
    }
}

impl TextFieldControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("textfield").expect("textfield catalog entry");
        let preview = shadcn::TextField::new("controls-doc-textfield-preview")
            .look(look.as_ref())
            .placeholder("Email address")
            .full_width(true)
            .spawn(cx);
        let required_preview = shadcn::TextField::new("controls-doc-textfield-required-preview")
            .look(look.as_ref())
            .placeholder("Required input")
            .full_width(true)
            .validator(Arc::new(|value: &str| !value.is_empty()))
            .spawn(cx);
        let set_sample_button = shadcn::Button::new("controls-textfield-set-sample")
            .look(look.as_ref())
            .secondary()
            .label("Set Sample")
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-textfield-event-log",
                "Edit the preview field; all emitted TextFieldEvent variants appear in the stream below.",
            )
        });
        let left_pane = cx.new(|_| TextFieldExpositionLeftPane {
            look: look.clone(),
            entry,
            preview: preview.clone(),
            required_preview,
            event_stream: event_stream.clone(),
            set_sample_button: set_sample_button.clone(),
        });

        let subscriptions = {
            preview.clone().on_event(cx, {
                let event_stream = event_stream.downgrade();
                move |_, event: &TextFieldEvent, cx| {
                    let Some(event_stream) = event_stream.upgrade() else {
                        return;
                    };
                    let line = format_textfield_event(event);
                    event_stream.update(cx, |stream, cx| {
                        stream.append_line(&line, cx);
                        cx.notify();
                    });
                }
            });
            Vec::new()
        };
        set_sample_button.clone().on_click(cx, {
            let preview = preview.downgrade();
            move |_, cx| {
let Some(preview) = preview.upgrade() else { return; };



                let value = SharedString::from(
                    "TextField drag-selection sample: left edge and right edge. \
Whitespace and punctuation:   / .,;:!? (parentheses) [brackets] {braces}. \
This line is intentionally long enough to overflow a compact single-line field so dragging past either boundary is easy to reproduce.",
                );
                preview.update(cx, |field, cx| field.set_value(value.as_ref(), cx));
            }
        });

        Self { look, entry, preview, event_stream, left_pane, _subscriptions: subscriptions }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn fills_viewport(&self) -> bool {
        true
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.preview.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look.clone(), cx));
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        cx.notify();
    }
}

impl Render for TextFieldControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-textfield-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.left_pane.clone())
        })
    }
}

fn format_textfield_event(event: &TextFieldEvent) -> String {
    match event {
        TextFieldEvent::Change { value } => format!("TextFieldEvent::Change {{ value: \"{value}\" }}"),
        TextFieldEvent::Submit { value } => format!("TextFieldEvent::Submit {{ value: \"{value}\" }}"),
        TextFieldEvent::FocusChanged { focused } => {
            format!("TextFieldEvent::FocusChanged {{ focused: {focused} }}")
        }
        TextFieldEvent::EnabledChanged { enabled } => {
            format!("TextFieldEvent::EnabledChanged {{ enabled: {enabled} }}")
        }
        _ => "TextFieldEvent::(unknown)".to_string(),
    }
}
