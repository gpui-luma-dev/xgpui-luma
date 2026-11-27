//! The toolbar shell read from `.lss` (issue #285): `toolbar.lss`.

use crate::style::PartRead;
use xgpui_luma_style::PartId;

use super::theme::{ToolbarLook, ToolbarTheme, ToolbarVariant};
use crate::style::StyleBinding;

/// The toolbar for `binding` (the `toolbar` control), enabled or not, in `variant`.
pub fn styled_toolbar_look(binding: &StyleBinding, enabled: bool, variant: ToolbarVariant) -> ToolbarLook {
    let attrs: &[(&str, &str)] = match variant {
        ToolbarVariant::Outline => &[],
        ToolbarVariant::Ghost => &[("variant", "ghost")],
    };
    let resolved = binding.resolve_for(attrs, &[("disabled", !enabled)], 1.0);
    let root = resolved.root();
    ToolbarLook {
        border_width: root.number("border_width"),
        separator_width: root.number("separator_width"),
        background: root.color("background"),
        border: root.color("border"),
        separator: resolved.part(PartId::new("separator")).color("background"),
        radius: root.number("radius"),
        padding_x: root.number("padding_x"),
        padding_y: root.number("padding_y"),
        gap: root.number("gap"),
        separator_height: root.number("separator_height"),
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
