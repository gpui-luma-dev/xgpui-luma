use std::sync::Arc;

use gpui::{AnyElement, IntoElement};

use super::model::{ControlGroupItemLike, ControlGroupItemRenderModel};
use super::template::{ControlGroupItemTemplate, make_control_group_item_template};
use crate::controls::button_family::{ButtonFamilyRole, ButtonSize};
use crate::controls::button::{ButtonRenderModel, ButtonTemplate};

pub fn button_item_template<T>(
    button_template: Arc<dyn ButtonTemplate<bool>>,
    role_fn: impl Fn(bool) -> ButtonFamilyRole + Send + Sync + 'static,
    round: bool,
    content_fn: impl Fn(&T) -> AnyElement + Send + Sync + 'static,
) -> ControlGroupItemTemplate<T>
where
    T: ControlGroupItemLike + Clone + Send + Sync + 'static,
{
    let content_fn = Arc::new(content_fn);
    let role_fn = Arc::new(role_fn);
    make_control_group_item_template(move |item: &ControlGroupItemRenderModel<'_, T>, window, cx| {
        let item_data = item.item.clone();
        let render_model = ButtonRenderModel {
            id: format!("{}-{}", item.group_id, item.item.id()).into(),
            data: item.selected,
            content: Arc::new({
                let content_fn = Arc::clone(&content_fn);
                move |_, _| content_fn(&item_data)
            }),
            role: role_fn(item.selected),
            size: ButtonSize::Sm,
            state: item.state.interaction_state(),
            round,
            radius_override: std::cell::Cell::new(None),
            elevation: true,
            compact: false,
            look: None,
            ..Default::default()
        };

        button_template.render(&render_model, window, cx).into_any_element()
    })
}
