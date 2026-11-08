use gpui::Hsla;

use xgpui_luma::theme::{InteractionLayer, ThemeMode};

use crate::catalog::CssTokenMap;
use xgpui_luma::color::{ColorValue, GamutMapping, gpui_bridge};
use crate::palette::ShadcnPalette;
use crate::tokens::ShadcnToken;

const TOKEN_COUNT: usize = 19;
const LAYER_COUNT: usize = 4;

/// Pre-indexed `(ShadcnToken, InteractionLayer)` colors for O(1) runtime lookup.
#[derive(Clone, Debug)]
pub struct StateColorTable {
    sources: [ColorValue; TOKEN_COUNT * LAYER_COUNT],
    colors: [Hsla; TOKEN_COUNT * LAYER_COUNT],
}

impl StateColorTable {
    pub fn from_catalog(catalog: &CssTokenMap, palette: &ShadcnPalette, theme_mode: ThemeMode) -> Self {
        let mut colors = [Hsla::default(); TOKEN_COUNT * LAYER_COUNT];
        let mut sources = [ColorValue::srgb(0.0, 0.0, 0.0, 0.0); TOKEN_COUNT * LAYER_COUNT];
        for token in ShadcnToken::ALL {
            let base = token_base_from_palette(palette, token);
            for layer in all_layers() {
                let source = resolve_state_source(catalog, palette, token, layer, base, theme_mode);
                sources[slot(token, layer)] = source;
                colors[slot(token, layer)] = preview(source, base);
            }
        }
        Self { sources, colors }
    }

    pub fn source(&self, token: ShadcnToken, layer: InteractionLayer) -> ColorValue {
        self.sources[slot(token, layer)]
    }

    pub fn get(&self, token: ShadcnToken, layer: InteractionLayer) -> Hsla {
        self.colors[slot(token, layer)]
    }
}

fn resolve_state_source(
    catalog: &CssTokenMap,
    palette: &ShadcnPalette,
    token: ShadcnToken,
    layer: InteractionLayer,
    base: Hsla,
    theme_mode: ThemeMode,
) -> ColorValue {
    if let Some(color) = catalog_state_source(catalog, token.css_name(), layer) {
        return color;
    }
    let source = catalog.source_color(token.css_name()).unwrap_or_else(|_| gpui_bridge::from_hsla(base));
    match layer {
        InteractionLayer::Default => source,
        InteractionLayer::Disabled => {
            let key = if token.is_foreground() {
                "muted-foreground"
            } else {
                "muted"
            };
            catalog.source_color(key).unwrap_or_else(|_| gpui_bridge::from_hsla(disabled_color(token, palette)))
        }
        _ => algorithmic_state_source(source, layer, theme_mode, true).unwrap_or(source),
    }
}

pub(crate) fn catalog_state_source(
    catalog: &CssTokenMap,
    css_name: &str,
    layer: InteractionLayer,
) -> Option<ColorValue> {
    catalog.source_color(&state_key(css_name, layer)).ok()
}

pub(crate) fn catalog_state_color(catalog: &CssTokenMap, css_name: &str, layer: InteractionLayer) -> Option<Hsla> {
    catalog_state_source(catalog, css_name, layer)
        .and_then(|source| gpui_bridge::to_hsla(source, GamutMapping::CssLocalMinde).ok())
}

pub(crate) fn algorithmic_state_color(
    base: Hsla,
    layer: InteractionLayer,
    theme_mode: ThemeMode,
    filled: bool,
) -> Hsla {
    algorithmic_state_source(gpui_bridge::from_hsla(base), layer, theme_mode, filled)
        .map(|source| preview(source, base))
        .unwrap_or(base)
}

