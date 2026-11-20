//! The floating menu read from `.lss` (issue #285): `floating_menu.lss`. Menus, selector popups and
//! selection panels build their looks from this one.

use std::sync::Arc;

use gpui::Hsla;
use xgpui_luma_style::{PartId, PartView, Value, box_shadows};

use super::theme::{FloatingMenuLook, FloatingMenuTheme};
use crate::style::StyleBinding;
use crate::theme::LumaTextStyle;

fn color(part: PartView<'_>, name: &str) -> Hsla {
    part.value(name).and_then(Value::as_color).map(Hsla::from).unwrap_or_else(gpui::transparent_black)
}

fn number(part: PartView<'_>, name: &str) -> f32 {
    part.value(name).and_then(Value::as_number).unwrap_or(0.0)
}

/// The floating menu for `binding` (the `floating_menu` control): the panel, a row at rest, hovered and
/// disabled, and the divider.
pub fn styled_floating_menu_look(binding: &StyleBinding) -> FloatingMenuLook {
    let rest = binding.resolve(&[], 1.0);
    let hover = binding.resolve(&[("hover", true)], 1.0);
    let disabled = binding.resolve(&[("disabled", true)], 1.0);
    let (root, item, separator) = (rest.root(), rest.part(PartId::new("item")), rest.part(PartId::new("separator")));
    FloatingMenuLook {
        window_margin: number(root, "window_margin"),
        positive: color(root, "positive"),
        border_width: number(root, "border_width"),
        background: color(root, "background"),
        foreground: color(root, "foreground"),
        border: color(root, "border"),
        shadow: root.shadow().map(|layers| box_shadows(&layers)).unwrap_or_default(),
        radius: number(root, "radius"),
        padding: number(root, "padding"),
        min_width: number(root, "min_width"),
        item_disabled_foreground: color(disabled.part(PartId::new("item")), "foreground"),
        item_hover_background: color(hover.part(PartId::new("item")), "background"),
        item_hover_foreground: color(hover.part(PartId::new("item")), "foreground"),
        item_typography: LumaTextStyle {
            size: item.font_size().unwrap_or(14.0),
            line_height: item.line_height().unwrap_or(20.0),
            weight: gpui::FontWeight(number(item, "font_weight").max(100.0)),
        },
        item_height: number(item, "height"),
        item_padding_x: number(item, "padding_x"),
        item_gap: number(item, "gap"),
        item_icon_size: number(root, "item_icon_size"),
        item_radius: number(item, "radius"),
        disabled_opacity: number(root, "disabled_opacity"),
        submenu_offset_x: number(root, "submenu_offset_x"),
        separator_color: color(separator, "background"),
        separator_thickness: number(separator, "height"),
        separator_spacing: number(separator, "padding_y"),
        separator_inset: number(separator, "padding_x"),
    }
}

/// A floating-menu theme read from `.lss`.
#[derive(Clone)]
pub struct StyledFloatingMenuTheme {
    binding: StyleBinding,
}

impl StyledFloatingMenuTheme {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self { binding: StyleBinding::sdk("floating_menu") }
    }
}

impl FloatingMenuTheme for StyledFloatingMenuTheme {
    fn resolve(&self) -> FloatingMenuLook {
        styled_floating_menu_look(&self.binding)
    }
}

/// The SDK's floating-menu theme: `floating_menu.lss` alone.
pub(super) fn sdk_theme() -> Arc<dyn FloatingMenuTheme> {
    Arc::new(StyledFloatingMenuTheme::sdk())
}
