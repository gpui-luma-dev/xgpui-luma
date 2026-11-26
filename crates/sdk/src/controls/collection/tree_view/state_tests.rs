//! Headless control transactions; GPUI TestWindow never launches a platform app.
use std::{cell::RefCell, rc::Rc};

use gpui::{Entity, TestAppContext, VisualTestContext};

use super::*;

fn leaf(id: &str) -> TreeNode<()> {
    TreeNode::new(id.to_owned(), id.to_owned(), ())
}

fn items() -> Vec<TreeNode<()>> {
    vec![leaf("a").expanded(true).children([leaf("x"), leaf("y")]), leaf("b").branch(true)]
}

fn setup(
    app: &mut TestAppContext,
    mode: TreeViewSelectionMode,
) -> (Entity<TreeViewControl<()>>, &mut VisualTestContext) {
    app.add_window_view(|_, cx| {
        TreeViewControl::from_builder(
            TreeViewBuilder::new("tree-test").items(items()).selection_mode(mode).motion_enabled(false),
            cx,
        )
    })
}

fn record(tree: &Entity<TreeViewControl<()>>, cx: &mut VisualTestContext) -> Rc<RefCell<Vec<TreeViewEvent<()>>>> {
    cx.run_until_parked();
    let events = Rc::new(RefCell::new(Vec::new()));
    cx.update(|_, app| {
        let events = events.clone();
        app.subscribe(tree, move |_, event, _| {
            // Relayout may move the pointer to another row or shift a keyed
            // scroll anchor. These tests assert semantic state transactions.
            if !matches!(event, TreeViewEvent::RowHoverChanged { .. } | TreeViewEvent::ScrollChanged { .. }) {
                events.borrow_mut().push(event.clone());
            }
        })
        .detach();
    });
    events
}

#[test]
fn reorder_and_reparent_preserve_keys_expansion_and_hidden_selection() {
    let mut app = TestAppContext::single();
    let (tree, cx) = setup(&mut app, TreeViewSelectionMode::Multiple);
    tree.update(cx, |tree, cx| {
        tree.select_node_by_id("x", cx);
        tree.collapse("a", cx);
    });
    let events = record(&tree, cx);
    tree.update(cx, |tree, cx| {
        tree.replace_items(
            [
                leaf("b").children([leaf("x")]),
                leaf("a").expanded(true).children([leaf("y")]),
                leaf("new").expanded(true).children([leaf("z")]),
            ],
            cx,
        )
        .unwrap();
        assert_eq!(tree.selected_ids(), &HashSet::from(["x".into()]));
        assert!(!tree.is_expanded(&"a".into())); // existing initial hint must not reopen it
        assert!(!tree.is_expanded(&"b".into()));
        assert!(tree.is_expanded(&"new".into()));
        assert_eq!(tree.flat_cache.iter().map(|n| n.id.as_ref()).collect::<Vec<_>>(), ["b", "a", "new", "z"]);
    });
    cx.run_until_parked();
    assert!(!events.borrow().iter().any(|event| matches!(event, TreeViewEvent::SelectionChanged { .. })));
}

#[test]
fn active_reparenting_prefers_new_visible_ancestor_and_removal_prefers_old_ancestor() {
    let mut app = TestAppContext::single();
    let (tree, cx) = setup(&mut app, TreeViewSelectionMode::Single);
    tree.update(cx, |tree, cx| tree.select_node_by_id("x", cx));
    let events = record(&tree, cx);
    tree.update(cx, |tree, cx| {
        tree.replace_items([leaf("a").children([leaf("y")]), leaf("b").children([leaf("x")])], cx).unwrap();
        assert_eq!(tree.active_node_id().map(|id| id.as_ref()), Some("b"));
        assert!(tree.selected_ids().contains("x"));
    });
    cx.run_until_parked();
    assert_eq!(events.borrow().iter().filter(|e| matches!(e, TreeViewEvent::ActiveNodeChanged { .. })).count(), 1);
    events.borrow_mut().clear();
    tree.update(cx, |tree, cx| {
        tree.expand("b", cx);
        tree.select_node_by_id("x", cx);
    });
    cx.run_until_parked();
    events.borrow_mut().clear();
    tree.update(cx, |tree, cx| {
        tree.replace_items([leaf("b").branch(true)], cx).unwrap();
        assert_eq!(tree.active_node_id().map(|id| id.as_ref()), Some("b"));
        assert!(tree.selected_ids().is_empty());
    });
    cx.run_until_parked();
    let events = events.borrow();
    assert!(matches!(&events[0], TreeViewEvent::SelectionChanged { selected_ids } if selected_ids.is_empty()));
    assert!(matches!(&events[1], TreeViewEvent::ActiveNodeChanged { node_id } if node_id.as_deref() == Some("b")));
    assert_eq!(events.len(), 2, "{events:?}");
}

