//! Actual popup wheel dispatch through a nested document; no native GUI launch.
use std::{cell::RefCell, rc::Rc};
use gpui::{Entity, MouseMoveEvent, ScrollDelta, ScrollHandle, ScrollEvent, TestAppContext, VisualTestContext, point, px};
use super::*;
use crate::interaction::{ScrollBoundaryPolicy, WheelFocusScope, WheelScrollPolicy};
use crate::interaction_tests::Page;

type TestPage = Page<Selector>;
fn setup(app: &mut TestAppContext, count: usize) -> (Entity<TestPage>, &mut VisualTestContext) {
    app.add_window_view(|window, cx| {
        window.activate_window();
        crate::key_handling::bind_default_control_keys(cx);
        let focus = cx.focus_handle();
        focus.focus(window, cx);
        Page {
            child: Selector::new("wheel-selector")
                .label("Choose")
                .items((0..count).map(|i| SelectorItem::new(i.to_string()).label(format!("Option {i}"))))
                .selected_id("2")
                .placement(SelectorPlacement::BelowStart)
                .spawn(cx),
            scroll: ScrollHandle::new(),
            focus,
        }
    })
}
fn open(page: &Entity<TestPage>, focused: bool, cx: &mut VisualTestContext) -> Entity<Selector> {
    cx.run_until_parked();
    let selector = cx.update(|window, app| {
        let child = page.read(app).child.clone();
        if focused {
            child.read(app).focus_handle(app).focus(window, app);
        }
        child
    });
    selector.update(cx, |s, cx| {
        s.open_with_default_active(cx);
        cx.notify();
    });
    cx.run_until_parked();
    selector
}
fn scroll(selector: &Entity<Selector>, delta: f32, cx: &mut VisualTestContext) {
    let position = cx.update(|_, app| {
        let bounds = selector.read(app).popup_scroll.bounds();
        point(bounds.left() + px(20.0), bounds.top() + (bounds.size.height / 2.0).min(px(15.0)))
    });
    cx.simulate_event(ScrollEvent {
        position,
        delta: ScrollDelta::Pixels(point(px(0.0), px(delta))),
        ..Default::default()
    });
    cx.run_until_parked();
}
fn reset_page(page: &Entity<TestPage>, cx: &mut VisualTestContext) {
    page.update(cx, |p, cx| {
        p.scroll.set_offset(point(px(0.0), px(0.0)));
        cx.notify();
    });
    cx.run_until_parked();
}

#[test]
fn popup_wheel_policy_matrix_preserves_selection_focus_and_events() {
    for policy in [WheelScrollPolicy::Pointer, WheelScrollPolicy::RequireFocus, WheelScrollPolicy::PassThrough] {
        for boundary in [ScrollBoundaryPolicy::Contain, ScrollBoundaryPolicy::Chain] {
            for focused in [false, true] {
                let mut app = TestAppContext::single();
                let (page, cx) = setup(&mut app, 80);
                let selector = open(&page, focused, cx);
                selector.update(cx, |s, cx| {
                    s.set_wheel_scroll_policy(policy, cx);
                    s.set_scroll_boundary_policy(boundary, cx);
                });
                cx.run_until_parked();
                let events = Rc::new(RefCell::new(Vec::new()));
                let _subscription = cx.update(|_, app| {
                    app.subscribe(&selector, {
                        let events = events.clone();
                        move |_, event: &SelectorEvent, _| events.borrow_mut().push(event.clone())
                    })
                });
                let accepted = policy.accepts(focused);
                scroll(&selector, -25.0, cx);
                cx.update(|window, app| {
                    let s = selector.read(app);
                    assert_eq!(
                        s.popup_scroll.offset().y,
                        px(if accepted { -25.0 } else { 0.0 }),
                        "{policy:?} {boundary:?} {focused}"
                    );
                    assert_eq!(page.read(app).scroll.offset().y, px(if accepted { 0.0 } else { -25.0 }));
                    assert_eq!(s.selected_id().map(|id| id.as_ref()), Some("2"));
                    assert_eq!(s.focus_handle(app).is_focused(window), focused);
                });
                assert!(events.borrow().is_empty());
                reset_page(&page, cx);
                selector.update(cx, |s, cx| {
                    s.popup_scroll.set_offset(point(px(0.0), -s.popup_scroll.max_offset().y));
                    cx.notify();
                });
                cx.run_until_parked();
                let bottom = cx.update(|_, app| selector.read(app).popup_scroll.offset().y);
                assert!(bottom < px(-25.0));
                scroll(&selector, -25.0, cx);
                cx.update(|_, app| {
                    assert_eq!(selector.read(app).popup_scroll.offset().y, bottom);
                    assert_eq!(
                        page.read(app).scroll.offset().y,
                        px(if accepted && boundary == ScrollBoundaryPolicy::Contain {
                            0.0
                        } else {
                            -25.0
                        })
                    );
                });
                reset_page(&page, cx);
                // At five pixels from the bottom, consume the whole event if any movement occurs.
                selector.update(cx, |s, cx| {
                    s.popup_scroll.set_offset(point(px(0.0), bottom + px(5.0)));
                    cx.notify();
                });
                cx.run_until_parked();
                scroll(&selector, -25.0, cx);
                cx.update(|_, app| {
                    assert_eq!(
                        selector.read(app).popup_scroll.offset().y,
                        if accepted { bottom } else { bottom + px(5.0) }
                    );
                    assert_eq!(page.read(app).scroll.offset().y, px(if accepted { 0.0 } else { -25.0 }));
                });
            }
        }
    }
}

