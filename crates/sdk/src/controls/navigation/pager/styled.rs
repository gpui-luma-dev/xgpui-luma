//! The pager read from `.lss` (issue #285): `pager.lss`. Page buttons are drawn by a button template the
//! look supplies (its `.lss` button).

use std::sync::Arc;

use gpui::Hsla;
use xgpui_luma_style::{PartView, Value};

use super::PagerStyle;
use super::theme::{PagerLook, PagerTheme};
use crate::controls::button::{ButtonTemplate, default_button_template};
use crate::style::StyleBinding;
use crate::theme::LumaTextStyle;

/// Supplies the page buttons' template for a resolved pager.
pub type PagerButtonSource = Arc<dyn Fn(&PagerLook) -> Arc<dyn ButtonTemplate<()>> + Send + Sync>;

fn color(part: PartView<'_>, name: &str) -> Hsla {
    part.value(name).and_then(Value::as_color).map(Hsla::from).unwrap_or_else(gpui::transparent_black)
}

fn number(part: PartView<'_>, name: &str) -> f32 {
    part.value(name).and_then(Value::as_number).unwrap_or(0.0)
}

/// The pager for `binding` (the `pager` control), enabled or not, in `style`.
pub fn styled_pager_look(binding: &StyleBinding, enabled: bool, style: PagerStyle) -> PagerLook {
    let attrs: &[(&str, &str)] = match style {
        PagerStyle::Minimal => &[("variant", "minimal")],
        _ => &[],
    };
    let resolved = binding.resolve_for(attrs, &[("disabled", !enabled)], 1.0);
    let root = resolved.root();
    PagerLook {
        gap_icon_size: number(root, "gap_icon_size"),
        border_width: number(root, "border_width"),
        page_size_label_gap: number(root, "page_size_label_gap"),
        popup_offset_y: number(root, "popup_offset_y"),
        popup_gap: number(root, "popup_gap"),
        popup_padding: number(root, "popup_padding"),
        popup_item_padding_y: number(root, "popup_item_padding_y"),
        popup_item_radius_inset: number(root, "popup_item_radius_inset"),
        panel_background: color(root, "background"),
        border: color(root, "border"),
        body_text: color(root, "foreground"),
        muted_text: color(root, "muted_text"),
        selected_background: color(root, "selected_background"),
        selected_foreground: color(root, "selected_foreground"),
        hover_background: color(root, "hover_background"),
        shadow: root.shadow().map(|layers| xgpui_luma_style::box_shadows(&layers)).unwrap_or_default(),
        typography: LumaTextStyle {
            size: root.font_size().unwrap_or(11.0),
            line_height: root.line_height().unwrap_or(16.0),
            weight: gpui::FontWeight(number(root, "font_weight").max(100.0)),
        },
        button_size: number(root, "button_size"),
        button_min_width: number(root, "button_min_width"),
        control_height: number(root, "height"),
        radius: number(root, "radius"),
        padding_x: number(root, "padding_x"),
        padding_y: number(root, "padding_y"),
        gap: number(root, "gap"),
        group_gap: number(root, "group_gap"),
        disabled_opacity: number(root, "disabled_opacity"),
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
