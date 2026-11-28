//! Radix progress bar styling and SDK template adapter.
#![allow(deprecated)] // Still reads a deprecated Look until its family has a style-override replacement (#433).
use std::sync::Arc;
use gpui::{BoxShadow, div, point, prelude::*, px};
use xgpui_luma::controls::progress::{ProgressTemplate, ProgressTheme, StyledProgressTheme, ThemedLinearProgressTemplate};
use crate::RadixSize;
use crate::{Look, Paint, Radius, ScaleFamily, Tone};

/// Radix progress bar variants.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ProgressVariant {
    #[default]
    Surface,
    Soft,
}
impl ProgressVariant {
    pub const ALL: [Self; 2] = [Self::Surface, Self::Soft];
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Surface => "surface",
            Self::Soft => "soft",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Surface => "Surface",
            Self::Soft => "Soft",
        }
    }
}
/// Radix progress sizes, with heights 4, 6, and 8 pixels.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ProgressSize {
    One,
    #[default]
    Two,
    Three,
}
impl ProgressSize {
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
    pub fn control_size(self) -> RadixSize {
        match self {
            Self::One => RadixSize::One,
            Self::Two => RadixSize::Two,
            Self::Three => RadixSize::Three,
        }
    }
}
/// The track height Radix's `progress.lss` gives `size`.
fn height(size: RadixSize) -> f32 {
    crate::style::lss_number("progress", &[("size", size.key())], "track_height")
}
/// Matches Radix's radius-factor and radius-thumb rules, capped at half the height.
pub fn resolve_progress_radius(size: RadixSize, radius: Radius) -> f32 {
    progress_radius_for_height(height(size), radius)
}

fn progress_radius_for_height(h: f32, radius: Radius) -> f32 {
    let thumb = match radius {
        Radius::None | Radius::Small => 0.5,
        _ => 9999.0,
    };
    (radius.factor() * (h / 3.0).max(thumb)).min(h / 2.0)
}
pub fn progress_theme(look: &Look) -> Arc<dyn ProgressTheme> {
    progress_theme_with(look, ProgressVariant::Surface, Paint::accent())
}
/// Radix progress drawn from `.lss` (`style/progress.lss`) at the look's size; follows the look's later edits.
pub fn progress_theme_with(look: &Look, variant: ProgressVariant, paint: Paint) -> Arc<dyn ProgressTheme> {
    let size = look.size();
    let binding = xgpui_luma::style::StyleBinding::new("progress", crate::textfield::live_source(look))
        .state("high_contrast", paint.high_contrast);
    let binding = if variant == ProgressVariant::Soft {
        binding.variant("soft")
    } else {
        binding
    };
    let binding = if paint.tone == Tone::Gray {
        binding.attr("tone", "gray")
    } else {
        binding
    };
    let binding = if size != RadixSize::Two {
        binding.attr("size", size.key())
    } else {
        binding
    };
    Arc::new(StyledProgressTheme::new(binding))
}
pub fn progress_template(
    look: &Look,
    variant: ProgressVariant,
    paint: Paint,
    radius: Radius,
) -> Arc<dyn ProgressTemplate> {
    let theme = progress_theme_with(look, variant, paint);
    let template = ThemedLinearProgressTemplate::new(theme.clone())
        .with_track_radius(move |height| progress_radius_for_height(height, radius));
    if variant == ProgressVariant::Surface {
        let look = look.clone();
        Arc::new(template.with_modifier(move |root, model| {
            let height = {
                use xgpui_luma::style::PartRead;
                xgpui_luma::controls::progress::resolve_progress(&theme.style(), model.enabled)
                    .root()
                    .number("track_height")
            };
            root.child(div().absolute().inset_0().rounded(px(progress_radius_for_height(height, radius))).shadow(vec![
                BoxShadow {
                    offset: point(px(0.0), px(0.0)),
                    blur_radius: px(0.0),
                    spread_radius: px(1.0),
                    color: look.resolve_step(ScaleFamily::Gray, 4).hsla(),
                    inset: true,
                },
            ]))
        }))
    } else {
        Arc::new(template)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xgpui_luma::theme::ThemeMode;
    #[test]
    fn sizes_and_radii_match_radix_geometry() {
        for size in ProgressSize::ALL {
            let size = size.control_size();
            assert_eq!(resolve_progress_radius(size, Radius::None), 0.0);
            assert_eq!(resolve_progress_radius(size, Radius::Small), height(size) / 4.0);
            for radius in [Radius::Medium, Radius::Large, Radius::Full] {
                assert_eq!(resolve_progress_radius(size, radius), height(size) / 2.0);
            }
        }
        assert_eq!([height(RadixSize::One), height(RadixSize::Two), height(RadixSize::Three)], [4.0, 6.0, 8.0]);
    }
    #[test]
    fn palettes_follow_mode_tone_contrast_and_disabled_state() {
        let look = Look::built_in();
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            for variant in ProgressVariant::ALL {
                for tone in [Tone::Accent, Tone::Gray] {
                    let family = if tone == Tone::Accent {
                        ScaleFamily::Color
                    } else {
                        ScaleFamily::Gray
                    };
                    for high_contrast in [false, true] {
                        let theme = progress_theme_with(&look, variant, Paint { tone, high_contrast });
                        let normal = theme.resolve(true);
                        let step = if high_contrast {
                            12
                        } else if variant == ProgressVariant::Soft {
                            8
                        } else {
                            9
                        };
                        crate::control_group::assert_close(
                            normal.progress_color,
                            look.resolve_step(family, step).hsla(),
                        );
                        crate::control_group::assert_close(
                            theme.resolve(false).progress_color,
                            look.resolve_step(ScaleFamily::Gray, 8).hsla(),
                        );
                    }
                }
            }
        }
    }
}