#[test]
fn live_policy_updates_preserve_popup_geometry_and_position() {
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, 80);
    let selector = open(&page, true, cx);
    scroll(&selector, -40.5, cx);
    let bounds = cx.update(|_, app| selector.read(app).popup_scroll.bounds());
    selector.update(cx, |s, cx| {
        s.set_wheel_scroll_policy(WheelScrollPolicy::PassThrough, cx);
        s.set_scroll_boundary_policy(ScrollBoundaryPolicy::Chain, cx);
        s.set_wheel_focus_scope(WheelFocusScope::Descendants, cx);
    });
    cx.run_until_parked();
    cx.update(|_, app| {
        let s = selector.read(app);
        assert_eq!(s.popup_scroll.bounds(), bounds);
        assert_eq!(s.popup_scroll.offset().y, px(-40.5));
        assert!(s.open);
    });
    scroll(&selector, -20.0, cx);
    cx.update(|_, app| assert_eq!(selector.read(app).popup_scroll.offset().y, px(-40.5)));
    reset_page(&page, cx);
    selector.update(cx, |s, cx| s.set_wheel_scroll_policy(WheelScrollPolicy::RequireFocus, cx));
    cx.run_until_parked();
    scroll(&selector, -20.0, cx);
    cx.update(|_, app| assert_eq!(selector.read(app).popup_scroll.offset().y, px(-60.5)));
}

#[test]
fn short_popup_boundaries_and_closed_disabled_popup_pass_through() {
    for count in [0, 3] {
        let mut app = TestAppContext::single();
        let (page, cx) = setup(&mut app, count);
        let selector = open(&page, true, cx);
        scroll(&selector, -25.0, cx);
        cx.update(|_, app| assert_eq!(page.read(app).scroll.offset().y, px(0.0)));
        selector.update(cx, |s, cx| s.set_scroll_boundary_policy(ScrollBoundaryPolicy::Chain, cx));
        cx.run_until_parked();
        scroll(&selector, -25.0, cx);
        cx.update(|_, app| assert!(page.read(app).scroll.offset().y < px(0.0)));
    }
    for disabled in [false, true] {
        let mut app = TestAppContext::single();
        let (page, cx) = setup(&mut app, 80);
        let selector = open(&page, true, cx);
        let position = cx.update(|_, app| selector.read(app).popup_scroll.bounds().origin + point(px(20.0), px(15.0)));
        selector.update(cx, |s, cx| {
            if disabled {
                s.set_enabled(false, cx)
            } else {
                s.dismiss(cx)
            }
        });
        cx.run_until_parked();
        cx.simulate_event(ScrollEvent {
            position,
            delta: ScrollDelta::Pixels(point(px(0.0), px(-25.0))),
            ..Default::default()
        });
        cx.run_until_parked();
        cx.update(|_, app| {
            assert!(!selector.read(app).open);
            assert_eq!(selector.read(app).popup_scroll.offset().y, px(0.0));
            assert!(page.read(app).scroll.offset().y < px(0.0));
        });
    }
}

