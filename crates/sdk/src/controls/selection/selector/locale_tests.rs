use super::*;
use std::{cell::Cell, rc::Rc};
use gpui::TestAppContext;
use crate::infra::locale::{LayoutDirection, LocaleContext, set_locale_context};

#[test]
fn direction_refresh_moves_caret_and_preserves_open_selection_and_focus() {
    let mut app = TestAppContext::single();
    let (selector, cx) = app.add_window_view(|_, cx| {
        Selector::from_builder(
            Selector::new("rtl-selector")
                .items([SelectorItem::new("first").label("First"), SelectorItem::new("second").label("Second")])
                .selected_id("second")
                .motion_enabled(false),
            cx,
        )
    });
    cx.run_until_parked();
    cx.simulate_click(gpui::point(gpui::px(25.), gpui::px(15.)), Default::default());
    cx.run_until_parked();
    cx.update(|window, cx| {
        let view = selector.read(cx);
        assert!(view.open);
        assert!(view.focus_handle(cx).is_focused(window));
    });
    let changes = Rc::new(Cell::new(0));
    let capture = changes.clone();
    let _subscription = cx.update(|_, cx| {
        cx.subscribe(&selector, move |_, event: &SelectorEvent, _| {
            if matches!(event, SelectorEvent::Change { .. } | SelectorEvent::OpenChanged { .. }) {
                capture.set(capture.get() + 1);
            }
        })
    });
    for direction in [LayoutDirection::Ltr, LayoutDirection::Rtl, LayoutDirection::Ltr] {
        cx.update(|_, cx| set_locale_context(LocaleContext { direction, ..Default::default() }, cx));
        cx.run_until_parked();
        let label = cx.debug_bounds("selector-trigger-label").unwrap();
        let caret = cx.debug_bounds("selector-trigger-caret").unwrap();
        assert_eq!(label.left() < caret.left(), direction == LayoutDirection::Ltr);
        cx.update(|window, cx| {
            let view = selector.read(cx);
            assert!(view.open);
            assert_eq!(view.selected_id().unwrap(), "second");
            assert!(view.focus_handle(cx).is_focused(window));
        });
    }
    assert_eq!(changes.get(), 0);
}
