//! Native headless gestures; no GUI process is launched.
use super::*;
use crate::infra::drag_drop::{DragDropEvent as E, DragEndReason};
use gpui::{
    Context, Entity, Render, Window, TestAppContext, VisualTestContext, IntoElement, MouseButton, div, point, px,
    prelude::*,
};
struct Page {
    accordion: Entity<AccordionControl>,
    other: Entity<AccordionControl>,
    events: Vec<AccordionEvent>,
    named: Vec<&'static str>,
}
impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(400.)).flex().flex_col().child(self.accordion.clone()).child(self.other.clone())
    }
}
fn item(id: &str, height: f32) -> AccordionItem {
    AccordionItem::new(
        id.to_string(),
        AccordionTrigger::new(id.to_string()),
        AccordionContent::custom(move |_, _| div().h(px(height)).child("Editor").into_any_element()),
    )
}
fn setup(app: &mut TestAppContext, mode: u8) -> (Entity<Page>, &mut VisualTestContext) {
    let (page, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        crate::key_handling::bind_default_control_keys(cx);
        let accordion = new("drag-accordion")
            .multiple()
            .reordering(true)
            .motion_enabled(false)
            .items([
                item("a", 48.).expanded(true),
                item("b", 75.).expanded(true),
                item("c", 12.),
                item("disabled", 12.).enabled(false),
            ])
            .on_event(cx, |owner: &mut Page, event, _| owner.events.push(event.clone()))
            .on_drag_drop(cx, |owner, _, _| owner.named.push("aggregate"))
            .on_drag_started(cx, |owner, _, _, _| owner.named.push("start"))
            .on_items_reordered(cx, |owner, _, _, _, _| owner.named.push("reorder"))
            .on_dropped(cx, |owner, _, _, _, _, _, cx| {
                owner.named.push("drop");
                // Event callbacks may update the emitting control.
                owner.accordion.update(cx, |_, cx| cx.notify());
            })
            .on_drop_rejected(cx, |owner, _, _, _, _, _, _| owner.named.push("reject"))
            .on_drag_ended(cx, |owner, _, _, _, _| owner.named.push("end"))
            .spawn(cx);
        if mode != 0 {
            accordion.update(cx, |a, _| {
                a.set_drop_handler(move |a, p, _, cx| {
                    if mode == 1 {
                        a.reject_drop(p, "policy", cx);
                    }
                })
            });
        }
        Page {
            accordion,
            other: new("other").reordering(true).items([item("x", 1.), item("y", 1.)]).spawn(cx),
            events: vec![],
            named: vec![],
        }
    });
    cx.run_until_parked();
    (page, cx)
}
fn bounds(cx: &mut VisualTestContext, name: String) -> gpui::Bounds<gpui::Pixels> {
    cx.debug_bounds(Box::leak(name.into_boxed_str())).unwrap()
}
fn start(cx: &mut VisualTestContext, from: &str) {
    let start = bounds(cx, format!("accordion-drag-{from}")).center();
    cx.simulate_mouse_down(start, MouseButton::Left, Default::default());
    cx.simulate_mouse_move(start + point(px(12.), px(0.)), MouseButton::Left, Default::default());
    cx.run_until_parked();
}
fn finish(cx: &mut VisualTestContext, to: &str, fraction: f32) {
    let target = bounds(cx, format!("accordion-item-{to}"));
    let end = point(target.center().x, target.top() + target.size.height * fraction);
    cx.simulate_mouse_move(end, MouseButton::Left, Default::default());
    cx.run_until_parked();
    cx.simulate_mouse_up(end, MouseButton::Left, Default::default());
    cx.run_until_parked();
}
fn events(page: &Entity<Page>, cx: &mut VisualTestContext) -> Vec<AccordionDragEvent> {
    cx.update(|_, cx| {
        page.read(cx)
            .events
            .iter()
            .filter_map(|e| {
                if let AccordionEvent::DragDrop(e) = e {
                    Some(e.clone())
                } else {
                    None
                }
            })
            .collect()
    })
}
#[test]
fn native_move_preserves_expansion_measurements_focus_and_callback_order() {
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, 0);
    let accordion = cx.update(|_, cx| page.read(cx).accordion.clone());
    let old = cx.update(|_, cx| accordion.read(cx).content_heights_px.clone());
    start(cx, "a");
    finish(cx, "c", 0.8);
    cx.update(|_, cx| {
        let p = page.read(cx);
        let a = accordion.read(cx);
        assert_eq!(a.items().iter().map(|i| i.id().as_ref()).collect::<Vec<_>>(), ["b", "c", "a", "disabled"]);
        assert!(a.is_expanded(&"a".into()) && a.is_expanded(&"b".into()));
        assert_eq!(a.focused_item_index, Some(2));
        assert_eq!(a.content_heights_px[2], old[0]);
        assert_eq!(a.content_heights_px[0], old[1]);
        assert!(!p.events.iter().any(|e| matches!(e, AccordionEvent::ExpandedChanged { .. })));
        assert_eq!(p.named, ["aggregate", "start", "aggregate", "reorder", "aggregate", "drop", "aggregate", "end"]);
        assert!(!a.drop_layout.is_animating()); // motion_enabled(false)
    });
    assert!(matches!(
        &events(&page, cx)[..],
        [
            E::DragStarted { .. },
            E::ItemsReordered { .. },
            E::Dropped { changed: true, .. },
            E::DragEnded { reason: DragEndReason::Reordered, .. }
        ]
    ));
}
#[test]
fn noops_policy_rejection_and_unanswered_proposals_finish_once_without_motion() {
    for mode in 0..3 {
        let mut app = TestAppContext::single();
        let (page, cx) = setup(&mut app, mode);
        start(cx, "a");
        finish(cx, "a", 0.2);
        let e = events(&page, cx);
        assert_eq!(e.len(), 3);
        if mode == 0 {
            assert!(matches!(e[1], E::Dropped { changed: false, .. }));
        } else {
            assert!(matches!(e[1], E::DropRejected { .. }));
        }
        cx.update(|_, cx| {
            let a = page.read(cx).accordion.read(cx);
            assert_eq!(a.items()[0].id().as_ref(), "a");
            assert!(!a.drop_layout.is_animating());
        });
        assert!(cx.debug_bounds("accordion-drag-disabled").is_none());
    }
}
#[test]
fn escape_foreign_drop_and_disabling_end_once() {
    for action in 0..3 {
        let mut app = TestAppContext::single();
        let (page, cx) = setup(&mut app, 0);
        start(cx, "a");
        if action == 0 {
            cx.simulate_keystrokes("escape");
        } else if action == 1 {
            finish(cx, "x", 0.8);
        } else {
            let a = cx.update(|_, cx| page.read(cx).accordion.clone());
            a.update(cx, |a, cx| a.set_reordering(false, cx));
            let end = point(px(390.), px(600.));
            cx.simulate_mouse_up(end, MouseButton::Left, Default::default());
        }
        cx.run_until_parked();
        let e = events(&page, cx);
        assert_eq!(
            e.iter().filter(|e| matches!(e, E::DragEnded { reason: DragEndReason::Cancelled, .. })).count(),
            1,
            "action {action}: {e:?}"
        );
        assert_eq!(e.len(), 2);
    }
}
#[test]
fn adjacent_halves_share_one_gap_marker_for_expanded_items() {
    for side in [-2., 2.] {
        let mut app = TestAppContext::single();
        let (page, cx) = setup(&mut app, 0);
        let b = bounds(cx, "accordion-item-b".into());
        let c = bounds(cx, "accordion-item-c".into());
        let boundary = (b.bottom() + c.top()) / 2.;
        start(cx, "a");
        let target = cx.debug_bounds("accordion-gap-before-c").unwrap();
        assert!((target.top() - b.center().y).abs() < px(0.5), "target {target:?}, b {b:?}, c {c:?}");
        assert!((target.bottom() - c.center().y).abs() < px(0.5));
        let marker = cx.debug_bounds("accordion-gap-before-c-marker").unwrap();
        assert!((marker.center().y - boundary).abs() < px(0.5));
        for offset in [-2., 2., side] {
            cx.simulate_mouse_move(point(b.center().x, boundary + px(offset)), MouseButton::Left, Default::default());
            cx.run_until_parked();
            assert_eq!(cx.debug_bounds("accordion-gap-before-c-marker").unwrap(), marker);
            cx.update(|window, _| {
                assert_eq!(
                    window
                        .painted_quads()
                        .iter()
                        .filter(|q| q.bounds == marker.scale(window.scale_factor())
                            && q.background.as_solid().is_some_and(|c| c.a > 0.9))
                        .count(),
                    1
                )
            });
        }
        cx.simulate_mouse_up(point(b.center().x, boundary + px(side)), MouseButton::Left, Default::default());
        cx.run_until_parked();
        assert!(
            events(&page, cx)
                .iter()
                .any(|e| matches!(e,E::Dropped{before,changed:true,..} if before.as_deref()==Some("c")))
        );
    }
}
#[test]
fn header_clicks_toggle_without_starting_drag() {
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, 0);
    let target = cx.debug_bounds("accordion-trigger-c").unwrap();
    cx.simulate_click(target.center(), Default::default());
    cx.run_until_parked();
    cx.update(|_, cx| assert!(page.read(cx).accordion.read(cx).is_expanded(&"c".into())));
    assert!(events(&page, cx).is_empty());
}

