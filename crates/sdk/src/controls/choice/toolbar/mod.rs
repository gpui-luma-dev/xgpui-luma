mod styled;
pub use styled::{StyledToolbarTheme, styled_toolbar_look};
mod control;
mod model;
mod template;
pub(crate) mod theme;

pub use control::{ToolbarControl, ToolbarEvent, ToolbarValue, horizontal_arrow_policy};
pub use model::{
    ToolbarBuilder, ToolbarChangeHandler, ToolbarClickHandler, ToolbarItem, ToolbarItemKind, ToolbarItemRenderModel,
    ToolbarItemSource, ToolbarRenderModel,
};
pub use template::{
    ThemedToolbarTemplate, ToolbarRenderedItem, ToolbarTemplate, ToolbarTemplateModifier, default_toolbar_template,
    toolbar_template_with_theme,
};
pub use theme::{ToolbarLook, ToolbarTheme, ToolbarVariant, default_toolbar_theme};

use gpui::{Entity, SharedString};

pub type Toolbar = Entity<ToolbarControl>;

pub fn new(id: impl Into<SharedString>) -> ToolbarBuilder {
    ToolbarBuilder::new(id)
}

mod callback;
pub use callback::ToolbarEventExt;
