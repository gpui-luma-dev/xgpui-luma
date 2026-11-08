//! Provenance types for inspect-path color resolution. Paint-path code uses [`ResolvedColor::hsla`] only.

use gpui::{Hsla, hsla};

use xgpui_luma::theme::{InteractionLayer, ThemeMode};

use crate::catalog::CssTokenMap;
use xgpui_luma::color::{ColorValue, GamutMapping, gpui_bridge};
use crate::state_color::{algorithmic_state_source, catalog_state_color, catalog_state_source};

#[derive(Clone, Debug)]
pub struct ResolvedColor {
    pub value: Hsla,
    pub source: ColorSource,
}

/// Source color and provenance, retained independently of the GPUI preview.
#[derive(Clone, Debug)]
pub struct ResolvedSourceColor {
    pub value: ColorValue,
    pub source: ColorSource,
}

impl ResolvedSourceColor {
    /// Produce the current backend's bounded sRGB render representation.
    pub fn srgb_preview(self) -> anyhow::Result<ResolvedColor> {
        Ok(
            ResolvedColor {
                value: gpui_bridge::to_hsla(self.value, GamutMapping::CssLocalMinde)?,
                source: self.source,
            },
        )
    }
}

#[derive(Clone, Debug)]
pub struct ResolvedMetric {
    pub value_px: f32,
    pub source: MetricSource,
}

#[derive(Clone, Debug)]
pub enum MetricSource {
    CssVar { token: String },
    Derived { note: String },
    Scaffold { path: String },
    Constant { label: String },
}

#[derive(Clone, Debug)]
pub enum TypographySource {
    Constant { label: String },
    Derived { note: String },
    CssVar { token: String },
    Scaffold { path: String },
}

#[derive(Clone, Debug)]
pub struct ResolvedTypography {
    pub value: String,
    pub source: TypographySource,
}

#[derive(Clone, Debug)]
pub enum ColorSource {
    Transparent,
    CssVar { token: String },
    TokenAlpha { token: String, alpha_percent: f32 },
    StateVar { base_token: String, layer: InteractionLayer },
    Algorithmic { base_token: String, layer: InteractionLayer },
    Derived { note: String },
}

#[derive(Clone, Debug)]
pub struct TableRuleMetadata {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

pub struct LookResolver<'a> {
    catalog: &'a CssTokenMap,
    theme_mode: ThemeMode,
    _label: &'a str,
    stylesheet: &'a crate::stylesheet::StylesheetConfig,
}

impl<'a> LookResolver<'a> {
    pub fn new(catalog: &'a CssTokenMap, theme_mode: ThemeMode, label: &'a str) -> Self {
        Self { catalog, theme_mode, _label: label, stylesheet: crate::stylesheet::embedded_stylesheet() }
    }

    pub(crate) fn with_stylesheet(mut self, stylesheet: &'a crate::stylesheet::StylesheetConfig) -> Self {
        self.stylesheet = stylesheet;
        self
    }

    pub(crate) fn stylesheet(&self) -> &crate::stylesheet::StylesheetConfig {
        self.stylesheet
    }

    /// Render projection; use `resolve_source_decl` for persistent values and derivations.
    pub fn resolve_decl(&self, decl: &str) -> anyhow::Result<ResolvedColor> {
        self.resolve_source_decl(decl)?.srgb_preview()
    }

