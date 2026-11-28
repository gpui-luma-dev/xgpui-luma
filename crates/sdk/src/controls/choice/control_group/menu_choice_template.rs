use crate::style::PartRead;
use std::sync::Arc;

use gpui::{AnyElement, App, Window, div, prelude::*, px};

use super::model::{ControlGroupItemLike, ControlGroupItemRenderModel};
use super::template::{ControlGroupItemElementTemplate, ControlGroupTemplate, render_control_group_item_elements};
use super::theme::{ControlGroupItemVisualContext, ControlGroupTheme};
use crate::theme::StandardBoxScale;

pub type MenuChoiceRowContentFn<T> = Arc<
    dyn for<'a> Fn(
            &'a ControlGroupItemRenderModel<'a, T>,
            &ControlGroupItemVisualContext,
            &mut Window,
            &mut App,
        ) -> AnyElement
        + Send
        + Sync,
>;

pub fn menu_choice_group_template<T>() -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    Arc::new(|model, handlers, window, cx| {
        div()
            .id(model.id.clone())
            .w_full()
            .flex()
            .flex_col()
            .children(render_control_group_item_elements(model, handlers, window, cx).into_elements())
    })
}

pub fn menu_choice_row_item_element_template<T>(
    theme: Arc<dyn ControlGroupTheme>,
    row_height: f32,
    row_radius: f32,
    content: MenuChoiceRowContentFn<T>,
) -> ControlGroupItemElementTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    Arc::new(move |item, _item_template, window, cx| {
        let scale_factor = window.scale_factor();
        let scale = StandardBoxScale::compute(&theme.metrics(), scale_factor);
        let visual =
            theme.resolve_item_visual(item.selected, item.state.interaction_state(), &scale, row_height, row_radius);
        let row_content = content(item, &visual, window, cx);

        let mut row = div()
            .id((item.item.id().clone(), 0usize))
            .relative()
            .w_full()
            .h(px(row_height))
            .flex()
            .items_center()
            .rounded(px(visual.radius))
            .bg(visual.background)
            .text_color(visual.foreground)
            .font_family(visual.font_family.clone())
            .text_size(px(visual.typography.size))
            .line_height(px(visual.typography.line_height))
            .font_weight(visual.typography.weight)
            .child(row_content);

        if item.state.disabled {
            row = crate::style::fade(
                row,
                theme.resolve_list_style(false).part(xgpui_luma_style::PartId::new("item")).opacity(),
            );
        } else {
            row = row.cursor_pointer();
        }

        row
    })
}

pub fn configure_menu_choice_group<T>(
    template: ControlGroupTemplate<T>,
    theme: Arc<dyn ControlGroupTheme>,
    row_height: f32,
    row_radius: f32,
    content: MenuChoiceRowContentFn<T>,
) -> (ControlGroupTemplate<T>, ControlGroupItemElementTemplate<T>)
where
    T: ControlGroupItemLike + 'static,
{
    (template, menu_choice_row_item_element_template(theme, row_height, row_radius, content))
}
