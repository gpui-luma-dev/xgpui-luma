//! Keyed collection state and input binding for host-composed selectable lists.
//!
//! The borrowed builder composes windowing, rows, input, drag/drop, scrolling,
//! and motion over persistent SDK state. Hosts supply item content; looks supply
//! chrome through templates. Named callbacks deliver committed event batches;
//! hosts can also handle returned events. This module never chooses a look's
//! appearance or depends on a particular look.

mod builder;
pub use builder::ListBoxBuilder;
mod template;
mod styled;
pub use styled::{ListBoxRowLook, ListBoxSurfaceLook, StyledListBox};
pub use template::{ListBoxTemplate, ListBoxRenderModel, ListBoxRowRenderModel, ListBoxDragLook};
mod binding;
mod callback;
mod control;
mod drag_scroll;
mod drag;
pub use drag::{ListBoxDragDrop, ListBoxDropProposal, ListBoxDragDropBinding};
mod item_template;
mod layout;
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
