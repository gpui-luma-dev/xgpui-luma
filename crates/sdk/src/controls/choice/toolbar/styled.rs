//! The toolbar shell read from `.lss` (issue #285): `toolbar.lss`.

use gpui::Hsla;
use xgpui_luma_style::{PartId, PartView, Value};

use super::theme::{ToolbarLook, ToolbarTheme, ToolbarVariant};
use crate::style::StyleBinding;

fn color(part: PartView<'_>, name: &str) -> Hsla {
    part.value(name).and_then(Value::as_color).map(Hsla::from).unwrap_or_else(gpui::transparent_black)
}

fn number(part: PartView<'_>, name: &str) -> f32 {
    part.value(name).and_then(Value::as_number).unwrap_or(0.0)
}

/// The toolbar for `binding` (the `toolbar` control), enabled or not, in `variant`.
pub fn styled_toolbar_look(binding: &StyleBinding, enabled: bool, variant: ToolbarVariant) -> ToolbarLook {
    let attrs: &[(&str, &str)] = match variant {
        ToolbarVariant::Outline => &[],
        ToolbarVariant::Ghost => &[("variant", "ghost")],
    };
    let resolved = binding.resolve_for(attrs, &[("disabled", !enabled)], 1.0);
    let root = resolved.root();
    ToolbarLook {
        border_width: number(root, "border_width"),
        separator_width: number(root, "separator_width"),
        background: color(root, "background"),
        border: color(root, "border"),
        separator: color(resolved.part(PartId::new("separator")), "background"),
        radius: number(root, "radius"),
        padding_x: number(root, "padding_x"),
        padding_y: number(root, "padding_y"),
        gap: number(root, "gap"),
        separator_height: number(root, "separator_height"),
    }
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
    fn resolve(&self, enabled: bool, variant: ToolbarVariant) -> ToolbarLook {
        styled_toolbar_look(&self.binding, enabled, variant)
    }
}
