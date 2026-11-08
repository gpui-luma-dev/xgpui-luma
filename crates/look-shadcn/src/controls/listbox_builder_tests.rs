//! Exercise Shadcn's SDK ListBox composition through a headless GPUI TestWindow.
use std::{cell::Cell, rc::Rc, sync::Arc};

use gpui::{MouseButton, TestAppContext, VisualTestContext, point};
use xgpui_luma::controls::listbox::{ListBoxSnapshot, SelectionMode};
use xgpui_luma::infra::drag_drop::DragEndReason;

use super::*;

struct Preview;
impl Render for Preview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size(px(20.0))
    }
}

struct Harness {
    lists: [ListBoxState<u32, u32>; 2],
    bindings: [ListBoxBinding; 2],
    scrolls: [ListBoxScrollHandle<u32>; 2],
    look: Arc<ShadcnLook>,
    dnd: bool,
    unanswered: bool,
    content_sized: bool,
    virtualization: ListBoxVirtualization,
    previews: Rc<Cell<usize>>,
    events: Vec<DragDropEvent<usize, u32>>,
    drops: Vec<(usize, Option<u32>)>,
}

impl Harness {
    fn new(dnd: bool, cx: &mut Context<Self>) -> Self {
        xgpui_luma::focus::bind_default_focus_keys(cx);
        Self {
            lists: [0, 10].map(|offset| {
                let snapshot =
                    ListBoxSnapshot::try_with_enabled((1..=4).map(|n| n + offset), |n| *n, |n| *n != 3).unwrap();
                ListBoxState::from_snapshot(snapshot, SelectionMode::Multiple)
            }),
            bindings: std::array::from_fn(|_| ListBoxBinding::new(cx)),
            scrolls: std::array::from_fn(|_| ListBoxScrollHandle::default().require_focus_for_scroll(true)),
            look: Arc::new(ShadcnLook::built_in()),
            dnd,
            unanswered: false,
            content_sized: false,
            virtualization: ListBoxVirtualization::Eager,
            previews: Rc::default(),
            events: Vec::new(),
            drops: Vec::new(),
        }
    }

    fn input(&mut self, side: usize, input: ListBoxInput<u32>, cx: &mut Context<Self>) {
        let update = self.lists[side].apply(input);
        self.scrolls[side].handle_update(&update, cx);
    }
    fn left_input(&mut self, input: ListBoxInput<u32>, _: &mut Window, cx: &mut Context<Self>) {
        self.input(0, input, cx);
    }
    fn right_input(&mut self, input: ListBoxInput<u32>, _: &mut Window, cx: &mut Context<Self>) {
        self.input(1, input, cx);
    }
    fn event(&mut self, event: DragDropEvent<usize, u32>, cx: &mut Context<Self>) {
        self.events.push(event);
        cx.notify();
    }
    fn drop(&mut self, proposal: DropProposal<usize, u32>, _: &mut Window, cx: &mut Context<Self>) {
        self.drops.push((*proposal.target(), proposal.before().copied()));
        if self.unanswered {
            return;
        }
        for event in proposal.committed(true, proposal.keys().to_vec()) {
            self.event(event, cx);
        }
    }
}

impl Render for Harness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content_sized = self.content_sized;
        let lists = [0, 1].map(|side| {
            let previews = self.previews.clone();
            let mut builder = ListBoxBuilder::new(
                if side == 0 { "left" } else { "right" },
                &self.lists[side],
                &mut self.bindings[side],
                &self.scrolls[side],
                if side == 0 { Self::left_input } else { Self::right_input },
                |key| ("item", *key).into(),
                |surface, item, _| {
                    if content_sized {
                        surface.w_full().child(div().h(px(if item.item % 2 == 0 { 90.0 } else { 60.0 })))
                    } else {
                        surface.size_full()
                    }
                },
            )
            .empty(div().debug_selector(move || format!("empty-{side}")).size(px(20.0)))
            .virtualization(self.virtualization);
            if content_sized {
                builder = builder.content_sized();
            }
            if self.dnd {
                let preview = move |_: &u32, _: &[u32], _: &mut Window, cx: &mut App| {
                    previews.set(previews.get() + 1);
                    cx.new(|_| Preview)
                };
                builder = builder.drag_and_drop(side, Self::drop, Self::event, preview);
            }
            // Theme may be supplied after DnD without freezing the old look.
            builder.look(&self.look).build(window, cx).w(px(250.0)).flex_shrink_0()
        });
        div().flex().gap(px(20.0)).children(lists)
    }
}

