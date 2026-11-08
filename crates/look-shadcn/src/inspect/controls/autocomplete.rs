//! Inspect metadata for `autocomplete`.

use xgpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use crate::{LookContext, LookResolver, ResolvedColor, ShadcnModeTokens};

use super::floating_menu::{FloatingMenuInspectMetrics, FloatingMenuInspectPalette};

pub struct AutocompleteChromeInspectPalette {
    pub status_color: ResolvedColor,
    pub muted_text_color: ResolvedColor,
    pub clear_icon_color: ResolvedColor,
    pub clear_icon_hover_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct AutocompleteInspectMetrics {
    pub textfield: crate::inspect::controls::textfield::TextFieldInspectMetrics,
    pub menu: FloatingMenuInspectMetrics,
}

pub fn inspect_autocomplete_chrome_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
) -> AutocompleteChromeInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let resolver =
        LookResolver::new(ctx.catalog(), theme_mode, "autocomplete_chrome_inspect").with_stylesheet(mode.stylesheet());
    let colors = crate::tables::resolve_autocomplete_chrome_colors(&resolver, true)
        .unwrap_or_else(|_| crate::tables::AutocompleteChromeColorTable::fallback());
    AutocompleteChromeInspectPalette {
        status_color: colors.status_color,
        muted_text_color: colors.muted_text_color,
        clear_icon_color: colors.clear_icon_color,
        clear_icon_hover_color: colors.clear_icon_hover_color,
    }
}

pub fn inspect_autocomplete_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> AutocompleteInspectMetrics {
    AutocompleteInspectMetrics {
        textfield: crate::inspect::controls::textfield::inspect_textfield_metrics(mode, theme_mode, size),
        menu: crate::inspect::controls::floating_menu::inspect_floating_menu_metrics(mode, theme_mode, size),
    }
}

pub fn inspect_autocomplete_menu_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> FloatingMenuInspectPalette {
    crate::inspect::controls::floating_menu::inspect_floating_menu_color_palette(mode, theme_mode, size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspect::test_support::sample_catalog;

    #[test]
    fn autocomplete_chrome_metadata_matches_table() {
        assert_eq!(
            crate::stylesheet::resolve_autocomplete_chrome_colors_metadata(crate::embedded_stylesheet()).len(),
            1
        );
    }

    #[test]
    fn inspect_autocomplete_chrome_uses_primary() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let palette = inspect_autocomplete_chrome_color_palette(&mode, ThemeMode::Light);
        assert_eq!(palette.status_color.value, catalog.color("primary").expect("primary"));
    }
}
