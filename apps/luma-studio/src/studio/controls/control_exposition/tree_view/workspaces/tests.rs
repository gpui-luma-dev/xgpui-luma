use xgpui_luma::controls::button::ButtonEvent;
use super::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use gpui::{TestAppContext, VisualTestContext, point, MouseButton};
use xgpui_luma::controls::tree_view::{TreeViewDragEvent, TreeDragEndReason};

fn setup(
    app: &mut TestAppContext,
) -> (Entity<Workspaces>, &mut VisualTestContext, Rc<RefCell<Vec<TreeViewDragEvent>>>) {
    let (view, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        xgpui_luma::key_handling::bind_default_control_keys(cx);
        xgpui_luma::focus::bind_default_focus_keys(cx);
        Workspaces::new(Arc::new(ShadcnLook::built_in()), cx)
    });
    cx.run_until_parked();
    let events = Rc::new(RefCell::new(Vec::new()));
    cx.update(|_, app| {
        for tree in &view.read(app).trees.clone() {
            let events = events.clone();
            app.subscribe(tree, move |_, event, _| {
                if let TreeViewEvent::DragDrop(event) = event {
                    events.borrow_mut().push(event.clone());
                }
            })
            .detach();
        }
    });
    (view, cx, events)
}
fn row(cx: &mut VisualTestContext, side: &str, id: &str) -> gpui::Bounds<gpui::Pixels> {
    cx.debug_bounds(Box::leak(format!("workspace-{side}/node/{id}").into_boxed_str()))
        .unwrap_or_else(|| panic!("missing {side}/{id}"))
}
fn begin(cx: &mut VisualTestContext, side: &str, id: &str) {
    let start = row(cx, side, id).center();
    cx.simulate_mouse_down(start, MouseButton::Left, Default::default());
    cx.simulate_mouse_move(start + point(px(12.0), px(0.0)), MouseButton::Left, Default::default());
    cx.run_until_parked();
    cx.update(|_, app| assert!(app.has_active_drag()));
}
fn drop_on(cx: &mut VisualTestContext, side: &str, id: &str, fraction: f32) {
    let bounds = row(cx, side, id);
    let point = point(bounds.center().x, bounds.top() + bounds.size.height * fraction);
    cx.simulate_mouse_move(point, MouseButton::Left, Default::default());
    cx.run_until_parked();
    cx.simulate_mouse_up(point, MouseButton::Left, Default::default());
    cx.run_until_parked();
}
fn phases(events: &[TreeViewDragEvent]) -> Vec<&'static str> {
    events
        .iter()
        .map(|event| match event {
            TreeViewDragEvent::Started { .. } => "start",
            TreeViewDragEvent::ItemsRemoved { .. } => "remove",
            TreeViewDragEvent::ItemsAdded { .. } => "add",
            TreeViewDragEvent::ItemsReordered { .. } => "reorder",
            TreeViewDragEvent::Dropped { .. } => "drop",
            TreeViewDragEvent::Rejected { .. } => "reject",
            TreeViewDragEvent::Ended { .. } => "end",
        })
        .collect()
}

fn ended(events: &[TreeViewDragEvent]) -> Vec<TreeDragEndReason> {
    events
        .iter()
        .filter_map(|e| match e {
            TreeViewDragEvent::Ended { reason, .. } => Some(*reason),
            _ => None,
        })
        .collect()
}

