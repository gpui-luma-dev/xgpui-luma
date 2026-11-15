//! Keyed collection state and input binding for host-composed selectable lists.
//!
//! Hosts and looks own content and viewport composition. Named callbacks deliver
//! committed event batches; hosts can also handle the returned events. An
//! optional persistent control combines state and bindings; fixed-item geometry
//! and content-template contracts support look-owned builders. This module never
//! chooses appearance or depends on a particular look.

mod binding;
mod callback;
mod control;
mod drag_scroll;
mod drag;
pub use drag::{ListBoxDragDrop, ListBoxDropProposal, ListBoxDragDropBinding};
mod item_template;
mod layout;
mod render;
pub use render::ListBoxRenderGeometry;
mod model;
mod scroll;
mod virtualization;
mod measured;

pub use virtualization::{ListBoxVirtualization, ListBoxVirtualWindow};
pub use scroll::ListBoxScrollHandle;
pub use control::{ListBoxControl, ListBoxInputHandler, ListBoxRenderParts};
pub use item_template::{ListBoxItemRenderModel, ListBoxItemTemplate, make_listbox_item_template};
pub use layout::{ListBoxFlow, ListBoxLayout};

pub use binding::{ListBoxAxis, ListBoxBinding};
pub use model::{
    ListBoxError, ListBoxEvent, ListBoxInput, ListBoxItemState, ListBoxNavigation, ListBoxSnapshot, ListBoxState,
    ListBoxUpdate, ListBoxVisibleItem, SelectionMode, SelectionPolicy, ListBoxSelectionModifiers,
};

#[cfg(test)]
mod selection_tests;

#[cfg(test)]
mod projection_tests;

pub use control::ListBoxMotion;
