//! Popup selector control (trigger + anchored option list).
//!
//! Distinct from [`crate::controls::selection_panel`] (always-visible list) and
//! [`crate::controls::selector_list`] (shared list chrome, not a control).
//!
//! # Examples
//!
//! Default built-in items:
//! ```ignore
//! use xgpui_luma::controls::selector::{Selector, SelectorItem};
//!
//! let selector = Selector::new("status")
//!     .items([
//!         SelectorItem::new("todo").label("To Do"),
//!         SelectorItem::new("doing").label("In Progress"),
//!         SelectorItem::new("done").label("Done"),
//!     ])
//!     .selected_id("doing");
//! ```
//!
//! Typed custom items:
//! ```ignore
//! use gpui::SharedString;
//! use xgpui_luma::controls::selector::{Selector, SelectorItemLike};
//!
//! #[derive(Clone)]
//! struct SwatchItem {
//!     id: SharedString,
//!     enabled: bool,
//! }
//!
//! impl SelectorItemLike for SwatchItem {
//!     fn id(&self) -> &SharedString { &self.id }
//!     fn is_enabled(&self) -> bool { self.enabled }
//! }
//!
//! let selector = Selector::<SwatchItem>::new_typed("palette");
//! ```
mod control;
mod item_template;
mod model;
mod template;
pub(crate) mod theme;

pub use control::{Selector, SelectorEvent};

pub use model::{
    SelectorBuilder, SelectorIcons, SelectorItemRenderModel, SelectorItemTemplate, SelectorModel, SelectorPath,
    SelectorOpeningMode, SelectorPlacement, SelectorRenderModel, SelectorItem, SelectorItemLike, SelectorTriggerStyle,
};
pub use template::{
    SelectorTemplate, SelectorTemplateHandlers, SelectorTemplateModifier, ThemedSelectorTemplate,
    default_selector_template,
};
pub use theme::{
    DefaultSelectorTheme, SelectorLook, SelectorPalette, SelectorTheme, SelectorVisualState, default_selector_theme,
};
#[allow(unused_imports)]
pub(crate) use theme::compose_selector_look;

pub use crate::infra::state::ControlFocusState;

mod callback;
pub use callback::SelectorEventExt;

pub use model::SelectorMotion;
