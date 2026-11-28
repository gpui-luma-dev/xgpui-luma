mod control;
mod model;
mod styled;

pub use styled::{StyledOverlayWindowTheme, resolve_overlay_window};
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

pub use theme::{OverlayWindowTheme, default_overlay_window_theme};

use gpui::Entity;

pub type OverlayWindow = Entity<OverlayWindowControl>;

mod callback;
pub use callback::OverlayWindowEventExt;