#[test]
fn native_drop_motion_commits_before_interpolation_and_respects_zero_and_reduced_motion() {
    use std::time::Duration;
    for policy in 0..3 {
        let mut app = TestAppContext::single();
        let (page, cx) = setup(&mut app, 0);
        let a = cx.update(|_, cx| page.read(cx).accordion.clone());
        a.update(cx, |a, _| {
            a.model.motion_enabled = true;
            a.model.motion.configure(move |m| {
                m.drop_layout = crate::motion::Transition::timed(
                    if policy == 1 {
                        Duration::ZERO
                    } else {
                        Duration::from_millis(1000)
                    },
                    crate::motion::Easing::OutCubic,
                )
            });
            a.model.motion.freeze();
        });
        if policy == 2 {
            cx.update(|_, cx| cx.set_reduce_motion(true));
        }
        cx.run_until_parked();
        let old = cx.debug_bounds("accordion-item-a").unwrap().top();
        start(cx, "a");
        finish(cx, "c", 0.8);
        assert!(events(&page, cx).iter().any(|e| matches!(e, E::DragEnded { reason: DragEndReason::Reordered, .. })));
        cx.update(|_, cx| assert_eq!(a.read(cx).items()[2].id().as_ref(), "a"));
        if policy == 0 {
            let first = cx.debug_bounds("accordion-item-a").unwrap().top();
            assert!((first - old).abs() < px(2.));
            a.update(cx, |a, _| a.drop_layout.elapse_for_test(Duration::from_millis(500)));
            cx.update(|window, cx| window.simulate_next_frame(cx));
            cx.run_until_parked();
            let middle = cx.debug_bounds("accordion-item-a").unwrap().top();
            a.update(cx, |a, _| a.drop_layout.elapse_for_test(Duration::from_millis(1100)));
            cx.update(|window, cx| window.simulate_next_frame(cx));
            cx.run_until_parked();
            let end = cx.debug_bounds("accordion-item-a").unwrap().top();
            assert!(middle > old && middle < end);
        }
        cx.update(|_, cx| assert!(!a.read(cx).drop_layout.is_animating()));
    }
}

