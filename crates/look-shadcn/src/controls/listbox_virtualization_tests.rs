//! Headless integration coverage for windowed rendering on both axes.
use std::{cell::RefCell, rc::Rc};
use gpui::{Bounds, Pixels, ScrollDelta, ScrollEvent, TestAppContext, TouchPhase, VisualTestContext, point};
use xgpui_luma::controls::listbox::{ListBoxSnapshot, SelectionMode};
use super::*;

struct Harness {
    list: ListBoxControl<Self, u32, u32>,
    look: ShadcnLook,
    horizontal: bool,
    width: f32,
    visible: usize,
    rendered: Rc<RefCell<Vec<u32>>>,
    parent_wheels: usize,
}

impl Harness {
    fn new(horizontal: bool, virtualized: bool, cx: &mut Context<Self>) -> Self {
        let snapshot = ListBoxSnapshot::try_with_enabled(1..=10_000, |n| *n, |n| *n != 7).unwrap();
        Self {
            list: ListBoxControl::new(
                ListBoxState::from_snapshot(snapshot, SelectionMode::Extended),
                Self::input,
                |key| ("item", *key).into(),
                |n| format!("Item {n}").into(),
                cx,
            )
            .require_focus_for_scroll(true)
            .virtualization(if virtualized {
                ListBoxVirtualization::Uniform { overscan: 2 }
            } else {
                ListBoxVirtualization::Eager
            }),
            look: ShadcnLook::built_in(),
            horizontal,
            width: 250.0,
            visible: 5,
            rendered: Rc::default(),
            parent_wheels: 0,
        }
    }

    fn input(&mut self, input: ListBoxInput<u32>, _: &mut Window, cx: &mut Context<Self>) {
        self.list.apply(input, cx);
    }
}

impl Render for Harness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.rendered.borrow_mut().clear();
        // Deliberately borrow a local non-Send value: virtualization must not
        // impose a retained/'static template callback.
        let rendered = self.rendered.clone();
        let flow = if self.horizontal {
            ListBoxFlow::Horizontal { item_width: 136.0, item_height: 36.0, gap: 8.0 }
        } else {
            ListBoxFlow::Vertical { visible_items: self.visible, item_height: 36.0, gap: 4.0 }
        };
        let surface = self
            .look
            .render_listbox(
                &mut self.list,
                "virtual-list",
                flow,
                (8.0, 8.0),
                |model, _| {
                    rendered.borrow_mut().push(*model.item);
                    div().size_full().child(model.item.to_string())
                },
                window,
                cx,
            )
            .w(px(self.width))
            .flex_shrink_0();
        div().id("parent").child(surface).on_scroll(cx.listener(|this, _, _, _| this.parent_wheels += 1))
    }
}

fn settle(cx: &mut VisualTestContext) {
    // TestWindow has no platform frame loop; deliver requested layout frames.
    for _ in 0..4 {
        cx.run_until_parked();
        if cx.update(|window, app| window.simulate_next_frame(app)) == 0 {
            break;
        }
    }
    cx.run_until_parked();
}

fn bounds(cx: &mut VisualTestContext, selector: &'static str) -> Bounds<Pixels> {
    cx.debug_bounds(selector).unwrap_or_else(|| panic!("missing {selector}"))
}

fn assert_revealed(cx: &mut VisualTestContext, selector: &'static str, horizontal: bool) {
    settle(cx);
    let viewport = bounds(cx, "virtual-list-viewport");
    let item = bounds(cx, selector);
    if horizontal {
        assert!(
            item.left() >= viewport.left() && item.right() <= viewport.right(),
            "{selector}: {item:?} vs {viewport:?}"
        );
    } else {
        assert!(
            item.top() >= viewport.top() && item.bottom() <= viewport.bottom(),
            "{selector}: {item:?} vs {viewport:?}"
        );
    }
}

fn wheel(cx: &mut VisualTestContext, horizontal: bool, delta: f32) {
    cx.simulate_event(ScrollEvent {
        position: point(px(30.0), px(25.0)),
        delta: ScrollDelta::Pixels(if horizontal {
            point(px(delta), px(0.0))
        } else {
            point(px(0.0), px(delta))
        }),
        modifiers: Default::default(),
        source: gpui::ScrollSource::Pointer,
        touch_phase: TouchPhase::Moved,
    });
    settle(cx);
}

