//! Radix slider theme adapter (Classic / Surface / Soft).
//!
//! Matches `@radix-ui/themes` Slider variant CSS at a paint level:
//! - **Surface** — gray track with an inset rim; accent fill; flat thumb ring
//! - **Classic** — same gray track, raised thumb (shadow-1 style)
//! - **Soft** — gray tint track; accent-6 fill; softer thumb elevation
#![allow(deprecated)] // Still reads a deprecated Look until its family has a style-override replacement (#433).

use std::sync::Arc;

use xgpui_luma::controls::slider::{SliderTemplate, SliderTheme, StyledSliderTheme, ThemedSliderTemplate};

use crate::look::Look;

/// Radix Themes slider visual variants.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SliderVariant {
    Classic,
    #[default]
    Surface,
    Soft,
}

impl SliderVariant {
    pub const ALL: [Self; 3] = [Self::Classic, Self::Surface, Self::Soft];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Surface => "surface",
            Self::Soft => "soft",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::Surface => "Surface",
            Self::Soft => "Soft",
        }
    }
}

/// Radix `size` prop on Slider. Default is [`Self::Two`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SliderSize {
    One,
    #[default]
    Two,
    Three,
}

impl SliderSize {
    pub const ALL: [Self; 3] = [Self::One, Self::Two, Self::Three];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::One => "1",
            Self::Two => "2",
            Self::Three => "3",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::One => "Size 1",
            Self::Two => "Size 2",
            Self::Three => "Size 3",
        }
    }

    fn from_control_size(size: crate::RadixSize) -> Self {
        match size {
            crate::RadixSize::One => Self::One,
            crate::RadixSize::Two => Self::Two,
            crate::RadixSize::Three => Self::Three,
        }
    }

    pub fn control_size(self) -> crate::RadixSize {
        match self {
            Self::One => crate::RadixSize::One,
            Self::Two => crate::RadixSize::Two,
            Self::Three => crate::RadixSize::Three,
        }
    }
}

pub fn slider_theme(look: &Look) -> Arc<dyn SliderTheme> {
    slider_theme_with(look, SliderVariant::default())
}

pub fn slider_theme_with(look: &Look, variant: SliderVariant) -> Arc<dyn SliderTheme> {
    slider_theme_for(look, variant, SliderSize::from_control_size(look.size()))
}

/// Bind Radix's semantic size before adapting to the SDK theme contract.
/// The Radix slider drawn from `.lss` (`style/slider.lss`); follows the look's later edits.
pub fn slider_theme_for(look: &Look, variant: SliderVariant, size: SliderSize) -> Arc<dyn SliderTheme> {
    let binding = xgpui_luma::style::StyleBinding::new("slider", crate::textfield::live_source(look));
    let binding = match variant {
        SliderVariant::Surface => binding,
        SliderVariant::Classic => binding.variant("classic"),
        SliderVariant::Soft => binding.variant("soft"),
    };
    let binding = if size != SliderSize::Two {
        binding.attr("size", size.as_str())
    } else {
        binding
    };
    Arc::new(StyledSliderTheme::new(binding))
}

pub fn slider_template_for(look: &Look, variant: SliderVariant, size: SliderSize) -> Arc<dyn SliderTemplate> {
    Arc::new(ThemedSliderTemplate::new(slider_theme_for(look, variant, size)))
}

