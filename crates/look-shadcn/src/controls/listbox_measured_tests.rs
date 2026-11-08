//! Content determines row height; the scroll adapter only measures its result.
use std::{cell::RefCell, rc::Rc};
use gpui::{Bounds, Pixels, ScrollDelta, ScrollEvent, TestAppContext, TouchPhase, VisualTestContext, point};
use xgpui_luma::controls::listbox::{ListBoxSnapshot, SelectionMode};
use super::*;

struct Harness {
    list: ListBoxControl<Self, u32, u32>,
    look: ShadcnLook,
    width: f32,
    expanded: bool,
    rendered: Rc<RefCell<Vec<u32>>>,
    parent_wheels: usize,
}

impl Harness {
    fn new(virtualized: bool, cx: &mut Context<Self>) -> Self {
        Self {
            list: ListBoxControl::new(
                ListBoxState::try_new(1..=1_000, |n| *n, SelectionMode::Extended).unwrap(),
                Self::input,
                |key| ("row", *key).into(),
                |n| format!("Item {n}").into(),
                cx,
            )
            .require_focus_for_scroll(true)
            .virtualization(if virtualized {
                ListBoxVirtualization::Measured { estimated_height: 48.0, overscan: 2 }
            } else {
                ListBoxVirtualization::Eager
            }),
            look: ShadcnLook::built_in(),
            width: 250.0,
            expanded: false,
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
        let rendered = self.rendered.clone();
        let expanded = self.expanded;
        let surface = self
            .look
            .render_listbox(
                &mut self.list,
                "measured-list",
                ListBoxFlow::VerticalContent { viewport_height: 240.0, gap: 4.0 },
                (8.0, 8.0),
                |model, _| {
                    rendered.borrow_mut().push(*model.item);
                    // Real wrapping content, not a computed outer row height.
                    let copies = if expanded { 15 } else { (*model.item as usize % 3) + 1 };
                    div()
                        .w_full()
                        .px(px(6.0))
                        .py(px(4.0))
                        .text_size(px(14.0))
                        .line_height(px(18.0))
                        .child("A row with wrapping text. ".repeat(copies))
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
    for _ in 0..30 {
        cx.run_until_parked();
        if cx.update(|window, app| window.simulate_next_frame(app)) == 0 {
            return;
        }
    }
    panic!("measured layout did not settle");
}

fn bounds(cx: &mut VisualTestContext, selector: &'static str) -> Bounds<Pixels> {
    cx.debug_bounds(selector).unwrap_or_else(|| panic!("missing {selector}"))
}

fn wheel(cx: &mut VisualTestContext, delta: f32) {
    cx.simulate_event(ScrollEvent {
        position: point(px(30.0), px(25.0)),
        delta: ScrollDelta::Pixels(point(px(0.0), px(delta))),
        modifiers: Default::default(),
        source: gpui::ScrollSource::Pointer,
        touch_phase: TouchPhase::Moved,
    });
    settle(cx);
}

#[test]
fn listbox_projection_rebuilds_measured_geometry_without_replacing_source() {
    for virtualized in [false, true] {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| Harness::new(virtualized, cx));
        settle(cx);
        cx.update(|_, app| {
            view.update(app, |host, cx| {
                host.list.apply(ListBoxInput::Select(1), cx);
                host.list.scroll_to_center(990, cx);
            })
        });
        settle(cx);
        cx.update(|_, app| {
            view.update(app, |host, cx| {
                host.list.set_projection((1..=100).rev().map(|n| n * 7), cx).unwrap();
                host.list.scroll_to_center(350, cx);
            })
        });
        settle(cx);
        let row = bounds(cx, "measured-list-item-50");
        let viewport = bounds(cx, "measured-list-viewport");
        assert!((row.center().y - viewport.center().y).abs() < px(1.0));
        cx.update(|_, app| {
            let host = view.read(app);
            assert_eq!(host.list.state.snapshot().items().len(), 1_000);
            assert_eq!(host.list.state.selected_key(), Some(&1));
            let rendered = host.rendered.borrow();
            assert!(rendered.contains(&350));
            assert!(rendered.iter().all(|key| key % 7 == 0));
            if virtualized {
                assert!(rendered.len() < 15);
            }
        });
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
                host.list.scroll_to_center(350, cx);
            })
        });
        settle(cx);
        let row = bounds(cx, "measured-list-item-349");
        assert!((row.center().y - viewport.center().y).abs() < px(1.0));
    }
}

