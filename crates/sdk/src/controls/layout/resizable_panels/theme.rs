use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::{InteractionLayer, InteractionState, ThemeTokens};

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

#[derive(Clone, Debug, Default)]
pub struct DefaultResizablePanelsTheme {
    tokens: ThemeTokens,
}

pub fn default_resizable_panels_theme() -> Arc<dyn ResizablePanelsTheme> {
    static THEME: OnceLock<Arc<dyn ResizablePanelsTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultResizablePanelsTheme::default())).clone()
}

impl DefaultResizablePanelsTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl ResizablePanelsTheme for DefaultResizablePanelsTheme {
    fn resize_handle_metrics(&self) -> super::ResizeHandleMetrics {
        super::ResizeHandleMetrics::from_stylesheet(&self.tokens.stylesheet)
    }

    fn resolve(&self, state: InteractionState) -> ResizablePanelsLook {
        let palette = self.tokens.stylesheet.recipes.palette("resizable_panels", &self.tokens.palette);
        let disabled = state.layer() == InteractionLayer::Disabled;
        let geometry = self.tokens.stylesheet.resizable_panels.resolve_geometry(Default::default());

        ResizablePanelsLook {
            border: palette.border.default,
            border_width: geometry.border_width,
            divider_width: geometry.divider_width,
            divider: if disabled {
                palette.state.disabled.background
            } else {
                palette.border.strong
            },
            grip: if disabled {
                palette.state.disabled.foreground
            } else {
                palette.border.strong
            },
            grip_emphasis: if disabled {
                palette.state.disabled.foreground
            } else {
                palette.state.hover.background
            },
            disabled_opacity: if disabled { geometry.disabled_opacity } else { 1.0 },
        }
    }
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
