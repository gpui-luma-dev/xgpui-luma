//! The slider read from `.lss` (issue #285): `slider.lss`.

use std::sync::Arc;

use crate::style::StyleStates;
use xgpui_luma_style::Resolved;

use super::theme::SliderTheme;
use crate::style::StyleBinding;
use crate::theme::InteractionState;

/// The slider's style for `binding` (the `slider` control) in `state` (#433): templates draw from its root and its
/// `track`, `fill` and `thumb` parts.
pub fn resolve_slider(binding: &StyleBinding, state: InteractionState) -> Arc<Resolved> {
    binding.resolve_states(StyleStates::new(state), 1.0)
}

/// A slider theme read from `.lss`, resolved on every render.
#[derive(Clone)]
pub struct StyledSliderTheme {
    binding: StyleBinding,
}

impl StyledSliderTheme {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self { binding: StyleBinding::sdk("slider") }
    }
}

impl SliderTheme for StyledSliderTheme {
    fn style(&self) -> StyleBinding {
        self.binding.clone()
    }
}
