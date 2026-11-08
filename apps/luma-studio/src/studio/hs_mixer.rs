//! HS mixer / palette generation is isolated here so the rest of the studio can treat it as a
//! single module boundary, even if the underlying implementation later moves fully to OKLCH.

use std::collections::HashMap;

use xgpui_luma::color::ColorValue;
use anyhow::Result;
use xgpui_luma::theme::ThemeMode;

pub const PALETTE_VIVIDNESS_AMOUNT_MIN: f32 = -1.0;
pub const PALETTE_VIVIDNESS_AMOUNT_MAX: f32 = 1.0;
pub const PALETTE_TEMPERATURE_AMOUNT_MIN: f32 = -1.0;
pub const PALETTE_TEMPERATURE_AMOUNT_MAX: f32 = 1.0;

#[derive(Clone, Debug, PartialEq)]
pub struct ThemePaletteHsOverride {
    pub active: bool,
    pub vividness_amount: f32,
    pub temperature_amount: f32,
}

impl Default for ThemePaletteHsOverride {
    fn default() -> Self {
        Self { active: false, vividness_amount: 0.0, temperature_amount: 0.0 }
    }
}

pub fn clamp_palette_vividness_amount(amount: f32) -> f32 {
    amount.clamp(PALETTE_VIVIDNESS_AMOUNT_MIN, PALETTE_VIVIDNESS_AMOUNT_MAX)
}

pub fn clamp_palette_temperature_amount(amount: f32) -> f32 {
    amount.clamp(PALETTE_TEMPERATURE_AMOUNT_MIN, PALETTE_TEMPERATURE_AMOUNT_MAX)
}

pub fn derive_palette_hs_color_overrides(
    mode: ThemeMode,
    primary: ColorValue,
    palette_hs: &ThemePaletteHsOverride,
) -> Result<HashMap<String, ColorValue>> {
    if !palette_hs.active {
        return Ok(HashMap::new());
    }

    let source = primary.to_oklcha_unclamped()?;
    let base = BaseHueChroma { hue: source.hue.into_raw_degrees(), chroma: source.chroma };
    let hue = adjusted_hue(base.hue, palette_hs.temperature_amount);
    let chroma = adjusted_chroma(base.chroma, palette_hs.vividness_amount);

    let chroma_bg = chroma * 0.5;
    let chroma_text = chroma.min(0.1);
    let chroma_action = chroma.max(0.1);
    let chroma_alert = chroma.max(0.05);
    let hue_secondary = (hue + 180.0).rem_euclid(360.0);

    let palette = match mode {
        ThemeMode::Dark => GeneratedPalette {
            bg_dark: oklch_color(0.10, chroma_bg, hue),
            bg: oklch_color(0.15, chroma_bg, hue),
            bg_light: oklch_color(0.20, chroma_bg, hue),
            text: oklch_color(0.96, chroma_text, hue),
            text_muted: oklch_color(0.76, chroma_text, hue),
            highlight: oklch_color(0.50, chroma, hue),
            border: oklch_color(0.40, chroma, hue),
            border_muted: oklch_color(0.30, chroma, hue),
            primary: oklch_color(0.76, chroma_action, hue),
            secondary: oklch_color(0.76, chroma_action, hue_secondary),
            danger: oklch_color(0.70, chroma_alert, 30.0),
            warning: oklch_color(0.70, chroma_alert, 100.0),
            success: oklch_color(0.70, chroma_alert, 160.0),
            info: oklch_color(0.70, chroma_alert, 260.0),
            on_color: oklch_color(0.15, chroma_text, hue),
        },
        ThemeMode::Light => GeneratedPalette {
            bg_dark: oklch_color(0.92, chroma_bg, hue),
            bg: oklch_color(0.96, chroma_bg, hue),
            bg_light: oklch_color(1.00, chroma_bg, hue),
            text: oklch_color(0.15, chroma, hue),
            text_muted: oklch_color(0.40, chroma, hue),
            highlight: oklch_color(1.00, chroma, hue),
            border: oklch_color(0.60, chroma, hue),
            border_muted: oklch_color(0.70, chroma, hue),
            primary: oklch_color(0.40, chroma_action, hue),
            secondary: oklch_color(0.40, chroma_action, hue_secondary),
            danger: oklch_color(0.50, chroma_alert, 30.0),
            warning: oklch_color(0.50, chroma_alert, 100.0),
            success: oklch_color(0.50, chroma_alert, 160.0),
            info: oklch_color(0.50, chroma_alert, 260.0),
            on_color: oklch_color(0.96, chroma_bg, hue),
        },
    };

    Ok(generated_palette_to_token_overrides(palette, mode))
}

#[derive(Clone, Copy, Debug)]
struct BaseHueChroma {
    hue: f32,
    chroma: f32,
}

#[derive(Clone, Copy, Debug)]
struct GeneratedPalette {
    bg_dark: ColorValue,
    bg: ColorValue,
    bg_light: ColorValue,
    text: ColorValue,
    text_muted: ColorValue,
    highlight: ColorValue,
    border: ColorValue,
    border_muted: ColorValue,
    primary: ColorValue,
    secondary: ColorValue,
    danger: ColorValue,
    warning: ColorValue,
    success: ColorValue,
    info: ColorValue,
    on_color: ColorValue,
}

fn oklch_color(lightness: f32, chroma: f32, hue: f32) -> ColorValue {
    ColorValue::oklch(lightness, chroma, hue, 1.0)
}

fn adjusted_hue(base_hue: f32, amount: f32) -> f32 {
    let amount = clamp_palette_temperature_amount(amount);
    if amount.abs() < f32::EPSILON {
        return base_hue.rem_euclid(360.0);
    }

    let target_hue = if amount < 0.0 { 45.0 } else { 225.0 };
    interpolate_hue(base_hue, target_hue, amount.abs())
}

