//! The pager read from `.lss` (issue #285): `pager.lss`. Page buttons are drawn by a button template the
//! look supplies (its `.lss` button).

use crate::style::PartRead;
use std::sync::Arc;

use super::PagerStyle;
#[allow(deprecated)]
use super::theme::PagerLook;
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

/// The pager for `binding` (the `pager` control), enabled or not, in `style`.
#[deprecated(note = "the template draws from the resolved style; use `resolve_pager` (#433)")]
#[allow(deprecated)]
pub fn styled_pager_look(binding: &StyleBinding, enabled: bool, style: PagerStyle) -> PagerLook {
    PagerLook::from_resolved(&resolve_pager(binding, enabled, style))
}

#[allow(deprecated)]
impl PagerLook {
    /// The look `resolved` (a pager's resolved style) describes: the compatibility path for code that still reads
    /// a `PagerLook` (#433).
    pub fn from_resolved(resolved: &xgpui_luma_style::Resolved) -> Self {
        let root = resolved.root();
        PagerLook {
            gap_icon_size: root.number("gap_icon_size"),
            border_width: root.number("border_width"),
            page_size_label_gap: root.number("page_size_label_gap"),
            popup_offset_y: root.number("popup_offset_y"),
            popup_gap: root.number("popup_gap"),
            popup_padding: root.number("popup_padding"),
            popup_item_padding_y: root.number("popup_item_padding_y"),
            popup_item_radius_inset: root.number("popup_item_radius_inset"),
            panel_background: root.color("background"),
            border: root.color("border"),
            body_text: root.color("foreground"),
            muted_text: root.color("muted_text"),
            selected_background: root.color("selected_background"),
            selected_foreground: root.color("selected_foreground"),
            hover_background: root.color("hover_background"),
            shadow: root.box_shadow(),
            typography: root.typography(11.0, 16.0),
            button_size: root.number("button_size"),
            button_min_width: root.number("button_min_width"),
            control_height: root.number("height"),
            radius: root.number("radius"),
            padding_x: root.number("padding_x"),
            padding_y: root.number("padding_y"),
            gap: root.number("gap"),
            group_gap: root.number("group_gap"),
        }
    }
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
