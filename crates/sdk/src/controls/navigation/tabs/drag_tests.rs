//! Headless native gesture dispatch; no platform application is launched.
use super::*;
use gpui::{
    Context, Entity, Render, Window, TestAppContext, VisualTestContext, IntoElement, MouseButton, div, point, px,
    prelude::*,
};
use crate::infra::drag_drop::{DragDropEvent as E, DragEndReason};
struct Page {
    tabs: Entity<Tabs>,
    other: Entity<Tabs>,
    events: Vec<TabsEvent>,
    named: Vec<&'static str>,
}
impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(450.)).flex().flex_col().child(self.tabs.clone()).child(self.other.clone())
    }
}
fn setup(app: &mut TestAppContext, mode: u8) -> (Entity<Page>, &mut VisualTestContext) {
    let (page, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        crate::key_handling::bind_default_control_keys(cx);
        let tabs = Tabs::new("drag-tabs")
            .reordering(true)
            .active("b")
            .motion_enabled(false)
            .motion(|m| {
                m.drop_layout = crate::motion::Transition::timed(
                    std::time::Duration::from_millis(1000),
                    crate::motion::Easing::OutCubic,
                )
            })
            .items([
                TabsItem::new("a"),
                TabsItem::new("b"),
                TabsItem::new("c"),
                TabsItem::new("disabled").enabled(false),
                TabsItem::new("menu").dropdown_trigger(),
            ])
            .on_event(cx, |owner: &mut Page, event, _| owner.events.push(event.clone()))
            .on_drag_drop(cx, |owner, _, _| owner.named.push("aggregate"))
            .on_drag_started(cx, |owner, _, _, _| owner.named.push("start"))
            .on_items_reordered(cx, |owner, _, _, _, _| owner.named.push("reorder"))
            .on_dropped(cx, |owner, _, _, _, _, _, cx| {
                owner.named.push("drop");
                cx.notify();
            })
            .on_drop_rejected(cx, |owner, _, _, _, _, _, _| owner.named.push("reject"))
            .on_drag_ended(cx, |owner, _, _, _, _| owner.named.push("end"))
            .spawn(cx);
        if mode != 0 {
            tabs.update(cx, |tabs, _| {
                tabs.set_drop_handler(move |tabs, proposal, _, cx| {
                    if mode == 1 {
                        tabs.reject_drop(proposal, "policy", cx);
                    }
                })
            });
        }
        Page {
            other: Tabs::new("other").reordering(true).items([TabsItem::new("x"), TabsItem::new("y")]).spawn(cx),
            tabs,
            events: vec![],
            named: vec![],
        }
    });
    cx.run_until_parked();
    (page, cx)
}
fn drag(cx: &mut VisualTestContext, from: &str, to: &str, fraction: f32) {
    let source = cx.debug_bounds(Box::leak(format!("tab-drag-{from}").into_boxed_str())).unwrap();
    let target = cx.debug_bounds(Box::leak(format!("tab-drag-{to}").into_boxed_str())).unwrap();
    let start = source.center();
    let end = point(target.left() + target.size.width * fraction, target.center().y);
    cx.simulate_mouse_down(start, MouseButton::Left, Default::default());
    cx.simulate_mouse_move(point(start.x + px(12.), start.y), MouseButton::Left, Default::default());
    cx.run_until_parked();
    cx.simulate_mouse_move(end, MouseButton::Left, Default::default());
    cx.run_until_parked();
    cx.simulate_mouse_up(end, MouseButton::Left, Default::default());
    cx.run_until_parked();
}
#[test]
fn native_reorder_preserves_active_and_reports_exact_order_without_activation() {
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, 0);
    drag(cx, "a", "c", 0.8);
    cx.update(|_, cx| {
        let page = page.read(cx);
        let tabs = page.tabs.read(cx);
        assert_eq!(
            tabs.items().iter().map(|i| i.id().as_ref()).collect::<Vec<_>>(),
            ["b", "c", "a", "disabled", "menu"]
        );
        assert_eq!(tabs.active_id.as_deref(), Some("b"));
        assert!(!page.events.iter().any(|e| matches!(
            e,
            TabsEvent::Change { .. }
                | TabsEvent::Activate { .. }
                | TabsEvent::Reactivate { .. }
                | TabsEvent::DropdownRequested { .. }
        )));
        let events = page
            .events
            .iter()
            .filter_map(|e| if let TabsEvent::DragDrop(e) = e { Some(e) } else { None })
            .collect::<Vec<_>>();
        assert_eq!(events.len(), 4);
        assert!(matches!(events[0], E::DragStarted { keys, .. } if keys == &[gpui::SharedString::from("a")]));
        assert!(matches!(events[1], E::ItemsReordered { before, .. } if before.as_deref()==Some("disabled")));
        assert!(matches!(events[2], E::Dropped { changed: true, .. }));
        assert!(matches!(events[3], E::DragEnded { reason: DragEndReason::Reordered, .. }));
        assert_eq!(page.named, ["aggregate", "start", "aggregate", "reorder", "aggregate", "drop", "aggregate", "end"]);
    });
}
#[test]
fn native_noop_rejection_unanswered_and_disabled_sources() {
    for mode in 0..3 {
        let mut app = TestAppContext::single();
        let (page, cx) = setup(&mut app, mode);
        drag(cx, "a", "a", 0.2);
        cx.update(|_, cx| {
            let page = page.read(cx);
            let events = page
                .events
                .iter()
                .filter_map(|e| if let TabsEvent::DragDrop(e) = e { Some(e) } else { None })
                .collect::<Vec<_>>();
            assert_eq!(events.len(), 3);
            assert!(
                matches!(events[1], E::Dropped { changed: false, .. }) && mode == 0
                    || matches!(events[1], E::DropRejected { .. }) && mode != 0
            );
        });
        page.update(cx, |p, _| {
            p.events.clear();
            p.named.clear();
        });
        drag(cx, "disabled", "c", 0.8);
        drag(cx, "menu", "c", 0.8);
        cx.update(|_, cx| assert!(!page.read(cx).events.iter().any(|e| matches!(e, TabsEvent::DragDrop(_)))));
    }
}