#[test]
fn first_and_append_gaps_and_stale_completion_preserve_content() {
    use std::{cell::RefCell, rc::Rc};
    for (from, to, fraction, expected) in
        [("a", "disabled", 0.9, ["b", "c", "disabled", "a"]), ("c", "a", 0.1, ["c", "a", "b", "disabled"])]
    {
        let mut app = TestAppContext::single();
        let (page, cx) = setup(&mut app, 0);
        let a = cx.update(|_, cx| page.read(cx).accordion.clone());
        let saved = Rc::new(RefCell::new(None));
        let capture = saved.clone();
        a.update(cx, |a, _| {
            a.set_drop_handler(move |a, p, _, cx| {
                *capture.borrow_mut() = Some(p.clone());
                a.commit_drop(p, cx).unwrap();
            })
        });
        start(cx, from);
        finish(cx, to, fraction);
        a.update(cx, |a, cx| {
            assert_eq!(a.items().iter().map(|i| i.id().as_ref()).collect::<Vec<_>>(), expected);
            assert!(a.commit_drop(saved.borrow().as_ref().unwrap(), cx).is_err());
        });
        cx.run_until_parked();
        assert_eq!(events(&page, cx).iter().filter(|e| matches!(e, E::DragEnded { .. })).count(), 1);
    }
}

