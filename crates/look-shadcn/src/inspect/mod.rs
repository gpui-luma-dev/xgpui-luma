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
    format_color_source, format_css_style_ref, format_font_weight, format_inspect_css_key,
    format_inspect_metric_provenance, format_inspect_metric_source, format_inspect_provenance,
    format_inspect_typography_provenance, format_inspect_typography_source, format_metric_px, format_typography_px,
};

pub use controls::{
    format_inspect_box_shadow_layer, AccordionContentInspectPalette, AccordionInspectMetrics,
    AccordionTriggerInspectPalette, AutocompleteChromeInspectPalette, AutocompleteInspectMetrics, BadgeInspectMetrics,
    BadgeInspectPalette, ButtonInspectElevation, ButtonInspectElevationLayer, ButtonInspectMetrics,
    ButtonInspectPalette, ButtonInspectTypography, CardInspectMetrics, ContextMenuInspectMetrics,
    ContextMenuInspectPalette, ControlGroupInspectMetrics, ControlGroupListInspectPalette, FloatingMenuInspectMetrics,
    FloatingMenuInspectPalette, ListBoxListInspectPalette, ListBoxRowInspectPalette, TableInspectMetrics,
    TableInspectPalette, TableRowInspectPalette, SidebarContainerInspectPalette, SidebarInspectMetrics,
    SidebarItemInspectPalette, SidebarSectionInspectPalette, OverlayWindowInspectMetrics, OverlayWindowInspectPalette,
    PopupMenuInspectMetrics, PopupMenuInspectPalette, PagerInspectMetrics, PagerShellInspectPalette,
    ProgressInspectMetrics, ProgressInspectPalette, RadioButtonInspectMetrics, RadioButtonInspectPalette,
    StepperInspectMetrics, StepperInspectPalette, ResizablePanelsInspectMetrics, ResizablePanelsInspectPalette,
    ScrollbarInspectMetrics, ScrollbarInspectPalette, SelectorInspectMetrics, SelectorInspectPalette,
    SliderInspectMetrics, SliderInspectPalette, SplitViewInspectMetrics, SplitViewInspectPalette, SwitchInspectMetrics,
    SwitchInspectPalette, TabsInspectMetrics, TabsItemInspectPalette, TabsListInspectPalette, TextFieldInspectMetrics,
    TextFieldInspectPalette, ToolbarInspectMetrics, ToolbarInspectPalette, TreeViewInspectMetrics,
    TreeViewRowInspectPalette, ColorChromeInspectSection, ColorChromeProfile, COLOR_ARC_CHROME_PROFILES,
    COLOR_FIELD_CHROME_PROFILES, COLOR_RING_CHROME_PROFILES, COLOR_SLIDER_CHROME_PROFILES,
    inspect_accordion_content_color_palette, inspect_accordion_metrics, inspect_accordion_metrics_at_scale,
    inspect_accordion_trigger_color_palette, inspect_autocomplete_chrome_color_palette,
    inspect_autocomplete_menu_color_palette, inspect_autocomplete_metrics, inspect_badge_color_palette,
    inspect_badge_metrics, inspect_button_color_palette, inspect_button_elevation, inspect_button_metrics,
    inspect_button_typography, inspect_button_typography_for_size, inspect_card_metrics,
    inspect_context_menu_color_palette, inspect_context_menu_metrics, inspect_control_group_list_color_palette,
    inspect_control_group_metrics, inspect_floating_menu_color_palette, inspect_floating_menu_metrics,
    inspect_table_color_palette, inspect_table_metrics, inspect_table_row_color_palette,
    inspect_listbox_list_color_palette, inspect_listbox_row_color_palette, inspect_sidebar_branch_color_palette,
    inspect_sidebar_container_color_palette, inspect_sidebar_item_color_palette, inspect_sidebar_metrics,
    inspect_sidebar_section_color_palette, inspect_overlay_window_color_palette, inspect_overlay_window_metrics,
    inspect_pager_metrics, inspect_pager_shell_color_palette, inspect_popup_menu_color_palette,
    inspect_popup_menu_metrics, inspect_progress_color_palette, inspect_progress_metrics,
    inspect_stepper_color_palette, inspect_stepper_metrics, inspect_radio_button_color_palette,
    inspect_radio_button_elevation, inspect_radio_button_metrics, inspect_resizable_panels_color_palette,
    inspect_resizable_panels_metrics, inspect_scrollbar_color_palette, inspect_scrollbar_metrics,
    inspect_selector_color_palette, inspect_selector_metrics, inspect_slider_color_palette, inspect_slider_metrics,
    inspect_split_view_color_palette, inspect_split_view_metrics, inspect_switch_color_palette,
    inspect_switch_elevation, inspect_switch_metrics, inspect_tabs_item_color_palette, inspect_tabs_list_color_palette,
    inspect_tabs_metrics, inspect_textarea_color_palette, inspect_textarea_metrics, inspect_textfield_color_palette,
    inspect_textfield_elevation, inspect_textfield_metrics, inspect_toolbar_color_palette, inspect_toolbar_metrics,
    inspect_tree_view_metrics, inspect_tree_view_row_color_palette, inspect_color_chrome_sections,
};

#[cfg(test)]
mod parity_tests;
