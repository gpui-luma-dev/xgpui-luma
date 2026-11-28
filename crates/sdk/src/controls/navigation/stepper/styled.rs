//! The stepper read from `.lss` (issue #285): `stepper.lss`.

use super::theme::StepperTheme;
use crate::style::StyleBinding;

/// The stepper's style for `binding` (the `stepper` control), enabled or not (#433): the template draws from its
/// root.
pub fn resolve_stepper(binding: &StyleBinding, enabled: bool) -> std::sync::Arc<xgpui_luma_style::Resolved> {
    binding.resolve(&[("disabled", !enabled)], 1.0)
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