#[test]
fn escape_foreign_strip_and_data_replacement_end_once() {
    for action in 0..3 {
        let mut app = TestAppContext::single();
        let (page, cx) = setup(&mut app, 0);
        let start = cx.debug_bounds("tab-drag-a").unwrap().center();
        cx.simulate_mouse_down(start, MouseButton::Left, Default::default());
        cx.simulate_mouse_move(point(start.x + px(12.), start.y), MouseButton::Left, Default::default());
        cx.run_until_parked();
        if action == 0 {
            cx.simulate_keystrokes("escape");
        } else if action == 1 {
            let end = cx.debug_bounds("tab-drag-x").unwrap().center();
            cx.simulate_mouse_move(end, MouseButton::Left, Default::default());
            cx.run_until_parked();
            cx.simulate_mouse_up(end, MouseButton::Left, Default::default());
        } else {
            let tabs = cx.update(|_, cx| page.read(cx).tabs.clone());
            tabs.update(cx, |tabs, cx| tabs.set_items(tabs.items().to_vec(), cx));
            cx.simulate_mouse_up(start, MouseButton::Left, Default::default());
        }
        cx.run_until_parked();
        cx.update(|_, cx| {
            let page = page.read(cx);
            let ends = page
                .events
                .iter()
                .filter(|e| matches!(e, TabsEvent::DragDrop(E::DragEnded { reason: DragEndReason::Cancelled, .. })))
                .count();
            assert_eq!(ends, 1, "action {action}");
            assert_eq!(page.tabs.read(cx).items()[0].id().as_ref(), "a");
        });
    }
}
#[test]
fn retained_view_focus_and_body_motion_survive_native_move_and_stale_commit_fails() {
    use std::{cell::RefCell, rc::Rc};
    use gpui::Focusable;
    struct Panel(usize);
    impl Render for Panel {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().debug_selector(|| "retained-drag-panel".into()).child(self.0.to_string())
        }
    }
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, 0);
    let tabs = cx.update(|_, cx| page.read(cx).tabs.clone());
    let panel = cx.update(|_, cx| cx.new(|_| Panel(42)));
    let weak = panel.downgrade();
    let view = panel.clone();
    tabs.update(cx, |tabs, cx| {
        tabs.set_content("b", TabsContent::new(move |_, _| view.clone()), cx);
    });
    drop(panel);
    cx.run_until_parked();
    assert!(cx.debug_bounds("retained-drag-panel").is_some());
    let captured = Rc::new(RefCell::new(None));
    let capture = captured.clone();
    tabs.update(cx, |tabs, _| {
        tabs.set_drop_handler(move |tabs, p, _, cx| {
            *capture.borrow_mut() = Some(p.clone());
            tabs.commit_drop(p, cx).unwrap();
        })
    });
    drag(cx, "a", "c", 0.8);
    cx.update(|window, cx| {
        let control = tabs.read(cx);
        assert_eq!(control.body_transition.progress(), 1.);
        assert!(control.focus_handle(cx).contains_focused(window, cx));
        assert_eq!(weak.upgrade().unwrap().read(cx).0, 42);
    });
    tabs.update(cx, |tabs, cx| assert!(tabs.commit_drop(captured.borrow().as_ref().unwrap(), cx).is_err()));
    cx.run_until_parked();
    cx.update(|_, cx| {
        assert_eq!(
            page.read(cx)
                .events
                .iter()
                .filter(|e| matches!(e, TabsEvent::DragDrop(E::DragEnded { .. })))
                .count(),
            1
        )
    });
}

