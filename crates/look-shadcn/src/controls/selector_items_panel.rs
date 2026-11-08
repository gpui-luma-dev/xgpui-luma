//! Selector / combobox dropdown panel — same surface as floating menu (`popover` + accent item hover).

use xgpui_luma::controls::selector_list::SelectorItemsPanelLook;
use xgpui_luma::theme::{ControlSize, ThemeMode};

use super::floating_menu::floating_menu_look;
use crate::mode::ShadcnModeTokens;

pub fn selector_items_panel_look(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> SelectorItemsPanelLook {
    SelectorItemsPanelLook::from_menu(
        floating_menu_look(mode, theme_mode, size),
        &mode.metrics,
        &mode.stylesheet().common,
        size.stylesheet_key(),
    )
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;
    use xgpui_luma::theme::ThemeMode;

    use xgpui_luma::theme::ControlSize;

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
        let look = selector_items_panel_look(&mode, xgpui_luma::theme::ThemeMode::Light, ControlSize::Md);

        assert_eq!(look.background, catalog.color("popover").expect("popover"));
        assert_eq!(look.item_hover_background, catalog.color("accent").expect("accent"));
        assert_eq!(look.item_hover_foreground, catalog.color("accent-foreground").expect("accent-foreground"));
    }
}