#[test]
fn listbox_content_height_wheel_reveal_resize_and_replacement() {
    for virtualized in [false, true] {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| Harness::new(virtualized, cx));
        cx.update(|window, _| window.activate_window());
        settle(cx);
        let first = bounds(cx, "measured-list-item-0");
        let second = bounds(cx, "measured-list-item-1");
        assert!(second.size.height > first.size.height);
        assert_eq!(second.top() - first.bottom(), px(4.0));
        cx.update(|_, app| {
            let count = view.read(app).rendered.borrow().len();
            if virtualized {
                assert!(count < 20);
            } else {
                assert_eq!(count, 1_000);
            }
        });
        wheel(cx, -700.0);
        assert_eq!(bounds(cx, "measured-list-item-0"), first);
        cx.simulate_click(point(px(30.0), px(25.0)), Default::default());
        wheel(cx, -700.0);
        let window_before =
            cx.update(|_, app| view.update(app, |host, _| host.list.render_parts().scroll.rendered_window().unwrap()));
        assert!(window_before.visible_range.start > 0);
        let anchor_selector: &'static str =
            Box::leak(format!("measured-list-item-{}", window_before.visible_range.start).into_boxed_str());
        let anchor_top = bounds(cx, anchor_selector).top();
        // Width invalidation reflows rows while retaining the top visible key.
        cx.update(|_, app| {
            view.update(app, |host, cx| {
                host.width = 180.0;
                cx.notify();
            })
        });
        settle(cx);
        assert!((bounds(cx, anchor_selector).top() - anchor_top).abs() <= px(1.0));
        cx.simulate_keystrokes("end enter");
        settle(cx);
        let viewport = bounds(cx, "measured-list-viewport");
        let last = bounds(cx, "measured-list-item-999");
        assert!(last.top() >= viewport.top() && last.bottom() <= viewport.bottom(), "{last:?} / {viewport:?}");
        cx.update(|_, app| {
            let host = view.read(app);
            assert_eq!(host.list.state.selected_keys().copied().collect::<Vec<_>>(), vec![1_000]);
            assert_eq!(host.parent_wheels, 1);
        });
        wheel(cx, -1_000_000.0);
        assert_eq!(bounds(cx, "measured-list-item-999").bottom(), viewport.bottom());
        cx.simulate_keystrokes("home");
        settle(cx);
        // An external template change can invalidate measurements without replacing data.
        cx.update(|_, app| {
            view.update(app, |host, cx| {
                host.expanded = true;
                host.list.invalidate_measurements(None, cx);
            })
        });
        settle(cx);
        let tall = bounds(cx, "measured-list-item-0");
        assert!(tall.size.height > viewport.size.height);
        cx.simulate_keystrokes("end");
        settle(cx);
        assert_eq!(bounds(cx, "measured-list-item-999").top(), viewport.top());
        // Shrinking the final oversized row must fill the viewport and remain
        // at the collection end after GPUI clamps the newly shorter content.
        cx.update(|_, app| {
            view.update(app, |host, cx| {
                host.expanded = false;
                host.list.invalidate_measurements(None, cx);
            })
        });
        settle(cx);
        assert_eq!(bounds(cx, "measured-list-item-999").bottom(), viewport.bottom());

        for items in [vec![1, 2, 3], vec![], vec![3, 1, 2]] {
            cx.update(|_, app| {
                view.update(app, |host, cx| {
                    let update = host.list.state.replace_snapshot(ListBoxSnapshot::try_new(items, |n| *n).unwrap());
                    host.list.handle_update(&update, cx);
                })
            });
            settle(cx);
        }
        cx.simulate_keystrokes("home");
        settle(cx);
        assert_eq!(bounds(cx, "measured-list-item-0").top(), viewport.top());
    }
}

