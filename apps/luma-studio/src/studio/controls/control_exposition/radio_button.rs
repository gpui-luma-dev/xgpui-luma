//! Radio button control exposition — live primary/secondary previews and event stream.

use xgpui_luma::controls::radio_button::RadioButtonEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma::controls::radio_button::{RadioButton, RadioButtonEvent};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::inspector::StateMotionReport;
use super::radio_button_theme_inspector::RadioButtonThemeInspector;
use super::template::render_control_exposition_card;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RadioButtonVariant {
    Primary,
    Secondary,
    ContentOnly,
}

impl RadioButtonVariant {
    fn label(self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::Secondary => "Secondary",
            Self::ContentOnly => "Content Only",
        }
    }
}

pub struct RadioButtonControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    secondary_radio: RadioButton,
    primary_radio: RadioButton,
    content_only_radio: RadioButton,
    event_stream: Entity<ControlEventStream>,
    left_pane: Entity<RadioButtonExpositionLeftPane>,
    theme_inspector: Entity<RadioButtonThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct RadioButtonExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    secondary_radio: RadioButton,
    primary_radio: RadioButton,
    content_only_radio: RadioButton,
    event_stream: Entity<ControlEventStream>,
}

impl RadioButtonExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for radio in [&self.secondary_radio, &self.primary_radio, &self.content_only_radio] {
            radio.update(cx, |_, cx| cx.notify());
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for RadioButtonExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(16.0))
                .child(
                    div()
                        .w_full()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .justify_start()
                        .gap(px(12.0))
                        .child(self.primary_radio.clone())
                        .child(self.secondary_radio.clone())
                        .child(self.content_only_radio.clone()),
                )
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-radio-button-left-pane")
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

impl RadioButtonControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("radio-button").expect("radio-button catalog entry");
        let secondary_radio = shadcn::Radio::new("controls-doc-radio-secondary")
            .look(look.as_ref())
            .secondary()
            .with_data(false)
            .content(|_, _| div().child("Secondary").into_any_element())
            .spawn(cx);
        let primary_radio = shadcn::Radio::new("controls-doc-radio-primary")
            .look(look.as_ref())
            .primary()
            .with_data(false)
            .content(|_, _| div().child("Primary").into_any_element())
            .spawn(cx);
        let content_only_radio = shadcn::Radio::new("controls-doc-radio-content-only")
            .look(look.as_ref())
            .content_only()
            .with_data(false)
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-radio-event-log",
                "Select the preview radio buttons; all emitted RadioButtonEvent variants appear in the stream below.",
            )
        });

        let left_pane = cx.new(|_| RadioButtonExpositionLeftPane {
            look: look.clone(),
            entry,
            secondary_radio: secondary_radio.clone(),
            primary_radio: primary_radio.clone(),
            content_only_radio: content_only_radio.clone(),
            event_stream: event_stream.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-radio-button-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &super::inspector::RADIO_BUTTON_INSPECTOR_SPEC,
            super::inspector::RadioButtonInspectorAdapter::shared(),
        );
        theme_inspector.update(cx, |inspector, cx| {
            inspector.set_motion_report(
                Arc::new(StateMotionReport::new(
                    "State",
                    [
                        ("Primary", primary_radio.clone()),
                        ("Secondary", secondary_radio.clone()),
                        ("Content only", content_only_radio.clone()),
                    ],
                    |radio| radio.motion_enabled(),
                    |radio| radio.motion_profile().state.clone(),
                    |look| look.motion_defaults().radio.state.clone(),
                )),
                cx,
            );
        });

        let mut subscriptions = Vec::new();
        subscriptions.extend(subscribe_radio(
            &secondary_radio,
            RadioButtonVariant::Secondary,
            "controls-doc-radio-secondary",
            event_stream.clone(),
            cx,
        ));
        subscriptions.extend(subscribe_radio(
            &primary_radio,
            RadioButtonVariant::Primary,
            "controls-doc-radio-primary",
            event_stream.clone(),
            cx,
        ));
        subscriptions.extend(subscribe_radio(
            &content_only_radio,
            RadioButtonVariant::ContentOnly,
            "controls-doc-radio-content-only",
            event_stream.clone(),
            cx,
        ));
        secondary_radio.clone().on_event(cx, |this, event, cx| {
            if matches!(event, RadioButtonEvent::Change { selected: true }) {
                this.primary_radio.update(cx, |radio, cx| radio.set_data(false, cx));
                this.content_only_radio.update(cx, |radio, cx| radio.set_data(false, cx));
            }
        });
        primary_radio.clone().on_event(cx, |this, event, cx| {
            if matches!(event, RadioButtonEvent::Change { selected: true }) {
                this.secondary_radio.update(cx, |radio, cx| radio.set_data(false, cx));
                this.content_only_radio.update(cx, |radio, cx| radio.set_data(false, cx));
            }
        });
        content_only_radio.clone().on_event(cx, |this, event, cx| {
            if matches!(event, RadioButtonEvent::Change { selected: true }) {
                this.secondary_radio.update(cx, |radio, cx| radio.set_data(false, cx));
                this.primary_radio.update(cx, |radio, cx| radio.set_data(false, cx));
            }
        });

        Self {
            look,
            entry,
            secondary_radio,
            primary_radio,
            content_only_radio,
            event_stream,
            left_pane,
            theme_inspector,
            inspector_split,
            _subscriptions: subscriptions,
        }
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
        for radio in [&self.secondary_radio, &self.primary_radio, &self.content_only_radio] {
            radio.update(cx, |_, cx| cx.notify());
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look.clone(), cx));
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        sync_viewport_inspector(look, &self.theme_inspector, &self.inspector_split, cx);
        cx.notify();
    }
}

impl Render for RadioButtonControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let inspector_split = self.inspector_split.clone();
        with_look(&self.look, || {
            div()
                .id("controls-doc-radio-button-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(inspector_split)
        })
    }
}

fn subscribe_radio(
    radio: &RadioButton,
    variant: RadioButtonVariant,
    radio_id: &'static str,
    event_stream: Entity<ControlEventStream>,
    cx: &mut Context<RadioButtonControlExposition>,
) -> Vec<Subscription> {
    {
        radio.clone().on_event(cx, move |_, event: &RadioButtonEvent, cx| {
            let line = format!("{} - {} (\"{radio_id}\")", format_radio_event(event), variant.label());
            event_stream.update(cx, |stream, cx| {
                stream.append_line(&line, cx);
                cx.notify();
            });
        });
        Vec::new()
    }
}

fn format_radio_event(event: &RadioButtonEvent) -> String {
    match event {
        RadioButtonEvent::Change { selected } => format!("RadioButtonEvent::Change {{ selected: {selected} }}"),
        RadioButtonEvent::FocusChanged { focused } => {
            format!("RadioButtonEvent::FocusChanged {{ focused: {focused} }}")
        }
        RadioButtonEvent::EnabledChanged { enabled } => {
            format!("RadioButtonEvent::EnabledChanged {{ enabled: {enabled} }}")
        }
        RadioButtonEvent::HoverChanged { hovered } => {
            format!("RadioButtonEvent::HoverChanged {{ hovered: {hovered} }}")
        }
        _ => "RadioButtonEvent::(unknown)".to_string(),
    }
}
