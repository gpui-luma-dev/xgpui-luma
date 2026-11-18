use super::*;
use super::super::{NavNode, SidebarPanelEngineBuilder};
use crate::infra::locale::{LayoutDirection, LocaleContext, set_locale_context};
use gpui::TestAppContext;
use lucide_svg_static::Icon;

#[test]
fn sidebar_indent_and_expansion_keys_follow_live_direction() {
    let mut app = TestAppContext::single();
    let (engine, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        crate::key_handling::bind_default_control_keys(cx);
        SidebarPanelEngine::from_builder(
            SidebarPanelEngineBuilder::new("locale-sidebar").motion_enabled(false).item(
                NavNode::new("parent")
                    .label("Parent")
                    .icon(Icon::Folder)
                    .child(NavNode::new("child").label("Child").icon(Icon::Clock)),
            ),
            cx,
        )
    });
    cx.run_until_parked();
    cx.update(|window, cx| {
        let focus = engine.read(cx).row_focus_handles["parent"].clone();
        focus.focus(window, cx);
    });
    for direction in [LayoutDirection::Ltr, LayoutDirection::Rtl, LayoutDirection::Ltr] {
        cx.update(|_, cx| set_locale_context(LocaleContext { direction, ..Default::default() }, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes(if direction == LayoutDirection::Rtl {
            "left"
        } else {
            "right"
        });
        cx.run_until_parked();
        let parent = cx.debug_bounds("sidebar-label-parent").unwrap();
        let child = cx.debug_bounds("sidebar-label-child").unwrap();
        if direction == LayoutDirection::Rtl {
            assert!(child.right() < parent.right());
        } else {
            assert!(child.left() > parent.left());
        }
        cx.update(|window, cx| {
            assert!(engine.read(cx).model.nodes[0].expanded);
            assert!(engine.read(cx).row_focus_handles["parent"].is_focused(window));
        });
        cx.simulate_keystrokes(if direction == LayoutDirection::Rtl {
            "right"
        } else {
            "left"
        });
        cx.run_until_parked();
        cx.update(|_, cx| assert!(!engine.read(cx).model.nodes[0].expanded));
    }
    engine.update(cx, |view, cx| {
        view.set_collapsed(true, cx);
    });
    cx.run_until_parked();
    for direction in [LayoutDirection::Ltr, LayoutDirection::Rtl, LayoutDirection::Ltr] {
        cx.update(|_, cx| set_locale_context(LocaleContext { direction, ..Default::default() }, cx));
        cx.run_until_parked();
        let icon = cx.debug_bounds("sidebar-rail-icon-parent").unwrap();
        let disclosure = cx.debug_bounds("sidebar-rail-disclosure-parent").unwrap();
        assert_eq!(icon.left() < disclosure.left(), direction == LayoutDirection::Ltr);
        cx.update(|window, cx| {
            let focus = engine.read(cx).rail_focus_handles["parent"].clone();
            focus.focus(window, cx);
        });
        cx.run_until_parked();
        cx.update(|_, cx| assert!(engine.read(cx).open_rail_submenu.is_none()));
        // #266: the expand arrow alone must open the rail parent's submenu.
        cx.simulate_keystrokes(if direction == LayoutDirection::Rtl {
            "left"
        } else {
            "right"
        });
        cx.run_until_parked();
        cx.update(|_, cx| assert!(engine.read(cx).open_rail_submenu.is_some()));
        cx.simulate_keystrokes(if direction == LayoutDirection::Rtl {
            "right"
        } else {
            "left"
        });
        cx.run_until_parked();
        cx.update(|_, cx| assert!(engine.read(cx).open_rail_submenu.is_none()));
    }
}