fn drag(cx: &mut VisualTestContext, destination: (f32, f32)) {
    cx.simulate_mouse_down(point(px(30.0), px(25.0)), MouseButton::Left, Default::default());
    cx.simulate_mouse_move(point(px(60.0), px(25.0)), MouseButton::Left, Default::default());
    cx.simulate_mouse_move(point(px(destination.0), px(destination.1)), MouseButton::Left, Default::default());
}

#[test]
fn projected_dnd_gaps_use_the_next_visible_key() {
    for virtualization in [ListBoxVirtualization::Eager, ListBoxVirtualization::Uniform { overscan: 0 }] {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| {
            let mut host = Harness::new(true, cx);
            host.virtualization = virtualization;
            host.lists[0].set_selected_keys([1, 2]).unwrap();
            host.lists[0].set_projection([2, 1]).unwrap();
            host.lists[1].set_projection([14, 12]).unwrap();
            host
        });
        cx.run_until_parked();
        drag(cx, (300.0, 38.0)); // lower half of key 14; the next visible key is 12
        cx.simulate_mouse_up(point(px(300.0), px(38.0)), MouseButton::Left, Default::default());
        cx.update(|_, app| {
            let host = view.read(app);
            assert_eq!(host.drops, vec![(1, Some(12))]);
            assert_eq!(host.events[0], DragDropEvent::DragStarted { source: 0, keys: vec![1, 2] });
        });
    }
}

#[test]
fn filtered_empty_view_shows_empty_content_without_removing_source_items() {
    let mut app = TestAppContext::single();
    let (view, cx) = app.add_window_view(|_, cx| {
        let mut host = Harness::new(false, cx);
        host.lists[0].set_projection([]).unwrap();
        host
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds("empty-0").is_some());
    assert!(cx.debug_bounds("empty-1").is_none());
    cx.update(|_, app| assert_eq!(view.read(app).lists[0].snapshot().items().len(), 4));
}

#[test]
fn selected_group_reaches_keyed_gaps_and_same_list_targets() {
    for (destination, expected) in [
        ((300.0, 20.0), (1, Some(11))),
        ((300.0, 59.0), (1, Some(12))),
        ((300.0, 80.0), (1, Some(13))),
        ((300.0, 160.0), (1, None)),
        ((30.0, 100.0), (0, Some(3))),
    ] {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| {
            let mut harness = Harness::new(true, cx);
            harness.lists[0].set_selected_keys([2, 1]).unwrap();
            harness
        });
        cx.update(|_, app| assert_eq!(view.read(app).previews.get(), 0));
        drag(cx, destination);
        cx.simulate_mouse_up(point(px(destination.0), px(destination.1)), MouseButton::Left, Default::default());
        cx.update(|_, app| {
            let state = view.read(app);
            assert_eq!(state.drops, vec![expected]);
            assert_eq!(state.previews.get(), 1);
            assert_eq!(state.events[0], DragDropEvent::DragStarted { source: 0, keys: vec![1, 2] });
            assert_eq!(state.events.iter().filter(|event| matches!(event, DragDropEvent::DragEnded { .. })).count(), 1);
            assert!(!app.has_active_drag());
        });
    }
}