#[test]
fn default_popup_keeps_hover_keyboard_and_click_selection() {
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, 4);
    cx.run_until_parked();
    let selector = cx.update(|_, app| page.read(app).child.clone());
    let trigger = cx.update(|_, app| selector.read(app).trigger_bounds.unwrap().center());
    cx.simulate_click(trigger, Default::default());
    cx.run_until_parked();
    let first = cx.update(|_, app| selector.read(app).popup_scroll.bounds().origin + point(px(20.0), px(10.0)));
    cx.simulate_event(MouseMoveEvent { position: first, ..Default::default() });
    cx.run_until_parked();
    cx.update(|window, app| {
        let s = selector.read(app);
        assert!(s.open);
        assert_eq!(s.active_index, Some(0));
        assert_eq!(s.selected_id().map(|id| id.as_ref()), Some("2"));
        assert!(s.focus_handle(app).is_focused(window));
    });
    cx.simulate_keystrokes("down enter");
    cx.run_until_parked();
    cx.update(|_, app| {
        let s = selector.read(app);
        assert!(!s.open);
        assert_eq!(s.selected_id().map(|id| id.as_ref()), Some("1"));
    });
    cx.simulate_click(trigger, Default::default());
    cx.run_until_parked();
    cx.simulate_click(first, Default::default());
    cx.run_until_parked();
    cx.update(|_, app| {
        let s = selector.read(app);
        assert!(!s.open);
        assert_eq!(s.selected_id().map(|id| id.as_ref()), Some("0"));
    });
}

#[test]
fn default_popup_geometry_scrolling_and_reopening_match_the_legacy_native_template() {
    use crate::controls::selector_list::{
        SelectorItemsRenderModel, SelectorItemsTemplateHandlers, default_selector_items_template,
    };
    struct LegacyPanel;
    impl SelectorItemsTemplate<SelectorItem> for LegacyPanel {
        fn render(
            &self,
            model: &SelectorItemsRenderModel<'_, SelectorItem>,
            mut handlers: SelectorItemsTemplateHandlers,
            cx: &mut App,
        ) -> gpui::Stateful<gpui::Div> {
            let legacy = SelectorItemsRenderModel { scroll_handle: None, menu: model.menu.clone(), ..*model };
            handlers.scroll_wheel = None;
            default_selector_items_template().render(&legacy, handlers, cx)
        }
    }
    let mut results = Vec::new();
    for legacy in [true, false] {
        let mut app = TestAppContext::single();
        let (page, cx) = setup(&mut app, 80);
        let selector = cx.update(|_, app| page.read(app).child.clone());
        selector.update(cx, |s, cx| {
            s.set_item_template(
                Some(crate::controls::selector_list::make_selector_item_template::<SelectorItem, _, _>(|model, _| {
                    let index = model.index;
                    div().debug_selector(move || format!("option-{index}")).child(model.item.label().clone())
                })),
                cx,
            );
            if legacy {
                s.set_panel_template(Arc::new(LegacyPanel), cx);
            }
        });
        open(&page, true, cx);
        let before = cx.debug_bounds("option-2").unwrap();
        let position = before.center();
        cx.simulate_event(ScrollEvent {
            position,
            delta: ScrollDelta::Pixels(point(px(0.0), px(-25.0))),
            ..Default::default()
        });
        cx.run_until_parked();
        let after = cx.debug_bounds("option-2").unwrap();
        assert_eq!(after.top(), before.top() - px(25.0));
        selector.update(cx, |s, cx| s.dismiss(cx));
        cx.run_until_parked();
        open(&page, true, cx);
        let reopened = cx.debug_bounds("option-2").unwrap();
        results.push((before, after, reopened));
    }
    assert_eq!(results[0], results[1]);
}

