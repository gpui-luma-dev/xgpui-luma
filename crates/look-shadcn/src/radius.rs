//! Shadcn's named corner radii.
//!
//! The SDK has no radius vocabulary. Shadcn names radii with
//! [`ButtonRadiusPreset`] (none, small, medium, large, full) and resolves them here
//! to pixels before handing them to SDK controls.

use crate::controls::button::ButtonRadiusPreset;

/// Shadcn corner radii in pixels, derived from the theme's CSS `--radius`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadcnRadii {
    pub none: f32,
    pub small: f32,
    pub medium: f32,
    pub large: f32,
    /// Extra-large radius used by window-scale chrome (overlay windows).
    pub xl: f32,
    /// Fully rounded (pill / circle).
    pub full: f32,
}

impl Default for ShadcnRadii {
    fn default() -> Self {
        Self { none: 0.0, small: 3.0, medium: 6.0, large: 8.0, xl: 12.0, full: xgpui_luma::theme::FULL_RADIUS_PX }
    }
}

impl ShadcnRadii {
    /// Scale the radii so `medium` matches the catalog's base `--radius`.
    pub(crate) fn scaled_to(self, base_px: f32) -> Self {
        let scale = base_px / self.medium;
        Self { small: self.small * scale, medium: base_px, large: self.large * scale, xl: self.xl * scale, ..self }
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
}
