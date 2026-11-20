//! Optional inspection data and formatting for Shadcn looks.
//!
//! Enable the `inspect` feature to use [`ShadcnInspect`]. Resolution stays in
//! the runtime tables so inspection and rendered controls share the same values.

mod controls;
mod look;
mod format;

#[cfg(test)]
mod test_support;

pub use crate::{ColorSource, MetricSource, ResolvedColor, ResolvedMetric, ResolvedTypography, TypographySource};
pub use crate::{ShadcnButtonStyle, ShadcnTextFieldStyle};
pub use look::ShadcnInspect;
pub use format::{
    format_css_style_ref, format_inspect_css_key, format_inspect_metric_provenance, format_inspect_metric_source,
    format_inspect_provenance, format_metric_px,
};

pub use controls::{
    AccordionInspectMetrics, AccordionTriggerInspectPalette, AutocompleteChromeInspectPalette, BadgeInspectMetrics,
    BadgeInspectPalette, ButtonInspectElevation, ButtonInspectElevationLayer, ButtonInspectMetrics,
    ButtonInspectPalette, ButtonInspectTypography, CardInspectMetrics, ContextMenuInspectMetrics,
    ControlGroupInspectMetrics, ControlGroupListInspectPalette, FloatingMenuInspectMetrics, ListBoxRowInspectPalette,
    TableInspectMetrics, SidebarContainerInspectPalette, SidebarInspectMetrics, SidebarItemInspectPalette,
    OverlayWindowInspectMetrics, PagerInspectMetrics, ProgressInspectMetrics, ProgressInspectPalette,
    StepperInspectMetrics, StepperInspectPalette, ResizablePanelsInspectMetrics, ResizablePanelsInspectPalette,
    ScrollbarInspectMetrics, SelectorInspectMetrics, SliderInspectMetrics, SplitViewInspectMetrics,
    SplitViewInspectPalette, TabsInspectMetrics, TextFieldInspectMetrics, TextFieldInspectPalette,
    ToolbarInspectMetrics, TreeViewInspectMetrics, TreeViewRowInspectPalette, ColorChromeInspectSection,
    ColorChromeProfile, COLOR_ARC_CHROME_PROFILES, COLOR_FIELD_CHROME_PROFILES, COLOR_RING_CHROME_PROFILES,
    COLOR_SLIDER_CHROME_PROFILES, inspect_accordion_trigger_color_palette, inspect_autocomplete_chrome_color_palette,
    inspect_badge_color_palette, inspect_badge_metrics, inspect_button_color_palette, inspect_button_elevation,
    inspect_button_metrics, inspect_button_typography, inspect_button_typography_for_size,
    inspect_control_group_list_color_palette, inspect_floating_menu_metrics, inspect_listbox_row_color_palette,
    inspect_sidebar_container_color_palette, inspect_sidebar_item_color_palette, inspect_progress_color_palette,
    inspect_progress_metrics, inspect_stepper_color_palette, inspect_stepper_metrics,
    inspect_resizable_panels_color_palette, inspect_resizable_panels_metrics, inspect_selector_metrics,
    inspect_split_view_color_palette, inspect_split_view_metrics, inspect_textfield_color_palette,
    inspect_tree_view_row_color_palette, inspect_color_chrome_sections,
};
