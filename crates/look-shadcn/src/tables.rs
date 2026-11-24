//! Shared color, metric, and typography resolution for rendering and inspection.
//!
//! Rule metadata lives in [`crate::stylesheet`] and is keyed by [`crate::embedded_stylesheet`].

pub use crate::controls::accordion::{AccordionContentColorTable, AccordionTriggerColorTable};
pub use crate::controls::autocomplete::{AutocompleteChromeColorTable};
pub use crate::elements::badge::{BadgeColorTable};
pub use crate::controls::button::{ButtonColorPalette};
pub use crate::controls::control_group::{ControlGroupListColorTable};
pub use crate::controls::floating_menu::{FloatingMenuColorTable, GhostTriggerColorTable};
pub use crate::controls::table::{TableRowColorTable, TableSurfaceColorTable};
pub use crate::controls::listbox::{ListBoxListColorTable, ListBoxRowColorTable};
pub use crate::controls::progress::{ProgressColorTable};
pub use crate::controls::stepper::{StepperColorTable};
pub use crate::controls::resizable_panels::{ResizablePanelsColorTable};
pub use crate::controls::split_view::{SplitViewColorTable};
pub use crate::controls::tabs::{TabsItemColorTable, TabsListColorTable};
pub use crate::controls::textfield::{TextFieldColorTable};
pub use crate::controls::tree_view::{TreeViewRowColorTable};

/// Shared metric values and source metadata.
pub mod metrics;

/// Typed typography shared by rendering and inspection.
pub mod typography;
