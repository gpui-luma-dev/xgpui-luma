use super::*;
use std::{cell::Cell, rc::Rc};
use gpui::{AppContext, MouseButton, MouseDownEvent, MouseUpEvent, TestAppContext, point, px};
use crate::infra::locale::{LayoutDirection, LocaleContext, set_locale_context};

#[test]
fn live_horizontal_direction_preserves_value_and_changes_pointer_and_keyboard_mapping() {
    let mut app = TestAppContext::single();
    let (slider, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        crate::key_handling::bind_default_control_keys(cx);
        SliderControl::from_builder(
            SliderBuilder::new("rtl-slider").range(0.0..100.0).value(25.).step(5.).motion_enabled(false),
            cx,
        )
    });
    cx.run_until_parked();
    cx.update(|window, cx| slider.read(cx).focus_handle(cx).focus(window, cx));
    let changes = Rc::new(Cell::new(0));
    let observed = changes.clone();
    let _subscription = cx.update(|_, cx| {
        cx.subscribe(&slider, move |_, event: &SliderEvent, _| {
            if matches!(event, SliderEvent::Change { .. }) {
                observed.set(observed.get() + 1);
            }
        })
    });
    for direction in [LayoutDirection::Ltr, LayoutDirection::Rtl, LayoutDirection::Ltr] {
        slider.update(cx, |view, cx| view.set_value(25., cx));
        let count = changes.get();
        cx.update(|_, cx| set_locale_context(LocaleContext { direction, ..Default::default() }, cx));
        cx.run_until_parked();
        cx.update(|window, cx| {
            let view = slider.read(cx);
            assert_eq!(view.value(), 25.);
            assert_eq!(view.model.reversed, direction == LayoutDirection::Rtl);
            let bounds = view.track_bounds.unwrap();
            let anchor = view.attachment_anchor(bounds);
            let expected = if direction == LayoutDirection::Rtl { 0.75 } else { 0.25 };
            assert!((anchor.origin.x - (bounds.left() + bounds.size.width * expected)).abs() < px(1.));
            assert!(view.focus_handle(cx).is_focused(window));
        });
        assert_eq!(changes.get(), count);
        cx.simulate_keystrokes("right");
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert_eq!(slider.read(cx).value(), if direction == LayoutDirection::Rtl { 20. } else { 30. })
        });
        slider.update(cx, |view, cx| view.set_value(50., cx));
        cx.run_until_parked();
        let bounds = cx.update(|_, cx| slider.read(cx).track_bounds.unwrap());
        let position = point(bounds.left() + bounds.size.width * 0.75, bounds.center().y);
        cx.simulate_event(MouseDownEvent { button: MouseButton::Left, position, click_count: 1, ..Default::default() });
        cx.simulate_event(MouseUpEvent { button: MouseButton::Left, position, click_count: 1, ..Default::default() });
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert!((slider.read(cx).value() - if direction == LayoutDirection::Rtl { 25. } else { 75. }).abs() < 0.01)
        });
    }
}

#[test]
fn explicit_reversal_and_nonhorizontal_strategies_ignore_locale() {
    for (strategy, explicit) in [
        (SliderInputStrategy::Horizontal, None),
        (SliderInputStrategy::Horizontal, Some(false)),
        (SliderInputStrategy::Horizontal, Some(true)),
        (SliderInputStrategy::Vertical, None),
        (SliderInputStrategy::Angular { min_angle: 0., max_angle: 3. }, None),
    ] {
        let app = TestAppContext::single();
        app.update(|cx| {
            set_locale_context(LocaleContext { direction: LayoutDirection::Rtl, ..Default::default() }, cx);
            let slider = cx.new(|cx| {
                let mut builder = SliderBuilder::new("configured").strategy(strategy);
                if let Some(reversed) = explicit {
                    builder = builder.reversed(reversed);
                }
                SliderControl::from_builder(builder, cx)
            });
            assert_eq!(slider.read(cx).model.reversed, explicit.unwrap_or(strategy == SliderInputStrategy::Horizontal));
            set_locale_context(LocaleContext::default(), cx);
            slider.update(cx, |view, cx| view.model.resolve_locale_direction(cx));
            assert_eq!(slider.read(cx).model.reversed, explicit.unwrap_or(false));
        });
    }
}
