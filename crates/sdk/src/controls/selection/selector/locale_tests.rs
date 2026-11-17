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

#[test]
fn list_rows_mirror_without_reordering_or_changing_disabled_and_selection_behavior() {
    let mut app = TestAppContext::single();
    let (selector, cx) = app.add_window_view(|_, cx| {
        Selector::from_builder(
            Selector::new("rtl-list")
                .items([
                    SelectorItem::new("first").label("First"),
                    SelectorItem::new("disabled").label("Disabled").enabled(false),
                    SelectorItem::new("last").label("Last"),
                ])
                .selected_id("first")
                .motion_enabled(false),
            cx,
        )
    });
    cx.run_until_parked();
    cx.simulate_click(gpui::point(gpui::px(25.), gpui::px(15.)), Default::default());
    cx.run_until_parked();
    let changes = Rc::new(Cell::new(0));
    let capture = changes.clone();
    let _subscription = cx.update(|_, cx| {
        cx.subscribe(&selector, move |_, event: &SelectorEvent, _| {
            if matches!(event, SelectorEvent::Change { .. }) {
                capture.set(capture.get() + 1);
            }
        })
    });
    for direction in [LayoutDirection::Ltr, LayoutDirection::Rtl, LayoutDirection::Ltr] {
        cx.update(|_, cx| set_locale_context(LocaleContext { direction, ..Default::default() }, cx));
        cx.run_until_parked();
        let mut bottom = None;
        for (content_id, check_id, row_id) in [
            ("selector-row-content-0", "selector-row-check-0", "selector-row-0"),
            ("selector-row-content-1", "selector-row-check-1", "selector-row-1"),
            ("selector-row-content-2", "selector-row-check-2", "selector-row-2"),
        ] {
            let content = cx.debug_bounds(content_id).unwrap();
            let check = cx.debug_bounds(check_id).unwrap();
            let row = cx.debug_bounds(row_id).unwrap();
            assert_eq!(content.left() < check.left(), direction == LayoutDirection::Ltr);
            if let Some(previous) = bottom {
                assert!(row.top() >= previous);
            }
            bottom = Some(row.bottom());
        }
        cx.update(|_, cx| {
            let view = selector.read(cx);
            assert!(view.open);
            assert_eq!(view.selected_id().unwrap(), "first");
        });
        assert_eq!(changes.get(), 0);
    }
    // Disabled rows remain noninteractive after mirroring.
    cx.update(|_, cx| set_locale_context(LocaleContext { direction: LayoutDirection::Rtl, ..Default::default() }, cx));
    cx.run_until_parked();
    let disabled = cx.debug_bounds("selector-row-1").unwrap();
    cx.simulate_click(disabled.center(), Default::default());
    cx.run_until_parked();
    cx.update(|_, cx| {
        assert_eq!(selector.read(cx).selected_id().unwrap(), "first");
    });
    assert_eq!(changes.get(), 0);
    selector.update(cx, |view, cx| {
        view.open_with_default_active(cx);
        cx.notify();
    });
    cx.run_until_parked();
    let last = cx.debug_bounds("selector-row-2").unwrap();
    cx.simulate_click(last.center(), Default::default());
    cx.run_until_parked();
    cx.update(|_, cx| {
        assert!(!selector.read(cx).open);
        assert_eq!(selector.read(cx).selected_id().unwrap(), "last");
    });
    assert_eq!(changes.get(), 1);
}