#[test]
fn tree_view_workspace_drag_transfers_hidden_selection_and_expansion_once() {
    let mut app = TestAppContext::single();
    let (view, cx, events) = setup(&mut app);
    let [source, target] = cx.update(|_, app| view.read(app).trees.clone());
    source.update(cx, |tree, cx| {
        tree.select_node_by_id("guide", cx);
        tree.set_motion_enabled(false, cx);
        tree.collapse("docs", cx);
    });
    cx.run_until_parked();
    begin(cx, "left", "docs");
    drop_on(cx, "right", "archive", 0.5);
    cx.update(|_, app| {
        assert_eq!(source.read(app).items()[0].children[0].id.as_ref(), "empty");
        assert_eq!(target.read(app).items()[0].children[0].id.as_ref(), "docs");
        assert!(target.read(app).selected_ids().contains("guide"));
        assert!(!target.read(app).is_expanded(&"docs".into()));
        assert!(target.read(app).is_expanded(&"archive".into()));
        assert!(source.read(app).selected_ids().is_empty());
    });
    assert_eq!(ended(&events.borrow()), [TreeDragEndReason::MovedAcrossTrees]);
    assert_eq!(phases(&events.borrow()), ["start", "remove", "add", "drop", "end"]);
    assert!(events.borrow().iter().any(|event| matches!(event,
        TreeViewDragEvent::ItemsAdded { node_ids, location, .. }
            if node_ids == &["docs"] && location.position == TreeDropPosition::Into
                && location.parent_id.as_deref() == Some("archive")
    )));
    assert_eq!(events.borrow().iter().filter(|e| matches!(e, TreeViewDragEvent::Dropped { .. })).count(), 1);
}

#[test]
fn tree_view_workspace_before_after_into_and_root_append_follow_hierarchy() {
    for (anchor, fraction, parent, expected_index) in [
        ("docs", 0.1, Some("project"), 0),
        ("docs", 0.9, Some("project"), 1),
        ("empty", 0.5, Some("empty"), 0),
    ] {
        let mut app = TestAppContext::single();
        let (view, cx, events) = setup(&mut app);
        begin(cx, "left", "draft");
        drop_on(cx, "left", anchor, fraction);
        cx.update(|_, app| {
            let mut data = view.read(app).trees[0].read(app).items().to_vec();
            let siblings = &find_mut(&mut data, &parent.unwrap().into()).unwrap().children;
            assert_eq!(siblings[expected_index].id.as_ref(), "draft");
        });
        assert_eq!(ended(&events.borrow()), [TreeDragEndReason::MovedWithinTree]);
        assert_eq!(phases(&events.borrow()), ["start", "reorder", "drop", "end"]);
        assert!(events.borrow().iter().any(|event| matches!(event,
            TreeViewDragEvent::ItemsReordered { node_ids, location, .. }
                if node_ids == &["draft"] && location.parent_id.as_deref() == parent
                    && location.anchor_id.as_deref() == Some(anchor)
                    && location.position == if fraction < 0.25 { TreeDropPosition::Before } else if fraction > 0.75 { TreeDropPosition::After } else { TreeDropPosition::Into }
        )));
    }
    let mut app = TestAppContext::single();
    let (view, cx, events) = setup(&mut app);
    begin(cx, "left", "guide");
    let target = row(cx, "right", "archive").center() + point(px(0.0), px(120.0));
    cx.simulate_mouse_move(target, MouseButton::Left, Default::default());
    cx.simulate_mouse_up(target, MouseButton::Left, Default::default());
    cx.run_until_parked();
    cx.update(|_, app| assert_eq!(view.read(app).trees[1].read(app).items()[1].id.as_ref(), "guide"));
    assert_eq!(ended(&events.borrow()), [TreeDragEndReason::MovedAcrossTrees]);
}

#[test]
fn tree_view_workspace_cycles_disabled_targets_and_escape_do_not_mutate() {
    for (source, target) in [("project", "docs"), ("draft", "locked")] {
        let mut app = TestAppContext::single();
        let (view, cx, events) = setup(&mut app);
        let before = cx.update(|_, app| shape(view.read(app).trees[0].read(app).items()));
        begin(cx, "left", source);
        drop_on(cx, "left", target, 0.5);
        cx.update(|_, app| assert_eq!(shape(view.read(app).trees[0].read(app).items()), before));
        assert_eq!(ended(&events.borrow()), [TreeDragEndReason::Cancelled]);
    }
    let mut app = TestAppContext::single();
    let (_, cx, events) = setup(&mut app);
    begin(cx, "left", "guide");
    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    cx.update(|_, app| assert!(!app.has_active_drag()));
    assert_eq!(ended(&events.borrow()), [TreeDragEndReason::Cancelled]);
}

