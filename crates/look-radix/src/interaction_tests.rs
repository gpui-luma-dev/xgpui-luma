//! Headless input regression coverage for Radix collection adapters.
use gpui::{Context, IntoElement, Render, TestAppContext, Window, div, prelude::*, px};
use xgpui_luma::controls::control_group::{ControlGroup, ControlGroupItem};
use xgpui_luma::controls::radio_group::RadioGroup as SdkRadioGroup;
use xgpui_luma::controls::listbox::{
    ListBoxControl, ListBoxInput, ListBoxSnapshot, ListBoxState, SelectionMode, make_listbox_item_template,
};
use super::*;

struct Harness {
    checkboxes: ControlGroup<ControlGroupItem>,
    radios: SdkRadioGroup<ControlGroupItem>,
    list: ListBoxControl<Self, u32, u32>,
    look: Look,
}
impl Harness {
    fn new(cx: &mut Context<Self>) -> Self {
        xgpui_luma::focus::bind_default_focus_keys(cx);
        xgpui_luma::key_handling::bind_default_control_keys(cx);
        let look = Look::built_in();
        let items = || {
            [
                ControlGroupItem::new("a").label("A"),
                ControlGroupItem::new("b").label("B"),
                ControlGroupItem::new("c").label("C").enabled(false),
            ]
        };
        let checkboxes = CheckboxGroup::new("checks").look(&look).items(items()).spawn(cx);
        let radios = RadioGroup::new("radios").look(&look).items(items()).selected("a").spawn(cx);
        let snapshot = ListBoxSnapshot::try_with_enabled([1, 2, 3], |n| *n, |n| *n != 3).unwrap();
        let list = ListBoxControl::new(
            ListBoxState::from_snapshot(snapshot, SelectionMode::Multiple),
            Self::input,
            |key| format!("list-item-{key}").into(),
            |n| format!("Item {n}").into(),
            cx,
        );
        Self { checkboxes, radios, list, look }
    }
    fn input(&mut self, input: ListBoxInput<u32>, _: &mut Window, cx: &mut Context<Self>) {
        let update = self.list.state.apply(input);
        self.list.handle_update(&update, cx);
        cx.notify();
    }
}
impl Render for Harness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let list = ListBoxBuilder::new(
            "list",
            &mut self.list,
            make_listbox_item_template(|model, _| div().child(format!("Item {}", model.item))),
        )
        .look(&self.look)
        .build(window, cx);
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_4()
            .child(self.checkboxes.clone())
            .child(self.radios.clone())
            .child(list)
    }
}
#[test]
fn groups_and_list_preserve_selection_and_ignore_disabled_items() {
    let mut app = TestAppContext::single();
    let (view, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        Harness::new(cx)
    });
    cx.simulate_resize(gpui::size(px(600.0), px(600.0)));
    cx.run_until_parked();
    let click = |cx: &mut gpui::VisualTestContext, id: &'static str| {
        let bounds = cx.debug_bounds(id).unwrap_or_else(|| panic!("missing {id}"));
        cx.simulate_click(bounds.center(), Default::default());
        cx.run_until_parked();
    };
    click(cx, "checks-a");
    click(cx, "checks-b");
    click(cx, "checks-c");
    cx.update(|_, app| {
        assert_eq!(view.read(app).checkboxes.read(app).selected_ids(), &[gpui::SharedString::from("a"), "b".into()])
    });
    click(cx, "checks-a");
    cx.update(|_, app| {
        assert_eq!(view.read(app).checkboxes.read(app).selected_ids(), &[gpui::SharedString::from("b")])
    });
    cx.simulate_keystrokes("space");
    cx.run_until_parked();
    cx.update(|_, app| {
        assert_eq!(view.read(app).checkboxes.read(app).selected_ids(), &[gpui::SharedString::from("b"), "a".into()])
    });
    click(cx, "radios-b");
    click(cx, "radios-c");
    cx.update(|_, app| assert_eq!(view.read(app).radios.read(app).selected_id(app), Some("b".into())));
    cx.simulate_keystrokes("up space");
    cx.run_until_parked();
    cx.update(|_, app| assert_eq!(view.read(app).radios.read(app).selected_id(app), Some("a".into())));
    click(cx, "list-item-0");
    click(cx, "list-item-1");
    click(cx, "list-item-2");
    cx.update(|_, app| assert_eq!(view.read(app).list.state.selected_keys().copied().collect::<Vec<_>>(), vec![1, 2]));
}
