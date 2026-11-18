use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::ThemeTokens;

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

#[derive(Clone, Debug, Default)]
pub struct DefaultDockSplitterTheme {
    tokens: ThemeTokens,
}

pub fn default_dock_splitter_theme() -> Arc<dyn DockSplitterTheme> {
    static THEME: OnceLock<Arc<dyn DockSplitterTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultDockSplitterTheme::default())).clone()
}

impl DefaultDockSplitterTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl DockSplitterTheme for DefaultDockSplitterTheme {
    fn resolve(&self, enabled: bool) -> DockSplitterLook {
        let palette = self.tokens.stylesheet.recipes.palette("dock_splitter", &self.tokens.palette);
        let geometry = self.tokens.stylesheet.dock_splitter.resolve_geometry(Default::default());

        DockSplitterLook {
            line_color: if enabled {
                palette.border.default
            } else {
                palette.state.disabled.foreground
            },
            hover_color: if enabled {
                palette.state.selected.background
            } else {
                palette.state.disabled.foreground
            },
            thumb_color: if enabled {
                palette.state.selected.background
            } else {
                palette.state.disabled.foreground
            },
            hit_target_px: geometry.hit_target,
            visible_line_px: geometry.visible_line,
            geometry,
        }
    }
}

crate::theme::stylesheet::geometry::geometry_family!(
    DockSplitterStylesheet,
    DockSplitterGeometryStylesheet,
    DockSplitterGeometry,
    ResolvedDockSplitterGeometry,
    "dock_splitter",
    [hit_target, visible_line, thumb_width, thumb_height, thumb_radius],
    unit[thumb_rest_opacity]
);

impl DockSplitterStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: DockSplitterGeometryStylesheet {
                thumb_width: Some(4.0),
                thumb_height: Some(36.0),
                thumb_radius: Some(8.0),
                thumb_rest_opacity: Some(0.4),
                hit_target: Some(8.0),
                visible_line: Some(1.0),
            },
            ..Default::default()
        }
    }
}