#[test]
fn listbox_projection_renders_and_positions_visible_keys_on_both_axes() {
    for horizontal in [false, true] {
        for virtualized in [false, true] {
            let mut app = TestAppContext::single();
            let (view, cx) = app.add_window_view(|_, cx| Harness::new(horizontal, virtualized, cx));
            settle(cx);
            cx.update(|_, app| {
                view.update(app, |host, cx| {
                    host.list.apply(ListBoxInput::Select(1), cx);
                    host.list.scroll_to_center(9_000, cx);
                })
            });
            settle(cx);
            cx.update(|_, app| {
                view.update(app, |host, cx| {
                    host.list.set_projection((1..=250).rev().map(|n| n * 2), cx).unwrap();
                    host.list.scroll_to_center(200, cx);
                })
            });
            assert_revealed(cx, "virtual-list-item-150", horizontal);
            cx.update(|_, app| {
                let host = view.read(app);
                assert_eq!(host.list.state.snapshot().items().len(), 10_000);
                assert_eq!(host.list.state.selected_key(), Some(&1));
                assert_eq!(host.list.state.visible_index(&1), None);
                let rows = host.rendered.borrow();
                assert!(rows.contains(&200));
                assert!(rows.iter().all(|n| n % 2 == 0 && *n <= 500));
                assert!(rows.windows(2).all(|pair| pair[0] > pair[1]));
                if virtualized {
                    assert!(rows.len() <= 10);
                }
            });
            let before = bounds(cx, "virtual-list-item-150");
            cx.update(|_, app| view.update(app, |host, cx| host.list.scroll_to(1, cx)));
            settle(cx);
            assert_eq!(bounds(cx, "virtual-list-item-150"), before); // hidden key ignored
            cx.update(|_, app| {
                view.update(app, |host, cx| {
                    host.list.set_projection([], cx).unwrap();
                })
            });
            settle(cx);
            cx.update(|_, app| {
                view.update(app, |host, cx| {
                    assert!(host.rendered.borrow().is_empty());
                    host.list.reset_projection(cx);
                    host.list.scroll_to(1, cx);
                })
            });
            assert_revealed(cx, "virtual-list-item-0", horizontal);
        }
    }
}

#[test]
fn listbox_virtualization_bounds_templates_and_reveals_offscreen_selection_on_both_axes() {
    for horizontal in [false, true] {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| Harness::new(horizontal, true, cx));
        cx.update(|window, _| window.activate_window());
        settle(cx);
        cx.update(|_, app| assert_eq!(view.read(app).rendered.borrow().len(), if horizontal { 4 } else { 7 }));
        cx.simulate_click(point(px(30.0), px(25.0)), Default::default());
        cx.simulate_keystrokes("shift-end");
        assert_revealed(cx, "virtual-list-item-9999", horizontal);
        cx.update(|_, app| {
            let host = view.read(app);
            assert_eq!(host.list.state.active_key(), Some(&10_000));
            assert_eq!(host.list.state.selected_keys().count(), 9_999); // disabled seventh item
            let rendered = host.rendered.borrow();
            assert!(rendered.len() <= 9 && rendered.contains(&10_000) && !rendered.contains(&1));
        });
        // Resize with the active item at the far end; preserve its reveal.
        cx.update(|_, app| {
            view.update(app, |host, cx| {
                host.width = 200.0;
                host.visible = 3;
                cx.notify();
            })
        });
        assert_revealed(cx, "virtual-list-item-9999", horizontal);
        cx.simulate_keystrokes("home");
        assert_revealed(cx, "virtual-list-item-0", horizontal);
        cx.simulate_keystrokes(if horizontal {
            "right right right right right right"
        } else {
            "down down down down down down"
        });
        assert_revealed(cx, "virtual-list-item-7", horizontal);
        cx.update(|_, app| assert_eq!(view.read(app).list.state.active_key(), Some(&8)));
    }
}