#[test]
fn overflowing_strip_scrolls_during_admitted_drag_and_stops_after_escape() {
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, 0);
    let tabs = cx.update(|_, cx| page.read(cx).tabs.clone());
    tabs.update(cx, |tabs, cx| {
        tabs.set_items((0..40).map(|i| TabsItem::new(format!("t{i}")).label(format!("Document number {i}"))), cx)
    });
    cx.run_until_parked();
    let start = cx.debug_bounds("tab-drag-t0").unwrap().center();
    let bounds = cx.update(|_, cx| tabs.read(cx).drag_scroll.bounds());
    assert!(bounds.size.width <= px(450.));
    let end = point(bounds.right() - px(2.), bounds.center().y);
    cx.simulate_mouse_down(start, MouseButton::Left, Default::default());
    cx.simulate_mouse_move(start + point(px(12.), px(0.)), MouseButton::Left, Default::default());
    cx.run_until_parked();
    cx.simulate_mouse_move(end, MouseButton::Left, Default::default());
    cx.run_until_parked();
    cx.update(|window, cx| {
        window.simulate_next_frame(cx);
    });
    cx.run_until_parked();
    let offset = cx.update(|_, cx| tabs.read(cx).drag_scroll.offset().x);
    assert!(
        offset < px(0.),
        "bounds {bounds:?}, max {:?}, pending {}, active {}",
        cx.update(|_, cx| tabs.read(cx).drag_scroll.max_offset()),
        cx.update(|_, cx| tabs.read(cx).drag_scroll_pending.get()),
        cx.update(|_, cx| cx.has_active_drag())
    );
    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    cx.update(|window, cx| {
        window.simulate_next_frame(cx);
    });
    let stopped = cx.update(|_, cx| tabs.read(cx).drag_scroll.offset().x);
    cx.simulate_mouse_move(end, MouseButton::Left, Default::default());
    cx.run_until_parked();
    cx.update(|_, cx| assert_eq!(tabs.read(cx).drag_scroll.offset().x, stopped));
}

#[test]
fn compact_drag_preview_stays_beside_pointer() {
    let mut app = TestAppContext::single();
    let (_, cx) = setup(&mut app, 0);
    let start = cx.debug_bounds("tab-drag-a").unwrap().center();
    let pointer = start + point(px(12.), px(0.));
    cx.simulate_mouse_down(start, MouseButton::Left, Default::default());
    cx.simulate_mouse_move(pointer, MouseButton::Left, Default::default());
    cx.run_until_parked();
    let preview = cx.debug_bounds("tabs-drag-preview").unwrap();
    assert_eq!(preview.size.width, px(220.));
    assert!((preview.left() - pointer.x - px(12.)).abs() < px(1.));
    assert!((preview.top() - pointer.y - px(12.)).abs() < px(1.));
    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
}