#[test]
fn disabled_removed_and_empty_replacements_reconcile_once() {
    let mut app = TestAppContext::single();
    let (tree, cx) = setup(&mut app, TreeViewSelectionMode::Single);
    tree.update(cx, |tree, cx| tree.select_node_by_id("x", cx));
    let events = record(&tree, cx);
    tree.update(cx, |tree, cx| {
        tree.replace_items([leaf("a").enabled(false).children([leaf("x").enabled(false)]), leaf("b")], cx)
            .unwrap();
        assert!(tree.selected_ids().is_empty());
        assert_eq!(tree.active_node_id().map(|id| id.as_ref()), Some("b"));
        assert!(!tree.is_expanded(&"b".into()));
    });
    cx.run_until_parked();
    assert_eq!(events.borrow().len(), 2, "{:?}", events.borrow());
    tree.update(cx, |tree, cx| {
        tree.replace_items([], cx).unwrap();
        assert!(tree.active_node_id().is_none());
        assert!(tree.expanded_ids.is_empty());
        tree.replace_items(items(), cx).unwrap();
        assert!(tree.active_node_id().is_none()); // loading data does not activate a row
    });
}

#[test]
fn duplicate_descendants_leave_data_motion_scroll_and_events_untouched() {
    let mut app = TestAppContext::single();
    let (tree, cx) = setup(&mut app, TreeViewSelectionMode::Single);
    tree.update(cx, |tree, cx| {
        tree.select_node_by_id("x", cx);
        tree.set_motion_enabled(true, cx);
        tree.collapse("a", cx);
        tree.hovered_node_id = Some("a".into());
        tree.pressed_node_id = Some("a".into());
    });
    let events = record(&tree, cx);
    tree.update(cx, |tree, cx| {
        let old_top = tree.list_state.logical_scroll_top();
        let expanded = tree.expanded_ids.clone();
        let active = tree.active_node_id.clone();
        let motion: HashMap<_, _> = tree
            .expand_transitions
            .iter()
            .map(|(id, motion)| (id.clone(), (motion.progress(), motion.is_animating())))
            .collect();
        let duplicate =
            || vec![leaf("first").children([leaf("duplicate")]), leaf("second").children([leaf("duplicate")])];
        assert_eq!(tree.replace_items(duplicate(), cx), Err(TreeViewError::DuplicateId("duplicate".into())));
        assert_eq!(tree.try_set_items(duplicate(), cx), Err(TreeViewError::DuplicateId("duplicate".into())));
        tree.set_items(duplicate(), cx);
        assert_eq!(tree.items()[0].id.as_ref(), "a");
        assert_eq!(tree.selected_ids(), &HashSet::from(["x".into()]));
        assert_eq!(tree.active_node_id, active);
        assert_eq!(tree.expanded_ids, expanded);
        assert_eq!(
            tree.expand_transitions
                .iter()
                .map(|(id, motion)| (id.clone(), (motion.progress(), motion.is_animating())))
                .collect::<HashMap<_, _>>(),
            motion
        );
        assert_eq!(tree.hovered_node_id, Some("a".into()));
        assert_eq!(tree.pressed_node_id, Some("a".into()));
        assert_eq!(tree.list_state.logical_scroll_top().item_ix, old_top.item_ix);
        assert_eq!(tree.list_state.logical_scroll_top().offset_in_item, old_top.offset_in_item);
    });
    // Do not drive animation frames here: only the rejected transactions are under test.
    cx.update(|_, _| {});
    assert!(events.borrow().is_empty(), "{:?}", events.borrow());
}

#[test]
fn repeat_selection_and_membership_preserving_updates_emit_no_selection_events() {
    let mut app = TestAppContext::single();
    let (tree, cx) = setup(&mut app, TreeViewSelectionMode::Single);
    tree.update(cx, |tree, cx| tree.select_node_by_id("x", cx));
    let events = record(&tree, cx);
    tree.update(cx, |tree, cx| {
        tree.select_node_by_id("x", cx);
        tree.handle_node_select(1, cx);
        tree.replace_items([leaf("b").branch(true), leaf("a").children([leaf("y"), leaf("x")])], cx)
            .unwrap();
        assert_eq!(tree.active_node_id().map(|id| id.as_ref()), Some("x"));
        tree.replace_items(tree.items().to_vec(), cx).unwrap();
    });
    cx.run_until_parked();
    assert!(events.borrow().is_empty(), "{:?}", events.borrow());
}

