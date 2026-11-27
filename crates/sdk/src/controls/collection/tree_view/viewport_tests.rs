//! GPUI TestWindow dispatch checks. No native GUI application is launched.
use gpui::{Entity, ScrollDelta, ScrollHandle, TestAppContext, VisualTestContext, point};
use super::*;

struct Page {
    tree: Entity<TreeViewControl<()>>,
    scroll: ScrollHandle,
    focus_exits: usize,
    tree_height: f32,
    _subscription: Subscription,
}

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("page")
            .w(px(300.0))
            .h(px(240.0))
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .child(div().h(px(self.tree_height)).w_full().child(self.tree.clone()))
            .child(div().h(px(900.0)).w_full())
    }
}

fn node(id: &str) -> TreeNode<()> {
    TreeNode::new(id.to_owned(), id.to_owned(), ())
}

fn setup(app: &mut TestAppContext, required: bool) -> (Entity<Page>, &mut VisualTestContext) {
    let (page, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        crate::key_handling::bind_default_control_keys(cx);
        let tree = TreeViewBuilder::new("viewport-tree")
            .items((0..150).map(|i| node(&i.to_string())))
            .motion_enabled(false)
            .wrap_navigation(false)
            .require_focus_for_scroll(required)
            .scrollbar_visibility(crate::controls::ScrollbarVisibility::AlwaysVisible)
            .scrollbar_template(crate::controls::scrollbar::default_scrollbar_template())
            .spawn(cx);
        let subscription = cx.subscribe(&tree, |page: &mut Page, _, event, _| {
            if matches!(event, TreeViewEvent::FocusChanged { focused: false }) {
                page.focus_exits += 1;
            }
        });
        Page { tree, scroll: ScrollHandle::new(), focus_exits: 0, tree_height: 120.0, _subscription: subscription }
    });
    cx.run_until_parked();
    (page, cx)
}

fn wheel(cx: &mut VisualTestContext, y: f32) {
    cx.simulate_event(ScrollEvent {
        position: point(px(20.0), px(30.0)),
        delta: ScrollDelta::Pixels(point(px(0.0), px(y))),
        ..Default::default()
    });
    cx.run_until_parked();
}

#[test]
fn unfocused_wheel_passes_to_page_focused_wheel_is_contained_and_click_away_exits_once() {
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, true);
    let tree = cx.update(|_, app| page.read(app).tree.clone());
    wheel(cx, -40.0);
    cx.update(|_, app| {
        assert_eq!(tree.read(app).list_state.logical_scroll_top().item_ix, 0);
        assert_eq!(tree.read(app).list_state.logical_scroll_top().offset_in_item, px(0.0));
        assert!(page.read(app).scroll.offset().y < px(0.0));
        page.read(app).scroll.set_offset(point(px(0.0), px(0.0)));
    });
    page.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();
    cx.simulate_click(point(px(20.0), px(30.0)), Default::default());
    wheel(cx, -60.0);
    cx.update(|window, app| {
        assert!(tree.read(app).focus_handle.is_focused(window));
        assert!(tree.read(app).list_state.scroll_px_offset_for_scrollbar().y < px(0.0));
        assert_eq!(page.read(app).scroll.offset().y, px(0.0));
    });
    for _ in 0..4 {
        wheel(cx, -100_000.0);
    }
    cx.update(|_, app| assert_eq!(page.read(app).scroll.offset().y, px(0.0)));
    for _ in 0..2 {
        wheel(cx, 100_000.0);
    }
    cx.update(|_, app| assert_eq!(page.read(app).scroll.offset().y, px(0.0)));
    cx.simulate_click(point(px(20.0), px(180.0)), Default::default());
    cx.run_until_parked();
    cx.simulate_click(point(px(20.0), px(180.0)), Default::default());
    cx.run_until_parked();
    cx.update(|window, app| {
        assert!(!tree.read(app).focus_handle.is_focused(window));
        assert_eq!(page.read(app).focus_exits, 1);
    });
    wheel(cx, -40.0);
    cx.update(|_, app| assert!(page.read(app).scroll.offset().y < px(0.0)));
}

