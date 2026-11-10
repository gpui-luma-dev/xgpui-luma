//! Chromaticity calculations ported from xgpui's extensions-web color_tasks_model.
use xgpui_color::{EncodedRgb, GamutMapping, RgbSpace, XyzD65};

pub const SPACES: [RgbSpace; 4] = [RgbSpace::Srgb, RgbSpace::DisplayP3, RgbSpace::AdobeRgb1998, RgbSpace::Rec2020];
pub const NAMES: [&str; 4] = ["sRGB", "Display P3", "Adobe RGB (1998)", "Rec.2020"];
pub const PATCHES: [(&str, [f32; 3]); 7] = [
    ("Green", [0., 1., 0.]),
    ("Cyan", [0., 1., 1.]),
    ("Mid gray", [0.18; 3]),
    ("Near black", [0.0001; 3]),
    ("White", [1.; 3]),
    ("Red", [1., 0., 0.]),
    ("Magenta", [1., 0., 1.]),
];
pub const LUMINANCE: [f64; 3] = [0.05, 0.18, 0.5];
pub const COLORS: [u32; 4] = [0x65d7ff, 0xff9a62, 0xc4a0ff, 0x91df86];

pub fn xy(xyz: [f64; 3]) -> Option<[f64; 2]> {
    let sum: f64 = xyz.into_iter().sum();
    (sum.is_finite() && sum > 0.).then(|| [xyz[0] / sum, xyz[1] / sum])
}
pub fn locus() -> Vec<[f64; 2]> {
    include_str!("../../assets/color-labs/colorimetry/CIE_cc_1931_2deg.csv")
        .lines()
        .map(|line| {
            let v: Vec<f64> = line.split(',').map(|v| v.parse().expect("bundled CIE datum")).collect();
            xy([v[1], v[2], v[3]]).expect("nonzero spectral coordinate")
        })
        .collect()
}
fn contains(points: &[[f64; 2]], p: [f64; 2]) -> bool {
    let mut inside = false;
    let mut previous = points[points.len() - 1];
    for &current in points {
        if (current[1] > p[1]) != (previous[1] > p[1])
            && p[0] < (previous[0] - current[0]) * (p[1] - current[1]) / (previous[1] - current[1]) + current[0]
        {
            inside = !inside;
        }
        previous = current;
    }
    inside
}
pub struct Cell {
    #[cfg(all(test, feature = "test-support", target_os = "macos"))]
    pub xy: [f64; 2],
    #[cfg(test)]
    pub rgb: [f32; 3],
    pub outside: bool,
}
pub fn cells(
    space: RgbSpace,
    luminance: f64,
    _mapping: GamutMapping,
    boundary: &[[f64; 2]],
) -> anyhow::Result<Vec<Cell>> {
    let mut cells = Vec::new();
    for row in 0..54 {
        for col in 0..48 {
            let p = [(col as f64 + 0.5) * 0.8 / 48., (row as f64 + 0.5) * 0.9 / 54.];
            if !contains(boundary, p) {
                continue;
            }
            let xyz = XyzD65::new([p[0] * luminance / p[1], luminance, (1. - p[0] - p[1]) * luminance / p[1]])?;
            let rgb = xyz.to_linear(space)?;
            let outside = rgb.components().into_iter().any(|v| !(-1e-12..=1. + 1e-12).contains(&v));
            // CPU colors are an independent oracle for shader readback tests.
            // The live UI only needs the sampled out-of-gamut count.
            #[cfg(test)]
            let preview = rgb
                .map_gamut(space, _mapping)?
                .map_gamut(RgbSpace::Srgb, GamutMapping::Clip)?
                .encode()?
                .components()
                .map(|v| v as f32);
            cells.push(Cell {
                #[cfg(all(test, feature = "test-support", target_os = "macos"))]
                xy: [col as f64 * 0.8 / 48., row as f64 * 0.9 / 54.],
                #[cfg(test)]
                rgb: preview,
                outside,
            });
        }
    }
    Ok(cells)
}
pub fn source_xy(space: RgbSpace, rgb: [f32; 3]) -> Option<[f64; 2]> {
    let linear = EncodedRgb::new(space, rgb.map(f64::from)).ok()?.decode().ok()?;
    xy(linear.to_xyz().ok()?.components())
}

/// Encoded source values are reinterpreted without changing the pixel controls.
pub fn source_linear(source: usize, rgb: [f32; 3]) -> anyhow::Result<xgpui_color::LinearRgb> {
    Ok(EncodedRgb::new(SPACES[source], rgb.map(f64::from))?.decode()?)
}

pub fn preview(source: usize, rgb: [f32; 3], simulated: usize, mapping: GamutMapping) -> anyhow::Result<[f64; 3]> {
    Ok(source_linear(source, rgb)?
        .map_gamut(SPACES[simulated], mapping)?
        .map_gamut(RgbSpace::Srgb, GamutMapping::Clip)?
        .encode()?
        .components())
}

/// Translate a plot-local pointer using the same inset and scale as the shader.
pub fn pointer_xy(position: [f64; 2], size: [f64; 2], uv: bool) -> Option<[f64; 2]> {
    let extent = if uv { [0.65, 0.65] } else { [0.8, 0.9] };
    let scale = ((size[0] - 48.) / extent[0]).min((size[1] - 48.) / extent[1]).max(1.);
    let p = [(position[0] - (size[0] - scale * extent[0]) * 0.5) / scale, (size[1] - 24. - position[1]) / scale];
    if !p.iter().zip(extent).all(|(v, max)| (0. ..=max).contains(v)) {
        return None;
    }
    let xy = if uv {
        let d = 6. * p[0] - 16. * p[1] + 12.;
        if d <= 1e-12 {
            return None;
        }
        [9. * p[0] / d, 4. * p[1] / d]
    } else {
        p
    };
    (xy[1] > 1e-9 && xy[0] >= 0. && xy[0] + xy[1] <= 1.).then_some(xy)
}

