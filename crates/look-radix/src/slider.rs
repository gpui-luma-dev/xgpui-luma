//! Radix slider theme adapter (Classic / Surface / Soft).
//!
//! Matches `@radix-ui/themes` Slider variant CSS at a paint level:
//! - **Surface** — gray track with an inset rim; accent fill; flat thumb ring
//! - **Classic** — same gray track, raised thumb (shadow-1 style)
//! - **Soft** — gray tint track; accent-6 fill; softer thumb elevation

use std::sync::Arc;

use gpui::{BoxShadow, Hsla, point, px};
use xgpui_luma::controls::slider::{SliderLook, SliderTemplate, SliderTheme, SliderThumbSize, ThemedSliderTemplate};
use xgpui_luma::theme::{ControlSize, ControlSizes, InteractionLayer, InteractionState};

use crate::look::Look;
use crate::scale::ScaleFamily;
use crate::semantic::SemanticRole;

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

    fn from_control_size(size: ControlSize) -> Self {
        match size {
            ControlSize::Sm => Self::One,
            ControlSize::Md => Self::Two,
            ControlSize::Lg => Self::Three,
        }
    }

    pub fn control_size(self) -> ControlSize {
        match self {
            Self::One => ControlSize::Sm,
            Self::Two => ControlSize::Md,
            Self::Three => ControlSize::Lg,
        }
    }
}

struct SliderThemeAdapter {
    look: Look,
    variant: SliderVariant,
    size: Option<SliderSize>,
}

fn alpha(color: Hsla, a: f32) -> Hsla {
    Hsla { a, ..color }
}

impl SliderTheme for SliderThemeAdapter {
    fn resolve(&self, size: ControlSize, thumb_size: Option<SliderThumbSize>, state: InteractionState) -> SliderLook {
        let look = &self.look;
        let gray = |n| look.resolve_step(ScaleFamily::Gray, n).hsla();
        let accent = |n| look.resolve_step(ScaleFamily::Color, n).hsla();
        let layer = state.layer();

        let fill = if state.disabled {
            gray(8)
        } else {
            match (self.variant, layer) {
                // Soft range is `--accent-6` (see radix slider.css).
                (SliderVariant::Soft, _) => accent(6),
                (_, InteractionLayer::Pressed) => accent(11),
                (_, InteractionLayer::Hovered) => accent(10),
                _ => look.resolve_role(SemanticRole::Primary).hsla(),
            }
        };

        // Soft track is gray (not accent). Surface/Classic share gray-a3.
        let track = if state.disabled {
            alpha(gray(4), 0.55)
        } else {
            match self.variant {
                SliderVariant::Soft => alpha(gray(4), 0.65),
                SliderVariant::Surface | SliderVariant::Classic => alpha(gray(3), 0.55),
            }
        };

        let thumb_bg = if state.disabled {
            look.light_gray_step(3)
        } else {
            gpui::white()
        };

        let thumb_border = if state.disabled {
            gray(6)
        } else {
            match self.variant {
                SliderVariant::Soft => alpha(accent(6), 0.35),
                SliderVariant::Surface => alpha(gpui::black(), 0.16),
                SliderVariant::Classic => alpha(gpui::black(), 0.22),
            }
        };

        let thumb_shadow = if state.disabled {
            Vec::new()
        } else {
            match self.variant {
                SliderVariant::Surface => surface_thumb_shadow(),
                SliderVariant::Classic => classic_thumb_shadow(),
                SliderVariant::Soft => soft_thumb_shadow(),
            }
        };

        let radix_size = self.size.unwrap_or_else(|| SliderSize::from_control_size(size));
        let (geometry, _) = slider_geometry(look, radix_size, thumb_size);
        SliderLook {
            thumb_border_width: look.metrics().border_width.default,
            decoration: look.stylesheet_snapshot().slider.decoration(radix_size.control_size()),
            track_background: track,
            fill_background: fill,
            thumb_background: thumb_bg,
            thumb_border,
            thumb_shadow,
            width: geometry.width,
            height: geometry.height,
            track_height: geometry.track_height,
            thumb_size: geometry.thumb_size,
            radius: geometry.height / 2.0,
        }
    }
}

