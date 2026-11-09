//! The one boundary between source colors and paint inputs.
//!
//! Hosts store [`ColorValue`]s, unclipped. Renderers consume encoded RGB in
//! their *output* primaries: GPUI paints, the CPU rasterizers and PNG export
//! are sRGB; the Display P3 shaders/producers (`p3`, `mesh_p3`,
//! `freeform_p3`, the P3 color field) are Display P3. Each render path names
//! its [`OutputSpace`] and encodes here, mapping into that gamut with the SDK's
//! CSS local-MINDE policy only at this point.
//!
//! The paint layer carries the encoded values in `gpui::Hsla` (its existing
//! interpolation math). A P3-encoded `Hsla` is created only here and only for
//! P3 consumers; it must never be passed to a GPUI paint.

use gpui::Hsla;
use xgpui_luma::color::{ColorValue, GamutMapping, gpui_bridge};

use super::paint::{MeshPoint, PaintPoint};

/// Gamut-mapping policy applied when encoding for an output.
pub(crate) const OUTPUT_MAPPING: GamutMapping = GamutMapping::CssLocalMinde;

/// Primaries (and gamut) of a render path's encoded RGB.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum OutputSpace {
    /// GPUI's native output and the CPU rasterizers.
    Srgb,
    /// Wide-gamut SDR producers on a P3-capable window.
    DisplayP3,
}

impl OutputSpace {
    pub fn wide(wide: bool) -> Self {
        if wide { Self::DisplayP3 } else { Self::Srgb }
    }
}

/// Encoded RGBA (straight alpha) in `space`, bounded to its gamut. Invalid
/// sources encode as transparent, as SDK swatches do.
pub(crate) fn encode_rgba(color: ColorValue, space: OutputSpace) -> [f32; 4] {
    let encoded = match space {
        OutputSpace::Srgb => color.to_srgba_fallback(OUTPUT_MAPPING).map(|c| [c.red, c.green, c.blue, c.alpha]),
        OutputSpace::DisplayP3 => {
            color.to_display_p3_fallback(OUTPUT_MAPPING).map(|c| [c.red, c.green, c.blue, c.alpha])
        }
    };
    encoded.unwrap_or([0.0; 4])
}

/// [`encode_rgba`] in the paint layer's `Hsla` carrier.
pub(crate) fn encode(color: ColorValue, space: OutputSpace) -> Hsla {
    let [r, g, b, a] = encode_rgba(color, space);
    gpui_bridge::preview_hsla(gpui::Rgba { r, g, b, a })
}

pub(crate) fn encode_stops(stops: &[(f32, ColorValue)], space: OutputSpace) -> Vec<(f32, Hsla)> {
    stops.iter().map(|(position, color)| (*position, encode(*color, space))).collect()
}

pub(crate) fn encode_points(points: &[MeshPoint], space: OutputSpace) -> Vec<PaintPoint> {
    points.iter().map(|point| point.with_color(encode(point.color, space))).collect()
}

/// sRGB-authored color (palettes, presets) as a source value.
pub(crate) fn srgb_source(color: Hsla) -> ColorValue {
    gpui_bridge::from_hsla(color)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: [f32; 4], b: [f32; 4]) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-4)
    }

    #[test]
    fn p3_sources_reach_p3_output_unclipped_and_srgb_output_mapped() {
        let red = ColorValue::display_p3(1.0, 0.0, 0.0, 0.8);
        assert_eq!(encode_rgba(red, OutputSpace::DisplayP3), [1.0, 0.0, 0.0, 0.8]);
        let srgb = encode_rgba(red, OutputSpace::Srgb);
        assert!(srgb[..3].iter().all(|v| (0.0..=1.0).contains(v)));
        assert_eq!(srgb[3], 0.8);
        // The Hsla carrier round-trips the P3 encoding for P3 consumers.
        let carried = encode(red, OutputSpace::DisplayP3).to_rgb();
        assert!(close([carried.r, carried.g, carried.b, carried.a], [1.0, 0.0, 0.0, 0.8]));
    }

    #[test]
    fn srgb_sources_keep_their_meaning_in_p3_output() {
        // sRGB red is not P3 red: P3 output re-encodes it instead of relabeling.
        let red = ColorValue::srgb(1.0, 0.0, 0.0, 1.0);
        assert_eq!(encode_rgba(red, OutputSpace::Srgb), [1.0, 0.0, 0.0, 1.0]);
        let p3 = encode_rgba(red, OutputSpace::DisplayP3);
        assert!(p3[0] < 0.95 && p3[1] > 0.1 && p3[2] > 0.05, "{p3:?}");
    }

    #[test]
    fn points_keep_geometry_and_encode_color() {
        let point = MeshPoint { row: 1, col: 2, u: 0.25, v: 0.75, color: ColorValue::display_p3(0.0, 1.0, 0.0, 1.0) };
        let [encoded] = encode_points(&[point], OutputSpace::DisplayP3)[..] else {
            panic!()
        };
        assert_eq!((encoded.row, encoded.col, encoded.u, encoded.v), (1, 2, 0.25, 0.75));
        let rgb = encoded.color.to_rgb();
        assert!(close([rgb.r, rgb.g, rgb.b, rgb.a], [0.0, 1.0, 0.0, 1.0]));
    }
}
