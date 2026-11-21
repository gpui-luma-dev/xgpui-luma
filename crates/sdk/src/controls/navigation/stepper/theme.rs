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

crate::theme::stylesheet::geometry::geometry_family!(
    StepperStylesheet,
    StepperGeometryStylesheet,
    StepperGeometry,
    ResolvedStepperGeometry,
    "stepper",
    [
        step_badge_size,
        track_thickness,
        side_label_width,
        label_width,
        content_gap,
        label_gap,
        side_label_gap,
        content_height,
        font_size,
        line_height,
        glyph_scale,
        number_scale,
        border_width
    ]
);

impl StepperStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: StepperGeometryStylesheet {
                side_label_width: Some(120.0),
                label_width: Some(48.0),
                content_gap: Some(16.0),
                label_gap: Some(6.0),
                side_label_gap: Some(10.0),
                content_height: Some(120.0),
                font_size: Some(12.0),
                line_height: Some(16.0),
                glyph_scale: Some(0.45),
                number_scale: Some(0.38),
                border_width: Some(1.0),
                step_badge_size: Some(32.0),
                track_thickness: Some(2.0),
            },
            sizes: Default::default(),
        }
    }
}
