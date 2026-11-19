mod adapters;
pub mod box_model;
pub mod collection;
pub mod common;
pub mod input;
pub mod metrics;
pub mod motion;
pub mod occupation;
pub mod provenance;
pub mod render;
pub mod schema;
pub mod specs;

pub mod color_chrome;
mod components;

pub use render::{layout, render_category_content};
pub use schema::{
    ControlInspectorSpec, InspectorCategory, InspectorPart, InspectorSelection, InspectorStateSpec, InspectorVariant,
    SharedInspectorResolver,
};

pub use adapters::*;
pub use motion::{
    ControlGroupMotionReport, SharedMotionReport, StateMotionReport, TabsMotionReport, accordion_motion_report,
    autocomplete_motion_report, combobox_motion_report, context_menu_motion_report, control_group_motion_source,
    listbox_motion_report, popup_menu_motion_report, progress_motion_report, render_motion_examples,
    resizable_panels_motion_report, search_selector_motion_report, selector_motion_report, sidebar_motion_report,
    slide_panel_motion_report, slider_motion_report, stepper_motion_report, table_motion_report,
    tree_view_motion_report, tree_view_motion_source,
};
