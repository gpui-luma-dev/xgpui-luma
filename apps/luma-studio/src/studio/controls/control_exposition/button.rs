//! Reference control exposition template — copy this module when adding a new control doc.

use xgpui_luma::controls::switch::SwitchEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::button::{Button, ButtonEvent};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma::controls::switch::{Switch};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::button_theme_inspector::ButtonThemeInspector;
use super::event_stream::ControlEventStream;
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::template::render_control_exposition_card;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ButtonVariant {
    Primary,
    Secondary,
    Outline,
    Ghost,
}

impl ButtonVariant {
    fn label(self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::Secondary => "Secondary",
            Self::Outline => "Outline",
            Self::Ghost => "Ghost",
        }
    }
}

struct ButtonExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    primary: Entity<Button>,
    secondary: Entity<Button>,
    outline: Entity<Button>,
    ghost: Entity<Button>,
    enabled_switch: Switch,
    event_stream: Entity<ControlEventStream>,
}

impl ButtonExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for button in [&self.primary, &self.secondary, &self.outline, &self.ghost] {
            button.update(cx, |_, cx| cx.notify());
        }
        self.enabled_switch.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for ButtonExpositionLeftPane {
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
                        .child(self.enabled_switch.clone())
                        .child(self.primary.clone())
                        .child(self.secondary.clone())
                        .child(self.outline.clone())
                        .child(self.ghost.clone()),
                )
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-button-left-pane")
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

pub struct ButtonControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    primary: Entity<Button>,
    secondary: Entity<Button>,
    outline: Entity<Button>,
    ghost: Entity<Button>,
    enabled_switch: Switch,
    buttons_enabled: bool,
    event_stream: Entity<ControlEventStream>,
    left_pane: Entity<ButtonExpositionLeftPane>,
    theme_inspector: Entity<ButtonThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

impl ButtonControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("button").expect("button catalog entry");
        let primary = shadcn::Button::new("controls-doc-button-primary")
            .look(look.as_ref())
            .primary()
            .label("Primary")
            .spawn(cx);
        let secondary = shadcn::Button::new("controls-doc-button-secondary")
            .look(look.as_ref())
            .secondary()
            .label("Secondary")
            .spawn(cx);
        let outline = shadcn::Button::new("controls-doc-button-outline")
            .look(look.as_ref())
            .outline()
            .label("Outline")
            .spawn(cx);
        let ghost = shadcn::Button::new("controls-doc-button-ghost")
            .look(look.as_ref())
            .ghost()
            .label("Ghost")
            .spawn(cx);
        let enabled_switch = shadcn::Switch::new("controls-doc-button-enabled-switch")
            .look(look.as_ref())
            .primary()
            .with_data(true)
            .content(|_, _| div().child("Enabled").into_any_element())
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-button-event-log",
                "Interact with the preview buttons; all emitted ButtonEvent variants appear in the stream below.",
            )
        });
        let left_pane = cx.new(|_| ButtonExpositionLeftPane {
            look: look.clone(),
            entry,
            primary: primary.clone(),
            secondary: secondary.clone(),
            outline: outline.clone(),
            ghost: ghost.clone(),
            enabled_switch: enabled_switch.clone(),
            event_stream: event_stream.clone(),
        });
        let theme_inspector = cx.new(|cx| ButtonThemeInspector::for_embedded_pane(look.clone(), cx));
        let inspector_split = cx.new(|cx| {
            let left_pane = left_pane.clone();
            let theme_inspector = theme_inspector.clone();
            InspectorSplitShell::new(
                cx,
                look.clone(),
                "controls-doc-button-pane",
                move || left_pane.clone().into_any_element(),
                move || theme_inspector.clone().into_any_element(),
            )
        });

        bind_button_events(&primary, ButtonVariant::Primary, "controls-doc-button-primary", event_stream.clone(), cx);
        bind_button_events(
            &secondary,
            ButtonVariant::Secondary,
            "controls-doc-button-secondary",
            event_stream.clone(),
            cx,
        );
        bind_button_events(&outline, ButtonVariant::Outline, "controls-doc-button-outline", event_stream.clone(), cx);
        bind_button_events(&ghost, ButtonVariant::Ghost, "controls-doc-button-ghost", event_stream.clone(), cx);
        enabled_switch.clone().on_value_changed(cx, |this, on, cx| {
            this.set_buttons_enabled(on, cx);
        });

        Self {
            look,
            entry,
            primary,
            secondary,
            outline,
            ghost,
            enabled_switch,
            buttons_enabled: true,
            event_stream,
            left_pane,
            theme_inspector,
            inspector_split,
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
        for button in [&self.primary, &self.secondary, &self.outline, &self.ghost] {
            button.update(cx, |_, cx| cx.notify());
        }
        self.enabled_switch.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look.clone(), cx));
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        self.theme_inspector.update(cx, |inspector, cx| inspector.sync_look(look.clone(), cx));
        self.inspector_split.update(cx, |split, cx| split.sync_look(look, cx));
        cx.notify();
    }

    fn set_buttons_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.buttons_enabled == enabled {
            return;
        }

        self.buttons_enabled = enabled;
        for button in [&self.primary, &self.secondary, &self.outline, &self.ghost] {
            button.update(cx, |button, cx| button.set_enabled(enabled, cx));
        }
        cx.notify();
    }
}

impl Render for ButtonControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let inspector_split = self.inspector_split.clone();
        with_look(&self.look, || {
            div()
                .id("controls-doc-button-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(inspector_split)
        })
    }
}

fn bind_button_events(
    button: &Entity<Button>,
    variant: ButtonVariant,
    button_id: &'static str,
    event_stream: Entity<ControlEventStream>,
    cx: &mut Context<ButtonControlExposition>,
) {
    let event_stream = event_stream.downgrade();
    button.clone().on_event(cx, move |_, event, cx| {
        let Some(event_stream) = event_stream.upgrade() else {
            return;
        };
        let line = format!("{} - {} (\"{button_id}\")", format_button_event(event), variant.label());
        event_stream.update(cx, |stream, cx| {
            stream.append_line(&line, cx);
            cx.notify();
        });
    });
}

fn format_button_event(event: &ButtonEvent) -> String {
    match event {
        ButtonEvent::Click => "ButtonEvent::Click".to_string(),
        ButtonEvent::FocusChanged { focused } => format!("ButtonEvent::FocusChanged {{ focused: {focused} }}"),
        ButtonEvent::EnabledChanged { enabled } => format!("ButtonEvent::EnabledChanged {{ enabled: {enabled} }}"),
        ButtonEvent::HoverChanged { hovered } => format!("ButtonEvent::HoverChanged {{ hovered: {hovered} }}"),
        _ => "ButtonEvent::(unknown)".to_string(),
    }
}
