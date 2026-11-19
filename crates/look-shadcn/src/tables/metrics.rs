//! Metric tables derived from the same paint helpers and SDK scales used by rendering.
//! Inspection DTOs and display formatting remain in the companion crate.

pub(crate) mod helpers;
pub use crate::controls::accordion::{AccordionMetricTable, resolve_accordion_metrics};
pub use crate::elements::badge::{BadgeMetricTable, resolve_badge_metrics};
pub use crate::controls::button::{ButtonMetricTable, resolve_button_metrics, resolve_button_metrics_with_stylesheet};
pub use crate::controls::card::{CardMetricTable, resolve_card_metrics};
pub use crate::controls::context_menu::{ContextMenuMetricTable, resolve_context_menu_metrics};
pub use crate::controls::control_group::{ControlGroupMetricTable, resolve_control_group_metrics};
pub use crate::controls::floating_menu::{FloatingMenuMetricTable, resolve_floating_menu_metrics};
pub use crate::controls::overlay_window::{OverlayWindowMetricTable, resolve_overlay_window_metrics};
pub use crate::controls::pager::{PagerMetricTable, resolve_pager_metrics};
pub use crate::controls::progress::{ProgressMetricTable, resolve_progress_metrics, resolve_progress_metrics_for_size};
pub use crate::controls::radio::{RadioButtonMetricTable, resolve_radio_button_metrics};
pub use crate::controls::resizable_panels::{ResizablePanelsMetricTable, resolve_resizable_panels_metrics};
pub use crate::controls::scrollbar::{ScrollbarMetricTable, resolve_scrollbar_metrics, resolve_scrollbar_metrics_for_size};
pub use crate::controls::sidebar::{SidebarMetricTable, resolve_sidebar_metrics};
pub use crate::controls::slider::{
    resolve_slider_metrics_for_size, SliderMetricTable, resolve_slider_metrics_with_stylesheet, resolve_slider_metrics,
};
pub use crate::controls::split_view::{
    SplitViewMetricTable, resolve_split_view_metrics, resolve_split_view_metrics_with_stylesheet,
};
pub use crate::controls::stepper::{StepperMetricTable, resolve_stepper_metrics, resolve_stepper_metrics_for_size};
pub use crate::controls::switch::{SwitchMetricTable, resolve_switch_metrics, resolve_switch_metrics_with_stylesheet};
pub use crate::controls::table::{TableMetricTable, resolve_table_metrics};
pub use crate::controls::tabs::{TabsMetricTable, resolve_tabs_metrics, resolve_tabs_metrics_with_stylesheet};
pub(crate) use crate::controls::tabs::resolve_common_tabs_geometry;
pub use crate::controls::textfield::{TextFieldMetricTable, resolve_textfield_metrics};
pub use crate::controls::toolbar_item::{ToolbarMetricTable, resolve_toolbar_metrics};
pub use crate::controls::tree_view::{TreeViewMetricTable, resolve_tree_view_metrics};

#[cfg(feature = "inspect")]
pub(crate) use crate::controls::radio::resolve_radio_button_metrics_with_stylesheet;
