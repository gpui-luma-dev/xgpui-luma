//! A basic SDK button. The surrounding card and event log use the Lab look.
use std::sync::Arc;
use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::button::{Button, ButtonEventExt, HasPresenter};
use xgpui_luma::controls::switch::{Switch, SwitchEventExt};
use crate::theme::Look;
use super::event_stream::ControlEventStream;

pub struct ButtonControlExposition {
    look: Arc<Look>,
    button: Entity<Button>,
    enabled_switch: Switch,
    buttons_enabled: bool,
    event_stream: Entity<ControlEventStream>,
}
impl ButtonControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<Look>) -> Self {
        let button =
            Button::new("workshop-sdk-button").content(|_, _| div().child(crate::l10n::ui("Button"))).spawn(cx);
        let enabled_switch = xgpui_luma::controls::switch::new("workshop-sdk-button-enabled")
            .with_data(true)
            .content(|_, _| div().child(crate::l10n::ui("Enabled")))
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-button-event-log",
                "Activate the button to see its SDK events.",
            )
        });
        button.clone().on_event(cx, |this: &mut Self, event, cx| {
            let line = format!("{event:?}");
            this.event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
        });
        enabled_switch.clone().on_value_changed(cx, |this: &mut Self, on, cx| {
            this.buttons_enabled = on;
            this.button.update(cx, |button, cx| button.set_enabled(on, cx));
            cx.notify();
        });
        Self { look, button, enabled_switch, buttons_enabled: true, event_stream }
    }
    pub fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}
impl Render for ButtonControlExposition {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        use super::sample_layout::{card, page};
        page(
            "workshop-sdk-button-page",
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(28.))
                .child(card(
                    &self.look,
                    "command-button-basic",
                    "Button",
                    "Default SDK appearance and behavior.",
                    420.,
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap(px(12.))
                        .child(self.button.clone())
                        .child(self.enabled_switch.clone()),
                ))
                .child(self.event_stream.clone()),
        )
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{Focusable, TestAppContext};
    use crate::workshop::{Workshop, Sample};
    #[test]
    fn sdk_button_retains_enabled_state_across_app_theme_and_navigation() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::key_handling::bind_default_control_keys(cx));
        let (workshop, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            Workshop::new(crate::theme::default_look(), cx)
        });
        workshop.update(cx, |v, cx| v.select("button", cx));
        cx.run_until_parked();
        let sample = cx.update(|_, cx| match &workshop.read(cx).samples["button"] {
            Sample::Button(v) => v.clone(),
            _ => unreachable!(),
        });
        let switch = cx.update(|_, cx| sample.read(cx).enabled_switch.clone());
        cx.update(|window, cx| switch.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("space");
        cx.run_until_parked();
        workshop.update(cx, |v, cx| {
            v.select("color-field", cx);
            v.sync_theme(xgpui_luma::theme::ThemeMode::Light, cx);
            v.select("button", cx);
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            let v = sample.read(cx);
            assert!(!v.buttons_enabled);
            assert!(v.button.read(cx).render_model(window).state.disabled);
        });
    }
    #[test]
    fn sdk_button_palette_does_not_follow_radix_theme() {
        use std::sync::Mutex;
        let mut app = TestAppContext::single();
        let look = crate::theme::default_look();
        let (sample, cx) = app.add_window_view(|_, cx| ButtonControlExposition::new(cx, look.clone()));
        let button = cx.update(|_, cx| sample.read(cx).button.clone());
        let observed = Arc::new(Mutex::new(None));
        let target = observed.clone();
        button.update(cx, |v, cx| {
            v.set_presenter(
                Arc::new(move |model, _| {
                    *target.lock().unwrap() = Some((model.look.background, model.look.foreground));
                    div().child("SDK").into_any_element()
                }),
                cx,
            )
        });
        cx.run_until_parked();
        let original = *observed.lock().unwrap();
        assert!(original.is_some());
        for mode in [xgpui_luma::theme::ThemeMode::Light, xgpui_luma::theme::ThemeMode::Dark] {
            look.set_mode(mode);
            sample.update(cx, |v, cx| v.sync_look(look.clone(), cx));
            button.update(cx, |_, cx| cx.notify());
            cx.run_until_parked();
            assert_eq!(*observed.lock().unwrap(), original);
            cx.update(|window, cx| assert!(button.read(cx).render_model(window).look.is_none()));
        }
    }
}
