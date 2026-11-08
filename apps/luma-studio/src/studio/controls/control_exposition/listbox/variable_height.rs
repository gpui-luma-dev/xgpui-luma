//! The same content-sized template in eager and measured-virtualized lists.
use xgpui_luma::controls::selector::SelectorEventExt;
use xgpui_luma::controls::checkbox::CheckboxEventExt;
use std::{cell::Cell, sync::Arc};
use gpui::{Context, Div, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::{
    checkbox::{Checkbox},
    button::{Button},
    selector::{Selector, SelectorEvent, SelectorItem},
    listbox::{
        ListBoxControl, ListBoxInput, ListBoxItemRenderModel, ListBoxState, ListBoxVirtualization,
        ListBoxVirtualWindow, SelectionMode, SelectionPolicy,
    },
};
use xgpui_luma::{hstack, vstack};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma_look_shadcn::{self as shadcn, LumaTypographyExt, ShadcnLook, ShadcnTextSize};
use xgpui_luma_look_shadcn::prelude::*;
use super::{
    markup::listbox,
    presentation::{ExamplePresentation, SelectionMark},
};
use super::super::event_stream::ControlEventStream;

const ITEM_COUNT: u32 = 1_000;

struct Note {
    id: u32,
    title: String,
    description: &'static str,
}

impl Note {
    fn new(id: u32) -> Self {
        let description = match id % 3 {
            0 => "Ready for review.",
            1 => {
                "The description wraps naturally at the available width. The template chooses its content; the list measures the resulting height."
            }
            _ => {
                "A longer note includes context, progress, and the next step. Narrow the layout or expand the details to see the same item reflow. Its identity and selection remain stable while its height changes."
            }
        };
        Self { id, title: format!("Note {id}"), description }
    }
}

fn note_template(model: &ListBoxItemRenderModel<'_, Note>, look: &ShadcnLook, expanded: bool) -> Div {
    vstack! { gap=6.0;
        hstack! { gap=6.0 align=center;
            SelectionMark { selected: model.selected },
            div().typography_style(look.typography_scale(ShadcnTextSize::Sm)).child(model.item.title.clone()),
        },
        div().typography_style(look.typography_scale(ShadcnTextSize::Xs))
            .text_color(look.chrome().muted_text).child(model.item.description),
    }.w_full().px(px(8.0)).py(px(8.0)).when(expanded, |row| {
        row.child(div().typography_style(look.typography_scale(ShadcnTextSize::Xs))
            .child("Additional details belong to this item. Expanding them changes its measured height without changing the collection or selection."))
    })
}

struct NotesList {
    id: &'static str,
    presentation: ExamplePresentation,
    list: ListBoxControl<Self, Note, u32>,
    expanded: bool,
    width: f32,
    stats: Option<ListBoxVirtualWindow>,
    template_calls: usize,
}

impl NotesList {
    pub(super) fn captured_motion(&self) -> (bool, xgpui_luma::controls::listbox::ListBoxMotion) {
        (self.list.is_motion_enabled(), self.list.motion_profile())
    }

    fn new(
        id: &'static str,
        title: &'static str,
        policy: ListBoxVirtualization,
        look: Arc<ShadcnLook>,
        events: Entity<ControlEventStream>,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut state = ListBoxState::try_new((1..=ITEM_COUNT).map(Note::new), |item| item.id, SelectionMode::Extended)
            .expect("sample note keys are unique");
        state.set_selection_policy(super::selection_controls::DEFAULT_POLICY);
        Self {
            id,
            presentation: ExamplePresentation::new(look, title, events),
            list: ListBoxControl::new(
                state,
                Self::input,
                |key| ("note", *key).into(),
                |item| item.title.clone().into(),
                cx,
            )
            .require_focus_for_scroll(true)
            .virtualization(policy),
            expanded: false,
            width: 250.0,
            stats: None,
            template_calls: 0,
        }
    }

    fn input(&mut self, input: ListBoxInput<u32>, _: &mut Window, cx: &mut Context<Self>) {
        let update = self.list.apply(input, cx);
        self.presentation.record(&update.events, cx);
    }

    fn set_selection_policy(&mut self, policy: SelectionPolicy, cx: &mut Context<Self>) {
        let update = self.list.set_selection_policy(policy, cx);
        self.presentation.record(&update.events, cx);
    }

    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.presentation.sync_look(look, cx);
        self.list.invalidate_measurements(None, cx);
    }
}

