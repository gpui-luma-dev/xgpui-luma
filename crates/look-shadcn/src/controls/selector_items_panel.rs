//! Selector / combobox dropdown panel — same surface as floating menu (`popover` + accent item hover).

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

/// The selector and combobox dropdown panel: the `.lss` floating menu (`style/floating_menu.lss`) with the
/// size's metrics and panel geometry.
pub fn selector_items_panel_look(
    mode: &ShadcnModeTokens,
    _theme_mode: ThemeMode,
    size: crate::ShadcnSize,
) -> SelectorItemsPanelLook {
    xgpui_luma::controls::selector_list::styled_selector_items_panel_look(
        &mode_binding(mode, "floating_menu", size),
        &mode.metrics_for(size),
    )
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
