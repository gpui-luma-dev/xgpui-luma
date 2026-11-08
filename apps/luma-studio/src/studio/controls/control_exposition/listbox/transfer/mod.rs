//! A selection transfer example. GPUI owns the drag gesture; this
//! host validates the payload and commits transfers or reordering on drop.

mod model;
mod item;

use std::sync::Arc;

use gpui::{Context, Div, Entity, FontWeight, IntoElement, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::listbox::{ListBoxBinding, ListBoxInput, ListBoxScrollHandle, SelectionPolicy};
use xgpui_luma::interaction::{ScrollBoundaryPolicy, WheelScrollPolicy};
use xgpui_luma::infra::drag_drop::{DragDropElementExt, DragDropEvent, DropProposal};
use xgpui_luma::{hstack, vstack};
use xgpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextSize};

use model::{Side, TransferModel};
use item::{DragPreview, TransferRow};
use xgpui_luma_look_shadcn::ListBoxBuilder;
use xgpui_luma::controls::listbox::{ListBoxLayout, ListBoxVirtualization};
use super::{ListBoxSampleLayout, VERTICAL_LIST_WIDTH};
use super::presentation::ExamplePresentation;
use super::super::event_stream::ControlEventStream;

const IDS: [&str; 2] = ["listbox-transfer-left", "listbox-transfer-right"];
const ROW_HEIGHT: f32 = 36.0;
const ROW_GAP: f32 = 4.0;
const VISIBLE_ROWS: f32 = 5.0;
const LAYOUT: ListBoxSampleLayout = ListBoxSampleLayout {
    item_height: ROW_HEIGHT,
    spacing: ROW_GAP,
    item_padding: 8.0,
    viewport_height: ROW_HEIGHT * VISIBLE_ROWS + ROW_GAP * (VISIBLE_ROWS - 1.0),
    inset_x: 8.0,
    inset_y: 8.0,
    radius: 8.0,
    border: 1.0,
};

type ItemDrop = DropProposal<Side, u32>;

pub(super) struct TransferExample {
    model: TransferModel,
    drop_layout: [xgpui_luma::motion::DropLayoutMotion<u32>; 2],
    bindings: [ListBoxBinding; 2],
    scrolls: [ListBoxScrollHandle<u32>; 2],
    presentations: [ExamplePresentation; 2],
}

impl TransferExample {
    pub fn new(look: Arc<ShadcnLook>, events: Entity<ControlEventStream>, cx: &mut Context<Self>) -> Self {
        let mut model = TransferModel::new();
        for list in &mut model.lists {
            list.set_selection_policy(super::selection_controls::DEFAULT_POLICY);
        }
        let drop_layout: [xgpui_luma::motion::DropLayoutMotion<u32>; 2] = std::array::from_fn(|_| Default::default());
        Self {
            model,
            drop_layout,
            bindings: Side::ALL.map(|_| ListBoxBinding::new(cx)),
            scrolls: [
                ListBoxScrollHandle::default()
                    .wheel_scroll_policy(WheelScrollPolicy::Pointer)
                    .scroll_boundary_policy(ScrollBoundaryPolicy::Chain),
                ListBoxScrollHandle::default()
                    .wheel_scroll_policy(WheelScrollPolicy::RequireFocus)
                    .scroll_boundary_policy(ScrollBoundaryPolicy::Contain),
            ],
            presentations: [
                ExamplePresentation::new(look.clone(), "Left", events.clone()),
                ExamplePresentation::new(look, "Right", events),
            ],
        }
    }

    pub(super) fn set_selection_policy(&mut self, policy: SelectionPolicy, cx: &mut Context<Self>) {
        for (index, list) in self.model.lists.iter_mut().enumerate() {
            let update = list.set_selection_policy(policy);
            self.scrolls[index].handle_update(&update, cx);
            self.presentations[index].record(&update.events, cx);
        }
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        for presentation in &mut self.presentations {
            presentation.sync_look(look.clone(), cx);
        }
    }

    fn left_input(&mut self, input: ListBoxInput<u32>, _: &mut Window, cx: &mut Context<Self>) {
        self.handle_input(Side::Left, input, cx);
    }

    fn right_input(&mut self, input: ListBoxInput<u32>, _: &mut Window, cx: &mut Context<Self>) {
        self.handle_input(Side::Right, input, cx);
    }

    fn handle_input(&mut self, side: Side, input: ListBoxInput<u32>, cx: &mut Context<Self>) {
        let index = side.index();
        let update = self.model.lists[index].apply(input);
        self.scrolls[index].handle_update(&update, cx);
        self.presentations[index].record(&update.events, cx);
    }

