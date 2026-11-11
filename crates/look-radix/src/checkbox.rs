//! Radix checkbox theme adapter.
//!
//! Geometry matches `@radix-ui/themes` BaseCheckbox at `--scaling: 1`:
//! - sizes 1–3 → box 14 / 16 / 20 (`space-4` × 0.875 / 1 / 1.25)
//! - radius from `--radius-1` × size mul × theme radius factor (never pill)

use std::sync::Arc;

use gpui::{BoxShadow, Hsla, point, px};
use xgpui_luma::controls::button::ButtonTemplate;
use xgpui_luma::controls::checkbox::{CheckboxPalette, CheckboxScale, CheckboxTheme, ThemedCheckboxTemplate};
use xgpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, MetricTokens};

use crate::button::Paint;
use crate::button_layout::Radius;
use crate::look::Look;
use crate::scale::ScaleFamily;
use crate::semantic::SemanticRole;
use crate::tone::Tone;
use crate::typography::{font_family, label_typography};

/// The Radix Themes checkbox variant scheme.
///
/// `Surface` is Radix's default: a panel face behind a gray edge that fills with the
/// accent when checked. `Classic` adds the raised inset treatment. `Soft` drops the
/// edge entirely and keeps an accent tint in both check states.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CheckboxVariant {
    Classic,
    #[default]
    Surface,
    Soft,
}

/// Radix `size` prop on Checkbox. Default is [`Self::Two`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CheckboxSize {
    One,
    #[default]
    Two,
    Three,
}

impl CheckboxSize {
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

    /// Outer indicator box edge in px (`--checkbox-size`).
    pub fn box_size(self) -> f32 {
        crate::look::embedded_common_stylesheet()
            .checkbox
            .resolve_geometry(self.as_str(), Default::default())
            .indicator_size
    }

    /// Checkmark glyph box (`--checkbox-indicator-size`).
    pub fn glyph_size(self) -> f32 {
        crate::look::embedded_common_stylesheet()
            .checkbox
            .resolve_geometry(self.as_str(), Default::default())
            .glyph_size
    }

    /// Multiplier applied to `--radius-1` for this size.
    fn radius_mul(self) -> f32 {
        match self {
            Self::One => 0.875,
            Self::Two => 1.0,
            Self::Three => 1.25,
        }
    }
}

/// `--radius-1` at factor 1 before size / theme multipliers.
const RADIUS_1: f32 = 3.0;

/// Theme radius applied to checkbox geometry. `Full` shares Large's factor — Radix never
/// pills checkboxes (avoids looking like a radio).
pub fn resolve_checkbox_radius(size: CheckboxSize, radius: Radius) -> f32 {
    let factor = match radius {
        Radius::Full => Radius::Large.factor(),
        other => other.factor(),
    };
    RADIUS_1 * size.radius_mul() * factor
}

pub fn checkbox_scale_for(size: CheckboxSize, radius: Radius) -> CheckboxScale {
    checkbox_scale_with_stylesheet(crate::look::embedded_common_stylesheet(), size, radius)
}

fn checkbox_scale_with_stylesheet(
    stylesheet: &xgpui_luma::theme::stylesheet::CommonStylesheet,
    size: CheckboxSize,
    radius: Radius,
) -> CheckboxScale {
    let fallback = CheckboxScale::compute(size.control_size(), &MetricTokens::default(), 1.0);
    let geometry = stylesheet.checkbox.resolve_geometry(
        size.as_str(),
        xgpui_luma::theme::stylesheet::CheckboxGeometry {
            indicator_size: fallback.indicator_size,
            glyph_size: fallback.glyph_size,
            gap: fallback.gap,
            indicator_radius: resolve_checkbox_radius(size, radius),
            ..Default::default()
        },
    );
    let box_size = geometry.indicator_size;
    let glyph = geometry.glyph_size;
    let indicator_radius = geometry.indicator_radius;
    CheckboxScale {
        disabled_opacity: geometry.disabled_opacity,
        control_radius: indicator_radius,
        control_padding_x: geometry.control_padding_x,
        control_padding_y: geometry.control_padding_y,
        indicator_size: box_size,
        indicator_radius,
        height: box_size,
        gap: geometry.gap,
        label_baseline_shift: 0.0,
        glyph_size: glyph,
    }
}

