mod control;
mod model;
mod styled;
pub use styled::StyledContextMenuTheme;
mod template;
pub(crate) mod theme;

pub use control::{ContextMenu, ContextMenuEvent};
pub use model::{ContextMenuBuilder, ContextMenuModel, ContextMenuRenderModel, ContextMenuTargetContent};
pub use template::{
    ContextMenuTemplate, ContextMenuTemplateHandlers, ContextMenuTemplateModifier, ThemedContextMenuTemplate,
    default_context_menu_template,
};

#[allow(deprecated)]
pub use theme::ContextMenuLook;
pub use theme::{ContextMenuTheme, default_context_menu_theme};

pub use crate::infra::menu_item::{MenuItem, MenuItemIcon};
pub use crate::infra::state::{ControlFocusState, MenuPath};

mod callback;
pub use callback::ContextMenuEventExt;

pub use model::ContextMenuMotion;
