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
    /// Shared visual preset; custom looks resolve it through their stylesheet.
    fn resize_handle_metrics(&self, size: super::ResizeHandleSize) -> super::ResizeHandleMetrics {
        size.metrics()
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
    fn resize_handle_metrics(&self, size: super::ResizeHandleSize) -> super::ResizeHandleMetrics {
        size.metrics_with_stylesheet(&self.tokens.stylesheet)
    }

    fn resolve(&self, state: InteractionState) -> ResizablePanelsLook {
        let palette = self.tokens.stylesheet.recipes.palette("resizable_panels", &self.tokens.palette);
        let disabled = state.layer() == InteractionLayer::Disabled;
        let geometry = self.tokens.stylesheet.resizable_panels.resolve_geometry_for("md", Default::default());

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
                ..Default::default()
            },
            sizes: [("sm", 8.0, 12.0, 4.0, 36.0), ("md", 12.0, 14.0, 6.0, 40.0), ("lg", 18.0, 20.0, 8.0, 44.0)]
                .into_iter()
                .map(|(key, lane, hit, cross, main)| {
                    (
                        key.into(),
                        ResizablePanelsGeometryStylesheet {
                            lane_width: Some(lane),
                            hit_target: Some(hit),
                            grip_cross_axis: Some(cross),
                            grip_main_axis: Some(main),
                            ..Default::default()
                        },
                    )
                })
                .collect(),
        }
    }
}
