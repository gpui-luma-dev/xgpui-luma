//! The stepper read from `.lss` (issue #285): `stepper.lss`.

use gpui::Hsla;
use xgpui_luma_style::{PartView, Value};

use super::theme::{StepperLook, StepperTheme};
use crate::style::StyleBinding;
use crate::theme::stylesheet::StepperGeometry;

fn color(part: PartView<'_>, name: &str) -> Hsla {
    part.value(name).and_then(Value::as_color).map(Hsla::from).unwrap_or_else(gpui::transparent_black)
}

fn number(part: PartView<'_>, name: &str) -> f32 {
    part.value(name).and_then(Value::as_number).unwrap_or(0.0)
}

/// The stepper for `binding` (the `stepper` control), enabled or not.
pub fn styled_stepper_look(binding: &StyleBinding, enabled: bool) -> StepperLook {
    let resolved = binding.resolve(&[("disabled", !enabled)], 1.0);
    let root = resolved.root();
    let geometry = StepperGeometry {
        step_badge_size: number(root, "step_badge_size"),
        track_thickness: number(root, "track_thickness"),
        side_label_width: number(root, "side_label_width"),
        label_width: number(root, "label_width"),
        content_gap: number(root, "content_gap"),
        label_gap: number(root, "label_gap"),
        side_label_gap: number(root, "side_label_gap"),
        content_height: number(root, "content_height"),
        font_size: root.font_size().unwrap_or(12.0),
        line_height: root.line_height().unwrap_or(16.0),
        glyph_scale: number(root, "glyph_scale"),
        number_scale: number(root, "number_scale"),
        border_width: number(root, "border_width"),
    };
    StepperLook {
        number_weight: gpui::FontWeight(number(root, "font_weight").max(100.0)),
        complete_bg: color(root, "complete_background"),
        complete_fg: color(root, "complete_foreground"),
        in_progress_bg: color(root, "current_background"),
        in_progress_fg: color(root, "current_foreground"),
        incomplete_bg: color(root, "incomplete_background"),
        incomplete_border: color(root, "incomplete_border"),
        incomplete_fg: color(root, "incomplete_foreground"),
        track_active_color: color(root, "track_active"),
        track_muted_color: color(root, "track_muted"),
        step_badge_size: geometry.step_badge_size,
        track_thickness: geometry.track_thickness,
        geometry,
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
    fn resolve(&self, enabled: bool) -> StepperLook {
        styled_stepper_look(&self.binding, enabled)
    }
}
