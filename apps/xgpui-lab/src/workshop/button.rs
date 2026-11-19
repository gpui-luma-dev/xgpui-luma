//! A basic SDK button. The surrounding settings and event log use the Lab look.
use std::sync::Arc;
use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::button::{Button, ButtonEventExt, HasPresenter};
use xgpui_luma::controls::switch::{Switch, SwitchEventExt};
use crate::theme::Look;
use super::event_stream::ControlEventStream;
use super::theme_inspector::{InspectorSplit, ThemeInspector};
use xgpui_luma::controls::button_family::ButtonStyle;

pub struct ButtonControlExposition {
    look: Arc<Look>,
    button: Entity<Button>,
    enabled_switch: Switch,
    buttons_enabled: bool,
    event_stream: Entity<ControlEventStream>,
    split: Entity<InspectorSplit>,
    body: Entity<ButtonBody>,
    inspector: Entity<ThemeInspector<ButtonStyle>>,
}
/// The button page, rendered as the left panel of the inspector split.
struct ButtonBody {
    page: gpui::WeakEntity<ButtonControlExposition>,
}
impl Render for ButtonBody {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(page) = self.page.upgrade() else {
            return div().into_any_element();
        };
        let look = page.read(cx).look.clone();
        let preview = super::template_preview::button_template_preview(&look, window, cx);
        page.read(cx).page(preview)
    }
}
impl ButtonControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<Look>) -> Self {
        let button = Button::new("workshop-sdk-button").content(|_, _| div().child("Button")).spawn(cx);
        let enabled_switch = xgpui_luma_look_radix::Switch::new("workshop-sdk-button-enabled")
            .look(&look)
            .checked(true)
            .content(|_, _| div().child("Enabled"))
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-button-event-log",
                "Activate the button to see its SDK events.",
            )
            .without_card()
        });
        button.clone().on_event(cx, |this: &mut Self, event, cx| {
            let line = format!("{event:?}");
            this.event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
        });
        enabled_switch.clone().on_value_changed(cx, |this: &mut Self, on, cx| {
            this.buttons_enabled = on;
            this.button.update(cx, |button, cx| button.set_enabled(on, cx));
            this.refresh(cx);
        });
        let tokens = xgpui_luma::theme::ThemeTokens::default();
        let style = ButtonStyle::from_stylesheet(&tokens.stylesheet);
        let inspector_look = look.clone();
        let inspector = cx.new(|cx| ThemeInspector::new(inspector_look, "Button", "button", tokens, style, cx));
        let page = cx.weak_entity();
        let body = cx.new(|_| ButtonBody { page });
        let (left, right) = (body.clone().into(), inspector.clone().into());
        let split = cx.new(|cx| InspectorSplit::new("workshop-button-inspector-split", left, right, cx));
        Self { look, button, enabled_switch, buttons_enabled: true, event_stream, split, body, inspector }
    }
    /// Re-render this page, including the split's left panel.
    fn refresh(&mut self, cx: &mut Context<Self>) {
        self.body.update(cx, |_, cx| cx.notify());
        cx.notify();
    }
    pub fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look.clone(), cx));
        self.inspector.update(cx, |inspector, cx| inspector.sync_look(look, cx));
        self.refresh(cx);
    }
}
impl ButtonControlExposition {
    fn page(&self, preview: gpui::AnyElement) -> gpui::AnyElement {
        use super::sample_layout::{section, page, sdk_preview};
        page(
            "workshop-sdk-button-page",
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(28.))
                .child(section(
                    &self.look,
                    "command-button-basic",
                    "Button",
                    "Default SDK appearance and behavior.",
                    420.,
                    sdk_preview("workshop-sdk-button-preview", self.button.clone()),
                ))
                .child(self.enabled_switch.clone())
                .child(section(
                    &self.look,
                    "command-button-template-preview",
                    "Template Preview",
                    "The SDK template for the Text and Icon roles in each interaction state.",
                    720.,
                    preview,
                ))
                .child(self.event_stream.clone()),
        )
    }
}
impl Render for ButtonControlExposition {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.split.clone())
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{Focusable, TestAppContext};
    use crate::workshop::{Workshop, Sample};
    #[test]
    fn button_page_has_a_theme_inspector_and_template_preview() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (workshop, cx) = app.add_window_view(|_, cx| Workshop::new(crate::theme::default_look(), cx));
        workshop.update(cx, |workshop, cx| workshop.select("button", cx));
        cx.run_until_parked();
        let inspector = cx.update(|_, cx| match &workshop.read(cx).samples["button"] {
            Sample::Button(sample) => sample.read(cx).inspector.clone(),
            _ => unreachable!(),
        });
        assert_eq!(cx.update(|_, cx| inspector.read(cx).toggle_count()), 3, "disabled, pressed and hovered");
    }

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
        for mode in [xgpui_luma::theme::ThemeMode::Dark, xgpui_luma::theme::ThemeMode::Light] {
            workshop.update(cx, |v, cx| v.sync_theme(mode, cx));
            cx.run_until_parked();
            assert!(cx.debug_bounds("workshop-sdk-button-preview").is_some());
            cx.update(|window, cx| {
                let v = sample.read(cx);
                assert_eq!(v.look.mode(), mode);
                // Unconverted pages keep their fixed light look.
                assert_eq!(workshop.read(cx).page_look.mode(), xgpui_luma::theme::ThemeMode::Light);
                assert!(!v.buttons_enabled);
                let model = v.button.read(cx).render_model(window);
                assert!(model.state.disabled);
                assert!(model.look.is_none());
            });
        }
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
