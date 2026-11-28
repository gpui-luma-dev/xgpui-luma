//! The tooltip read from `.lss` (issue #285): `tooltip.lss`.

use std::sync::Arc;

use super::theme::TooltipTheme;
use crate::style::StyleBinding;

/// The tooltip's style for `binding` (the `tooltip` control) (#433): the tooltip draws from its root (see
/// [`TooltipRenderModel::from_resolved`](super::TooltipRenderModel::from_resolved)).
pub fn resolve_tooltip(binding: &StyleBinding) -> Arc<xgpui_luma_style::Resolved> {
    binding.resolve(&[], 1.0)
}

/// A tooltip theme read from `.lss`, resolved on every presentation.
#[derive(Clone)]
pub struct StyledTooltipTheme {
    binding: StyleBinding,
}

impl StyledTooltipTheme {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self { binding: StyleBinding::sdk("tooltip") }
    }
}

impl TooltipTheme for StyledTooltipTheme {
    fn style(&self) -> StyleBinding {
        self.binding.clone()
    }
}
