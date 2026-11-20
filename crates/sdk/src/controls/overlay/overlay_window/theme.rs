use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla, SharedString, black};

use crate::theme::{LumaTextStyle, ThemeTokens};

use super::OverlayWindowMode;

#[derive(Clone, Debug)]
pub struct OverlayWindowLook {
    /// Placement inset, clamp margin and draggable header hit height in pixels.
    pub viewport_padding: f32,
    pub window_margin: f32,
    pub drag_header_height: f32,

    /// Shell border thickness in pixels.
    pub border_width: f32,
    pub background: Hsla,
    pub border: Hsla,
    pub foreground: Hsla,
    pub backdrop_background: Hsla,
    pub shadow: Vec<BoxShadow>,
    pub radius: f32,
    pub padding: f32,
    pub min_width: f32,
    pub max_width: f32,
    pub estimated_height: f32,
    pub body: LumaTextStyle,
    pub font_family: SharedString,
}

pub trait OverlayWindowTheme: Send + Sync {
    fn resolve(&self, mode: OverlayWindowMode) -> OverlayWindowLook;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultOverlayWindowTheme {
    tokens: ThemeTokens,
}

pub fn default_overlay_window_theme() -> Arc<dyn OverlayWindowTheme> {
    static THEME: OnceLock<Arc<dyn OverlayWindowTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledOverlayWindowTheme::sdk())).clone()
}

impl DefaultOverlayWindowTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl OverlayWindowTheme for DefaultOverlayWindowTheme {
    fn resolve(&self, mode: OverlayWindowMode) -> OverlayWindowLook {
        let palette = self.tokens.stylesheet.recipes.palette("overlay_window", &self.tokens.palette);
        let metrics = &self.tokens.metrics;
        let typography = &self.tokens.typography;
        let elevation = &self.tokens.elevation;
        let geometry =
            self.tokens
                .stylesheet
                .overlay_window
                .resolve_geometry(crate::theme::stylesheet::OverlayWindowGeometry {
                    padding: metrics.padding_x(),
                    radius: 12.0,
                    font_size: typography.text.body.size,
                    line_height: typography.text.body.line_height,
                    ..Default::default()
                });

        OverlayWindowLook {
            viewport_padding: geometry.viewport_padding,
            window_margin: geometry.window_margin,
            drag_header_height: geometry.drag_header_height,

            border_width: geometry.border_width,
            background: palette.surface.floating.background,
            border: palette.surface.floating.border,
            foreground: palette.surface.floating.foreground,
            backdrop_background: if mode == OverlayWindowMode::Modal {
                Hsla { a: geometry.backdrop_opacity, ..black() }
            } else {
                Hsla { a: 0.0, ..black() }
            },
            shadow: if mode == OverlayWindowMode::Modal {
                elevation.dialog.to_box_shadows()
            } else {
                elevation.overlay_window.to_box_shadows()
            },
            radius: geometry.radius,
            padding: geometry.padding,
            min_width: geometry.min_width,
            max_width: geometry.max_width,
            estimated_height: geometry.estimated_height,
            body: LumaTextStyle { size: geometry.font_size, line_height: geometry.line_height, ..typography.text.body },
            font_family: typography.font.sans.family.clone().into(),
        }
    }
}

crate::theme::stylesheet::geometry::geometry_family!(
    OverlayWindowStylesheet,
    OverlayWindowGeometryStylesheet,
    OverlayWindowGeometry,
    ResolvedOverlayWindowGeometry,
    "overlay_window",
    [
        viewport_padding,
        window_margin,
        drag_header_height,
        padding,
        min_width,
        max_width,
        estimated_height,
        font_size,
        line_height,
        border_width,
        radius
    ],
    unit[backdrop_opacity]
);

impl OverlayWindowStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: OverlayWindowGeometryStylesheet {
                viewport_padding: Some(24.0),
                window_margin: Some(12.0),
                drag_header_height: Some(56.0),
                border_width: Some(1.0),
                backdrop_opacity: Some(0.44),
                min_width: Some(400.0),
                max_width: Some(520.0),
                estimated_height: Some(228.0),
                ..Default::default()
            },
            sizes: Default::default(),
        }
    }
}
