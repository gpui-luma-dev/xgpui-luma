mod menu_choice_template;
mod button_item_template;
mod toggle_button_item_template;
mod control;
mod model;
mod template;
pub(crate) mod theme;
mod themed_template;

pub use control::{ControlGroupControl, ControlGroupEvent};
pub use model::{
    ControlGroupArrowAxis, ControlGroupArrowPolicy, ControlGroupBuilder, ControlGroupChromeModel,
    ControlGroupFocusStrategy, ControlGroupFocusTarget, ControlGroupFocusTargetProvider, ControlGroupItem,
    ControlGroupItemLike, ControlGroupItemRenderModel, ControlGroupLayout, ControlGroupModel, ControlGroupRenderModel,
    ControlGroupStateMode, ControlSelectionMode,
};
pub use button_item_template::button_item_template;
pub use menu_choice_template::{
    MenuChoiceRowContentFn, configure_menu_choice_group, menu_choice_group_template,
    menu_choice_row_item_element_template,
};
pub use toggle_button_item_template::{animated_toggle_button_item_template, toggle_button_item_template};
pub use template::{
    ControlGroupBoundsHandler, ControlGroupClickHandler, ControlGroupHoverHandler, ControlGroupItemElement,
    ControlGroupItemElementTemplate, ControlGroupItemElements, ControlGroupItemHandlerExt, ControlGroupItemHandlers,
    ControlGroupItemLayout, ControlGroupItemTemplate, ControlGroupMouseDownHandler, ControlGroupMouseUpHandler,
    ControlGroupTemplate, ControlGroupTemplateHandlers, control_group_item_layout_template,
    control_group_template_with_theme, default_control_group_template, make_control_group_item_element_template,
    make_control_group_item_template, render_control_group_item_elements, shared_control_group_template,
};
pub use theme::{
    ControlGroupItemPalette, ControlGroupItemVisualContext, DefaultControlGroupTheme, ControlGroupListLook,
    ControlGroupTheme, default_control_group_theme,
};
pub use themed_template::ThemedControlGroupTemplate;

pub use crate::infra::state::{CompositeItemState as ControlGroupItemState, ControlFocusState};

use gpui::{Entity, SharedString};

pub type ControlGroup<T> = Entity<ControlGroupControl<T>>;

pub fn new<T>(id: impl Into<SharedString>) -> ControlGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    ControlGroupBuilder::new(id)
}

/// Themed template; set layout via [`ControlGroupBuilder::horizontal`] or [`ControlGroupBuilder::vertical`].
pub fn vertical_group_template<T>() -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    shared_control_group_template()
}

pub fn horizontal_group_template<T>() -> ControlGroupTemplate<T>
where
    T: ControlGroupItemLike + 'static,
{
    shared_control_group_template()
}

mod callback;
pub use callback::ControlGroupEventExt;

pub use model::ControlGroupMotion;
