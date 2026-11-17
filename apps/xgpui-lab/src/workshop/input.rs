//! Basic native SDK inputs; the Lab look styles only the surrounding sample chrome.
use std::sync::Arc;
use gpui::{AnyView, Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::{scrollbar, slider, textarea, textfield};
use crate::theme::{ColorVizLookExt, Look, TextSize, TypographyExt};
use super::{
    event_stream::ControlEventStream,
    sample_layout::{card, page},
};

#[derive(Clone, Copy)]
pub(super) enum InputKind {
    Scrollbar,
    Slider,
    TextArea,
    TextField,
}
impl InputKind {
    fn title(self) -> &'static str {
        match self {
            Self::Scrollbar => "Scrollbar",
            Self::Slider => "Slider",
            Self::TextArea => "Text Area",
            Self::TextField => "Text Field",
        }
    }
    fn description(self) -> &'static str {
        match self {
            Self::Scrollbar => "Native SDK scrollbar. Drag, scroll, or use arrow keys to change its offset.",
            Self::Slider => "Native SDK slider. Drag the thumb or use arrow keys to change its value.",
            Self::TextArea => "Native SDK multiline text input. Enter inserts a new line.",
            Self::TextField => "Native SDK single-line text input. Enter submits its value.",
        }
    }
}
enum Control {
    Scrollbar(Entity<scrollbar::Scrollbar>),
    Slider(slider::Slider),
    TextArea(Entity<textarea::TextArea>),
    TextField(textfield::TextField),
}
impl Control {
    fn view(&self) -> AnyView {
        match self {
            Self::Scrollbar(v) => v.clone().into(),
            Self::Slider(v) => v.clone().into(),
            Self::TextArea(v) => v.clone().into(),
            Self::TextField(v) => v.clone().into(),
        }
    }
    fn state(&self, cx: &gpui::App) -> String {
        match self {
            Self::Scrollbar(v) => format!("Offset: {:.0}", v.read(cx).value()),
            Self::Slider(v) => format!("Value: {:.0}", v.read(cx).value()),
            Self::TextArea(v) => format!("Value: {:?}", v.read(cx).value()),
            Self::TextField(v) => format!("Value: {:?}", v.read(cx).value()),
        }
    }
}
pub(super) struct InputSample {
    kind: InputKind,
    look: Arc<Look>,
    control: Control,
    events: Entity<ControlEventStream>,
}
impl InputSample {
    pub(super) fn new(kind: InputKind, look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let control = match kind {
            InputKind::Scrollbar => Control::Scrollbar(
                scrollbar::ScrollbarBuilder::new("workshop-sdk-scrollbar")
                    .viewport(0., 1000., 0., 200.)
                    .length(240.)
                    .spawn(cx),
            ),
            InputKind::Slider => {
                Control::Slider(slider::SliderBuilder::new("workshop-sdk-slider").value(50.).spawn(cx))
            }
            InputKind::TextArea => Control::TextArea(
                textarea::TextAreaBuilder::new("workshop-sdk-textarea")
                    .placeholder("Enter multiple lines…")
                    .spawn(cx),
            ),
            InputKind::TextField => Control::TextField(crate::l10n::localize_field(
                textfield::TextFieldBuilder::new("workshop-sdk-textfield").placeholder("Enter text…").spawn(cx),
                cx,
            )),
        };
        match &control {
            Control::Scrollbar(v) => {
                cx.subscribe(v, |this: &mut Self, _, event: &scrollbar::ScrollbarEvent, cx| {
                    if !matches!(
                        event,
                        scrollbar::ScrollbarEvent::HoverChanged { .. } | scrollbar::ScrollbarEvent::FocusChanged { .. }
                    ) {
                        this.record(event, cx);
                    }
                })
                .detach();
            }
            Control::Slider(v) => {
                cx.subscribe(v, |this: &mut Self, _, event: &slider::SliderEvent, cx| {
                    if !matches!(
                        event,
                        slider::SliderEvent::HoverChanged { .. } | slider::SliderEvent::FocusChanged { .. }
                    ) {
                        this.record(event, cx);
                    }
                })
                .detach();
            }
            Control::TextArea(v) => {
                cx.subscribe(v, |this: &mut Self, _, event: &textarea::TextAreaEvent, cx| {
                    if !matches!(event, textarea::TextAreaEvent::FocusChanged { .. }) {
                        this.record(event, cx);
                    }
                })
                .detach();
            }
            Control::TextField(v) => {
                cx.subscribe(v, |this: &mut Self, _, event: &textfield::TextFieldEvent, cx| {
                    if !matches!(event, textfield::TextFieldEvent::FocusChanged { .. }) {
                        this.record(event, cx);
                    }
                })
                .detach();
            }
        }
        let events = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "workshop-input-events",
                "Interact with the SDK input to see its events.",
            )
        });
        Self { kind, look, control, events }
    }
    fn record(&mut self, event: &impl std::fmt::Debug, cx: &mut Context<Self>) {
        self.events.update(cx, |events, cx| events.append_line(&format!("{event:?}"), cx));
        cx.notify();
    }
    pub(super) fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.events.update(cx, |events, cx| events.sync_look(look, cx));
        cx.notify();
    }
}
impl Render for InputSample {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        page(
            "workshop-input-page",
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(28.))
                .child(card(
                    &self.look,
                    "workshop-input-example",
                    self.kind.title(),
                    self.kind.description(),
                    700.,
                    div().w_full().flex().flex_col().gap(px(16.)).child(self.control.view()).child(
                        div()
                            .typography_style(self.look.typography_scale(TextSize::Xs))
                            .text_color(self.look.chrome().muted_text)
                            .child(self.control.state(cx)),
                    ),
                ))
                .child(self.events.clone()),
        )
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{Focusable, TestAppContext};
    use crate::workshop::{Sample, Workshop};

    #[test]
    fn native_inputs_edit_and_retain_values_across_navigation_and_theme_changes() {
        let mut app = TestAppContext::single();
        app.update(|cx| {
            xgpui_luma::init(cx).unwrap();
            xgpui_luma::key_handling::bind_default_control_keys(cx);
        });
        let (workshop, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            Workshop::new(crate::theme::default_look(), cx)
        });
        for (id, keys, text, expected) in [
            ("slider", "end", None, "Value: 100"),
            ("scrollbar", "end", None, "Offset: 800"),
            ("textfield", "", Some("Hello"), "Value: \"Hello\""),
            ("textarea", "", Some("First"), "Value: \"First\\nSecond\""),
        ] {
            workshop.update(cx, |v, cx| v.select(id, cx));
            cx.run_until_parked();
            let sample = cx.update(|window, cx| {
                let Sample::Input(sample) = &workshop.read(cx).samples[id] else {
                    panic!("missing SDK input")
                };
                let sample = sample.clone();
                let focus = match &sample.read(cx).control {
                    Control::Scrollbar(v) => v.read(cx).focus_handle(cx),
                    Control::Slider(v) => v.read(cx).focus_handle(cx),
                    Control::TextArea(v) => v.read(cx).focus_handle(cx),
                    Control::TextField(v) => v.read(cx).focus_handle(cx),
                };
                focus.focus(window, cx);
                sample
            });
            cx.run_until_parked();
            if let Some(text) = text {
                cx.simulate_input(text);
            }
            if !keys.is_empty() {
                cx.simulate_keystrokes(keys);
            }
            if id == "textarea" {
                cx.simulate_keystrokes("enter");
                cx.simulate_input("Second");
            }
            cx.run_until_parked();
            cx.update(|_, cx| assert_eq!(sample.read(cx).control.state(cx), expected));
            workshop.update(cx, |v, cx| {
                v.select("button", cx);
                v.sync_theme(xgpui_luma::theme::ThemeMode::Light, cx);
                v.select(id, cx);
            });
            cx.run_until_parked();
            cx.update(|_, cx| {
                let Sample::Input(retained) = &workshop.read(cx).samples[id] else {
                    unreachable!()
                };
                assert_eq!(retained, &sample);
                assert_eq!(sample.read(cx).control.state(cx), expected);
            });
        }
    }
}
