//! Spawnable control families. Shared seams live in [`crate::infra`] and [`crate::motion`].

pub mod button;
pub mod choice;
pub mod collection;
pub mod display;
pub mod layout;
pub mod navigation;
pub mod overlay;
pub mod range;
pub mod selection;
pub mod text;

pub use button::icon_button;
pub use button::family as button_family;
pub use button::split_button;

pub use choice::{checkbox, control_group, icon_group, radio_button, radio_group, switch, toggle, toolbar};

pub use text::{textarea, textfield};

pub use overlay::{tooltip, context_menu, floating_menu, overlay_window, popover_button, popup_menu, slide_panel};

pub use collection::{listbox, table, tree_view};

pub use display::{badge, card};

pub use selection::{autocomplete, combobox, search_selector, selection_panel, selector, selector_list};

pub use navigation::{accordion, pager, sidebar, stepper, tabs};

pub use layout::{frame, dock_splitter, popup_scroll_surface, resizable_panels, scroll_container, scrollbar, split_view};
pub use layout::{ScrollbarAutoHideActivate, ScrollbarPlacement, ScrollbarVisibility};

pub use range::{progress, slider};

#[cfg(all(test, feature = "test-support"))]
mod event_callback_tests;
