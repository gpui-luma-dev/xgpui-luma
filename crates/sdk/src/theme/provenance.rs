//! Look-agnostic resolved values and provenance.
//!
//! Paint paths should use [`ResolvedColor::hsla`]. Inspect / Studio paths keep the
//! [`ColorSource`] so callers can explain origin
//! without knowing whether the look is CSS-, scale-, or constant-backed.
//!
//! # Authoring a look
//!
//! Implement the SDK's control theme traits and expose look-local factories that
//! bind builders to templates and themes. Keep variants, source interpretation
//! (CSS catalogs, color scales), and active-look helpers in the look crate;
//! this module contains only look-agnostic resolved values and provenance.

use gpui::{Hsla, hsla};
use crate::theme::InteractionLayer;

/// A resolved color plus how it was obtained.
#[derive(Clone, Debug)]
pub struct ResolvedColor {
    pub value: Hsla,
    pub source: ColorSource,
}

/// Retained source color and provenance, separate from a backend preview.
#[derive(Clone, Debug)]
pub struct ResolvedSourceColor {
    pub value: crate::color::ColorValue,
    pub source: ColorSource,
}

impl ResolvedSourceColor {
    pub fn srgb_preview(self, mapping: crate::color::GamutMapping) -> anyhow::Result<ResolvedColor> {
        Ok(ResolvedColor { value: crate::color::gpui_bridge::to_hsla(self.value, mapping)?, source: self.source })
    }
}

/// Origin of a resolved color. Sources are intentionally not CSS-only.
#[derive(Clone, Debug)]
pub enum ColorSource {
    Transparent,
    /// Authored token or variable key (CSS custom property, design-token name, etc.).
    Authored {
        key: String,
    },
    /// Named scale family + 1-based step (e.g. Radix gray step 9).
    ScaleStep {
        family: String,
        step: u8,
    },
    /// Explicit interaction override of a base key.
    StateLayer {
        base: String,
        layer: InteractionLayer,
    },
    /// Algorithmically adjusted from a base (hover darken, contrast pick, …).
    Algorithmic {
        base: String,
        layer: InteractionLayer,
    },
    Derived {
        note: String,
    },
    Constant {
        label: String,
    },
}

impl ResolvedColor {
    pub fn new(value: Hsla, source: ColorSource) -> Self {
        Self { value, source }
    }

    pub fn transparent() -> Self {
        Self { value: hsla(0.0, 0.0, 0.0, 0.0), source: ColorSource::Transparent }
    }

    pub fn constant(value: Hsla, label: impl Into<String>) -> Self {
        Self { value, source: ColorSource::Constant { label: label.into() } }
    }

    pub fn authored(value: Hsla, key: impl Into<String>) -> Self {
        Self { value, source: ColorSource::Authored { key: key.into() } }
    }

    pub fn scale_step(value: Hsla, family: impl Into<String>, step: u8) -> Self {
        Self { value, source: ColorSource::ScaleStep { family: family.into(), step } }
    }

    pub fn hsla(&self) -> Hsla {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_step_preserves_family_and_step() {
        let color = ResolvedColor::scale_step(hsla(0.55, 0.7, 0.5, 1.0), "accent", 9);
        match &color.source {
            ColorSource::ScaleStep { family, step } => {
                assert_eq!(family, "accent");
                assert_eq!(*step, 9);
            }
            other => panic!("unexpected source: {other:?}"),
        }
        assert_eq!(color.hsla().a, 1.0);
    }
}
