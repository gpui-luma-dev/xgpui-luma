use super::*;
use gpui::TestAppContext;
use crate::controls::context_menu::ContextMenuBuilder;
use crate::infra::{locale::set_locale_context, menu_item::MenuItem};

#[test]
fn context_rows_and_submenus_follow_direction_and_physical_arrows() {
    use lucide_svg_static::Icon;
    let mut app = TestAppContext::single();
    let (menu, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        crate::key_handling::bind_default_control_keys(cx);
        ContextMenu::from_builder(
            ContextMenuBuilder::new("rtl-submenu")
                .label("Menu")
                .items([
                    MenuItem::new("parent")
                        .icon(Icon::House)
                        .submenu([MenuItem::new("disabled").enabled(false), MenuItem::new("leaf").icon(Icon::Clock)]),
                    MenuItem::new("other"),
                ])
                .motion_enabled(false),
            cx,
        )
    });
    cx.run_until_parked();
    cx.update(|window, cx| menu.read(cx).focus_handle(cx).focus(window, cx));
    menu.update(cx, |view, cx| {
        view.open_menu_at(point(px(300.0), px(160.0)), Some(MenuPath::Root(0)), cx);
        cx.notify();
    });
    cx.run_until_parked();
    for direction in [LayoutDirection::Ltr, LayoutDirection::Rtl, LayoutDirection::Ltr] {
        cx.update(|_, cx| set_locale_context(LocaleContext { direction, ..Default::default() }, cx));
        cx.run_until_parked();
        let icon = cx.debug_bounds("floating-icon-parent").unwrap();
        let label = cx.debug_bounds("floating-label-parent").unwrap();
        assert_eq!(icon.left() < label.left(), direction == LayoutDirection::Ltr);
        cx.update(|window, cx| {
            assert_eq!(menu.read(cx).menu_position, Some(point(px(300.0), px(160.0))));
            assert_eq!(menu.read(cx).menu_state.active_path(), Some(MenuPath::Root(0)));
            assert!(menu.read(cx).focus_handle(cx).is_focused(window));
        });
        cx.simulate_keystrokes(if direction == LayoutDirection::Rtl {
            "left"
        } else {
            "right"
        });
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert_eq!(menu.read(cx).menu_state.active_path(), Some(MenuPath::Submenu { parent: 0, child: 1 }))
        });
        let root = cx.debug_bounds("floating-row-parent").unwrap();
        let submenu = cx.debug_bounds("floating-submenu-parent").unwrap();
        assert_eq!(submenu.left() < root.left(), direction == LayoutDirection::Rtl);
        let icon = cx.debug_bounds("floating-icon-leaf").unwrap();
        let label = cx.debug_bounds("floating-label-leaf").unwrap();
        assert_eq!(icon.left() < label.left(), direction == LayoutDirection::Ltr);
        // Repaint an already-open submenu without losing its selected child.
        for live_direction in [
            if direction == LayoutDirection::Rtl {
                LayoutDirection::Ltr
            } else {
                LayoutDirection::Rtl
            },
            direction,
        ] {
            cx.update(|_, cx| {
                set_locale_context(LocaleContext { direction: live_direction, ..Default::default() }, cx)
            });
            cx.run_until_parked();
            let root = cx.debug_bounds("floating-row-parent").unwrap();
            let submenu = cx.debug_bounds("floating-submenu-parent").unwrap();
            assert_eq!(submenu.left() < root.left(), live_direction == LayoutDirection::Rtl);
            cx.update(|window, cx| {
                assert_eq!(menu.read(cx).menu_position, Some(point(px(300.0), px(160.0))));
                assert_eq!(menu.read(cx).menu_state.active_path(), Some(MenuPath::Submenu { parent: 0, child: 1 }));
                assert!(menu.read(cx).focus_handle(cx).is_focused(window));
            });
        }
        cx.simulate_keystrokes(if direction == LayoutDirection::Rtl {
            "right"
        } else {
            "left"
        });
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(menu.read(cx).menu_state.active_path(), Some(MenuPath::Root(0))));
    }
}
