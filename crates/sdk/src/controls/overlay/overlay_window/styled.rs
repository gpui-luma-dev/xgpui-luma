//! The overlay window read from `.lss` (issue #285): `overlay_window.lss`.

use std::sync::Arc;

use super::OverlayWindowMode;
#[allow(deprecated)]
use super::theme::OverlayWindowLook;
use super::theme::OverlayWindowTheme;
use crate::style::StyleBinding;

/// The overlay window's style for `binding` (the `overlay_window` control) in `mode` (#433): the control places and
/// draws the window from its root.
pub fn resolve_overlay_window(binding: &StyleBinding, mode: OverlayWindowMode) -> Arc<xgpui_luma_style::Resolved> {
    let attrs: &[(&str, &str)] = match mode {
        OverlayWindowMode::Modal => &[("variant", "modal")],
        _ => &[],
    };
    binding.resolve_for(attrs, &[], 1.0)
}

/// The overlay window for `binding` (the `overlay_window` control) in `mode`.
#[deprecated(note = "the template draws from the resolved style; use `resolve_overlay_window` (#433)")]
#[allow(deprecated)]
pub fn styled_overlay_window_look(binding: &StyleBinding, mode: OverlayWindowMode) -> OverlayWindowLook {
    OverlayWindowLook::from_resolved(&resolve_overlay_window(binding, mode))
}

/// An overlay-window theme read from `.lss`, resolved on every presentation.
#[derive(Clone)]
pub struct StyledOverlayWindowTheme {
    binding: StyleBinding,
}

impl StyledOverlayWindowTheme {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self { binding: StyleBinding::sdk("overlay_window") }
    }
}

impl OverlayWindowTheme for StyledOverlayWindowTheme {
    fn style(&self) -> StyleBinding {
        self.binding.clone()
    }
}
