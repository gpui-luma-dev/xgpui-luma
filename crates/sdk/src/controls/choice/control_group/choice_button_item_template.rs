use std::sync::Arc;

use gpui::{div, prelude::*, px};

use super::{ControlGroupItemElementTemplate, ControlGroupItemLike};
use crate::controls::button::{ButtonRenderModel, ButtonTemplate};
use crate::controls::button_family::{ButtonFamilyRole, ButtonSize};

/// Adapt choice indicators to group rows without taking over group state or input.
/// Custom content is placed beside an indicator-only button; element overrides are
/// selected by the group row renderer before this fallback is called.
pub fn choice_button_item_element_template<T, D>(
    template: Arc<dyn ButtonTemplate<D>>,
    size: ButtonSize,
    data: impl Fn(bool) -> D + Send + Sync + 'static,
) -> ControlGroupItemElementTemplate<T>
where
    T: ControlGroupItemLike + 'static,
    D: Default + 'static,
{
    Arc::new(move |item, custom, window, cx| {
        let id: gpui::SharedString = format!("{}-{}", item.group_id, item.item.id()).into();
        let label = item.item.label().clone();
        let model = ButtonRenderModel {
            id: id.clone(),
            data: data(item.selected),
            size,
            state: item.state.interaction_state(),
            role: if custom.is_some() {
                ButtonFamilyRole::Icon
            } else {
                ButtonFamilyRole::Text
            },
            content: Arc::new(move |_, _| div().child(label.clone()).into_any_element()),
            compact: custom.is_some(),
            elevation: custom.is_none(),
            ..Default::default()
        };
        let button = template.render(&model, window, cx);
        let row = if let Some(custom) = custom {
            let gap = item
                .stylesheet
                .radio_group
                .authored_geometry(match item.layout {
                    super::ControlGroupLayout::Vertical => "vertical",
                    super::ControlGroupLayout::Horizontal => "horizontal",
                })
                .item_gap
                .unwrap_or_default();
            div()
                .id((gpui::ElementId::from(item.group_id.clone()), item.item.id().clone()))
                .flex()
                .items_center()
                .gap(px(gap))
                .child(button)
                .child(div().flex_1().min_w_0().child(custom(item, window, cx)))
        } else {
            button
        };
        row.debug_selector(move || id.to_string())
    })
}