struct CheckboxThemeAdapter {
    look: Look,
    variant: CheckboxVariant,
    paint: Paint,
    geometry: Option<(CheckboxSize, Radius)>,
}

/// Radix `--shadow-1`: a hairline rim with a soft top fade, so the resting face reads
/// slightly recessed before it is checked.
fn resting_shadow(edge: Hsla, fade: Hsla) -> Vec<BoxShadow> {
    vec![inset(0.0, 0.0, 1.0, edge), inset(1.5, 2.0, 0.0, fade)]
}

/// The checked Classic face: a half-pixel light rim on top and a dark one underneath,
/// standing in for Radix's `white-a3 -> transparent -> black-a1` gradient, which a flat
/// indicator fill cannot carry.
fn checked_shadow(highlight: Hsla, shade: Hsla) -> Vec<BoxShadow> {
    vec![inset(0.5, 0.5, 0.0, highlight), inset(-0.5, 0.5, 0.0, shade)]
}

fn inset(offset_y: f32, blur: f32, spread: f32, color: Hsla) -> BoxShadow {
    BoxShadow {
        offset: point(px(0.0), px(offset_y)),
        blur_radius: px(blur),
        spread_radius: px(spread),
        color,
        inset: true,
    }
}

fn alpha(color: Hsla, a: f32) -> Hsla {
    Hsla { a, ..color }
}

