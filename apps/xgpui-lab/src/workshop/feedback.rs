//! Native SDK feedback controls. Host actions drive the passive indicators.
use std::sync::Arc;
use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::{
    button::{Button, HasPresenter},
    overlay_window::{self, OverlayWindowMode, OverlayWindowPosition},
    progress, slider, stepper,
};
use crate::theme::{ColorVizLookExt, Look, TextSize, TypographyExt};
use super::{
    event_stream::ControlEventStream,
    sample_layout::{card, page},
};

#[derive(Clone, Copy)]
pub(super) enum FeedbackKind {
    Progress,
    Stepper,
    Modal,
    Modeless,
    Positioning,
    Draggable,
}
impl FeedbackKind {
    fn title(self) -> &'static str {
        match self {
            Self::Progress => "Progress",
            Self::Stepper => "Stepper",
            Self::Modal => "Dialog Modal",
            Self::Modeless => "Dialog Modeless",
            Self::Positioning => "Dialog Positioning",
            Self::Draggable => "Dialog Draggable",
        }
    }
    fn description(self) -> &'static str {
        match self {
            Self::Progress => "SDK Progress is a passive indicator. The SDK slider supplies its value.",
            Self::Stepper => "SDK Stepper is a passive workflow indicator. Next and Reset update its native state.",
            Self::Modal => "SDK OverlayWindow in its default modal mode. Close or Escape dismisses it.",
            Self::Modeless => "SDK OverlayWindow in modeless mode. The surrounding page remains interactive.",
            Self::Positioning => "SDK OverlayWindow using its native BottomRight positioning in modeless mode.",
            Self::Draggable => {
                "SDK OverlayWindow with native dragging enabled and an explicit starting position. Drag its built-in header handle."
            }
        }
    }
}
enum Control {
    Progress { view: progress::Progress, driver: slider::Slider },
    Stepper { view: stepper::Stepper, next: Entity<Button>, reset: Entity<Button> },
    Dialog { view: overlay_window::OverlayWindow, trigger: Entity<Button> },
}
pub(super) struct FeedbackSample {
    kind: FeedbackKind,
    look: Arc<Look>,
    control: Control,
    events: Entity<ControlEventStream>,
}
impl FeedbackSample {
    pub(super) fn new(kind: FeedbackKind, look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let control = match kind {
            FeedbackKind::Progress => {
                let view = progress::ProgressBuilder::new("workshop-sdk-progress").value(40.).spawn(cx);
                let driver = slider::SliderBuilder::new("workshop-sdk-progress-value").value(40.).spawn(cx);
                cx.subscribe(&driver, |this: &mut Self, _, event: &slider::SliderEvent, cx| {
                    if let slider::SliderEvent::Change { value, .. } = event {
                        if let Control::Progress { view, .. } = &this.control {
                            view.update(cx, |v, cx| v.set_value(*value, cx));
                        }
                        this.record(&format!("SDK slider changed progress value to {value:.0}"), cx);
                    }
                })
                .detach();
                Control::Progress { view, driver }
            }
            FeedbackKind::Stepper => {
                let view = stepper::StepperBuilder::new("workshop-sdk-stepper", 3)
                    .labels(vec!["Prepare", "Review", "Finish"])
                    .spawn(cx);
                let next = Button::new("workshop-sdk-stepper-next")
                    .content(|_, _| div().child(crate::l10n::ui("Next")))
                    .on_click(cx, |this: &mut Self, cx| this.next_step(cx))
                    .spawn(cx);
                let reset = Button::new("workshop-sdk-stepper-reset")
                    .content(|_, _| div().child(crate::l10n::ui("Reset")))
                    .on_click(cx, |this: &mut Self, cx| this.reset_step(cx))
                    .spawn(cx);
                Control::Stepper { view, next, reset }
            }
            _ => {
                let close = Button::new("workshop-sdk-dialog-close")
                    .content(|_, _| div().child(crate::l10n::ui("Close")))
                    .on_click(cx, |this: &mut Self, cx| {
                        if let Control::Dialog { view, .. } = &this.control {
                            view.update(cx, |v, cx| v.dismiss(cx));
                        }
                    })
                    .spawn(cx);
                let mut builder = overlay_window::OverlayWindowBuilder::new("workshop-sdk-dialog")
                    .with_template_modifier(|root, _| root.debug_selector(|| "workshop-sdk-dialog-bounds".into()));
                match kind {
                    FeedbackKind::Modeless => builder = builder.mode(OverlayWindowMode::Modeless),
                    FeedbackKind::Positioning => {
                        builder = builder.mode(OverlayWindowMode::Modeless).position(OverlayWindowPosition::BottomRight)
                    }
                    FeedbackKind::Draggable => {
                        // Centered dragging uses an estimated height in the SDK.
                        // An explicit origin avoids a jump when the first drag starts.
                        builder = builder.draggable(true).absolute_position(100., 100.);
                    }
                    _ => {}
                }
                let title = kind.title();
                let view = builder
                    .content(move |_, _, _| {
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(16.))
                            .child(crate::l10n::ui(title))
                            .child(crate::l10n::ui("Native SDK OverlayWindow content."))
                            .child(close.clone())
                            .into_any_element()
                    })
                    .spawn(cx);
                cx.subscribe(&view, |this: &mut Self, _, event: &overlay_window::OverlayWindowEvent, cx| {
                    this.record(&format!("{event:?}"), cx)
                })
                .detach();
                let trigger = Button::new("workshop-sdk-dialog-open")
                    .content(|_, _| div().child(crate::l10n::ui("Open dialog")))
                    .on_click(cx, |this: &mut Self, cx| {
                        if let Control::Dialog { view, trigger } = &this.control {
                            use gpui::Focusable;
                            let focus = trigger.read(cx).focus_handle(cx);
                            view.update(cx, |v, cx| v.open_from(Some(focus), cx));
                        }
                    })
                    .spawn(cx);
                Control::Dialog { view, trigger }
            }
        };
        let events = cx.new(|cx| ControlEventStream::new(cx, look.clone(), "workshop-feedback-events", "Overlay events and host-driven indicator updates appear here. Progress and Stepper do not emit control events."));
        Self { kind, look, control, events }
    }
    fn next_step(&mut self, cx: &mut Context<Self>) {
        if let Control::Stepper { view, .. } = &self.control {
            view.update(cx, |v, cx| {
                if v.current_step() + 1 < v.step_count() {
                    v.set_current_step(v.current_step() + 1, cx);
                } else {
                    v.complete(cx);
                }
            });
        }
        self.record("Host action: advance SDK Stepper", cx);
    }
    fn reset_step(&mut self, cx: &mut Context<Self>) {
        if let Control::Stepper { view, .. } = &self.control {
            view.update(cx, |v, cx| v.set_current_step(0, cx));
        }
        self.record("Host action: reset SDK Stepper", cx);
    }
    fn record(&mut self, line: &str, cx: &mut Context<Self>) {
        self.events.update(cx, |events, cx| events.append_line(line, cx));
        cx.notify();
    }
    pub(super) fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.events.update(cx, |events, cx| events.sync_look(look, cx));
        cx.notify();
    }
}
impl Render for FeedbackSample {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (preview, state) = match &self.control {
            Control::Progress { view, driver } => (
                div().flex().flex_col().gap(px(16.)).child(view.clone()).child(driver.clone()),
                format!("Value: {:.0}", view.read(cx).value()),
            ),
            Control::Stepper { view, next, reset } => (
                div()
                    .flex()
                    .flex_col()
                    .gap(px(16.))
                    .child(view.clone())
                    .child(div().flex().gap(px(8.)).child(next.clone()).child(reset.clone())),
                format!(
                    "Step {} of {} · Complete: {}",
                    view.read(cx).current_step() + 1,
                    view.read(cx).step_count(),
                    view.read(cx).is_complete()
                ),
            ),
            Control::Dialog { view, trigger } => (
                div().child(trigger.clone()),
                format!("Dialog: {}", if view.read(cx).is_open() { "open" } else { "closed" }),
            ),
        };
        let content = page(
            "workshop-feedback-page",
            div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(28.))
                .child(card(
                    &self.look,
                    "workshop-feedback-example",
                    self.kind.title(),
                    self.kind.description(),
                    700.,
                    div().w_full().flex().flex_col().gap(px(16.)).child(preview).child(
                        div()
                            .typography_style(self.look.typography_scale(TextSize::Xs))
                            .text_color(self.look.chrome().muted_text)
                            .child(state),
                    ),
                ))
                .child(self.events.clone()),
        );
        div().size_full().child(content).when_some(
            match &self.control {
                Control::Dialog { view, .. } => Some(view.clone()),
                _ => None,
            },
            |root, view| root.child(view),
        )
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{Focusable, MouseButton, TestAppContext, VisualTestContext, point};
    use crate::workshop::{Sample, Workshop};
    fn setup(app: &mut TestAppContext) -> (Entity<Workshop>, &mut VisualTestContext) {
        app.update(|cx| {
            xgpui_luma::init(cx).unwrap();
            xgpui_luma::key_handling::bind_default_control_keys(cx);
        });
        app.add_window_view(|window, cx| {
            window.activate_window();
            Workshop::new(crate::theme::default_look(), cx)
        })
    }
    fn select(workshop: &Entity<Workshop>, id: &'static str, cx: &mut VisualTestContext) -> Entity<FeedbackSample> {
        workshop.update(cx, |v, cx| v.select(id, cx));
        cx.run_until_parked();
        cx.update(|_, cx| {
            let Sample::Feedback(v) = &workshop.read(cx).samples[id] else {
                panic!("missing SDK feedback sample")
            };
            v.clone()
        })
    }
    #[test]
    fn native_progress_tracks_slider_and_retains_value_across_navigation_and_theme() {
        let mut app = TestAppContext::single();
        let (workshop, cx) = setup(&mut app);
        let sample = select(&workshop, "progress", cx);
        let view = cx.update(|window, cx| {
            let Control::Progress { view, driver } = &sample.read(cx).control else {
                unreachable!()
            };
            let view = view.clone();
            let focus = driver.read(cx).focus_handle(cx);
            focus.focus(window, cx);
            view
        });
        cx.run_until_parked();
        cx.simulate_keystrokes("end left");
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(view.read(cx).value(), 99.));
        workshop.update(cx, |v, cx| {
            v.select("button", cx);
            v.sync_theme(xgpui_luma::theme::ThemeMode::Light, cx);
        });
        assert_eq!(select(&workshop, "progress", cx), sample);
        cx.update(|_, cx| assert_eq!(view.read(cx).value(), 99.));
    }
    #[test]
    fn native_stepper_advances_completes_and_resets_through_sdk_buttons() {
        let mut app = TestAppContext::single();
        let (workshop, cx) = setup(&mut app);
        let sample = select(&workshop, "stepper", cx);
        let (view, reset) = cx.update(|window, cx| {
            let Control::Stepper { view, next, reset } = &sample.read(cx).control else {
                unreachable!()
            };
            let result = (view.clone(), reset.clone());
            let focus = next.read(cx).focus_handle(cx);
            focus.focus(window, cx);
            result
        });
        cx.run_until_parked();
        cx.simulate_keystrokes("space space space");
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert_eq!(view.read(cx).current_step(), 2);
            assert!(view.read(cx).is_complete());
        });
        workshop.update(cx, |v, cx| {
            v.select("button", cx);
            v.sync_theme(xgpui_luma::theme::ThemeMode::Light, cx);
        });
        assert_eq!(select(&workshop, "stepper", cx), sample);
        cx.update(|window, cx| {
            assert!(view.read(cx).is_complete());
            reset.read(cx).focus_handle(cx).focus(window, cx);
        });
        cx.run_until_parked();
        cx.simulate_keystrokes("space");
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert_eq!(view.read(cx).current_step(), 0);
            assert!(!view.read(cx).is_complete());
        });
    }
    #[test]
    fn sdk_dialog_modes_open_dismiss_position_and_drag_without_custom_chrome() {
        let mut app = TestAppContext::single();
        let (workshop, cx) = setup(&mut app);
        for id in ["dialog-modal", "dialog-modeless", "dialog-positioning", "dialog-draggable"] {
            let sample = select(&workshop, id, cx);
            let view = cx.update(|window, cx| {
                let Control::Dialog { view, trigger } = &sample.read(cx).control else {
                    unreachable!()
                };
                let result = view.clone();
                let focus = trigger.read(cx).focus_handle(cx);
                focus.focus(window, cx);
                result
            });
            cx.run_until_parked();
            cx.simulate_keystrokes("space");
            cx.run_until_parked();
            cx.update(|_, cx| assert!(view.read(cx).is_open(), "{id} must open via SDK button"));
            let bounds = cx.debug_bounds("workshop-sdk-dialog-bounds").unwrap();
            if id == "dialog-positioning" {
                let viewport = cx.update(|window, _| window.viewport_size());
                assert!(bounds.center().x > viewport.width / 2.);
                assert!(bounds.center().y > viewport.height / 2.);
            }
            if id == "dialog-draggable" {
                let start = point(bounds.center().x, bounds.top() + px(5.));
                let end = start + point(px(30.), px(20.));
                cx.simulate_mouse_down(start, MouseButton::Left, Default::default());
                cx.simulate_mouse_move(end, MouseButton::Left, Default::default());
                cx.simulate_mouse_move(end, MouseButton::Left, Default::default());
                cx.simulate_mouse_up(end, MouseButton::Left, Default::default());
                cx.run_until_parked();
                let moved = cx.debug_bounds("workshop-sdk-dialog-bounds").unwrap();
                assert!(moved.left() > bounds.left());
                assert!(moved.top() > bounds.top());
            }
            workshop.update(cx, |v, cx| v.sync_theme(xgpui_luma::theme::ThemeMode::Light, cx));
            cx.run_until_parked();
            cx.update(|_, cx| assert!(view.read(cx).is_open()));
            let bounds = cx.debug_bounds("workshop-sdk-dialog-bounds").unwrap();
            cx.simulate_click(point(bounds.left() + px(5.), bounds.center().y), Default::default());
            cx.simulate_keystrokes("escape");
            cx.run_until_parked();
            cx.update(|_, cx| assert!(!view.read(cx).is_open(), "{id} must dismiss"));
            workshop.update(cx, |v, cx| v.select("button", cx));
            assert_eq!(select(&workshop, id, cx), sample);
            cx.update(|_, cx| assert!(!view.read(cx).is_open()));
        }
        workshop.update(cx, |v, cx| v.select("badge", cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-sdk-unavailable").is_some());
        cx.update(|_, cx| assert!(!workshop.read(cx).samples.contains_key("badge")));
    }
}
