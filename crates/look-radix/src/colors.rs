//! Static Radix Colors catalog generated from the pinned local v3.0.0 source.

#[path = "colors_data.rs"]
mod colors_data;

use gpui::Hsla;
use xgpui_luma::color::{ColorValue, GamutMapping, gpui_bridge};
use xgpui_luma::theme::ThemeMode;

pub use colors_data::{BLACK_ALPHA_STEPS, DARK_FAMILIES, LIGHT_FAMILIES, COLORS_VERSION, WHITE_ALPHA_STEPS};
pub use colors_data::{RawColorScale, ColorValueKind};

pub fn families(mode: ThemeMode) -> &'static [RawColorScale] {
    match mode {
        ThemeMode::Light => LIGHT_FAMILIES,
        ThemeMode::Dark => DARK_FAMILIES,
    }
}

/// Twelve parsed steps for one named family, or `None` when the catalog has no such family.
#[cfg(test)]
pub fn family_steps(mode: ThemeMode, family: &str) -> Option<[Hsla; 12]> {
    let scale = families(mode).iter().find(|candidate| candidate.family == family)?;
    Some(scale.steps.map(parse_color))
}

/// Parse catalog data into retained source colors. Malformed input returns an error.
pub fn parse_source_color(value: &str) -> anyhow::Result<ColorValue> {
    if let Some(inner) = value.strip_prefix("rgba(").and_then(|value| value.strip_suffix(')')) {
        let values: Vec<f32> =
            inner.split(',').map(|component| component.trim().parse::<f32>()).collect::<Result<_, _>>()?;
        anyhow::ensure!(values.len() == 4, "expected four rgba channels");
        let source = ColorValue::srgb(values[0] / 255.0, values[1] / 255.0, values[2] / 255.0, values[3]);
        source.validate()?;
        return Ok(source);
    }
    ColorValue::parse_css(value)
}

/// Named source steps. The pinned catalog is sRGB; authored scales can be wide gamut.
pub fn family_source_steps(mode: ThemeMode, family: &str) -> anyhow::Result<Option<[ColorValue; 12]>> {
    let Some(scale) = families(mode).iter().find(|candidate| candidate.family == family) else {
        return Ok(None);
    };
    let mut steps = [ColorValue::srgb(0.0, 0.0, 0.0, 1.0); 12];
    for (index, value) in scale.steps.iter().enumerate() {
        steps[index] = parse_source_color(value)?;
    }
    Ok(Some(steps))
}

/// Current-backend catalog preview; invalid values use a transparent fallback.
pub fn parse_color(value: &str) -> Hsla {
    parse_source_color(value)
        .and_then(|source| gpui_bridge::to_hsla(source, GamutMapping::CssLocalMinde))
        .unwrap_or_else(|_| gpui::hsla(0.0, 0.0, 0.0, 0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_parser_keeps_p3_and_fractional_rgb_and_rejects_malformed_hex() {
        assert_eq!(parse_source_color("color(display-p3 1 0 0)").unwrap(), ColorValue::display_p3(1.0, 0.0, 0.0, 1.0));
        assert_eq!(
            parse_source_color("rgba(12.5, 20, 30, 0.37)").unwrap(),
            ColorValue::srgb(12.5 / 255.0, 20.0 / 255.0, 30.0 / 255.0, 0.37)
        );
        for invalid in ["#", "#éa", "rgba(1,2,NaN,1)"] {
            assert!(parse_source_color(invalid).is_err());
        }
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            for family in families(mode) {
                assert!(family_source_steps(mode, family.family).unwrap().is_some());
            }
        }
    }

    #[test]
    fn catalog_contains_all_v3_families_in_order() {
        assert_eq!(families(ThemeMode::Light).len(), 31);
        assert_eq!(families(ThemeMode::Light).first().unwrap().family, "gray");
        assert_eq!(families(ThemeMode::Light).last().unwrap().family, "gold");
        assert_eq!(COLORS_VERSION, "3.0.0");
    }

    #[test]
    fn alpha_values_keep_transparency() {
        assert!(parse_color(BLACK_ALPHA_STEPS[0]).a < 1.0);
        assert!(parse_color(WHITE_ALPHA_STEPS[11]).a > parse_color(WHITE_ALPHA_STEPS[0]).a);
    }
}