#[test]
fn hover_scroll_and_scrollbar_share_position_and_geometry() {
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, false);
    let tree = cx.update(|_, app| page.read(app).tree.clone());
    wheel(cx, -60.0);
    let scrollbar = cx.update(|_, app| {
        let state = tree.read(app);
        let scrollbar = state.scrollbar.clone().unwrap();
        assert_eq!(scrollbar.read(app).value(), -state.list_state.scroll_px_offset_for_scrollbar().y.as_f32());
        assert!(scrollbar.read(app).range().end > 0.0);
        assert_eq!(page.read(app).scroll.offset().y, px(0.0));
        scrollbar
    });
    // Click the actual scrollbar track below its thumb to page forward.
    cx.simulate_click(point(px(294.0), px(105.0)), Default::default());
    cx.run_until_parked();
    cx.update(|_, app| {
        let state = tree.read(app);
        assert!(scrollbar.read(app).value() > 60.0);
        assert_eq!(scrollbar.read(app).value(), -state.list_state.scroll_px_offset_for_scrollbar().y.as_f32());
    });
    tree.update(cx, |tree, cx| tree.set_items([node("only")], cx));
    cx.run_until_parked();
    cx.update(|_, app| {
        assert_eq!(scrollbar.read(app).value(), 0.0);
        assert_eq!(scrollbar.read(app).thumb_fraction(), 1.0);
        assert_eq!(tree.read(app).list_state.max_offset_for_scrollbar().y, px(0.0));
        assert_eq!(tree.read(app).list_state.logical_scroll_top().item_ix, 0);
    });
}

#[test]
fn tree_keys_skip_disabled_children_and_empty_branches_never_visit_siblings() {
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, true);
    let tree = cx.update(|_, app| page.read(app).tree.clone());
    tree.update(cx, |tree, cx| {
        tree.set_items(
            [
                node("root").expanded(true).children([
                    node("disabled").enabled(false).expanded(true).children([node("grandchild")]),
                    node("child"),
                ]),
                node("empty").branch(true).expanded(true),
                node("sibling"),
            ],
            cx,
        );
        tree.select_node_by_id("root", cx);
    });
    cx.update(|window, app| tree.read(app).focus_handle.clone().focus(window, app));
    cx.simulate_keystrokes("right");
    cx.update(|_, app| assert_eq!(tree.read(app).active_node_id().map(|id| id.as_ref()), Some("child")));
    tree.update(cx, |tree, cx| tree.select_node_by_id("grandchild", cx));
    cx.simulate_keystrokes("left");
    cx.update(|_, app| assert_eq!(tree.read(app).active_node_id().map(|id| id.as_ref()), Some("root")));
    tree.update(cx, |tree, cx| tree.select_node_by_id("empty", cx));
    cx.simulate_keystrokes("right");
    cx.update(|_, app| assert_eq!(tree.read(app).active_node_id().map(|id| id.as_ref()), Some("empty")));
    cx.simulate_keystrokes("end down");
    cx.update(|_, app| assert_eq!(tree.read(app).active_node_id().map(|id| id.as_ref()), Some("sibling")));
    cx.simulate_keystrokes("home up left");
    cx.update(|_, app| {
        assert_eq!(tree.read(app).active_node_id().map(|id| id.as_ref()), Some("root"));
        assert!(!tree.read(app).is_expanded(&"root".into()));
    });
}

#[test]
fn keyboard_reveals_rows_without_selecting_and_disabled_tree_passes_wheel() {
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, true);
    let tree = cx.update(|_, app| page.read(app).tree.clone());
    cx.simulate_click(point(px(20.0), px(30.0)), Default::default());
    let selected = cx.update(|_, app| tree.read(app).selected_ids.clone());
    cx.simulate_keystrokes("end");
    cx.run_until_parked();
    cx.update(|_, app| {
        let tree = tree.read(app);
        assert_eq!(tree.active_node_id().map(|id| id.as_ref()), Some("149"));
        assert!(tree.list_state.logical_scroll_top().item_ix > 100);
        assert_eq!(tree.selected_ids, selected);
        assert_eq!(
            tree.scrollbar.as_ref().unwrap().read(app).value(),
            -tree.list_state.scroll_px_offset_for_scrollbar().y.as_f32()
        );
    });
    tree.update(cx, |tree, cx| tree.set_enabled(false, cx));
    cx.simulate_keystrokes("home up right left enter");
    wheel(cx, -40.0);
    cx.update(|_, app| {
        assert_eq!(tree.read(app).active_node_id().map(|id| id.as_ref()), Some("149"));
        assert_eq!(tree.read(app).selected_ids, selected);
        assert!(page.read(app).scroll.offset().y < px(0.0));
    });
}

