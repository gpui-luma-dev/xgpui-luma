use std::sync::{Arc, OnceLock};

pub trait StepperTheme: Send + Sync {
    /// The stepper's style (#433): the binding the template resolves (see
    /// [`resolve_stepper`](super::resolve_stepper)) and draws from.
    fn style(&self) -> crate::style::StyleBinding {
        crate::style::StyleBinding::sdk("stepper")
    }
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