#[test]
fn tree_view_workspace_embedded_sdk_action_does_not_select_expand_or_drag() {
    let mut app = TestAppContext::single();
    let (view, cx, events) = setup(&mut app);
    let clicks = Rc::new(Cell::new(0));
    cx.update(|_, app| {
        for action in &view.read(app).actions.clone() {
            let clicks = clicks.clone();
            app.subscribe(action, move |_, event, _| {
                if matches!(event, ButtonEvent::Click) {
                    clicks.set(clicks.get() + 1);
                }
            })
            .detach();
        }
    });
    let point = cx.debug_bounds("workspace-action-guide").unwrap().center();
    cx.simulate_click(point, Default::default());
    cx.run_until_parked();
    cx.simulate_mouse_down(point, MouseButton::Left, Default::default());
    cx.simulate_mouse_move(point + gpui::point(px(30.0), px(0.0)), MouseButton::Left, Default::default());
    cx.update(|_, app| {
        assert!(!app.has_active_drag());
        assert!(view.read(app).trees[0].read(app).selected_ids().is_empty());
    });
    assert_eq!(clicks.get(), 1);
    assert!(events.borrow().is_empty());
}

#[test]
fn tree_view_workspace_prepare_move_rejects_collisions_and_preserves_noops() {
    let [left, right] = sample_items();
    let into = |parent: &str| TreeDropLocation {
        parent_id: Some(parent.into()),
        anchor_id: Some(parent.into()),
        position: TreeDropPosition::Into,
    };
    assert!(prepare_move(&left, &left, &["project".into()], &into("docs"), true).is_err());
    assert!(prepare_move(&left, &left, &["draft".into()], &into("locked"), true).is_err());
    let duplicate = vec![node("guide", "collision", "")];
    let root = TreeDropLocation { parent_id: None, anchor_id: None, position: TreeDropPosition::Into };
    assert!(prepare_move(&left, &duplicate, &["docs".into()], &root, false).is_err());
    assert!(!prepare_move(&left, &right, &["draft".into()], &root, true).unwrap().2);
    let mut after = root;
    after.position = TreeDropPosition::After;
    after.anchor_id = Some("project".into());
    assert!(!prepare_move(&left, &left, &["draft".into()], &after, true).unwrap().2);
}

#[test]
fn tree_view_workspace_disclosure_is_separate_from_row_selection() {
    let mut app = TestAppContext::single();
    let (view, cx, events) = setup(&mut app);
    let tree = cx.update(|_, app| view.read(app).trees[0].clone());
    let center = row(cx, "left", "docs").center();
    cx.simulate_click(center, Default::default());
    cx.update(|_, app| {
        assert!(tree.read(app).is_expanded(&"docs".into()));
        assert!(tree.read(app).selected_ids().contains("docs"));
    });
    tree.update(cx, |tree, cx| tree.select_node_by_id("guide", cx));
    let chevron = cx.debug_bounds("tree-disclosure-docs").unwrap().center();
    cx.simulate_click(chevron, Default::default());
    cx.update(|_, app| {
        assert!(!tree.read(app).is_expanded(&"docs".into()));
        assert!(tree.read(app).selected_ids().contains("guide"));
    });
    cx.simulate_mouse_down(chevron, MouseButton::Left, Default::default());
    cx.simulate_mouse_move(chevron + point(px(20.0), px(0.0)), MouseButton::Left, Default::default());
    cx.update(|_, app| assert!(!app.has_active_drag()));
    assert!(events.borrow().is_empty());
}

#[test]
fn tree_view_workspace_empty_destination_and_noop_have_one_completion() {
    let mut app = TestAppContext::single();
    let (view, cx, events) = setup(&mut app);
    let tree = cx.update(|_, app| view.read(app).trees[1].clone());
    tree.update(cx, |tree, cx| tree.set_items([], cx));
    cx.run_until_parked();
    begin(cx, "left", "docs");
    let target = cx.debug_bounds("workspace-right-viewport").unwrap().center();
    cx.simulate_mouse_move(target, MouseButton::Left, Default::default());
    cx.simulate_mouse_up(target, MouseButton::Left, Default::default());
    cx.run_until_parked();
    cx.update(|_, app| assert_eq!(tree.read(app).items()[0].id.as_ref(), "docs"));
    assert_eq!(ended(&events.borrow()), [TreeDragEndReason::MovedAcrossTrees]);
    events.borrow_mut().clear();
    begin(cx, "right", "docs");
    drop_on(cx, "right", "docs", 0.1);
    assert_eq!(ended(&events.borrow()), [TreeDragEndReason::Unchanged]);
}

