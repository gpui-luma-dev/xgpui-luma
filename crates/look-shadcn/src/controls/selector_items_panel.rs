//! Selector / combobox dropdown panel — same surface as floating menu (`popover` + accent item hover).
#![allow(deprecated)] // Still reads a deprecated Look until its family has a style-override replacement (#433).

use xgpui_luma::controls::selector_list::SelectorItemsPanelLook;
use xgpui_luma::theme::{ThemeMode};

use crate::mode::ShadcnModeTokens;

/// A binding of `control` to the mode's `.lss` at `size`.
pub(crate) fn mode_binding(
    mode: &ShadcnModeTokens,
    control: &'static str,
    size: crate::ShadcnSize,
) -> xgpui_luma::style::StyleBinding {
    let binding =
        xgpui_luma::style::StyleBinding::new(control, xgpui_luma::style::ThemeSource::Fixed(mode.style_theme()));
    if size != crate::ShadcnSize::Md {
        binding.attr("size", size.key())
    } else {
        binding
    }
}

/// The selector and combobox dropdown panel's style (#433): the `.lss` floating menu (`style/floating_menu.lss`) at
/// `size`, which the SDK's panel templates resolve and draw from.
pub fn selector_items_panel_style(mode: &ShadcnModeTokens, size: crate::ShadcnSize) -> xgpui_luma::style::StyleBinding {
    mode_binding(mode, "floating_menu", size)
}

/// The selector and combobox dropdown panel, built from [`selector_items_panel_style`].
#[deprecated(note = "use `selector_items_panel_style`; the SDK's panel templates draw from the style (#433)")]
pub fn selector_items_panel_look(
    mode: &ShadcnModeTokens,
    _theme_mode: ThemeMode,
    size: crate::ShadcnSize,
) -> SelectorItemsPanelLook {
    SelectorItemsPanelLook::from_style(&selector_items_panel_style(mode, size))
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;
    use xgpui_luma::theme::ThemeMode;

    use crate::size::ShadcnSize;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::selector_items_panel_look;

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("secondary".into(), "oklch(0.6437 0.1019 187.3840)".into()),
            ("secondary-foreground".into(), "oklch(1 0 0)".into()),
            ("background".into(), "oklch(0.9735 0.0261 90.0953)".into()),
            ("foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("muted".into(), "oklch(0.6979 0.0159 196.7940)".into()),
            ("muted-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("accent-foreground".into(), "oklch(1 0 0)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("popover".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("popover-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
        ]))
    }

    #[test]
    fn selector_items_panel_uses_popover_surface_and_accent_item_hover() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = selector_items_panel_look(&mode, xgpui_luma::theme::ThemeMode::Light, ShadcnSize::Md);

        crate::assert_close(look.background, catalog.color("popover").expect("popover"));
        crate::assert_close(look.item_hover_background, catalog.color("accent").expect("accent"));
        crate::assert_close(look.item_hover_foreground, catalog.color("accent-foreground").expect("accent-foreground"));
    }
}

/// The panel style draws the border width the old look took from the size's metrics, at every size (#433).
#[cfg(test)]
mod parity_tests {
    use xgpui_luma::style::PartRead;

    #[test]
    fn the_style_keeps_each_sizes_border_width() {
        let look = crate::ShadcnLook::from_css_str(crate::FALLBACK_CSS).unwrap();
        let mode = look.mode_tokens();
        for size in [crate::ShadcnSize::Sm, crate::ShadcnSize::Md, crate::ShadcnSize::Lg] {
            let style = super::selector_items_panel_style(mode.as_ref(), size);
            let border_width = style.resolve(&[], 1.0).root().number("border_width");
            assert_eq!(border_width, mode.metrics_for(size).border_width, "{size:?}");
        }
    }
}
