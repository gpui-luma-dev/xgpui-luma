mod control;
mod model;
mod styled;
pub use styled::{StyledOverlayWindowTheme, styled_overlay_window_look};
mod template;
pub(crate) mod theme;

pub use control::OverlayWindowControl;
pub use model::{
    OverlayWindowBuilder, OverlayWindowElementRenderer, OverlayWindowEvent, OverlayWindowMode, OverlayWindowModel,
    OverlayWindowPosition, OverlayWindowRenderModel, new as overlay_window,
};
pub use template::{
    OverlayWindowTemplate, OverlayWindowTemplateHandlers, OverlayWindowTemplateModifier, OverlayWindowTemplateParts,
    ThemedOverlayWindowTemplate, default_overlay_window_template,
};
pub use theme::{OverlayWindowLook, OverlayWindowTheme, default_overlay_window_theme};

use gpui::Entity;

pub type OverlayWindow = Entity<OverlayWindowControl>;

mod callback;
pub use callback::OverlayWindowEventExt;
