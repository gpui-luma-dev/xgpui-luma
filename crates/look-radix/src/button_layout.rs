//! Radix Themes button size (1–4) and radius (`none`…`full`) geometry.
//!
//! Values match `@radix-ui/themes` at `--scaling: 1`:
//! - heights from `--space-5..8`
//! - padding / gap / type from button size rules
//! - radius from `--radius-{size}` × `--radius-factor`, or pill when `full`

use gpui::FontWeight;
use xgpui_luma::controls::button_family::{ButtonFamilyLook, ButtonFamilyPalette, ButtonFamilyRole};
use xgpui_luma::theme::{ControlMetricTokens, LumaTextStyle, MetricTokens};

/// Radix `size` prop on Button. Default is [`Self::Two`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonSize {
    One,
    #[default]
    Two,
    Three,
    Four,
}

impl ButtonSize {
    pub const ALL: [Self; 4] = [Self::One, Self::Two, Self::Three, Self::Four];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::One => "1",
            Self::Two => "2",
            Self::Three => "3",
            Self::Four => "4",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::One => "Size 1",
            Self::Two => "Size 2",
            Self::Three => "Size 3",
            Self::Four => "Size 4",
        }
    }

    /// Nearest SDK [`RadixSize`] for layout-cache paths (size 4 maps to [`RadixSize::Three`]
    /// and must still override geometry via [`button_box_for`]).
    pub fn control_size(self) -> crate::RadixSize {
        match self {
            Self::One => crate::RadixSize::One,
            Self::Two => crate::RadixSize::Two,
            Self::Three | Self::Four => crate::RadixSize::Three,
        }
    }
}

/// Radix `radius` prop: `none` | `small` | `medium` | `large` | `full`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Radius {
    None,
    Small,
    #[default]
    Medium,
    Large,
    Full,
}

impl Radius {
    pub const ALL: [Self; 5] = [Self::None, Self::Small, Self::Medium, Self::Large, Self::Full];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
            Self::Full => "full",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::None => "No radius",
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
            Self::Full => "Full",
        }
    }

    /// `--radius-factor` from Radix `[data-radius=…]`.
    pub fn factor(self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Small => 0.75,
            Self::Medium => 1.0,
            Self::Large | Self::Full => 1.5,
        }
    }
}

/// Base `--radius-1..4` at factor 1 (before radius-factor).
const RADIUS_BASE: [f32; 4] = [3.0, 4.0, 6.0, 8.0];

#[derive(Clone, Copy, Debug)]
pub struct ButtonBox {
    pub height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub radius: f32,
    pub icon_size: f32,
    pub typography: LumaTextStyle,
}

/// Default look metrics; the control scale is Radix size 2. Sizes 1 and 3 live in
/// [`crate::RadixControlMetrics`].
pub fn metric_tokens() -> MetricTokens {
    let mut metrics = MetricTokens::default();
    {
        let geometry = button_box_for(ButtonSize::Two, Radius::Medium);
        metrics.control = ControlMetricTokens::new(
            geometry.height,
            geometry.padding_x,
            geometry.padding_y,
            geometry.gap,
            geometry.radius,
            geometry.icon_size,
        );
    }
    metrics
}

/// Radix chrome corner radii in pixels (Radix `--radius-1..4`, plus full).
///
/// The SDK has no radius vocabulary; Radix names radii with [`Radius`] and these
/// steps, and hands SDK controls the resulting pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RadixRadii {
    pub none: f32,
    /// `--radius-1`.
    pub small: f32,
    /// `--radius-2`.
    pub medium: f32,
    /// `--radius-3`.
    pub large: f32,
    /// `--radius-4`, used by window-scale chrome (overlay windows).
    pub xl: f32,
    /// Fully rounded (pill / circle).
    pub full: f32,
}

impl RadixRadii {
    pub const DEFAULT: Self =
        Self { none: 0.0, small: 3.0, medium: 4.0, large: 6.0, xl: 8.0, full: xgpui_luma::theme::FULL_RADIUS_PX };

    /// Pixels for a named [`Radius`].
    pub fn get(&self, radius: Radius) -> f32 {
        match radius {
            Radius::None => self.none,
            Radius::Small => self.small,
            Radius::Medium => self.medium,
            Radius::Large => self.large,
            Radius::Full => self.full,
        }
    }
}

impl Default for RadixRadii {
    fn default() -> Self {
        Self::DEFAULT
    }
}