impl Render for NotesList {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let look = &self.presentation.look;
        let calls = Cell::new(0);
        let expanded = self.expanded;
        let surface = listbox! { window, cx;
            id = self.id;
            control = &mut self.list;
            look = look;
            aria_label = "Content-sized notes";
            width = self.width;
            padding_x = 8.0;
            padding_y = 8.0;
            scroll_view! { vertical;
                viewport_height = 320.0;
                vstack! {
                    gap = 4.0;
                    item_height = content;
                    item_template = |model, _cx| {
                        calls.set(calls.get() + 1);
                        note_template(model, look, expanded)
                    };
                }
            }
        };
        self.stats = self.list.render_parts().scroll.rendered_window();
        self.template_calls = calls.get();
        let mut stats = vstack! { gap=2.0; }
            .typography_style(look.typography_scale(ShadcnTextSize::Xs))
            .text_color(look.chrome().muted_text)
            .child(format!("{ITEM_COUNT} items · {} template calls", self.template_calls));
        if let Some(window) = &self.stats {
            let range = &window.visible_range;
            stats = stats.child(if range.is_empty() {
                "Visible: —".to_owned()
            } else {
                format!("Visible: {}–{} · Constructed: {}", range.start + 1, range.end, window.range.len())
            });
            let measured = window.measured_items.unwrap_or(0);
            stats = stats.child(format!("Measured: {measured} · Estimated: {}", ITEM_COUNT as usize - measured));
        }
        vstack! { gap=8.0;
            self.presentation.section(self.list.state.selected_keys().count(), surface),
            stats,
        }
        .w(px(self.width))
        .flex_shrink_0()
    }
}

pub(super) struct VariableHeightExample {
    look: Arc<ShadcnLook>,
    lists: [Entity<NotesList>; 2],
    expanded: Checkbox,
    narrow: Checkbox,
    smooth: Checkbox,
    smooth_enabled: bool,
    target: Entity<Selector>,
    target_key: u32,
    make_visible: Entity<Button>,
    center_item: Entity<Button>,
}

impl VariableHeightExample {
    pub(super) fn captured_motions(&self, cx: &gpui::App) -> [(bool, xgpui_luma::controls::listbox::ListBoxMotion); 2] {
        [self.lists[0].read(cx).captured_motion(), self.lists[1].read(cx).captured_motion()]
    }

    pub fn new(look: Arc<ShadcnLook>, events: Entity<ControlEventStream>, cx: &mut Context<Self>) -> Self {
        let eager = cx.new(|cx| {
            NotesList::new(
                "notes-eager",
                "Content · eager",
                ListBoxVirtualization::Eager,
                look.clone(),
                events.clone(),
                cx,
            )
        });
        let measured = cx.new(|cx| {
            NotesList::new(
                "notes-measured",
                "Content · virtualized",
                ListBoxVirtualization::Measured { estimated_height: 100.0, overscan: 2 },
                look.clone(),
                events,
                cx,
            )
        });
        let expanded = shadcn::Checkbox::new("notes-expanded")
            .look(&look)
            .content(|_, _| div().child("Expand details").into_any_element())
            .spawn(cx);
        let narrow = shadcn::Checkbox::new("notes-narrow")
            .look(&look)
            .content(|_, _| div().child("Narrow layout").into_any_element())
            .spawn(cx);
        let smooth = shadcn::Checkbox::new("notes-smooth-scroll")
            .look(&look)
            .content(|_, _| div().child("Smooth scrolling").into_any_element())
            .spawn(cx);
        let target = shadcn::Selector::new("notes-scroll-target")
            .look(&look)
            .label("Target item")
            .items(
                [1, 7, 50, 500, ITEM_COUNT].map(|key| SelectorItem::new(key.to_string()).label(format!("Note {key}"))),
            )
            .selected_id("7")
            .spawn(cx);
        let make_visible =
            shadcn::Button::new("notes-make-visible").look(&look).outline().label("Make visible").spawn(cx);
        let center_item =
            shadcn::Button::new("notes-center-item").look(&look).outline().label("Center in viewport").spawn(cx);

        smooth.clone().on_value_changed(cx, |this, checked, _| {
            this.smooth_enabled = checked;
        });
        target.clone().on_event(cx, |this, event, _| {
            if let SelectorEvent::Change { item_id, .. } = event
                && let Ok(key) = item_id.parse::<u32>()
            {
                this.target_key = key;
            }
        });
        make_visible.clone().on_click(cx, |this, cx| {
            for list in &this.lists {
                list.update(cx, |list, cx| {
                    if this.smooth_enabled {
                        list.list.scroll_to_smooth(this.target_key, cx);
                    } else {
                        list.list.scroll_to(this.target_key, cx);
                    }
                });
            }
        });
        center_item.clone().on_click(cx, |this, cx| {
            for list in &this.lists {
                list.update(cx, |list, cx| {
                    if this.smooth_enabled {
                        list.list.scroll_to_center_smooth(this.target_key, cx);
                    } else {
                        list.list.scroll_to_center(this.target_key, cx);
                    }
                });
            }
        });
        expanded.clone().on_value_changed(cx, |this, checked, cx| {
            for list in &this.lists {
                list.update(cx, |list, cx| {
                    list.expanded = checked;
                    list.list.invalidate_measurements(None, cx);
                });
            }
        });
        narrow.clone().on_value_changed(cx, |this, checked, cx| {
            for list in &this.lists {
                list.update(cx, |list, cx| {
                    list.width = if checked { 180.0 } else { 250.0 };
                    cx.notify();
                });
            }
        });
        Self {
            look,
            lists: [eager, measured],
            expanded,
            narrow,
            smooth,
            smooth_enabled: false,
            target,
            target_key: 7,
            make_visible,
            center_item,
        }
    }

