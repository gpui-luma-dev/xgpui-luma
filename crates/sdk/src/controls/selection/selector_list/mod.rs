//! Shared selector item-list chrome used by [`super::selector`], combobox, and autocomplete.
//!
//! This is not a spawnable control. Prefer [`super::selector`] (popup dropdown) or
//! [`super::selection_panel`] (listbox-in-a-panel).

mod item_template;
mod items_template;
mod model;

pub use item_template::{SelectorItemRenderModel, SelectorItemTemplate, make_selector_item_template};
pub use items_template::{
    SelectorPopupGeometry, SelectorPopupGeometryHandler, DefaultSelectorItemsTemplate, SelectorItemsPanelLook,
    SelectorItemsRenderModel, SelectorItemsTemplate, SelectorItemsTemplateHandlers, SelectorItemsTemplateModifier,
    SelectorPanelClickHandler, SelectorPanelHoverHandler, SelectorPanelMouseDownHandler,
    SelectorPanelScrollWheelHandler, default_selector_items_panel_look, default_selector_items_template,
    styled_selector_items_panel_look,
};
pub use model::{SelectorItem, SelectorItemLike, SelectorPath, normalize_selector_items};
pub(crate) use items_template::items_template_with_modifier;