#[test]
fn completing_multiple_collapses_preserves_a_surviving_middle_scroll_anchor() {
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, false);
    let tree = cx.update(|_, app| page.read(app).tree.clone());
    tree.update(cx, |tree, cx| {
        let children = |prefix: &str| (0..50).map(|i| node(&format!("{prefix}-{i}"))).collect::<Vec<_>>();
        tree.set_items(
            [
                node("first").expanded(true).children(children("a")),
                node("middle"),
                node("last").expanded(true).children(children("b")),
                node("tail").expanded(true).children(children("c")),
            ],
            cx,
        );
        tree.select_node_by_id("a-0", cx);
        tree.set_motion_enabled(true, cx);
        tree.list_state.scroll_to(gpui::ListOffset { item_ix: 51, offset_in_item: px(3.0) });
        tree.rows_pointer.hover("a-0".into(), true);
        tree.rows_pointer.press("a-0".into());
        tree.collapse("first", cx);
        tree.collapse("last", cx);
        assert_eq!(tree.active_node_id().map(|id| id.as_ref()), Some("first"));
        assert!(tree.selected_ids().contains("a-0"));
        assert!(tree.rows_pointer.hovered().is_none());
        assert!(tree.rows_pointer.pressed().is_none());
        assert!(!tree.flat_cache[1].enabled);
        // Settle the clock-dependent motion without sleeping. The next render
        // takes the same projection-rebuild path as natural animation completion.
        tree.expand_transitions.insert("first".into(), DisclosureMotion::new(0.0, true));
        tree.expand_transitions.insert("last".into(), DisclosureMotion::new(0.0, true));
    });
    cx.run_until_parked();
    cx.update(|_, app| {
        let tree = tree.read(app);
        let top = tree.list_state.logical_scroll_top();
        assert_eq!(tree.flat_cache[top.item_ix].id.as_ref(), "middle");
        assert_eq!(top.offset_in_item, px(3.0));
        assert_eq!(tree.list_state.item_count(), 54);
        assert_eq!(tree.active_node_id().map(|id| id.as_ref()), Some("first"));
    });
}

#[test]
fn disabling_animation_settles_an_in_progress_collapse() {
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, false);
    let tree = cx.update(|_, app| page.read(app).tree.clone());
    tree.update(cx, |tree, cx| {
        tree.set_items([node("root").expanded(true).children([node("child")])], cx);
        tree.set_motion_enabled(true, cx);
        tree.collapse("root", cx);
        assert_eq!(tree.flat_cache.len(), 2);
        tree.set_motion_enabled(false, cx);
        assert_eq!(tree.flat_cache.len(), 1);
        assert_eq!(tree.list_state.item_count(), 1);
    });
    cx.run_until_parked();
}

