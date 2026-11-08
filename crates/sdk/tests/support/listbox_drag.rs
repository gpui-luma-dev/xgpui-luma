// Shared native-dispatch contract exercised by each ListBox Look adapter.
use gpui::{Context, Entity, MouseButton, Render, TestAppContext, Window, div, point, prelude::*, px};
use xgpui_luma::controls::listbox::{
    ListBoxControl, ListBoxDragDrop, ListBoxDropProposal, ListBoxInput, ListBoxSnapshot, ListBoxState, SelectionMode,
};
use xgpui_luma::infra::drag_drop::{DragDropEvent, DragEndReason};

#[derive(Clone, Copy)]
enum Decision {
    Commit,
    Reject,
    Unanswered,
}
struct DragHarness {
    lists: [ListBoxControl<Self, u32, u32>; 2],
    decision: Decision,
    events: Vec<(usize, DragDropEvent<gpui::EntityId, u32>)>,
    aggregate: Vec<(usize, DragDropEvent<gpui::EntityId, u32>)>,
    named: Vec<(usize, &'static str)>,
}
impl DragHarness {
    fn new(cx: &mut Context<Self>) -> Self {
        xgpui_luma::focus::bind_default_focus_keys(cx);
        let lists = std::array::from_fn(|side| {
            let state =
                ListBoxState::try_new((1..=4).map(|key| key + side as u32 * 10), |key| *key, SelectionMode::Multiple)
                    .unwrap();
            ListBoxControl::new(
                state,
                if side == 0 { Self::left_input } else { Self::right_input },
                |key| ("key", *key).into(),
                |row| row.to_string().into(),
                cx,
            )
            .drag_drop(ListBoxDragDrop::new(cx.entity_id(), Self::drop))
            .on_event(cx, move |owner: &mut Self, event, _| {
                if let xgpui_luma::controls::listbox::ListBoxEvent::DragDrop(event) = event {
                    owner.events.push((side, event.clone()));
                }
            })
            .on_drag_drop(cx, move |owner, event, _| owner.aggregate.push((side, event.clone())))
            .on_drag_started(cx, move |owner, _, keys, _| {
                assert_eq!(keys, &[1, 2]);
                owner.named.push((side, "start"));
            })
            .on_items_removed(cx, move |owner, _, _, keys, _| {
                assert_eq!(keys, &[1, 2]);
                assert_eq!(owner.items(0), [3, 4]);
                owner.named.push((side, "remove"));
            })
            .on_items_added(cx, move |owner, _, _, keys, before, _| {
                assert_eq!(keys, &[1, 2]);
                assert!(before.is_none());
                assert_eq!(owner.items(1), [11, 12, 13, 14, 1, 2]);
                owner.named.push((side, "add"));
            })
            .on_items_reordered(cx, move |owner, _, keys, before, _| {
                assert_eq!(keys, &[1, 2]);
                assert!(before.is_none());
                assert_eq!(owner.items(0), [3, 4, 1, 2]);
                owner.named.push((side, "reorder"));
            })
            .on_dropped(cx, move |owner, _, _, keys, _, changed, cx| {
                assert_eq!(keys, &[1, 2]);
                if changed {
                    assert!(owner.items(0) == [3, 4] || owner.items(0) == [3, 4, 1, 2]);
                }
                owner.named.push((side, "drop"));
                // The named handler may safely update the emitting control.
                owner.lists[side].apply(ListBoxInput::Focus(true), cx);
                cx.notify();
            })
            .on_drop_rejected(cx, move |owner, _, _, keys, _, error, _| {
                assert_eq!(keys, &[1, 2]);
                assert!(!error.is_empty());
                owner.named.push((side, "reject"));
            })
            .on_drag_ended(cx, move |owner, _, keys, reason, _| {
                assert_eq!(keys, &[1, 2]);
                owner.named.push((
                    side,
                    if reason == DragEndReason::Cancelled {
                        "cancel"
                    } else {
                        "end"
                    },
                ));
            })
        });
        Self { lists, decision: Decision::Commit, events: vec![], aggregate: vec![], named: vec![] }
    }
    fn items(&self, side: usize) -> Vec<u32> {
        self.lists[side].state.snapshot().items().to_vec()
    }
    fn left_input(&mut self, input: ListBoxInput<u32>, _: &mut Window, cx: &mut Context<Self>) {
        self.lists[0].apply(input, cx);
    }
    fn right_input(&mut self, input: ListBoxInput<u32>, _: &mut Window, cx: &mut Context<Self>) {
        self.lists[1].apply(input, cx);
    }
    fn replace(&mut self, side: usize, items: Vec<u32>, cx: &mut Context<Self>) {
        let update = self.lists[side].state.replace_snapshot(ListBoxSnapshot::try_new(items, |key| *key).unwrap());
        self.lists[side].handle_update(&update, cx);
    }
    fn drop(&mut self, proposal: ListBoxDropProposal<u32>, _: &mut Window, cx: &mut Context<Self>) {
        match self.decision {
            Decision::Reject => {
                proposal.rejected("Locked", cx);
                return;
            }
            Decision::Unanswered => return,
            Decision::Commit => {}
        }
        if let Err(error) = proposal.validate() {
            proposal.rejected(error, cx);
            return;
        }
        let source = self.lists.iter().position(|list| list.drag_drop_id() == proposal.source()).unwrap();
        let target = self.lists.iter().position(|list| list.drag_drop_id() == proposal.target()).unwrap();
        let source_items = self.items(source);
        let target_items = self.items(target);
        let moving: Vec<_> = source_items.iter().filter(|key| proposal.keys().contains(key)).copied().collect();
        let gap = proposal
            .before()
            .map_or(target_items.len(), |key| target_items.iter().position(|item| item == key).unwrap());
        let index = target_items[..gap].iter().filter(|key| source != target || !moving.contains(key)).count();
        let mut next: Vec<_> =
            target_items.iter().filter(|key| source != target || !moving.contains(key)).copied().collect();
        next.splice(index..index, moving.iter().copied());
        let changed = next != target_items;
        if changed {
            if source != target {
                self.replace(source, source_items.into_iter().filter(|key| !moving.contains(key)).collect(), cx);
            }
            self.replace(target, next, cx);
        }
        proposal.committed(changed, moving.clone(), cx);
        proposal.committed(changed, moving, cx); // Completion clones cannot emit duplicates.
        proposal.rejected("late", cx);
        cx.notify();
    }
}
impl Render for DragHarness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().flex().gap(px(20.0)).children(
            self.lists
                .iter_mut()
                .enumerate()
                .map(|(side, list)| render_drag_list(side, list, window, cx).w(px(250.0)).flex_shrink_0()),
        )
    }
}
fn start_native(cx: &mut gpui::VisualTestContext, destination: (f32, f32)) {
    cx.simulate_mouse_down(point(px(30.0), px(25.0)), MouseButton::Left, Default::default());
    cx.simulate_mouse_move(point(px(60.0), px(25.0)), MouseButton::Left, Default::default());
    cx.simulate_mouse_move(point(px(destination.0), px(destination.1)), MouseButton::Left, Default::default());
}
#[test]
fn persistent_control_routes_native_transactions_and_named_callbacks() {
    for (destination, decision, expected) in [
        (
            (300.0, 185.0),
            Decision::Commit,
            vec![(0, "start"), (0, "remove"), (1, "add"), (1, "drop"), (0, "end")],
        ),
        ((30.0, 185.0), Decision::Commit, vec![(0, "start"), (0, "reorder"), (0, "drop"), (0, "end")]),
        ((30.0, 10.0), Decision::Commit, vec![(0, "start"), (0, "drop"), (0, "end")]),
        ((300.0, 185.0), Decision::Reject, vec![(0, "start"), (1, "reject"), (0, "end")]),
        ((300.0, 185.0), Decision::Unanswered, vec![(0, "start"), (1, "reject"), (0, "end")]),
        ((700.0, 400.0), Decision::Commit, vec![(0, "start"), (0, "cancel")]),
    ] {
        let mut app = TestAppContext::single();
        let (view, cx): (Entity<DragHarness>, _) = app.add_window_view(|window, cx| {
            window.activate_window();
            let mut host = DragHarness::new(cx);
            host.decision = decision;
            host.lists[0].state.set_selected_keys([1, 2]).unwrap();
            host
        });
        cx.run_until_parked();
        // Rebuilding visual adapters must not register another handler.
        for _ in 0..3 {
            view.update(cx, |_, cx| cx.notify());
            cx.run_until_parked();
        }
        start_native(cx, destination);
        cx.simulate_mouse_up(point(px(destination.0), px(destination.1)), MouseButton::Left, Default::default());
        cx.run_until_parked();
        cx.update(|_, app| {
            let host = view.read(app);
            assert_eq!(host.named, expected, "destination {destination:?}");
            assert_eq!(host.events, host.aggregate);
            assert_eq!(host.events.len(), expected.len());
            if expected.iter().any(|(_, phase)| *phase == "reject" || *phase == "cancel") {
                assert_eq!(host.items(0), [1, 2, 3, 4]);
                assert_eq!(host.items(1), [11, 12, 13, 14]);
            }
        });
    }
}