#[test]
#[cfg_attr(
    any(target_os = "windows", target_os = "linux"),
    ignore = "xgpui test windows never report hover on Windows/Linux, and edge auto-scroll stops unless Window::is_window_hovered(); passes on macOS"
)]
fn viewport_auto_scroll_stops_after_cancel() {
    struct Scroller(Entity<AccordionControl>);
    impl Render for Scroller {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(px(300.)).h(px(180.)).flex().child(self.0.clone())
        }
    }
    let mut app = TestAppContext::single();
    let (page, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        crate::key_handling::bind_default_control_keys(cx);
        Scroller(new("scrolling").reordering(true).items((0..40).map(|i| item(&format!("s{i}"), 10.))).spawn(cx))
    });
    cx.run_until_parked();
    let a = cx.update(|_, cx| page.read(cx).0.clone());
    start(cx, "s0");
    let viewport = cx.update(|_, cx| a.read(cx).drag_scroll.bounds());
    assert!(viewport.size.height <= px(180.));
    let end = point(viewport.center().x, viewport.bottom() - px(2.));
    cx.simulate_mouse_move(end, MouseButton::Left, Default::default());
    cx.run_until_parked();
    cx.update(|window, cx| window.simulate_next_frame(cx));
    cx.run_until_parked();
    let offset = cx.update(|_, cx| a.read(cx).drag_scroll.offset().y);
    assert!(offset < px(0.));
    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    let stopped = cx.update(|_, cx| a.read(cx).drag_scroll.offset());
    cx.update(|window, cx| window.simulate_next_frame(cx));
    cx.run_until_parked();
    cx.update(|_, cx| assert_eq!(a.read(cx).drag_scroll.offset(), stopped));
}

#[test]
fn keyboard_navigation_and_activation_survive_drag_focus_context() {
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, 0);
    let trigger = cx.debug_bounds("accordion-trigger-c").unwrap();
    cx.simulate_click(trigger.center(), Default::default());
    cx.run_until_parked();
    cx.simulate_keystrokes("up");
    cx.run_until_parked();
    cx.update(|_, cx| assert_eq!(page.read(cx).accordion.read(cx).focused_item_index, Some(1)));
    cx.simulate_keystrokes("space");
    cx.run_until_parked();
    cx.update(|_, cx| assert!(!page.read(cx).accordion.read(cx).is_expanded(&"b".into())));
}

#[test]
fn disabled_keys_and_missing_anchors_reject_without_mutation() {
    for invalid in 0..2 {
        let mut app = TestAppContext::single();
        let (page, cx) = setup(&mut app, 0);
        start(cx, "a");
        let a = cx.update(|_, cx| page.read(cx).accordion.clone());
        a.update(cx, |a, cx| {
            if invalid == 0 {
                a.model.items[0].enabled = false;
            }
            let drag = a.active_drag.as_ref().unwrap();
            let p = drag
                .propose(cx.entity_id(), cx.entity_id(), Some(if invalid == 0 { "c" } else { "missing" }.into()))
                .unwrap();
            assert!(a.commit_drop(&p, cx).is_err());
            assert_eq!(a.items()[0].id().as_ref(), "a");
        });
        cx.simulate_mouse_up(point(px(390.), px(650.)), MouseButton::Left, Default::default());
        cx.run_until_parked();
        let e = events(&page, cx);
        assert_eq!(e.len(), 3);
        assert!(matches!(e[1], E::DropRejected { .. }));
    }
}

