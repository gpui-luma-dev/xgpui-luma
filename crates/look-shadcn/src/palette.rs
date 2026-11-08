use anyhow::{Context as _, Result};
use gpui::{Hsla, hsla};

use xgpui_luma::theme::ThemeMode;

use crate::catalog::CssTokenMap;
use crate::color::with_alpha;
use crate::state_color::{algorithmic_state_source, catalog_state_source, token_base_from_palette};
use xgpui_luma::color::{GamutMapping, gpui_bridge};
use crate::tokens::ShadcnToken;
use xgpui_luma::theme::InteractionLayer;

#[derive(Clone, Copy, Debug)]
pub struct ShadcnActionRole {
    pub background: Hsla,
    pub foreground: Hsla,
    pub hover_background: Hsla,
    pub pressed_background: Hsla,
    pub border: Hsla,
}

#[derive(Clone, Copy, Debug)]
pub struct ShadcnPalette {
    pub primary: ShadcnActionRole,
    pub secondary: ShadcnActionRole,
    pub outline: ShadcnActionRole,
    pub ghost: ShadcnActionRole,
    pub disabled_background: Hsla,
    pub disabled_foreground: Hsla,
    pub selected_background: Hsla,
    pub selected_foreground: Hsla,
    pub focus_ring: Hsla,
    pub app_background: Hsla,
    pub app_foreground: Hsla,
    pub app_muted_foreground: Hsla,
    pub border_default: Hsla,
    pub input_background: Hsla,
    pub muted_background: Hsla,
    pub panel_background: Hsla,
    pub body_text: Hsla,
    pub accent_background: Hsla,
    pub accent_foreground: Hsla,
    pub destructive_background: Hsla,
    pub destructive_foreground: Hsla,
}

impl ShadcnPalette {
    pub fn from_catalog(catalog: &CssTokenMap, theme_mode: ThemeMode) -> Result<Self> {
        let primary = filled_role(catalog, "primary", "primary-foreground", theme_mode)?;
        let secondary = filled_role(catalog, "secondary", "secondary-foreground", theme_mode)?;
        let outline = outline_role(catalog, theme_mode)?;
        let ghost = ghost_role(catalog, theme_mode)?;
        let accent_background = catalog.optional_color(&["accent"])?.unwrap_or(catalog.color("muted")?);
        let accent_foreground = catalog.optional_color(&["accent-foreground"])?.unwrap_or(catalog.color("foreground")?);
        let destructive_background = catalog.optional_color(&["destructive"])?.unwrap_or(catalog.color("muted")?);
        let destructive_foreground =
            catalog.optional_color(&["destructive-foreground"])?.unwrap_or(catalog.color("muted-foreground")?);

        Ok(Self {
            primary,
            secondary,
            outline,
            ghost,
            disabled_background: catalog.color("muted")?,
            disabled_foreground: catalog.color("muted-foreground")?,
            selected_background: primary.background,
            selected_foreground: primary.foreground,
            focus_ring: catalog.color("ring")?,
            app_background: catalog.color("background")?,
            app_foreground: catalog.color("foreground")?,
            app_muted_foreground: catalog.color("muted-foreground")?,
            border_default: catalog.color("border")?,
            input_background: catalog.color("input")?,
            muted_background: catalog.color("muted")?,
            panel_background: catalog.optional_color(&["card", "background"])?.context("card or background")?,
            body_text: catalog.color("foreground")?,
            accent_background,
            accent_foreground,
            destructive_background,
            destructive_foreground,
        })
    }
}

fn filled_role(
    catalog: &CssTokenMap,
    background_key: &str,
    foreground_key: &str,
    theme_mode: ThemeMode,
) -> Result<ShadcnActionRole> {
    let source = catalog.source_color(background_key)?;
    let background = gpui_bridge::to_hsla(source, GamutMapping::CssLocalMinde)?;
    let foreground = catalog.color(foreground_key)?;
    let state = |layer| -> Result<Hsla> {
        let derived = match catalog_state_source(catalog, background_key, layer) {
            Some(value) => value,
            None => algorithmic_state_source(source, layer, theme_mode, true)?,
        };
        gpui_bridge::to_hsla(derived, GamutMapping::CssLocalMinde)
    };
    let hover_background = state(InteractionLayer::Hovered)?;
    let pressed_background = state(InteractionLayer::Pressed)?;
    Ok(ShadcnActionRole { background, foreground, hover_background, pressed_background, border: background })
}