    fn record_drag_event(&mut self, event: DragDropEvent<Side, u32>, cx: &mut Context<Self>) {
        let index = event.list().index();
        if matches!(event, DragDropEvent::DragStarted { .. }) {
            for motion in &self.drop_layout {
                motion.cancel();
            }
        }
        self.presentations[index].record_message(&format!("{event:?}"), cx);
    }

    fn accept_drop(&mut self, proposal: ItemDrop, window: &mut Window, cx: &mut Context<Self>) {
        if !proposal.is_active() {
            return;
        }
        let target = *proposal.target();
        let updates =
            match self.model.move_items(*proposal.source(), target, proposal.keys(), proposal.before().copied()) {
                Ok(updates) => updates,
                Err(error) => {
                    for event in proposal.rejected(format!("{error:?}")) {
                        self.record_drag_event(event, cx);
                    }
                    return;
                }
            };
        // Domain mutations and final ListBox events remain host-owned. Report
        // the result to the shared lifecycle only after both lists are committed.
        for (index, update) in updates.iter().enumerate() {
            if update.changed {
                self.drop_layout[index].start();
            }
            self.scrolls[index].handle_update(update, cx);
            self.presentations[index].record(&update.events, cx);
        }
        let keys = self.model.lists[target.index()]
            .snapshot()
            .items()
            .iter()
            .filter(|item| proposal.keys().contains(&item.id))
            .map(|item| item.id)
            .collect();
        for event in proposal.committed(updates.iter().any(|update| update.changed), keys) {
            self.record_drag_event(event, cx);
        }
        self.bindings[target.index()].focus_handle().focus(window, cx);
    }

    fn render_list(&mut self, side: Side, window: &mut Window, cx: &mut Context<Self>) -> Div {
        let index = side.index();
        let presentation = &self.presentations[index];
        let preview_look = presentation.look.clone();
        let on_input = match side {
            Side::Left => Self::left_input,
            Side::Right => Self::right_input,
        };
        let surface = ListBoxBuilder::new(
            IDS[index],
            &self.model.lists[index],
            &mut self.bindings[index],
            &self.scrolls[index],
            on_input,
            |key| ("item", *key).into(),
            |surface, item, _| TransferRow::new(surface, item),
        )
        .look(&presentation.look)
        .drop_layout_motion(self.drop_layout[index].clone())
        .virtualization(ListBoxVirtualization::Uniform { overscan: 2 })
        .layout(ListBoxLayout {
            item_height: LAYOUT.item_height,
            spacing: LAYOUT.spacing,
            viewport_height: LAYOUT.viewport_height,
            inset_x: LAYOUT.inset_x,
            inset_y: LAYOUT.inset_y,
        })
        .aria_label(if side == Side::Left {
            "Left transfer list"
        } else {
            "Right transfer list"
        })
        .empty(div().text_color(presentation.look.chrome().muted_text).child("Drop here"))
        .drag_and_drop(side, Self::accept_drop, Self::record_drag_event, move |item, keys, _, cx| {
            DragPreview::spawn(preview_look.clone(), item, keys, cx)
        })
        .build(window, cx);
        vstack! { gap=4.0;
            presentation.section(self.model.lists[index].selected_keys().count(), surface),
            div().typography_style(presentation.look.typography_scale(ShadcnTextSize::Xs))
                .text_color(presentation.look.chrome().muted_text)
                .child(format!("{} items · {}", self.model.lists[index].snapshot().items().len(), if side == Side::Left {
                    "Pointer/Chain"
                } else {
                    "RequireFocus/Contain"
                })),
        }
        .w(px(VERTICAL_LIST_WIDTH))
        .flex_shrink_0()
    }
}

impl Render for TransferExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let left = self.render_list(Side::Left, window, cx);
        let right = self.render_list(Side::Right, window, cx);
        let look = &self.presentations[0].look;
        vstack! { gap=8.0;
            div().typography_style(look.typography_scale(ShadcnTextSize::Sm)).font_weight(FontWeight::SEMIBOLD)
                .text_color(look.chrome().title_text).child("Drag and drop"),
            div().typography_style(look.typography_scale(ShadcnTextSize::Xs)).text_color(look.chrome().muted_text)
                .child("Click or Space toggles selection. Drag within or between lists. A selected row moves the selection; an unselected row moves alone. Hold near the top or bottom edge to scroll. Drop at the line. Escape cancels."),
            hstack! { gap=12.0; left, right }.w_full().min_w(px(0.0)),
        }
        .id("listbox-transfer-example")
        .w_full()
        .min_w(px(0.0))
        .cancel_drag_on_escape()
    }
}
