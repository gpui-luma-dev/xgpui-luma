//! #266: every horizontal group mirrors visually in RTL, so arrows must follow.
use gpui::{Focusable, TestAppContext};
use crate::{
    controls::{
        control_group::{
            self, ControlGroupBuilder, ControlGroupControl, ControlGroupItem, default_control_group_theme,
        },
        icon_group,
    },
    infra::locale::{LayoutDirection, LocaleContext, set_locale_context},
};

fn items() -> [ControlGroupItem; 3] {
    [ControlGroupItem::new("first"), ControlGroupItem::new("middle"), ControlGroupItem::new("last")]
}

#[test]
fn horizontal_control_and_icon_groups_follow_visual_arrows_in_rtl() {
    let builders: [fn() -> ControlGroupBuilder<ControlGroupItem>; 2] = [
        || control_group::new("rtl-control-group").horizontal(),
        || icon_group::icon_toolbar("rtl-icon-group", default_control_group_theme()),
    ];
    for builder in builders {
        let mut app = TestAppContext::single();
        let (group, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            crate::key_handling::bind_default_control_keys(cx);
            ControlGroupControl::from_builder(
                builder().active_descendant().items(items()).active("middle").motion_enabled(false),
                cx,
            )
        });
        cx.run_until_parked();
        cx.update(|window, cx| group.read(cx).focus_handle(cx).focus(window, cx));
        for direction in [LayoutDirection::Ltr, LayoutDirection::Rtl, LayoutDirection::Ltr] {
            cx.update(|_, cx| set_locale_context(LocaleContext { direction, ..Default::default() }, cx));
            group.update(cx, |view, cx| view.set_active("middle", cx));
            cx.run_until_parked();
            cx.simulate_keystrokes("right");
            cx.run_until_parked();
            let expected = if direction == LayoutDirection::Rtl {
                "first"
            } else {
                "last"
            };
            cx.update(|_, cx| assert_eq!(group.read(cx).active_id().unwrap(), expected, "{direction:?}"));
        }
    }
}
