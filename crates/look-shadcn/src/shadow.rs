use std::collections::HashMap;

use anyhow::{Context as _, Result};
use gpui::{BoxShadow, point, px};
use crate::catalog::CssTokenMap;
use xgpui_luma::color::{ColorValue, GamutMapping, gpui_bridge};

pub const SHADOW_LADDER_TOKENS: [&str; 8] = [
    "shadow-2xs",
    "shadow-xs",
    "shadow-sm",
    "shadow",
    "shadow-md",
    "shadow-lg",
    "shadow-xl",
    "shadow-2xl",
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowTokenParts {
    pub color: ColorValue,
    pub blur_px: f32,
    pub spread_px: f32,
    pub offset_x_px: f32,
    pub offset_y_px: f32,
}

impl ShadowTokenParts {
    pub fn to_css_value(self) -> Result<String> {
        format_shadow_layer(self.offset_x_px, self.offset_y_px, self.blur_px, self.spread_px, self.color)
    }
}

/// Generates the standard Shadcn shadow ladder from one editable shadow part set.
pub fn shadow_ladder_overrides(parts: ShadowTokenParts) -> Result<HashMap<String, String>> {
    parts.color.validate()?;
    let levels = [
        ("shadow-2xs", 0.5, None),
        ("shadow-xs", 0.5, None),
        ("shadow-sm", 1.0, Some((1.0, 2.0, -1.0, 1.0))),
        ("shadow", 1.0, Some((1.0, 2.0, -1.0, 1.0))),
        ("shadow-md", 1.0, Some((2.0, 4.0, -1.0, 1.0))),
        ("shadow-lg", 1.0, Some((4.0, 6.0, -1.0, 1.0))),
        ("shadow-xl", 1.0, Some((8.0, 10.0, -1.0, 1.0))),
        ("shadow-2xl", 2.5, None),
    ];

    levels
        .into_iter()
        .map(|(token, primary_alpha, secondary)| {
            let primary = ShadowTokenParts {
                color: parts.color.with_alpha((parts.color.alpha() * primary_alpha).clamp(0.0, 1.0))?,
                ..parts
            };
            let value = if let Some((offset_y_factor, blur_px, spread_px, _)) = secondary {
                let secondary = format_shadow_layer(
                    parts.offset_x_px,
                    parts.offset_y_px * offset_y_factor,
                    blur_px,
                    spread_px,
                    parts.color,
                )?;
                format!("{}, {}", primary.to_css_value()?, secondary)
            } else {
                primary.to_css_value()?
            };
            Ok((token.to_string(), value))
        })
        .collect()
}

fn format_shadow_layer(offset_x: f32, offset_y: f32, blur: f32, spread: f32, color: ColorValue) -> Result<String> {
    anyhow::ensure!(
        [offset_x, offset_y, blur, spread].iter().all(|v| v.is_finite()) && blur >= 0.0,
        "invalid shadow geometry"
    );
    Ok(format!("{offset_x}px {offset_y}px {blur}px {spread}px {}", color.to_css()?))
}

pub(crate) fn parse_shadow_token(catalog: &CssTokenMap, token_key: &str) -> Result<Vec<BoxShadow>> {
    let raw = catalog.get(token_key).with_context(|| format!("missing css token `--{token_key}`"))?;
    parse_box_shadow_value(raw)
}

pub(crate) fn parse_shadow_source_token(catalog: &CssTokenMap, token_key: &str) -> Result<Vec<ShadowTokenParts>> {
    let raw = catalog.get(token_key).with_context(|| format!("missing css token `--{token_key}`"))?;
    if raw.trim() == "none" {
        return Ok(Vec::new());
    }
    split_shadow_layers(raw).into_iter().map(parse_shadow_layer).collect()
}

fn parse_box_shadow_value(raw: &str) -> Result<Vec<BoxShadow>> {
    if raw.trim() == "none" {
        return Ok(Vec::new());
    }
    let layers = split_shadow_layers(raw);
    layers
        .into_iter()
        .map(|layer| {
            let source = parse_shadow_layer(layer)?;
            Ok(BoxShadow {
                color: gpui_bridge::to_hsla(source.color, GamutMapping::CssLocalMinde)?,
                offset: point(px(source.offset_x_px), px(source.offset_y_px)),
                blur_radius: px(source.blur_px),
                spread_radius: px(source.spread_px),
                inset: false,
            })
        })
        .collect()
}

fn split_shadow_layers(raw: &str) -> Vec<&str> {
    let mut layers = Vec::new();
    let mut start = 0;
    let mut depth: usize = 0;

    for (index, ch) in raw.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                let layer = raw[start..index].trim();
                if !layer.is_empty() {
                    layers.push(layer);
                }
                start = index + 1;
            }
            _ => {}
        }
    }

    let tail = raw[start..].trim();
    if !tail.is_empty() {
        layers.push(tail);
    }

    layers
}

