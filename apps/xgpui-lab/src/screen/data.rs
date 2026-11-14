//! Data colors: the colors a screen is *about* (gamut outlines, markers).
//! Never add chrome colors here; those come from the look.
use gpui::{Hsla, rgb};

/// Gamut outline colors, in `chromaticity_model::SPACES` order.
pub(crate) const GAMUT_SRGB: u32 = 0x65d7ff;
pub(crate) const GAMUT_P3: u32 = 0xff9a62;
pub(crate) const GAMUT_ADOBE: u32 = 0xc4a0ff;
pub(crate) const GAMUT_REC2020: u32 = 0x91df86;
pub(crate) const GAMUTS: [u32; 4] = [GAMUT_SRGB, GAMUT_P3, GAMUT_ADOBE, GAMUT_REC2020];

/// The subject point (the source pixel) on a diagram, and the dark halo that keeps
/// this light color readable on light backgrounds and fills. The reference point
/// (D65) uses the look's foreground with a background halo instead.
pub(crate) const MARKER_SUBJECT: u32 = 0xffdf62;
pub(crate) const MARKER_SUBJECT_HALO: u32 = 0x1c1c1c;

/// Marks gradient samples that fall outside the destination gamut before mapping.
pub(crate) const OUT_OF_GAMUT: u32 = 0xfbbf24;

/// Benchmark series, one per artwork in run order; repeats after six.
pub(crate) const SERIES: [u32; 6] = [0x3984d6, 0xd17b32, 0x35a779, 0xa16acb, 0xd45068, 0x79923d];

/// Marks measurements whose output the raster budget capped.
pub(crate) const CAPPED: u32 = 0xff3b30;

pub(crate) fn series(index: usize) -> Hsla {
    color(SERIES[index % SERIES.len()])
}

pub(crate) fn color(hex: u32) -> Hsla {
    rgb(hex).into()
}
