//! Rust port of Radix website's custom 12-step scale generator.
//!
//! Reference: radix-ui/website bb424082fd33fadc244a6dd276d3ced55caa6234,
//! `components/generate-radix-colors.tsx`, @radix-ui/colors 3.0.0, Color.js 0.5.2.
//! Uses the original P3 reference scales, Lab interpolation, OKLCH adjustments,
//! background easing. Derived Oklch values and original seeds are retained; the
//! SDK bridge supplies precomputed sRGB previews for GPUI.
//! See LICENSES.txt and tools/generate-custom-colors.mjs for attribution/fixtures.

// Authored RGB channels can happen to resemble mathematical constants.
#[allow(clippy::approx_constant)]
mod catalog;
mod color;
mod matrices;

use std::sync::LazyLock;
use gpui::Hsla;
use xgpui_luma::color::{ColorValue, GamutMapping, gpui_bridge};
use xgpui_luma::theme::ThemeMode;
use crate::ColorScale;
use color::Color;

type Scale = [Color; 12];
static LIGHT: LazyLock<[Scale; 29]> = LazyLock::new(|| catalog::LIGHT.map(|scale| scale.map(Color::from_p3)));
static DARK: LazyLock<[Scale; 29]> = LazyLock::new(|| catalog::DARK.map(|scale| scale.map(Color::from_p3)));

/// Opaque seed colors for a custom Radix palette. Appearance is supplied separately.
#[derive(Clone, Copy, Debug)]
pub struct CustomColors {
    pub accent: ColorValue,
    pub gray: ColorValue,
    pub background: ColorValue,
}

/// Retained generated scales with precomputed sRGB previews and the foreground for solid accent controls.
#[derive(Clone, Copy, Debug)]
pub struct GeneratedColors {
    pub accent: ColorScale,
    pub gray: ColorScale,
    pub accent_contrast: Hsla,
    pub accent_contrast_source: ColorValue,
    pub inputs: CustomColors,
}

/// Generate a custom palette using Radix's perceptual scale adaptation algorithm.
/// Seeds must be opaque; invalid colors return an error. No 8-bit rounding is performed.
pub fn generate_colors(inputs: CustomColors, mode: ThemeMode) -> anyhow::Result<GeneratedColors> {
    for input in [inputs.accent, inputs.gray, inputs.background] {
        input.validate()?;
        anyhow::ensure!(input.alpha() == 1.0, "Radix custom color seeds must be opaque");
    }
    let scales = match mode {
        ThemeMode::Light => &*LIGHT,
        ThemeMode::Dark => &*DARK,
    };
    let background = Color::from_source(inputs.background)?;
    let source = Color::from_source(inputs.accent)?;
    let gray = scale_from_color(Color::from_source(inputs.gray)?, &scales[..6], background, mode);
    let mut accent = scale_from_color(source, scales, background, mode);
    if source.l <= 0.000001 || source.l >= 0.999999 {
        accent = gray;
    }
    // Near-background seeds need a useful solid rather than white-on-white/black-on-black.
    let keeps_seed = source.distance(accent[0]) * 100.0 >= 25.0;
    if keeps_seed {
        accent[8] = source;
    }
    let accent_contrast_source = accent[8].text_color().source();
    let accent_contrast = gpui_bridge::to_hsla(accent_contrast_source, GamutMapping::CssLocalMinde)?;
    accent[9] = button_hover(accent[8], &accent);
    let max_text_chroma = accent[8].c.max(accent[7].c);
    accent[10].c = accent[10].c.min(max_text_chroma);
    accent[11].c = accent[11].c.min(max_text_chroma);
    let mut accent_sources = accent.map(Color::source);
    if keeps_seed {
        accent_sources[8] = inputs.accent;
    }
    Ok(GeneratedColors {
        accent: ColorScale::new(accent_sources, GamutMapping::CssLocalMinde)?,
        gray: ColorScale::new(gray.map(Color::source), GamutMapping::CssLocalMinde)?,
        accent_contrast,
        accent_contrast_source,
        inputs,
    })
}

