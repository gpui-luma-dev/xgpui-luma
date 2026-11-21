//! Color-control chrome read from `.lss` (issue #285): `color_chrome.lss`.

use gpui::Hsla;
use xgpui_luma_style::Value;

use crate::style::StyleBinding;

/// The color controls' edge and panel fill.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorChromeLook {
    pub border: Hsla,
    pub background: Hsla,
}

/// The color chrome's `.lss` values for one binding (the `color_chrome` control).
#[derive(Clone)]
pub struct StyledColorChrome {
    binding: StyleBinding,
}

impl StyledColorChrome {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self::new(StyleBinding::sdk("color_chrome"))
    }

    pub fn look(&self) -> ColorChromeLook {
        let resolved = self.binding.resolve(&[], 1.0);
        let root = resolved.root();
        let color = |name: &str| {
            root.value(name).and_then(Value::as_color).map(Hsla::from).unwrap_or_else(gpui::transparent_black)
        };
        ColorChromeLook { border: color("border"), background: color("background") }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sdk_chrome_has_an_edge_and_a_fill() {
        let look = StyledColorChrome::sdk().look();
        assert!(look.border.a > 0.0 && look.background.a > 0.0);
    }
}
