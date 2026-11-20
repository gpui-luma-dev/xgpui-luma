mod control;
mod model;
mod styled;
pub use styled::{MenuSource, StyledPopupMenuTheme};
mod template;
pub(crate) mod theme;

pub use control::{PopupMenu, PopupMenuEvent};
pub use model::{
    PopupMenuBuilder, PopupMenuModel, PopupMenuPlacement, PopupMenuRenderModel, PopupMenuTriggerModel,
    PopupMenuTriggerStyle, icon_content,
};
pub use template::{
    SplitFaceHandlers, PopupMenuTemplate, PopupMenuTemplateHandlers, PopupMenuTemplateModifier,
    ThemedPopupMenuTemplate, default_popup_menu_template,
};
pub use theme::{
    DefaultPopupMenuTheme, PopupMenuLook, PopupMenuPalette, PopupMenuTheme, PopupMenuTriggerMetrics,
    compose_popup_menu_look, default_popup_menu_theme,
};

pub use crate::infra::presenter::{ControlPresenter, HasPresenter};
pub use crate::infra::state::{ControlFocusState, MenuPath};

mod callback;
pub use callback::PopupMenuEventExt;

pub use model::PopupMenuMotion;
