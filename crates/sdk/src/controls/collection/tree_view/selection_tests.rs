//! Selection contracts through headless GPUI pointer and keyboard dispatch.
use super::*;
use super::super::TreeViewSelectionPolicy;
use gpui::{Entity, Modifiers, TestAppContext, VisualTestContext};
use std::{cell::RefCell, rc::Rc};

fn leaf(id: &str) -> TreeNode<()> {
    TreeNode::new(id.to_owned(), id.to_owned(), ())
}
fn items() -> Vec<TreeNode<()>> {
    vec![
        leaf("folder")
            .expanded(true)
            .children([leaf("c"), leaf("disabled").enabled(false), leaf("a"), leaf("b")]),
        leaf("last"),
    ]
}
fn setup(app: &mut TestAppContext) -> (Entity<TreeViewControl<()>>, &mut VisualTestContext) {
    app.add_window_view(|window, cx| {
        window.activate_window();
        crate::key_handling::bind_default_control_keys(cx);
        TreeViewControl::from_builder(
            TreeViewBuilder::new("selection-tree")
                .items(items())
                .selection_mode(TreeViewSelectionMode::Extended)
                .expand_on_row_click(false)
                .motion_enabled(false)
                .wrap_navigation(false),
            cx,
        )
    })
}
fn click(cx: &mut VisualTestContext, id: &str, modifiers: Modifiers) {
    cx.run_until_parked();
    let selector: &'static str = Box::leak(format!("selection-tree/node/{id}").into_boxed_str());
    let bounds = cx.debug_bounds(selector).unwrap();
    cx.simulate_click(bounds.center(), modifiers);
}
fn selected(tree: &Entity<TreeViewControl<()>>, cx: &mut VisualTestContext, ids: &[&str]) {
    cx.update(|_, app| {
        assert_eq!(tree.read(app).selected_ids(), &ids.iter().map(|id| SharedString::from((*id).to_owned())).collect())
    });
}
fn record(tree: &Entity<TreeViewControl<()>>, cx: &mut VisualTestContext) -> Rc<RefCell<Vec<TreeViewEvent<()>>>> {
    cx.run_until_parked();
    let events = Rc::new(RefCell::new(Vec::new()));
    cx.update(|_, app| {
        let events = events.clone();
        app.subscribe(tree, move |_, event, _| events.borrow_mut().push(event.clone())).detach();
    });
    events
}

#[test]
fn row_lookup_tracks_sort_filter_expansion_and_replacement() {
    let mut app = TestAppContext::single();
    let (tree, cx) = setup(&mut app);
    let handlers = tree.update(cx, |tree, cx| {
        assert_eq!(tree.flat_index_for_id(&"c".into()), Some(1));
        let handlers = tree.template_row_handlers("c".into(), cx);
        let reused = tree.template_row_handlers("c".into(), cx);
        assert!(Rc::ptr_eq(&handlers.hover, &reused.hover));
        assert!(Rc::ptr_eq(&handlers.click, &reused.click));
        tree.set_sort(|a, b| a.label.cmp(&b.label), cx);
        assert_eq!(tree.flat_index_for_id(&"c".into()), Some(3));
        handlers
    });
    // A handler built before sorting must resolve the node's current position.
    cx.update(|window, app| (handlers.hover)(&true, window, app));
    cx.update(|_, app| assert_eq!(tree.read(app).rows_pointer.hovered().map(|id| id.as_ref()), Some("c")));
    click(cx, "c", Modifiers::default());
    selected(&tree, cx, &["c"]);
    tree.update(cx, |tree, cx| {
        tree.collapse("folder", cx);
        assert_eq!(tree.flat_index_for_id(&"c".into()), None);
        tree.expand("folder", cx);
        assert_eq!(tree.flat_index_for_id(&"c".into()), Some(3));
        tree.set_filter(|node| node.id == "c", cx);
        assert_eq!(tree.flat_index_for_id(&"c".into()), Some(1));
        assert_eq!(tree.flat_index_for_id(&"last".into()), None);
        tree.clear_filter(cx);
        tree.clear_sort(cx);
        tree.set_items([leaf("last"), leaf("c")], cx);
        assert_eq!(tree.flat_index_for_id(&"c".into()), Some(1));
        assert_eq!(tree.flat_index_for_id(&"folder".into()), None);
        tree.set_items([], cx);
        assert_eq!(tree.flat_index_for_id(&"c".into()), None);
        assert!(tree.flat_indices.is_empty());
        assert!(tree.row_handlers.borrow().is_empty());
    });
}