#[test]
fn optional_dnd_preserves_selection_keyboard_focus_and_disabled_rows() {
    let mut app = TestAppContext::single();
    let (view, cx) = app.add_window_view(|_, cx| Harness::new(false, cx));
    // TestWindow starts inactive; focus observers require an active test window.
    cx.update(|window, _| window.activate_window());
    cx.run_until_parked();
    cx.simulate_click(point(px(30.0), px(25.0)), Default::default());
    cx.simulate_keystrokes("down down space");
    cx.update(|window, app| {
        let state = view.read(app);
        assert!(state.bindings[0].focus_handle().is_focused(window));
        assert!(state.lists[0].is_focused());
        assert_eq!(state.lists[0].active_key(), Some(&4));
        assert_eq!(state.lists[0].selected_keys().copied().collect::<Vec<_>>(), vec![1, 4]);
    });
    drag(cx, (300.0, 25.0));
    cx.update(|_, app| {
        let state = view.read(app);
        assert!(!app.has_active_drag());
        assert_eq!(state.previews.get(), 0);
        assert!(state.events.is_empty());
    });
    cx.simulate_mouse_up(point(px(300.0), px(25.0)), MouseButton::Left, Default::default());
    cx.simulate_click(point(px(30.0), px(100.0)), Default::default());
    cx.update(|_, app| assert_eq!(view.read(app).lists[0].selected_keys().copied().collect::<Vec<_>>(), vec![1, 4]));
}

#[test]
fn empty_destination_accepts_append_and_escape_cancels_once() {
    for cancel in [false, true] {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| {
            let mut harness = Harness::new(true, cx);
            harness.lists[1].replace_snapshot(ListBoxSnapshot::try_new([], |n: &u32| *n).unwrap());
            harness
        });
        drag(cx, (300.0, 90.0));
        if cancel {
            cx.simulate_keystrokes("escape");
        }
        cx.simulate_mouse_up(point(px(300.0), px(90.0)), MouseButton::Left, Default::default());
        cx.update(|_, app| {
            let state = view.read(app);
            assert_eq!(state.drops, if cancel { vec![] } else { vec![(1, None)] });
            let reason = if cancel {
                DragEndReason::Cancelled
            } else {
                DragEndReason::Transferred
            };
            assert_eq!(state.events.last(), Some(&DragDropEvent::DragEnded { source: 0, keys: vec![1], reason }));
            assert_eq!(state.events.iter().filter(|event| matches!(event, DragDropEvent::DragEnded { .. })).count(), 1);
        });
    }
}

#[test]
fn virtualized_group_drops_use_collection_gaps_after_rows_are_unmounted() {
    use gpui::{ScrollDelta, ScrollEvent, TouchPhase};
    for same_list in [false, true] {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| {
            let mut host = Harness::new(true, cx);
            host.virtualization = ListBoxVirtualization::Uniform { overscan: 0 };
            for (side, start) in [(0, 1), (1, 10_001)] {
                host.lists[side].replace_snapshot(ListBoxSnapshot::try_new(start..start + 1_000, |n| *n).unwrap());
                host.scrolls[side].set_drag_auto_scroll_speed(0.0);
            }
            host.lists[0].set_selected_keys([1, 2]).unwrap();
            host
        });
        cx.update(|window, _| window.activate_window());
        cx.run_until_parked();
        let target_x = if same_list { 30.0 } else { 300.0 };
        // Focus the destination to enable wheel input, without toggling a row.
        cx.simulate_click(point(px(target_x), px(5.0)), Default::default());
        if same_list {
            // Begin the gesture before windowing out its source row.
            cx.simulate_mouse_down(point(px(30.0), px(25.0)), MouseButton::Left, Default::default());
            cx.simulate_mouse_move(point(px(60.0), px(25.0)), MouseButton::Left, Default::default());
        }
        cx.simulate_event(ScrollEvent {
            position: point(px(target_x), px(25.0)),
            delta: ScrollDelta::Pixels(point(px(0.0), px(-400.0))),
            modifiers: Default::default(),
            source: gpui::ScrollSource::Pointer,
            touch_phase: TouchPhase::Moved,
        });
        cx.run_until_parked();
        let last = cx.debug_bounds(if same_list { "left-item-14" } else { "right-item-14" }).unwrap();
        // Bottom half of the last rendered row targets the NEXT collection key,
        // even though that key has no rendered element (zero overscan).
        let destination = point(px(target_x), last.bottom() - px(2.0));
        if same_list {
            cx.simulate_mouse_move(destination, MouseButton::Left, Default::default());
        } else {
            drag(cx, (target_x, f32::from(destination.y)));
        }
        cx.simulate_mouse_up(destination, MouseButton::Left, Default::default());
        cx.update(|_, app| {
            let host = view.read(app);
            assert_eq!(host.drops, vec![(if same_list { 0 } else { 1 }, Some(if same_list { 16 } else { 10_016 }))]);
            assert_eq!(host.events[0], DragDropEvent::DragStarted { source: 0, keys: vec![1, 2] });
            assert_eq!(host.previews.get(), 1);
            assert!(!app.has_active_drag());
        });
    }
}

