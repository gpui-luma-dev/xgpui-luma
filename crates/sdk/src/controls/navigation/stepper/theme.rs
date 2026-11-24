use std::sync::{Arc, OnceLock};

use gpui::Hsla;

#[derive(Clone, Debug)]
pub struct StepperLook {
    pub number_weight: gpui::FontWeight,
    /// Resolved decoration and layout defaults shared with templates.
    pub geometry: crate::theme::stylesheet::StepperGeometry,
    pub complete_bg: Hsla,
    pub complete_fg: Hsla,
    pub in_progress_bg: Hsla,
    pub in_progress_fg: Hsla,
    pub incomplete_bg: Hsla,
    pub incomplete_border: Hsla,
    pub incomplete_fg: Hsla,
    pub track_active_color: Hsla,
    pub track_muted_color: Hsla,
    pub step_badge_size: f32,
    pub track_thickness: f32,
}

pub trait StepperTheme: Send + Sync {
    fn resolve(&self, enabled: bool) -> StepperLook;
}

pub fn default_stepper_theme() -> Arc<dyn StepperTheme> {
    static THEME: OnceLock<Arc<dyn StepperTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledStepperTheme::sdk())).clone()
}

/// The stepper's geometry as its `.lss` resolves it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StepperGeometry {
    pub step_badge_size: f32,
    pub track_thickness: f32,
    pub side_label_width: f32,
    pub label_width: f32,
    pub content_gap: f32,
    pub label_gap: f32,
    pub side_label_gap: f32,
    pub content_height: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub glyph_scale: f32,
    pub number_scale: f32,
    pub border_width: f32,
}