#[test]
fn pointer_ranges_contract_toggle_and_skip_disabled_in_displayed_order() {
    let mut app = TestAppContext::single();
    let (tree, cx) = setup(&mut app);
    click(cx, "c", Modifiers::default());
    click(cx, "b", Modifiers { shift: true, ..Default::default() });
    selected(&tree, cx, &["c", "a", "b"]);
    click(cx, "a", Modifiers { shift: true, ..Default::default() });
    selected(&tree, cx, &["c", "a"]);
    click(cx, "last", Modifiers { platform: true, ..Default::default() });
    selected(&tree, cx, &["c", "a", "last"]);
    click(cx, "b", Modifiers { control: true, shift: true, ..Default::default() });
    selected(&tree, cx, &["c", "a", "b", "last"]);
    click(cx, "c", Modifiers { control: true, ..Default::default() });
    selected(&tree, cx, &["a", "b", "last"]);
    click(cx, "disabled", Modifiers::default());
    selected(&tree, cx, &["a", "b", "last"]);
    tree.update(cx, |tree, cx| {
        tree.set_sort(|a, b| a.label.cmp(&b.label), cx);
        tree.set_filter(|node| node.id != "c", cx);
    });
    click(cx, "a", Modifiers::default());
    click(cx, "last", Modifiers { shift: true, ..Default::default() });
    selected(&tree, cx, &["a", "b", "last"]);
}
#[test]
fn keyboard_focus_space_activation_ranges_and_select_all_are_distinct() {
    let mut app = TestAppContext::single();
    let (tree, cx) = setup(&mut app);
    click(cx, "c", Modifiers::default());
    let events = record(&tree, cx);
    cx.simulate_keystrokes("down");
    selected(&tree, cx, &["c"]);
    cx.simulate_keystrokes("shift-down");
    selected(&tree, cx, &["a", "b"]);
    cx.simulate_keystrokes("shift-up");
    selected(&tree, cx, &["a"]);
    cx.simulate_keystrokes("end space");
    selected(&tree, cx, &["last"]);
    cx.simulate_keystrokes("ctrl-space");
    selected(&tree, cx, &[]);
    cx.simulate_keystrokes("home enter");
    selected(&tree, cx, &[]);
    cx.update(|_, app| assert!(tree.read(app).is_expanded(&"folder".into())));
    cx.simulate_keystrokes("shift-end");
    selected(&tree, cx, &["folder", "c", "a", "b", "last"]);
    cx.simulate_keystrokes("shift-down"); // range remains bounded
    selected(&tree, cx, &["folder", "c", "a", "b", "last"]);
    cx.simulate_keystrokes("cmd-shift-a cmd-a ctrl-shift-a ctrl-a");
    selected(&tree, cx, &["folder", "c", "a", "b", "last"]);
    cx.run_until_parked();
    assert_eq!(
        events
            .borrow()
            .iter()
            .filter(|event| matches!(event, TreeViewEvent::NodeActivated { node_id, .. } if node_id == "folder"))
            .count(),
        1
    );
    assert!(!events.borrow().iter().any(|event| matches!(event, TreeViewEvent::NodeCollapsed { .. })));
    cx.simulate_keystrokes("escape");
    // Without an enclosing focus scope, Escape leaves selection alone.
    selected(&tree, cx, &["folder", "c", "a", "b", "last"]);
}
#[test]
fn hidden_selection_survives_select_all_and_clear_emits_only_deltas() {
    let mut app = TestAppContext::single();
    let (tree, cx) = setup(&mut app);
    click(cx, "c", Modifiers::default());
    tree.update(cx, |tree, cx| {
        tree.collapse("folder", cx);
    });
    let events = record(&tree, cx);
    tree.update(cx, |tree, cx| {
        tree.select_all(cx);
        tree.select_all(cx);
    });
    selected(&tree, cx, &["c", "folder", "last"]);
    tree.update(cx, |tree, cx| {
        tree.clear_selection(cx);
        tree.clear_selection(cx);
    });
    selected(&tree, cx, &[]);
    cx.run_until_parked();
    assert_eq!(
        events
            .borrow()
            .iter()
            .filter(|event| matches!(event, TreeViewEvent::SelectionChanged { .. }))
            .count(),
        2
    );
    cx.update(|_, app| assert_eq!(tree.read(app).active_node_id().map(|id| id.as_ref()), Some("folder")));
    // A cleared anchor is seeded from the old active ID before mouse-down moves it.
    click(cx, "last", Modifiers { shift: true, ..Default::default() });
    selected(&tree, cx, &["folder", "last"]);
}
#[test]
fn anchor_reconciles_on_collapse_projection_disable_and_removal() {
    let mut app = TestAppContext::single();
    let (tree, cx) = setup(&mut app);
    tree.update(cx, |tree, cx| {
        tree.select_node_by_id("c", cx);
        tree.collapse("folder", cx);
        assert_eq!(tree.range_anchor_id().map(|id| id.as_ref()), Some("folder"));
        tree.expand("folder", cx);
        tree.select_node_by_id("c", cx);
        tree.set_filter(|node| node.id == "a", cx);
        assert_eq!(tree.range_anchor_id().map(|id| id.as_ref()), Some("folder"));
        tree.clear_filter(cx);
        tree.select_node_by_id("c", cx);
        let mut data = items();
        data[0].children[0].enabled = false;
        tree.replace_items(data, cx).unwrap();
        assert_eq!(tree.range_anchor_id().map(|id| id.as_ref()), Some("folder"));
        tree.replace_items([leaf("last")], cx).unwrap();
        assert_eq!(tree.range_anchor_id().map(|id| id.as_ref()), Some("last"));
        tree.set_filter(|_| false, cx);
        assert!(tree.range_anchor_id().is_none());
    });
}
#[test]
fn runtime_policies_retain_single_deterministically_and_opt_in_to_following() {
    let mut app = TestAppContext::single();
    let (tree, cx) = setup(&mut app);
    click(cx, "c", Modifiers::default());
    click(cx, "a", Modifiers { platform: true, ..Default::default() });
    let events = record(&tree, cx);
    tree.update(cx, |tree, cx| {
        let policy = TreeViewSelectionPolicy {
            mode: TreeViewSelectionMode::Single,
            toggle_off: true,
            selection_follows_active: true,
        };
        tree.set_selection_policy(policy, cx);
        tree.set_selection_policy(policy, cx);
    });
    selected(&tree, cx, &["a"]);
    cx.simulate_keystrokes("down");
    selected(&tree, cx, &["b"]);
    click(cx, "b", Modifiers::default());
    selected(&tree, cx, &[]);
    cx.run_until_parked();
    assert_eq!(
        events
            .borrow()
            .iter()
            .filter(|event| matches!(event, TreeViewEvent::SelectionPolicyChanged { .. }))
            .count(),
        1
    );
    tree.update(cx, |tree, cx| {
        tree.set_selection_policy(TreeViewSelectionPolicy::new(TreeViewSelectionMode::Multiple), cx)
    });
    click(cx, "c", Modifiers::default());
    click(cx, "a", Modifiers::default());
    selected(&tree, cx, &["c", "a"]);
    tree.update(cx, |tree, cx| {
        tree.set_selection_policy(
            TreeViewSelectionPolicy {
                mode: TreeViewSelectionMode::Extended,
                toggle_off: true,
                selection_follows_active: false,
            },
            cx,
        )
    });
    click(cx, "a", Modifiers::default());
    selected(&tree, cx, &["c"]);
    tree.update(cx, |tree, cx| {
        tree.set_selection_policy(TreeViewSelectionPolicy::new(TreeViewSelectionMode::None), cx)
    });
    click(cx, "b", Modifiers::default());
    selected(&tree, cx, &[]);
}
#[test]
fn extended_plain_navigation_respects_wrap_but_ranges_do_not() {
    let mut app = TestAppContext::single();
    let (tree, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        crate::key_handling::bind_default_control_keys(cx);
        TreeViewControl::from_builder(
            TreeViewBuilder::new("selection-tree")
                .items(items())
                .selection_mode(TreeViewSelectionMode::Extended)
                .motion_enabled(false),
            cx,
        )
    });
    click(cx, "last", Modifiers::default());
    cx.simulate_keystrokes("shift-down");
    selected(&tree, cx, &["last"]);
    cx.simulate_keystrokes("down");
    cx.update(|_, app| assert_eq!(tree.read(app).active_node_id().map(|id| id.as_ref()), Some("folder")));
    selected(&tree, cx, &["last"]);
}