#[test]
fn listbox_scroll_to_center_uses_the_requested_rows_measured_height() {
    for virtualized in [false, true] {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| {
            let mut host = Harness::new(virtualized, cx);
            host.list.state.set_selected(Some(7)).unwrap();
            host.list.scroll_to_center(500, cx);
            host
        });
        cx.update(|window, _| window.activate_window());
        settle(cx);
        let viewport = bounds(cx, "measured-list-viewport");
        let row = bounds(cx, "measured-list-item-499");
        assert!(((row.top() + row.bottom() - viewport.top() - viewport.bottom()) / 2.0).abs() < px(0.1));
        cx.update(|_, app| view.update(app, |host, cx| host.list.scroll_to(500, cx)));
        settle(cx);
        assert_eq!(bounds(cx, "measured-list-item-499"), row);
        // Center the selected key itself, not a nearest/active/collection-middle item.
        cx.update(|_, app| view.update(app, |host, cx| host.list.scroll_to_center(7, cx)));
        settle(cx);
        let selected = bounds(cx, "measured-list-item-6");
        assert!(((selected.top() + selected.bottom() - viewport.top() - viewport.bottom()) / 2.0).abs() < px(0.1));
        cx.update(|_, app| {
            view.update(app, |host, cx| {
                host.expanded = true;
                host.list.invalidate_measurements(None, cx);
                host.list.scroll_to_center(700, cx);
            })
        });
        settle(cx);
        let oversized = bounds(cx, "measured-list-item-699");
        assert!(oversized.size.height > viewport.size.height);
        assert!(((oversized.top() + oversized.bottom() - viewport.top() - viewport.bottom()) / 2.0).abs() < px(0.1));
        cx.update(|_, app| view.update(app, |host, cx| host.list.scroll_to(700, cx)));
        settle(cx);
        assert!((bounds(cx, "measured-list-item-699").top() - viewport.top()).abs() < px(0.1));
        cx.update(|_, app| {
            let state = &view.read(app).list.state;
            assert_eq!(state.selected_key(), Some(&7));
            assert!(!state.is_focused());
        });
        // Once fulfilled, the request must not pull the list back during scrolling.
        cx.simulate_click(point(px(30.0), px(25.0)), Default::default());
        settle(cx);
        let before = bounds(cx, "measured-list-item-699");
        wheel(cx, -20.0);
        assert!((bounds(cx, "measured-list-item-699").top() - before.top() + px(20.0)).abs() < px(0.1));
    }
}

#[test]
fn listbox_smooth_center_tracks_offscreen_measured_rows_and_allows_interruption() {
    for virtualized in [false, true] {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| Harness::new(virtualized, cx));
        cx.update(|window, _| window.activate_window());
        settle(cx);
        let drive = |cx: &mut VisualTestContext, count| {
            cx.run_until_parked();
            for _ in 0..count {
                cx.executor().advance_clock(std::time::Duration::from_millis(16));
                cx.update(|window, app| window.simulate_next_frame(app));
                cx.run_until_parked();
            }
        };
        for target in [500, 7, 700] {
            cx.update(|_, app| view.update(app, |host, cx| host.list.scroll_to_center_smooth(target, cx)));
            drive(cx, 32);
            let id: &'static str = Box::leak(format!("measured-list-item-{}", target - 1).into_boxed_str());
            let item = bounds(cx, id);
            let viewport = bounds(cx, "measured-list-viewport");
            assert!(
                ((item.top() + item.bottom() - viewport.top() - viewport.bottom()) / 2.0).abs() < px(0.1),
                "virtual={virtualized} key={target}: {item:?} {viewport:?}"
            );
        }
        // Content replacement during travel must cancel a stale target.
        cx.update(|_, app| view.update(app, |host, cx| host.list.scroll_to_smooth(500, cx)));
        drive(cx, 2);
        cx.update(|_, app| {
            view.update(app, |host, cx| {
                let update = host.list.state.replace_snapshot(ListBoxSnapshot::try_new(1..=3, |n| *n).unwrap());
                host.list.handle_update(&update, cx);
            })
        });
        drive(cx, 32);
        assert_eq!(bounds(cx, "measured-list-item-0").top(), bounds(cx, "measured-list-viewport").top());
    }
}
