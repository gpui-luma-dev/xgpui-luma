//! Twelve-step Radix scales (named or generated; color + gray).
//!
//! Radix Themes always carry at least:
//! - a chromatic **color** scale (12 steps)
//! - a neutral **gray** scale (12 steps)
//! - optional **destructive** (and other) scales
//!
//! Page chrome often mixes families (e.g. color step 3 → gray step 1).

use gpui::Hsla;
use xgpui_luma::color::{ColorValue, GamutMapping, gpui_bridge::SrgbRenderCache};
use xgpui_luma::theme::ThemeMode;
use xgpui_luma::theme::provenance::ResolvedColor;

/// 1-based Radix-style scale step (`1`…`12`).
pub type ScaleStep = u8;

pub const SCALE_LEN: usize = 12;

/// Named scale family used by semantic mappings and provenance.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ScaleFamily {
    /// Neutral 12-step scale.
    Gray,
    /// Chromatic 12-step scale (Radix "color" / accent family).
    Color,
    Destructive,
}

impl ScaleFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gray => "gray",
            Self::Color => "color",
            Self::Destructive => "destructive",
        }
    }
}

/// Twelve retained source steps and precomputed sRGB previews for one family in one mode.
///
/// `palette` is the Radix palette the steps came from (`"indigo"`, `"slate"`, …) and rides
/// along as color provenance. Hand-seeded scales carry [`CUSTOM_PALETTE`].
#[derive(Clone, Copy, Debug)]
pub struct ColorScale {
    sources: [ColorValue; SCALE_LEN],
    steps: [Hsla; SCALE_LEN],
    mapping: GamutMapping,
    palette: &'static str,
}

/// Provenance for a scale generated from a seed rather than a named Radix palette.
pub const CUSTOM_PALETTE: &str = "custom";

impl ColorScale {
    pub fn new(sources: [ColorValue; SCALE_LEN], mapping: GamutMapping) -> anyhow::Result<Self> {
        Self::named(CUSTOM_PALETTE, sources, mapping)
    }

    pub fn named(
        palette: &'static str,
        sources: [ColorValue; SCALE_LEN],
        mapping: GamutMapping,
    ) -> anyhow::Result<Self> {
        let mut cache = SrgbRenderCache::default();
        let mut steps = [gpui::black(); SCALE_LEN];
        for (index, source) in sources.iter().enumerate() {
            steps[index] = cache
                .resolve_hsla(*source, mapping)
                .map_err(|error| anyhow::anyhow!("{palette} step {}: {error}", index + 1))?;
        }
        Ok(Self { sources, steps, palette, mapping })
    }

    pub(crate) fn black() -> Self {
        Self {
            sources: [ColorValue::srgb(0.0, 0.0, 0.0, 1.0); SCALE_LEN],
            steps: [gpui::black(); SCALE_LEN],
            palette: CUSTOM_PALETTE,
            mapping: GamutMapping::CssLocalMinde,
        }
    }

    /// Source color, without conversion or mapping; same clamped 1-based indexing as `step`.
    pub fn source_step(self, step: ScaleStep) -> ColorValue {
        self.sources[step.clamp(1, SCALE_LEN as u8) as usize - 1]
    }

    pub fn mapping(self) -> GamutMapping {
        self.mapping
    }