    /// Resolve token, alpha, and interaction state without gamut mapping.
    pub fn resolve_source_decl(&self, decl: &str) -> anyhow::Result<ResolvedSourceColor> {
        let decl = decl.trim().strip_prefix("--").unwrap_or(decl.trim());

        if decl.eq_ignore_ascii_case("transparent") {
            return Ok(ResolvedSourceColor {
                value: ColorValue::srgb(0.0, 0.0, 0.0, 0.0),
                source: ColorSource::Transparent,
            });
        }

        if let Some((base, alpha_percent)) = parse_slash_alpha(decl)? {
            let base_color = self.catalog.source_color(base)?;
            return Ok(ResolvedSourceColor {
                value: base_color.with_alpha(alpha_percent / 100.0)?,
                source: ColorSource::TokenAlpha { token: base.to_string(), alpha_percent },
            });
        }

        if self.catalog.get(decl).is_some() {
            return Ok(ResolvedSourceColor {
                value: self.catalog.source_color(decl)?,
                source: ColorSource::CssVar { token: decl.to_string() },
            });
        }

        if let Some((base, layer)) = parse_state_suffix(decl) {
            let base_color = self.catalog.source_color(base)?;
            let filled = is_filled_token(base);
            if let Some(value) = catalog_state_source(self.catalog, base, layer) {
                return Ok(ResolvedSourceColor {
                    value,
                    source: ColorSource::StateVar { base_token: base.to_string(), layer },
                });
            }
            return Ok(ResolvedSourceColor {
                value: algorithmic_state_source(base_color, layer, self.theme_mode, filled)?,
                source: ColorSource::Algorithmic { base_token: base.to_string(), layer },
            });
        }

        anyhow::bail!("unknown color declaration `{decl}`")
    }

    /// First catalog token that exists, otherwise the first token.
    pub fn resolve_first_decl(&self, tokens: &[&str]) -> anyhow::Result<ResolvedColor> {
        for token in tokens {
            if self.catalog.get(token).is_some() {
                return self.resolve_decl(token);
            }
        }
        tokens.first().map_or_else(
            || anyhow::bail!("resolve_first_decl called with empty token list"),
            |token| self.resolve_decl(token),
        )
    }

    /// Like [`resolve_color_first_layer`](crate::resolve::resolve_color_first_layer) with provenance.
    pub fn resolve_first_layer_decl(&self, tokens: &[&str], layer: InteractionLayer) -> anyhow::Result<ResolvedColor> {
        for token in tokens {
            if catalog_state_color(self.catalog, token, layer).is_some() {
                let state_decl = match layer {
                    InteractionLayer::Default => token.to_string(),
                    InteractionLayer::Hovered => format!("{token}-hover"),
                    InteractionLayer::Pressed => format!("{token}-pressed"),
                    InteractionLayer::Disabled => format!("{token}-disabled"),
                };
                return self.resolve_decl(&state_decl);
            }
        }
        let base = tokens.iter().find(|token| self.catalog.get(token).is_some()).copied().unwrap_or(tokens[0]);
        let state_decl = match layer {
            InteractionLayer::Default => base.to_string(),
            InteractionLayer::Hovered => format!("{base}-hover"),
            InteractionLayer::Pressed => format!("{base}-pressed"),
            InteractionLayer::Disabled => format!("{base}-disabled"),
        };
        self.resolve_decl(&state_decl)
    }

    /// Outline / unchecked selection surface (card/background, accent hover, muted pressed).
    pub fn resolve_outline_layer_decl(&self, layer: InteractionLayer) -> anyhow::Result<ResolvedColor> {
        match layer {
            InteractionLayer::Disabled | InteractionLayer::Pressed => self.resolve_decl("muted"),
            InteractionLayer::Hovered => self.resolve_first_layer_decl(&["accent", "muted"], layer),
            InteractionLayer::Default => self.resolve_first_layer_decl(&["card", "background"], layer),
        }
    }

    /// Filled action background for a button style and interaction layer.
    pub fn resolve_action_layer_decl(
        &self,
        style: crate::controls::ShadcnButtonStyle,
        layer: InteractionLayer,
    ) -> anyhow::Result<ResolvedColor> {
        let (background, _) = crate::action::style_token_pair(style);
        match layer {
            InteractionLayer::Disabled => self.resolve_decl("muted"),
            InteractionLayer::Default => self.resolve_decl(background),
            InteractionLayer::Hovered => self.resolve_decl(&format!("{background}-hover")),
            InteractionLayer::Pressed => self.resolve_decl(&format!("{background}-pressed")),
        }
    }

    /// Foreground token paired with a button style (`primary-foreground`, etc.).
    pub fn resolve_action_foreground_decl(
        &self,
        style: crate::controls::ShadcnButtonStyle,
    ) -> anyhow::Result<ResolvedColor> {
        let (_, foreground) = crate::action::style_token_pair(style);
        self.resolve_decl(foreground)
    }

