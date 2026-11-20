use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::theme::ThemeTokens;

#[derive(Clone, Debug)]
pub struct ProgressLook {
    /// Resolved sweep and shimmer appearance, independent of phase/timing.
    pub geometry: crate::theme::stylesheet::ProgressGeometry,
    pub track_color: Hsla,
    pub progress_color: Hsla,
    pub thumb_color: Hsla,
    pub track_height: f32,
    pub thumb_size: f32,
    pub size: f32,
    pub stroke_width: f32,
}

pub trait ProgressTheme: Send + Sync {
    fn resolve(&self, enabled: bool) -> ProgressLook;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultProgressTheme {
    tokens: ThemeTokens,
}

pub fn default_progress_theme() -> Arc<dyn ProgressTheme> {
    static THEME: OnceLock<Arc<dyn ProgressTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledProgressTheme::sdk())).clone()
}

impl DefaultProgressTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl ProgressTheme for DefaultProgressTheme {
    fn resolve(&self, enabled: bool) -> ProgressLook {
        let geometry = self
            .tokens
            .stylesheet
            .progress
            .resolve_geometry(crate::theme::stylesheet::ProgressGeometry::default());
        let palette = self.tokens.stylesheet.recipes.palette("progress", &self.tokens.palette);

        let track_color = if enabled {
            palette.surface.subtle.background
        } else {
            palette.state.disabled.background
        };
        let progress_color = if enabled {
            palette.state.selected.background
        } else {
            palette.state.disabled.foreground
        };

        ProgressLook {
            track_color,
            progress_color,
            thumb_color: progress_color,
            track_height: geometry.track_height,
            thumb_size: geometry.thumb_size,
            size: geometry.size,
            stroke_width: geometry.stroke_width,
            geometry,
        }
    }
}

crate::theme::stylesheet::geometry::geometry_family!(
    ProgressStylesheet,
    ProgressGeometryStylesheet,
    ProgressGeometry,
    ResolvedProgressGeometry,
    "progress",
    [track_height, size, stroke_width, thumb_size],
    unit[indeterminate_fraction, shimmer_soft_opacity, shimmer_mid_opacity, shimmer_mid_offset, shimmer_mid_width, shimmer_core_offset, shimmer_core_width]
);

impl ProgressStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: ProgressGeometryStylesheet {
                indeterminate_fraction: Some(0.28),
                shimmer_soft_opacity: Some(0.35),
                shimmer_mid_opacity: Some(0.85),
                shimmer_mid_offset: Some(0.2),
                shimmer_mid_width: Some(0.6),
                shimmer_core_offset: Some(0.35),
                shimmer_core_width: Some(0.3),
                track_height: Some(6.0),
                size: Some(64.0),
                stroke_width: Some(6.0),
                thumb_size: Some(16.0),
            },
            sizes: Default::default(),
        }
    }
}
