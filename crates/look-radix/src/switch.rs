//! Radix switch theme adapter.
//!
//! Geometry matches `@radix-ui/themes` Switch at `--scaling: 1`:
//! - sizes 1–3 → height 16 / 20 / 24; width = height × 1.75; thumb inset 1px
//! - radius = `max(--radius-{1|2} × factor, --radius-thumb)` (pill for medium+)

use std::sync::Arc;

use gpui::{BoxShadow, Hsla, point, px};
use xgpui_luma::controls::button::ButtonTemplate;
use xgpui_luma::controls::switch::{SwitchPalette, SwitchScale, SwitchTheme, ThemedSwitchTemplate};
use xgpui_luma::theme::{ControlSize, InteractionState, MetricTokens};

use crate::button::Paint;
use crate::button_layout::Radius;
use crate::look::Look;
use crate::scale::ScaleFamily;
use crate::semantic::SemanticRole;
use crate::tone::Tone;
use crate::typography::{font_family, label_typography};

/// The Radix Themes switch variant scheme (Classic / Surface / Soft).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SwitchVariant {
    Classic,
    #[default]
    Surface,
    Soft,
}

/// Radix `size` prop on Switch. Default is [`Self::Two`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SwitchSize {
    One,
    #[default]
    Two,
    Three,
}

impl SwitchSize {
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

    /// Track height (`--switch-height`).
    pub fn height(self) -> f32 {
        switch_scale_for(self, Radius::Medium).track_height
    }

    /// `--radius-1` for size 1, `--radius-2` for sizes 2–3 (before factor).
    fn radius_base(self) -> f32 {
        match self {
            Self::One => 3.0,
            Self::Two | Self::Three => 4.0,
        }
    }
}

/// Radix `--radius-thumb` for the given theme radius setting.
fn radius_thumb(radius: Radius) -> f32 {
    match radius {
        Radius::None | Radius::Small => 0.5,
        Radius::Medium | Radius::Large | Radius::Full => 9999.0,
    }
}

pub fn resolve_switch_radius(size: SwitchSize, radius: Radius) -> f32 {
    let themed = size.radius_base() * radius.factor();
    themed.max(radius_thumb(radius)).min(size.height() / 2.0)
}

pub fn switch_scale_for(size: SwitchSize, radius: Radius) -> SwitchScale {
    switch_scale_from_stylesheet(crate::look::embedded_common_stylesheet(), &MetricTokens::default(), size, radius)
}

fn switch_scale_from_stylesheet(
    stylesheet: &xgpui_luma::theme::stylesheet::CommonStylesheet,
    metrics: &MetricTokens,
    size: SwitchSize,
    radius: Radius,
) -> SwitchScale {
    let mut fallback = SwitchScale::compute(size.control_size(), metrics, 1.0);
    fallback.label_baseline_shift = 0.0;
    let mut scale = stylesheet.switch.resolve_geometry(size.as_str(), fallback, 1.0);
    scale.track_radius = (size.radius_base() * radius.factor()).max(radius_thumb(radius)).min(scale.track_height / 2.0);
    scale
}

/// Shared authored dimensions and provenance used by the Radix switch adapter.
pub fn switch_geometry(look: &Look, size: SwitchSize) -> xgpui_luma::theme::stylesheet::ResolvedSwitchGeometry {
    let fallback = SwitchScale::compute(size.control_size(), &look.metrics(), 1.0);
    look.stylesheet_snapshot().switch.inspect_geometry(size.as_str(), fallback, 1.0)
}

struct SwitchThemeAdapter {
    look: Look,
    variant: SwitchVariant,
    paint: Paint,
    geometry: Option<(SwitchSize, Radius)>,
}

fn alpha(color: Hsla, a: f32) -> Hsla {
    Hsla { a, ..color }
}

fn thumb_shadow(on: bool) -> Vec<BoxShadow> {
    let soft = alpha(gpui::black(), if on { 0.12 } else { 0.08 });
    let hairline = alpha(gpui::black(), 0.06);
    vec![
        BoxShadow {
            offset: point(px(0.0), px(1.0)),
            blur_radius: px(3.0),
            spread_radius: px(0.0),
            color: soft,
            inset: false,
        },
        BoxShadow {
            offset: point(px(0.0), px(0.0)),
            blur_radius: px(0.0),
            spread_radius: px(1.0),
            color: hairline,
            inset: false,
        },
    ]
}