    /// Label text color for enabled/disabled controls.
    pub fn resolve_label_decl(&self, disabled: bool) -> anyhow::Result<ResolvedColor> {
        if disabled {
            self.resolve_decl("muted-foreground")
        } else {
            self.resolve_decl("foreground")
        }
    }

    /// Scrollbar thumb pressed state: darken the border token.
    pub fn resolve_darken_border_decl(&self, amount: f32) -> anyhow::Result<ResolvedColor> {
        let base = self.resolve_source_decl("border")?;
        ResolvedSourceColor {
            value: base.value.adjust_ui_lightness(-amount)?,
            source: ColorSource::Derived { note: format!("darken(border, {}%)", (amount * 100.0).round() as i32) },
        }
        .srgb_preview()
    }

    /// List row hover tint (`accent/40` with `first(accent,muted)` fallback).
    pub fn resolve_accent_whisper_decl(&self, alpha_percent: u8) -> anyhow::Result<ResolvedColor> {
        let base = self.catalog.source_color_first(&["accent", "muted"])?;
        ResolvedSourceColor {
            value: base.with_alpha(f32::from(alpha_percent) / 100.0)?,
            source: ColorSource::Derived { note: format!("--accent/{alpha_percent} · first(accent,muted)") },
        }
        .srgb_preview()
    }

    /// Pressed list row derived from the accent whisper hover tint.
    pub fn resolve_accent_whisper_pressed_decl(&self, alpha_percent: u8) -> anyhow::Result<ResolvedColor> {
        let hover = self
            .catalog
            .source_color_first(&["accent", "muted"])?
            .with_alpha(f32::from(alpha_percent) / 100.0)?;
        ResolvedSourceColor {
            value: algorithmic_state_source(hover, InteractionLayer::Pressed, self.theme_mode, false)?,
            source: ColorSource::Derived { note: format!("accent whisper {alpha_percent}% · pressed") },
        }
        .srgb_preview()
    }
}

impl ResolvedColor {
    pub fn transparent() -> Self {
        Self { value: hsla(0.0, 0.0, 0.0, 0.0), source: ColorSource::Transparent }
    }

    pub fn fallback_foreground() -> Self {
        Self { value: hsla(0.0, 0.0, 0.0, 1.0), source: ColorSource::CssVar { token: "foreground".to_string() } }
    }

    pub fn hsla(&self) -> Hsla {
        self.value
    }

    pub fn into_hsla(self) -> Hsla {
        self.value
    }
}

fn parse_slash_alpha(decl: &str) -> anyhow::Result<Option<(&str, f32)>> {
    let Some((base, alpha_raw)) = decl.split_once('/') else {
        return Ok(None);
    };
    let alpha_raw = alpha_raw.trim().strip_suffix('%').unwrap_or(alpha_raw.trim());
    let alpha = alpha_raw.parse::<f32>().map_err(|_| anyhow::anyhow!("invalid alpha in `{decl}`"))?;
    anyhow::ensure!(alpha.is_finite(), "alpha must be finite in `{decl}`");
    let max = if decl.contains('%') || alpha > 1.0 { 100.0 } else { 1.0 };
    anyhow::ensure!((0.0..=max).contains(&alpha), "alpha out of range in `{decl}`");
    let alpha_percent = if decl.contains('%') || alpha > 1.0 {
        alpha
    } else {
        alpha * 100.0
    };
    Ok(Some((base, alpha_percent)))
}

fn parse_state_suffix(decl: &str) -> Option<(&str, InteractionLayer)> {
    const SUFFIXES: [(&str, InteractionLayer); 4] = [
        ("-hover", InteractionLayer::Hovered),
        ("-pressed", InteractionLayer::Pressed),
        ("-active", InteractionLayer::Pressed),
        ("-disabled", InteractionLayer::Disabled),
    ];

    for (suffix, layer) in SUFFIXES {
        if let Some(base) = decl.strip_suffix(suffix) {
            return Some((base, layer));
        }
    }
    None
}

