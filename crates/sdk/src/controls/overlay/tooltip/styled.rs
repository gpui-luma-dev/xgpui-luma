//! The tooltip read from `.lss` (issue #285): `tooltip.lss`.

use gpui::Hsla;
use xgpui_luma_style::{PartView, Value, box_shadows};

use super::theme::{TooltipLook, TooltipPlacementGeometry, TooltipTheme};
use crate::style::StyleBinding;

fn color(part: PartView<'_>, name: &str) -> Hsla {
    part.value(name).and_then(Value::as_color).map(Hsla::from).unwrap_or_else(gpui::transparent_black)
}

fn number(part: PartView<'_>, name: &str) -> f32 {
    part.value(name).and_then(Value::as_number).unwrap_or(0.0)
}

/// The tooltip for `binding` (the `tooltip` control).
pub fn styled_tooltip_look(binding: &StyleBinding) -> TooltipLook {
    let resolved = binding.resolve(&[], 1.0);
    let root = resolved.root();
    TooltipLook {
        placement: TooltipPlacementGeometry {
            gap: number(root, "target_gap"),
            margin: number(root, "window_margin"),
            pointer_size: number(root, "pointer_size"),
            pointer_inset: number(root, "pointer_inset"),
        },
        shortcut_gap: number(root, "shortcut_gap"),
        shortcut_font_size: number(root, "shortcut_font_size"),
        shortcut_opacity: number(root, "shortcut_opacity"),
        background: color(root, "background"),
        foreground: color(root, "foreground"),
        padding: number(root, "padding_x"),
        padding_y: number(root, "padding_y"),
        radius: number(root, "radius"),
        max_width: number(root, "max_width"),
        text_size: root.font_size().unwrap_or(14.0),
        line_height: root.line_height().unwrap_or(20.0),
        shadow: root.shadow().map(|layers| box_shadows(&layers)).unwrap_or_default(),
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
