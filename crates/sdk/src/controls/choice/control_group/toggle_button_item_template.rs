use std::sync::Arc;

use gpui::{AnyElement, IntoElement};

use super::button_item_template::button_item_template;
use super::model::{ControlGroupItemLike, ControlGroupItemRenderModel};
use super::template::{ControlGroupItemTemplate, make_control_group_item_template};
use crate::controls::button_family::ButtonFamilyRole;
use crate::controls::button::ButtonTemplate;
use crate::controls::toggle::ToggleData;

/// Icon/control-group items with [`ButtonFamilyRole::Toggle`].
///
/// Pass a button template whose theme resolves selected vs unselected toggle segments
/// (for example a styled secondary button-family template from the active theme).
pub fn toggle_button_item_template<T>(
    button_template: Arc<dyn ButtonTemplate<bool>>,
    round: bool,
    content_fn: impl Fn(&T) -> AnyElement + Send + Sync + 'static,
) -> ControlGroupItemTemplate<T>
where
    T: ControlGroupItemLike + Clone + Send + Sync + 'static,
{
    button_item_template(button_template, |selected| ButtonFamilyRole::Toggle { selected }, round, content_fn)
}

/// Toggle items backed by the ControlGroup selection transition.
pub fn animated_toggle_button_item_template<T>(
    button_template: Arc<dyn ButtonTemplate<ToggleData>>,
    round: bool,
    content_fn: impl Fn(&T) -> AnyElement + Send + Sync + 'static,
) -> ControlGroupItemTemplate<T>
where
    T: ControlGroupItemLike + Clone + Send + Sync + 'static,
{
    let content_fn = Arc::new(content_fn);
    make_control_group_item_template(move |item: &ControlGroupItemRenderModel<'_, T>, window, cx| {
        let item_data = item.item.clone();
        let render_model = crate::controls::button::ButtonRenderModel {
            id: format!("{}-{}", item.group_id, item.item.id()).into(),
            data: ToggleData { selected: item.selected, progress: item.selection_progress },
            content: Arc::new({
                let content_fn = Arc::clone(&content_fn);
                move |_, _| content_fn(&item_data)
            }),
            role: ButtonFamilyRole::Toggle { selected: item.selected },
            state: item.state.interaction_state(),
            round,
            radius_override: std::cell::Cell::new(None),
            elevation: true,
            compact: false,
            ..Default::default()
        };

        button_template.render(&render_model, window, cx).into_any_element()
    })
}
