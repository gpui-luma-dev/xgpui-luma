//! HSLA ingress for external stylesheets.
use palette::{Hsla, Srgba, convert::FromColorUnclamped};

/// Normalize authored colors to the source-color persistence representation.
/// Only palette/elevation subtrees are passed here; tagged source colors remain
/// atomic so their explicit color space and extended-range components survive.
pub(super) fn normalize(value: &mut toml::Value) -> Result<(), String> {
    match value {
        toml::Value::String(source) => {
            let components = parse(source)?;
            let mut color = toml::map::Map::new();
            color.insert("space".into(), toml::Value::String("srgb".into()));
            color.insert(
                "components".into(),
                toml::Value::Array(components.map(|n| toml::Value::Float(f64::from(n))).to_vec()),
            );
            *value = toml::Value::Table(color);
        }
        toml::Value::Table(table) if !table.contains_key("space") => {
            for (key, value) in table {
                normalize(value).map_err(|error| format!("{key}: {error}"))?;
            }
        }
        toml::Value::Array(values) => {
            for value in values {
                normalize(value)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn parse(source: &str) -> Result<[f32; 4], String> {
    let invalid = || format!("expected hsla(h, s%, l%, a), got {source:?}");
    let inner = source.trim().strip_prefix("hsla(").and_then(|s| s.strip_suffix(')')).ok_or_else(invalid)?;
    let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
    if parts.len() != 4 {
        return Err(invalid());
    }
    let number =
        |part: &str| -> Result<f32, String> { part.parse::<f32>().ok().filter(|n| n.is_finite()).ok_or_else(invalid) };
    let percentage = |part: &str| -> Result<f32, String> {
        let value = number(part.strip_suffix('%').ok_or_else(invalid)?)?;
        if !(0.0..=100.0).contains(&value) {
            return Err(invalid());
        }
        Ok(value / 100.0)
    };
    let hue = number(parts[0])?;
    let saturation = percentage(parts[1])?;
    let lightness = percentage(parts[2])?;
    let alpha = number(parts[3])?;
    if !(0.0..=1.0).contains(&alpha) {
        return Err(invalid());
    }
    let color = Srgba::from_color_unclamped(Hsla::new(hue, saturation, lightness, alpha));
    Ok([color.red, color.green, color.blue, color.alpha])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "the focus ring is red; value and design-rule checks move to a colorful fixture (#314)"]
    fn packaged_palette_is_monochrome_including_shell_and_data_roles() {
        fn check(value: &toml::Value) {
            if let Some(components) = value.get("components").and_then(toml::Value::as_array) {
                assert_eq!(components[0], components[1]);
                assert_eq!(components[1], components[2]);
            } else if let Some(table) = value.as_table() {
                for value in table.values() {
                    check(value);
                }
            }
        }
        let palettes = &super::super::TokenStylesheet::default().palette;
        check(&toml::Value::try_from(palettes).unwrap());
    }

    #[test]
    fn title_bar_keeps_dark_shell_in_both_modes() {
        let palettes = super::super::TokenStylesheet::default().palette;
        assert_eq!(palettes.light.shell, palettes.dark.shell);
        let palette = crate::theme::ThemeTokens::from_source(
            super::super::TokenStylesheet::default().source(crate::theme::ThemeMode::Dark),
            crate::color::GamutMapping::Clip,
        )
        .unwrap()
        .palette;
        assert!(palette.shell.background.l < 0.15);
        assert_eq!(palette.shell.foreground.l, 1.0);
    }

    #[test]
    fn readable_hsla_preserves_rgb8_and_alpha() {
        let rgba = parse("hsla(222.22, 47.37%, 11.18%, 0.2)").unwrap();
        assert_eq!(rgba[..3].iter().map(|n| (n * 255.0).round() as u8).collect::<Vec<_>>(), [15, 23, 42]);
        assert_eq!(rgba[3], 0.2);
        assert_eq!(parse("hsla(0, 0%, 0%, 0)").unwrap(), [0.0; 4]);
        assert_eq!(parse("hsla(360, 100%, 50%, 1)").unwrap(), [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn malformed_and_out_of_range_hsla_is_rejected() {
        for source in [
            "rgb(0, 0, 0)",
            "hsla(0, 0, 0, 1)",
            "hsla(0, 0%, 0%)",
            "hsla(nan, 0%, 0%, 1)",
            "hsla(0, 101%, 0%, 1)",
            "hsla(0, 0%, -1%, 1)",
            "hsla(0, 0%, 0%, 2)",
        ] {
            assert!(parse(source).is_err(), "accepted {source}");
        }
    }
}
