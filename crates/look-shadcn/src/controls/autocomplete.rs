//! Autocomplete / combobox chrome — textfield + floating menu tokens.

use serde::Deserialize;

use xgpui_luma::controls::autocomplete::AutocompleteLook;
use xgpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::stylesheet::{StylesheetConfig, find_autocomplete_chrome_color_rule, resolve_autocomplete_chrome_color_rule};

use super::floating_menu::floating_menu_look;

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

pub fn resolve_autocomplete_chrome_colors(
    resolver: &LookResolver<'_>,
    _present: bool,
) -> anyhow::Result<AutocompleteChromeColorTable> {
    resolve_autocomplete_chrome_colors_with_stylesheet(resolver, resolver.stylesheet())
}

pub fn resolve_autocomplete_chrome_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
) -> anyhow::Result<AutocompleteChromeColorTable> {
    let rule = find_autocomplete_chrome_color_rule(stylesheet)
        .ok_or_else(|| anyhow::anyhow!("no matching autocomplete chrome color rule"))?;
    let colors = resolve_autocomplete_chrome_color_rule(resolver, rule)?;
    Ok(AutocompleteChromeColorTable {
        status_color: colors.status_color,
        muted_text_color: colors.muted_text_color,
        clear_icon_color: colors.clear_icon_color,
        clear_icon_hover_color: colors.clear_icon_hover_color,
    })
}

pub fn autocomplete_textbox_look(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> AutocompleteLook {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let resolver =
        LookResolver::new(ctx.catalog(), ctx.theme_mode, "autocomplete_chrome").with_stylesheet(mode.stylesheet());
    let colors = resolve_autocomplete_chrome_colors(&resolver, true)
        .unwrap_or_else(|_| AutocompleteChromeColorTable::fallback());

    let geometry = mode.stylesheet().common.autocomplete.resolve_geometry(
        size.stylesheet_key(),
        xgpui_luma::theme::stylesheet::AutocompleteGeometry {
            status_font_size: mode.typography.text.scale.sm.size,
            status_line_height: mode.typography.text.scale.sm.line_height,
            ..Default::default()
        },
    );
    let mut status_typography = mode.typography.text.scale.sm;
    status_typography.size = geometry.status_font_size;
    status_typography.line_height = geometry.status_line_height;
    AutocompleteLook {
        status_typography,
        status_color: colors.status_color.hsla(),
        muted_text_color: colors.muted_text_color.hsla(),
        clear_icon_color: colors.clear_icon_color.hsla(),
        clear_icon_hover_color: colors.clear_icon_hover_color.hsla(),
        menu: floating_menu_look(ctx.tokens, ctx.theme_mode, size),
    }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use xgpui_luma::theme::ThemeMode;

    use xgpui_luma::theme::ControlSize;

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
        let look = autocomplete_textbox_look(&mode, ThemeMode::Light, ControlSize::Md);
        assert_eq!(look.status_color, catalog.color("primary").expect("primary"));
        assert_eq!(look.muted_text_color, catalog.color("muted-foreground").expect("muted-foreground"));
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct AutocompleteStylesheet {
    #[serde(default)]
    pub chrome: AutocompleteChromeStylesheet,
}

impl AutocompleteStylesheet {
    /// Native bundled chrome colors.
    pub fn native_default() -> Self {
        Self {
            chrome: AutocompleteChromeStylesheet {
                color_rules: vec![AutocompleteChromeColorRule {
                    status_color: "primary".into(),
                    muted_text_color: "muted-foreground".into(),
                    clear_icon_color: "muted-foreground".into(),
                    clear_icon_hover_color: "foreground".into(),
                }],
            },
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct AutocompleteChromeStylesheet {
    #[serde(default)]
    pub color_rules: Vec<AutocompleteChromeColorRule>,
}

impl AutocompleteChromeStylesheet {
    pub fn color_rule(&self) -> Option<&AutocompleteChromeColorRule> {
        self.color_rules.first()
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct AutocompleteChromeColorRule {
    pub status_color: String,
    pub muted_text_color: String,
    pub clear_icon_color: String,
    pub clear_icon_hover_color: String,
}
