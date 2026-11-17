use gpui::{AnyElement, App, div, prelude::*, px};

use crate::infra::locale::{LocaleContext, LogicalStyled};

use super::{SelectorRenderModel, theme::SelectorLook};
use crate::controls::selector_list::{SelectorItemLike, SelectorItemRenderModel};

pub(super) fn render_item_content<T>(
    model: &SelectorRenderModel<'_, T>,
    look: &SelectorLook,
    cx: &mut App,
) -> AnyElement
where
    T: SelectorItemLike + 'static,
{
    if let (Some(selected_index), Some(item_template)) = (model.selected_index, model.item_template)
        && let Some(item) = model.items.get(selected_index)
    {
        let active = model.active_path.is_some_and(|path| path.is_item(selected_index));
        let item_model = SelectorItemRenderModel {
            selector_id: model.id,
            item,
            index: selected_index,
            selected: true,
            active,
            open: model.open,
            enabled: model.enabled,
        };
        return item_template(&item_model, cx);
    }

    div()
        .flex()
        .flex_row_logical(LocaleContext::current(cx).direction)
        .items_center()
        .gap(px(look.trigger_gap))
        .child(model.label.clone())
        .into_any_element()
}