#[test]
fn tree_view_workspace_filter_and_sort_controls_preserve_hidden_selection_and_source_order() {
    let mut app = TestAppContext::single();
    let (view, cx, _) = setup(&mut app);
    let trees = cx.update(|_, app| view.read(app).trees.clone());
    trees[0].update(cx, |tree, cx| {
        tree.select_node_by_id("readme", cx);
        tree.collapse("project", cx);
    });
    let query = |value: &str, cx: &mut VisualTestContext| {
        cx.update(|_, app| {
            view.read(app).search.clone().update(app, |field, cx| {
                field.set_value(value.to_owned(), cx);
                cx.emit(TextFieldEvent::Change { value: value.to_owned() });
            });
        });
        cx.run_until_parked();
    };
    query("GUIDE", cx);
    cx.update(|_, app| {
        assert_eq!(
            trees[0].read(app).visible_ids(),
            vec![SharedString::from("project"), "docs".into(), "guide".into()]
        );
        assert!(trees[0].read(app).selected_ids().contains("readme"));
        assert!(trees[1].read(app).visible_ids().is_empty());
    });
    query("", cx);
    cx.update(|_, app| {
        assert!(!trees[0].read(app).is_expanded(&"project".into()));
        view.read(app).sort.clone().update(app, |_, cx| {
            cx.emit(SelectorEvent::Change { item_id: "ascending".into(), label: "Name A–Z".into() });
        });
    });
    cx.run_until_parked();
    cx.update(|_, app| {
        assert_eq!(trees[0].read(app).visible_ids(), vec![SharedString::from("draft"), "project".into()]);
        assert_eq!(trees[0].read(app).items()[0].id.as_ref(), "project");
        assert!(trees[0].read(app).selected_ids().contains("readme"));
        view.read(app).sort.clone().update(app, |_, cx| {
            cx.emit(SelectorEvent::Change { item_id: "source".into(), label: "Source order".into() });
        });
    });
    cx.run_until_parked();
    cx.update(|_, app| {
        assert_eq!(trees[0].read(app).visible_ids(), vec![SharedString::from("project"), "draft".into()])
    });
    query("8 KB", cx); // domain metadata is searchable as well as names
    assert!(cx.debug_bounds("workspace-left/node/guide").is_some());
    assert!(cx.debug_bounds("workspace-left/node/readme").is_none());
}

#[test]
fn tree_view_workspace_projected_destinations_reject_reorder_but_allow_into_moves() {
    for filtered in [true, false] {
        let mut app = TestAppContext::single();
        let (view, cx, events) = setup(&mut app);
        let [source, target] = cx.update(|_, app| view.read(app).trees.clone());
        target.update(cx, |tree, cx| {
            if filtered {
                tree.set_filter(|_| true, cx);
            } else {
                tree.set_sort(|a, b| a.label.cmp(&b.label), cx);
            }
        });
        cx.run_until_parked();
        for fraction in [0.1, 0.9] {
            begin(cx, "left", "draft");
            drop_on(cx, "right", "archive", fraction);
            cx.update(|_, app| {
                assert!(target.read(app).items()[0].children.is_empty());
                assert_eq!(source.read(app).items()[1].id.as_ref(), "draft");
            });
        }
        begin(cx, "left", "draft");
        drop_on(cx, "right", "archive", 0.5);
        cx.update(|_, app| assert_eq!(target.read(app).items()[0].children[0].id.as_ref(), "draft"));
        assert_eq!(
            ended(&events.borrow()),
            [TreeDragEndReason::Cancelled, TreeDragEndReason::Cancelled, TreeDragEndReason::MovedAcrossTrees]
        );
    }
}

