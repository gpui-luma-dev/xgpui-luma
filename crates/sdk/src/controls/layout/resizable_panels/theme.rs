use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::InteractionState;

#[derive(Clone, Debug)]
pub struct ResizablePanelsLook {
    pub border: Hsla,
    /// Outer frame thickness in pixels, when the frame is enabled.
    pub border_width: f32,
    /// Painted split divider thickness in pixels; independent of the resize hit target.
    pub divider_width: f32,
    pub divider: Hsla,
    pub grip: Hsla,
    pub grip_emphasis: Hsla,
    pub disabled_opacity: f32,
}

pub trait ResizablePanelsTheme: Send + Sync {
    /// Handle pixels; custom looks resolve their own handle size through a sized stylesheet.
    fn resize_handle_metrics(&self) -> super::ResizeHandleMetrics {
        super::ResizeHandleMetrics::default()
    }

    fn resolve(&self, state: InteractionState) -> ResizablePanelsLook;
}

pub fn default_resizable_panels_theme() -> Arc<dyn ResizablePanelsTheme> {
    static THEME: OnceLock<Arc<dyn ResizablePanelsTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::StyledResizablePanelsTheme::sdk())).clone()
}

crate::theme::stylesheet::geometry::geometry_family!(
    ResizablePanelsStylesheet,
    ResizablePanelsGeometryStylesheet,
    ResizablePanelsGeometry,
    ResolvedResizablePanelsGeometry,
    "resizable_panels",
    [lane_width, hit_target, grip_cross_axis, grip_main_axis, grip_radius, border_width, divider_width],
    unit[disabled_opacity]
);

impl ResizablePanelsStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: ResizablePanelsGeometryStylesheet {
                border_width: Some(1.0),
                divider_width: Some(1.0),
                grip_radius: Some(8.0),
                disabled_opacity: Some(0.45),
                // The SDK's single handle scale (the builder's long-standing default).
                lane_width: Some(8.0),
                hit_target: Some(12.0),
                grip_cross_axis: Some(4.0),
                grip_main_axis: Some(36.0),
            },
            sizes: Default::default(),
        }
    }
}
