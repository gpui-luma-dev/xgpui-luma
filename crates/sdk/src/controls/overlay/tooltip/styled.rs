//! The tooltip read from `.lss` (issue #285): `tooltip.lss`.

use crate::style::PartRead;

use super::theme::{TooltipLook, TooltipPlacementGeometry, TooltipTheme};
use crate::style::StyleBinding;

/// The tooltip for `binding` (the `tooltip` control).
pub fn styled_tooltip_look(binding: &StyleBinding) -> TooltipLook {
    let resolved = binding.resolve(&[], 1.0);
    let root = resolved.root();
    TooltipLook {
        placement: TooltipPlacementGeometry {
            gap: root.number("target_gap"),
            margin: root.number("window_margin"),
            pointer_size: root.number("pointer_size"),
            pointer_inset: root.number("pointer_inset"),
        },
        shortcut_gap: root.number("shortcut_gap"),
        shortcut_font_size: root.number("shortcut_font_size"),
        shortcut_opacity: root.number("shortcut_opacity"),
        background: root.color("background"),
        foreground: root.color("foreground"),
        padding: root.number("padding_x"),
        padding_y: root.number("padding_y"),
        radius: root.number("radius"),
        max_width: root.number("max_width"),
        text_size: root.font_size().unwrap_or(14.0),
        line_height: root.line_height().unwrap_or(20.0),
        shadow: root.box_shadow(),
    }
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
    fn resolve(&self) -> TooltipLook {
        styled_tooltip_look(&self.binding)
    }
}