#[cfg(all(test, feature = "test-support"))]
mod render_tests {
    use super::*;
    use gpui::{Context, Render, TestAppContext, Window};
    use xgpui_luma::controls::progress::{ProgressDirection, ProgressRenderModel};

    struct Harness {
        template: Arc<dyn ProgressTemplate>,
        direction: ProgressDirection,
        percentage: f32,
    }

    impl Render for Harness {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let id = "progress".into();
            let model = ProgressRenderModel {
                id: &id,
                range: Default::default(),
                value: self.percentage * 100.0,
                percentage: self.percentage,
                enabled: true,
                direction: self.direction,
                show_thumb: false,
                indeterminate: false,
                phase: 0.0,
            };
            div().size(px(100.0)).child(self.template.render(&model, window, cx))
        }
    }

    #[test]
    fn radix_progress_paints_rounded_fill_in_every_direction() {
        let look = Look::built_in();
        for variant in ProgressVariant::ALL {
            for radius in Radius::ALL {
                for size in ProgressSize::ALL {
                    let size = size.control_size();
                    for direction in [
                        ProgressDirection::LeftToRight,
                        ProgressDirection::RightToLeft,
                        ProgressDirection::BottomToTop,
                        ProgressDirection::TopToBottom,
                    ] {
                        for percentage in [0.0, 0.01, 0.5, 0.99, 1.0] {
                            let mut app = TestAppContext::single();
                            let (_, cx) = app.add_window_view(|_, _| Harness {
                                template: progress_template(&look.with_size(size), variant, Paint::accent(), radius),
                                direction,
                                percentage,
                            });
                            cx.run_until_parked();
                            cx.update(|window, _| {
                                let colors =
                                    progress_theme_with(&look.with_size(size), variant, Paint::accent()).resolve(true);
                                let quads = window.painted_quads();
                                let track =
                                    quads.iter().find(|q| q.background.as_solid() == Some(colors.track_color)).unwrap();
                                let fills: Vec<_> = quads
                                    .iter()
                                    .filter(|q| q.background.as_solid() == Some(colors.progress_color))
                                    .collect();
                                if percentage == 0.0 {
                                    assert!(fills.is_empty());
                                } else {
                                    assert_eq!(fills.len(), 1);
                                    let fill = fills[0];
                                    assert_eq!(
                                        fill.corner_radii,
                                        track.corner_radii.clamp_radii_for_quad_size(fill.bounds.size)
                                    );
                                    let expected_radius =
                                        px(resolve_progress_radius(size, radius)).scale(window.scale_factor());
                                    assert_eq!(track.corner_radii.top_left, expected_radius);
                                    assert!(fill.bounds.left() >= track.bounds.left());
                                    assert!(fill.bounds.right() <= track.bounds.right());
                                    assert!(fill.bounds.top() >= track.bounds.top());
                                    assert!(fill.bounds.bottom() <= track.bounds.bottom());
                                    if percentage == 1.0 {
                                        assert_eq!(fill.bounds, track.bounds);
                                    }
                                }
                            });
                        }
                    }
                }
            }
        }
    }
}
