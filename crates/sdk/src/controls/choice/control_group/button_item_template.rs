use std::sync::Arc;

use gpui::{
    AnyElement, IntoElement, ParentElement, Styled, px,
    prelude::{FluentBuilder, InteractiveElement},
};

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

/// Shared button chrome for group items, retaining window-aware custom content.
/// `resolve` owns look geometry; input and focus remain on the group engine.
pub fn button_item_element_template<T: ControlGroupItemLike + 'static>(
    template: crate::controls::button::DefaultButtonTemplate<bool>,
    size: ButtonSize,
    round: bool,
    resolve: impl Fn(
        bool,
        crate::theme::InteractionState,
        &crate::theme::StandardBoxScale,
    ) -> crate::controls::button_family::ButtonFamilyLook
    + Send
    + Sync
    + 'static,
) -> super::template::ControlGroupItemElementTemplate<T> {
    use crate::theme::{InteractionState, StandardBoxScale};
    Arc::new(move |item, custom, window, cx| {
        let scale = StandardBoxScale::compute(size, &template.theme.metrics(), window.scale_factor());
        let state = InteractionState { focused: item.state.focus_visible, ..item.state.interaction_state() };
        let look = resolve(item.selected, state, &scale);
        let radius = look.radius;
        let height = look.height;
        let model = ButtonRenderModel {
            id: format!("{}-{}", item.group_id, item.item.id()).into(),
            data: item.selected,
            role: ButtonFamilyRole::Toggle { selected: item.selected },
            size,
            state,
            round,
            compact: true,
            elevation: false,
            radius_override: std::cell::Cell::new(Some(radius)),
            look: Some(Arc::new(move |_| look.clone())),
            ..Default::default()
        };
        let selector = model.id.to_string();
        template
            .render_with_content(&model, window, cx, |context, window, cx| {
                if let Some(custom) = custom {
                    custom(
                        &ControlGroupItemRenderModel {
                            group_id: item.group_id,
                            stylesheet: item.stylesheet,
                            layout: item.layout,
                            item: item.item,
                            index: item.index,
                            sibling_count: item.sibling_count,
                            selected: item.selected,
                            selection_progress: item.selection_progress,
                            active: item.active,
                            enabled: item.enabled,
                            state: item.state,
                            selection_mode: item.selection_mode,
                            foreground: Some(context.look.foreground),
                        },
                        window,
                        cx,
                    )
                } else {
                    gpui::div().child(item.item.label().clone()).into_any_element()
                }
            })
            .debug_selector(move || selector.clone())
            .min_h(px(height))
            .when(round, |row| row.w(px(height)).min_w(px(height)).max_w(px(height)))
    })
}