    pub(super) fn set_selection_policy(&mut self, policy: SelectionPolicy, cx: &mut Context<Self>) {
        for list in &self.lists {
            list.update(cx, |list, cx| list.set_selection_policy(policy, cx));
        }
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for list in &self.lists {
            list.update(cx, |list, cx| list.sync_look(look.clone(), cx));
        }
        self.expanded.update(cx, |_, cx| cx.notify());
        self.narrow.update(cx, |_, cx| cx.notify());
        self.smooth.update(cx, |_, cx| cx.notify());
        self.target.update(cx, |_, cx| cx.notify());
        self.make_visible.update(cx, |_, cx| cx.notify());
        self.center_item.update(cx, |_, cx| cx.notify());
        cx.notify();
    }
}

impl Render for VariableHeightExample {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        with_look(&self.look, || {
            vstack! { gap=12.0;
                div().typography_style(self.look.typography_scale(ShadcnTextSize::Sm))
                    .text_color(self.look.chrome().title_text).child("Variable-height items"),
                hstack! { gap=8.0 align=center;
                    div().w(px(160.0)).child(self.target.clone()),
                    self.make_visible.clone(), self.center_item.clone(), self.smooth.clone(),
                }.flex_wrap(),
                div().typography_style(self.look.typography_scale(ShadcnTextSize::Xs)).text_color(self.look.chrome().muted_text)
                    .child("Choose a target, then position it in both lists. Selection stays unchanged. Centering near either end is limited by the collection boundary."),
                hstack! { gap=12.0; self.expanded.clone(), self.narrow.clone(), },
                hstack! { gap=16.0; self.lists[0].clone(), self.lists[1].clone(), }.flex_wrap(),
                div().typography_style(self.look.typography_scale(ShadcnTextSize::Xs)).text_color(self.look.chrome().muted_text)
                    .child("Identical content, independent selection and scrolling. Click a list to scroll; Home/End jumps to its ends. Visible includes partial rows; constructed includes the buffer."),
            }
        })
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use xgpui_luma::controls::checkbox::CheckboxEvent;
    use xgpui_luma::controls::button::ButtonEvent;
    use gpui::{TestAppContext, VisualTestContext};
    use super::*;

    fn settle(cx: &mut VisualTestContext) {
        for _ in 0..30 {
            cx.run_until_parked();
            if cx.update(|window, app| window.simulate_next_frame(app)) == 0 {
                return;
            }
        }
        panic!("content-sized exposition failed to settle");
    }

