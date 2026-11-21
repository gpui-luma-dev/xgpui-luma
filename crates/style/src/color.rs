//! Colors (CSS Color Level 4): stored as OKLCH, mixed in OKLab, written out as gamut-mapped sRGB.

use std::f32::consts::PI;

/// An OKLCH color with alpha. Values are not clamped, so wide-gamut colors keep their value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    /// Lightness, 0 to 1.
    pub l: f32,
    /// Chroma, 0 to about 0.4.
    pub c: f32,
    /// Hue in degrees, 0 to 360.
    pub h: f32,
    /// Alpha, 0 to 1.
    pub alpha: f32,
}

/// The predefined RGB spaces of `color()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RgbSpace {
    Srgb,
    SrgbLinear,
    DisplayP3,
    A98Rgb,
    ProphotoRgb,
    Rec2020,
}

impl RgbSpace {
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "srgb" => Self::Srgb,
            "srgb_linear" => Self::SrgbLinear,
            "display_p3" => Self::DisplayP3,
            "a98_rgb" => Self::A98Rgb,
            "prophoto_rgb" => Self::ProphotoRgb,
            "rec2020" => Self::Rec2020,
            _ => return None,
        })
    }
}

type Mat3 = [[f64; 3]; 3];

fn mul(m: &Mat3, v: [f64; 3]) -> [f64; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

// CSS Color 4 conversion matrices (linear RGB → CIE XYZ, D65 unless noted).
const P3_TO_XYZ: Mat3 = [
    [0.486_570_948_648_216_2, 0.265_667_693_169_093_06, 0.198_217_285_234_362_5],
    [0.228_974_564_069_748_8, 0.691_738_521_836_506_4, 0.079_286_914_093_745],
    [0.0, 0.045_113_381_858_902_64, 1.043_944_368_900_976],
];
const A98_TO_XYZ: Mat3 = [
    [0.576_669_042_910_130_5, 0.185_558_237_906_546_3, 0.188_228_646_234_994_7],
    [0.297_344_975_250_536_05, 0.627_363_566_255_466_1, 0.075_291_458_493_997_88],
    [0.027_031_361_386_412_34, 0.070_688_852_535_827_23, 0.991_337_536_837_638_8],
];
const REC2020_TO_XYZ: Mat3 = [
    [0.636_958_048_301_291_4, 0.144_616_903_586_208_32, 0.168_880_975_164_172_1],
    [0.262_700_212_011_267_1, 0.677_998_071_518_870_8, 0.059_301_716_469_861_96],
    [0.0, 0.028_072_693_049_087_428, 1.060_985_057_710_791],
];
// ProPhoto is D50; adapt to D65 with Bradford afterwards.
const PROPHOTO_TO_XYZ_D50: Mat3 = [
    [0.797_760_489_672_302_7, 0.135_185_837_175_740_31, 0.031_349_349_581_524_8],
    [0.288_071_128_229_293_4, 0.711_843_217_810_101_4, 0.000_085_653_960_605_259],
    [0.0, 0.0, 0.825_104_602_510_460_1],
];
const D50_TO_D65: Mat3 = [
    [0.955_473_452_704_218_2, -0.023_098_536_874_261_423, 0.063_259_308_661_021_7],
    [-0.028_369_706_963_208_136, 1.009_995_458_005_822_6, 0.021_041_398_966_943_008],
    [0.012_314_001_688_319_899, -0.020_507_696_433_477_912, 1.330_365_936_608_075_3],
];
const XYZ_TO_LINEAR_SRGB: Mat3 = [
    [3.240_969_941_904_522_6, -1.537_383_177_570_094, -0.498_610_760_293_003_4],
    [-0.969_243_636_280_879_6, 1.875_967_501_507_720_2, 0.041_555_057_407_175_59],
    [0.055_630_079_696_993_66, -0.203_976_958_888_976_52, 1.056_971_514_242_878_6],
];

fn srgb_to_linear(c: f64) -> f64 {
    let a = c.abs();
    let v = if a <= 0.040_45 {
        a / 12.92
    } else {
        ((a + 0.055) / 1.055).powf(2.4)
    };
    v.copysign(c)
}

fn linear_to_srgb(c: f64) -> f64 {
    let a = c.abs();
    let v = if a <= 0.003_130_8 {
        a * 12.92
    } else {
        1.055 * a.powf(1.0 / 2.4) - 0.055
    };
    v.copysign(c)
}

fn a98_to_linear(c: f64) -> f64 {
    c.abs().powf(563.0 / 256.0).copysign(c)
}

fn prophoto_to_linear(c: f64) -> f64 {
    let a = c.abs();
    let v = if a <= 16.0 / 512.0 { a / 16.0 } else { a.powf(1.8) };
    v.copysign(c)
}

/// CSS Color 4 uses the display-referred BT.1886 curve (gamma 2.4) for `rec2020`.
fn rec2020_to_linear(c: f64) -> f64 {
    c.abs().powf(2.4).copysign(c)
}

/// Linear sRGB → OKLab (Björn Ottosson).
fn linear_srgb_to_oklab([r, g, b]: [f64; 3]) -> [f64; 3] {
    let l = 0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b;
    let m = 0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b;
    let s = 0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b;
    let (l, m, s) = (l.cbrt(), m.cbrt(), s.cbrt());
    [
        0.210_454_255_3 * l + 0.793_617_785_0 * m - 0.004_072_046_8 * s,
        1.977_998_495_1 * l - 2.428_592_205_0 * m + 0.450_593_709_9 * s,
        0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766_0 * s,
    ]
}

fn oklab_to_linear_srgb([l, a, b]: [f64; 3]) -> [f64; 3] {
    let l_ = l + 0.396_337_777_4 * a + 0.215_803_757_3 * b;
    let m_ = l - 0.105_561_345_8 * a - 0.063_854_172_8 * b;
    let s_ = l - 0.089_484_177_5 * a - 1.291_485_548_0 * b;
    let (l3, m3, s3) = (l_ * l_ * l_, m_ * m_ * m_, s_ * s_ * s_);
    [
        4.076_741_662_1 * l3 - 3.307_711_591_3 * m3 + 0.230_969_929_2 * s3,
        -1.268_438_004_6 * l3 + 2.609_757_401_1 * m3 - 0.341_319_396_5 * s3,
        -0.004_196_086_3 * l3 - 0.703_418_614_7 * m3 + 1.707_614_701_0 * s3,
    ]
}

impl Color {
    pub const TRANSPARENT: Color = Color { l: 0.0, c: 0.0, h: 0.0, alpha: 0.0 };
    pub const BLACK: Color = Color { l: 0.0, c: 0.0, h: 0.0, alpha: 1.0 };
    pub const WHITE: Color = Color { l: 1.0, c: 0.0, h: 0.0, alpha: 1.0 };

    pub fn oklch(l: f32, c: f32, h: f32, alpha: f32) -> Self {
        Self { l, c, h: h.rem_euclid(360.0), alpha }
    }

    pub fn oklab(l: f32, a: f32, b: f32, alpha: f32) -> Self {
        let c = (a * a + b * b).sqrt();
        let h = if c < 1e-6 { 0.0 } else { b.atan2(a) * 180.0 / PI };
        Self::oklch(l, c, h, alpha)
    }

    /// Gamma-encoded sRGB, components 0 to 1.
    pub fn srgb(r: f32, g: f32, b: f32, alpha: f32) -> Self {
        Self::rgb(RgbSpace::Srgb, r, g, b, alpha)
    }

    /// `color(<space> r g b / alpha)`.
    pub fn rgb(space: RgbSpace, r: f32, g: f32, b: f32, alpha: f32) -> Self {
        let v = [r as f64, g as f64, b as f64];
        let linear_srgb = match space {
            RgbSpace::Srgb => v.map(srgb_to_linear),
            RgbSpace::SrgbLinear => v,
            RgbSpace::DisplayP3 => mul(&XYZ_TO_LINEAR_SRGB, mul(&P3_TO_XYZ, v.map(srgb_to_linear))),
            RgbSpace::A98Rgb => mul(&XYZ_TO_LINEAR_SRGB, mul(&A98_TO_XYZ, v.map(a98_to_linear))),
            RgbSpace::ProphotoRgb => {
                mul(&XYZ_TO_LINEAR_SRGB, mul(&D50_TO_D65, mul(&PROPHOTO_TO_XYZ_D50, v.map(prophoto_to_linear))))
            }
            RgbSpace::Rec2020 => mul(&XYZ_TO_LINEAR_SRGB, mul(&REC2020_TO_XYZ, v.map(rec2020_to_linear))),
        };
        let [l, a, b] = linear_srgb_to_oklab(linear_srgb);
        Self::oklab(l as f32, a as f32, b as f32, alpha)
    }

    /// `hsla(h, s, l, a)` with hue in degrees and saturation and lightness 0 to 1 (sRGB).
    pub fn hsla(h: f32, s: f32, l: f32, alpha: f32) -> Self {
        let f = |n: f32| {
            let k = (n + h / 30.0).rem_euclid(12.0);
            let a = s * l.min(1.0 - l);
            l - a * (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0)
        };
        Self::srgb(f(0.0), f(8.0), f(4.0), alpha)
    }

    /// `#rrggbb` or `#rrggbbaa`.
    pub fn parse_hex(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix('#')?;
        let byte = |i: usize| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok().map(|v| v as f32 / 255.0);
        let alpha = if hex.len() == 8 { byte(6)? } else { 1.0 };
        if hex.len() != 6 && hex.len() != 8 {
            return None;
        }
        Some(Self::srgb(byte(0)?, byte(2)?, byte(4)?, alpha))
    }

    pub fn with_alpha(self, alpha: f32) -> Self {
        Self { alpha, ..self }
    }

    pub fn to_oklab(self) -> [f32; 3] {
        let h = self.h * PI / 180.0;
        [self.l, self.c * h.cos(), self.c * h.sin()]
    }

    fn linear_srgb(self) -> [f64; 3] {
        let [l, a, b] = self.to_oklab();
        oklab_to_linear_srgb([l as f64, a as f64, b as f64])
    }

    /// Gamma-encoded sRGB without gamut mapping (components may fall outside 0 to 1).
    pub fn to_srgb_unclamped(self) -> [f32; 4] {
        let [r, g, b] = self.linear_srgb().map(linear_to_srgb);
        [r as f32, g as f32, b as f32, self.alpha]
    }

    pub fn in_srgb_gamut(self) -> bool {
        const EPS: f32 = 1e-4;
        self.to_srgb_unclamped()[..3].iter().all(|c| (-EPS..=1.0 + EPS).contains(c))
    }

    /// sRGB for display, gamut-mapped with the CSS Color 4 algorithm (reduce chroma until it fits).
    pub fn to_srgb(self) -> [f32; 4] {
        let clip = |c: Color| {
            let [r, g, b, a] = c.to_srgb_unclamped();
            [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0), a]
        };
        if self.l >= 1.0 {
            return [1.0, 1.0, 1.0, self.alpha];
        }
        if self.l <= 0.0 {
            return [0.0, 0.0, 0.0, self.alpha];
        }
        if self.in_srgb_gamut() {
            return clip(self);
        }
        const JND: f32 = 0.02;
        const EPSILON: f32 = 0.0001;
        let delta = |a: Color, b: [f32; 4]| {
            let [l1, a1, b1] = a.to_oklab();
            let other = Color::srgb(b[0], b[1], b[2], b[3]);
            let [l2, a2, b2] = other.to_oklab();
            ((l1 - l2).powi(2) + (a1 - a2).powi(2) + (b1 - b2).powi(2)).sqrt()
        };
        let mut current = self;
        let mut clipped = clip(current);
        if delta(current, clipped) < JND {
            return clipped;
        }
        let (mut min, mut max) = (0.0, self.c);
        let mut min_in_gamut = true;
        while max - min > EPSILON {
            let chroma = (min + max) / 2.0;
            current.c = chroma;
            if min_in_gamut && current.in_srgb_gamut() {
                min = chroma;
                continue;
            }
            clipped = clip(current);
            let e = delta(current, clipped);
            if e < JND {
                if JND - e < EPSILON {
                    return clipped;
                }
                min_in_gamut = false;
                min = chroma;
            } else {
                max = chroma;
            }
        }
        clipped
    }

    /// HSLA for gpui (hue 0 to 1), from the gamut-mapped sRGB value.
    pub fn to_hsla(self) -> [f32; 4] {
        let [r, g, b, a] = self.to_srgb();
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;
        if (max - min).abs() < 1e-6 {
            return [0.0, 0.0, l, a];
        }
        let d = max - min;
        let s = if l > 0.5 {
            d / (2.0 - max - min)
        } else {
            d / (max + min)
        };
        let h = if max == r {
            (g - b) / d + if g < b { 6.0 } else { 0.0 }
        } else if max == g {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };
        [h / 6.0, s, l, a]
    }

    /// Mix in OKLab (the CSS default for `color-mix`). `t = 0` is `self`, `t = 1` is `other`.
    pub fn mix(self, other: Color, t: f32) -> Color {
        let [l1, a1, b1] = self.to_oklab();
        let [l2, a2, b2] = other.to_oklab();
        let lerp = |x: f32, y: f32| x + (y - x) * t;
        let mut c = Color::oklab(lerp(l1, l2), lerp(a1, a2), lerp(b1, b2), lerp(self.alpha, other.alpha));
        // Keep the chromatic side's hue when mixing with an achromatic color.
        if self.c < 1e-6 && other.c >= 1e-6 {
            c.h = other.h;
        } else if other.c < 1e-6 && self.c >= 1e-6 {
            c.h = self.h;
        }
        c
    }

    /// True when the two colors are within `tolerance` in OKLab and alpha.
    pub fn approx_eq(self, other: Color, tolerance: f32) -> bool {
        let [l1, a1, b1] = self.to_oklab();
        let [l2, a2, b2] = other.to_oklab();
        (l1 - l2).abs() <= tolerance
            && (a1 - a2).abs() <= tolerance
            && (b1 - b2).abs() <= tolerance
            && (self.alpha - other.alpha).abs() <= tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn srgb_red_matches_the_css_reference() {
        let red = Color::srgb(1.0, 0.0, 0.0, 1.0);
        assert!((red.l - 0.627_955_4).abs() < 1e-4, "{red:?}");
        assert!((red.c - 0.257_683_3).abs() < 1e-4, "{red:?}");
        assert!((red.h - 29.233_885).abs() < 1e-2, "{red:?}");
    }

    #[test]
    fn the_same_color_in_five_spaces_is_one_color() {
        // CSS Color 4 example: one color written in five predefined spaces.
        let srgb = Color::rgb(RgbSpace::Srgb, 0.41587, 0.503670, 0.36664, 1.0);
        for (space, rgb) in [
            (RgbSpace::DisplayP3, [0.43313, 0.50108, 0.37950]),
            (RgbSpace::A98Rgb, [0.44091, 0.49971, 0.37408]),
            (RgbSpace::ProphotoRgb, [0.36589, 0.41717, 0.31333]),
            (RgbSpace::Rec2020, [0.47428, 0.52316, 0.41425]),
        ] {
            let other = Color::rgb(space, rgb[0], rgb[1], rgb[2], 1.0);
            assert!(srgb.approx_eq(other, 2e-3), "{space:?}: {srgb:?} vs {other:?}");
        }
    }

    #[test]
    fn hex_round_trips_through_oklch() {
        let c = Color::parse_hex("#1e293b").expect("hex");
        let [r, g, b, a] = c.to_srgb();
        assert!((r - 30.0 / 255.0).abs() < 1e-3 && (g - 41.0 / 255.0).abs() < 1e-3 && (b - 59.0 / 255.0).abs() < 1e-3);
        assert_eq!(a, 1.0);
    }

    #[test]
    fn wide_gamut_colors_are_gamut_mapped_into_srgb() {
        let p3_red = Color::rgb(RgbSpace::DisplayP3, 1.0, 0.0, 0.0, 1.0);
        assert!(!p3_red.in_srgb_gamut());
        let [r, g, b, _] = p3_red.to_srgb();
        for v in [r, g, b] {
            assert!((0.0..=1.0).contains(&v));
        }
        assert!(r > 0.9, "still red: {r}");
    }

    #[test]
    fn hsla_matches_srgb() {
        let c = Color::hsla(0.0, 1.0, 0.5, 1.0);
        assert!(c.approx_eq(Color::srgb(1.0, 0.0, 0.0, 1.0), 1e-4));
    }
}
