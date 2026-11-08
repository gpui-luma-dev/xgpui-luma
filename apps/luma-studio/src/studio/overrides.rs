use std::collections::HashMap;

use xgpui_luma_color::color_slider::color_spec::{ColorSpecification, Hsl};
use xgpui_luma::color::ColorValue;
use xgpui_luma::theme::ThemeMode;
use xgpui_luma_look_shadcn::{ShadcnLook, ShadowTokenParts, shadow_ladder_overrides};

use super::hs_mixer::{ThemePaletteHsOverride, clamp_palette_temperature_amount, clamp_palette_vividness_amount};

pub const RADIUS_REM_MIN: f32 = 0.0;
pub const RADIUS_REM_MAX: f32 = 5.0;
pub const SPACING_REM_MIN: f32 = 0.0;
pub const SPACING_REM_MAX: f32 = 0.35;
pub const SHADOW_OPACITY_MIN: f32 = 0.0;
pub const SHADOW_OPACITY_MAX: f32 = 1.0;
pub const SHADOW_BLUR_MIN: f32 = 0.0;
pub const SHADOW_BLUR_MAX: f32 = 32.0;
pub const SHADOW_SPREAD_MIN: f32 = -12.0;
pub const SHADOW_SPREAD_MAX: f32 = 20.0;
pub const SHADOW_OFFSET_X_MIN: f32 = -20.0;
pub const SHADOW_OFFSET_X_MAX: f32 = 20.0;
pub const SHADOW_OFFSET_Y_MIN: f32 = -8.0;
pub const SHADOW_OFFSET_Y_MAX: f32 = 24.0;
pub const PALETTE_HUE_DEG_MIN: f32 = -180.0;
pub const PALETTE_HUE_DEG_MAX: f32 = 180.0;
pub const PALETTE_SATURATION_MULTIPLIER_MIN: f32 = 0.0;
pub const PALETTE_SATURATION_MULTIPLIER_MAX: f32 = 2.0;
pub const PALETTE_LIGHTNESS_MULTIPLIER_MIN: f32 = 0.0;
pub const PALETTE_LIGHTNESS_MULTIPLIER_MAX: f32 = 2.0;

#[derive(Clone, Debug, PartialEq)]
pub struct ThemePaletteHslOverride {
    pub hue_deg: f32,
    pub saturation_multiplier: f32,
    pub lightness_multiplier: f32,
}

impl Default for ThemePaletteHslOverride {
    fn default() -> Self {
        Self { hue_deg: 0.0, saturation_multiplier: 1.0, lightness_multiplier: 1.0 }
    }
}