#[test]
fn native_scrollbar_drag_and_resizing_use_list_geometry() {
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, true);
    let tree = cx.update(|_, app| page.read(app).tree.clone());
    let scrollbar = cx.update(|_, app| tree.read(app).scrollbar.clone().unwrap());
    cx.simulate_mouse_down(point(px(294.0), px(12.0)), gpui::MouseButton::Left, Default::default());
    cx.simulate_mouse_move(point(px(294.0), px(30.0)), gpui::MouseButton::Left, Default::default());
    cx.simulate_mouse_move(point(px(294.0), px(90.0)), gpui::MouseButton::Left, Default::default());
    cx.update(|_, app| assert!(tree.read(app).list_state.is_scrollbar_dragging()));
    cx.simulate_mouse_up(point(px(294.0), px(90.0)), gpui::MouseButton::Left, Default::default());
    cx.run_until_parked();
    cx.update(|_, app| {
        let tree = tree.read(app);
        assert!(scrollbar.read(app).value() > scrollbar.read(app).page_step());
        assert!(!tree.list_state.is_scrollbar_dragging());
        assert_eq!(scrollbar.read(app).value(), -tree.list_state.scroll_px_offset_for_scrollbar().y.as_f32());
        assert_eq!(page.read(app).scroll.offset().y, px(0.0));
    });
    page.update(cx, |page, cx| {
        page.tree_height = 180.0;
        cx.notify();
    });
    cx.run_until_parked();
    cx.update(|_, app| {
        assert_eq!(tree.read(app).list_state.viewport_bounds().size.height, px(180.0));
        let maximum = tree.read(app).list_state.max_offset_for_scrollbar().y.as_f32();
        assert!((scrollbar.read(app).thumb_fraction() - 180.0 / (180.0 + maximum)).abs() < 0.001);
        assert_eq!(scrollbar.read(app).value(), -tree.read(app).list_state.scroll_px_offset_for_scrollbar().y.as_f32());
    });
}

#[test]
fn programmatic_and_scrollbar_focus_each_exit_once() {
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, true);
    let tree = cx.update(|_, app| page.read(app).tree.clone());
    cx.update(|window, app| tree.read(app).focus_handle.clone().focus(window, app));
    cx.run_until_parked();
    cx.simulate_click(point(px(20.0), px(180.0)), Default::default());
    cx.run_until_parked();
    cx.update(|_, app| assert_eq!(page.read(app).focus_exits, 1));
    cx.simulate_click(point(px(294.0), px(12.0)), Default::default());
    cx.run_until_parked();
    cx.update(|window, app| assert!(tree.read(app).focus_handle.contains_focused(window, app)));
    cx.simulate_click(point(px(20.0), px(180.0)), Default::default());
    cx.run_until_parked();
    cx.update(|_, app| assert_eq!(page.read(app).focus_exits, 2));
}

#[test]
fn scrollbar_auto_hide_wakes_on_scroll_and_restarts_its_idle_timer() {
    use crate::controls::ScrollbarVisibility;
    use std::time::Duration;
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, false);
    let tree = cx.update(|_, app| page.read(app).tree.clone());
    tree.update(cx, |tree, cx| tree.set_scrollbar_visibility(ScrollbarVisibility::AutoHide, cx));
    cx.run_until_parked();
    assert!(cx.debug_bounds("viewport-tree-scrollbar-chrome").is_none());
    let width = cx.update(|_, app| tree.read(app).list_state.viewport_bounds().size.width);
    wheel(cx, -40.0);
    assert!(cx.debug_bounds("viewport-tree-scrollbar-chrome").is_some());
    cx.executor().advance_clock(Duration::from_millis(1000));
    wheel(cx, -40.0);
    cx.executor().advance_clock(Duration::from_millis(500));
    cx.run_until_parked();
    assert!(cx.debug_bounds("viewport-tree-scrollbar-chrome").is_some());
    cx.executor().advance_clock(Duration::from_millis(800));
    cx.run_until_parked();
    assert!(cx.debug_bounds("viewport-tree-scrollbar-chrome").is_none());
    cx.update(|_, app| assert_eq!(tree.read(app).list_state.viewport_bounds().size.width, width));
}

