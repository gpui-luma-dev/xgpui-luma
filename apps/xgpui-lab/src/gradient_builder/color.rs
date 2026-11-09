use gpui::{Hsla, hsla};
use palette::{FromColor, Hsl, Mix, Srgba};

use super::paint::GradientType;
use xgpui_luma::color::{ColorValue, Gamut};

#[allow(dead_code)]
pub fn parse_hex_color(raw: &str) -> Option<Hsla> {
    let hex = raw.trim().trim_start_matches('#');
    let rgba = match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Srgba::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Srgba::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
        }
        _ => return None,
    };
    Some(srgba_to_hsla(rgba))
}

pub fn interpolate_rgb(start: Hsla, end: Hsla, t: f32) -> Hsla {
    let mixed = hsla_to_srgba(start).mix(hsla_to_srgba(end), t.clamp(0.0, 1.0));
    srgba_to_hsla(mixed)
}

pub fn format_percent(position: f32) -> String {
    format!("{}%", (position * 100.0).round() as i32)
}

/// CSS color for a source stop: hex for sRGB colors, otherwise the source's own
/// CSS (e.g. `color(display-p3 1 0 0)`) so wide colors are not clipped.
pub fn format_css_color(color: ColorValue) -> String {
    match color {
        ColorValue::Srgb(c) if color.is_in_gamut(Gamut::Srgb).unwrap_or(false) && c.alpha >= 1.0 => format!(
            "#{:02x}{:02x}{:02x}",
            (c.red * 255.0).round() as u8,
            (c.green * 255.0).round() as u8,
            (c.blue * 255.0).round() as u8,
        ),
        _ => color.to_css().unwrap_or_else(|_| "transparent".into()),
    }
}

fn css_stop_list(stops: &[(f32, ColorValue)]) -> String {
    stops
        .iter()
        .map(|(position, color)| format!("{} {}", format_css_color(*color), format_percent(*position)))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn format_css_linear_gradient(stops: &[(f32, ColorValue)], rotation_deg: f32) -> String {
    if stops.is_empty() {
        return "linear-gradient(90deg, transparent 0%, transparent 100%)".to_string();
    }
    format!("linear-gradient({}deg, {})", rotation_deg.round() as i32, css_stop_list(stops))
}

pub fn format_css_radial_gradient(stops: &[(f32, ColorValue)]) -> String {
    if stops.is_empty() {
        return "radial-gradient(circle, transparent 0%, transparent 100%)".to_string();
    }
    format!("radial-gradient(circle, {})", css_stop_list(stops))
}

pub fn format_css_conic_gradient(stops: &[(f32, ColorValue)], rotation_deg: f32) -> String {
    if stops.is_empty() {
        return format!("conic-gradient(from {}deg, transparent 0%, transparent 100%)", rotation_deg.round() as i32);
    }
    format!("conic-gradient(from {}deg, {})", rotation_deg.round() as i32, css_stop_list(stops))
}

pub fn format_css_gradient(gradient_type: GradientType, stops: &[(f32, ColorValue)], rotation_deg: f32) -> String {
    match gradient_type {
        GradientType::Linear => format_css_linear_gradient(stops, rotation_deg),
        GradientType::Radial => format_css_radial_gradient(stops),
        GradientType::Conic => format_css_conic_gradient(stops, rotation_deg),
    }
}

fn hsla_to_srgba(color: Hsla) -> Srgba {
    let rgb = color.to_rgb();
    Srgba::new(rgb.r, rgb.g, rgb.b, color.a)
}

fn srgba_to_hsla(rgba: Srgba) -> Hsla {
    let hsl: Hsl = Hsl::from_color(rgba);
    hsla(hsl.hue.into_positive_degrees() / 360.0, hsl.saturation, hsl.lightness, rgba.alpha)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_export_keeps_wide_stops_and_hex_for_srgb() {
        let stops = [(0.0, ColorValue::srgb(1.0, 0.0, 0.0, 1.0)), (1.0, ColorValue::display_p3(0.0, 1.0, 0.0, 1.0))];
        let css = format_css_gradient(GradientType::Linear, &stops, 90.0);
        assert!(css.starts_with("linear-gradient(90deg, #ff0000 0%, color(display-p3 "), "{css}");
        assert!(css.ends_with(" 100%)"), "{css}");
    }
}
