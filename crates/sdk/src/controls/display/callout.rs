//! The callout read from `.lss` (issue #285): `callout.lss`. Looks render the element from [`CalloutLook`].

use gpui::Hsla;
use xgpui_luma_style::{PartView, Value};

use crate::style::StyleBinding;

/// A callout's colors, frame and text size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalloutLook {
    pub background: Hsla,
    pub foreground: Hsla,
    pub radius: f32,
    pub padding: f32,
    pub gap: f32,
    pub icon_size: f32,
    pub font_size: f32,
}

fn number(part: PartView<'_>, name: &str) -> f32 {
    part.value(name).and_then(Value::as_number).unwrap_or(0.0)
}

/// The callout's `.lss` values for one binding (the `callout` control).
#[derive(Clone)]
pub struct StyledCallout {
    binding: StyleBinding,
}

impl StyledCallout {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self::new(StyleBinding::sdk("callout"))
    }

    pub fn look(&self) -> CalloutLook {
        let resolved = self.binding.resolve(&[], 1.0);
        let root = resolved.root();
        let color = |name: &str| {
            root.value(name).and_then(Value::as_color).map(Hsla::from).unwrap_or_else(gpui::transparent_black)
        };
        CalloutLook {
            background: color("background"),
            foreground: color("foreground"),
            radius: number(root, "radius"),
            padding: number(root, "padding_x"),
            gap: number(root, "gap"),
            icon_size: number(root, "icon_size"),
            font_size: root.font_size().unwrap_or(14.0),
        }
    }
}