#[test]
fn listbox_virtualization_wheel_focus_endpoints_and_snapshot_shrink() {
    for horizontal in [false, true] {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| Harness::new(horizontal, true, cx));
        cx.update(|window, _| window.activate_window());
        settle(cx);
        let first = bounds(cx, "virtual-list-item-0");
        wheel(cx, horizontal, -10_000.0);
        assert_eq!(bounds(cx, "virtual-list-item-0"), first);
        cx.simulate_click(point(px(30.0), px(25.0)), Default::default());
        wheel(cx, horizontal, -10_000.0);
        cx.update(|_, app| {
            let host = view.read(app);
            assert_eq!(host.parent_wheels, 1);
            assert!(!host.rendered.borrow().contains(&1));
            assert!(host.rendered.borrow().len() <= 10);
        });
        wheel(cx, horizontal, -10_000_000.0);
        assert_revealed(cx, "virtual-list-item-9999", horizontal);
        wheel(cx, horizontal, -100.0);
        assert_revealed(cx, "virtual-list-item-9999", horizontal);
        cx.update(|_, app| assert_eq!(view.read(app).parent_wheels, 1));
        cx.update(|window, app| {
            let elsewhere = app.focus_handle();
            elsewhere.focus(window, app);
        });
        settle(cx);
        wheel(cx, horizontal, 500.0);
        assert_revealed(cx, "virtual-list-item-9999", horizontal);
        cx.update(|_, app| assert_eq!(view.read(app).parent_wheels, 2));
        for count in [3, 0, 20] {
            cx.update(|_, app| {
                view.update(app, |host, cx| {
                    let update = host.list.state.replace_snapshot(ListBoxSnapshot::try_new(1..=count, |n| *n).unwrap());
                    host.list.handle_update(&update, cx);
                })
            });
            settle(cx);
            cx.update(|_, app| {
                let rendered = view.read(app).rendered.borrow();
                assert!(rendered.len() <= 9);
                if count == 0 {
                    assert!(rendered.is_empty());
                } else {
                    assert!(rendered.contains(&1));
                }
            });
        }
    }
}

#[test]
fn listbox_eager_rendering_remains_the_default() {
    assert_eq!(ListBoxVirtualization::default(), ListBoxVirtualization::Eager);
    let mut app = TestAppContext::single();
    let (view, cx) = app.add_window_view(|_, cx| {
        let mut host = Harness::new(false, false, cx);
        host.list.state.replace_snapshot(ListBoxSnapshot::try_new(1..=20, |n| *n).unwrap());
        host
    });
    cx.update(|_, app| assert_eq!(view.read(app).rendered.borrow().len(), 20));
}

#[test]
fn listbox_scroll_to_positions_the_requested_key_without_changing_selection() {
    for horizontal in [false, true] {
        for virtualized in [false, true] {
            let mut app = TestAppContext::single();
            let (view, cx) = app.add_window_view(|_, cx| {
                let mut host = Harness::new(horizontal, virtualized, cx);
                host.list.state.replace_snapshot(ListBoxSnapshot::try_new(1..=100, |n| *n).unwrap());
                host.list.state.set_selected(Some(7)).unwrap();
                // Request before the first viewport measurement, without focus.
                host.list.scroll_to_center(50, cx);
                host
            });
            settle(cx);
            let viewport = bounds(cx, "virtual-list-viewport");
            let row = bounds(cx, "virtual-list-item-49");
            let center = |b: Bounds<Pixels>| {
                if horizontal {
                    (b.left() + b.right()) / 2.0
                } else {
                    (b.top() + b.bottom()) / 2.0
                }
            };
            assert!(
                (center(row) - center(viewport)).abs() < px(0.1),
                "horizontal={horizontal}, virtual={virtualized}: {row:?} vs {viewport:?}"
            );
            cx.update(|_, app| view.update(app, |host, cx| host.list.scroll_to(50, cx)));
            settle(cx);
            assert_eq!(bounds(cx, "virtual-list-item-49"), row, "visible item should not move");
            // Latest explicit request wins; requesting a missing key is a no-op.
            cx.update(|_, app| {
                view.update(app, |host, cx| {
                    host.list.scroll_to(1, cx);
                    host.list.scroll_to_center(100, cx);
                })
            });
            assert_revealed(cx, "virtual-list-item-99", horizontal);
            let last = bounds(cx, "virtual-list-item-99");
            assert_eq!(
                if horizontal { last.right() } else { last.bottom() },
                if horizontal {
                    viewport.right()
                } else {
                    viewport.bottom()
                }
            );
            cx.update(|_, app| view.update(app, |host, cx| host.list.scroll_to_center(999, cx)));
            settle(cx);
            assert_eq!(bounds(cx, "virtual-list-item-99"), last);
            // Resolve against the latest snapshot, not the index at request time.
            cx.update(|_, app| {
                view.update(app, |host, cx| {
                    host.list.scroll_to(1, cx);
                    let update =
                        host.list.state.replace_snapshot(ListBoxSnapshot::try_new((1..=100).rev(), |n| *n).unwrap());
                    host.list.handle_update(&update, cx);
                })
            });
            assert_revealed(cx, "virtual-list-item-99", horizontal);
            cx.update(|_, app| {
                let state = &view.read(app).list.state;
                assert_eq!(state.selected_key(), Some(&7));
                assert!(!state.is_focused());
            });
        }
    }
}

