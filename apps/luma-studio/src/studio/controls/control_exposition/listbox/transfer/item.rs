//! Domain row and drag preview; the builder supplies SDK interaction surfaces.

use std::sync::Arc;
use gpui::{
    App, Context, Div, Entity, IntoElement, Render, RenderOnce, SharedString, Stateful, Window, div, prelude::*, px,
};
use xgpui_luma::controls::listbox::{ListBoxItemState, ListBoxVisibleItem};
use xgpui_luma_look_shadcn::ShadcnLook;

use super::{LAYOUT, model::TransferItem};
use super::super::{ITEM_CONTENT_GAP, presentation::SelectionMark};

#[derive(IntoElement)]
pub(super) struct TransferRow {
    surface: Stateful<Div>,
    label: SharedString,
    selected: bool,
}

impl TransferRow {
    pub fn new(surface: Stateful<Div>, item: ListBoxVisibleItem<'_, TransferItem, u32>) -> Self {
        Self {
            surface: surface.aria_label(item.item.label.clone()),
            label: item.item.label.clone(),
            selected: item.state.selected,
        }
    }
}

impl RenderOnce for TransferRow {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.surface
            .flex()
            .items_center()
            .gap(px(ITEM_CONTENT_GAP))
            .flex_shrink_0()
            .w_full()
            .min_w(px(0.0))
            .h(px(LAYOUT.item_height))
            .px(px(LAYOUT.item_padding))
            .rounded(px(LAYOUT.radius))
            .overflow_hidden()
            .child(SelectionMark { selected: self.selected })
            .child(div().min_w(px(0.0)).truncate().child(self.label))
    }
}

pub(super) struct DragPreview {
    look: Arc<ShadcnLook>,
    label: SharedString,
}

impl DragPreview {
    pub fn spawn(look: Arc<ShadcnLook>, item: &TransferItem, keys: &[u32], cx: &mut App) -> Entity<Self> {
        let label = if keys.len() == 1 {
            item.label.clone()
        } else {
            format!("{} items", keys.len()).into()
        };
        cx.new(|_| Self { look, label })
    }
}

impl Render for DragPreview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let surface = self.look.listbox_row(
            "transfer-drag-preview",
            ListBoxItemState { selected: true, active: false, enabled: true },
            false,
        );
        div()
            .w(px(136.0))
            .opacity(0.9)
            .child(TransferRow { surface, label: self.label.clone(), selected: true })
    }
}