fn adjusted_chroma(base_chroma: f32, amount: f32) -> f32 {
    let amount = clamp_palette_vividness_amount(amount);
    if amount.abs() < f32::EPSILON {
        return base_chroma.max(0.0);
    }

    if amount >= 0.0 {
        let target = base_chroma.max(0.14);
        lerp(base_chroma, target, amount).max(0.0)
    } else {
        lerp(base_chroma, 0.0, -amount).max(0.0)
    }
}

fn generated_palette_to_token_overrides(palette: GeneratedPalette, mode: ThemeMode) -> HashMap<String, ColorValue> {
    let mut overrides = HashMap::new();

    insert(&mut overrides, "background", palette.bg);
    insert(&mut overrides, "foreground", palette.text);
    insert(&mut overrides, "card", palette.bg_light);
    insert(&mut overrides, "card-foreground", palette.text);
    insert(&mut overrides, "popover", palette.bg_light);
    insert(&mut overrides, "popover-foreground", palette.text);
    insert(&mut overrides, "primary", palette.primary);
    insert(&mut overrides, "primary-foreground", palette.on_color);
    insert(&mut overrides, "secondary", palette.secondary);
    insert(&mut overrides, "secondary-foreground", palette.on_color);
    insert(&mut overrides, "accent", palette.highlight);
    insert(&mut overrides, "accent-foreground", palette.on_color);
    insert(&mut overrides, "destructive", palette.danger);
    insert(&mut overrides, "destructive-foreground", palette.on_color);
    insert(&mut overrides, "border", palette.border);
    insert(&mut overrides, "input", palette.border_muted);
    insert(&mut overrides, "ring", palette.highlight);
    insert(&mut overrides, "muted-foreground", palette.text_muted);
    insert(
        &mut overrides,
        "muted",
        match mode {
            ThemeMode::Dark => palette.bg_light,
            ThemeMode::Light => palette.bg_dark,
        },
    );

    insert(&mut overrides, "sidebar", palette.bg_dark);
    insert(&mut overrides, "sidebar-foreground", palette.text);
    insert(&mut overrides, "sidebar-primary", palette.primary);
    insert(&mut overrides, "sidebar-primary-foreground", palette.on_color);
    insert(&mut overrides, "sidebar-accent", palette.highlight);
    insert(&mut overrides, "sidebar-accent-foreground", palette.on_color);
    insert(&mut overrides, "sidebar-border", palette.border);
    insert(&mut overrides, "sidebar-ring", palette.highlight);

    insert(&mut overrides, "chart-1", palette.primary);
    insert(&mut overrides, "chart-2", palette.secondary);
    insert(&mut overrides, "chart-3", palette.info);
    insert(&mut overrides, "chart-4", palette.success);
    insert(&mut overrides, "chart-5", palette.warning);

    overrides
}

fn insert(overrides: &mut HashMap<String, ColorValue>, token: &str, color: ColorValue) {
    overrides.insert(format!("--{token}"), color);
}

fn lerp(from: f32, to: f32, amount: f32) -> f32 {
    from + (to - from) * amount.clamp(0.0, 1.0)
}

fn interpolate_hue(from: f32, to: f32, amount: f32) -> f32 {
    let delta = shortest_hue_delta(from, to);
    (from + delta * amount).rem_euclid(360.0)
}

fn shortest_hue_delta(from: f32, to: f32) -> f32 {
    (to - from + 540.0).rem_euclid(360.0) - 180.0
}

#[cfg(test)]
mod tests {
    use xgpui_luma::theme::ThemeMode;

    use super::{ThemePaletteHsOverride, derive_palette_hs_color_overrides, shortest_hue_delta};

    #[test]
    fn palette_generation_retains_oklch_before_preview_mapping() {
        let source = xgpui_luma::color::ColorValue::oklch(0.65, 0.345678, 412.12345, 1.0);
        let overrides = derive_palette_hs_color_overrides(
            ThemeMode::Dark,
            source,
            &ThemePaletteHsOverride { active: true, vividness_amount: 0.0, temperature_amount: 0.0 },
        )
        .unwrap();
        let generated = overrides["--primary"].to_oklcha_unclamped().unwrap();
        assert_eq!(generated.chroma, 0.345678);
        assert!(!overrides["--primary"].is_in_gamut(xgpui_luma::color::Gamut::Srgb).unwrap());
    }

    #[test]
    fn identity_override_returns_no_palette_changes() {
        let overrides = derive_palette_hs_color_overrides(
            ThemeMode::Dark,
            xgpui_luma::color::gpui_bridge::from_hsla(gpui::hsla(0.6, 0.7, 0.5, 1.0)),
            &ThemePaletteHsOverride::default(),
        )
        .unwrap();
        assert!(overrides.is_empty());
    }

    #[test]
    fn shortest_hue_delta_wraps_across_zero() {
        assert_eq!(shortest_hue_delta(350.0, 10.0), 20.0);
        assert_eq!(shortest_hue_delta(10.0, 350.0), -20.0);
    }

    #[test]
    fn non_identity_override_generates_primary() {
        let overrides = derive_palette_hs_color_overrides(
            ThemeMode::Dark,
            xgpui_luma::color::gpui_bridge::from_hsla(gpui::hsla(0.6, 0.7, 0.5, 1.0)),
            &ThemePaletteHsOverride { active: true, vividness_amount: 0.5, temperature_amount: 0.0 },
        )
        .unwrap();
        assert!(overrides.contains_key("--primary"));
        assert!(overrides.contains_key("--background"));
    }
}
