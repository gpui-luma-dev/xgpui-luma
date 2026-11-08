//! Radix chrome for the SDK toolbar; hosted controls keep their own Radix looks.

use std::sync::Arc;

use xgpui_luma::controls::toolbar::{
    ToolbarLook, ToolbarTemplate, ToolbarTheme, ToolbarVariant, toolbar_template_with_theme,
};
use xgpui_luma::theme::ControlSize;

use crate::{Look, ScaleFamily};

/// Toolbar shell geometry in logical pixels. Colors still follow the bound look.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToolbarStyle {
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    /// None uses the look's medium radius.
    pub radius: Option<f32>,
    /// Separator height relative to the toolbar's selected control size.
    pub separator_height_ratio: f32,
}
impl Default for ToolbarStyle {
    fn default() -> Self {
        Self { padding_x: 4.0, padding_y: 3.0, gap: 6.0, radius: None, separator_height_ratio: 0.65 }
    }
}

struct RadixToolbarTheme {
    look: Look,
    style: Option<ToolbarStyle>,
}

impl ToolbarTheme for RadixToolbarTheme {
    fn resolve(&self, enabled: bool, size: ControlSize, variant: ToolbarVariant) -> ToolbarLook {
        let gray = |step| self.look.resolve_step(ScaleFamily::Gray, step).hsla();
        let transparent = gpui::hsla(0.0, 0.0, 0.0, 0.0);
        let metrics = self.look.metrics();
        let control = metrics.for_size(size);
        let (background, border) = match variant {
            ToolbarVariant::Outline => (gray(if enabled { 1 } else { 2 }), gray(6)),
            ToolbarVariant::Ghost => (transparent, transparent),
        };
        let fallback = ToolbarStyle::default();
        let geometry = self.look.common_stylesheet().toolbar.resolve_geometry(
            "",
            xgpui_luma::theme::stylesheet::ToolbarGeometry {
                padding_x: fallback.padding_x,
                padding_y: fallback.padding_y,
                gap: fallback.gap,
                border_width: metrics.border_width.default,
                separator_width: metrics.border_width.default,
            },
        );
        let style = self.style.unwrap_or(ToolbarStyle {
            padding_x: geometry.padding_x,
            padding_y: geometry.padding_y,
            gap: geometry.gap,
            ..fallback
        });
        ToolbarLook {
            border_width: geometry.border_width,
            separator_width: geometry.separator_width,
            background,
            border,
            separator: gray(6),
            radius: style.radius.unwrap_or(metrics.radius.md),
            padding_x: style.padding_x,
            padding_y: style.padding_y,
            gap: style.gap,
            separator_height: control.height * style.separator_height_ratio,
        }
    }
}

/// Toolbar shell colors resolve from the current mode and palette on render.
/// Notify affected views after mutating the look; see [`Look`].
pub fn toolbar_theme(look: &Look) -> Arc<dyn ToolbarTheme> {
    Arc::new(RadixToolbarTheme { look: look.clone(), style: None })
}

/// Override shell geometry without replacing palette resolution or SDK behavior.
pub fn toolbar_theme_with(look: &Look, style: ToolbarStyle) -> Arc<dyn ToolbarTheme> {
    Arc::new(RadixToolbarTheme { look: look.clone(), style: Some(style) })
}

pub fn toolbar_template_with(look: &Look, style: ToolbarStyle) -> Arc<dyn ToolbarTemplate> {
    toolbar_template_with_theme(toolbar_theme_with(look, style))
}

/// Theme the SDK toolbar while retaining its hosted controls and keyboard navigation.
pub fn toolbar_template(look: &Look) -> Arc<dyn ToolbarTemplate> {
    toolbar_template_with_theme(toolbar_theme(look))
}

#[cfg(test)]
mod tests {
    use super::*;
    use xgpui_luma::theme::ThemeMode;

    #[test]
    fn custom_geometry_keeps_palette_resolution_and_defaults_independent() {
        let look = Look::built_in();
        let custom = toolbar_theme_with(&look, ToolbarStyle { gap: 12.0, padding_x: 9.0, ..Default::default() });
        let normal = toolbar_theme(&look);
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let custom = custom.resolve(true, ControlSize::Sm, ToolbarVariant::Outline);
            let normal = normal.resolve(true, ControlSize::Sm, ToolbarVariant::Outline);
            assert_eq!(custom.gap, 12.0);
            assert_eq!(custom.padding_x, 9.0);
            assert_eq!(normal.gap, 6.0);
            assert_eq!(custom.background, normal.background);
        }
    }

    #[test]
    fn toolbar_tracks_mode_and_preserves_ghost_transparency() {
        let look = Look::built_in();
        let theme = toolbar_theme(&look);
        let light = theme.resolve(true, ControlSize::Sm, ToolbarVariant::Outline);
        look.set_mode(ThemeMode::Dark);
        let dark = theme.resolve(true, ControlSize::Sm, ToolbarVariant::Outline);
        assert_ne!(light.background, dark.background);
        assert_eq!(dark.border, look.resolve_step(ScaleFamily::Gray, 6).hsla());
        let ghost = theme.resolve(true, ControlSize::Sm, ToolbarVariant::Ghost);
        assert_eq!(ghost.background.a, 0.0);
        assert_eq!(ghost.border.a, 0.0);
    }
}

/// Native common geometry owned by this control.
pub(crate) fn native_toolbar_stylesheet() -> xgpui_luma::theme::stylesheet::ToolbarStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    ToolbarStylesheet {
        geometry: ToolbarGeometryStylesheet {
            padding_x: Some(4.0),
            padding_y: Some(3.0),
            gap: Some(6.0),
            ..Default::default()
        },
        ..Default::default()
    }
}
