use super::*;
use std::{cell::Cell, rc::Rc};
use gpui::TestAppContext;
use crate::infra::{
    locale::{LayoutDirection, LocaleContext, set_locale_context},
    menu_item::MenuItem,
};

#[test]
fn split_trigger_moves_disclosure_to_logical_end_and_preserves_activation_targets() {
    let mut app = TestAppContext::single();
    let (menu, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        PopupMenu::from_builder(
            PopupMenuBuilder::new("rtl-menu")
                .label("Save")
                .split()
                .items([MenuItem::new("save-as")])
                .motion_enabled(false),
            cx,
        )
    });
    cx.run_until_parked();
    cx.update(|window, cx| menu.read(cx).focus_handle(cx).focus(window, cx));
    let actions = Rc::new(Cell::new(0));
    let observed = actions.clone();
    let _sub = cx.update(|_, cx| {
        cx.subscribe(&menu, move |_, event: &PopupMenuEvent, _| {
            if matches!(event, PopupMenuEvent::ActionClick) {
                observed.set(observed.get() + 1);
            }
        })
    });
    for direction in [LayoutDirection::Ltr, LayoutDirection::Rtl, LayoutDirection::Ltr, LayoutDirection::Rtl] {
        cx.update(|_, cx| set_locale_context(LocaleContext { direction, ..Default::default() }, cx));
        cx.run_until_parked();
        let primary = cx.debug_bounds("rtl-menu/primary").unwrap();
        let disclosure = cx.debug_bounds("rtl-menu/disclosure").unwrap();
        assert_eq!(primary.left() < disclosure.left(), direction == LayoutDirection::Ltr);
        cx.update(|window, cx| {
            assert!(!menu.read(cx).lifecycle.is_open());
            assert!(menu.read(cx).focus_handle(cx).is_focused(window));
        });
        assert_eq!(actions.get(), 0);
    }
    let primary = cx.debug_bounds("rtl-menu/primary").unwrap();
    cx.simulate_click(primary.center(), Default::default());
    cx.run_until_parked();
    assert_eq!(actions.get(), 1);
    cx.update(|_, cx| assert!(!menu.read(cx).lifecycle.is_open()));
    let disclosure = cx.debug_bounds("rtl-menu/disclosure").unwrap();
    cx.simulate_click(disclosure.center(), Default::default());
    cx.run_until_parked();
    cx.update(|_, cx| assert!(menu.read(cx).lifecycle.is_open()));
    assert_eq!(actions.get(), 1);
}

#[test]
fn popup_rows_and_submenus_follow_direction_and_physical_arrows() {
    use lucide_svg_static::Icon;
    let mut app = TestAppContext::single();
    let (menu, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        crate::key_handling::bind_default_control_keys(cx);
        PopupMenu::from_builder(
            PopupMenuBuilder::new("rtl-submenu")
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
        view.open_menu_with(Some(MenuPath::Root(0)), cx);
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
            assert!(menu.read(cx).lifecycle.is_open());
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
        cx.simulate_keystrokes(if direction == LayoutDirection::Rtl {
            "right"
        } else {
            "left"
        });
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(menu.read(cx).menu_state.active_path(), Some(MenuPath::Root(0))));
    }
}