#[test]
fn tree_view_workspace_drop_into_filtered_empty_view_preserves_hidden_subtree_state() {
    let mut app = TestAppContext::single();
    let (view, cx, events) = setup(&mut app);
    let [source, target] = cx.update(|_, app| view.read(app).trees.clone());
    source.update(cx, |tree, cx| {
        tree.select_node_by_id("guide", cx);
        tree.set_motion_enabled(false, cx);
        tree.collapse("docs", cx);
    });
    target.update(cx, |tree, cx| tree.set_filter(|node| node.id == "missing", cx));
    cx.run_until_parked();
    begin(cx, "left", "docs");
    let point = cx.debug_bounds("workspace-right-viewport").unwrap().center();
    cx.simulate_mouse_move(point, MouseButton::Left, Default::default());
    cx.simulate_mouse_up(point, MouseButton::Left, Default::default());
    cx.run_until_parked();
    target.update(cx, |tree, cx| {
        assert!(tree.visible_ids().is_empty());
        assert!(tree.selected_ids().contains("guide"));
        assert_eq!(tree.items()[1].id.as_ref(), "docs");
        tree.clear_filter(cx);
        assert!(!tree.is_expanded(&"docs".into()));
        assert!(tree.selected_ids().contains("guide"));
    });
    assert_eq!(ended(&events.borrow()), [TreeDragEndReason::MovedAcrossTrees]);
}

#[test]
fn tree_view_workspace_extended_keys_do_not_intercept_embedded_button() {
    use xgpui_luma::controls::tree_view::{TreeViewSelectionMode, TreeViewSelectionPolicy};
    let mut app = TestAppContext::single();
    let (view, cx, _) = setup(&mut app);
    let tree = cx.update(|_, app| view.read(app).trees[0].clone());
    tree.update(cx, |tree, cx| {
        tree.set_selection_policy(TreeViewSelectionPolicy::new(TreeViewSelectionMode::Extended), cx);
        tree.select_node_by_id("readme", cx);
    });
    let activations = Rc::new(Cell::new(0));
    cx.update(|_, app| {
        let activations = activations.clone();
        app.subscribe(&tree, move |_, event, _| {
            if matches!(event, TreeViewEvent::NodeActivated { .. }) {
                activations.set(activations.get() + 1);
            }
        })
        .detach();
    });
    cx.run_until_parked();
    let point = cx.debug_bounds("workspace-action-guide").unwrap().center();
    cx.simulate_click(point, Default::default());
    cx.simulate_keystrokes("space enter cmd-a shift-down");
    cx.run_until_parked();
    assert_eq!(activations.get(), 0);
    cx.update(|_, app| {
        assert_eq!(tree.read(app).selected_ids(), &std::collections::HashSet::from(["readme".into()]));
        assert_eq!(tree.read(app).active_node_id().map(|id| id.as_ref()), Some("readme"));
    });
}

