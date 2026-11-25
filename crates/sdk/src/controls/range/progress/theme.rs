use std::sync::{Arc, OnceLock};

use gpui::Hsla;

/// Progress geometry as its `.lss` resolves it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ProgressGeometry {
    pub track_height: f32,
    pub size: f32,
    pub stroke_width: f32,
    pub thumb_size: f32,
    pub indeterminate_fraction: f32,
    pub shimmer_soft_opacity: f32,
    pub shimmer_mid_opacity: f32,
    pub shimmer_mid_offset: f32,
    pub shimmer_mid_width: f32,
    pub shimmer_core_offset: f32,
    pub shimmer_core_width: f32,
}

#[derive(Clone, Debug)]
pub struct ProgressLook {
    /// Resolved sweep and shimmer appearance, independent of phase/timing.
    pub geometry: crate::theme::stylesheet::ProgressGeometry,
    pub track_color: Hsla,
    /// The track's outline, drawn over the track and the filled amount (a linear bar). A zero width draws none.
    pub track_border: Hsla,
    pub track_border_width: f32,
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

pub fn default_progress_theme() -> Arc<dyn ProgressTheme> {
    static THEME: OnceLock<Arc<dyn ProgressTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledProgressTheme::sdk())).clone()
}
