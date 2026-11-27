//! The overlay window read from `.lss` (issue #285): `overlay_window.lss`.

use crate::style::PartRead;

use super::OverlayWindowMode;
use super::theme::{OverlayWindowLook, OverlayWindowTheme};
use crate::style::StyleBinding;

/// The overlay window for `binding` (the `overlay_window` control) in `mode`.
pub fn styled_overlay_window_look(binding: &StyleBinding, mode: OverlayWindowMode) -> OverlayWindowLook {
    let attrs: &[(&str, &str)] = match mode {
        OverlayWindowMode::Modal => &[("variant", "modal")],
        _ => &[],
    };
    let resolved = binding.resolve_for(attrs, &[], 1.0);
    let root = resolved.root();
    OverlayWindowLook {
        viewport_padding: root.number("viewport_padding"),
        window_margin: root.number("window_margin"),
        drag_header_height: root.number("drag_header_height"),
        border_width: root.number("border_width"),
        background: root.color("background"),
        border: root.color("border"),
        foreground: root.color("foreground"),
        backdrop_background: root.color("backdrop"),
        shadow: root.box_shadow(),
        radius: root.number("radius"),
        padding: root.number("padding"),
        min_width: root.number("min_width"),
        max_width: root.number("max_width"),
        estimated_height: root.number("estimated_height"),
        body: root.typography(14.0, 20.0),
        font_family: root.font_family_name(),
    }
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
    fn resolve(&self, mode: OverlayWindowMode) -> OverlayWindowLook {
        styled_overlay_window_look(&self.binding, mode)
    }
}