#[test]
fn hidden_scrollbar_has_no_gutter_and_keeps_wheel_and_keyboard_scrolling() {
    use crate::controls::ScrollbarVisibility;
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, false);
    let tree = cx.update(|_, app| page.read(app).tree.clone());
    wheel(cx, -60.0);
    let top = cx.update(|_, app| tree.read(app).list_state.logical_scroll_top());
    tree.update(cx, |tree, cx| tree.set_scrollbar_visibility(ScrollbarVisibility::Hidden, cx));
    cx.run_until_parked();
    assert!(cx.debug_bounds("viewport-tree-scrollbar-chrome").is_none());
    cx.update(|_, app| {
        let tree = tree.read(app);
        assert_eq!(tree.list_state.viewport_bounds().size.width, px(300.0));
        assert_eq!(tree.list_state.logical_scroll_top().item_ix, top.item_ix);
        assert_eq!(tree.list_state.logical_scroll_top().offset_in_item, top.offset_in_item);
    });
    wheel(cx, -40.0);
    cx.update(|_, app| assert!(tree.read(app).list_state.scroll_px_offset_for_scrollbar().y < px(-60.0)));
    cx.update(|window, app| tree.read(app).focus_handle.clone().focus(window, app));
    cx.simulate_keystrokes("end");
    cx.run_until_parked();
    assert!(cx.debug_bounds("viewport-tree-scrollbar-chrome").is_none());
    cx.update(|_, app| assert!(tree.read(app).list_state.logical_scroll_top().item_ix > 100));
    tree.update(cx, |tree, cx| tree.set_scrollbar_visibility(ScrollbarVisibility::AlwaysVisible, cx));
    cx.run_until_parked();
    assert!(cx.debug_bounds("viewport-tree-scrollbar-chrome").is_some());
    tree.update(cx, |tree, cx| tree.set_items([node("only")], cx));
    cx.run_until_parked();
    assert!(cx.debug_bounds("viewport-tree-scrollbar-chrome").is_none());
}

#[test]
fn auto_hide_ignores_rejected_wheel_and_keeps_the_bar_during_a_long_drag() {
    use crate::controls::ScrollbarVisibility;
    use std::time::Duration;
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, true);
    let tree = cx.update(|_, app| page.read(app).tree.clone());
    tree.update(cx, |tree, cx| tree.set_scrollbar_visibility(ScrollbarVisibility::AutoHide, cx));
    cx.run_until_parked();
    wheel(cx, -40.0);
    assert!(cx.debug_bounds("viewport-tree-scrollbar-chrome").is_none());
    page.update(cx, |page, cx| {
        page.scroll.set_offset(point(px(0.0), px(0.0)));
        cx.notify();
    });
    cx.run_until_parked();
    cx.simulate_click(point(px(20.0), px(30.0)), Default::default());
    wheel(cx, -40.0);
    cx.simulate_mouse_down(point(px(294.0), px(12.0)), gpui::MouseButton::Left, Default::default());
    cx.run_until_parked();
    cx.executor().advance_clock(Duration::from_secs(2));
    cx.run_until_parked();
    assert!(cx.debug_bounds("viewport-tree-scrollbar-chrome").is_some());
    cx.simulate_mouse_up(point(px(294.0), px(12.0)), gpui::MouseButton::Left, Default::default());
    cx.run_until_parked();
    cx.executor().advance_clock(Duration::from_secs(2));
    cx.run_until_parked();
    assert!(cx.debug_bounds("viewport-tree-scrollbar-chrome").is_none());
    cx.update(|window, app| {
        assert!(tree.read(app).focus_handle.is_focused(window));
        assert_eq!(page.read(app).focus_exits, 0);
    });
}

#[test]
fn runtime_policies_empty_content_and_programmatic_reveal_are_independent() {
    use crate::interaction::{WheelScrollPolicy, ScrollBoundaryPolicy};
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, false);
    let tree = cx.update(|_, app| page.read(app).tree.clone());
    tree.update(cx, |tree, cx| tree.set_wheel_scroll_policy(WheelScrollPolicy::PassThrough, cx));
    cx.run_until_parked();
    wheel(cx, -40.0);
    cx.update(|_, app| {
        assert_eq!(tree.read(app).list_state.logical_scroll_top().item_ix, 0);
        assert!(page.read(app).scroll.offset().y < px(0.0));
        page.read(app).scroll.set_offset(point(px(0.0), px(0.0)));
    });
    tree.update(cx, |tree, cx| {
        tree.reveal_item(80);
        cx.notify();
    });
    cx.run_until_parked();
    cx.update(|window, app| {
        assert!(tree.read(app).list_state.logical_scroll_top().item_ix > 0);
        assert!(!tree.read(app).focus_handle.is_focused(window));
        assert!(tree.read(app).selected_ids().is_empty());
    });
    tree.update(cx, |tree, cx| {
        tree.set_items([], cx);
        tree.set_wheel_scroll_policy(WheelScrollPolicy::Pointer, cx);
    });
    cx.run_until_parked();
    wheel(cx, -30.0);
    cx.update(|_, app| assert_eq!(page.read(app).scroll.offset().y, px(0.0)));
    tree.update(cx, |tree, cx| tree.set_scroll_boundary_policy(ScrollBoundaryPolicy::Chain, cx));
    cx.run_until_parked();
    wheel(cx, -30.0);
    cx.update(|_, app| assert!(page.read(app).scroll.offset().y < px(0.0)));
}