/// Retain brightness when possible; constrain unrepresentable positions to source RGB bounds.
pub fn rgb_at_xy(source: usize, xy: [f64; 2], luminance: f64) -> anyhow::Result<[f32; 3]> {
    anyhow::ensure!(
        xy.iter().all(|v| v.is_finite() && *v >= 0.) && xy[1] > 1e-9 && xy[0] + xy[1] <= 1.,
        "Invalid chromaticity"
    );
    let y = if luminance <= 1e-9 { 0.18 } else { luminance };
    let xyz = XyzD65::new([xy[0] * y / xy[1], y, (1. - xy[0] - xy[1]) * y / xy[1]])?;
    let linear = xyz.to_linear(SPACES[source])?.components().map(|v| v.max(0.));
    let peak = linear.into_iter().fold(1., f64::max);
    Ok(xgpui_color::LinearRgb::new(SPACES[source], linear.map(|v| v / peak))?
        .encode()?
        .components()
        .map(|v| v.clamp(0., 1.) as f32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagram_pointer_round_trips_xy_and_uv_at_different_sizes() {
        let xy = [0.3127, 0.3290];
        for uv in [false, true] {
            for size in [[600_f64, 420.], [900., 606.], [400., 606.]] {
                let extent = if uv { [0.65, 0.65] } else { [0.8, 0.9] };
                let scale = ((size[0] - 48.) / extent[0]).min((size[1] - 48.) / extent[1]);
                let d = -2. * xy[0] + 12. * xy[1] + 3.;
                let projected = if uv { [4. * xy[0] / d, 9. * xy[1] / d] } else { xy };
                let position =
                    [(size[0] - scale * extent[0]) * 0.5 + projected[0] * scale, size[1] - 24. - projected[1] * scale];
                let actual = pointer_xy(position, size, uv).unwrap();
                assert!((actual[0] - xy[0]).abs() < 1e-12);
                assert!((actual[1] - xy[1]).abs() < 1e-12);
                assert!(pointer_xy([-100., -100.], size, uv).is_none());
            }
        }
    }

    #[test]
    fn diagram_drag_preserves_representable_chromaticity_and_handles_black_and_gamut_edges() {
        for (source, space) in SPACES.into_iter().enumerate() {
            let original = [0.3, 0.6, 0.4];
            let linear = source_linear(source, original).unwrap();
            let y = linear.to_xyz().unwrap().components()[1];
            let xy = source_xy(space, original).unwrap();
            let actual = rgb_at_xy(source, xy, y).unwrap();
            assert!(actual.into_iter().zip(original).all(|(a, b)| (a - b).abs() < 1e-6));
            for target in [[0.1, 0.8], [0.7, 0.2], [0.2, 0.01]] {
                let rgb = rgb_at_xy(source, target, y).unwrap();
                assert!(rgb.iter().all(|v| v.is_finite() && (0. ..=1.).contains(v)));
            }
            assert!(rgb_at_xy(source, xy, 0.).unwrap().iter().any(|v| *v > 0.));
            assert!(rgb_at_xy(source, [0.3, 0.], y).is_err());
        }
    }

    #[test]
    fn encoded_source_and_mapping_previews_preserve_color_semantics() {
        let rgb = [0., 1., 0.];
        let source = source_linear(1, rgb).unwrap();
        let srgb = source.convert(RgbSpace::Srgb).unwrap().encode().unwrap().components();
        assert!(srgb[0] < 0. && srgb[1] > 1. && srgb[2] < 0.);
        let clip = preview(1, rgb, 0, GamutMapping::Clip).unwrap();
        let reduce = preview(1, rgb, 0, GamutMapping::ReduceChroma).unwrap();
        assert_ne!(clip, reduce);
        assert!(clip.into_iter().chain(reduce).all(|v| (0. ..=1.).contains(&v)));
        let rec = source_linear(3, [0.5; 3]).unwrap();
        assert!(rec.components().iter().all(|v| *v < 0.5));
    }

    #[test]
    fn gamut_fill_covers_the_same_locus_but_changes_mapping_and_outside_count() {
        let boundary = locus();
        assert_eq!(boundary.len(), 471);
        let srgb = cells(RgbSpace::Srgb, 0.18, GamutMapping::Clip, &boundary).unwrap();
        let p3 = cells(RgbSpace::DisplayP3, 0.18, GamutMapping::Clip, &boundary).unwrap();
        let reduced = cells(RgbSpace::DisplayP3, 0.18, GamutMapping::ReduceChroma, &boundary).unwrap();
        assert_eq!(srgb.len(), p3.len());
        assert!(srgb.iter().filter(|c| c.outside).count() > p3.iter().filter(|c| c.outside).count());
        assert!(p3.iter().zip(&reduced).any(|(a, b)| a.rgb != b.rgb));
        assert!(
            srgb.iter()
                .chain(&p3)
                .chain(&reduced)
                .all(|c| c.rgb.iter().all(|v| v.is_finite() && (0. ..=1.).contains(v)))
        );
        assert_ne!(source_xy(RgbSpace::Srgb, [0., 1., 0.]), source_xy(RgbSpace::DisplayP3, [0., 1., 0.]));
        assert!(source_xy(RgbSpace::Srgb, [0.; 3]).is_none());
    }
}
