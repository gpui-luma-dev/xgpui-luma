//! Listbox-in-a-panel selection control (always-visible option list).
//!
//! Distinct from [`crate::controls::selector`] (popup dropdown) and
//! [`crate::controls::selector_list`] (shared list chrome, not a control).

mod control;
mod item_template;
mod model;
mod template;
pub(crate) mod theme;

pub use control::{SelectionPanelControl, SelectionPanelEvent, SelectionPanelState, SelectionPanelStepDirection};
pub use item_template::{
    SelectionPanelItemRenderModel, SelectionPanelItemTemplate, SelectionPanelItemTemplateModifier,
    make_selection_panel_item_template,
};
pub use model::{
    SelectionPanelRole, HoverActivationPolicy, SelectionPanelBuilder, SelectionPanelLookProvider, SelectionPanelItem,
    SelectionPanelItemLike, SelectionPanelModel, SelectionPanelPath, new,
};
pub use template::{
    SelectionPanelClickHandler, SelectionPanelHoverHandler, SelectionPanelModifier, SelectionPanelMouseDownHandler,
    SelectionPanelMouseUpHandler, SelectionPanelRenderModel, SelectionPanelTemplate, SelectionPanelTemplateHandlers,
    default_selection_panel_template, render_selection_panel,
};
pub use theme::{SelectionPanelLook, default_selection_panel_look, styled_selection_panel_look};

mod callback;
pub use callback::SelectionPanelEventExt;