fn fresh_list(cx: &mut Context<DragHarness>) -> ListBoxControl<DragHarness, u32, u32> {
    ListBoxControl::new(
        ListBoxState::try_new(1..=4, |key| *key, SelectionMode::Multiple).unwrap(),
        DragHarness::left_input,
        |key| ("key", *key).into(),
        |row| row.to_string().into(),
        cx,
    )
}

#[test]
fn released_source_is_not_retained_by_native_preview_or_pending_proposal() {
    let mut app = TestAppContext::single();
    let capture = std::rc::Rc::new(());
    let (view, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        let mut host = DragHarness::new(cx);
        host.lists[0].state.set_selected_keys([1, 2]).unwrap();
        let original = std::mem::replace(&mut host.lists[0], fresh_list(cx));
        let capture = capture.clone();
        host.lists[0] = original.on_drag_ended(cx, move |_, _, _, _, _| {
            let _keep = &capture;
            panic!("released source handler must not run");
        });
        host
    });
    cx.run_until_parked();
    start_native(cx, (300.0, 185.0));
    cx.run_until_parked();
    assert_eq!(std::rc::Rc::strong_count(&capture), 2);
    view.update(cx, |host, cx| {
        host.lists[0] = fresh_list(cx);
        cx.notify();
    });
    cx.run_until_parked();
    assert_eq!(std::rc::Rc::strong_count(&capture), 1);
    cx.simulate_mouse_up(point(px(300.0), px(185.0)), MouseButton::Left, Default::default());
    cx.run_until_parked();
    cx.update(|_, app| {
        let host = view.read(app);
        assert_eq!(host.named, [(0, "start"), (1, "reject")]);
        assert_eq!(host.items(1), [11, 12, 13, 14]);
    });
}

