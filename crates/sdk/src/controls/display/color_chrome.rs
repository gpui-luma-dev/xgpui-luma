//! Color-control chrome read from `.lss` (issue #285): `color_chrome.lss`.

use crate::style::StyleBinding;

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
}

#[cfg(test)]
mod tests {}
