use super::{checkbox, radio_button};
use std::{cell::Cell, rc::Rc};
use gpui::{Context, Focusable, IntoElement, Render, TestAppContext, Window, div, prelude::*};
use crate::infra::{
    locale::{LayoutDirection, LocaleContext, set_locale_context},
    presenter::HasPresenter,
};
struct Harness {
    checkbox: checkbox::Checkbox,
    radio: radio_button::RadioButton,
}
impl Render for Harness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().flex().flex_col().child(self.checkbox.clone()).child(self.radio.clone())
    }
}
#[test]
fn indicators_follow_locale_without_changing_state_and_labels_keep_activation_targets() {
    for enabled in [true, false] {
        let mut app = TestAppContext::single();
        let (host, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            Harness {
                checkbox: checkbox::new("check")
                    .label("Check label")
                    .with_data(false)
                    .enabled(enabled)
                    .motion_enabled(false)
                    .spawn(cx),
                radio: radio_button::new("radio")
                    .label("Radio label")
                    .with_data(false)
                    .enabled(enabled)
                    .motion_enabled(false)
                    .spawn(cx),
            }
        });
        cx.run_until_parked();
        let (check, radio) = cx.update(|_, cx| {
            let host = host.read(cx);
            (host.checkbox.clone(), host.radio.clone())
        });
        if enabled {
            cx.update(|window, cx| check.read(cx).focus_handle(cx).focus(window, cx));
        }
        let changes = Rc::new(Cell::new(0));
        let observed = changes.clone();
        let _check_sub = cx.update(|_, cx| {
            cx.subscribe(&check, move |_, event: &checkbox::CheckboxEvent, _| {
                if matches!(event, checkbox::CheckboxEvent::Change { .. }) {
                    observed.set(observed.get() + 1);
                }
            })
        });
        let observed = changes.clone();
        let _radio_sub = cx.update(|_, cx| {
            cx.subscribe(&radio, move |_, event: &radio_button::RadioButtonEvent, _| {
                if matches!(event, radio_button::RadioButtonEvent::Change { .. }) {
                    observed.set(observed.get() + 1);
                }
            })
        });
        for direction in [LayoutDirection::Ltr, LayoutDirection::Rtl, LayoutDirection::Ltr, LayoutDirection::Rtl] {
            cx.update(|_, cx| set_locale_context(LocaleContext { direction, ..Default::default() }, cx));
            cx.run_until_parked();
            for (indicator, label) in
                [("checkbox-indicator", "checkbox-label"), ("radio_button-indicator", "radio_button-label")]
            {
                let indicator = cx.debug_bounds(indicator).unwrap();
                let label = cx.debug_bounds(label).unwrap();
                assert_eq!(indicator.left() < label.left(), direction == LayoutDirection::Ltr);
            }
            cx.update(|window, cx| {
                assert!(!*check.read(cx).data());
                assert!(!*radio.read(cx).data());
                if enabled {
                    assert!(check.read(cx).focus_handle(cx).is_focused(window));
                }
            });
            assert_eq!(changes.get(), 0);
        }
        for label in ["checkbox-label", "radio_button-label"] {
            let bounds = cx.debug_bounds(label).unwrap();
            cx.simulate_click(bounds.center(), Default::default());
            cx.run_until_parked();
        }
        cx.update(|_, cx| {
            assert_eq!(*check.read(cx).data(), enabled);
            assert_eq!(*radio.read(cx).data(), enabled);
        });
        assert_eq!(changes.get(), if enabled { 2 } else { 0 });
    }
}