#[test]
fn centered_opening_measures_custom_rows_and_keeps_selection_aligned() {
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, 80);
    page.update(cx, |page, cx| {
        page.child = Selector::new("centered-custom-rows")
            .items((0..80).map(|i| SelectorItem::new(i.to_string()).label(format!("Option {i}"))))
            .selected_id("40")
            .opening_mode(crate::controls::selector::SelectorOpeningMode::Centered)
            .with_item_template(|model, _| gpui::div().h(px(67.0)).child(model.item.label().clone()))
            .spawn(cx);
        cx.notify();
    });
    let selector = open(&page, true, cx);
    cx.update(|window, app| window.simulate_next_frame(app));
    cx.run_until_parked();
    cx.update(|_, app| {
        let s = selector.read(app);
        let geometry = s.popup_geometry.expect("measured geometry");
        assert!(geometry.row_center > px(40.0 * 60.0));
        assert!(!s.initialize_popup_scroll);
        let row_center = s.popup_scroll.bounds().top() + geometry.row_center + s.popup_scroll.offset().y;
        let trigger_center = s.trigger_bounds.expect("trigger bounds").center().y;
        assert!((row_center - trigger_center).abs() < px(1.0), "{row_center:?} vs {trigger_center:?}");
        assert!(s.popup_scroll.offset().y < px(0.0));
    });
    let initial = cx.update(|_, app| selector.read(app).popup_scroll.offset());
    scroll(&selector, -30.0, cx);
    cx.update(|_, app| assert!(selector.read(app).popup_scroll.offset().y < initial.y));
    selector.update(cx, |s, cx| {
        s.close_menu();
        s.open_with_default_active(cx);
        cx.notify();
    });
    cx.run_until_parked();
    cx.update(|window, app| window.simulate_next_frame(app));
    cx.run_until_parked();
    cx.update(|_, app| assert_eq!(selector.read(app).popup_scroll.offset(), initial));
}

#[test]
fn centered_no_selection_fallback_preserves_disabled_rows_and_keyboard_selection() {
    for count in [0, 4] {
        let mut app = TestAppContext::single();
        let (page, cx) = setup(&mut app, count);
        page.update(cx, |page, cx| {
            page.child = Selector::new("centered-unselected")
                .items(
                    (0..count).map(|i| SelectorItem::new(i.to_string()).label(format!("Option {i}")).enabled(i != 0)),
                )
                .opening_mode(crate::controls::selector::SelectorOpeningMode::Centered)
                .spawn(cx);
            cx.notify();
        });
        let selector = open(&page, true, cx);
        cx.update(|window, app| window.simulate_next_frame(app));
        cx.run_until_parked();
        cx.update(|_, app| {
            let s = selector.read(app);
            assert!(s.selected_id().is_none());
            assert_eq!(s.active_index, if count == 0 { None } else { Some(1) });
            assert_eq!(s.popup_geometry.is_some(), count > 0);
        });
        if count > 0 {
            let point_for_row = |index, cx: &mut VisualTestContext| {
                cx.update(|_, app| {
                    let scroll = &selector.read(app).popup_scroll;
                    let first = scroll.bounds_for_item(0).expect("first row");
                    let row = scroll.bounds_for_item(index).expect("target row");
                    point(
                        scroll.bounds().left() + px(20.0),
                        scroll.bounds().top() + row.center().y - first.top() + scroll.offset().y,
                    )
                })
            };
            let disabled_row = point_for_row(0, cx);
            cx.simulate_click(disabled_row, Default::default());
            cx.run_until_parked();
            cx.update(|_, app| assert!(selector.read(app).selected_id().is_none()));
            let last_row = point_for_row(3, cx);
            cx.simulate_click(last_row, Default::default());
            cx.run_until_parked();
            cx.update(|_, app| assert_eq!(selector.read(app).selected_id().map(|id| id.as_ref()), Some("3")));
            selector.update(cx, |s, cx| {
                s.open_with_default_active(cx);
                cx.notify();
            });
            cx.run_until_parked();
            cx.simulate_keystrokes("home enter");
            cx.run_until_parked();
            cx.update(|_, app| {
                assert_eq!(selector.read(app).selected_id().map(|id| id.as_ref()), Some("1"));
                assert!(!selector.read(app).open);
            });
        } else {
            cx.simulate_keystrokes("escape");
            cx.run_until_parked();
            cx.update(|_, app| assert!(!selector.read(app).open));
        }
    }
}
