use super::*;
use std::{cell::Cell, rc::Rc};
use gpui::TestAppContext;
use crate::{
    controls::{button::ButtonBuilder, textfield::TextFieldBuilder},
    infra::{
        locale::{LayoutDirection, LocaleContext, set_locale_context},
        presenter::HasPresenter,
    },
};

#[test]
fn toolbar_tracks_visual_direction_but_hosted_editor_keeps_horizontal_keys() {
    let mut app = TestAppContext::single();
    let (toolbar, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        crate::key_handling::bind_default_control_keys(cx);
        let first = ButtonBuilder::new("first-button")
            .label("First")
            .with_template_modifier(|root, _| root.debug_selector(|| "toolbar-first".into()))
            .spawn(cx);
        let disabled = ButtonBuilder::new("disabled-button").label("Disabled").enabled(false).spawn(cx);
        let middle = ButtonBuilder::new("middle-button")
            .label("Middle")
            .with_template_modifier(|root, _| root.debug_selector(|| "toolbar-middle".into()))
            .spawn(cx);
        let field = TextFieldBuilder::new("editor").value("abc").spawn(cx);
        let last = ButtonBuilder::new("last-button")
            .label("Last")
            .with_template_modifier(|root, _| root.debug_selector(|| "toolbar-last".into()))
            .spawn(cx);
        ToolbarControl::from_builder(
            ToolbarBuilder::new("rtl-toolbar").items([
                ToolbarItem::command_button("first", first, cx),
                ToolbarItem::command_button("disabled", disabled, cx).enabled(false),
                ToolbarItem::separator("separator"),
                ToolbarItem::command_button("middle", middle, cx),
                ToolbarItem::textfield_control("editor", field, cx),
                ToolbarItem::command_button("last", last, cx),
            ]),
            cx,
        )
    });
    cx.run_until_parked();
    let (middle, field) = cx.update(|_, cx| {
        let view = toolbar.read(cx);
        let ToolbarItemSource::CommandButton(middle) = view.items[3].event_source.as_ref().unwrap() else {
            panic!("button")
        };
        let ToolbarItemSource::TextField(field) = view.items[4].event_source.as_ref().unwrap() else {
            panic!("field")
        };
        (middle.clone(), field.clone())
    });
    let events = Rc::new(Cell::new(0));
    let observed = events.clone();
    let _sub =
        cx.update(|_, cx| cx.subscribe(&toolbar, move |_, _: &ToolbarEvent, _| observed.set(observed.get() + 1)));
    for direction in [LayoutDirection::Ltr, LayoutDirection::Rtl, LayoutDirection::Ltr] {
        cx.update(|window, cx| middle.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        cx.update(|_, cx| set_locale_context(LocaleContext { direction, ..Default::default() }, cx));
        cx.run_until_parked();
        let first = cx.debug_bounds("toolbar-first").unwrap();
        let last = cx.debug_bounds("toolbar-last").unwrap();
        assert_eq!(first.left() < last.left(), direction == LayoutDirection::Ltr);
        cx.update(|window, cx| {
            assert!(middle.read(cx).focus_handle(cx).is_focused(window));
            assert_eq!(field.read(cx).value(), "abc");
        });
        assert_eq!(events.get(), 0);
        cx.simulate_keystrokes("left");
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert_eq!(
                toolbar.read(cx).active_id(cx).as_deref(),
                Some(if direction == LayoutDirection::Rtl {
                    "editor"
                } else {
                    "first"
                })
            )
        });
        cx.update(|window, cx| field.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("right");
        cx.run_until_parked();
        cx.simulate_input("X");
        cx.run_until_parked();
        cx.update(|window, cx| {
            assert!(field.read(cx).focus_handle(cx).is_focused(window));
            assert_eq!(field.read(cx).value(), "abcX");
        });
        field.update(cx, |field, cx| field.set_value("abc", cx));
        cx.run_until_parked();
        events.set(0);
    }
}
