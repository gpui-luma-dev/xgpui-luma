//! The toolbar shell read from `.lss` (issue #285): `toolbar.lss`.

use std::sync::Arc;

use crate::style::PartRead;
use xgpui_luma_style::{PartId, Resolved};

#[allow(deprecated)]
use super::theme::ToolbarLook;
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

/// The toolbar for `binding` (the `toolbar` control), enabled or not, in `variant`.
#[deprecated(note = "templates draw from the resolved style; use `resolve_toolbar` (#433)")]
#[allow(deprecated)]
pub fn styled_toolbar_look(binding: &StyleBinding, enabled: bool, variant: ToolbarVariant) -> ToolbarLook {
    ToolbarLook::from_resolved(&resolve_toolbar(binding, enabled, variant))
}

#[allow(deprecated)]
impl ToolbarLook {
    /// The look `resolved` (see [`resolve_toolbar`]) describes: the compatibility path for code that still reads a
    /// `ToolbarLook` (#433).
    pub fn from_resolved(resolved: &Resolved) -> Self {
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
