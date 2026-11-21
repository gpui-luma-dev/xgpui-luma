use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla, SharedString};

use crate::theme::LumaTextStyle;

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

pub fn default_overlay_window_theme() -> Arc<dyn OverlayWindowTheme> {
    static THEME: OnceLock<Arc<dyn OverlayWindowTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledOverlayWindowTheme::sdk())).clone()
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
