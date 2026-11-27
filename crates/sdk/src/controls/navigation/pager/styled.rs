//! The pager read from `.lss` (issue #285): `pager.lss`. Page buttons are drawn by a button template the
//! look supplies (its `.lss` button).

use crate::style::PartRead;
use std::sync::Arc;

use super::PagerStyle;
use super::theme::{PagerLook, PagerTheme};
use crate::controls::button::{ButtonTemplate, default_button_template};
use crate::style::StyleBinding;

/// Supplies the page buttons' template for a resolved pager.
pub type PagerButtonSource = Arc<dyn Fn(&PagerLook) -> Arc<dyn ButtonTemplate<()>> + Send + Sync>;

/// The pager for `binding` (the `pager` control), enabled or not, in `style`.
pub fn styled_pager_look(binding: &StyleBinding, enabled: bool, style: PagerStyle) -> PagerLook {
    let attrs: &[(&str, &str)] = match style {
        PagerStyle::Minimal => &[("variant", "minimal")],
        _ => &[],
    };
    let resolved = binding.resolve_for(attrs, &[("disabled", !enabled)], 1.0);
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
    fn resolve(&self, enabled: bool, style: PagerStyle) -> PagerLook {
        styled_pager_look(&self.binding, enabled, style)
    }

    fn button_template(&self, pager_look: &PagerLook) -> Arc<dyn ButtonTemplate<()>> {
        (self.buttons)(pager_look)
    }
}
