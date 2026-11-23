//! Palette-backed source colors. Mapping and GPUI conversion produce separate previews.
//!
//! RGB components may be outside 0..=1; hue is in degrees. Source values are never
//! clamped or converted in place. Alpha must be finite and in 0..=1 for rendering
//! and persistence. Equality compares the tagged source, not perceptual equivalence.

use anyhow::{Result, ensure};
use palette::{LinSrgba, Oklcha, Srgba, encoding::p3::DisplayP3, rgb::Rgba};
use serde::{Deserialize, Serialize};

pub mod gpui_bridge;
mod css;
mod lss;
mod mapping;
pub use mapping::{Gamut, GamutMapping};

/// Encoded Display P3 with D65 white point and the sRGB transfer function.
pub type DisplayP3Color = Rgba<DisplayP3, f32>;

/// Source color retained independently of display/backend capabilities.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(try_from = "StoredColor")]
pub enum ColorValue {
    Srgb(Srgba),
    LinearSrgb(LinSrgba),
    DisplayP3(DisplayP3Color),
    Oklch(Oklcha),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum SourceSpace {
    Srgb,
    LinearSrgb,
    DisplayP3,
    Oklch,
}

/// Stable storage: a space tag plus unmodified f32 components (alpha last).
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredColor {
    space: SourceSpace,
    components: [f32; 4],
}

impl ColorValue {
    /// Convert a copy to Oklch without gamut mapping, retaining existing Oklch components.
    pub fn to_oklcha_unclamped(self) -> Result<Oklcha> {
        use palette::convert::FromColorUnclamped;
        self.validate()?;
        let color = match self {
            Self::Oklch(color) => color,
            _ => Oklcha::from_color_unclamped(self.to_srgba_unclamped()?),
        };
        Self::Oklch(color).validate()?;
        Ok(color)
    }

    /// Derive a UI color by shifting Oklch lightness into 0..=1, without gamut mapping.
    /// The original source remains unchanged; this is not an image editing operation.
    pub fn adjust_ui_lightness(self, delta: f32) -> Result<Self> {
        ensure!(delta.is_finite(), "lightness delta must be finite");
        let mut color = self.to_oklcha_unclamped()?;
        color.l = (color.l + delta).clamp(0.0, 1.0);
        Ok(Self::Oklch(color))
    }

    pub fn srgb(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self::Srgb(Srgba::new(red, green, blue, alpha))
    }

    pub fn linear_srgb(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self::LinearSrgb(LinSrgba::new(red, green, blue, alpha))
    }

    pub fn display_p3(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self::DisplayP3(DisplayP3Color::new(red, green, blue, alpha))
    }

    /// Hue is in degrees; no source lightness/chroma clipping is performed.
    pub fn oklch(lightness: f32, chroma: f32, hue_degrees: f32, alpha: f32) -> Self {
        Self::Oklch(Oklcha::new(lightness, chroma, hue_degrees, alpha))
    }

    pub fn alpha(self) -> f32 {
        self.components().1[3]
    }

    /// Replace alpha, rather than multiply opacity. Invalid alpha is rejected.
    pub fn with_alpha(self, alpha: f32) -> Result<Self> {
        let (space, mut components) = self.components();
        components[3] = alpha;
        Self::try_from(StoredColor { space, components })
    }

    /// Source channels may be extended range, but must be finite. Chroma is nonnegative.
    pub fn validate(self) -> Result<()> {
        let (space, components) = self.components();
        ensure!(components.iter().all(|value| value.is_finite()), "color components must be finite");
        ensure!((0.0..=1.0).contains(&components[3]), "color alpha must be in 0..=1");
        ensure!(space != SourceSpace::Oklch || components[1] >= 0.0, "Oklch chroma must be nonnegative");
        Ok(())
    }

    fn components(self) -> (SourceSpace, [f32; 4]) {
        match self {
            Self::Srgb(c) => (SourceSpace::Srgb, [c.red, c.green, c.blue, c.alpha]),
            Self::LinearSrgb(c) => (SourceSpace::LinearSrgb, [c.red, c.green, c.blue, c.alpha]),
            Self::DisplayP3(c) => (SourceSpace::DisplayP3, [c.red, c.green, c.blue, c.alpha]),
            Self::Oklch(c) => (SourceSpace::Oklch, [c.l, c.chroma, c.hue.into_raw_degrees(), c.alpha]),
        }
    }
}

impl From<ColorValue> for StoredColor {
    fn from(color: ColorValue) -> Self {
        let (space, components) = color.components();
        Self { space, components }
    }
}

impl Serialize for ColorValue {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        self.validate().map_err(serde::ser::Error::custom)?;
        StoredColor::from(*self).serialize(serializer)
    }
}

impl TryFrom<StoredColor> for ColorValue {
    type Error = anyhow::Error;

    fn try_from(stored: StoredColor) -> Result<Self> {
        let [x, y, z, alpha] = stored.components;
        let color = match stored.space {
            SourceSpace::Srgb => Self::srgb(x, y, z, alpha),
            SourceSpace::LinearSrgb => Self::linear_srgb(x, y, z, alpha),
            SourceSpace::DisplayP3 => Self::display_p3(x, y, z, alpha),
            SourceSpace::Oklch => Self::oklch(x, y, z, alpha),
        };
        color.validate()?;
        Ok(color)
    }
}

#[cfg(test)]
mod tests;
