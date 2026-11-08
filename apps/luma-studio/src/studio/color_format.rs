use gpui::Hsla;
use xgpui_luma::color::{ColorValue, GamutMapping, gpui_bridge};

/// Project only the Studio readout; the model retains its original source.
pub(crate) fn preview_color(color: ColorValue) -> Hsla {
    gpui_bridge::to_hsla(color, GamutMapping::CssLocalMinde).unwrap_or_else(|_| gpui::transparent_black())
}

/// Keep Studio's established compact HSL presentation separate from storage syntax.
pub(crate) fn format_color_readout(color: ColorValue) -> String {
    format_compact_hsla(preview_color(color))
}

/// Accept the established HSL input, with source parsing handled internally.
pub(crate) fn parse_color_input(raw: &str) -> Option<ColorValue> {
    parse_compact_hsla(raw).map(gpui_bridge::from_hsla).or_else(|| ColorValue::parse_css(raw).ok())
}

/// Formats an ordinary theme color for Luma Studio's compact readouts.
pub(crate) fn format_compact_hsla(color: Hsla) -> String {
    format!(
        "hsl({} {}% {}% / {})",
        rounded_channel(color.h.rem_euclid(1.0) * 360.0),
        rounded_channel(color.s.clamp(0.0, 1.0) * 100.0),
        rounded_channel(color.l.clamp(0.0, 1.0) * 100.0),
        compact_alpha(color.a),
    )
}

/// Parses the compact HSLA notation used by the Luma Studio theme color fields.
pub(crate) fn parse_compact_hsla(raw: &str) -> Option<Hsla> {
    let trimmed = raw.trim();
    let inner = if trimmed.len() >= 5 && trimmed[..4].eq_ignore_ascii_case("hsl(") && trimmed.ends_with(')') {
        &trimmed[4..trimmed.len() - 1]
    } else if trimmed.len() >= 6 && trimmed[..5].eq_ignore_ascii_case("hsla(") && trimmed.ends_with(')') {
        &trimmed[5..trimmed.len() - 1]
    } else {
        return None;
    };

    let normalized = inner.replace(',', " ");
    let (channels, alpha) = normalized
        .split_once('/')
        .map_or((normalized.as_str(), None), |(channels, alpha)| (channels, Some(alpha.trim())));
    let mut parts = channels.split_whitespace();
    let hue = parse_finite(parts.next()?)? / 360.0;
    let saturation = parse_percent(parts.next()?)?;
    let lightness = parse_percent(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }

    let alpha = alpha.map(parse_finite).unwrap_or(Some(1.0))?;

    Some(Hsla {
        h: hue.rem_euclid(1.0),
        s: saturation.clamp(0.0, 1.0),
        l: lightness.clamp(0.0, 1.0),
        a: alpha.clamp(0.0, 1.0),
    })
}

fn parse_finite(value: &str) -> Option<f32> {
    value.parse::<f32>().ok().filter(|value| value.is_finite())
}

fn parse_percent(value: &str) -> Option<f32> {
    value.strip_suffix('%').and_then(parse_finite).map(|value| value / 100.0)
}

fn rounded_channel(value: f32) -> i32 {
    value.round() as i32
}

fn compact_alpha(alpha: f32) -> String {
    let rounded = (alpha.clamp(0.0, 1.0) * 100.0).round() / 100.0;
    if (rounded - rounded.round()).abs() <= f32::EPSILON {
        format!("{}", rounded.round() as i32)
    } else {
        format!("{rounded:.2}").trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Hsla, hsla};

    use super::{format_compact_hsla, parse_compact_hsla, format_color_readout, parse_color_input};
    use xgpui_luma::color::{ColorValue, gpui_bridge};

    #[test]
    fn source_storage_syntax_does_not_appear_in_readouts() {
        let source = ColorValue::display_p3(1.123456, -0.123456, 0.234567, 0.345678);
        let readout = format_color_readout(source);
        assert!(readout.starts_with("hsl("));
        assert!(!readout.contains("color("));
        assert_eq!(
            parse_color_input("hsl(-30 120% -10%)"),
            parse_compact_hsla("hsl(-30 120% -10%)").map(gpui_bridge::from_hsla)
        );
    }

    #[test]
    fn formats_ordinary_color() {
        assert_eq!(format_compact_hsla(hsla(210.0 / 360.0, 0.5, 0.4, 0.75)), "hsl(210 50% 40% / 0.75)");
    }

    #[test]
    fn normalizes_hue_and_clamps_channels() {
        assert_eq!(format_compact_hsla(Hsla { h: -30.0 / 360.0, s: 1.2, l: -0.1, a: 1.2 }), "hsl(330 100% 0% / 1)");
    }

    #[test]
    fn formats_grayscale_and_transparent_colors() {
        assert_eq!(format_compact_hsla(hsla(0.0, 0.0, 0.5, 0.0)), "hsl(0 0% 50% / 0)");
    }

    #[test]
    fn parses_compact_hsla() {
        let color = parse_compact_hsla("hsla(210 50% 40% / 0.75)").expect("valid HSLA");
        assert_eq!(format_compact_hsla(color), "hsl(210 50% 40% / 0.75)");
    }

    #[test]
    fn parses_clamped_and_wrapped_hsla() {
        let color = parse_compact_hsla("hsl(-30 120% -10%)").expect("valid HSL");
        assert_eq!(format_compact_hsla(color), "hsl(330 100% 0% / 1)");
    }

    #[test]
    fn rounds_boundary_values_compactly() {
        assert_eq!(format_compact_hsla(hsla(0.999, 0.1234, 0.9876, 0.126)), "hsl(360 12% 99% / 0.13)");
    }
}