#[test]
fn legacy_reset_emits_final_selection_and_active_changes_and_restores_hints() {
    let mut app = TestAppContext::single();
    let (tree, cx) = setup(&mut app, TreeViewSelectionMode::Single);
    tree.update(cx, |tree, cx| {
        tree.collapse("a", cx);
        tree.select_node_by_id("b", cx);
    });
    let events = record(&tree, cx);
    tree.update(cx, |tree, cx| {
        tree.set_items(items(), cx);
        assert!(tree.selected_ids().is_empty());
        assert!(tree.active_node_id().is_none());
        assert!(tree.is_expanded(&"a".into()));
    });
    cx.run_until_parked();
    assert_eq!(events.borrow().len(), 2, "{:?}", events.borrow());
    events.borrow_mut().clear();
    tree.update(cx, |tree, cx| tree.set_items(items(), cx));
    cx.run_until_parked();
    assert!(events.borrow().is_empty(), "{:?}", events.borrow());
}

#[test]
fn no_selection_mode_only_changes_active_state() {
    let mut app = TestAppContext::single();
    let (tree, cx) = setup(&mut app, TreeViewSelectionMode::None);
    let events = record(&tree, cx);
    tree.update(cx, |tree, cx| {
        tree.handle_node_select(1, cx);
        tree.handle_node_select(1, cx);
    });
    cx.run_until_parked();
    assert_eq!(events.borrow().len(), 1);
    assert!(
        matches!(&events.borrow()[0], TreeViewEvent::ActiveNodeChanged { node_id } if node_id.as_deref() == Some("x"))
    );
}

#[test]
fn replacement_retains_scroll_anchor_and_invalidates_same_count_measurements() {
    let mut app = TestAppContext::single();
    let (tree, cx) = setup(&mut app, TreeViewSelectionMode::Single);
    tree.update(cx, |tree, cx| {
        tree.set_items((0..200).map(|i| leaf(&i.to_string())), cx);
        tree.list_state.scroll_to(gpui::ListOffset { item_ix: 80, offset_in_item: px(3.0) });
    });
    cx.run_until_parked();
    tree.update(cx, |tree, cx| {
        let top = tree.list_state.logical_scroll_top();
        let id = tree.flat_cache[top.item_ix].id.clone();
        tree.replace_items(tree.items().iter().rev().cloned().collect::<Vec<_>>(), cx).unwrap();
        let next = tree.list_state.logical_scroll_top();
        assert_eq!(tree.flat_cache[next.item_ix].id, id);
        assert_eq!(next.offset_in_item, top.offset_in_item);
        assert_eq!(tree.list_state.item_count(), 200);
        // reset discards previously measured row geometry even at equal counts.
        assert!(tree.list_state.bounds_for_item(next.item_ix).is_none());
    });
    cx.run_until_parked();
}

#[test]
fn only_existing_branches_can_have_expansion_state() {
    let mut app = TestAppContext::single();
    let (tree, cx) = setup(&mut app, TreeViewSelectionMode::Single);
    tree.update(cx, |tree, cx| {
        tree.expand("missing", cx);
        tree.expand("x", cx);
        assert!(!tree.is_expanded(&"missing".into()));
        assert!(!tree.is_expanded(&"x".into()));
        tree.replace_items([leaf("a").expanded(true), leaf("new").branch(true).expanded(true)], cx).unwrap();
        assert!(!tree.is_expanded(&"a".into())); // branch became a leaf
        assert!(tree.is_expanded(&"new".into())); // newly loaded lazy branch
        assert_eq!(tree.expand_transitions.len(), 1);
    });
}

/// A state override draws the mapped row in its mapped state and leaves the real state alone (#397).
#[test]
fn a_state_override_draws_one_row_hovered_without_changing_state() {
    let mut app = TestAppContext::single();
    let (tree, cx) = setup(&mut app, TreeViewSelectionMode::Single);
    let hover = crate::controls::tree_view::StyledTreeView::sdk()
        .row(crate::theme::InteractionState { hovered: true, ..Default::default() }, false)
        .background
        .expect("the SDK paints a hovered row");
    let hovered_rows = |cx: &mut VisualTestContext| {
        cx.run_until_parked();
        cx.update(|window, _| {
            window
                .painted_quads()
                .iter()
                .filter(|quad| quad.background == gpui::Background::from(hover))
                .count()
        })
    };
    let before = hovered_rows(cx);
    tree.update(cx, |tree, cx| {
        tree.set_item_state_override(
            Some(std::sync::Arc::new(|id: &SharedString, state: CompositeItemState| CompositeItemState {
                hovered: state.hovered || id.as_ref() == "x",
                ..state
            })),
            cx,
        );
    });
    assert_eq!(hovered_rows(cx), before + 1, "only the mapped row draws hovered");
    cx.update(|_, app| assert!(tree.read(app).selected_ids().is_empty(), "state is untouched"));
    tree.update(cx, |tree, cx| tree.set_item_state_override(None, cx));
    assert_eq!(hovered_rows(cx), before, "cleared");
}