/// Shared metric values and provenance consumed by the Radix slider adapter.
pub fn slider_geometry(
    look: &Look,
    size: SliderSize,
    thumb_size: Option<SliderThumbSize>,
) -> (xgpui_luma::theme::stylesheet::SliderGeometry, xgpui_luma::theme::stylesheet::SliderGeometrySources) {
    use xgpui_luma::theme::stylesheet::{CommonStylesheet, SliderGeometry};
    let fallback =
        SliderGeometry::sdk_default(&look.metrics(), size.control_size(), thumb_size, CommonStylesheet::sdk_default());
    let thumb_key = thumb_size.map(|size| match size {
        SliderThumbSize::Sm => ControlSize::Sm,
        SliderThumbSize::Md => ControlSize::Md,
        SliderThumbSize::Lg => ControlSize::Lg,
    });
    let (geometry, mut sources) =
        look.stylesheet_snapshot().slider.inspect_geometry(size.control_size(), thumb_key, &fallback);
    // Numeric identities belong to Radix, including inspector descriptions.
    for source in [&mut sources.width, &mut sources.height, &mut sources.track_height, &mut sources.thumb_size] {
        use xgpui_luma::theme::provenance::MetricSource;
        let text = match source {
            MetricSource::Authored { key } => key,
            MetricSource::Derived { note } => note,
            MetricSource::Constant { .. } | MetricSource::ScaleStep { .. } | MetricSource::Scaffold { .. } => continue,
        };
        for (semantic, numeric) in [("sm", "1"), ("md", "2"), ("lg", "3")] {
            *text = text
                .replace(&format!("common.slider.sizes.{semantic}."), &format!("common.slider.sizes.{numeric}."))
                .replace(&format!("instance thumb_size {semantic}"), &format!("instance thumb_size {numeric}"));
        }
    }
    (geometry, sources)
}

fn surface_thumb_shadow() -> Vec<BoxShadow> {
    vec![BoxShadow {
        offset: point(px(0.0), px(0.0)),
        blur_radius: px(0.0),
        spread_radius: px(1.0),
        color: alpha(gpui::black(), 0.16),
        inset: false,
    }]
}

fn classic_thumb_shadow() -> Vec<BoxShadow> {
    vec![
        BoxShadow {
            offset: point(px(0.0), px(0.0)),
            blur_radius: px(0.0),
            spread_radius: px(1.0),
            color: alpha(gpui::black(), 0.18),
            inset: false,
        },
        BoxShadow {
            offset: point(px(0.0), px(1.0)),
            blur_radius: px(3.0),
            spread_radius: px(0.0),
            color: alpha(gpui::black(), 0.08),
            inset: false,
        },
        BoxShadow {
            offset: point(px(0.0), px(2.0)),
            blur_radius: px(4.0),
            spread_radius: px(-1.0),
            color: alpha(gpui::black(), 0.06),
            inset: false,
        },
    ]
}

fn soft_thumb_shadow() -> Vec<BoxShadow> {
    vec![
        BoxShadow {
            offset: point(px(0.0), px(0.0)),
            blur_radius: px(0.0),
            spread_radius: px(1.0),
            color: alpha(gpui::black(), 0.14),
            inset: false,
        },
        BoxShadow {
            offset: point(px(0.0), px(1.0)),
            blur_radius: px(2.0),
            spread_radius: px(0.0),
            color: alpha(gpui::black(), 0.10),
            inset: false,
        },
    ]
}

pub fn slider_theme(look: &Look) -> Arc<dyn SliderTheme> {
    slider_theme_with(look, SliderVariant::default())
}