fn parse_shadow_layer(layer: &str) -> Result<ShadowTokenParts> {
    let (lengths, color_raw) = split_lengths_and_color(layer)?;
    let mut parts = lengths.split_whitespace();
    let offset_x = next_length(&mut parts, "shadow offset-x")?;
    let offset_y = next_length(&mut parts, "shadow offset-y")?;
    let blur = next_length(&mut parts, "shadow blur")?;
    let spread = parts
        .next()
        .map(|raw| parse_length_px(raw).with_context(|| format!("invalid shadow spread `{raw}`")))
        .transpose()?
        .unwrap_or(0.0);
    if parts.next().is_some() {
        anyhow::bail!("unexpected extra values in shadow layer `{layer}`");
    }

    let color = ColorValue::parse_css(color_raw)?;
    anyhow::ensure!(
        [offset_x, offset_y, blur, spread].iter().all(|v| v.is_finite()) && blur >= 0.0,
        "invalid shadow geometry"
    );
    Ok(ShadowTokenParts { color, offset_x_px: offset_x, offset_y_px: offset_y, blur_px: blur, spread_px: spread })
}

fn split_lengths_and_color(layer: &str) -> Result<(&str, &str)> {
    let index = ["hsl", "oklch(", "color(", "#", "transparent"].iter().filter_map(|prefix| layer.find(prefix)).min();
    if let Some(index) = index {
        let (lengths, color) = layer.split_at(index);
        return Ok((lengths.trim(), color.trim()));
    }
    anyhow::bail!("unsupported shadow color syntax in `{layer}`")
}

fn next_length<'a, I>(parts: &mut I, label: &str) -> Result<f32>
where
    I: Iterator<Item = &'a str>,
{
    let raw = parts.next().with_context(|| format!("missing {label}"))?;
    parse_length_px(raw).with_context(|| format!("parse {label} `{raw}`"))
}

fn parse_length_px(raw: &str) -> Option<f32> {
    let value = raw.trim();
    if let Some(rem) = value.strip_suffix("rem") {
        return rem.trim().parse::<f32>().ok().map(|n| n * 16.0);
    }
    if let Some(px) = value.strip_suffix("px") {
        return px.trim().parse().ok();
    }
    value.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_shadow_layer() {
        let shadows = parse_box_shadow_value("0px 2px 0px 0px hsl(0 0% 20% / 0.07)").unwrap();
        assert_eq!(shadows.len(), 1);
        assert!(shadows[0].color.a > 0.0);
    }

    #[test]
    fn parses_multi_layer_shadow() {
        let raw = "0px 2px 0px 0px hsl(0 0% 20% / 0.15), 0px 1px 2px -1px hsl(0 0% 20% / 0.15)";
        let shadows = parse_box_shadow_value(raw).unwrap();
        assert_eq!(shadows.len(), 2);
    }

    #[test]
    fn shadow_ladder_preserves_parts_and_restores_scale() {
        let parts = ShadowTokenParts {
            color: ColorValue::srgb(0.12, 0.28, 0.28, 0.2),
            blur_px: 10.0,
            spread_px: -2.0,
            offset_x_px: 3.0,
            offset_y_px: 4.0,
        };
        let ladder = shadow_ladder_overrides(parts).unwrap();

        assert_eq!(ladder.len(), SHADOW_LADDER_TOKENS.len());
        assert!(ladder["shadow-2xs"].contains("0.1"));
        assert!(ladder["shadow"].contains(", 3px 4px 2px -1px"));
        assert!(ladder["shadow-xl"].contains("3px 32px 10px -1px"));
        assert!(ladder["shadow-2xl"].contains("0.5"));
    }
}

#[cfg(test)]
mod source_tests {
    use super::*;
    #[test]
    fn ladder_round_trip_preserves_wide_gamut_components_and_geometry() {
        let parts = ShadowTokenParts {
            color: ColorValue::display_p3(1.123456, -0.123456, 0.234567, 0.123456),
            blur_px: 10.123456,
            spread_px: -2.123456,
            offset_x_px: 1.123456,
            offset_y_px: 4.123456,
        };
        let css = parts.to_css_value().unwrap();
        assert_eq!(parse_shadow_layer(&css).unwrap(), parts);
        let ladder = shadow_ladder_overrides(parts).unwrap();
        assert_eq!(parse_shadow_layer(split_shadow_layers(&ladder["shadow"])[0]).unwrap(), parts);
        assert!(parse_box_shadow_value(&css).is_ok());
    }
    #[test]
    fn ladder_rejects_invalid_source_alpha_before_deriving_opacity() {
        let parts = ShadowTokenParts {
            color: ColorValue::display_p3(1.0, 0.0, 0.0, 2.0),
            blur_px: 10.0,
            spread_px: 0.0,
            offset_x_px: 0.0,
            offset_y_px: 1.0,
        };
        assert!(shadow_ladder_overrides(parts).is_err());
    }

    #[test]
    fn malformed_geometry_is_rejected() {
        for css in ["0px 1px 2px broken #000", "0px 1px -2px #000", "NaN 1px 2px #000"] {
            assert!(parse_box_shadow_value(css).is_err());
        }
    }
}
