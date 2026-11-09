use anyhow::{Result, ensure};
use palette::{Oklab, Oklch, Srgba, convert::FromColorUnclamped};

use super::{ColorValue, DisplayP3Color};

/// Bounded preview/output gamut, independent of source storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Gamut {
    Srgb,
    DisplayP3,
}

/// Explicit UI preview policy; neither policy modifies source color data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GamutMapping {
    /// Clip destination RGB channels. Useful when clipping is deliberately desired.
    Clip,
    /// CSS Color 4 binary search with local MINDE, 30 September 2026 draft, §14.2.2.
    /// This is a UI color policy, not a photographic rendering intent.
    CssLocalMinde,
}

impl ColorValue {
    /// Convert without clipping. Extended-range RGB is retained in the result.
    pub fn to_srgba_unclamped(self) -> Result<Srgba> {
        self.validate()?;
        let rgb = match self {
            Self::Srgb(c) => c,
            Self::LinearSrgb(c) => Srgba::from_color_unclamped(c),
            Self::DisplayP3(c) => Srgba::from_color_unclamped(c),
            Self::Oklch(c) => Srgba::from_color_unclamped(c),
        };
        ensure!(rgb.red.is_finite() && rgb.green.is_finite() && rgb.blue.is_finite(), "RGB conversion overflow");
        Ok(rgb)
    }

    /// Check destination RGB channels; alpha is validated separately.
    pub fn is_in_gamut(self, gamut: Gamut) -> Result<bool> {
        let rgb = self.to_srgba_unclamped()?;
        Ok(match gamut {
            Gamut::Srgb => rgb_in_gamut(rgb),
            Gamut::DisplayP3 => {
                let c = self.to_display_p3_unclamped()?;
                channels_in_gamut([c.red, c.green, c.blue, c.alpha])
            }
        })
    }

    /// Convert without clipping into encoded Display P3 (D65, sRGB transfer).
    /// Display P3 sources are returned unchanged; extended-range values are retained.
    pub fn to_display_p3_unclamped(self) -> Result<DisplayP3Color> {
        self.validate()?;
        let p3 = match self {
            Self::DisplayP3(c) => c,
            _ => DisplayP3Color::from_color_unclamped(self.to_srgba_unclamped()?),
        };
        ensure!(p3.red.is_finite() && p3.green.is_finite() && p3.blue.is_finite(), "RGB conversion overflow");
        Ok(p3)
    }

    /// Resolve the current backend's bounded sRGB fallback without changing the source.
    pub fn to_srgba_fallback(self, policy: GamutMapping) -> Result<Srgba> {
        let [r, g, b, a] = self.map_to_gamut(Gamut::Srgb, policy)?;
        Ok(Srgba::new(r, g, b, a))
    }

    /// Bounded encoded Display P3 for wide-gamut output, without changing the source.
    /// Uses the same policies as [`Self::to_srgba_fallback`], with P3 as destination.
    pub fn to_display_p3_fallback(self, policy: GamutMapping) -> Result<DisplayP3Color> {
        let [r, g, b, a] = self.map_to_gamut(Gamut::DisplayP3, policy)?;
        Ok(DisplayP3Color::new(r, g, b, a))
    }

    /// Encoded destination RGB plus alpha, bounded to `gamut` by `policy`.
    fn map_to_gamut(self, gamut: Gamut, policy: GamutMapping) -> Result<[f32; 4]> {
        let encode = |c: Srgba| -> [f32; 4] {
            match gamut {
                Gamut::Srgb => [c.red, c.green, c.blue, c.alpha],
                Gamut::DisplayP3 => {
                    let p = DisplayP3Color::from_color_unclamped(c);
                    [p.red, p.green, p.blue, p.alpha]
                }
            }
        };
        let decode = |c: [f32; 4]| -> Srgba {
            match gamut {
                Gamut::Srgb => Srgba::new(c[0], c[1], c[2], c[3]),
                Gamut::DisplayP3 => Srgba::from_color_unclamped(DisplayP3Color::new(c[0], c[1], c[2], c[3])),
            }
        };
        let clamp = |c: [f32; 4]| [c[0].clamp(0.0, 1.0), c[1].clamp(0.0, 1.0), c[2].clamp(0.0, 1.0), c[3]];
        let rgb = self.to_srgba_unclamped()?;
        let destination = match (self, gamut) {
            (Self::DisplayP3(c), Gamut::DisplayP3) => [c.red, c.green, c.blue, c.alpha],
            _ => encode(rgb),
        };
        ensure!(destination.iter().all(|v| v.is_finite()), "RGB conversion overflow");
        if channels_in_gamut(destination) {
            return Ok(destination);
        }
        if policy == GamutMapping::Clip {
            return Ok(clamp(destination));
        }
        let origin = match self {
            Self::Oklch(c) => c,
            _ => palette::Oklcha::from_color_unclamped(rgb),
        };
        ensure!(
            origin.l.is_finite() && origin.chroma.is_finite() && origin.hue.into_raw_degrees().is_finite(),
            "Oklch conversion overflow"
        );
        if origin.l >= 1.0 {
            return Ok([1.0, 1.0, 1.0, origin.alpha]);
        }
        if origin.l <= 0.0 {
            return Ok([0.0, 0.0, 0.0, origin.alpha]);
        }
        let mut current = origin;
        let mut clipped = clamp(destination);
        const JND: f32 = 0.02;
        const EPSILON: f32 = 0.0001;
        if delta_ok(current.color, decode(clipped)) < JND {
            return Ok(clipped);
        }
        let mut min = 0.0;
        let mut max = origin.chroma;
        let mut min_in_gamut = true;
        // f32 bisection is bounded even for extended-range source chroma.
        for _ in 0..128 {
            if max - min <= EPSILON {
                break;
            }
            let chroma = min + (max - min) * 0.5;
            if chroma == min || chroma == max {
                break;
            }
            current.chroma = chroma;
            let candidate = encode(Srgba::from_color_unclamped(current));
            if min_in_gamut && channels_in_gamut(candidate) {
                min = chroma;
                continue;
            }
            clipped = clamp(candidate);
            let difference = delta_ok(current.color, decode(clipped));
            if difference < JND {
                if JND - difference < EPSILON {
                    return Ok(clipped);
                }
                min_in_gamut = false;
                min = chroma;
            } else {
                max = chroma;
            }
        }
        Ok(clipped)
    }
}

fn channels_in_gamut(c: [f32; 4]) -> bool {
    c[..3].iter().all(|v| (0.0..=1.0).contains(v))
}

fn rgb_in_gamut(rgb: Srgba) -> bool {
    [rgb.red, rgb.green, rgb.blue].iter().all(|v| (0.0..=1.0).contains(v))
}

fn delta_ok(origin: Oklch, clipped: Srgba) -> f32 {
    let a = Oklab::from_color_unclamped(origin);
    let b = Oklab::from_color_unclamped(clipped.color);
    ((a.l - b.l).powi(2) + (a.a - b.a).powi(2) + (a.b - b.b).powi(2)).sqrt()
}
