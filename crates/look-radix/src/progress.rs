//! Radix progress bar styling and SDK template adapter.
use std::sync::Arc;
use gpui::{BoxShadow, div, point, prelude::*, px};
use xgpui_luma::controls::progress::{ProgressLook, ProgressTemplate, ProgressTheme, ThemedLinearProgressTemplate};
use xgpui_luma::theme::ControlSize;
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
    pub fn control_size(self) -> ControlSize {
        match self {
            Self::One => ControlSize::Sm,
            Self::Two => ControlSize::Md,
            Self::Three => ControlSize::Lg,
        }
    }
}
fn height(size: ControlSize) -> f32 {
    crate::look::embedded_common_stylesheet()
        .progress
        .resolve_geometry(crate::look::sdk_size_key(size), Default::default())
        .track_height
}
/// Matches Radix's radius-factor and radius-thumb rules, capped at half the height.
pub fn resolve_progress_radius(size: ControlSize, radius: Radius) -> f32 {
    progress_radius_for_height(height(size), radius)
}

fn progress_radius_for_height(h: f32, radius: Radius) -> f32 {
    let thumb = match radius {
        Radius::None | Radius::Small => 0.5,
        _ => 9999.0,
    };
    (radius.factor() * (h / 3.0).max(thumb)).min(h / 2.0)
}
struct ProgressThemeAdapter {
    look: Look,
    variant: ProgressVariant,
    paint: Paint,
}
impl ProgressTheme for ProgressThemeAdapter {
    fn resolve(&self, enabled: bool, size: ControlSize) -> ProgressLook {
        let family = match self.paint.tone {
            Tone::Accent => ScaleFamily::Color,
            Tone::Gray => ScaleFamily::Gray,
        };
        let gray = |n| self.look.resolve_step(ScaleFamily::Gray, n).hsla();
        let step = |n| self.look.resolve_step(family, n).hsla();
        let track = match self.variant {
            ProgressVariant::Surface => gray(3),
            ProgressVariant::Soft => gray(4),
        };
        let fill = if !enabled {
            gray(8)
        } else if self.paint.high_contrast {
            step(12)
        } else {
            step(match self.variant {
                ProgressVariant::Surface => 9,
                ProgressVariant::Soft => 8,
            })
        };
        let height = self
            .look
            .stylesheet_snapshot()
            .progress
            .resolve_geometry(
                crate::look::sdk_size_key(size),
                xgpui_luma::theme::stylesheet::ProgressGeometry { track_height: height(size), ..Default::default() },
            )
            .track_height;
        ProgressLook {
            geometry: self
                .look
                .stylesheet_snapshot()
                .progress
                .resolve_geometry(size.stylesheet_key(), Default::default()),
            track_color: track,
            progress_color: fill,
            thumb_color: fill,
            track_height: height,
            thumb_size: 0.0,
            size: height,
            stroke_width: height,
        }
    }
}
pub fn progress_theme(look: &Look) -> Arc<dyn ProgressTheme> {
    progress_theme_with(look, ProgressVariant::Surface, Paint::accent())
}
pub fn progress_theme_with(look: &Look, variant: ProgressVariant, paint: Paint) -> Arc<dyn ProgressTheme> {
    Arc::new(ProgressThemeAdapter { look: look.clone(), variant, paint })
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
            let height = theme.resolve(model.enabled, model.size).track_height;
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
        assert_eq!([height(ControlSize::Sm), height(ControlSize::Md), height(ControlSize::Lg)], [4.0, 6.0, 8.0]);
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
                        let normal = theme.resolve(true, ControlSize::Md);
                        let step = if high_contrast {
                            12
                        } else if variant == ProgressVariant::Soft {
                            8
                        } else {
                            9
                        };
                        assert_eq!(normal.progress_color, look.resolve_step(family, step).hsla());
                        assert_eq!(
                            theme.resolve(false, ControlSize::Md).progress_color,
                            look.resolve_step(ScaleFamily::Gray, 8).hsla()
                        );
                    }
                }
            }
        }
    }
}

/// Native common geometry owned by this control.
pub(crate) fn native_progress_stylesheet() -> xgpui_luma::theme::stylesheet::ProgressStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    ProgressStylesheet {
        sizes: [("1", 4.0), ("2", 6.0), ("3", 8.0)]
            .into_iter()
            .map(|(key, height)| {
                (key.into(), ProgressGeometryStylesheet { track_height: Some(height), ..Default::default() })
            })
            .collect(),
        ..Default::default()
    }
}

#[cfg(all(test, feature = "test-support"))]
mod render_tests {
    use super::*;
    use gpui::{Context, Render, TestAppContext, Window};
    use xgpui_luma::controls::progress::{ProgressDirection, ProgressRenderModel};

    struct Harness {
        template: Arc<dyn ProgressTemplate>,
        size: ControlSize,
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
                size: self.size,
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
                                template: progress_template(&look, variant, Paint::accent(), radius),
                                size,
                                direction,
                                percentage,
                            });
                            cx.run_until_parked();
                            cx.update(|window, _| {
                                let colors = progress_theme_with(&look, variant, Paint::accent()).resolve(true, size);
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