#[test]
fn virtualized_drag_auto_scroll_mounts_new_targets_without_focus() {
    let mut app = TestAppContext::single();
    let (view, cx) = app.add_window_view(|_, cx| {
        let mut host = Harness::new(true, cx);
        host.virtualization = ListBoxVirtualization::Uniform { overscan: 1 };
        host.lists[1].replace_snapshot(ListBoxSnapshot::try_new(10_001..=11_000, |n| *n).unwrap());
        host.lists[0].set_selected_keys([1, 2]).unwrap();
        // Reach the endpoint on the first scheduled frame; no wall-clock sleeps.
        host.scrolls[1].set_drag_auto_scroll_speed(10_000_000.0);
        host
    });
    cx.update(|window, _| window.activate_window());
    cx.run_until_parked();
    drag(cx, (300.0, 202.0));
    for _ in 0..4 {
        cx.update(|window, app| window.simulate_next_frame(app));
        cx.run_until_parked();
    }
    let last = cx.debug_bounds("right-item-999").expect("auto-scroll must render the destination's final row");
    let viewport = cx.debug_bounds("right-viewport").unwrap();
    assert_eq!(last.bottom(), viewport.bottom());
    cx.update(|_, app| assert!(!view.read(app).lists[1].is_focused()));
    cx.simulate_mouse_up(point(px(300.0), last.top() + px(2.0)), MouseButton::Left, Default::default());
    cx.update(|_, app| {
        let host = view.read(app);
        assert_eq!(host.drops, vec![(1, Some(11_000))]);
        assert_eq!(host.events[0], DragDropEvent::DragStarted { source: 0, keys: vec![1, 2] });
    });
}

#[test]
fn content_sized_drops_follow_actual_row_halves_in_both_rendering_modes() {
    for virtualization in [
        ListBoxVirtualization::Eager,
        ListBoxVirtualization::Measured { estimated_height: 48.0, overscan: 1 },
    ] {
        for (side, after) in [(0, false), (0, true), (1, false), (1, true)] {
            let mut app = TestAppContext::single();
            let (view, cx) = app.add_window_view(|_, cx| {
                let mut host = Harness::new(true, cx);
                host.content_sized = true;
                host.virtualization = virtualization;
                host.lists[0].set_selected_keys([1, 2]).unwrap();
                for scroll in &host.scrolls {
                    scroll.set_drag_auto_scroll_speed(0.0);
                }
                host
            });
            for _ in 0..8 {
                cx.run_until_parked();
                cx.update(|window, app| window.simulate_next_frame(app));
            }
            // The 92px second row must have a 46px upper/lower target, not the
            // builder's default 18px half-height. Stay well away from the seam.
            let row = cx.debug_bounds(if side == 0 { "left-item-1" } else { "right-item-1" }).unwrap();
            assert_eq!(row.size.height, px(92.0));
            let destination = (
                if side == 0 { 30.0 } else { 300.0 },
                f32::from(row.top() + row.size.height * if after { 0.8 } else { 0.2 }),
            );
            drag(cx, destination);
            cx.simulate_mouse_up(point(px(destination.0), px(destination.1)), MouseButton::Left, Default::default());
            cx.update(|_, app| {
                let host = view.read(app);
                assert_eq!(
                    host.drops,
                    vec![(
                        side,
                        Some(if side == 0 {
                            if after { 3 } else { 2 }
                        } else if after {
                            13
                        } else {
                            12
                        })
                    )]
                );
                assert_eq!(host.events[0], DragDropEvent::DragStarted { source: 0, keys: vec![1, 2] });
            });
        }
    }
}