pub fn slider_theme_with(look: &Look, variant: SliderVariant) -> Arc<dyn SliderTheme> {
    Arc::new(SliderThemeAdapter { look: look.clone(), variant, size: None })
}

/// Bind Radix's semantic size before adapting to the SDK theme contract.
pub fn slider_theme_for(look: &Look, variant: SliderVariant, size: SliderSize) -> Arc<dyn SliderTheme> {
    Arc::new(SliderThemeAdapter { look: look.clone(), variant, size: Some(size) })
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

    #[test]
    fn numeric_overrides_reach_typed_sizes_and_explicit_thumbs() {
        let look = Look::built_in();
        let mut stylesheet = look.common_stylesheet();
        let numeric = xgpui_luma::theme::stylesheet::CommonStylesheet::parse(
            "[common.slider.sizes.\"1\"]\nheight = 0\n[common.slider.sizes.\"3\"]\nthumb_size = 31",
        )
        .unwrap();
        numeric.slider.apply_to(&mut stylesheet.slider);
        look.set_common_stylesheet(stylesheet);
        let (geometry, _) = slider_geometry(&look, SliderSize::One, Some(SliderThumbSize::Lg));
        assert_eq!(geometry.height, 0.0);
        assert_eq!(geometry.thumb_size, 31.0);
    }

    #[test]
    fn shared_slider_geometry_preserves_modes_states_sizes_and_explicit_thumbs() {
        let look = Look::built_in();
        for mode in [xgpui_luma::theme::ThemeMode::Light, xgpui_luma::theme::ThemeMode::Dark] {
            look.set_mode(mode);
            for variant in SliderVariant::ALL {
                let theme = slider_theme_with(&look, variant);
                for (size, height, track, thumb) in [
                    (ControlSize::Sm, 16.0, 4.0, 12.0),
                    (ControlSize::Md, 20.0, 6.0, 16.0),
                    (ControlSize::Lg, 24.0, 8.0, 20.0),
                ] {
                    for bits in 0..16 {
                        let state = InteractionState {
                            hovered: bits & 1 != 0,
                            pressed: bits & 2 != 0,
                            focused: bits & 4 != 0,
                            disabled: bits & 8 != 0,
                            ..Default::default()
                        };
                        for (selection, expected) in [
                            (None, thumb),
                            (Some(SliderThumbSize::Sm), 12.0),
                            (Some(SliderThumbSize::Md), 16.0),
                            (Some(SliderThumbSize::Lg), 20.0),
                        ] {
                            let painted = theme.resolve(size, selection, state);
                            assert_eq!(
                                (
                                    painted.width,
                                    painted.height,
                                    painted.track_height,
                                    painted.thumb_size,
                                    painted.radius
                                ),
                                (120.0, height, track, expected, height / 2.0)
                            );
                            let (inspected, _) = slider_geometry(&look, SliderSize::from_control_size(size), selection);
                            assert_eq!(inspected.width, painted.width);
                            assert_eq!(inspected.thumb_size, painted.thumb_size);
                        }
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
        assert_eq!(sizes.sm.height, Some(16.0));
        assert_eq!(sizes.md.height, Some(20.0));
        assert_eq!(sizes.lg.height, Some(24.0));
        let theme = slider_theme_for(&look, SliderVariant::Surface, SliderSize::One);
        assert_eq!(theme.resolve(ControlSize::Lg, None, Default::default()).height, 16.0);
        let (_, sources) = slider_geometry(&look, SliderSize::Two, Some(SliderThumbSize::Lg));
        assert!(matches!(sources.height,
            xgpui_luma::theme::provenance::MetricSource::Authored { key }
            if key == "common.slider.sizes.2.height"));
        assert!(matches!(sources.thumb_size,
            xgpui_luma::theme::provenance::MetricSource::Derived { note }
            if note.contains("common.slider.sizes.3.thumb_size")));
    }

    #[test]
    fn shared_slider_geometry_updates_live_themes_without_changing_palettes() {
        let look = Look::built_in();
        let theme = slider_theme(&look);
        let original = theme.resolve(ControlSize::Md, None, Default::default());
        let mut stylesheet = look.common_stylesheet();
        stylesheet.slider.geometry.width = Some(200.0);
        stylesheet.slider.sizes.md.height = Some(30.0);
        stylesheet.slider.sizes.md.thumb_size = Some(27.0);
        stylesheet.slider.sizes.lg.thumb_size = Some(31.0);
        look.set_common_stylesheet(stylesheet.clone());
        let custom = theme.resolve(ControlSize::Md, Some(SliderThumbSize::Lg), Default::default());
        assert_eq!((custom.width, custom.height, custom.thumb_size, custom.radius), (200.0, 30.0, 31.0, 15.0));
        assert_eq!(custom.track_background, original.track_background);
        assert_eq!(custom.fill_background, original.fill_background);
        let fork = look.fork();
        fork.set_common_stylesheet(Default::default());
        assert_eq!(look.common_stylesheet(), stylesheet);
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
                let light = theme.resolve(ControlSize::Md, None, state);
                look.set_mode(xgpui_luma::theme::ThemeMode::Dark);
                let dark = theme.resolve(ControlSize::Md, None, state);
                assert_eq!(light.thumb_background, dark.thumb_background);
            }
        }
    }

    #[test]
    fn surface_fill_uses_primary() {
        let look = Look::built_in();
        let theme = slider_theme_with(&look, SliderVariant::Surface);
        let resolved = theme.resolve(ControlSize::Md, None, InteractionState::default());
        assert_eq!(resolved.fill_background, look.resolve_role(SemanticRole::Primary).hsla());
    }

    #[test]
    fn soft_uses_gray_track_and_accent_fill() {
        let look = Look::built_in();
        let theme = slider_theme_with(&look, SliderVariant::Soft);
        let resolved = theme.resolve(ControlSize::Md, None, InteractionState::default());
        assert_eq!(resolved.fill_background, look.resolve_step(ScaleFamily::Color, 6).hsla());
        assert_ne!(
            resolved.track_background,
            look.resolve_step(ScaleFamily::Color, 4).hsla(),
            "soft track is gray, not accent"
        );
    }

    #[test]
    fn classic_thumb_is_more_elevated_than_surface() {
        let look = Look::built_in();
        let classic = slider_theme_with(&look, SliderVariant::Classic).resolve(
            ControlSize::Md,
            None,
            InteractionState::default(),
        );
        let surface = slider_theme_with(&look, SliderVariant::Surface).resolve(
            ControlSize::Md,
            None,
            InteractionState::default(),
        );
        assert!(classic.thumb_shadow.len() > surface.thumb_shadow.len());
    }
}

/// Constructs the native Radix slider definition in pure Rust without preloading TOML.
pub fn native_radix_slider() -> xgpui_luma::theme::stylesheet::SliderStylesheet {
    use xgpui_luma::theme::stylesheet::{SliderGeometryStylesheet, SliderStylesheet};
    SliderStylesheet {
        geometry: SliderGeometryStylesheet { width: Some(120.0), ..Default::default() },
        sizes: ControlSizes {
            sm: SliderGeometryStylesheet {
                height: Some(16.0),
                track_height: Some(4.0),
                thumb_size: Some(12.0),
                ..Default::default()
            },
            md: SliderGeometryStylesheet {
                height: Some(20.0),
                track_height: Some(6.0),
                thumb_size: Some(16.0),
                ..Default::default()
            },
            lg: SliderGeometryStylesheet {
                height: Some(24.0),
                track_height: Some(8.0),
                thumb_size: Some(20.0),
                ..Default::default()
            },
            ..Default::default()
        },
    }
}

/// Applies Radix slider geometry in Rust.
pub(crate) fn apply_radix_slider_stylesheet(sheet: &mut xgpui_luma::theme::stylesheet::CommonStylesheet) {
    native_radix_slider().apply_to(&mut sheet.slider);
}
