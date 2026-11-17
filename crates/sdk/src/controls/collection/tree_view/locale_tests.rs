use super::*;
use gpui::TestAppContext;
use crate::infra::locale::{LayoutDirection, LocaleContext, set_locale_context};
#[test]
fn rtl_tree_mirrors_indent_and_physical_expansion_keys_preserving_active_node() {
    let mut app = TestAppContext::single();
    let (tree, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        crate::key_handling::bind_default_control_keys(cx);
        TreeViewControl::from_builder(
            TreeViewBuilder::new("locale-tree")
                .motion_enabled(false)
                .items([TreeNode::new("parent", "Parent", ()).children([TreeNode::new("child", "Child", ())])]),
            cx,
        )
    });
    cx.run_until_parked();
    cx.update(|window, cx| {
        let focus = tree.read(cx).focus_handle.clone();
        focus.focus(window, cx);
    });
    for direction in [LayoutDirection::Ltr, LayoutDirection::Rtl, LayoutDirection::Ltr] {
        cx.update(|_, cx| set_locale_context(LocaleContext { direction, ..Default::default() }, cx));
        cx.simulate_keystrokes("home");
        cx.simulate_keystrokes(if direction == LayoutDirection::Rtl {
            "left"
        } else {
            "right"
        });
        cx.run_until_parked();
        let parent = cx.debug_bounds("tree-label-parent").unwrap();
        let child = cx.debug_bounds("tree-label-child").unwrap();
        if direction == LayoutDirection::Rtl {
            assert!(child.right() < parent.right());
        } else {
            assert!(child.left() > parent.left());
        }
        cx.update(|window, cx| {
            assert!(tree.read(cx).is_expanded(&"parent".into()));
            assert_eq!(tree.read(cx).active_node_id.as_deref(), Some("parent"));
            assert!(tree.read(cx).focus_handle.is_focused(window));
        });
        cx.simulate_keystrokes(if direction == LayoutDirection::Rtl {
            "right"
        } else {
            "left"
        });
        cx.run_until_parked();
        cx.update(|_, cx| assert!(!tree.read(cx).is_expanded(&"parent".into())));
    }
}