#[test]
fn commit_moves_disclosure_progress_and_content_renderers_without_reconstruction() {
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, 0);
    let a = cx.update(|_, cx| page.read(cx).accordion.clone());
    a.update(cx, |a, _| {
        a.transitions[1] = crate::motion::DisclosureMotion::new(0.4, true);
        a.transitions[1].set_target(1.);
        a.set_drop_handler(|a, p, _, cx| {
            let progress = a.transitions[1].progress();
            let content = a.model.items[0].content.element.clone().unwrap();
            a.commit_drop(p, cx).unwrap();
            assert_eq!(a.transitions[0].progress(), progress);
            assert!(std::sync::Arc::ptr_eq(a.model.items[2].content.element.as_ref().unwrap(), &content));
        });
    });
    start(cx, "a");
    finish(cx, "c", 0.8);
    assert!(events(&page, cx).iter().any(|e| matches!(e, E::Dropped { changed: true, .. })));
}

#[test]
fn lifecycle_registrations_hold_owners_weakly_and_release_captures() {
    use gpui::AppContext;
    use std::rc::Rc;
    struct Owner(usize);
    let app = TestAppContext::single();
    let owner = app.update(|cx| cx.new(|_| Owner(0)));
    let weak_owner = owner.downgrade();
    let marker = Rc::new(());
    let weak_marker = Rc::downgrade(&marker);
    let a = app.update(|cx| {
        owner.update(cx, |_, cx| {
            new("callbacks")
                .on_drag_drop(cx, move |owner: &mut Owner, _, _| {
                    let _ = &marker;
                    owner.0 += 1;
                })
                .spawn(cx)
        })
    });
    drop(owner);
    app.update(|_| {});
    assert!(weak_owner.upgrade().is_none());
    app.update(|cx| {
        a.update(cx, |_, cx| {
            cx.emit(AccordionEvent::DragDrop(E::DragStarted { source: cx.entity_id(), keys: vec!["a".into()] }))
        })
    });
    drop(a);
    app.update(|_| {});
    assert!(weak_marker.upgrade().is_none());
}

#[test]
fn only_the_handle_starts_drag_and_its_tooltip_dismisses_on_drag() {
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, 0);
    let header = cx.debug_bounds("accordion-trigger-a").unwrap();
    let position = point(header.center().x, header.center().y);
    cx.simulate_mouse_down(position, MouseButton::Left, Default::default());
    cx.simulate_mouse_move(position + point(px(12.), px(0.)), MouseButton::Left, Default::default());
    cx.run_until_parked();
    cx.simulate_mouse_up(position + point(px(12.), px(0.)), MouseButton::Left, Default::default());
    cx.run_until_parked();
    assert!(events(&page, cx).is_empty(), "header drag must not start DnD");
    let a = cx.update(|_, cx| page.read(cx).accordion.clone());
    a.update(cx, |a, cx| {
        let config = super::drag::reorder_help().template(|model| {
            assert_eq!(model.text.as_ref(), "Drag to reorder");
            div()
                .debug_selector(|| "accordion-reorder-tooltip".into())
                .child(crate::controls::tooltip::bubble(model))
                .into_any_element()
        });
        let tip =
            crate::infra::attachments::TooltipHandle::new_with_theme(config, a.model.template.drag_tooltip_theme(), cx);
        a.drag_tooltips.insert("a".into(), tip);
        cx.notify();
    });
    cx.run_until_parked();
    let handle = cx.debug_bounds("accordion-drag-a").unwrap();
    cx.simulate_event(gpui::MouseMoveEvent { position: handle.center(), ..Default::default() });
    cx.run_until_parked();
    cx.executor().advance_clock(std::time::Duration::from_millis(500));
    cx.run_until_parked();
    assert!(cx.debug_bounds("accordion-reorder-tooltip").is_some());
    start(cx, "a");
    assert!(cx.debug_bounds("accordion-reorder-tooltip").is_none());
    finish(cx, "c", 0.8);
    assert!(events(&page, cx).iter().any(|e| matches!(e, E::Dropped { changed: true, .. })));
}