fn scale_from_color(source: Color, scales: &[Scale], background: Color, mode: ThemeMode) -> Scale {
    // One closest swatch per reference family; stable sorting preserves upstream ties.
    let mut nearest: Vec<(usize, Color, f64)> = scales
        .iter()
        .enumerate()
        .map(|(index, scale)| {
            let mut closest = scale[0];
            let mut distance = source.distance(closest);
            for &color in &scale[1..] {
                let d = source.distance(color);
                if d < distance {
                    closest = color;
                    distance = d;
                }
            }
            (index, closest, distance)
        })
        .collect();
    nearest.sort_by(|a, b| a.2.total_cmp(&b.2));
    if scales.len() > 6 && nearest[0].0 < 6 {
        while nearest[1].0 < 6 {
            nearest.remove(1);
        }
    }
    let (index_a, color_a, b) = nearest[0];
    let (index_b, color_b, a) = nearest[1];
    let c = color_a.distance(color_b);
    // The reference's trigonometric ratio is undefined for an exact match or a
    // degenerate triangle. In those cases choose the nearest scale directly.
    let ratio = if b < 1e-12 || c < 1e-12 {
        0.0
    } else {
        let cos_a = ((b * b + c * c - a * a) / (2.0 * b * c)).clamp(-1.0, 1.0);
        let cos_b = ((a * a + c * c - b * b) / (2.0 * a * c)).clamp(-1.0, 1.0);
        let value = (cos_a / cos_a.acos().sin()) / (cos_b / cos_b.acos().sin());
        if value.is_finite() { value.max(0.0) * 0.5 } else { 0.0 }
    };
    let mut scale: Scale = std::array::from_fn(|i| scales[index_a][i].mix(scales[index_b][i], ratio));
    let mut closest = scale[0];
    for &color in &scale[1..] {
        if source.distance(color) < source.distance(closest) {
            closest = color;
        }
    }
    let chroma_ratio = if closest.c > 1e-12 { source.c / closest.c } else { 0.0 };
    for color in &mut scale {
        color.c = (source.c * 1.5).min(color.c * chroma_ratio);
        color.h = source.h;
    }
    let background_l = background.l.clamp(0.0, 1.0);
    match mode {
        ThemeMode::Light => {
            // The website prepends white before transposing, then removes that step.
            for (i, color) in scale.iter_mut().enumerate() {
                color.l -= (1.0 - background_l) * bezier(1.0 - (i + 1) as f64 / 12.0, [0.0, 2.0, 0.0, 2.0]);
            }
        }
        ThemeMode::Dark => {
            let first = scale[0].l;
            let ratio = background_l / first;
            let x = if ratio > 1.5 {
                0.0
            } else if ratio > 1.0 {
                (1.0 - (ratio - 1.0) * 3.0).max(0.0)
            } else {
                1.0
            };
            for (i, color) in scale.iter_mut().enumerate() {
                color.l -= (first - background_l) * bezier(1.0 - i as f64 / 11.0, [x, 0.0, x, 0.0]);
            }
        }
    }
    scale
}

fn bezier(x: f64, [x1, y1, x2, y2]: [f64; 4]) -> f64 {
    if x == 0.0 || x == 1.0 {
        return x;
    }
    if x1 == y1 && x2 == y2 {
        return x;
    }
    let sample = |t: f64, a: f64, b: f64| 3.0 * (1.0 - t).powi(2) * t * a + 3.0 * (1.0 - t) * t * t * b + t * t * t;
    let (mut low, mut high) = (0.0, 1.0);
    for _ in 0..48 {
        let mid = (low + high) * 0.5;
        if sample(mid, x1, x2) < x {
            low = mid;
        } else {
            high = mid;
        }
    }
    sample((low + high) * 0.5, y1, y2)
}

