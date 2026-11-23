use gpui::{BoxShadow, Hsla, SharedString};

use crate::theme::{LumaTextStyle, StandardBoxScale};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonFamilyRole {
    #[default]
    Text,
    Icon,
    Toggle {
        selected: bool,
    },
}

#[derive(Clone, Debug)]
pub struct ButtonFamilyPalette {
    /// Independent focus-adorner paint. Transparent paint retains the configured geometry.
    pub focus_border: gpui::Hsla,
    pub background: Hsla,
    pub foreground: Hsla,
    pub muted_foreground: Hsla,
    /// Explicit border color from the theme. `None` means borderless.
    pub border: Option<Hsla>,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
}

#[derive(Clone, Debug)]
pub struct ButtonFamilyLook {
    /// Independent focus-adorner paint. Transparent paint retains the configured geometry.
    pub focus_border: gpui::Hsla,
    pub background: Hsla,
    pub foreground: Hsla,
    pub muted_foreground: Hsla,
    /// Explicit border color from the theme. `None` means borderless.
    pub border: Option<Hsla>,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
    pub radius: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub height: f32,
    pub icon_size: f32,
    pub shadow: Option<Vec<BoxShadow>>,
}

/// Normalizes absent face-border paint to transparent.
///
/// Face-border paint never determines focus geometry or focus color.
pub fn button_family_effective_border(border: Option<Hsla>) -> Hsla {
    border.unwrap_or_else(|| gpui::hsla(0.0, 0.0, 0.0, 0.0))
}

pub fn compose_button_family_look(
    palette: &ButtonFamilyPalette,
    role: ButtonFamilyRole,
    scale: &StandardBoxScale,
    pill_radius: f32,
) -> ButtonFamilyLook {
    ButtonFamilyLook {
        focus_border: palette.focus_border,
        background: palette.background,
        foreground: palette.foreground,
        muted_foreground: palette.muted_foreground,
        border: palette.border,
        typography: palette.typography,
        font_family: palette.font_family.clone(),
        radius: match role {
            ButtonFamilyRole::Icon => pill_radius,
            _ => scale.radius,
        },
        padding_x: match role {
            ButtonFamilyRole::Icon => 0.0,
            _ => scale.padding_x,
        },
        padding_y: match role {
            ButtonFamilyRole::Icon => 0.0,
            _ => scale.padding_y,
        },
        gap: scale.gap,
        height: scale.height,
        icon_size: default_button_icon_size(role, scale, &palette.typography),
        shadow: None,
    }
}

fn default_button_icon_size(role: ButtonFamilyRole, scale: &StandardBoxScale, typography: &LumaTextStyle) -> f32 {
    if matches!(role, ButtonFamilyRole::Icon) {
        scale.icon_size
    } else {
        typography.size
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn effective_border_is_transparent_when_absent() {
        let effective = button_family_effective_border(None);
        assert_eq!(effective.a, 0.0);
    }
}

crate::theme::stylesheet::geometry::geometry_family!(
    ButtonStylesheet,
    ButtonGeometryStylesheet,
    ButtonGeometry,
    ResolvedButtonGeometry,
    "button",
    [height, padding_x, padding_y, gap, icon_size, font_size, line_height]
);