impl ThemePaletteHslOverride {
    /// Apply HSL controls to an unclamped source conversion, without using the preview.
    pub fn apply(&self, color: ColorValue) -> anyhow::Result<ColorValue> {
        if *self == Self::default() {
            color.validate()?;
            return Ok(color);
        }
        let spec = Hsl::from_color_value(color)?;
        let result = Hsl {
            h: (spec.h + self.hue_deg).rem_euclid(360.0),
            s: spec.s * self.saturation_multiplier,
            l: spec.l * self.lightness_multiplier,
            a: spec.a,
        }
        .to_color_value();
        result.validate()?;
        Ok(result)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ThemeShadowOverride {
    pub color: ColorValue,
    pub blur_px: f32,
    pub spread_px: f32,
    pub offset_x_px: f32,
    pub offset_y_px: f32,
}

impl ThemeShadowOverride {
    pub fn opacity(&self) -> f32 {
        self.color.alpha()
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        if let Ok(color) = self.color.with_alpha(clamp_shadow_opacity(opacity)) {
            self.color = color;
        }
    }

    fn to_parts(&self) -> ShadowTokenParts {
        ShadowTokenParts {
            color: self.color,
            blur_px: self.blur_px,
            spread_px: self.spread_px,
            offset_x_px: self.offset_x_px,
            offset_y_px: self.offset_y_px,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct StudioOverrides {
    pub global_color_overrides: HashMap<String, ColorValue>,
    pub light_palette_hsl: ThemePaletteHslOverride,
    pub dark_palette_hsl: ThemePaletteHslOverride,
    pub light_palette_hs: ThemePaletteHsOverride,
    pub dark_palette_hs: ThemePaletteHsOverride,
    pub radius_rem: Option<f32>,
    pub spacing_rem: Option<f32>,
    pub shadow: Option<ThemeShadowOverride>,
    pub font_sans: Option<String>,
    pub font_serif: Option<String>,
    pub font_mono: Option<String>,
}

impl StudioOverrides {
    pub fn global_color_override(&self, token: &str) -> Option<ColorValue> {
        self.global_color_overrides.get(token).copied()
    }

    pub fn palette_hsl(&self, mode: ThemeMode) -> &ThemePaletteHslOverride {
        match mode {
            ThemeMode::Light => &self.light_palette_hsl,
            ThemeMode::Dark => &self.dark_palette_hsl,
        }
    }

    pub fn palette_hs(&self, mode: ThemeMode) -> &ThemePaletteHsOverride {
        match mode {
            ThemeMode::Light => &self.light_palette_hs,
            ThemeMode::Dark => &self.dark_palette_hs,
        }
    }

    pub fn radius_rem(&self) -> Option<f32> {
        self.radius_rem
    }

    pub fn spacing_rem(&self) -> Option<f32> {
        self.spacing_rem
    }

    pub fn shadow_override(&self) -> Option<&ThemeShadowOverride> {
        self.shadow.as_ref()
    }

    pub fn font_sans(&self) -> Option<&str> {
        self.font_sans.as_deref()
    }

    pub fn font_serif(&self) -> Option<&str> {
        self.font_serif.as_deref()
    }

    pub fn font_mono(&self) -> Option<&str> {
        self.font_mono.as_deref()
    }

    pub fn font_stack(&self, token: &str) -> Option<&str> {
        match token {
            "font-sans" => self.font_sans(),
            "font-serif" => self.font_serif(),
            "font-mono" => self.font_mono(),
            _ => None,
        }
    }

    pub fn set_global_color(&mut self, token: String, color: ColorValue) {
        self.global_color_overrides.insert(token, color);
    }

    pub fn set_palette_hsl_override(&mut self, mode: ThemeMode, palette_hsl: ThemePaletteHslOverride) {
        let palette_hsl = ThemePaletteHslOverride {
            hue_deg: clamp_palette_hue_deg(palette_hsl.hue_deg),
            saturation_multiplier: clamp_palette_saturation_multiplier(palette_hsl.saturation_multiplier),
            lightness_multiplier: clamp_palette_lightness_multiplier(palette_hsl.lightness_multiplier),
        };

        match mode {
            ThemeMode::Light => self.light_palette_hsl = palette_hsl,
            ThemeMode::Dark => self.dark_palette_hsl = palette_hsl,
        }
    }

    pub fn set_palette_hs_override(&mut self, mode: ThemeMode, palette_hs: ThemePaletteHsOverride) {
        let palette_hs = ThemePaletteHsOverride {
            active: palette_hs.active,
            vividness_amount: clamp_palette_vividness_amount(palette_hs.vividness_amount),
            temperature_amount: clamp_palette_temperature_amount(palette_hs.temperature_amount),
        };

        match mode {
            ThemeMode::Light => self.light_palette_hs = palette_hs,
            ThemeMode::Dark => self.dark_palette_hs = palette_hs,
        }
    }

    pub fn set_radius_rem(&mut self, rem: f32) {
        self.radius_rem = Some(clamp_radius_rem(rem));
    }

    pub fn set_spacing_rem(&mut self, rem: f32) {
        self.spacing_rem = Some(clamp_spacing_rem(rem));
    }

    pub fn set_shadow_override(&mut self, shadow: ThemeShadowOverride) {
        self.shadow = Some(shadow);
    }

    pub fn set_font_sans(&mut self, stack: String) {
        self.font_sans = Some(stack);
    }

    pub fn set_font_serif(&mut self, stack: String) {
        self.font_serif = Some(stack);
    }

    pub fn set_font_mono(&mut self, stack: String) {
        self.font_mono = Some(stack);
    }

    pub fn token_overrides(&self) -> anyhow::Result<HashMap<String, String>> {
        let mut overrides = HashMap::new();
        if let Some(radius_rem) = self.radius_rem {
            overrides.insert("radius".to_string(), rem_css_value(radius_rem));
        }
        if let Some(spacing_rem) = self.spacing_rem {
            overrides.insert("spacing".to_string(), rem_css_value(spacing_rem));
        }
        if let Some(shadow) = &self.shadow {
            overrides.extend(shadow_ladder_overrides(shadow.to_parts())?);
        }
        if let Some(font_sans) = &self.font_sans {
            overrides.insert("font-sans".to_string(), font_sans.clone());
        }
        if let Some(font_serif) = &self.font_serif {
            overrides.insert("font-serif".to_string(), font_serif.clone());
        }
        if let Some(font_mono) = &self.font_mono {
            overrides.insert("font-mono".to_string(), font_mono.clone());
        }
        Ok(overrides)
    }

    pub fn clear_palette_hsl_overrides(&mut self) {
        self.light_palette_hsl = ThemePaletteHslOverride::default();
        self.dark_palette_hsl = ThemePaletteHslOverride::default();
    }

    pub fn clear_palette_hs_overrides(&mut self) {
        self.light_palette_hs = ThemePaletteHsOverride::default();
        self.dark_palette_hs = ThemePaletteHsOverride::default();
    }

    pub fn clear_metric_overrides(&mut self) {
        self.radius_rem = None;
        self.spacing_rem = None;
    }

    pub fn clear_shadow_override(&mut self) {
        self.shadow = None;
    }

    pub fn clear_font_overrides(&mut self) {
        self.font_sans = None;
        self.font_serif = None;
        self.font_mono = None;
    }

    pub fn clear_all_overrides(&mut self) {
        self.global_color_overrides.clear();
        self.clear_palette_hsl_overrides();
        self.clear_palette_hs_overrides();
        self.clear_metric_overrides();
        self.clear_shadow_override();
        self.clear_font_overrides();
    }
}

pub fn clamp_palette_hue_deg(hue_deg: f32) -> f32 {
    hue_deg.clamp(PALETTE_HUE_DEG_MIN, PALETTE_HUE_DEG_MAX)
}

pub fn clamp_palette_saturation_multiplier(multiplier: f32) -> f32 {
    multiplier.clamp(PALETTE_SATURATION_MULTIPLIER_MIN, PALETTE_SATURATION_MULTIPLIER_MAX)
}

pub fn clamp_palette_lightness_multiplier(multiplier: f32) -> f32 {
    multiplier.clamp(PALETTE_LIGHTNESS_MULTIPLIER_MIN, PALETTE_LIGHTNESS_MULTIPLIER_MAX)
}

pub fn clamp_radius_rem(rem: f32) -> f32 {
    rem.clamp(RADIUS_REM_MIN, RADIUS_REM_MAX)
}

pub fn clamp_spacing_rem(rem: f32) -> f32 {
    rem.clamp(SPACING_REM_MIN, SPACING_REM_MAX)
}

pub fn clamp_shadow_opacity(opacity: f32) -> f32 {
    opacity.clamp(SHADOW_OPACITY_MIN, SHADOW_OPACITY_MAX)
}

pub fn clamp_shadow_blur(blur_px: f32) -> f32 {
    blur_px.clamp(SHADOW_BLUR_MIN, SHADOW_BLUR_MAX)
}

pub fn clamp_shadow_spread(spread_px: f32) -> f32 {
    spread_px.clamp(SHADOW_SPREAD_MIN, SHADOW_SPREAD_MAX)
}

pub fn clamp_shadow_offset_x(offset_x_px: f32) -> f32 {
    offset_x_px.clamp(SHADOW_OFFSET_X_MIN, SHADOW_OFFSET_X_MAX)
}

pub fn clamp_shadow_offset_y(offset_y_px: f32) -> f32 {
    offset_y_px.clamp(SHADOW_OFFSET_Y_MIN, SHADOW_OFFSET_Y_MAX)
}

pub fn resolved_shadow_override(look: &ShadcnLook, overrides: &StudioOverrides) -> ThemeShadowOverride {
    overrides.shadow.clone().unwrap_or_else(|| default_shadow_override(look))
}

pub fn default_shadow_override(look: &ShadcnLook) -> ThemeShadowOverride {
    if let Ok(mut layers) = look.parse_shadow_source_token("shadow")
        && let Some(layer) = layers.drain(..).next()
    {
        return ThemeShadowOverride {
            color: layer.color,
            blur_px: layer.blur_px,
            spread_px: layer.spread_px,
            offset_x_px: layer.offset_x_px,
            offset_y_px: layer.offset_y_px,
        };
    }

    ThemeShadowOverride {
        color: ColorValue::srgb(0.0, 0.0, 0.0, 0.12),
        blur_px: 10.0,
        spread_px: 0.0,
        offset_x_px: 0.0,
        offset_y_px: 4.0,
    }
}

fn rem_css_value(rem: f32) -> String {
    format!("{:.4}rem", rem)
}

pub fn format_font_family_stack(family: &str, generic: &str) -> String {
    format!("{}, {generic}", quote_css_font_family(family))
}

pub fn quote_css_font_family(family: &str) -> String {
    let family = family.trim();
    if family.contains(' ') {
        format!("'{family}'")
    } else {
        family.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_adjustments_use_sources_and_preserve_identity() {
        let source = ColorValue::display_p3(1.123456, -0.123456, 0.234567, 0.345678);
        assert_eq!(ThemePaletteHslOverride::default().apply(source).unwrap(), source);
        let adjustment =
            ThemePaletteHslOverride { hue_deg: 10.0, saturation_multiplier: 1.2, lightness_multiplier: 1.0 };
        let adjusted = adjustment.apply(source).unwrap();
        assert_eq!(adjusted.alpha(), source.alpha());
        assert!(!adjusted.is_in_gamut(xgpui_luma::color::Gamut::Srgb).unwrap());
    }

    #[test]
    fn font_stack_quotes_spaced_family_names() {
        assert_eq!(format_font_family_stack("Helvetica Neue", "sans-serif"), "'Helvetica Neue', sans-serif");
        assert_eq!(format_font_family_stack("Inter", "sans-serif"), "Inter, sans-serif");
    }

    #[test]
    fn token_overrides_include_font_stacks() {
        let mut overrides = StudioOverrides::default();
        overrides.set_font_sans("Poppins, sans-serif".to_string());
        overrides.set_font_mono("'IBM Plex Mono', monospace".to_string());

        let tokens = overrides.token_overrides().unwrap();
        assert_eq!(tokens.get("font-sans").map(String::as_str), Some("Poppins, sans-serif"));
        assert_eq!(tokens.get("font-mono").map(String::as_str), Some("'IBM Plex Mono', monospace"));
    }

    #[test]
    fn shadow_override_updates_the_complete_ladder() {
        let mut overrides = StudioOverrides::default();
        overrides.set_shadow_override(ThemeShadowOverride {
            color: ColorValue::srgb(0.0, 0.0, 0.0, 0.2),
            blur_px: 10.0,
            spread_px: -2.0,
            offset_x_px: 0.0,
            offset_y_px: 4.0,
        });

        let tokens = overrides.token_overrides().unwrap();
        let shadow = tokens.get("shadow").expect("base shadow override");
        for token in xgpui_luma_look_shadcn::SHADOW_LADDER_TOKENS {
            assert!(tokens.contains_key(token));
        }
        assert!(tokens.get("shadow-md").expect("medium shadow override") != shadow);
        assert!(tokens.get("shadow-2xl").expect("largest shadow override") != shadow);
        assert!(shadow.contains(","));
    }
}