#[test]
fn neighboring_halves_share_one_gap_marker_and_destination() {
    use crate::theme::{ControlSize, InteractionState};
    struct GapTheme(f32);
    impl TabsTheme for GapTheme {
        fn resolve_list(&self, enabled: bool, size: ControlSize) -> TabsListLook {
            let mut look = DefaultTabsTheme::default().resolve_list(enabled, size);
            look.gap = self.0;
            look.padding = 6.;
            look
        }
        fn resolve_item(&self, active: bool, state: InteractionState, size: ControlSize) -> TabsItemLook {
            DefaultTabsTheme::default().resolve_item(active, state, size)
        }
        fn font_family(&self) -> gpui::SharedString {
            DefaultTabsTheme::default().font_family()
        }
    }
    for gap in [0., 12.] {
        for width_mode in [TabsWidthMode::Intrinsic, TabsWidthMode::Uniform] {
            for side in [-2., 2.] {
                let mut app = TestAppContext::single();
                let (page, cx) = setup(&mut app, 0);
                let tabs = cx.update(|_, cx| page.read(cx).tabs.clone());
                tabs.update(cx, |tabs, cx| {
                    tabs.set_width_mode(width_mode, cx);
                    tabs.set_template(
                        std::sync::Arc::new(ThemedTabsTemplate::new(std::sync::Arc::new(GapTheme(gap)))),
                        cx,
                    )
                });
                cx.run_until_parked();
                let start = cx.debug_bounds("tab-drag-a").unwrap().center();
                let b = cx.debug_bounds("tab-drag-b").unwrap();
                let c = cx.debug_bounds("tab-drag-c").unwrap();
                let boundary = (b.right() + c.left()) / 2.;
                cx.simulate_mouse_down(start, MouseButton::Left, Default::default());
                cx.simulate_mouse_move(start + point(px(12.), px(0.)), MouseButton::Left, Default::default());
                cx.run_until_parked();
                let target = cx.debug_bounds("tab-gap-before-c").unwrap();
                assert!((target.left() - b.center().x).abs() < px(0.5));
                assert!((target.right() - c.center().x).abs() < px(0.5));
                let marker = cx.debug_bounds("tab-gap-before-c-marker").unwrap();
                assert!((marker.center().x - boundary).abs() < px(0.5));
                for offset in [-2., 2., side] {
                    cx.simulate_mouse_move(
                        point(boundary + px(offset), b.center().y),
                        MouseButton::Left,
                        Default::default(),
                    );
                    cx.run_until_parked();
                    assert_eq!(cx.debug_bounds("tab-gap-before-c-marker").unwrap(), marker);
                    cx.update(|window, _| {
                        let painted = window
                            .painted_quads()
                            .into_iter()
                            .filter(|quad| {
                                quad.bounds == marker.scale(window.scale_factor())
                                    && quad.background.as_solid().is_some_and(|color| color.a > 0.9)
                            })
                            .count();
                        assert_eq!(painted, 1, "hovered insertion gap must paint one opaque marker");
                    });
                    cx.update(|_, cx| assert_eq!(cx.active_drag_cursor_style(), Some(gpui::CursorStyle::ClosedHand)));
                }
                cx.simulate_mouse_up(point(boundary + px(side), b.center().y), MouseButton::Left, Default::default());
                cx.run_until_parked();
                cx.update(|_, cx| {
                    let page = page.read(cx);
                    assert_eq!(
                        page.tabs.read(cx).items().iter().map(|i| i.id().as_ref()).collect::<Vec<_>>(),
                        ["b", "a", "c", "disabled", "menu"]
                    );
                    let drops = page
                        .events
                        .iter()
                        .filter_map(|e| {
                            if let TabsEvent::DragDrop(E::Dropped { before, .. }) = e {
                                Some(before.as_deref())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    assert_eq!(drops, [Some("c")]);
                });
            }
        }
    }
}

#[test]
fn native_drop_animates_positions_after_lifecycle_completion() {
    use std::time::Duration;
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, 0);
    let tabs = cx.update(|_, cx| page.read(cx).tabs.clone());
    tabs.update(cx, |tabs, cx| {
        tabs.set_motion_enabled(true, cx);
        // Hold the first frame at zero regardless of CI commit/render time.
        tabs.drop_layout_motion().elapse_for_test(Duration::ZERO);
    });
    cx.run_until_parked();
    let old = cx.debug_bounds("tab-drag-a").unwrap();
    drag(cx, "a", "c", 0.8);
    assert!((cx.debug_bounds("tab-drag-a").unwrap().left() - old.left()).abs() < px(1.));
    cx.update(|_, cx| {
        let page = page.read(cx);
        assert_eq!(page.tabs.read(cx).items()[2].id().as_ref(), "a");
        assert!(page.events.iter().any(|e| matches!(e, TabsEvent::DragDrop(E::DragEnded { .. }))));
        assert!(page.tabs.read(cx).drop_layout_motion().is_animating());
    });
    tabs.update(cx, |tabs, _| tabs.drop_layout_motion().elapse_for_test(Duration::from_millis(500)));
    cx.update(|window, cx| {
        window.simulate_next_frame(cx);
    });
    cx.run_until_parked();
    let mid = cx.debug_bounds("tab-drag-a").unwrap().left();
    tabs.update(cx, |tabs, _| tabs.drop_layout_motion().elapse_for_test(Duration::from_millis(1100)));
    cx.update(|window, cx| {
        window.simulate_next_frame(cx);
    });
    cx.run_until_parked();
    let end = cx.debug_bounds("tab-drag-a").unwrap().left();
    assert!(mid > old.left() && mid < end);
    cx.update(|_, cx| assert_eq!(tabs.read(cx).active_id().map(|id| id.as_ref()), Some("b")));
}

#[test]
fn rtl_reorder_uses_logical_before_targets_and_preserves_active_identity() {
    use crate::infra::locale::{LayoutDirection, LocaleContext, set_locale_context};
    let mut app = TestAppContext::single();
    app.update(|cx| set_locale_context(LocaleContext { direction: LayoutDirection::Rtl, ..Default::default() }, cx));
    let (page, cx) = setup(&mut app, 0);
    drag(cx, "a", "c", 0.2);
    cx.update(|_, cx| {
        let page = page.read(cx);
        let tabs = page.tabs.read(cx);
        assert_eq!(
            tabs.items().iter().map(|item| item.id().as_ref()).collect::<Vec<_>>(),
            ["b", "c", "a", "disabled", "menu"]
        );
        assert_eq!(tabs.active_id.as_deref(), Some("b"));
        assert!(!page.events.iter().any(|event| matches!(
            event,
            TabsEvent::Change { .. } | TabsEvent::Activate { .. } | TabsEvent::Reactivate { .. }
        )));
    });
}

#[test]
fn live_direction_reverses_tabs_and_arrows_and_keeps_indicator_on_active_item() {
    use crate::infra::locale::{LayoutDirection, LocaleContext, set_locale_context};
    use gpui::Focusable;
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, 0);
    let tabs = cx.update(|_, cx| page.read(cx).tabs.clone());
    for direction in [LayoutDirection::Ltr, LayoutDirection::Rtl, LayoutDirection::Ltr] {
        tabs.update(cx, |tabs, cx| tabs.set_active("b", cx));
        cx.update(|window, cx| tabs.read(cx).group.read(cx).focus_handle(cx).focus(window, cx));
        let events_before = cx.update(|_, cx| {
            page.read(cx).events.iter().filter(|event| matches!(event, TabsEvent::Change { .. })).count()
        });
        cx.update(|_, cx| set_locale_context(LocaleContext { direction, ..Default::default() }, cx));
        cx.run_until_parked();
        let a = cx.debug_bounds("tab-drag-a").unwrap();
        let c = cx.debug_bounds("tab-drag-c").unwrap();
        assert_eq!(a.left() < c.left(), direction == LayoutDirection::Ltr);
        cx.update(|_, cx| {
            let view = tabs.read(cx);
            assert_eq!(view.active_id.as_deref(), Some("b"));
            let paint = view.indicator_motion.paint().unwrap();
            let list = view.indicator_motion.list_bounds().unwrap();
            let b = view.item_bounds[1].unwrap();
            assert!(list.left() + px(paint.left) >= b.left());
            assert!(list.left() + px(paint.left + paint.width) <= b.right() + px(1.));
            assert_eq!(
                page.read(cx).events.iter().filter(|event| matches!(event, TabsEvent::Change { .. })).count(),
                events_before
            );
        });
        cx.simulate_keystrokes("right");
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert_eq!(
                tabs.read(cx).active_id.as_deref(),
                Some(if direction == LayoutDirection::Rtl { "a" } else { "c" })
            )
        });
        cx.simulate_keystrokes("left");
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(tabs.read(cx).active_id.as_deref(), Some("b")));
    }
}
