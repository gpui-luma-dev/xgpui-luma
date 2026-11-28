//! Color-control chrome read from `.lss` (issue #285): `color_chrome.lss`.

use crate::style::PartRead;
use gpui::Hsla;

use crate::style::StyleBinding;

/// The color controls' edge and panel fill.
#[deprecated(note = "read the chrome from `StyledColorChrome::resolve` (#433)")]
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

    /// The chrome's style (#433): read its root's `border` and `background`.
    pub fn resolve(&self) -> std::sync::Arc<xgpui_luma_style::Resolved> {
        self.binding.resolve(&[], 1.0)
    }

    #[deprecated(note = "read the chrome from `resolve()` (#433)")]
    #[allow(deprecated)]
    pub fn look(&self) -> ColorChromeLook {
        let resolved = self.resolve();
        let root = resolved.root();
        ColorChromeLook { border: root.color("border"), background: root.color("background") }
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;

    #[test]
    fn sdk_chrome_has_an_edge_and_a_fill() {
        let look = StyledColorChrome::sdk().look();
        assert!(look.border.a > 0.0 && look.background.a > 0.0);
    }
}
