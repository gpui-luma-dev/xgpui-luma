//! Extended-range HSL (CSS Color 4 §7.1 sample code), for values outside the
//! RGB cube. Palette's `Hsl::from_color_unclamped` clips negative channels,
//! which would hide how far a wide color lies outside sRGB.
//!
//! Hue is in degrees, saturation/lightness are fractions; neither is bounded.
//! The math is independent of primaries: callers say which encoded RGB it is.

/// `[r, g, b]` → `[h, s, l]`. A negative CSS saturation becomes a positive
/// one with the hue rotated half a turn, as the spec does.
pub(crate) fn rgb_to_hsl([r, g, b]: [f32; 3]) -> [f32; 3] {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let lightness = (min + max) / 2.0;
    let d = max - min;
    let (mut hue, mut saturation) = (0.0, 0.0);
    if d != 0.0 {
        let denominator = lightness.min(1.0 - lightness);
        saturation = if denominator == 0.0 {
            0.0
        } else {
            (max - lightness) / denominator
        };
        hue = if max == r {
            (g - b) / d + if g < b { 6.0 } else { 0.0 }
        } else if max == g {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        } * 60.0;
    }
    if saturation < 0.0 {
        hue += 180.0;
        saturation = -saturation;
    }
    [hue.rem_euclid(360.0), saturation, lightness]
}

/// `[h, s, l]` → `[r, g, b]`, inverse of [`rgb_to_hsl`].
pub(crate) fn hsl_to_rgb([hue, saturation, lightness]: [f32; 3]) -> [f32; 3] {
    let hue = hue.rem_euclid(360.0);
    let f = |n: f32| {
        let k = (n + hue / 30.0) % 12.0;
        let a = saturation * lightness.min(1.0 - lightness);
        lightness - a * (k - 3.0).min(9.0 - k).min(1.0).max(-1.0)
    };
    [f(0.0), f(8.0), f(4.0)]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: [f32; 3], b: [f32; 3]) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-4)
    }

    #[test]
    fn matches_standard_hsl_inside_the_cube() {
        assert!(close(rgb_to_hsl([1.0, 0.0, 0.0]), [0.0, 1.0, 0.5]));
        assert!(close(rgb_to_hsl([0.5, 0.5, 0.5]), [0.0, 0.0, 0.5]));
        assert!(close(hsl_to_rgb([120.0, 1.0, 0.25]), [0.0, 0.5, 0.0]));
    }

    #[test]
    fn extended_values_round_trip_without_clipping() {
        for rgb in [[-1.2049, 0.7392, -0.922], [1.0931, -0.2267, -0.1501], [1.2, 1.1, -0.3], [0.2, 0.4, 0.7]] {
            let hsl = rgb_to_hsl(rgb);
            assert!(close(hsl_to_rgb(hsl), rgb), "{rgb:?} -> {hsl:?} -> {:?}", hsl_to_rgb(hsl));
        }
        // sRGB coordinates of Display P3 red: saturation above 100%.
        assert!(rgb_to_hsl([1.0931, -0.2267, -0.1501])[1] > 1.0);
    }
}