pub(crate) fn algorithmic_state_source(
    base: ColorValue,
    layer: InteractionLayer,
    theme_mode: ThemeMode,
    filled: bool,
) -> anyhow::Result<ColorValue> {
    let delta = match (theme_mode, layer, filled) {
        (_, InteractionLayer::Default | InteractionLayer::Disabled, _) => 0.0,
        (ThemeMode::Light, InteractionLayer::Hovered, true) => -0.03,
        (ThemeMode::Light, InteractionLayer::Pressed, true) => -0.06,
        (ThemeMode::Light, InteractionLayer::Hovered, false) => -0.02,
        (ThemeMode::Light, InteractionLayer::Pressed, false) => -0.04,
        (ThemeMode::Dark, InteractionLayer::Hovered, true) => 0.04,
        (ThemeMode::Dark, InteractionLayer::Pressed, true) => 0.08,
        (ThemeMode::Dark, InteractionLayer::Hovered, false) => 0.03,
        (ThemeMode::Dark, InteractionLayer::Pressed, false) => 0.06,
    };
    if delta == 0.0 {
        base.validate()?;
        Ok(base)
    } else {
        base.adjust_ui_lightness(delta)
    }
}

fn preview(source: ColorValue, fallback: Hsla) -> Hsla {
    gpui_bridge::to_hsla(source, GamutMapping::CssLocalMinde).unwrap_or(fallback)
}

pub(crate) fn token_base_from_palette(palette: &ShadcnPalette, token: ShadcnToken) -> Hsla {
    match token {
        ShadcnToken::Background => palette.app_background,
        ShadcnToken::Foreground => palette.app_foreground,
        ShadcnToken::Card => palette.panel_background,
        ShadcnToken::CardForeground => palette.body_text,
        ShadcnToken::Popover => palette.panel_background,
        ShadcnToken::PopoverForeground => palette.body_text,
        ShadcnToken::Primary => palette.primary.background,
        ShadcnToken::PrimaryForeground => palette.primary.foreground,
        ShadcnToken::Secondary => palette.secondary.background,
        ShadcnToken::SecondaryForeground => palette.secondary.foreground,
        ShadcnToken::Muted => palette.muted_background,
        ShadcnToken::MutedForeground => palette.app_muted_foreground,
        ShadcnToken::Accent => palette.accent_background,
        ShadcnToken::AccentForeground => palette.accent_foreground,
        ShadcnToken::Destructive => palette.destructive_background,
        ShadcnToken::DestructiveForeground => palette.destructive_foreground,
        ShadcnToken::Border => palette.border_default,
        ShadcnToken::Input => palette.input_background,
        ShadcnToken::Ring => palette.focus_ring,
    }
}

fn disabled_color(token: ShadcnToken, palette: &ShadcnPalette) -> Hsla {
    if token.is_foreground() {
        palette.disabled_foreground
    } else {
        palette.disabled_background
    }
}

fn state_key(css_name: &str, layer: InteractionLayer) -> String {
    match layer {
        InteractionLayer::Default => css_name.to_string(),
        InteractionLayer::Hovered => format!("{css_name}-hover"),
        InteractionLayer::Pressed => format!("{css_name}-pressed"),
        InteractionLayer::Disabled => format!("{css_name}-disabled"),
    }
}

fn slot(token: ShadcnToken, layer: InteractionLayer) -> usize {
    token.index() * LAYER_COUNT + layer_index(layer)
}

fn layer_index(layer: InteractionLayer) -> usize {
    match layer {
        InteractionLayer::Default => 0,
        InteractionLayer::Hovered => 1,
        InteractionLayer::Pressed => 2,
        InteractionLayer::Disabled => 3,
    }
}

