//! Metric tables derived from the same paint helpers and SDK scales used by rendering.
//! Inspection DTOs and display formatting remain in the companion crate.

pub(crate) mod helpers;
pub use crate::controls::accordion::{AccordionMetricTable};
pub use crate::elements::badge::{BadgeMetricTable, resolve_badge_metrics};
pub use crate::controls::button::{ButtonMetricTable, resolve_button_metrics_with_stylesheet};
pub use crate::controls::card::{CardMetricTable};
pub use crate::controls::context_menu::{ContextMenuMetricTable};
pub use crate::controls::control_group::{ControlGroupMetricTable};
pub use crate::controls::floating_menu::{FloatingMenuMetricTable, resolve_floating_menu_metrics};
pub use crate::controls::overlay_window::{OverlayWindowMetricTable};
pub use crate::controls::pager::{PagerMetricTable};
pub use crate::controls::progress::{ProgressMetricTable, resolve_progress_metrics, resolve_progress_metrics_for_size};
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
pub use crate::controls::table::{TableMetricTable};
pub use crate::controls::tabs::{TabsMetricTable};
pub use crate::controls::textfield::{TextFieldMetricTable};
pub use crate::controls::toolbar_item::{ToolbarMetricTable};
pub use crate::controls::tree_view::{TreeViewMetricTable};