impl SwitchTheme for SwitchThemeAdapter {
    fn resolve(&self, on: bool, state: InteractionState, size: ControlSize) -> SwitchPalette {
        let look = &self.look;
        let high_contrast = self.paint.high_contrast;
        let family = match self.paint.tone {
            Tone::Accent => ScaleFamily::Color,
            Tone::Gray => ScaleFamily::Gray,
        };
        let step = |n| look.resolve_step(family, n).hsla();
        let gray = |n| look.resolve_step(ScaleFamily::Gray, n).hsla();
        let white = gpui::white();

        let accent_on = if high_contrast {
            step(12)
        } else {
            look.resolve_role(SemanticRole::Primary).hsla()
        };
        let soft_on = if high_contrast { step(6) } else { step(4) };

        let (track_background, track_border, thumb_background, thumb_border, thumb_shadow) = if state.disabled {
            let thumb = look.light_gray_step(2);
            (alpha(gray(3), 0.55), alpha(gray(3), 0.55), thumb, thumb, thumb_shadow(false))
        } else {
            match (self.variant, on) {
                (SwitchVariant::Surface, false) => {
                    (alpha(gray(3), 0.55), alpha(gray(5), 0.7), white, white, thumb_shadow(false))
                }
                (SwitchVariant::Surface, true) => (accent_on, accent_on, white, white, thumb_shadow(true)),
                (SwitchVariant::Classic, false) => {
                    (alpha(gray(4), 0.6), alpha(gray(5), 0.55), white, white, thumb_shadow(false))
                }
                (SwitchVariant::Classic, true) => (accent_on, alpha(gray(3), 0.5), white, white, thumb_shadow(true)),
                (SwitchVariant::Soft, false) => {
                    (alpha(gray(3), 0.55), alpha(gray(3), 0.55), white, white, thumb_shadow(false))
                }
                (SwitchVariant::Soft, true) => (soft_on, soft_on, white, white, thumb_shadow(true)),
            }
        };

        SwitchPalette {
            focus_border: look.resolve_role(SemanticRole::Focus).hsla(),
            track_background,
            track_border,
            thumb_background,
            thumb_border,
            thumb_shadow,
            label_color: if state.disabled {
                look.resolve_role(SemanticRole::MutedForeground).hsla()
            } else {
                look.resolve_role(SemanticRole::Foreground).hsla()
            },
            label_typography: label_typography(size),
            label_font_family: font_family(look),
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }

    fn scale(&self, size: ControlSize, scale_factor: f32) -> SwitchScale {
        let _ = scale_factor;
        if let Some((radix_size, radius)) = self.geometry {
            return switch_scale_from_stylesheet(
                &self.look.stylesheet_snapshot(),
                &self.look.metrics(),
                radix_size,
                radius,
            );
        }
        let radix_size = match size {
            ControlSize::Sm => SwitchSize::One,
            ControlSize::Md => SwitchSize::Two,
            ControlSize::Lg => SwitchSize::Three,
        };
        switch_scale_from_stylesheet(&self.look.stylesheet_snapshot(), &self.look.metrics(), radix_size, Radius::Medium)
    }
}

pub fn switch_theme(look: &Look) -> Arc<dyn SwitchTheme> {
    switch_theme_with(look, SwitchVariant::default(), Paint::accent())
}

pub fn switch_theme_with(look: &Look, variant: SwitchVariant, paint: Paint) -> Arc<dyn SwitchTheme> {
    Arc::new(SwitchThemeAdapter { look: look.clone(), variant, paint, geometry: None })
}

pub fn switch_theme_for(
    look: &Look,
    variant: SwitchVariant,
    paint: Paint,
    size: SwitchSize,
    radius: Radius,
) -> Arc<dyn SwitchTheme> {
    Arc::new(SwitchThemeAdapter { look: look.clone(), variant, paint, geometry: Some((size, radius)) })
}

pub fn switch_template(
    look: &Look,
    variant: SwitchVariant,
    paint: Paint,
) -> Arc<dyn ButtonTemplate<xgpui_luma::controls::switch::SwitchData>> {
    Arc::new(ThemedSwitchTemplate::new(switch_theme_with(look, variant, paint)))
}

pub fn switch_template_for(
    look: &Look,
    variant: SwitchVariant,
    paint: Paint,
    size: SwitchSize,
    radius: Radius,
) -> Arc<dyn ButtonTemplate<xgpui_luma::controls::switch::SwitchData>> {
    Arc::new(ThemedSwitchTemplate::new(switch_theme_for(look, variant, paint, size, radius)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn palette(variant: SwitchVariant, on: bool) -> SwitchPalette {
        let look = Look::built_in();
        switch_theme_with(&look, variant, Paint::accent()).resolve(on, InteractionState::default(), ControlSize::Md)
    }

    #[test]
    fn shared_geometry_preserves_radix_sizes_and_radius_recipes() {
        let look = Look::built_in();
        for mode in [xgpui_luma::theme::ThemeMode::Light, xgpui_luma::theme::ThemeMode::Dark] {
            look.set_mode(mode);
            for (size, width, height, thumb) in [
                (SwitchSize::One, 28.0, 16.0, 14.0),
                (SwitchSize::Two, 35.0, 20.0, 18.0),
                (SwitchSize::Three, 42.0, 24.0, 22.0),
            ] {
                for radius in [Radius::None, Radius::Small, Radius::Medium, Radius::Large, Radius::Full] {
                    for variant in [SwitchVariant::Classic, SwitchVariant::Surface, SwitchVariant::Soft] {
                        let theme = switch_theme_for(&look, variant, Paint::accent(), size, radius);
                        let scale = theme.scale(size.control_size(), 2.0);
                        assert_eq!(
                            (scale.track_width, scale.track_height, scale.thumb_size, scale.track_padding, scale.gap),
                            (width, height, thumb, 1.0, 8.0)
                        );
                        assert_eq!(scale.track_radius, resolve_switch_radius(size, radius));
                        let inspect = switch_geometry(&look, size);
                        assert_eq!(inspect.track_width.value_px, scale.track_width);
                        assert_eq!(inspect.thumb_size.value_px, scale.thumb_size);
                    }
                }
            }
        }
    }

    #[test]
    fn switch_configuration_updates_live_theme_and_respects_explicit_size() {
        let look = Look::built_in();
        let theme = switch_theme_for(&look, SwitchVariant::Surface, Paint::accent(), SwitchSize::Two, Radius::Medium);
        let mut config = look.common_stylesheet();
        config.switch.sizes.get_mut("2").unwrap().track_width = Some(50.0);
        config.switch.sizes.get_mut("2").unwrap().track_height = Some(30.0);
        config.switch.sizes.get_mut("2").unwrap().thumb_size = Some(28.0);
        config.switch.geometry.gap = Some(0.0);
        look.set_common_stylesheet(config.clone());
        let scale = theme.scale(ControlSize::Lg, 1.0);
        assert_eq!(
            (scale.track_width, scale.track_height, scale.thumb_size, scale.gap, scale.track_radius),
            (50.0, 30.0, 28.0, 0.0, 15.0)
        );
        assert_eq!(switch_geometry(&look, SwitchSize::Two).track_width.value_px, scale.track_width);
        let fork = look.fork();
        fork.set_common_stylesheet(Default::default());
        assert_eq!(look.common_stylesheet(), config);
    }

    #[test]
    fn thumbs_keep_light_colors_in_dark_mode() {
        let look = Look::built_in();
        for variant in [SwitchVariant::Classic, SwitchVariant::Surface, SwitchVariant::Soft] {
            let theme = switch_theme_with(&look, variant, Paint::accent());
            for state in [
                InteractionState::default(),
                InteractionState { hovered: true, ..Default::default() },
                InteractionState { pressed: true, ..Default::default() },
                InteractionState { focused: true, ..Default::default() },
                InteractionState { disabled: true, ..Default::default() },
            ] {
                for on in [false, true] {
                    look.set_mode(xgpui_luma::theme::ThemeMode::Light);
                    let light = theme.resolve(on, state, ControlSize::Md);
                    look.set_mode(xgpui_luma::theme::ThemeMode::Dark);
                    let dark = theme.resolve(on, state, ControlSize::Md);
                    assert_eq!(light.thumb_background, dark.thumb_background);
                    assert_eq!(light.thumb_border, dark.thumb_border);
                }
            }
        }
    }

    #[test]
    fn surface_on_uses_primary_fill() {
        let look = Look::built_in();
        assert_eq!(
            palette(SwitchVariant::Surface, true).track_background,
            look.resolve_role(SemanticRole::Primary).hsla()
        );
    }

    #[test]
    fn soft_on_uses_step_four() {
        let look = Look::built_in();
        assert_eq!(
            palette(SwitchVariant::Soft, true).track_background,
            look.resolve_step(ScaleFamily::Color, 4).hsla()
        );
    }

    #[test]
    fn surface_high_contrast_uses_step_twelve() {
        let look = Look::built_in();
        let paint = Paint::accent().high_contrast();
        let on = switch_theme_with(&look, SwitchVariant::Surface, paint).resolve(
            true,
            InteractionState::default(),
            ControlSize::Md,
        );
        assert_eq!(on.track_background, look.resolve_step(ScaleFamily::Color, 12).hsla());
    }

    #[test]
    fn medium_radius_pills_the_track() {
        let size = SwitchSize::Two;
        assert_eq!(resolve_switch_radius(size, Radius::Medium), size.height() / 2.0);
        assert!(resolve_switch_radius(size, Radius::None) < 1.0);
    }

    #[test]
    fn size_two_matches_radix_height() {
        let scale = switch_scale_for(SwitchSize::Two, Radius::Medium);
        assert_eq!(scale.track_height, 20.0);
        assert_eq!(scale.track_width, 35.0);
        assert_eq!(scale.thumb_size, 18.0);
    }
}

/// Constructs the native Radix switch definition in pure Rust without preloading TOML.
pub fn native_radix_switch() -> xgpui_luma::theme::stylesheet::SwitchStylesheet {
    let mut switch = xgpui_luma::theme::stylesheet::SwitchStylesheet::default();
    switch
        .track_padding(1.0)
        .gap(8.0)
        .size("1", |m| {
            m.dimensions(28.0, 16.0, 14.0);
        })
        .size("2", |m| {
            m.dimensions(35.0, 20.0, 18.0);
        })
        .size("3", |m| {
            m.dimensions(42.0, 24.0, 22.0);
        });
    switch
}

/// Applies Radix switch geometry in Rust.
pub(crate) fn apply_radix_switch_stylesheet(sheet: &mut xgpui_luma::theme::stylesheet::CommonStylesheet) {
    native_radix_switch().apply_to(&mut sheet.switch);
}
