//! `.lss` colors as SDK source colors (issue #314).
//!
//! The style crate stores every color as unclamped OKLCH, whatever space the file wrote it in
//! (`gray`, `srgb`, `hsla`, `color(display_p3 …)`, `oklch`). Converting to [`ColorValue::Oklch`]
//! keeps the color itself, wide-gamut values included; only the space the file wrote it in is
//! not kept. Rendering uses the gamut-mapped preview, so that tag doesn't change what's drawn.

use super::ColorValue;

impl From<xgpui_luma_style::Color> for ColorValue {
    fn from(color: xgpui_luma_style::Color) -> Self {
        ColorValue::oklch(color.l, color.c, color.h, color.alpha)
    }
}

#[cfg(test)]
mod tests {
    use xgpui_luma_style::{Color, RgbSpace};

    use crate::color::{ColorValue, GamutMapping, gpui_bridge::to_rgba};

    fn rgba(color: ColorValue) -> [f32; 4] {
        let c = to_rgba(color, GamutMapping::CssLocalMinde).expect("maps");
        [c.r, c.g, c.b, c.a]
    }

    fn near(a: [f32; 4], b: [f32; 4], tolerance: f32) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() <= tolerance)
    }

    /// The same color written in each space, as the style crate and as an SDK source color.
    fn samples() -> Vec<(&'static str, Color, ColorValue)> {
        vec![
            ("srgb gray", Color::srgb(0.4686, 0.4686, 0.4686, 1.0), ColorValue::srgb(0.4686, 0.4686, 0.4686, 1.0)),
            ("srgb white", Color::srgb(1.0, 1.0, 1.0, 1.0), ColorValue::srgb(1.0, 1.0, 1.0, 1.0)),
            ("srgb black", Color::srgb(0.0, 0.0, 0.0, 1.0), ColorValue::srgb(0.0, 0.0, 0.0, 1.0)),
            ("srgb red", Color::srgb(1.0, 0.0, 0.0, 1.0), ColorValue::srgb(1.0, 0.0, 0.0, 1.0)),
            ("srgb translucent", Color::srgb(0.2, 0.5, 0.8, 0.12), ColorValue::srgb(0.2, 0.5, 0.8, 0.12)),
            (
                "srgb_linear",
                Color::rgb(RgbSpace::SrgbLinear, 0.2, 0.4, 0.05, 1.0),
                ColorValue::linear_srgb(0.2, 0.4, 0.05, 1.0),
            ),
            (
                "display_p3 in sRGB",
                Color::rgb(RgbSpace::DisplayP3, 0.5, 0.4, 0.3, 1.0),
                ColorValue::display_p3(0.5, 0.4, 0.3, 1.0),
            ),
            (
                "display_p3 red, outside sRGB",
                Color::rgb(RgbSpace::DisplayP3, 1.0, 0.0, 0.0, 1.0),
                ColorValue::display_p3(1.0, 0.0, 0.0, 1.0),
            ),
            (
                "display_p3 green, outside sRGB",
                Color::rgb(RgbSpace::DisplayP3, 0.0, 1.0, 0.0, 1.0),
                ColorValue::display_p3(0.0, 1.0, 0.0, 1.0),
            ),
            ("oklch in sRGB", Color::oklch(0.6, 0.1, 250.0, 1.0), ColorValue::oklch(0.6, 0.1, 250.0, 1.0)),
            ("oklch outside sRGB", Color::oklch(0.7, 0.3, 150.0, 1.0), ColorValue::oklch(0.7, 0.3, 150.0, 1.0)),
        ]
    }

    /// Converting keeps what's drawn: the converted color maps to the same sRGB as the color in its
    /// own space, and as the style crate's own mapping (both are CSS Color 4 local MINDE).
    #[test]
    fn converted_colors_draw_the_same_in_every_space() {
        for (what, lss, native) in samples() {
            let converted = rgba(ColorValue::from(lss));
            assert!(near(converted, rgba(native), 2e-3), "{what}: {converted:?} vs native {:?}", rgba(native));
            assert!(near(converted, lss.to_srgb(), 2e-3), "{what}: {converted:?} vs .lss {:?}", lss.to_srgb());
        }
    }

    /// sRGB colors survive the trip through OKLCH to well below one 8-bit step.
    #[test]
    fn srgb_colors_round_trip_closely() {
        for (what, lss, native) in samples().into_iter().filter(|(what, ..)| what.starts_with("srgb ")) {
            let converted = ColorValue::from(lss).to_srgba_unclamped().expect("converts");
            let original = native.to_srgba_unclamped().expect("converts");
            let [a, b] = [converted, original].map(|c| [c.red, c.green, c.blue, c.alpha]);
            assert!(near(a, b, 1e-4), "{what}: {a:?} vs {b:?}");
        }
    }

    /// Wide-gamut colors stay wide: the converted value is in sRGB and in Display P3 exactly when
    /// the original is (with a hair of tolerance for colors on a gamut edge), so a wide-gamut display
    /// can still show it.
    #[test]
    fn wide_gamut_colors_keep_their_gamut() {
        let inside = |c: [f32; 3]| c.iter().all(|v| (-1e-3..=1.0 + 1e-3).contains(v));
        let srgb = |c: ColorValue| {
            let c = c.to_srgba_unclamped().expect("converts");
            [c.red, c.green, c.blue]
        };
        let p3 = |c: ColorValue| {
            let c = c.to_display_p3_unclamped().expect("converts");
            [c.red, c.green, c.blue]
        };
        for (what, lss, native) in samples() {
            let converted = ColorValue::from(lss);
            assert_eq!(inside(srgb(converted)), inside(srgb(native)), "{what} in sRGB");
            assert_eq!(inside(p3(converted)), inside(p3(native)), "{what} in Display P3");
            let (a, b) = (p3(converted), p3(native));
            assert!(a.iter().zip(b).all(|(x, y)| (x - y).abs() <= 1e-3), "{what} in P3: {a:?} vs {b:?}");
        }
        let outside = samples().into_iter().filter(|(_, lss, _)| !inside(srgb(ColorValue::from(*lss)))).count();
        assert!(outside >= 3, "the samples include wide-gamut colors");
    }
}
