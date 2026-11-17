use super::*;
use crate::infra::{locale::set_locale_context, menu_item::MenuItem, state::MenuPath};
use crate::motion::overlay_presence::OverlayPresence;
use gpui::{Context, Render, TestAppContext, size};

struct Harness;
impl Render for Harness {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(deferred(render_rail_submenu_overlay(
            RenderedRailSubmenu {
                id: "rail-popup".into(),
                parent_node_id: "rail-parent".into(),
                parent_bounds: Bounds::new(point(px(400.), px(100.)), size(px(40.), px(40.))),
                items: vec![
                    MenuItem::new("branch")
                        .icon(LucideIcon::Folder)
                        .submenu([MenuItem::new("leaf").icon(LucideIcon::Clock)]),
                ],
                open_submenu: Some(0),
                active_path: Some(MenuPath::Submenu { parent: 0, child: 0 }),
                presence: OverlayPresence::new(true, false),
                content_size: None,
            },
            LocaleContext::current(cx).direction,
            default_floating_menu_theme().resolve(),
            default_sidebar_theme().geometry(),
            vec![Box::new(|_, _, _| {})],
            vec![Box::new(|_, _, _| {})],
            Some(Rc::new(|_, _, _| {})),
            Rc::new(|_, _, _| {}),
        )))
    }
}

#[test]
fn rail_popup_and_nested_rows_follow_live_direction() {
    let mut app = TestAppContext::single();
    let (_, cx) = app.add_window_view(|_, _| Harness);
    for direction in [LayoutDirection::Ltr, LayoutDirection::Rtl, LayoutDirection::Ltr] {
        cx.update(|_, cx| set_locale_context(LocaleContext { direction, ..Default::default() }, cx));
        cx.run_until_parked();
        let root = cx.debug_bounds("floating-row-branch").unwrap();
        // The flyout is adjacent to the rail's physical edge for each direction.
        if direction == LayoutDirection::Rtl {
            assert!(root.right() <= px(400.));
        } else {
            assert!(root.left() >= px(440.));
        }
        let nested = cx.debug_bounds("floating-submenu-branch").unwrap();
        assert_eq!(nested.left() < root.left(), direction == LayoutDirection::Rtl);
        for (icon_selector, label_selector) in
            [("floating-icon-branch", "floating-label-branch"), ("floating-icon-leaf", "floating-label-leaf")]
        {
            let icon = cx.debug_bounds(icon_selector).unwrap();
            let label = cx.debug_bounds(label_selector).unwrap();
            assert_eq!(icon.left() < label.left(), direction == LayoutDirection::Ltr);
        }
    }
}
