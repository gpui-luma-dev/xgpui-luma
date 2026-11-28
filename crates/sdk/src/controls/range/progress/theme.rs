use std::sync::{Arc, OnceLock};

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

pub trait ProgressTheme: Send + Sync {
    /// Progress's style (#433): the binding templates resolve (see [`resolve_progress`](super::resolve_progress))
    /// and draw from.
    fn style(&self) -> crate::style::StyleBinding {
        crate::style::StyleBinding::sdk("progress")
    }
}

pub fn default_progress_theme() -> Arc<dyn ProgressTheme> {
    static THEME: OnceLock<Arc<dyn ProgressTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledProgressTheme::sdk())).clone()
}
