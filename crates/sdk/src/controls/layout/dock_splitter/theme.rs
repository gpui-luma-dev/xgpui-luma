use std::sync::{Arc, OnceLock};

use gpui::Hsla;

/// The dock splitter's geometry as its `.lss` resolves it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DockSplitterGeometry {
    pub hit_target: f32,
    pub visible_line: f32,
    pub thumb_width: f32,
    pub thumb_height: f32,
    pub thumb_radius: f32,
    pub thumb_rest_opacity: f32,
}

#[derive(Clone, Debug)]
pub struct DockSplitterLook {
    /// Captured, resolved thumb geometry and resting opacity.
    pub geometry: crate::theme::stylesheet::DockSplitterGeometry,
    pub line_color: Hsla,
    pub hover_color: Hsla,
    pub thumb_color: Hsla,
    pub hit_target_px: f32,
    pub visible_line_px: f32,
}

pub trait DockSplitterTheme: Send + Sync {
    fn resolve(&self, enabled: bool) -> DockSplitterLook;
}

pub fn default_dock_splitter_theme() -> Arc<dyn DockSplitterTheme> {
    static THEME: OnceLock<Arc<dyn DockSplitterTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::StyledDockSplitterTheme::sdk())).clone()
}
