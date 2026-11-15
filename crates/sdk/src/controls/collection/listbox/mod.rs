//! Keyed collection state and input binding for host-composed selectable lists.
//!
//! The shared renderer composes windowing, rows, input, scrolling, drag/drop,
//! and motion over persistent SDK state. Looks provide surface/row templates
//! and hosts provide content. Named callbacks deliver committed event batches;
//! hosts can also handle the returned events. This module never chooses a
//! look's appearance or depends on a particular look.

mod binding;
mod callback;
mod control;
mod drag_scroll;
mod drag;
pub use drag::{ListBoxDragDrop, ListBoxDropProposal, ListBoxDragDropBinding};
mod item_template;
mod layout;
mod render;
pub use render::{ListBoxRenderGeometry, ListBoxRenderBindings, render_listbox};
mod template;
pub use template::{ListBoxTemplate, ListBoxRenderModel, ListBoxRowRenderModel};
mod render_drag;
pub use render_drag::ListBoxDragDropRenderBindings;
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