#[test]
fn listbox_smooth_positions_animate_both_axes_and_yield_to_new_requests() {
    for horizontal in [false, true] {
        for virtualized in [false, true] {
            let mut app = TestAppContext::single();
            let (view, cx) = app.add_window_view(|_, cx| {
                let mut host = Harness::new(horizontal, virtualized, cx);
                host.list.state.replace_snapshot(ListBoxSnapshot::try_new(1..=100, |n| *n).unwrap());
                host.list.state.set_selected(Some(7)).unwrap();
                host
            });
            cx.update(|window, _| window.activate_window());
            settle(cx);
            cx.update(|_, app| view.update(app, |host, cx| host.list.scroll_to_center_smooth(50, cx)));
            cx.run_until_parked();
            let beginning = bounds(cx, "virtual-list-item-0");
            advance_smooth(cx, 1);
            let keys = cx.update(|_, app| view.read(app).rendered.borrow().clone());
            let item_id: &'static str =
                Box::leak(format!("virtual-list-item-{}", keys[keys.len() / 2] - 1).into_boxed_str());
            let moving = bounds(cx, item_id);
            advance_smooth(cx, 1);
            if let Some(next) = cx.debug_bounds(item_id) {
                assert!(if horizontal {
                    next.left() < moving.left()
                } else {
                    next.top() < moving.top()
                });
            }
            advance_smooth(cx, 24);
            let viewport = bounds(cx, "virtual-list-viewport");
            let row = bounds(cx, "virtual-list-item-49");
            let error = if horizontal {
                row.left() + row.right() - viewport.left() - viewport.right()
            } else {
                row.top() + row.bottom() - viewport.top() - viewport.bottom()
            };
            assert!(error.abs() < px(0.2));
            assert_ne!(row, beginning);
            // An immediate request supersedes the animator and stays in place.
            cx.update(|_, app| view.update(app, |host, cx| host.list.scroll_to_smooth(100, cx)));
            advance_smooth(cx, 2);
            cx.update(|_, app| view.update(app, |host, cx| host.list.scroll_to(1, cx)));
            advance_smooth(cx, 24);
            assert_revealed(cx, "virtual-list-item-0", horizontal);
            cx.update(|_, app| {
                assert_eq!(view.read(app).list.state.selected_key(), Some(&7));
                assert!(!view.read(app).list.state.is_focused());
            });
            // A wheel gesture stops the current motion at its current offset.
            cx.simulate_click(point(px(30.0), px(25.0)), Default::default());
            settle(cx);
            cx.update(|_, app| view.update(app, |host, cx| host.list.scroll_to_center_smooth(50, cx)));
            advance_smooth(cx, 3);
            wheel(cx, horizontal, -13.0);
            let key = cx.update(|_, app| *view.read(app).rendered.borrow().first().unwrap());
            let id: &'static str = Box::leak(format!("virtual-list-item-{}", key - 1).into_boxed_str());
            let stopped = bounds(cx, id);
            advance_smooth(cx, 24);
            assert_eq!(bounds(cx, id), stopped);
        }
    }
}

fn advance_smooth(cx: &mut VisualTestContext, frames: usize) {
    cx.run_until_parked();
    for _ in 0..frames {
        cx.executor().advance_clock(std::time::Duration::from_millis(16));
        cx.update(|window, app| window.simulate_next_frame(app));
        cx.run_until_parked();
    }
}
