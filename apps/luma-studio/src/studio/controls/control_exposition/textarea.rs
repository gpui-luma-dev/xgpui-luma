use xgpui_luma::controls::textarea::TextAreaEventExt;
use xgpui_luma::controls::checkbox::CheckboxEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, FontWeight, Render, SharedString, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::checkbox::{Checkbox, CheckboxEvent};
use xgpui_luma::controls::button::Button;
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma::controls::textarea::{TextArea, TextAreaEvent, Validator};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::input_theme_inspectors::TextAreaThemeInspector;
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::template::render_control_exposition_card;
use super::inspector::{TextAreaInspectorAdapter, TEXTAREA_INSPECTOR_SPEC};

pub struct TextAreaControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<TextAreaExpositionLeftPane>,
    theme_inspector: Entity<TextAreaThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct TextAreaExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    text_area: Entity<TextArea>,
    required_preview: Entity<TextArea>,
    event_stream: Entity<ControlEventStream>,
    set_sample_button: Entity<Button>,
    clear_button: Entity<Button>,
    enabled_checkbox: Checkbox,
    clean_on_escape_checkbox: Checkbox,
    validation_checkbox: Checkbox,
    enabled: bool,
    clean_on_escape: bool,
    strict_validation: bool,
}

impl TextAreaExpositionLeftPane {
    fn current_validator(&self) -> Option<Validator> {
        if !self.strict_validation {
            return None;
        }
        Some(Arc::new(|value: &str| {
            value.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == ' ' || ch == '\n')
        }))
    }

    fn sync_text_area_settings(&mut self, cx: &mut Context<Self>) {
        let enabled = self.enabled;
        let clean_on_escape = self.clean_on_escape;
        let validator = self.current_validator();
        self.text_area.update(cx, move |text_area, cx| {
            text_area.set_enabled(enabled, cx);
            text_area.set_clean_on_escape(clean_on_escape, cx);
            text_area.set_validator(validator, cx);
        });
    }

    fn handle_option_changed(&mut self, option: TextAreaOption, event: &CheckboxEvent, cx: &mut Context<Self>) {
        let CheckboxEvent::Change { checked } = event else {
            return;
        };
        match option {
            TextAreaOption::Enabled => self.enabled = *checked,
            TextAreaOption::CleanOnEscape => self.clean_on_escape = *checked,
            TextAreaOption::StrictValidation => self.strict_validation = *checked,
        }
        self.sync_text_area_settings(cx);
        cx.notify();
    }

    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.text_area.update(cx, |_, cx| cx.notify());
        self.required_preview.update(cx, |_, cx| cx.notify());
        self.set_sample_button.update(cx, |_, cx| cx.notify());
        self.clear_button.update(cx, |_, cx| cx.notify());
        self.enabled_checkbox.update(cx, |_, cx| cx.notify());
        self.clean_on_escape_checkbox.update(cx, |_, cx| cx.notify());
        self.validation_checkbox.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for TextAreaExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let foreground = self.look.token_color("foreground").unwrap_or_else(|_| self.look.chrome().body_text);
            let section_heading = self.look.typography_scale(ShadcnTextSize::Sm);
            let preview = div()
                .w_full()
                .max_w(px(620.0))
                .flex()
                .flex_col()
                .gap(px(16.0))
                .text_color(foreground)
                .child(self.text_area.clone())
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(
                            div()
                                .typography_style(section_heading)
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Required / Invalid"),
                        )
                        .child(self.required_preview.clone())
                        .child(
                            div()
                                .text_color(self.look.mode_tokens().palette.destructive_background)
                                .child("Please enter a message."),
                        ),
                )
                .child(div().flex().flex_wrap().gap(px(8.0)).children([
                    self.set_sample_button.clone().into_any_element(),
                    self.clear_button.clone().into_any_element(),
                ]))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap(px(14.0))
                        .child(self.enabled_checkbox.clone())
                        .child(self.clean_on_escape_checkbox.clone())
                        .child(self.validation_checkbox.clone()),
                )
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-textarea-left-pane")
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

