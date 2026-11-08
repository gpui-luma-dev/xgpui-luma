use crate::studio::color_format::{preview_color, parse_compact_hsla};
use xgpui_luma::color::gpui_bridge;
use std::collections::HashMap;

use gpui::Hsla;
use xgpui_luma_look_shadcn::ShadcnLook;

use super::model::{DEFAULT_RADIUS_REM, DEFAULT_SPACING_REM, REM_IN_PX};
use crate::studio::export::{token_css_name};
use crate::studio::overrides::StudioOverrides;
use xgpui_luma::color::ColorValue;

fn parse_number(value: &str, min: f32, max: f32) -> Option<f32> {
    value
        .trim()
        .parse::<f32>()
        .ok()
        .filter(|value| value.is_finite())
        .map(|value| value.clamp(min, max))
}

fn format_number(value: f32, rounding_scale: f32, decimals: usize) -> String {
    let rounded = (value * rounding_scale).round() / rounding_scale;
    let mut text = format!("{rounded:.decimals$}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    text
}

pub(super) fn parse_metric_rem(value: &str, min: f32, max: f32) -> Option<f32> {
    parse_number(value, min, max)
}

pub(super) fn format_metric_rem(value: f32) -> String {
    format_number(value, 1000.0, 3)
}

pub(super) fn parse_palette_hsl_number(value: &str, min: f32, max: f32) -> Option<f32> {
    parse_number(value, min, max)
}

pub(super) fn format_palette_hue_deg(value: f32) -> String {
    format_number(value, 1.0, 0)
}

pub(super) fn format_palette_hsl_multiplier(value: f32) -> String {
    format_number(value, 100.0, 2)
}

pub(super) fn effective_radius_rem(look: &ShadcnLook, overrides: &StudioOverrides) -> f32 {
    overrides.radius_rem().unwrap_or_else(|| {
        look.parse_pixel_token("radius").map(|value| value / REM_IN_PX).unwrap_or(DEFAULT_RADIUS_REM)
    })
}

pub(super) fn effective_spacing_rem(look: &ShadcnLook, overrides: &StudioOverrides) -> f32 {
    overrides.spacing_rem().unwrap_or_else(|| {
        look.parse_pixel_token("spacing").map(|value| value / REM_IN_PX).unwrap_or(DEFAULT_SPACING_REM)
    })
}

pub(super) fn format_shadow_color_input(color: ColorValue) -> String {
    let color = preview_color(color);
    format!(
        "hsla({} {}% {}% / {})",
        (color.h * 360.0).round(),
        (color.s * 100.0).round(),
        (color.l * 100.0).round(),
        format_number(color.a, 100.0, 2),
    )
}

pub(super) fn format_shadow_number(value: f32) -> String {
    format_number(value, 100.0, 2)
}

pub(super) fn parse_shadow_color_input(raw: &str) -> Option<ColorValue> {
    if let Some(color) = parse_compact_hsla(raw) {
        return Some(gpui_bridge::from_hsla(color));
    }
    let input = raw.trim();
    let hex = input.strip_prefix('#').unwrap_or(input);
    if matches!(hex.len(), 3 | 6) && hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return ColorValue::parse_css(&format!("#{hex}")).ok();
    }
    ColorValue::parse_css(input).ok()
}

pub(super) fn effective_token_color(
    look: &ShadcnLook,
    global_overrides: &HashMap<String, ColorValue>,
    token: &str,
) -> ColorValue {
    let css_name = token_css_name(token);
    global_overrides
        .get(&css_name)
        .copied()
        .or_else(|| look.token_source_color(token).ok())
        .unwrap_or(ColorValue::srgb(0.5, 0.5, 0.5, 1.0))
}
/// Resolve only the sidebar painting boundary to the current backend.
pub(super) fn token_color_with_fallback(
    look: &ShadcnLook,
    global_overrides: &HashMap<String, ColorValue>,
    token: &str,
    fallback: Hsla,
) -> Hsla {
    let css_name = token_css_name(token);
    global_overrides
        .get(&css_name)
        .copied()
        .or_else(|| look.token_source_color(token).ok())
        .and_then(|source| {
            xgpui_luma::color::gpui_bridge::to_hsla(source, xgpui_luma::color::GamutMapping::CssLocalMinde).ok()
        })
        .unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shadow_color_input_restores_compact_hsla_presentation() {
        let original = gpui_bridge::from_hsla(gpui::hsla(0.625, 0.42, 0.31, 0.37));
        let formatted = format_shadow_color_input(original);
        assert_eq!(formatted, "hsla(225 42% 31% / 0.37)");
        let parsed = preview_color(parse_shadow_color_input(&formatted).unwrap());
        assert!((parsed.a - 0.37).abs() < 0.005);
        assert!(parse_shadow_color_input("#3E63DD").is_some());
        assert!(parse_shadow_color_input("3E63DD").is_some());
        let p3 = ColorValue::display_p3(1.0, 0.0, 0.0, 0.5);
        assert!(format_shadow_color_input(p3).starts_with("hsla("));
    }
}
