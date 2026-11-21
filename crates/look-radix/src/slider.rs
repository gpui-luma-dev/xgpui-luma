//! Radix slider theme adapter (Classic / Surface / Soft).
//!
//! Matches `@radix-ui/themes` Slider variant CSS at a paint level:
//! - **Surface** — gray track with an inset rim; accent fill; flat thumb ring
//! - **Classic** — same gray track, raised thumb (shadow-1 style)
//! - **Soft** — gray tint track; accent-6 fill; softer thumb elevation

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

/// Shared metric values and provenance consumed by the Radix slider adapter.
pub fn slider_geometry(
    look: &Look,
    size: SliderSize,
) -> (xgpui_luma::theme::stylesheet::SliderGeometry, xgpui_luma::theme::stylesheet::SliderGeometrySources) {
    use xgpui_luma::theme::stylesheet::SliderGeometry;
    let control = size.control_size();
    let fallback = SliderGeometry::sdk_default(
        &look.metrics_for(control),
        xgpui_luma::theme::stylesheet::CommonStylesheet::sdk_default(),
    );
    let (geometry, sources) = look.stylesheet_snapshot().slider.inspect_geometry_for(control.key(), &fallback);
    (geometry, sources)
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
    fn numeric_overrides_reach_radix_sizes() {
        let look = Look::built_in();
        let mut stylesheet = look.common_stylesheet();
        let numeric = xgpui_luma::theme::stylesheet::CommonStylesheet::parse(
            "[common.slider.sizes.\"1\"]\nheight = 0\n[common.slider.sizes.\"3\"]\nthumb_size = 31",
        )
        .unwrap();
        numeric.slider.apply_to(&mut stylesheet.slider);
        look.set_common_stylesheet(stylesheet);
        assert_eq!(slider_geometry(&look, SliderSize::One).0.height, 0.0);
        assert_eq!(slider_geometry(&look, SliderSize::Three).0.thumb_size, 31.0);
    }

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
                    let (inspected, _) = slider_geometry(&look, size);
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
                        assert_eq!(inspected.width, painted.width);
                        assert_eq!(inspected.thumb_size, painted.thumb_size);
                    }
                }
            }
        }
    }

    #[test]
    fn explicit_radix_size_retains_numeric_semantics_at_sdk_boundary() {
        let look = Look::built_in();
        let stylesheet = look.common_stylesheet();
        let sizes = &stylesheet.slider.sizes;
        assert_eq!(sizes.get("1").and_then(|size| size.height), Some(16.0));
        assert_eq!(sizes.get("2").and_then(|size| size.height), Some(20.0));
        assert_eq!(sizes.get("3").and_then(|size| size.height), Some(24.0));
        let theme = slider_theme_for(&look, SliderVariant::Surface, SliderSize::One);
        assert_eq!(theme.resolve(Default::default()).height, 16.0);
        let (_, sources) = slider_geometry(&look, SliderSize::Two);
        assert!(matches!(sources.height,
            xgpui_luma::theme::provenance::MetricSource::Authored { key }
            if key == "common.slider.sizes.2.height"));
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

/// Constructs the native Radix slider definition in pure Rust without preloading TOML.
pub fn native_radix_slider() -> xgpui_luma::theme::stylesheet::SliderStylesheet {
    use xgpui_luma::theme::stylesheet::{SliderGeometryStylesheet, SliderStylesheet};
    SliderStylesheet {
        geometry: SliderGeometryStylesheet { width: Some(120.0), ..Default::default() },
        sizes: [("1", 16.0, 4.0, 12.0), ("2", 20.0, 6.0, 16.0), ("3", 24.0, 8.0, 20.0)]
            .into_iter()
            .map(|(key, height, track_height, thumb_size)| {
                (
                    key.to_string(),
                    SliderGeometryStylesheet {
                        height: Some(height),
                        track_height: Some(track_height),
                        thumb_size: Some(thumb_size),
                        ..Default::default()
                    },
                )
            })
            .collect(),
    }
}

/// Applies Radix slider geometry in Rust.
pub(crate) fn apply_radix_slider_stylesheet(sheet: &mut xgpui_luma::theme::stylesheet::CommonStylesheet) {
    native_radix_slider().apply_to(&mut sheet.slider);
}
