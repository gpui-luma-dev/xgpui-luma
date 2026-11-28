mod control;
mod macros;
mod math;
mod model;
mod template;
pub(crate) mod theme;

pub use control::{ResizablePanels, ResizablePanelsEvent, ResizablePanelsHandleDrag};
pub use model::{
    PanelHideMode, PanelId, PanelLayoutState, PanelRender, PanelSize, ResizeCollapseBehavior, ResizeCollapseDirection,
    ResizeCollapseMode, ResizeHandleMetrics, ResizeHandleVisibility, ResizablePanelSpec, ResizablePanelsBuilder,
    ResizablePanelsModel, ResizablePanelsOrientation, ResizablePanelsRenderModel, render_pane,
};
pub use math::{effective_handle_lane_px, handle_hit_target_main_axis_px, handle_layout_main_axis_px};
pub use template::{
    ResizablePanelsTemplate, ResizablePanelsTemplateModifier, ThemedResizablePanelsTemplate,
    default_resizable_panels_template, render_resizable_panels_surface, render_resizable_panels_handle,
};
pub use theme::{ResizablePanelsTheme, default_resizable_panels_theme};

mod callback;
pub use callback::ResizablePanelsEventExt;

pub use model::ResizablePanelsMotion;
pub use super::styled::{StyledResizablePanelsTheme, resolve_resizable_panels};