#[test]
fn foreign_list_scope_cancels_without_delivering_target_events() {
    let mut app = TestAppContext::single();
    let (view, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        let mut host = DragHarness::new(cx);
        host.lists[0].state.set_selected_keys([1, 2]).unwrap();
        let target = std::mem::replace(&mut host.lists[1], fresh_list(cx));
        let foreign_scope = cx.new(|_| ()).entity_id();
        host.lists[1] = target.drag_drop(ListBoxDragDrop::new(foreign_scope, DragHarness::drop));
        host
    });
    cx.run_until_parked();
    start_native(cx, (300.0, 185.0));
    cx.simulate_mouse_up(point(px(300.0), px(185.0)), MouseButton::Left, Default::default());
    cx.run_until_parked();
    cx.update(|_, app| {
        let host = view.read(app);
        assert_eq!(host.named, [(0, "start"), (0, "cancel")]);
        assert_eq!(host.items(0), [1, 2, 3, 4]);
        assert_eq!(host.items(1), [11, 12, 13, 14]);
    });
}

#[test]
fn committed_list_motion_keeps_callback_order_and_animates_visible_siblings() {
    use std::time::Duration;
    for destination in [(30.,185.),(300.,185.)] {
        let mut app=TestAppContext::single();let(view,cx)=app.add_window_view(|window,cx|{
            window.activate_window();let mut host=DragHarness::new(cx);host.lists[0].state.set_selected_keys([1,2]).unwrap();
            host.lists = host.lists.map(|list| list.motion(|m| m.drop_layout = xgpui_luma::motion::Transition::timed(Duration::from_secs(10), xgpui_luma::motion::Easing::OutCubic)));host
        });cx.run_until_parked();
        let first=cx.debug_bounds("sdk-left-item-0").unwrap();let third=cx.debug_bounds("sdk-left-item-2").unwrap();
        start_native(cx,destination);cx.simulate_mouse_up(point(px(destination.0),px(destination.1)),MouseButton::Left,Default::default());cx.run_until_parked();
        let moving=cx.debug_bounds("sdk-left-item-0").unwrap();assert!(moving.top()>first.top()&&moving.top()<=third.top());
        cx.update(|_,cx|{let host=view.read(cx);assert_eq!(host.items(0)[0],3);assert!(host.lists[0].drop_layout_motion().is_animating());assert!(host.named.iter().any(|(_,phase)|*phase=="end"));assert!(host.lists[0].drop_layout_motion().tracked_items()<=6);});
        view.update(cx,|host,cx|{for list in &mut host.lists {list.set_motion_enabled(false);}cx.notify();});cx.refresh().unwrap();cx.run_until_parked();assert_eq!(cx.debug_bounds("sdk-left-item-0").unwrap().top(),first.top());
    }
}