#[test]
fn scrollbar_focus_scope_is_independent_of_keyboard_navigation() {
    use crate::interaction::WheelFocusScope;
    let mut app = TestAppContext::single();
    let (page, cx) = setup(&mut app, true);
    let tree = cx.update(|_, app| page.read(app).tree.clone());
    cx.update(|window, app| {
        let handle = tree.read(app).scrollbar.as_ref().unwrap().read(app).focus_handle(app);
        handle.focus(window, app);
    });
    cx.run_until_parked();
    wheel(cx, -30.0);
    cx.update(|_, app| {
        assert_eq!(page.read(app).scroll.offset().y, px(0.0));
        assert!(tree.read(app).list_state.scroll_px_offset_for_scrollbar().y < px(0.0));
    });
    tree.update(cx, |tree, cx| tree.set_wheel_focus_scope(WheelFocusScope::Owner, cx));
    cx.run_until_parked();
    let before = cx.update(|_, app| tree.read(app).list_state.logical_scroll_top());
    wheel(cx, -30.0);
    cx.update(|_, app| {
        let after = tree.read(app).list_state.logical_scroll_top();
        assert_eq!((after.item_ix, after.offset_in_item), (before.item_ix, before.offset_in_item));
        assert!(page.read(app).scroll.offset().y < px(0.0));
        assert!(tree.read(app).selected_ids().is_empty());
    });
}

#[test]
fn embedded_editor_focus_does_not_authorize_tree_keys_or_default_wheel() {
    use crate::interaction::{WheelFocusScope, WheelScrollPolicy};
    use crate::controls::textfield::TextFieldBuilder;
    let mut app = TestAppContext::single();
    let field = TextFieldBuilder::new("embedded-editor").value("editable").spawn(&mut app);
    let (page, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        crate::key_handling::bind_default_control_keys(cx);
        let editor = field.clone();
        let tree = TreeViewBuilder::new("nested-focus-tree")
            .items((0..100).map(|i| node(&i.to_string())))
            .motion_enabled(false)
            .wheel_scroll_policy(WheelScrollPolicy::RequireFocus)
            .leaf_content(move |node, _, _| {
                if node.id.as_ref() == "0" {
                    editor.clone().into_any_element()
                } else {
                    div().child(node.label.clone()).into_any_element()
                }
            })
            .spawn(cx);
        let subscription = cx.subscribe(&tree, |_: &mut Page, _, _, _| {});
        Page { tree, scroll: ScrollHandle::new(), focus_exits: 0, tree_height: 120.0, _subscription: subscription }
    });
    cx.run_until_parked();
    cx.update(|window, app| field.read(app).focus_handle(app).focus(window, app));
    cx.run_until_parked();
    wheel(cx, -30.0);
    cx.update(|_, app| {
        let page = page.read(app);
        assert_eq!(page.tree.read(app).list_state.logical_scroll_top().item_ix, 0);
        assert!(page.scroll.offset().y < px(0.0));
        page.scroll.set_offset(point(px(0.0), px(0.0)));
    });
    let tree = cx.update(|_, app| page.read(app).tree.clone());
    tree.update(cx, |tree, cx| tree.set_wheel_focus_scope(WheelFocusScope::Descendants, cx));
    page.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();
    cx.simulate_keystrokes("down");
    cx.run_until_parked();
    cx.update(|window, app| {
        assert!(field.read(app).focus_handle(app).is_focused(window));
        assert!(tree.read(app).active_node_id().is_none());
        assert!(tree.read(app).selected_ids().is_empty());
    });
    wheel(cx, -30.0);
    cx.update(|_, app| {
        assert!(tree.read(app).list_state.scroll_px_offset_for_scrollbar().y < px(0.0));
        assert_eq!(page.read(app).scroll.offset().y, px(0.0));
    });
}
