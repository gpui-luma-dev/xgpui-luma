use super::*;
use std::{cell::Cell, rc::Rc};
use gpui::{TestAppContext, prelude::*};
use crate::{
    controls::{
        radio_group::{
            self, RadioGroupItem, RadioGroupLayout, radio_group_buttons_template,
            radio_group_button_item_element_template,
        },
        radio_button::default_radio_button_template,
    },
    infra::locale::{LayoutDirection, LocaleContext, set_locale_context},
};

#[test]
fn horizontal_radio_groups_follow_visual_arrows_and_skip_disabled_options() {
    for themed_rows in [false, true] {
        let mut app = TestAppContext::single();
        let (group, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            crate::key_handling::bind_default_control_keys(cx);
            let mut builder = radio_group::new("rtl-radios")
                .horizontal()
                .active_descendant()
                .selection_follows_active(true)
                .items([
                    RadioGroupItem::new("first"),
                    RadioGroupItem::new("disabled").enabled(false),
                    RadioGroupItem::new("middle"),
                    RadioGroupItem::new("last"),
                ])
                .selected("middle")
                .active("middle")
                .motion_enabled(false)
                .with_item_template(|item, _, _| {
                    let index = item.index;
                    div().debug_selector(move || format!("radio-label-{index}")).child(item.item.label().clone())
                });
            if themed_rows {
                builder = builder.template(radio_group_buttons_template(
                    default_radio_button_template(),
                    RadioGroupLayout::Horizontal,
                ));
            } else {
                builder = builder
                    .item_element_template(radio_group_button_item_element_template(default_radio_button_template()));
            }
            RadioGroupControl::from_builder(builder, cx)
        });
        cx.run_until_parked();
        cx.update(|window, cx| group.read(cx).focus_handle(cx).focus(window, cx));
        let changes = Rc::new(Cell::new(0));
        let observed = changes.clone();
        let _sub = cx.update(|_, cx| {
            cx.subscribe(&group, move |_, event: &RadioGroupEvent, _| {
                if matches!(event, RadioGroupEvent::Change { .. }) {
                    observed.set(observed.get() + 1);
                }
            })
        });
        for direction in [LayoutDirection::Ltr, LayoutDirection::Rtl, LayoutDirection::Ltr] {
            group.update(cx, |view, cx| {
                view.set_selected("middle", cx);
                view.set_active("middle", cx);
            });
            let before = changes.get();
            cx.update(|_, cx| set_locale_context(LocaleContext { direction, ..Default::default() }, cx));
            cx.run_until_parked();
            let first = cx.debug_bounds("radio-label-0").unwrap();
            let last = cx.debug_bounds("radio-label-3").unwrap();
            assert_eq!(first.left() < last.left(), direction == LayoutDirection::Ltr);
            assert_eq!(changes.get(), before);
            cx.update(|window, cx| {
                assert_eq!(group.read(cx).selected_id(cx).unwrap(), "middle");
                assert!(group.read(cx).focus_handle(cx).is_focused(window));
            });
            cx.simulate_keystrokes("right");
            cx.run_until_parked();
            cx.update(|_, cx| {
                assert_eq!(
                    group.read(cx).selected_id(cx).unwrap(),
                    if direction == LayoutDirection::Rtl {
                        "first"
                    } else {
                        "last"
                    }
                )
            });
            cx.simulate_keystrokes("left");
            cx.run_until_parked();
            cx.update(|_, cx| assert_eq!(group.read(cx).selected_id(cx).unwrap(), "middle"));
        }
    }
}

#[test]
fn vertical_radio_navigation_keeps_down_and_up_semantics_in_rtl() {
    let mut app = TestAppContext::single();
    app.update(|cx| set_locale_context(LocaleContext { direction: LayoutDirection::Rtl, ..Default::default() }, cx));
    let (group, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        crate::key_handling::bind_default_control_keys(cx);
        RadioGroupControl::from_builder(
            radio_group::new("vertical-radios")
                .vertical()
                .active_descendant()
                .selection_follows_active(true)
                .items([
                    RadioGroupItem::new("first"),
                    RadioGroupItem::new("disabled").enabled(false),
                    RadioGroupItem::new("last"),
                ])
                .selected("first")
                .active("first")
                .motion_enabled(false),
            cx,
        )
    });
    cx.run_until_parked();
    cx.update(|window, cx| group.read(cx).focus_handle(cx).focus(window, cx));
    for (key, expected) in [("down", "last"), ("up", "first"), ("end", "last"), ("home", "first")] {
        cx.simulate_keystrokes(key);
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(group.read(cx).selected_id(cx).unwrap(), expected));
    }
}