fn button_hover(source: Color, scale: &Scale) -> Color {
    let l = if source.l > 0.4 {
        source.l - 0.03 / (source.l + 0.1)
    } else {
        source.l + 0.03 / (source.l + 0.1)
    };
    let c = if source.l > 0.4 && source.h.is_finite() {
        source.c * 0.93
    } else {
        source.c
    };
    let hover = Color { l, c, h: source.h };
    let mut closest = scale[0];
    for &color in &scale[1..] {
        if hover.distance(color) < hover.distance(closest) {
            closest = color;
        }
    }
    Color { l, c: closest.c, h: closest.h }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture {
        name: &'static str,
        mode: ThemeMode,
        inputs: [u32; 3],
        accent: [u32; 12],
        gray: [u32; 12],
        contrast: u32,
    }
    include!("fixtures.rs");
    fn channels(color: Hsla) -> [i32; 3] {
        let color: gpui::Rgba = color.into();
        [color.r, color.g, color.b].map(|c| (c * 255.0).round() as i32)
    }
    fn check(actual: Hsla, expected: u32, label: &str) {
        let actual = channels(actual);
        let expected = channels(gpui::rgb(expected).into());
        for (a, b) in actual.iter().zip(expected) {
            assert!((a - b).abs() <= 1, "{label}: {actual:?} != {expected:?}");
        }
    }
    #[test]
    fn p3_and_oklch_seeds_survive_generation_without_quantization() {
        for accent in [ColorValue::display_p3(1.0, 0.0, 0.0, 1.0), ColorValue::oklch(0.65, 0.35, 725.0, 1.0)] {
            let inputs = CustomColors {
                accent,
                gray: ColorValue::srgb(0.5, 0.5, 0.5, 1.0),
                background: ColorValue::srgb(1.0, 1.0, 1.0, 1.0),
            };
            let colors = generate_colors(inputs, ThemeMode::Light).unwrap();
            assert_eq!(colors.inputs.accent, accent);
            assert_eq!(colors.accent.source_step(9), accent);
            assert!(!colors.accent.source_step(9).is_in_gamut(xgpui_luma::color::Gamut::Srgb).unwrap());
            assert_eq!(colors.accent.step(9), gpui_bridge::to_hsla(accent, GamutMapping::CssLocalMinde).unwrap());
            let rgb: gpui::Rgba = colors.accent.step(3).into();
            assert!([rgb.r, rgb.g, rgb.b].iter().any(|value| (value * 255.0 - (value * 255.0).round()).abs() > 0.01));
        }
    }

    #[test]
    fn invalid_or_transparent_generator_seeds_are_rejected() {
        for accent in [ColorValue::srgb(f32::NAN, 0.0, 0.0, 1.0), ColorValue::display_p3(1.0, 0.0, 0.0, 0.5)] {
            assert!(
                generate_colors(
                    CustomColors {
                        accent,
                        gray: ColorValue::srgb(0.5, 0.5, 0.5, 1.0),
                        background: ColorValue::srgb(1.0, 1.0, 1.0, 1.0)
                    },
                    ThemeMode::Light
                )
                .is_err()
            );
        }
    }

    #[test]
    fn matches_upstream_javascript_srgb_vectors() {
        for fixture in CASES {
            let [accent, gray, background] = fixture.inputs.map(|v| gpui_bridge::from_rgba(gpui::rgb(v)));
            let result = generate_colors(CustomColors { accent, gray, background }, fixture.mode).unwrap();
            let label = format!("{} {:?} {:?}", fixture.name, fixture.mode, fixture.inputs);
            for i in 0..12 {
                check(result.accent.step(i as u8 + 1), fixture.accent[i], &format!("{label} accent {}", i + 1));
                check(result.gray.step(i as u8 + 1), fixture.gray[i], &format!("{label} gray {}", i + 1));
            }
            check(result.accent_contrast, fixture.contrast, &format!("{label} contrast"));
        }
    }
    #[test]
    fn reference_seeds_and_extremes_stay_finite() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let references = match mode {
                ThemeMode::Light => &*LIGHT,
                ThemeMode::Dark => &*DARK,
            };
            for accent in references
                .iter()
                .flatten()
                .map(|c| c.source())
                .chain([ColorValue::srgb(0.0, 0.0, 0.0, 1.0), ColorValue::srgb(1.0, 1.0, 1.0, 1.0)])
            {
                for background in [ColorValue::srgb(0.0, 0.0, 0.0, 1.0), ColorValue::srgb(1.0, 1.0, 1.0, 1.0)] {
                    let result = generate_colors(CustomColors { accent, gray: accent, background }, mode).unwrap();
                    for step in 1..=12 {
                        for c in [result.accent.step(step), result.gray.step(step), result.accent_contrast] {
                            assert!(
                                [c.h, c.s, c.l, c.a].iter().all(|v| v.is_finite() && *v >= 0.0 && *v <= 1.0),
                                "{mode:?} seed {accent:?} bg {background:?} step {step}: {c:?}"
                            );
                        }
                    }
                }
            }
        }
    }
}
