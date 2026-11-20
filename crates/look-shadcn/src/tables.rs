//! Shared color, metric, and typography resolution for rendering and inspection.
//!
//! Rule metadata lives in [`crate::stylesheet`] and is keyed by [`crate::embedded_stylesheet`].

pub use crate::controls::accordion::{
    AccordionContentColorTable, resolve_accordion_trigger_colors, AccordionTriggerColorTable,
};
pub use crate::controls::autocomplete::{resolve_autocomplete_chrome_colors, AutocompleteChromeColorTable};
pub use crate::elements::badge::{BadgeColorTable, resolve_badge_colors};
pub use crate::controls::button::{ButtonColorPalette};
pub use crate::controls::control_group::{resolve_control_group_list_colors, ControlGroupListColorTable};
pub use crate::controls::floating_menu::{resolve_floating_menu_colors, FloatingMenuColorTable, GhostTriggerColorTable};
pub use crate::controls::table::{TableRowColorTable, TableSurfaceColorTable};
pub use crate::controls::listbox::{resolve_listbox_row_colors, ListBoxListColorTable, ListBoxRowColorTable};
pub use crate::controls::sidebar::{
    resolve_sidebar_container_colors, resolve_sidebar_item_colors, SidebarBranchColorTable, SidebarContainerColorTable,
    SidebarItemColorTable, SidebarSectionColorTable,
};
pub use crate::controls::progress::{resolve_progress_colors, ProgressColorTable};
pub use crate::controls::stepper::{resolve_stepper_colors, StepperColorTable};
pub use crate::controls::resizable_panels::{resolve_resizable_panels_colors, ResizablePanelsColorTable};
pub use crate::controls::scrollbar::{resolve_scrollbar_colors, ScrollbarColorTable};
pub use crate::controls::slider::{resolve_slider_colors, SliderColorTable};
pub use crate::controls::split_view::{resolve_split_view_colors, SplitViewColorTable};
pub use crate::controls::tabs::{TabsItemColorTable, TabsListColorTable};
pub use crate::controls::textfield::{resolve_textfield_colors, TextFieldColorTable};
pub use crate::controls::tree_view::{resolve_tree_view_row_colors, TreeViewRowColorTable};

/// Shared metric values and source metadata.
pub mod metrics;

pub use crate::controls::button::resolve_button_palette;

pub use crate::controls::sidebar::resolve_sidebar_focus_border;

/// Typed typography shared by rendering and inspection.
pub mod typography;

pub use crate::controls::textfield::resolve_textfield_palette;

mod color_chrome;
pub use color_chrome::{resolve_color_chrome, ColorChromeTable};