fn outline_role(catalog: &CssTokenMap, theme_mode: ThemeMode) -> Result<ShadcnActionRole> {
    let foreground = catalog.color("foreground")?;
    let accent = catalog.optional_color(&["accent"])?.unwrap_or(catalog.color("muted")?);

    match theme_mode {
        ThemeMode::Light => {
            let background = catalog.color("background")?;
            let border = catalog.color("border")?;
            Ok(ShadcnActionRole {
                background,
                foreground,
                hover_background: accent,
                pressed_background: accent,
                border,
            })
        }
        ThemeMode::Dark => {
            let input = catalog.color("input")?;
            Ok(ShadcnActionRole {
                background: with_alpha(input, 0.30),
                foreground,
                hover_background: with_alpha(input, 0.50),
                pressed_background: with_alpha(input, 0.50),
                border: input,
            })
        }
    }
}

fn ghost_role(catalog: &CssTokenMap, theme_mode: ThemeMode) -> Result<ShadcnActionRole> {
    let transparent = hsla(0.0, 0.0, 0.0, 0.0);
    let foreground = catalog.color("foreground")?;
    let accent = catalog.optional_color(&["accent"])?.unwrap_or(catalog.color("muted")?);

    match theme_mode {
        ThemeMode::Light => Ok(ShadcnActionRole {
            background: transparent,
            foreground,
            hover_background: accent,
            pressed_background: accent,
            border: transparent,
        }),
        ThemeMode::Dark => Ok(ShadcnActionRole {
            background: transparent,
            foreground,
            hover_background: with_alpha(accent, 0.50),
            pressed_background: with_alpha(accent, 0.50),
            border: transparent,
        }),
    }
}

impl ShadcnPalette {
    pub fn action(&self, style: crate::controls::ShadcnButtonStyle) -> ShadcnActionRole {
        match style {
            crate::controls::ShadcnButtonStyle::Primary => self.primary,
            crate::controls::ShadcnButtonStyle::Secondary => self.secondary,
            crate::controls::ShadcnButtonStyle::Outline => self.outline,
            crate::controls::ShadcnButtonStyle::Ghost => self.ghost,
            crate::controls::ShadcnButtonStyle::ContentOnly => self.ghost,
        }
    }

    pub fn token_color(&self, token: ShadcnToken) -> Hsla {
        token_base_from_palette(self, token)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use xgpui_luma::theme::ThemeMode;

    use crate::catalog::CssTokenMap;

    use super::ShadcnPalette;

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
            ("destructive".into(), "oklch(0.55 0.22 25)".into()),
            ("destructive-foreground".into(), "oklch(1 0 0)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
        ]))
    }

    #[test]
    fn ghost_light_hover_uses_accent_fill() {
        let catalog = sample_catalog();
        let palette = ShadcnPalette::from_catalog(&catalog, ThemeMode::Light).expect("tokens should parse");
        assert_eq!(palette.ghost.hover_background, catalog.color("accent").expect("accent"));
    }

    #[test]
    fn ghost_dark_hover_uses_accent_at_half_opacity() {
        let catalog = sample_catalog();
        let palette = ShadcnPalette::from_catalog(&catalog, ThemeMode::Dark).expect("tokens should parse");
        let accent = catalog.color("accent").expect("accent");
        assert!((palette.ghost.hover_background.a - 0.50).abs() < f32::EPSILON);
        assert_eq!(palette.ghost.hover_background.h, accent.h);
    }

    #[test]
    fn outline_light_uses_background_and_accent_hover() {
        let catalog = sample_catalog();
        let palette = ShadcnPalette::from_catalog(&catalog, ThemeMode::Light).expect("tokens should parse");
        let outline = palette.outline;

        assert_eq!(outline.background, catalog.color("background").expect("background"));
        assert_eq!(outline.border, catalog.color("border").expect("border"));
        assert_eq!(outline.hover_background, catalog.color("accent").expect("accent"));
    }

    #[test]
    fn outline_dark_uses_input_alpha_and_input_border() {
        let catalog = sample_catalog();
        let palette = ShadcnPalette::from_catalog(&catalog, ThemeMode::Dark).expect("tokens should parse");
        let input = catalog.color("input").expect("input");
        let outline = palette.outline;

        assert_eq!(outline.border, input);
        assert!((outline.background.a - 0.30).abs() < f32::EPSILON);
        assert_eq!(outline.background.h, input.h);
        assert!((outline.hover_background.a - 0.50).abs() < f32::EPSILON);
    }

    #[test]
    fn from_catalog_builds_distinct_action_roles() {
        let palette = ShadcnPalette::from_catalog(&sample_catalog(), ThemeMode::Light).expect("tokens should parse");
        assert_ne!(palette.primary.background, palette.outline.background);
        assert_ne!(palette.primary.hover_background, palette.primary.background);
    }

    #[test]
    fn dark_primary_hover_lightens() {
        let palette = ShadcnPalette::from_catalog(&sample_catalog(), ThemeMode::Dark).expect("tokens should parse");
        assert!(palette.primary.hover_background.l > palette.primary.background.l);
    }
}
