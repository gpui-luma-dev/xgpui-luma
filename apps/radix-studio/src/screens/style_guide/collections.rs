//! Live Radix listbox, checkbox-group, and non-segmented radio-group examples.
use xgpui_luma::controls::radio_group::RadioGroupEventExt;
use xgpui_luma::controls::control_group::ControlGroupEventExt;
use gpui::{AnyElement, Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::control_group::{ControlGroup, ControlGroupEvent, ControlGroupItem};
use xgpui_luma::controls::radio_group::{RadioGroup, RadioGroupEvent};
use xgpui_luma::controls::listbox::{ListBoxControl, ListBoxInput, ListBoxSnapshot, ListBoxState, SelectionMode};
use xgpui_luma::vstack;
use xgpui_luma_look_radix::{self as radix, Look, SemanticRole};
use super::super::section::section;

pub(crate) struct Collections {
    look: Look,
    checkboxes: Vec<(&'static str, ControlGroup<ControlGroupItem>)>,
    radios: Vec<(&'static str, RadioGroup<ControlGroupItem>)>,
    lists: Vec<(&'static str, ListBoxControl<Self, u32, u32>)>,
}
impl Collections {
    pub(crate) fn new(look: &Look, cx: &mut Context<Self>) -> Self {
        let items = || {
            [
                ControlGroupItem::new("email").label("Email"),
                ControlGroupItem::new("sms").label("SMS"),
                ControlGroupItem::new("push").label("Push (disabled)").enabled(false),
            ]
        };

        let mut checkboxes = Vec::new();
        let mut radios = Vec::new();
        for (label, checkbox_variant, radio_variant) in [
            ("Surface", radix::CheckboxVariant::Surface, radix::RadioVariant::Surface),
            ("Classic", radix::CheckboxVariant::Classic, radix::RadioVariant::Classic),
            ("Soft", radix::CheckboxVariant::Soft, radix::RadioVariant::Soft),
        ] {
            let checks = radix::CheckboxGroup::new(format!("guide-checkbox-group-{label}"))
                .look(look)
                .variant(checkbox_variant)
                .items(items())
                .selected_ids(["email"])
                .spawn(cx);
            checks.clone().on_event(cx, |_, _: &ControlGroupEvent, cx| cx.notify());
            checkboxes.push((label, checks));
            let group = radix::RadioGroup::new(format!("guide-radio-group-{label}"))
                .look(look)
                .variant(radio_variant)
                .items(items())
                .selected("email")
                .spawn(cx);
            group.clone().on_event(cx, |_, _: &RadioGroupEvent, cx| cx.notify());
            radios.push((label, group));
        }
        let mut lists = Vec::new();
        for (label, mode) in
            [("Single selection", SelectionMode::SingleRequired), ("Multiple selection", SelectionMode::Multiple)]
        {
            match ListBoxSnapshot::try_with_enabled(1..=8, |n| *n, |n| *n != 3) {
                Ok(snapshot) => lists.push((
                    label,
                    ListBoxControl::new(
                        ListBoxState::from_snapshot(snapshot, mode),
                        if mode == SelectionMode::Multiple {
                            Self::multiple_input
                        } else {
                            Self::single_input
                        },
                        |key| format!("guide-list-item-{key}").into(),
                        |n| format!("Item {n}").into(),
                        cx,
                    ),
                )),
                Err(error) => eprintln!("failed to build listbox example: {error}"),
            }
        }
        Self { look: look.clone(), checkboxes, radios, lists }
    }
    fn input(&mut self, index: usize, input: ListBoxInput<u32>, cx: &mut Context<Self>) {
        if let Some((_, list)) = self.lists.get_mut(index) {
            let update = list.state.apply(input);
            list.handle_update(&update, cx);
            cx.notify();
        }
    }
    fn single_input(&mut self, input: ListBoxInput<u32>, _: &mut Window, cx: &mut Context<Self>) {
        self.input(0, input, cx);
    }
    fn multiple_input(&mut self, input: ListBoxInput<u32>, _: &mut Window, cx: &mut Context<Self>) {
        self.input(1, input, cx);
    }
}
#[derive(Clone, Copy)]
enum Kind {
    Checkboxes,
    Radios,
    Lists,
}

/// Separate views share the existing control state so sections can be ordered by name.
#[derive(Clone)]
pub(crate) struct Previews {
    pub checkboxes: Entity<CollectionSection>,
    pub radios: Entity<CollectionSection>,
    pub lists: Entity<CollectionSection>,
}
impl Previews {
    pub fn new<M: 'static>(look: &Look, cx: &mut Context<M>) -> Self {
        let controls = cx.new(|cx| Collections::new(look, cx));
        let mut view = |kind| {
            cx.new(|cx| {
                cx.observe(&controls, |_, _, cx| cx.notify()).detach();
                CollectionSection { controls: controls.clone(), kind }
            })
        };
        Self { checkboxes: view(Kind::Checkboxes), radios: view(Kind::Radios), lists: view(Kind::Lists) }
    }
}
pub(crate) struct CollectionSection {
    controls: Entity<Collections>,
    kind: Kind,
}
impl Render for CollectionSection {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.controls.update(cx, |controls, cx| controls.render_section(self.kind, window, cx))
    }
}
impl Collections {
    fn render_section(&mut self, kind: Kind, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let fg = self.look.resolve_role(SemanticRole::Foreground).hsla();
        let muted = self.look.resolve_role(SemanticRole::MutedForeground).hsla();
        let border = self.look.resolve_role(SemanticRole::Border).hsla();
        match kind {
            Kind::Checkboxes => {
                let checks = div()
                    .flex()
                    .flex_wrap()
                    .gap(px(24.0))
                    .children(self.checkboxes.iter().map(|(label, group)| {
                        let selected =
                            group.read(cx).selected_ids().iter().map(|id| id.as_ref()).collect::<Vec<_>>().join(", ");
                        vstack! { gap=8;
                            div().text_sm().text_color(muted).child(*label), group.clone(),
                            div().text_xs().text_color(muted).child(format!("Selected: {selected}")),
                        }
                        .w(px(200.0))
                    }))
                    .into_any_element();
                section(
                    "Checkbox Group",
                    "Toggle each option independently. Space toggles the active option; arrows move between options.",
                    fg,
                    muted,
                    border,
                    checks,
                )
            }
            Kind::Radios => {
                let radios = div()
                    .flex()
                    .flex_wrap()
                    .gap(px(24.0))
                    .children(self.radios.iter().map(|(label, group)| {
                        let selected = group.read(cx).selected_id(cx).map(|id| id.to_string()).unwrap_or_default();
                        vstack! { gap=8;
                            div().text_sm().text_color(muted).child(*label), group.clone(),
                            div().text_xs().text_color(muted).child(format!("Selected: {selected}")),
                        }
                        .w(px(200.0))
                    }))
                    .into_any_element();
                section(
                    "Radio Group",
                    "Standard radio indicators with a single selection. Use arrows and Space to choose an option.",
                    fg,
                    muted,
                    border,
                    radios,
                )
            }
            Kind::Lists => {
                let mut lists = div().flex().flex_wrap().gap(px(24.0));
                for (label, control) in &mut self.lists {
                    let id = if *label == "Single selection" {
                        "guide-listbox-single"
                    } else {
                        "guide-listbox-multiple"
                    };
                    let list = radix::ListBoxBuilder::new(
                        id,
                        control,
                        radix::listbox_template(|n: &u32| {
                            if *n == 3 {
                                "Item 3 (disabled)".into()
                            } else {
                                format!("Item {n}").into()
                            }
                        }),
                    )
                    .look(&self.look)
                    .build(window, cx);
                    lists = lists
                        .child(vstack! { gap=8; div().text_sm().text_color(muted).child(*label), list }.w(px(280.0)));
                }
                section(
                    "Listbox",
                    "Single and multiple selection with keyboard navigation and scrolling. Item 3 is disabled.",
                    fg,
                    muted,
                    border,
                    lists.into_any_element(),
                )
            }
        }
    }
}
#[cfg(test)]
impl Render for Collections {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(px(28.0))
            .children([Kind::Checkboxes, Kind::Radios, Kind::Lists].map(|kind| self.render_section(kind, window, cx)))
            .w_full()
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{TestAppContext, VisualTestContext};
    #[test]
    fn examples_select_independently_and_keep_disabled_items_unselected() {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            xgpui_luma::focus::bind_default_focus_keys(cx);
            xgpui_luma::key_handling::bind_default_control_keys(cx);
            Collections::new(&Look::built_in(), cx)
        });
        cx.simulate_resize(gpui::size(px(900.0), px(1200.0)));
        cx.run_until_parked();
        let click = |cx: &mut VisualTestContext, selector: &'static str| {
            let bounds = cx.debug_bounds(selector).unwrap_or_else(|| panic!("missing {selector}"));
            cx.simulate_click(bounds.center(), Default::default());
            cx.run_until_parked();
        };
        click(cx, "guide-checkbox-group-Surface-sms");
        click(cx, "guide-checkbox-group-Surface-push");
        cx.update(|_, app| {
            assert_eq!(
                view.read(app).checkboxes[0].1.read(app).selected_ids(),
                &[gpui::SharedString::from("email"), "sms".into()]
            )
        });
        click(cx, "guide-radio-group-Surface-sms");
        cx.simulate_keystrokes("up");
        cx.run_until_parked();
        cx.update(|_, app| assert_eq!(view.read(app).radios[0].1.read(app).selected_id(app), Some("email".into())));
        click(cx, "guide-listbox-single-item-1");
        click(cx, "guide-listbox-multiple-item-0");
        click(cx, "guide-listbox-multiple-item-1");
        click(cx, "guide-listbox-multiple-item-2");
        cx.update(|_, app| {
            assert_eq!(view.read(app).lists[0].1.state.selected_key(), Some(&2));
            assert_eq!(view.read(app).lists[1].1.state.selected_keys().copied().collect::<Vec<_>>(), vec![1, 2]);
        });
    }
}
