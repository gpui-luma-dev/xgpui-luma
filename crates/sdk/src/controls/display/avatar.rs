//! The avatar read from `.lss` (issue #285): `avatar.lss`. Looks render the element from [`AvatarLook`].

use gpui::{Hsla, SharedString};
use xgpui_luma_style::{PartView, Value};

use crate::style::StyleBinding;

/// An avatar's colors, size and fallback text.
#[derive(Clone, Debug, PartialEq)]
pub struct AvatarLook {
    pub background: Hsla,
    pub foreground: Hsla,
    pub diameter: f32,
    pub radius: f32,
    pub icon_size: f32,
    pub font_size: f32,
    pub two_letter_font_size: f32,
    pub line_height: f32,
    pub font_weight: gpui::FontWeight,
    pub font_family: SharedString,
}

fn number(part: PartView<'_>, name: &str) -> f32 {
    part.value(name).and_then(Value::as_number).unwrap_or(0.0)
}

/// The avatar's `.lss` values for one binding (the `avatar` control, with the look's variant and size).
#[derive(Clone)]
pub struct StyledAvatar {
    binding: StyleBinding,
}

impl StyledAvatar {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self::new(StyleBinding::sdk("avatar"))
    }

    pub fn look(&self) -> AvatarLook {
        let resolved = self.binding.resolve(&[], 1.0);
        let root = resolved.root();
        let color = |name: &str| {
            root.value(name).and_then(Value::as_color).map(Hsla::from).unwrap_or_else(gpui::transparent_black)
        };
        AvatarLook {
            background: color("background"),
            foreground: color("foreground"),
            diameter: number(root, "height"),
            radius: number(root, "radius"),
            icon_size: number(root, "icon_size"),
            font_size: root.font_size().unwrap_or(16.0),
            two_letter_font_size: number(root, "two_letter_font_size"),
            line_height: root.line_height().unwrap_or(16.0),
            font_weight: gpui::FontWeight(number(root, "font_weight").max(100.0)),
            font_family: root.font_family().unwrap_or("").to_string().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sdk_avatar_is_a_filled_circle() {
        let look = StyledAvatar::sdk().look();
        assert!(look.diameter > 0.0 && look.radius >= look.diameter / 2.0);
        assert!(look.background.a > 0.0 && look.font_size > 0.0);
    }
}