fn is_filled_token(base: &str) -> bool {
    matches!(base, "primary" | "secondary" | "destructive")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use xgpui_luma::theme::ThemeMode;

    use super::*;
    use crate::catalog::CssTokenMap;

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "hsl(330 64% 52%)".into()),
            ("primary-hover".into(), "hsl(320 64% 45%)".into()),
            ("primary-foreground".into(), "hsl(0 0% 100%)".into()),
            ("accent".into(), "hsl(18 80% 44%)".into()),
            ("accent-foreground".into(), "hsl(0 0% 100%)".into()),
            ("foreground".into(), "hsl(192 81% 14%)".into()),
            ("input".into(), "hsl(186 8% 55%)".into()),
            ("muted".into(), "hsl(180 7% 60%)".into()),
            ("muted-foreground".into(), "hsl(192 81% 14%)".into()),
        ]))
    }

    #[test]
    fn wide_gamut_resolution_derives_from_source_not_preview() {
        let catalog = CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(.7 .4 725 / .37)".into()),
            ("accent".into(), "color(display-p3 1 0 0 / .37)".into()),
        ]));
        let resolver = LookResolver::new(&catalog, ThemeMode::Light, "test");
        let source = resolver.resolve_source_decl("primary").unwrap().value;
        let hover = resolver.resolve_source_decl("primary-hover").unwrap().value;
        assert_eq!(hover, source.adjust_ui_lightness(-0.03).unwrap());
        assert_eq!(
            resolver.resolve_decl("primary-hover").unwrap().value,
            gpui_bridge::to_hsla(hover, GamutMapping::CssLocalMinde).unwrap()
        );
        let alpha = resolver.resolve_source_decl("accent/12.3456%").unwrap();
        assert_eq!(alpha.value, ColorValue::display_p3(1.0, 0.0, 0.0, 12.3456 / 100.0));
        let pressed = resolver.resolve_accent_whisper_pressed_decl(40).unwrap();
        let expected =
            catalog.source_color("accent").unwrap().with_alpha(0.4).unwrap().adjust_ui_lightness(-0.04).unwrap();
        assert_eq!(pressed.value, gpui_bridge::to_hsla(expected, GamutMapping::CssLocalMinde).unwrap());
        for invalid in ["primary/-10", "primary/200", "primary/NaN"] {
            assert!(resolver.resolve_source_decl(invalid).is_err());
        }
    }

    #[test]
    fn resolve_transparent() {
        let catalog = sample_catalog();
        let resolver = LookResolver::new(&catalog, ThemeMode::Light, "test");
        let color = resolver.resolve_decl("transparent").expect("transparent");
        assert_eq!(color.value, hsla(0.0, 0.0, 0.0, 0.0));
        assert!(matches!(color.source, ColorSource::Transparent));
    }

    #[test]
    fn resolve_slash_alpha() {
        let catalog = sample_catalog();
        let resolver = LookResolver::new(&catalog, ThemeMode::Dark, "test");
        let input = catalog.color("input").expect("input");
        let color = resolver.resolve_decl("input/50").expect("alpha");
        assert!((color.value.a - 0.50).abs() < f32::EPSILON);
        assert_eq!(color.value.h, input.h);
        assert!(matches!(color.source, ColorSource::TokenAlpha { alpha_percent, .. } if alpha_percent == 50.0));
    }

    #[test]
    fn resolve_state_suffix_uses_catalog_override() {
        let catalog = sample_catalog();
        let resolver = LookResolver::new(&catalog, ThemeMode::Light, "test");
        let hover = resolver.resolve_decl("primary-hover").expect("hover");
        let primary = catalog.color("primary").expect("primary");
        assert_ne!(hover.value, primary);
        assert!(matches!(hover.source, ColorSource::CssVar { .. }));
    }

    #[test]
    fn resolve_state_suffix_falls_back_to_algorithmic() {
        let catalog = sample_catalog();
        let resolver = LookResolver::new(&catalog, ThemeMode::Light, "test");
        let pressed = resolver.resolve_decl("primary-pressed").expect("pressed");
        let primary = catalog.color("primary").expect("primary");
        assert_ne!(pressed.value, primary);
        assert!(matches!(pressed.source, ColorSource::Algorithmic { .. }));
    }
}