pub fn button_box_for(size: ButtonSize, radius: Radius) -> ButtonBox {
    button_box_with_stylesheet(crate::look::embedded_common_stylesheet(), size, radius)
}

pub(crate) fn button_box_with_stylesheet(
    stylesheet: &xgpui_luma::theme::stylesheet::CommonStylesheet,
    size: ButtonSize,
    radius: Radius,
) -> ButtonBox {
    let control = crate::size::scaffold_metrics(size.control_size()).control;
    let typography = xgpui_luma::theme::ThemeTokens::default().typography.text.label;
    let geometry = stylesheet.button.resolve_geometry_for(
        size.as_str(),
        xgpui_luma::theme::stylesheet::ButtonGeometry {
            height: control.height,
            padding_x: control.padding_x,
            padding_y: control.padding_y,
            gap: control.gap,
            icon_size: control.icon_size,
            font_size: typography.size,
            line_height: typography.line_height,
        },
    );

    ButtonBox {
        height: geometry.height,
        padding_x: geometry.padding_x,
        padding_y: geometry.padding_y,
        gap: geometry.gap,
        radius: if radius == Radius::Full {
            geometry.height / 2.0
        } else {
            resolve_button_radius(size, radius)
        },
        icon_size: geometry.icon_size,
        typography: LumaTextStyle {
            size: geometry.font_size,
            line_height: geometry.line_height,
            weight: FontWeight::MEDIUM,
        },
    }
}

/// `border-radius: max(var(--radius-N) * factor, var(--radius-full))`.
pub fn resolve_button_radius(size: ButtonSize, radius: Radius) -> f32 {
    if matches!(radius, Radius::Full) {
        return button_box_height(size) / 2.0;
    }
    let index = match size {
        ButtonSize::One => 0,
        ButtonSize::Two => 1,
        ButtonSize::Three => 2,
        ButtonSize::Four => 3,
    };
    RADIUS_BASE[index] * radius.factor()
}

fn button_box_height(size: ButtonSize) -> f32 {
    crate::look::embedded_common_stylesheet()
        .button
        .resolve_geometry_for(size.as_str(), Default::default())
        .height
}

/// Applies Radix size/radius geometry onto a resolved color palette.
pub fn apply_button_box(palette: &ButtonFamilyPalette, role: ButtonFamilyRole, box_: ButtonBox) -> ButtonFamilyLook {
    ButtonFamilyLook {
        focus_border: palette.focus_border,
        background: palette.background,
        foreground: palette.foreground,
        muted_foreground: palette.muted_foreground,
        border: palette.border,
        typography: box_.typography,
        font_family: palette.font_family.clone(),
        radius: box_.radius,
        padding_x: match role {
            ButtonFamilyRole::Icon => 0.0,
            _ => box_.padding_x,
        },
        padding_y: match role {
            ButtonFamilyRole::Icon => 0.0,
            _ => box_.padding_y,
        },
        gap: box_.gap,
        height: box_.height,
        icon_size: match role {
            ButtonFamilyRole::Icon => box_.icon_size,
            _ => box_.typography.size,
        },
        shadow: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_two_medium_matches_radix_defaults() {
        let box_ = button_box_for(ButtonSize::Two, Radius::Medium);
        assert_eq!(box_.height, 32.0);
        assert_eq!(box_.padding_x, 12.0);
        assert_eq!(box_.gap, 8.0);
        assert_eq!(box_.radius, 4.0);
        assert_eq!(box_.typography.size, 14.0);
    }

    #[test]
    fn radius_scales_with_size_and_factor() {
        assert_eq!(resolve_button_radius(ButtonSize::One, Radius::None), 0.0);
        assert_eq!(resolve_button_radius(ButtonSize::One, Radius::Small), 3.0 * 0.75);
        assert_eq!(resolve_button_radius(ButtonSize::Four, Radius::Medium), 8.0);
        assert_eq!(resolve_button_radius(ButtonSize::Three, Radius::Large), 6.0 * 1.5);
        assert_eq!(resolve_button_radius(ButtonSize::Two, Radius::Full), 16.0);
    }

    #[test]
    fn control_metrics_map_to_sizes_1_2_3() {
        let metrics = metric_tokens();
        let controls = crate::RadixControlMetrics::from_button_boxes(metrics.control);
        assert_eq!(controls.one.height, 24.0);
        assert_eq!(metrics.control.height, 32.0);
        assert_eq!(controls.three.height, 40.0);
        assert_eq!(metrics.control.radius, 4.0);
    }
}