impl CheckboxTheme for CheckboxThemeAdapter {
    fn resolve(&self, checked: bool, state: InteractionState, size: ControlSize) -> CheckboxPalette {
        let look = &self.look;
        let layer = state.layer();
        let high_contrast = self.paint.high_contrast;
        let family = match self.paint.tone {
            Tone::Accent => ScaleFamily::Color,
            Tone::Gray => ScaleFamily::Gray,
        };
        let step = |n| look.resolve_step(family, n).hsla();
        let gray = |n| look.resolve_step(ScaleFamily::Gray, n).hsla();
        let surface = look.resolve_role(SemanticRole::Surface).hsla();
        let contrast = if high_contrast {
            // HC fill is step-12; checkmark uses the light end of the same scale.
            step(1)
        } else {
            look.resolve_role(SemanticRole::PrimaryForeground).hsla()
        };

        // Press uses the resting/focused fill; only hover changes the checked fill.
        let filled = if high_contrast {
            step(12)
        } else {
            match layer {
                InteractionLayer::Hovered => step(10),
                _ => look.resolve_role(SemanticRole::Primary).hsla(),
            }
        };

        let soft_check = if high_contrast { step(12) } else { step(11) };

        let (indicator_background, indicator_border, checkmark_color, indicator_shadow) =
            match (layer, self.variant, checked) {
                (InteractionLayer::Disabled, CheckboxVariant::Classic, _) => {
                    (alpha(surface, 0.0), gray(6), gray(8), Some(resting_shadow(gray(5), alpha(gray(3), 0.4))))
                }
                (InteractionLayer::Disabled, CheckboxVariant::Soft, _) => {
                    (alpha(surface, 0.0), alpha(surface, 0.0), gray(8), None)
                }
                (InteractionLayer::Disabled, CheckboxVariant::Surface, _) => {
                    (alpha(surface, 0.0), gray(6), gray(8), None)
                }
                // Soft keeps the same accent tint whether or not it is checked.
                (_, CheckboxVariant::Soft, _) => (step(5), step(5), soft_check, None),
                (_, CheckboxVariant::Surface, false) => (surface, gray(7), gray(11), None),
                (_, CheckboxVariant::Surface, true) => (filled, filled, contrast, None),
                (_, CheckboxVariant::Classic, false) => {
                    (surface, gray(3), gray(11), Some(resting_shadow(gray(5), alpha(gray(2), 0.5))))
                }
                (_, CheckboxVariant::Classic, true) => {
                    (filled, filled, contrast, Some(checked_shadow(alpha(contrast, 0.35), alpha(gpui::black(), 0.28))))
                }
            };

        let label_color = if state.disabled {
            look.resolve_role(SemanticRole::MutedForeground).hsla()
        } else {
            look.resolve_role(SemanticRole::Foreground).hsla()
        };

        CheckboxPalette {
            focus_border: look.resolve_role(SemanticRole::Focus).hsla(),
            control_background: None,
            control_border: None,
            indicator_background,
            indicator_border,
            checkmark_color,
            label_color,
            label_typography: label_typography(size),
            label_font_family: font_family(look),
            indicator_shadow,
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }

    fn scale(&self, size: ControlSize, scale_factor: f32) -> CheckboxScale {
        if let Some((radix_size, radius)) = self.geometry {
            return checkbox_scale_with_stylesheet(&self.look.stylesheet_snapshot(), radix_size, radius);
        }
        let radix_size = match size {
            ControlSize::Sm => CheckboxSize::One,
            ControlSize::Md => CheckboxSize::Two,
            ControlSize::Lg => CheckboxSize::Three,
        };
        let _ = scale_factor;
        checkbox_scale_with_stylesheet(&self.look.stylesheet_snapshot(), radix_size, Radius::Medium)
    }
}

pub fn checkbox_theme(look: &Look, variant: CheckboxVariant) -> Arc<dyn CheckboxTheme> {
    checkbox_theme_with(look, variant, Paint::accent())
}

pub fn checkbox_theme_with(look: &Look, variant: CheckboxVariant, paint: Paint) -> Arc<dyn CheckboxTheme> {
    Arc::new(CheckboxThemeAdapter { look: look.clone(), variant, paint, geometry: None })
}

/// Theme for a size × radius style-guide cell.
pub fn checkbox_theme_for(
    look: &Look,
    variant: CheckboxVariant,
    paint: Paint,
    size: CheckboxSize,
    radius: Radius,
) -> Arc<dyn CheckboxTheme> {
    Arc::new(CheckboxThemeAdapter { look: look.clone(), variant, paint, geometry: Some((size, radius)) })
}

pub fn checkbox_template(
    look: &Look,
    variant: CheckboxVariant,
    paint: Paint,
) -> Arc<dyn ButtonTemplate<xgpui_luma::controls::checkbox::CheckboxData>> {
    Arc::new(ThemedCheckboxTemplate::new(checkbox_theme_with(look, variant, paint)))
}

pub fn checkbox_template_for(
    look: &Look,
    variant: CheckboxVariant,
    paint: Paint,
    size: CheckboxSize,
    radius: Radius,
) -> Arc<dyn ButtonTemplate<xgpui_luma::controls::checkbox::CheckboxData>> {
    Arc::new(ThemedCheckboxTemplate::new(checkbox_theme_for(look, variant, paint, size, radius)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn palette(variant: CheckboxVariant, checked: bool) -> CheckboxPalette {
        let look = Look::built_in();
        checkbox_theme(&look, variant).resolve(checked, InteractionState::default(), ControlSize::Md)
    }

    #[test]
    fn pressed_face_preserves_focused_fill_for_every_variant() {
        let look = Look::built_in();
        for mode in [xgpui_luma::theme::ThemeMode::Light, xgpui_luma::theme::ThemeMode::Dark] {
            look.set_mode(mode);
            for variant in [CheckboxVariant::Classic, CheckboxVariant::Surface, CheckboxVariant::Soft] {
                for checked in [false, true] {
                    for paint in [Paint::accent(), Paint::gray(), Paint::accent().high_contrast()] {
                        let theme = checkbox_theme_with(&look, variant, paint);
                        let focused = theme.resolve(
                            checked,
                            InteractionState { focused: true, ..Default::default() },
                            ControlSize::Md,
                        );
                        for hovered in [false, true] {
                            let pressed = theme.resolve(
                                checked,
                                InteractionState { focused: true, pressed: true, hovered, ..Default::default() },
                                ControlSize::Md,
                            );
                            assert_eq!(pressed.indicator_background, focused.indicator_background);
                            assert_eq!(pressed.checkmark_color, focused.checkmark_color);
                            assert_eq!(pressed.indicator_border, focused.indicator_border);
                        }
                        let disabled = theme.resolve(
                            checked,
                            InteractionState { disabled: true, ..Default::default() },
                            ControlSize::Md,
                        );
                        let disabled_press = theme.resolve(
                            checked,
                            InteractionState { disabled: true, pressed: true, ..Default::default() },
                            ControlSize::Md,
                        );
                        assert_eq!(disabled.indicator_background, disabled_press.indicator_background);
                        assert_eq!(disabled.indicator_border, disabled_press.indicator_border);
                    }
                }
            }
        }
    }

    #[test]
    fn surface_fills_with_the_accent_behind_a_gray_edge() {
        let look = Look::built_in();
        let unchecked = palette(CheckboxVariant::Surface, false);
        let checked = palette(CheckboxVariant::Surface, true);

        assert_eq!(unchecked.indicator_border, look.resolve_step(ScaleFamily::Gray, 7).hsla());
        assert_eq!(checked.indicator_background, look.resolve_role(SemanticRole::Primary).hsla());
        assert!(unchecked.indicator_shadow.is_none(), "surface stays flat");
    }

    #[test]
    fn soft_keeps_one_accent_tint_in_both_check_states() {
        let tint = Look::built_in().resolve_step(ScaleFamily::Color, 5).hsla();

        for checked in [false, true] {
            assert_eq!(palette(CheckboxVariant::Soft, checked).indicator_background, tint);
        }
        assert_eq!(
            palette(CheckboxVariant::Soft, true).checkmark_color,
            Look::built_in().resolve_step(ScaleFamily::Color, 11).hsla()
        );
    }

    #[test]
    fn classic_is_the_only_variant_that_carries_a_shadow() {
        for checked in [false, true] {
            assert!(palette(CheckboxVariant::Classic, checked).indicator_shadow.is_some());
            assert!(palette(CheckboxVariant::Surface, checked).indicator_shadow.is_none());
            assert!(palette(CheckboxVariant::Soft, checked).indicator_shadow.is_none());
        }
    }

    #[test]
    fn surface_high_contrast_uses_step_twelve_fill() {
        let look = Look::built_in();
        let paint = Paint::accent().high_contrast();
        let checked = checkbox_theme_with(&look, CheckboxVariant::Surface, paint).resolve(
            true,
            InteractionState::default(),
            ControlSize::Md,
        );
        assert_eq!(checked.indicator_background, look.resolve_step(ScaleFamily::Color, 12).hsla());
        assert_eq!(checked.checkmark_color, look.resolve_step(ScaleFamily::Color, 1).hsla());
    }

    #[test]
    fn soft_high_contrast_darkens_the_checkmark_only() {
        let look = Look::built_in();
        let paint = Paint::accent().high_contrast();
        let checked = checkbox_theme_with(&look, CheckboxVariant::Soft, paint).resolve(
            true,
            InteractionState::default(),
            ControlSize::Md,
        );
        assert_eq!(checked.indicator_background, look.resolve_step(ScaleFamily::Color, 5).hsla());
        assert_eq!(checked.checkmark_color, look.resolve_step(ScaleFamily::Color, 12).hsla());
    }

    #[test]
    fn checkbox_radius_never_pills_on_full() {
        let size = CheckboxSize::Two;
        assert_eq!(resolve_checkbox_radius(size, Radius::Full), resolve_checkbox_radius(size, Radius::Large));
        assert!(resolve_checkbox_radius(size, Radius::Full) < size.box_size() / 2.0);
    }

    #[test]
    fn size_two_medium_matches_radius_one() {
        assert_eq!(resolve_checkbox_radius(CheckboxSize::Two, Radius::Medium), 3.0);
        assert_eq!(checkbox_scale_for(CheckboxSize::Two, Radius::Medium).indicator_size, 16.0);
    }
}

/// Constructs the native Radix checkbox definition in pure Rust without preloading TOML.
pub fn native_radix_checkbox() -> xgpui_luma::theme::stylesheet::CheckboxStylesheet {
    let mut checkbox = xgpui_luma::theme::stylesheet::CheckboxStylesheet::default();
    checkbox
        .gap(8.0)
        .size("1", |m| {
            m.indicator_size(14.0).glyph_size(9.0);
        })
        .size("2", |m| {
            m.indicator_size(16.0).glyph_size(10.0);
        })
        .size("3", |m| {
            m.indicator_size(20.0).glyph_size(12.0);
        });
    checkbox
}

/// Applies Radix checkbox geometry in Rust.
pub(crate) fn apply_radix_checkbox_stylesheet(sheet: &mut xgpui_luma::theme::stylesheet::CommonStylesheet) {
    native_radix_checkbox().apply_to(&mut sheet.checkbox);
}