    #[test]
    fn variable_height_exposition_compares_the_same_template_and_reflows() {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| {
            let look = Arc::new(ShadcnLook::built_in());
            let events = cx.new(|cx| ControlEventStream::new(cx, look.clone(), "notes-events", ""));
            VariableHeightExample::new(look, events, cx)
        });
        settle(cx);
        let eager = cx.debug_bounds("notes-eager-item-0").unwrap();
        let measured = cx.debug_bounds("notes-measured-item-0").unwrap();
        assert_eq!(eager.size, measured.size);
        assert!(cx.debug_bounds("notes-measured-item-2").unwrap().size.height < measured.size.height);
        cx.update(|_, app| {
            let example = view.read(app);
            assert_eq!(example.lists[0].read(app).template_calls, ITEM_COUNT as usize);
            assert!(example.lists[1].read(app).template_calls < 15);
            assert_eq!(example.lists[0].read(app).stats.as_ref().unwrap().measured_items, Some(ITEM_COUNT as usize));
        });
        cx.update(|window, _| window.activate_window());
        for (index, row) in [eager, measured].into_iter().enumerate() {
            cx.simulate_click(row.center(), Default::default());
            settle(cx);
            cx.update(|_, app| assert_eq!(view.read(app).lists[index].read(app).list.state.selected_key(), Some(&1)));
            cx.simulate_click(row.center(), Default::default());
            settle(cx);
            cx.update(|_, app| assert_eq!(view.read(app).lists[index].read(app).list.state.selected_keys().count(), 0));
        }
        // Exercise the actual subscriptions shared by the two SDK checkboxes.
        cx.update(|_, app| {
            view.read(app)
                .expanded
                .clone()
                .update(app, |_, cx| cx.emit(CheckboxEvent::Change { checked: true }))
        });
        settle(cx);
        let expanded = cx.debug_bounds("notes-measured-item-0").unwrap();
        assert!(expanded.size.height > measured.size.height);
        assert_eq!(cx.debug_bounds("notes-eager-item-0").unwrap().size, expanded.size);
        cx.update(|_, app| {
            view.read(app).narrow.clone().update(app, |_, cx| cx.emit(CheckboxEvent::Change { checked: true }))
        });
        settle(cx);
        let narrow = cx.debug_bounds("notes-measured-item-0").unwrap();
        assert!(narrow.size.height > expanded.size.height);
        assert!(narrow.size.width < expanded.size.width);
        assert_eq!(cx.debug_bounds("notes-eager-item-0").unwrap().size, narrow.size);
    }
    #[test]
    fn positioning_fixture_buttons_target_both_lists_and_preserve_selection() {
        let mut app = TestAppContext::single();
        let (view, cx) = app.add_window_view(|_, cx| {
            let look = Arc::new(ShadcnLook::built_in());
            let events = cx.new(|cx| ControlEventStream::new(cx, look.clone(), "position-events", ""));
            VariableHeightExample::new(look, events, cx)
        });
        settle(cx);
        // Exercise the fixture subscriptions, including a target outside the
        // virtual window. The positioning buttons must not select that target.
        cx.update(|_, app| {
            for list in view.read(app).lists.clone() {
                list.update(app, |host, cx| {
                    let update = host.list.state.set_selected(Some(7)).unwrap();
                    host.list.handle_update(&update, cx);
                });
            }
            view.read(app).target.clone().update(app, |_, cx| {
                cx.emit(SelectorEvent::Change { item_id: "500".into(), label: "Note 500".into() })
            });
        });
        cx.run_until_parked();
        cx.update(|_, app| view.read(app).center_item.clone().update(app, |_, cx| cx.emit(ButtonEvent::Click)));
        settle(cx);
        for (row_id, viewport_id) in [
            ("notes-eager-item-499", "notes-eager-viewport"),
            ("notes-measured-item-499", "notes-measured-viewport"),
        ] {
            let row = cx.debug_bounds(row_id).unwrap();
            let viewport = cx.debug_bounds(viewport_id).unwrap();
            assert!(((row.top() + row.bottom() - viewport.top() - viewport.bottom()) / 2.0).abs() < px(0.1));
        }
        cx.update(|_, app| {
            view.read(app)
                .target
                .clone()
                .update(app, |_, cx| cx.emit(SelectorEvent::Change { item_id: "7".into(), label: "Note 7".into() }))
        });
        cx.run_until_parked();
        cx.update(|_, app| view.read(app).make_visible.clone().update(app, |_, cx| cx.emit(ButtonEvent::Click)));
        settle(cx);
        for (row_id, viewport_id) in
            [("notes-eager-item-6", "notes-eager-viewport"), ("notes-measured-item-6", "notes-measured-viewport")]
        {
            let row = cx.debug_bounds(row_id).unwrap();
            let viewport = cx.debug_bounds(viewport_id).unwrap();
            assert!(row.top() >= viewport.top() && row.bottom() <= viewport.bottom());
        }
        // Exercise the same fixture with shared smooth scrolling enabled.
        cx.update(|window, app| {
            window.activate_window();
            view.read(app).smooth.clone().update(app, |_, cx| cx.emit(CheckboxEvent::Change { checked: true }));
            view.read(app).target.clone().update(app, |_, cx| {
                cx.emit(SelectorEvent::Change { item_id: "500".into(), label: "Note 500".into() })
            });
        });
        cx.run_until_parked();
        cx.update(|_, app| view.read(app).center_item.clone().update(app, |_, cx| cx.emit(ButtonEvent::Click)));
        cx.run_until_parked();
        for _ in 0..32 {
            cx.executor().advance_clock(std::time::Duration::from_millis(16));
            cx.update(|window, app| window.simulate_next_frame(app));
            cx.run_until_parked();
        }
        for (row_id, viewport_id) in [
            ("notes-eager-item-499", "notes-eager-viewport"),
            ("notes-measured-item-499", "notes-measured-viewport"),
        ] {
            let row = cx.debug_bounds(row_id).unwrap();
            let viewport = cx.debug_bounds(viewport_id).unwrap();
            assert!(((row.top() + row.bottom() - viewport.top() - viewport.bottom()) / 2.0).abs() < px(0.1));
        }
        cx.update(|_, app| {
            for list in &view.read(app).lists {
                assert_eq!(list.read(app).list.state.selected_key(), Some(&7));
                assert!(!list.read(app).list.state.is_focused());
            }
        });
    }
}
