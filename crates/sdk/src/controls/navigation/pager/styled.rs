//! The pager read from `.lss` (issue #285): `pager.lss`. Page buttons are drawn by a button template the
//! look supplies (its `.lss` button).

use std::sync::Arc;

use super::PagerStyle;

use super::theme::PagerTheme;
use crate::controls::button::{ButtonTemplate, default_button_template};
use crate::style::StyleBinding;

/// Supplies the page buttons' template for a pager's resolved root (#433).
pub type PagerButtonSource = Arc<dyn Fn(xgpui_luma_style::PartView<'_>) -> Arc<dyn ButtonTemplate<()>> + Send + Sync>;

/// The pager's style for `binding` (the `pager` control), enabled or not, in `style` (#433): the template draws
/// from its root.
pub fn resolve_pager(
    binding: &StyleBinding,
    enabled: bool,
    style: PagerStyle,
) -> std::sync::Arc<xgpui_luma_style::Resolved> {
    let attrs: &[(&str, &str)] = match style {
        PagerStyle::Minimal => &[("variant", "minimal")],
        _ => &[],
    };
    binding.resolve_for(attrs, &[("disabled", !enabled)], 1.0)
}

/// A pager theme read from `.lss`, resolved on every render.
#[derive(Clone)]
pub struct StyledPagerTheme {
    binding: StyleBinding,
    buttons: PagerButtonSource,
}

impl StyledPagerTheme {
    /// The `pager` control from `binding`, with page buttons from `buttons`.
    pub fn new(binding: StyleBinding, buttons: PagerButtonSource) -> Self {
        Self { binding, buttons }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self { binding: StyleBinding::sdk("pager"), buttons: Arc::new(|_| default_button_template()) }
    }
}

impl PagerTheme for StyledPagerTheme {
    fn style(&self) -> StyleBinding {
        self.binding.clone()
    }

    fn button_template(&self, pager: xgpui_luma_style::PartView<'_>) -> Arc<dyn ButtonTemplate<()>> {
        (self.buttons)(pager)
    }
}