#[test]
fn measured_drag_auto_scroll_mounts_targets_without_focus() {
    let mut app = TestAppContext::single();
    let (view, cx) = app.add_window_view(|_, cx| {
        let mut host = Harness::new(true, cx);
        host.content_sized = true;
        host.virtualization = ListBoxVirtualization::Measured { estimated_height: 48.0, overscan: 1 };
        host.lists[1].replace_snapshot(ListBoxSnapshot::try_new(10_001..=11_000, |n| *n).unwrap());
        host.scrolls[1].set_drag_auto_scroll_speed(10_000_000.0);
        host
    });
    cx.update(|window, _| window.activate_window());
    for _ in 0..8 {
        cx.run_until_parked();
        cx.update(|window, app| window.simulate_next_frame(app));
    }
    drag(cx, (300.0, 202.0));
    for _ in 0..12 {
        cx.run_until_parked();
        cx.update(|window, app| window.simulate_next_frame(app));
    }
    let last = cx.debug_bounds("right-item-999").unwrap();
    assert_eq!(last.bottom(), cx.debug_bounds("right-viewport").unwrap().bottom());
    cx.update(|_, app| assert!(!view.read(app).lists[1].is_focused()));
    let destination = point(px(300.0), last.top() + px(5.0));
    cx.simulate_mouse_move(destination, MouseButton::Left, Default::default());
    cx.simulate_mouse_up(destination, MouseButton::Left, Default::default());
    cx.update(|_, app| assert_eq!(view.read(app).drops, vec![(1, Some(11_000))]));
}

#[test]
fn constrained_flex_column_preserves_surface_height_around_viewport() {
    struct ConstrainedList {
        state: ListBoxState<u32, u32>,
        binding: ListBoxBinding,
        scroll: ListBoxScrollHandle<u32>,
        look: ShadcnLook,
    }

    impl Render for ConstrainedList {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let surface = ListBoxBuilder::new(
                "constrained-list",
                &self.state,
                &mut self.binding,
                &self.scroll,
                |_, _, _, _| {},
                |key| ("item", *key).into(),
                |surface, _, _| surface.size_full(),
            )
            .look(&self.look)
            .build(window, cx);
            // Leave flex shrinking enabled: the surface must contain its viewport.
            div().w(px(250.0)).h(px(100.0)).flex().flex_col().child(surface)
        }
    }

    let mut app = TestAppContext::single();
    let (_, cx) = app.add_window_view(|_, cx| ConstrainedList {
        state: ListBoxState::from_snapshot(ListBoxSnapshot::try_new(1..=4, |n| *n).unwrap(), SelectionMode::Multiple),
        binding: ListBoxBinding::new(cx),
        scroll: ListBoxScrollHandle::default(),
        look: ShadcnLook::built_in(),
    });
    cx.run_until_parked();
    let surface = cx.debug_bounds("constrained-list-surface").unwrap();
    let viewport = cx.debug_bounds("constrained-list-viewport").unwrap();
    assert_eq!(viewport.size.height, px(196.0));
    assert_eq!(surface.size.height, px(214.0));
    assert_eq!(viewport.top() - surface.top(), px(9.0));
    assert_eq!(surface.bottom() - viewport.bottom(), px(9.0));
}

#[test]
fn raw_composition_rejects_unanswered_proposals_instead_of_reporting_cancellation() {
    let mut app = TestAppContext::single();
    let (view, cx) = app.add_window_view(|_, cx| {
        let mut host = Harness::new(true, cx);
        host.unanswered = true;
        host
    });
    cx.run_until_parked();
    drag(cx, (300.0, 38.0));
    cx.simulate_mouse_up(point(px(300.0), px(38.0)), MouseButton::Left, Default::default());
    cx.run_until_parked();
    cx.update(|_, app| {
        let host = view.read(app);
        assert_eq!(host.events.len(), 3);
        assert!(matches!(&host.events[1], DragDropEvent::DropRejected { error, .. } if error.as_ref() == "owner did not accept the drop"));
        assert!(matches!(host.events[2], DragDropEvent::DragEnded { reason: DragEndReason::Rejected, .. }));
    });
}