pub fn slider_template(look: &Look, variant: SliderVariant) -> Arc<dyn SliderTemplate> {
    Arc::new(ThemedSliderTemplate::new(slider_theme_with(look, variant)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scale::ScaleFamily;
    use crate::semantic::SemanticRole;
    use xgpui_luma::theme::InteractionState;

    #[test]
    fn shared_slider_geometry_preserves_modes_states_and_sizes() {
        let look = Look::built_in();
        for mode in [xgpui_luma::theme::ThemeMode::Light, xgpui_luma::theme::ThemeMode::Dark] {
            look.set_mode(mode);
            for variant in SliderVariant::ALL {
                for (size, height, track, thumb) in [
                    (SliderSize::One, 16.0, 4.0, 12.0),
                    (SliderSize::Two, 20.0, 6.0, 16.0),
                    (SliderSize::Three, 24.0, 8.0, 20.0),
                ] {
                    let theme = slider_theme_with(&look.with_size(size.control_size()), variant);
                    for bits in 0..16 {
                        let state = InteractionState {
                            hovered: bits & 1 != 0,
                            pressed: bits & 2 != 0,
                            focused: bits & 4 != 0,
                            disabled: bits & 8 != 0,
                            ..Default::default()
                        };
                        let painted = theme.resolve(state);
                        assert_eq!(
                            (painted.width, painted.height, painted.track_height, painted.thumb_size, painted.radius),
                            (120.0, height, track, thumb, height / 2.0)
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn explicit_radix_size_retains_numeric_semantics_at_sdk_boundary() {
        let look = Look::built_in();
        let theme = slider_theme_for(&look, SliderVariant::Surface, SliderSize::One);
        assert_eq!(theme.resolve(Default::default()).height, 16.0);
    }

    #[test]
    fn thumbs_keep_light_colors_in_dark_mode() {
        let look = Look::built_in();
        for variant in SliderVariant::ALL {
            let theme = slider_theme_with(&look, variant);
            for state in [
                InteractionState::default(),
                InteractionState { hovered: true, ..Default::default() },
                InteractionState { pressed: true, ..Default::default() },
                InteractionState { focused: true, ..Default::default() },
                InteractionState { disabled: true, ..Default::default() },
            ] {
                look.set_mode(xgpui_luma::theme::ThemeMode::Light);
                let light = theme.resolve(state);
                look.set_mode(xgpui_luma::theme::ThemeMode::Dark);
                let dark = theme.resolve(state);
                assert_eq!(light.thumb_background, dark.thumb_background);
            }
        }
    }

    #[test]
    fn surface_fill_uses_primary() {
        let look = Look::built_in();
        let theme = slider_theme_with(&look, SliderVariant::Surface);
        let resolved = theme.resolve(InteractionState::default());
        crate::control_group::assert_close(resolved.fill_background, look.resolve_role(SemanticRole::Primary).hsla());
    }

    #[test]
    fn soft_uses_gray_track_and_accent_fill() {
        let look = Look::built_in();
        let theme = slider_theme_with(&look, SliderVariant::Soft);
        let resolved = theme.resolve(InteractionState::default());
        crate::control_group::assert_close(resolved.fill_background, look.resolve_step(ScaleFamily::Color, 6).hsla());
        assert_ne!(
            resolved.track_background,
            look.resolve_step(ScaleFamily::Color, 4).hsla(),
            "soft track is gray, not accent"
        );
    }

    #[test]
    fn classic_thumb_is_more_elevated_than_surface() {
        let look = Look::built_in();
        let classic = slider_theme_with(&look, SliderVariant::Classic).resolve(InteractionState::default());
        let surface = slider_theme_with(&look, SliderVariant::Surface).resolve(InteractionState::default());
        assert!(classic.thumb_shadow.len() > surface.thumb_shadow.len());
    }
}

#[cfg(test)]
mod track_border_tests {
    /// Radix tracks keep no outline although the SDK's have one (#397).
    #[test]
    fn radix_tracks_draw_no_outline() {
        let look = crate::Look::built_in();
        for variant in [super::SliderVariant::Classic, super::SliderVariant::Surface, super::SliderVariant::Soft] {
            let slider = super::slider_theme_for(&look, variant, super::SliderSize::Two).resolve(Default::default());
            assert_eq!(slider.track_border_width, 0.0, "{variant:?}");
        }
        assert_eq!(crate::progress::progress_theme(&look).resolve(true).track_border_width, 0.0);
    }
}
