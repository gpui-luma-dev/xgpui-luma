//! Autocomplete / combobox chrome — textfield + floating menu tokens.

use xgpui_luma::controls::autocomplete::AutocompleteLook;
use xgpui_luma::theme::ThemeMode;

use crate::mode::ShadcnModeTokens;
use crate::provenance::ResolvedColor;

#[derive(Clone, Debug)]
pub struct AutocompleteChromeColorTable {
    pub status_color: ResolvedColor,
    pub muted_text_color: ResolvedColor,
    pub clear_icon_color: ResolvedColor,
    pub clear_icon_hover_color: ResolvedColor,
}

impl AutocompleteChromeColorTable {
    pub fn fallback() -> Self {
        Self {
            status_color: ResolvedColor::fallback_foreground(),
            muted_text_color: ResolvedColor::fallback_foreground(),
            clear_icon_color: ResolvedColor::fallback_foreground(),
            clear_icon_hover_color: ResolvedColor::fallback_foreground(),
        }
    }
}

/// The Shadcn autocomplete at `size`, drawn from `.lss` (`style/autocomplete.lss`) for `mode`'s catalog.
pub fn autocomplete_textbox_look(
    mode: &ShadcnModeTokens,
    _theme_mode: ThemeMode,
    size: crate::ShadcnSize,
) -> AutocompleteLook {
    let binding =
        xgpui_luma::style::StyleBinding::new("autocomplete", xgpui_luma::style::ThemeSource::Fixed(mode.style_theme()));
    let binding = if size != crate::ShadcnSize::Md {
        binding.attr("size", size.key())
    } else {
        binding
    };
    let menu = xgpui_luma::controls::floating_menu::styled_floating_menu_look(
        &super::selector_items_panel::mode_binding(mode, "floating_menu", size),
    );
    xgpui_luma::controls::autocomplete::styled_autocomplete_look(&binding, menu)
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use xgpui_luma::theme::ThemeMode;

    use crate::size::ShadcnSize;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::autocomplete_textbox_look;

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
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("popover".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("popover-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("accent-foreground".into(), "oklch(1 0 0)".into()),
        ]))
    }

    #[test]
    fn autocomplete_chrome_uses_primary_and_muted_tokens() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = autocomplete_textbox_look(&mode, ThemeMode::Light, ShadcnSize::Md);
        crate::assert_close(look.status_color, catalog.color("primary").expect("primary"));
        crate::assert_close(look.muted_text_color, catalog.color("muted-foreground").expect("muted-foreground"));
    }
}
