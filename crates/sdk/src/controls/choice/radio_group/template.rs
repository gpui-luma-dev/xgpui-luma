use std::sync::Arc;

use gpui::{AnyElement, App, Window, div, px, prelude::*};

use crate::controls::button_family::ButtonSize;
use crate::controls::button::ButtonTemplate;
use crate::controls::control_group::{
    self as control_group, choice_button_item_element_template, ControlGroupItemElementTemplate,
    ControlGroupItemHandlerExt, ControlGroupItemLike, ControlGroupItemRenderModel, ControlGroupItemTemplate,
    ControlGroupLayout, ControlGroupRenderModel, ControlGroupTemplate, ControlGroupTemplateHandlers,
};
use crate::controls::radio_button::RadioButtonData;

use super::model::RadioGroupTemplate;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RadioGroupLayout {
    Vertical,
    Horizontal,
}

pub fn radio_group_buttons_template<T>(
    button_template: Arc<dyn ButtonTemplate<RadioButtonData>>,
    layout: RadioGroupLayout,
) -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    Arc::new(move |model, handlers, window, cx| {
        let rows = render_radio_button_rows(model, handlers, &button_template, window, cx);

        let key = match layout {
            RadioGroupLayout::Vertical => "vertical",
            RadioGroupLayout::Horizontal => "horizontal",
        };
        let recipe = model.stylesheet.radio_group.authored_geometry(key);
        let root = div().id(model.id.clone()).flex().gap(px(recipe.gap.unwrap_or_default()));
        match layout {
            RadioGroupLayout::Vertical => root.flex_col().items_start().children(rows),
            RadioGroupLayout::Horizontal => root.items_center().children(rows),
        }
    })
}

pub fn radio_group_button_item_element_template<T>(
    button_template: Arc<dyn ButtonTemplate<RadioButtonData>>,
) -> ControlGroupItemElementTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    Arc::new(move |item, item_template, window, cx| {
        render_radio_button_option(item, item_template, &button_template, window, cx)
    })
}

pub fn render_radio_button_option<T>(
    item: &ControlGroupItemRenderModel<'_, T>,
    item_template: Option<&ControlGroupItemTemplate<T>>,
    button_template: &Arc<dyn ButtonTemplate<RadioButtonData>>,
    window: &mut Window,
    cx: &mut App,
) -> gpui::Stateful<gpui::Div>
where
    T: ControlGroupItemLike + 'static,
{
    choice_button_item_element_template(button_template.clone(), ButtonSize::Md, RadioButtonData::new)(
        item,
        item_template,
        window,
        cx,
    )
}

pub fn render_radio_button_rows<T>(
    model: &ControlGroupRenderModel<'_, T>,
    handlers: ControlGroupTemplateHandlers,
    button_template: &Arc<dyn ButtonTemplate<RadioButtonData>>,
    window: &mut Window,
    cx: &mut App,
) -> Vec<AnyElement>
where
    T: ControlGroupItemLike + 'static,
{
    let mut rows = Vec::with_capacity(model.items.len());

    for (item, item_handlers) in model.items.iter().zip(handlers.into_item_handlers()) {
        let mut row = if let Some(template) = model.item_element_template {
            template(item, model.item_template, window, cx)
        } else {
            render_radio_button_option(item, model.item_template, button_template, window, cx)
        }
        .control_group_item_handlers(item_handlers);

        if model.layout == ControlGroupLayout::Horizontal {
            row = row.flex_1().h_full();
        }

        if item.enabled {
            row = row.cursor_pointer();
        } else {
            row = row.opacity(
                model
                    .stylesheet
                    .radio_group
                    .authored_geometry(match model.layout {
                        ControlGroupLayout::Vertical => "vertical",
                        ControlGroupLayout::Horizontal => "horizontal",
                    })
                    .disabled_opacity
                    .unwrap_or_default(),
            );
        }

        rows.push(row.into_any_element());
    }

    rows
}

pub fn radio_group_container_template<T>() -> RadioGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    control_group::vertical_group_template()
}

pub fn horizontal_group_container_template<T>() -> RadioGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    control_group::horizontal_group_template()
}

pub fn default_radio_group_container_template<T>() -> RadioGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    radio_group_container_template()
}

pub fn default_horizontal_group_container_template<T>() -> RadioGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    horizontal_group_container_template()
}
