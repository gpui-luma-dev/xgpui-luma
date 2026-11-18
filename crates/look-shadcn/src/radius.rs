//! Shadcn's named corner radii.
//!
//! The SDK has no radius vocabulary. Shadcn names radii with
//! [`ButtonRadiusPreset`] (none, small, medium, large, full) and
//! [`crate::ShadcnRadius`] (none, sm, md, lg, xl), and resolves both here to
//! pixels before handing them to SDK controls.

use crate::controls::button::ButtonRadiusPreset;

/// `--radius` used when a theme does not declare one.
pub(crate) const DEFAULT_BASE_RADIUS_PX: f32 = 8.0;

/// Shadcn corner radii in pixels, derived from the theme's CSS `--radius` with the
/// Tailwind steps the theme CSS declares: `--radius-sm = radius - 4px`,
/// `--radius-md = radius - 2px`, `--radius-lg = radius`, `--radius-xl = radius + 4px`.
///
/// This is Shadcn's single radius table.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadcnRadii {
    pub none: f32,
    /// `--radius-sm`.
    pub small: f32,
    /// `--radius-md`.
    pub medium: f32,
    /// `--radius-lg`, the theme's `--radius`.
    pub large: f32,
    /// `--radius-xl`.
    pub xl: f32,
    /// Fully rounded (pill / circle).
    pub full: f32,
}

impl Default for ShadcnRadii {
    fn default() -> Self {
        Self::from_base(DEFAULT_BASE_RADIUS_PX)
    }
}

impl ShadcnRadii {
    /// Radii for a theme whose `--radius` is `base_px`.
    pub fn from_base(base_px: f32) -> Self {
        Self {
            none: 0.0,
            small: (base_px - 4.0).max(0.0),
            medium: (base_px - 2.0).max(0.0),
            large: base_px,
            xl: base_px + 4.0,
            full: xgpui_luma::theme::FULL_RADIUS_PX,
        }
    }

    /// Pixels for a named preset. `Full` returns [`Self::full`]; callers that need a
    /// size-relative pill use their own cross-axis size instead.
    pub fn get(&self, preset: ButtonRadiusPreset) -> f32 {
        match preset {
            ButtonRadiusPreset::None => self.none,
            ButtonRadiusPreset::Small => self.small,
            ButtonRadiusPreset::Medium => self.medium,
            ButtonRadiusPreset::Large => self.large,
            ButtonRadiusPreset::Full => self.full,
        }
    }

    /// Pixels for a [`crate::ShadcnRadius`] step.
    pub fn step(&self, step: crate::ShadcnRadius) -> f32 {
        match step {
            crate::ShadcnRadius::None => self.none,
            crate::ShadcnRadius::Sm => self.small,
            crate::ShadcnRadius::Md => self.medium,
            crate::ShadcnRadius::Lg => self.large,
            crate::ShadcnRadius::Xl => self.xl,
        }
    }
}