fn all_layers() -> [InteractionLayer; 4] {
    [
        InteractionLayer::Default,
        InteractionLayer::Hovered,
        InteractionLayer::Pressed,
        InteractionLayer::Disabled,
    ]
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use xgpui_luma::theme::ThemeMode;

    use crate::catalog::CssTokenMap;
    use crate::palette::ShadcnPalette;

    use super::{algorithmic_state_color, catalog_state_color, StateColorTable};
    use crate::tokens::ShadcnToken;

    use xgpui_luma::theme::InteractionLayer;

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("primary-hover".into(), "oklch(0.45 0.20 355.8943)".into()),
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
        ]))
    }

    #[test]
    fn explicit_catalog_override_wins() {
        let catalog = sample_catalog();
        let hover = catalog_state_color(&catalog, "primary", InteractionLayer::Hovered).expect("override");
        let palette = ShadcnPalette::from_catalog(&catalog, ThemeMode::Light).expect("palette");
        let base = palette.primary.background;
        assert_ne!(hover, base);
    }

    #[test]
    fn light_primary_hover_darkens() {
        let catalog = CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.75 0.15 250)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("secondary".into(), "oklch(0.65 0.10 250)".into()),
            ("secondary-foreground".into(), "oklch(1 0 0)".into()),
            ("background".into(), "oklch(0.98 0.01 250)".into()),
            ("foreground".into(), "oklch(0.20 0.05 250)".into()),
            ("muted".into(), "oklch(0.90 0.01 250)".into()),
            ("muted-foreground".into(), "oklch(0.45 0.02 250)".into()),
            ("accent".into(), "oklch(0.90 0.05 250)".into()),
            ("accent-foreground".into(), "oklch(0.20 0.05 250)".into()),
            ("border".into(), "oklch(0.85 0.01 250)".into()),
            ("input".into(), "oklch(0.85 0.01 250)".into()),
            ("ring".into(), "oklch(0.55 0.15 250)".into()),
            ("card".into(), "oklch(0.96 0.01 250)".into()),
        ]));
        let palette = ShadcnPalette::from_catalog(&catalog, ThemeMode::Light).expect("palette");
        let base = palette.primary.background;
        let hover = StateColorTable::from_catalog(&catalog, &palette, ThemeMode::Light)
            .get(ShadcnToken::Primary, InteractionLayer::Hovered);
        assert!(hover.l < base.l, "light hover should darken");
    }

    #[test]
    fn dark_primary_hover_lightens() {
        let catalog = CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.35 0.15 250)".into()),
            ("primary-foreground".into(), "oklch(0.98 0 0)".into()),
            ("secondary".into(), "oklch(0.40 0.10 250)".into()),
            ("secondary-foreground".into(), "oklch(0.98 0 0)".into()),
            ("background".into(), "oklch(0.15 0.02 250)".into()),
            ("foreground".into(), "oklch(0.95 0.01 250)".into()),
            ("muted".into(), "oklch(0.25 0.02 250)".into()),
            ("muted-foreground".into(), "oklch(0.70 0.01 250)".into()),
            ("accent".into(), "oklch(0.30 0.05 250)".into()),
            ("accent-foreground".into(), "oklch(0.95 0.01 250)".into()),
            ("border".into(), "oklch(0.30 0.02 250)".into()),
            ("input".into(), "oklch(0.30 0.02 250)".into()),
            ("ring".into(), "oklch(0.55 0.15 250)".into()),
            ("card".into(), "oklch(0.18 0.02 250)".into()),
        ]));
        let palette = ShadcnPalette::from_catalog(&catalog, ThemeMode::Dark).expect("palette");
        let base = palette.primary.background;
        let hover = StateColorTable::from_catalog(&catalog, &palette, ThemeMode::Dark)
            .get(ShadcnToken::Primary, InteractionLayer::Hovered);
        assert!(hover.l > base.l, "dark hover should lighten");
    }

    #[test]
    fn algorithmic_light_and_dark_move_in_opposite_directions() {
        let base = gpui::hsla(0.6, 0.5, 0.5, 1.0);
        let light_hover = algorithmic_state_color(base, InteractionLayer::Hovered, ThemeMode::Light, true);
        let dark_hover = algorithmic_state_color(base, InteractionLayer::Hovered, ThemeMode::Dark, true);
        assert!(light_hover.l < base.l);
        assert!(dark_hover.l > base.l);
    }
}