    pub fn palette(self) -> &'static str {
        self.palette
    }

    /// Returns the color for a 1-based step, clamping into `1..=12`.
    pub fn step(self, step: ScaleStep) -> Hsla {
        let idx = step.clamp(1, SCALE_LEN as u8) as usize - 1;
        self.steps[idx]
    }

    pub fn resolved_source(self, step: ScaleStep) -> xgpui_luma::theme::provenance::ResolvedSourceColor {
        xgpui_luma::theme::provenance::ResolvedSourceColor {
            value: self.source_step(step),
            source: xgpui_luma::theme::provenance::ColorSource::ScaleStep {
                family: self.palette.into(),
                step: step.clamp(1, SCALE_LEN as u8),
            },
        }
    }

    pub fn resolved(self, step: ScaleStep) -> ResolvedColor {
        ResolvedColor::scale_step(self.step(step), self.palette, step.clamp(1, SCALE_LEN as u8))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ModeScales {
    pub gray: ColorScale,
    pub color: ColorScale,
    pub destructive: ColorScale,
}

impl ModeScales {
    pub fn family(self, family: ScaleFamily) -> ColorScale {
        match family {
            ScaleFamily::Gray => self.gray,
            ScaleFamily::Color => self.color,
            ScaleFamily::Destructive => self.destructive,
        }
    }

    pub fn resolved(self, family: ScaleFamily, step: ScaleStep) -> ResolvedColor {
        self.family(family).resolved(step)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ScalePair {
    pub light: ModeScales,
    pub dark: ModeScales,
}

impl ScalePair {
    pub fn for_mode(self, mode: ThemeMode) -> ModeScales {
        match mode {
            ThemeMode::Light => self.light,
            ThemeMode::Dark => self.dark,
        }
    }
}

#[cfg(test)]
mod tests {
    use gpui::hsla;

    use super::*;
    use crate::palette::{Accent, Gray, scale_pair};

    fn color_scale_from_seed(seed: Hsla, mode: ThemeMode) -> ColorScale {
        crate::generate_colors(
            crate::CustomColors {
                accent: xgpui_luma::color::gpui_bridge::from_hsla(seed),
                gray: ColorValue::srgb(139.0 / 255.0, 141.0 / 255.0, 152.0 / 255.0, 1.0),
                background: if mode == ThemeMode::Light {
                    ColorValue::srgb(1.0, 1.0, 1.0, 1.0)
                } else {
                    ColorValue::srgb(0.0, 0.0, 0.0, 1.0)
                },
            },
            mode,
        )
        .unwrap()
        .accent
    }

    fn indigo_slate() -> ScalePair {
        scale_pair(Accent::Indigo, Gray::Auto)
    }

    #[test]
    fn step_clamps_and_is_one_based() {
        let gray = indigo_slate().light.gray;
        assert_eq!(gray.step(1).l, gray.step(0).l);
        assert_eq!(gray.step(12).l, gray.step(99).l);
        assert!(gray.step(1).l > gray.step(12).l);
    }

    #[test]
    fn dark_gray_inverts_lightness_direction() {
        let scales = indigo_slate();
        assert!(scales.light.gray.step(1).l > scales.light.gray.step(12).l);
        assert!(scales.dark.gray.step(1).l < scales.dark.gray.step(12).l);
    }

    /// Seeded scales belong to no palette. Named provenance is covered by
    /// `palette::tests::indigo_slate_carries_real_radix_steps_and_provenance`.
    #[test]
    fn seeded_scales_report_custom_provenance() {
        let seeded = color_scale_from_seed(hsla(0.58, 0.72, 0.58, 1.0), ThemeMode::Light);
        match seeded.resolved(9).source {
            xgpui_luma::theme::provenance::ColorSource::ScaleStep { family, step } => {
                assert_eq!(family, CUSTOM_PALETTE);
                assert_eq!(step, 9);
            }
            other => panic!("unexpected source: {other:?}"),
        }
    }

    #[test]
    fn color_and_gray_are_distinct_families() {
        let scales = indigo_slate().for_mode(ThemeMode::Light);
        let color_3 = scales.resolved(ScaleFamily::Color, 3).hsla();
        let gray_3 = scales.resolved(ScaleFamily::Gray, 3).hsla();
        assert_ne!(color_3.s, gray_3.s);
    }

    #[test]
    fn generated_color_scale_preserves_step_nine_seed() {
        let seed = hsla(0.58, 0.72, 0.58, 1.0);
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let actual: gpui::Rgba = color_scale_from_seed(seed, mode).step(9).into();
            let expected: gpui::Rgba = seed.into();
            assert!((actual.r - expected.r).abs() <= 1.0 / 255.0);
            assert!((actual.g - expected.g).abs() <= 1.0 / 255.0);
            assert!((actual.b - expected.b).abs() <= 1.0 / 255.0);
        }
    }
}
