//! Button samples copied from Studio, without its inspector and documentation host.

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

use super::event_stream::ControlEventStream;

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
                .p(px(16.0))
                .child(preview)
        })
    }
}

pub struct ButtonControlExposition {
    look: Arc<ShadcnLook>,
    primary: Entity<Button>,
    secondary: Entity<Button>,
    outline: Entity<Button>,
    ghost: Entity<Button>,
    enabled_switch: Switch,
    buttons_enabled: bool,
    event_stream: Entity<ControlEventStream>,
    left_pane: Entity<ButtonExpositionLeftPane>,
}

impl ButtonControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
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
            primary: primary.clone(),
            secondary: secondary.clone(),
            outline: outline.clone(),
            ghost: ghost.clone(),
            enabled_switch: enabled_switch.clone(),
            event_stream: event_stream.clone(),
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
            primary,
            secondary,
            outline,
            ghost,
            enabled_switch,
            buttons_enabled: true,
            event_stream,
            left_pane,
        }
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for button in [&self.primary, &self.secondary, &self.outline, &self.ghost] {
            button.update(cx, |_, cx| cx.notify());
        }
        self.enabled_switch.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look.clone(), cx));
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
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
        with_look(&self.look, || {
            div()
                .id("controls-doc-button-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.left_pane.clone())
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

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::{Focusable, TestAppContext};
    use crate::workshop::{Sample, Workshop};

    #[test]
    fn command_button_enabled_switch_controls_every_variant_and_retains_state() {
        let mut app = TestAppContext::single();
        app.update(|cx| {
            xgpui_luma::init(cx).unwrap();
            xgpui_luma::key_handling::bind_default_control_keys(cx);
        });
        let (workshop, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            Workshop::new(crate::theme::default_look(), cx)
        });
        workshop.update(cx, |workshop, cx| workshop.select("button", cx));
        cx.run_until_parked();
        let sample = cx.update(|_, cx| match &workshop.read(cx).samples["button"] {
            Sample::Button(sample) => sample.clone(),
            _ => unreachable!(),
        });
        let switch = cx.update(|_, cx| sample.read(cx).enabled_switch.clone());
        cx.update(|window, cx| switch.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("space");
        cx.run_until_parked();
        workshop.update(cx, |workshop, cx| {
            workshop.select("color-field", cx);
            workshop.sync_theme(xgpui_luma::theme::ThemeMode::Light, cx);
            workshop.select("button", cx);
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            let sample = sample.read(cx);
            assert!(!sample.buttons_enabled);
            assert!(!switch.read(cx).data());
            for button in [&sample.primary, &sample.secondary, &sample.outline, &sample.ghost] {
                assert!(button.read(cx).render_model(window).state.disabled);
            }
        });
        cx.update(|window, cx| switch.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("space");
        cx.run_until_parked();
        cx.update(|window, cx| {
            let sample = sample.read(cx);
            assert!(sample.buttons_enabled);
            for button in [&sample.primary, &sample.secondary, &sample.outline, &sample.ghost] {
                assert!(!button.read(cx).render_model(window).state.disabled);
            }
        });
    }
}