fn toggle(cx: &mut VisualTestContext, id: &str) {
    let center = row(cx, "left", id).center();
    cx.simulate_click(center, gpui::Modifiers { platform: true, ..Default::default() });
    cx.run_until_parked();
}
fn started_roots(events: &[TreeViewDragEvent]) -> Vec<Vec<SharedString>> {
    events
        .iter()
        .filter_map(|event| match event {
            TreeViewDragEvent::Started { node_ids, .. } => Some(node_ids.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn tree_view_group_moves_siblings_once_and_observers_see_both_final_snapshots() {
    let mut app = TestAppContext::single();
    let (view, cx, events) = setup(&mut app);
    let [source, target] = cx.update(|_, app| view.read(app).trees.clone());
    target.update(cx, |tree, cx| tree.select_node_by_id("archive", cx));
    toggle(cx, "guide");
    toggle(cx, "readme"); // click order differs from displayed order
    let changes = Rc::new(Cell::new(0));
    cx.update(|_, app| {
        for tree in [&source, &target] {
            let source = source.clone();
            let target = target.clone();
            let changes = changes.clone();
            app.subscribe(tree, move |_, event, app| {
                if matches!(event, TreeViewEvent::SelectionChanged { .. }) {
                    changes.set(changes.get() + 1);
                    let src = shape(source.read(app).items());
                    let dst = shape(target.read(app).items());
                    assert!(!src.iter().any(|(id, _)| id == "guide" || id == "readme"));
                    assert!(dst.iter().any(|(id, _)| id == "guide"));
                    assert!(dst.iter().any(|(id, _)| id == "readme"));
                }
            })
            .detach();
        }
    });
    begin(cx, "left", "guide");
    drop_on(cx, "right", "archive", 0.5);
    cx.update(|_, app| {
        let dst = target.read(app);
        assert_eq!(
            dst.items()[0].children.iter().map(|node| node.id.as_ref()).collect::<Vec<_>>(),
            ["readme", "guide"]
        );
        assert_eq!(dst.selected_ids(), &HashSet::from(["archive".into(), "readme".into(), "guide".into()]));
        assert_eq!(dst.active_node_id().map(|id| id.as_ref()), Some("guide"));
        assert_eq!(dst.range_anchor_id().map(|id| id.as_ref()), Some("readme"));
        assert!(source.read(app).selected_ids().is_empty());
    });
    assert_eq!(changes.get(), 2);
    assert_eq!(started_roots(&events.borrow()), [vec![SharedString::from("readme"), "guide".into()]]);
    assert_eq!(ended(&events.borrow()), [TreeDragEndReason::MovedAcrossTrees]);
    assert_eq!(events.borrow().iter().filter(|e| matches!(e, TreeViewDragEvent::Dropped { .. })).count(), 1);
}

#[test]
fn tree_view_group_normalizes_ancestors_and_leaves_unrelated_hidden_selection() {
    let mut app = TestAppContext::single();
    let (view, cx, events) = setup(&mut app);
    let [source, target] = cx.update(|_, app| view.read(app).trees.clone());
    toggle(cx, "docs");
    toggle(cx, "guide");
    toggle(cx, "draft");
    begin(cx, "left", "guide");
    drop_on(cx, "right", "archive", 0.5);
    assert_eq!(started_roots(&events.borrow()), [vec![SharedString::from("docs"), "draft".into()]]);
    cx.update(|_, app| {
        let dst = target.read(app);
        assert_eq!(dst.items()[0].children.iter().map(|node| node.id.as_ref()).collect::<Vec<_>>(), ["docs", "draft"]);
        assert_eq!(dst.items()[0].children[0].children.len(), 2);
        assert!(dst.is_expanded(&"docs".into()));
        assert!(dst.selected_ids().contains("guide"));
    });
    // A separately selected collapsed descendant must stay behind.
    source.update(cx, |tree, cx| tree.set_items(sample_items()[0].clone(), cx));
    target.update(cx, |tree, cx| tree.set_items(sample_items()[1].clone(), cx));
    toggle(cx, "guide");
    toggle(cx, "draft");
    source.update(cx, |tree, cx| {
        tree.set_motion_enabled(false, cx);
        tree.collapse("docs", cx);
    });
    cx.run_until_parked();
    events.borrow_mut().clear();
    begin(cx, "left", "draft");
    drop_on(cx, "right", "archive", 0.5);
    assert_eq!(started_roots(&events.borrow()), [vec![SharedString::from("draft")]]);
    cx.update(|_, app| {
        assert!(source.read(app).selected_ids().contains("guide"));
        assert_eq!(target.read(app).items()[0].children.len(), 1);
    });
}

#[test]
fn tree_view_group_uses_sorted_visible_order_and_keeps_filtered_selection() {
    let mut app = TestAppContext::single();
    let (view, cx, events) = setup(&mut app);
    let [source, target] = cx.update(|_, app| view.read(app).trees.clone());
    toggle(cx, "readme");
    toggle(cx, "guide");
    toggle(cx, "draft");
    source.update(cx, |tree, cx| {
        tree.set_sort(|a, b| a.label.cmp(&b.label), cx);
        tree.set_filter(|node| node.id != "draft", cx);
    });
    cx.run_until_parked();
    begin(cx, "left", "readme");
    drop_on(cx, "right", "archive", 0.5);
    assert_eq!(started_roots(&events.borrow()), [vec![SharedString::from("guide"), "readme".into()]]);
    cx.update(|_, app| {
        assert_eq!(
            target.read(app).items()[0].children.iter().map(|node| node.id.as_ref()).collect::<Vec<_>>(),
            ["guide", "readme"]
        );
        assert!(source.read(app).selected_ids().contains("draft"));
    });
}

#[test]
fn tree_view_group_collision_or_cycle_rejects_every_root_and_escape_cancels_once() {
    for collision in [true, false] {
        let mut app = TestAppContext::single();
        let (view, cx, events) = setup(&mut app);
        let [source, target] = cx.update(|_, app| view.read(app).trees.clone());
        toggle(cx, "docs");
        toggle(cx, "draft");
        if collision {
            target.update(cx, |tree, cx| {
                tree.set_items(
                    [node("archive", "Archive", "").branch(true).children([node("draft", "Collision", "")])],
                    cx,
                )
            });
        }
        let before = cx.update(|_, app| [shape(source.read(app).items()), shape(target.read(app).items())]);
        cx.run_until_parked();
        begin(cx, "left", "draft");
        if collision {
            drop_on(cx, "right", "archive", 0.5);
        } else {
            drop_on(cx, "left", "docs", 0.5);
        }
        cx.update(|_, app| assert_eq!([shape(source.read(app).items()), shape(target.read(app).items())], before));
        assert_eq!(ended(&events.borrow()), [TreeDragEndReason::Cancelled]);
        assert!(!events.borrow().iter().any(|e| matches!(e, TreeViewDragEvent::Dropped { .. })));
        events.borrow_mut().clear();
        begin(cx, "left", "draft");
        cx.simulate_keystrokes("escape");
        cx.run_until_parked();
        assert_eq!(ended(&events.borrow()), [TreeDragEndReason::Cancelled]);
    }
}

#[test]
fn tree_view_group_same_tree_noop_and_reparent_preserve_selection() {
    let mut app = TestAppContext::single();
    let (view, cx, events) = setup(&mut app);
    toggle(cx, "readme");
    toggle(cx, "guide");
    begin(cx, "left", "guide");
    drop_on(cx, "left", "readme", 0.1);
    assert_eq!(ended(&events.borrow()), [TreeDragEndReason::Unchanged]);
    events.borrow_mut().clear();
    begin(cx, "left", "guide");
    drop_on(cx, "left", "empty", 0.5);
    cx.update(|_, app| {
        let tree = view.read(app).trees[0].read(app);
        assert_eq!(
            tree.items()[0].children[1].children.iter().map(|node| node.id.as_ref()).collect::<Vec<_>>(),
            ["readme", "guide"]
        );
        assert_eq!(tree.selected_ids(), &HashSet::from(["readme".into(), "guide".into()]));
    });
    assert_eq!(ended(&events.borrow()), [TreeDragEndReason::MovedWithinTree]);
}

#[test]
fn tree_view_group_prepare_resolves_moving_anchors_and_rejects_overlaps_atomically() {
    let data: Vec<_> = ["a", "b", "c", "d", "e"].into_iter().map(|id| node(id, id, "")).collect();
    for (anchor, position, expected) in [
        ("e", TreeDropPosition::After, vec!["a", "c", "e", "b", "d"]),
        ("b", TreeDropPosition::Before, vec!["a", "b", "d", "c", "e"]),
        ("d", TreeDropPosition::After, vec!["a", "c", "b", "d", "e"]),
        ("a", TreeDropPosition::Before, vec!["b", "d", "a", "c", "e"]),
    ] {
        let location = TreeDropLocation { parent_id: None, anchor_id: Some(anchor.into()), position };
        let (_, dst, changed) = prepare_move(&data, &data, &["b".into(), "d".into()], &location, true).unwrap();
        assert!(changed);
        assert_eq!(dst.iter().map(|node| node.id.as_ref()).collect::<Vec<_>>(), expected);
    }
    let [left, right] = sample_items();
    let location = TreeDropLocation { parent_id: None, anchor_id: None, position: TreeDropPosition::Into };
    for roots in [
        vec!["docs".into(), "guide".into()],
        vec!["guide".into(), "docs".into()],
        vec!["draft".into(), "missing".into()],
        vec!["draft".into(), "draft".into()],
    ] {
        assert!(prepare_move(&left, &right, &roots, &location, false).is_err());
    }
    assert_eq!(shape(&left), shape(&sample_items()[0]));
}
