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
