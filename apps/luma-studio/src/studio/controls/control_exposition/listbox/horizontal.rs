//! Independent horizontal ListBox example with an inline content template.

use std::sync::Arc;

use gpui::{Context, Entity, IntoElement, Render, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::listbox::{ListBoxInput, ListBoxSnapshot, ListBoxState, SelectionMode, SelectionPolicy};
use xgpui_luma::hstack;
use xgpui_luma_look_shadcn::ShadcnLook;

use super::{ListBoxSampleLayout};
use super::presentation::{ExamplePresentation, SelectionMark};
use super::markup::listbox;
use xgpui_luma::controls::listbox::{ListBoxControl, ListBoxVirtualization};
use super::super::event_stream::ControlEventStream;

const ID: &str = "listbox-horizontal";
pub(in super::super) const CARD_WIDTH: f32 = 136.0;
// Inspector mirror of the markup below; headless geometry tests guard against drift.
pub(in super::super) const LAYOUT: ListBoxSampleLayout = ListBoxSampleLayout {
    item_height: 36.0,
    spacing: 8.0,
    item_padding: 12.0,
    viewport_height: 36.0,
    inset_x: 16.0,
    inset_y: 8.0,
    radius: 8.0,
    border: 1.0,
};

struct CardItem {
    id: u32,
    label: SharedString,
}

pub(super) struct HorizontalListExample {
    presentation: ExamplePresentation,
    list: ListBoxControl<Self, CardItem, u32>,
}

impl HorizontalListExample {
    pub(super) fn captured_motion(&self) -> (bool, xgpui_luma::controls::listbox::ListBoxMotion) {
        (self.list.is_motion_enabled(), self.list.motion_profile())
    }

    pub(super) fn new(look: Arc<ShadcnLook>, event_stream: Entity<ControlEventStream>, cx: &mut Context<Self>) -> Self {
        let items = (1..=1_000).map(|id| CardItem { id, label: format!("Card {id}").into() });
        let snapshot = ListBoxSnapshot::try_with_enabled(items, |item| item.id, |item| item.id != 7)
            .expect("sample items have unique numeric keys");
        let mut state = ListBoxState::from_snapshot(snapshot, SelectionMode::Extended);
        state.set_selection_policy(super::selection_controls::DEFAULT_POLICY);
        Self {
            presentation: ExamplePresentation::new(look, "Horizontal", event_stream),
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

impl Render for HorizontalListExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let surface = listbox! { window, cx;
            id = ID;
            control = &mut self.list;
            look = &self.presentation.look;
            aria_label = "Horizontal list";
            padding_x = 16.0;
            padding_y = 8.0;

            scroll_view! { horizontal;
                hstack! {
                    gap = 8.0;
                    item_width = 136.0;
                    item_height = 36.0;

                    item_template = |model, _cx| {
                        hstack! { gap=6.0 align=center justify=center;
                            SelectionMark { selected: model.selected },
                            div().whitespace_nowrap().child(model.item.label.clone()),
                        }
                        .size_full()
                        .px(px(12.0))
                    };
                }
            }
        };
        self.presentation.section(self.list.state.selected_keys().count(), surface)
    }
}