impl TextAreaControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("textarea").expect("textarea catalog entry");
        let text_area = shadcn::TextArea::new("controls-doc-textarea")
            .look(look.as_ref())
            .placeholder("Write a multiline message")
            .full_width(true)
            .rows(6)
            .clean_on_escape(true)
            .select_all_on_tab_focus(true)
            .spawn(cx);
        let required_preview = shadcn::TextArea::new("controls-doc-textarea-required-preview")
            .look(look.as_ref())
            .placeholder("Required message")
            .full_width(true)
            .rows(3)
            .validator(Arc::new(|value: &str| !value.is_empty()))
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-textarea-event-log",
                "Edit the text area; TextAreaEvent variants appear below.",
            )
        });
        let set_sample_button = shadcn::Button::new("controls-textarea-set-sample")
            .look(look.as_ref())
            .secondary()
            .label("Set Sample")
            .spawn(cx);
        let clear_button = shadcn::Button::new("controls-textarea-clear")
            .look(look.as_ref())
            .secondary()
            .label("Clear")
            .spawn(cx);
        let enabled_checkbox = shadcn::Checkbox::new("controls-textarea-enabled")
            .look(look.as_ref())
            .primary()
            .with_data(true)
            .content(|_, _| div().child("Enabled").into_any_element())
            .spawn(cx);
        let clean_on_escape_checkbox = shadcn::Checkbox::new("controls-textarea-clean-on-escape")
            .look(look.as_ref())
            .primary()
            .with_data(true)
            .content(|_, _| div().child("Escape clears").into_any_element())
            .spawn(cx);
        let validation_checkbox = shadcn::Checkbox::new("controls-textarea-validation")
            .look(look.as_ref())
            .primary()
            .with_data(false)
            .content(|_, _| div().child("Strict validation").into_any_element())
            .spawn(cx);

        let left_pane = cx.new(|_| TextAreaExpositionLeftPane {
            look: look.clone(),
            entry,
            text_area: text_area.clone(),
            required_preview,
            event_stream: event_stream.clone(),
            set_sample_button: set_sample_button.clone(),
            clear_button: clear_button.clone(),
            enabled_checkbox: enabled_checkbox.clone(),
            clean_on_escape_checkbox: clean_on_escape_checkbox.clone(),
            validation_checkbox: validation_checkbox.clone(),
            enabled: true,
            clean_on_escape: true,
            strict_validation: false,
        });

        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-textarea-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &TEXTAREA_INSPECTOR_SPEC,
            TextAreaInspectorAdapter::shared(),
        );

        let subscriptions = {
            text_area.clone().on_event(cx, {
                let event_stream = event_stream.downgrade();
                move |_, event: &TextAreaEvent, cx| {
                    let Some(event_stream) = event_stream.upgrade() else {
                        return;
                    };
                    if let Some(line) = format_textarea_event(event) {
                        event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                    }
                }
            });
            Vec::new()
        };
        set_sample_button.clone().on_click(cx, {
            let text_area = text_area.downgrade();
            move |_, cx| {
let Some(text_area) = text_area.upgrade() else { return; };


                let value = SharedString::from(
                    "Short line\nSeveral hard lines\nA very long line that should exceed the visible viewport width before wrapping is implemented\nMultibyte smoke test: café 東京\nEnough lines\nTo make vertical space\nFeel like a textarea",
                );
                text_area.update(cx, |area, cx| area.set_value(value.as_ref(), cx));
            }
        });
        clear_button.clone().on_click(cx, {
            let text_area = text_area.downgrade();
            move |_, cx| {
                let Some(text_area) = text_area.upgrade() else {
                    return;
                };

                text_area.update(cx, |area, cx| area.set_value("", cx));
            }
        });
        for (checkbox, option) in [
            (enabled_checkbox.clone(), TextAreaOption::Enabled),
            (clean_on_escape_checkbox.clone(), TextAreaOption::CleanOnEscape),
            (validation_checkbox.clone(), TextAreaOption::StrictValidation),
        ] {
            let left_pane = left_pane.downgrade();
            checkbox.clone().on_event(cx, move |_, event: &CheckboxEvent, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };
                left_pane.update(cx, |pane, cx| pane.handle_option_changed(option, event, cx));
            });
        }

        Self { look, entry, left_pane, theme_inspector, inspector_split, _subscriptions: subscriptions }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn fills_viewport(&self) -> bool {
        true
    }

    pub fn request_layout_refresh(&mut self, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.request_layout_refresh(cx));
    }

    pub fn set_viewport_size(&mut self, size: gpui::Size<gpui::Pixels>, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.set_viewport_size(size, cx));
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        sync_viewport_inspector(look, &self.theme_inspector, &self.inspector_split, cx);
        cx.notify();
    }
}

#[derive(Clone, Copy)]
enum TextAreaOption {
    Enabled,
    CleanOnEscape,
    StrictValidation,
}

impl Render for TextAreaControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-textarea-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn format_textarea_event(event: &TextAreaEvent) -> Option<String> {
    match event {
        TextAreaEvent::Change { value } => {
            Some(format!("TextAreaEvent::Change {{ value: \"{}\" }}", value.replace('\n', "\\n")))
        }
        TextAreaEvent::FocusChanged { focused } => {
            Some(format!("TextAreaEvent::FocusChanged {{ focused: {focused} }}"))
        }
        TextAreaEvent::EnabledChanged { enabled } => {
            Some(format!("TextAreaEvent::EnabledChanged {{ enabled: {enabled} }}"))
        }
        _ => None,
    }
}
