//! Independent vertical ListBox example with an inline content template.

use std::sync::Arc;

use gpui::{Context, Entity, IntoElement, Render, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::listbox::{ListBoxInput, ListBoxSnapshot, ListBoxState, SelectionMode, SelectionPolicy};
use xgpui_luma::hstack;
use xgpui_luma_look_shadcn::ShadcnLook;

use super::{ListBoxSampleLayout, VERTICAL_LIST_WIDTH};
use super::presentation::{ExamplePresentation, SelectionMark};
use super::markup::listbox;
use xgpui_luma::controls::listbox::{ListBoxControl, ListBoxVirtualization};
use super::super::event_stream::ControlEventStream;

const ID: &str = "listbox-vertical";
// Inspector mirror of the markup below; headless geometry tests guard against drift.
pub(in super::super) const LAYOUT: ListBoxSampleLayout = ListBoxSampleLayout {
    item_height: 36.0,
    spacing: 4.0,
    item_padding: 12.0,
    viewport_height: 196.0,
    inset_x: 16.0,
    inset_y: 8.0,
    radius: 8.0,
    border: 1.0,
};

struct RowItem {
    id: u32,
    label: SharedString,
}

pub(super) struct VerticalListExample {
    presentation: ExamplePresentation,
    list: ListBoxControl<Self, RowItem, u32>,
}

impl VerticalListExample {
    pub(super) fn captured_motion(&self) -> (bool, xgpui_luma::controls::listbox::ListBoxMotion) {
        (self.list.is_motion_enabled(), self.list.motion_profile())
    }

    pub(super) fn new(look: Arc<ShadcnLook>, event_stream: Entity<ControlEventStream>, cx: &mut Context<Self>) -> Self {
        let items = (1..=1_000).map(|id| RowItem { id, label: format!("Item {id}").into() });
        let snapshot = ListBoxSnapshot::try_with_enabled(items, |item| item.id, |item| item.id != 7)
            .expect("sample items have unique numeric keys");
        let mut state = ListBoxState::from_snapshot(snapshot, SelectionMode::Extended);
        state.set_selection_policy(super::selection_controls::DEFAULT_POLICY);
        Self {
            presentation: ExamplePresentation::new(look, "Vertical", event_stream),
            list: ListBoxControl::new(
                state,
                Self::handle_input,
                |key| (ID, *key).into(),
                |item| item.label.clone(),
                cx,
            )
            .require_focus_for_scroll(true)
            .virtualization(ListBoxVirtualization::Uniform { overscan: 2 }),
        }
    }

    fn handle_input(&mut self, input: ListBoxInput<u32>, _window: &mut Window, cx: &mut Context<Self>) {
        let update = self.list.apply(input, cx);
        self.presentation.record(&update.events, cx);
    }

    pub(super) fn set_selection_policy(&mut self, policy: SelectionPolicy, cx: &mut Context<Self>) {
        let update = self.list.set_selection_policy(policy, cx);
        self.presentation.record(&update.events, cx);
    }

    pub(super) fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.presentation.sync_look(look, cx);
    }
}

impl Render for VerticalListExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let surface = listbox! { window, cx;
            id = ID;
            control = &mut self.list;
            look = &self.presentation.look;
            aria_label = "Vertical list";
            width = 250.0;
            padding_x = 16.0;
            padding_y = 8.0;

            scroll_view! { vertical;
                visible_items = 5;

                vstack! {
                    gap = 4.0;
                    item_height = 36.0;

                    item_template = |model, _cx| {
                        hstack! { gap=6.0 align=center;
                            SelectionMark { selected: model.selected },
                            div().min_w(px(0.0)).truncate().child(model.item.label.clone()),
                        }
                        .size_full()
                        .px(px(12.0))
                    };
                }
            }
        };
        self.presentation
            .section(self.list.state.selected_keys().count(), surface)
            .w(px(VERTICAL_LIST_WIDTH))
            .flex_shrink_0()
    }
}
