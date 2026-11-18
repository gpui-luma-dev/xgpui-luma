//! Single-focus primary action with an independently enabled disclosure menu.
//!
//! Enter/Space activates the primary action while closed; Up/Down opens the
//! menu. While open, the shared menu engine handles navigation and selection.
//! Escape dismisses without moving focus; Tab leaves the control and dismisses.
//! The retained control owns one PopupMenu engine, forwarding semantic events.
mod model;
mod control;
mod template;
mod callback;

pub use control::{SplitButton, SplitButtonEvent};
pub use model::{SplitButtonBuilder, SplitButtonMenuLabel, SplitButtonModel, SplitButtonState, SplitButtonText};
pub use callback::SplitButtonEventExt;
pub use template::{SplitButtonTemplate, ThemedSplitButtonTemplate, default_split_button_template};
pub use crate::controls::popup_menu::SplitFaceHandlers as SplitButtonFaceHandlers;
// These are deliberately shared presentation contracts, not retained-control aliases.
pub use crate::controls::popup_menu::{
    PopupMenuTriggerModel as SplitButtonTriggerModel, PopupMenuRenderModel as SplitButtonRenderModel,
    PopupMenuTemplateHandlers as SplitButtonTemplateHandlers, PopupMenuTheme as SplitButtonTheme,
};

#[cfg(all(test, feature = "test-support"))]
mod tests;
