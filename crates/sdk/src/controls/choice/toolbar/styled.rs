//! The toolbar shell read from `.lss` (issue #285): `toolbar.lss`.

use std::sync::Arc;

use xgpui_luma_style::Resolved;

use super::theme::{ToolbarTheme, ToolbarVariant};
use crate::style::StyleBinding;

/// The toolbar's style for `binding` (the `toolbar` control), enabled or not, in `variant` (#433): templates draw
/// from its root and its `separator` part.
pub fn resolve_toolbar(binding: &StyleBinding, enabled: bool, variant: ToolbarVariant) -> Arc<Resolved> {
    let attrs: &[(&str, &str)] = match variant {
        ToolbarVariant::Outline => &[],
        ToolbarVariant::Ghost => &[("variant", "ghost")],
    };
    binding.resolve_for(attrs, &[("disabled", !enabled)], 1.0)
}

/// A toolbar theme read from `.lss`, resolved on every render.
#[derive(Clone)]
pub struct StyledToolbarTheme {
    binding: StyleBinding,
}

impl StyledToolbarTheme {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self { binding: StyleBinding::sdk("toolbar") }
    }
}

impl ToolbarTheme for StyledToolbarTheme {
    fn style(&self) -> StyleBinding {
        self.binding.clone()
    }
}
