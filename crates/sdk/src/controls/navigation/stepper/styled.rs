//! The stepper read from `.lss` (issue #285): `stepper.lss`.

use crate::style::PartRead;

#[allow(deprecated)]
use super::theme::StepperLook;
use super::theme::StepperTheme;
use crate::style::StyleBinding;
use crate::theme::stylesheet::StepperGeometry;

/// The stepper's style for `binding` (the `stepper` control), enabled or not (#433): the template draws from its
/// root.
pub fn resolve_stepper(binding: &StyleBinding, enabled: bool) -> std::sync::Arc<xgpui_luma_style::Resolved> {
    binding.resolve(&[("disabled", !enabled)], 1.0)
}

/// The stepper for `binding` (the `stepper` control), enabled or not.
#[deprecated(note = "the template draws from the resolved style; use `resolve_stepper` (#433)")]
#[allow(deprecated)]
pub fn styled_stepper_look(binding: &StyleBinding, enabled: bool) -> StepperLook {
    StepperLook::from_resolved(&resolve_stepper(binding, enabled))
}

#[allow(deprecated)]
impl StepperLook {
    /// The look `resolved` (a stepper's resolved style) describes: the compatibility path for code that still
    /// reads a `StepperLook` (#433).
    pub fn from_resolved(resolved: &xgpui_luma_style::Resolved) -> Self {
        let root = resolved.root();
        let geometry = StepperGeometry {
            step_badge_size: root.number("step_badge_size"),
            track_thickness: root.number("track_thickness"),
            side_label_width: root.number("side_label_width"),
            label_width: root.number("label_width"),
            content_gap: root.number("content_gap"),
            label_gap: root.number("label_gap"),
            side_label_gap: root.number("side_label_gap"),
            content_height: root.number("content_height"),
            font_size: root.font_size().unwrap_or(12.0),
            line_height: root.line_height().unwrap_or(16.0),
            glyph_scale: root.number("glyph_scale"),
            number_scale: root.number("number_scale"),
            border_width: root.number("border_width"),
        };
        StepperLook {
            number_weight: root.font_weight(),
            complete_bg: root.color("complete_background"),
            complete_fg: root.color("complete_foreground"),
            in_progress_bg: root.color("current_background"),
            in_progress_fg: root.color("current_foreground"),
            incomplete_bg: root.color("incomplete_background"),
            incomplete_border: root.color("incomplete_border"),
            incomplete_fg: root.color("incomplete_foreground"),
            track_active_color: root.color("track_active"),
            track_muted_color: root.color("track_muted"),
            step_badge_size: geometry.step_badge_size,
            track_thickness: geometry.track_thickness,
            geometry,
        }
    }
}

/// A stepper theme read from `.lss`, resolved on every render.
#[derive(Clone)]
pub struct StyledStepperTheme {
    binding: StyleBinding,
}

impl StyledStepperTheme {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self { binding: StyleBinding::sdk("stepper") }
    }
}

impl StepperTheme for StyledStepperTheme {
    fn style(&self) -> StyleBinding {
        self.binding.clone()
    }
}
