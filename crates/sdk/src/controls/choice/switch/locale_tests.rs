use super::*;
use std::{cell::Cell, rc::Rc};
use gpui::TestAppContext;
use crate::infra::locale::{LayoutDirection, LocaleContext, set_locale_context};

#[test]
fn live_locale_mirrors_horizontal_thumb_and_label_without_changing_switch_semantics() {
    for orientation in [SwitchOrientation::Horizontal, SwitchOrientation::Vertical] {
        for enabled in [true, false] {
            for checked in [true, false] {
                let mut app = TestAppContext::single();
                let (switch, cx) = app.add_window_view(|window, cx| {
                    window.activate_window();
                    SwitchControl::from_builder(
                        new("localized-switch")
                            .label("Switch label")
                            .with_data(checked)
                            .enabled(enabled)
                            .orientation(orientation)
                            .motion_enabled(false),
                        cx,
                    )
                });
                cx.run_until_parked();
                if enabled {
                    cx.update(|window, cx| switch.read(cx).focus_handle(cx).focus(window, cx));
                }
                let changes = Rc::new(Cell::new(0));
                let observed = changes.clone();
                let _sub = cx.update(|_, cx| {
                    cx.subscribe(&switch, move |_, event: &SwitchEvent, _| {
                        if matches!(event, SwitchEvent::Change { .. }) {
                            observed.set(observed.get() + 1);
                        }
                    })
                });
                for direction in
                    [LayoutDirection::Ltr, LayoutDirection::Rtl, LayoutDirection::Ltr, LayoutDirection::Rtl]
                {
                    cx.update(|_, cx| set_locale_context(LocaleContext { direction, ..Default::default() }, cx));
                    cx.run_until_parked();
                    let track = cx.debug_bounds("switch-track").unwrap();
                    let thumb = cx.debug_bounds("switch-thumb").unwrap();
                    let label = cx.debug_bounds("switch-label").unwrap();
                    assert_eq!(track.left() < label.left(), direction == LayoutDirection::Ltr);
                    match orientation {
                        SwitchOrientation::Horizontal => assert_eq!(
                            thumb.center().x > track.center().x,
                            checked != (direction == LayoutDirection::Rtl)
                        ),
                        SwitchOrientation::Vertical => assert_eq!(thumb.center().y < track.center().y, checked),
                    }
                    cx.update(|window, cx| {
                        assert_eq!(*switch.read(cx).data(), checked);
                        if enabled {
                            assert!(switch.read(cx).focus_handle(cx).is_focused(window));
                        }
                    });
                    assert_eq!(changes.get(), 0);
                }
                let label = cx.debug_bounds("switch-label").unwrap();
                cx.simulate_click(label.center(), Default::default());
                cx.run_until_parked();
                cx.update(|_, cx| assert_eq!(*switch.read(cx).data(), if enabled { !checked } else { checked }));
                assert_eq!(changes.get(), if enabled { 1 } else { 0 });
            }
        }
    }
}
