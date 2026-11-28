//! The floating menu read from `.lss` (issue #285): `floating_menu.lss`. Menus, selector popups and
//! selection panels build their looks from this one.

use crate::style::PartRead;
use std::sync::Arc;

use xgpui_luma_style::PartId;

use super::theme::{FloatingMenuLook, FloatingMenuTheme};
use crate::style::StyleBinding;

/// The floating menu for `binding` (the `floating_menu` control): the panel, a row at rest, hovered and
/// disabled, and the divider.
pub fn styled_floating_menu_look(binding: &StyleBinding) -> FloatingMenuLook {
    let rest = binding.resolve(&[], 1.0);
    let hover = binding.resolve(&[("hover", true)], 1.0);
    let disabled = binding.resolve(&[("disabled", true)], 1.0);
    let (root, item, separator) = (rest.root(), rest.part(PartId::new("item")), rest.part(PartId::new("separator")));
    FloatingMenuLook {
        window_margin: root.number("window_margin"),
        positive: root.color("positive"),
        border_width: root.number("border_width"),
        background: root.color("background"),
        foreground: root.color("foreground"),
        border: root.color("border"),
        shadow: root.box_shadow(),
        radius: root.number("radius"),
        padding: root.number("padding"),
        min_width: root.number("min_width"),
        item_disabled_foreground: disabled.part(PartId::new("item")).color("foreground"),
        item_hover_background: hover.part(PartId::new("item")).color("background"),
        item_hover_foreground: hover.part(PartId::new("item")).color("foreground"),
        item_typography: item.typography(14.0, 20.0),
        item_height: item.number("height"),
        item_padding_x: item.number("padding_x"),
        item_gap: item.number("gap"),
        item_icon_size: root.number("item_icon_size"),
        item_radius: item.number("radius"),
        item_disabled_opacity: disabled.part(PartId::new("item")).opacity(),
        submenu_offset_x: root.number("submenu_offset_x"),
        separator_color: separator.color("background"),
        separator_thickness: separator.number("height"),
        separator_spacing: separator.number("padding_y"),
        separator_inset: separator.number("padding_x"),
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
    fn style(&self) -> StyleBinding {
        self.binding.clone()
    }
}

/// The SDK's floating-menu theme: `floating_menu.lss` alone.
pub(super) fn sdk_theme() -> Arc<dyn FloatingMenuTheme> {
    Arc::new(StyledFloatingMenuTheme::sdk())
}

/// The menu a popup opens, read from `source`'s `floating_menu.lss` each time it is drawn (#397): the menu hook
/// for popup menus, split buttons, context menus and autocompletes, so they open the host's menu, not the SDK's.
pub fn floating_menu_source(source: crate::style::ThemeSource) -> crate::controls::popup_menu::MenuSource {
    let binding = StyleBinding::new("floating_menu", source);
    Arc::new(move || binding.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::{ThemeSource, dark_theme};

    /// The menu hooks open the host theme's floating menu, not the SDK's (#397).
    #[test]
    fn menu_sources_read_the_host_theme() {
        let sdk = styled_floating_menu_look(&StyleBinding::sdk("floating_menu"));
        let menu = styled_floating_menu_look(&floating_menu_source(ThemeSource::Fixed(dark_theme()))());
        assert_ne!(menu.background, sdk.background, "the dark theme's menu, not the SDK's");
        let dark = styled_floating_menu_look(&StyleBinding::new("floating_menu", ThemeSource::Fixed(dark_theme())));
        assert_eq!((menu.background, menu.foreground, menu.border), (dark.background, dark.foreground, dark.border));

        let panel = styled_floating_menu_look(&crate::controls::selector_list::selector_items_panel_source(
            ThemeSource::Fixed(dark_theme()),
        )());
        assert_eq!((panel.background, panel.foreground, panel.border), (dark.background, dark.foreground, dark.border));
    }
}
