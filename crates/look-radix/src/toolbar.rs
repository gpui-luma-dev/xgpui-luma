//! Radix chrome for the SDK toolbar; hosted controls keep their own Radix looks.

use std::sync::Arc;

use xgpui_luma::controls::toolbar::{
    ToolbarLook, ToolbarTemplate, ToolbarTheme, ToolbarVariant, toolbar_template_with_theme,
};

use crate::Look;

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

/// The `.lss` toolbar (`style/toolbar.lss`), with an instance's explicit shell geometry over it.
struct RadixToolbarTheme {
    binding: xgpui_luma::style::StyleBinding,
    look: Look,
    style: Option<ToolbarStyle>,
}

impl ToolbarTheme for RadixToolbarTheme {
    fn resolve(&self, enabled: bool, variant: ToolbarVariant) -> ToolbarLook {
        let mut resolved = xgpui_luma::controls::toolbar::styled_toolbar_look(&self.binding, enabled, variant);
        if let Some(style) = self.style {
            resolved.padding_x = style.padding_x;
            resolved.padding_y = style.padding_y;
            resolved.gap = style.gap;
            if let Some(radius) = style.radius {
                resolved.radius = radius;
            }
            resolved.separator_height =
                self.look.control_sizes().for_size(self.look.size()).height * style.separator_height_ratio;
        }
        resolved
    }
}

fn toolbar_binding(look: &Look) -> xgpui_luma::style::StyleBinding {
    let binding = xgpui_luma::style::StyleBinding::new("toolbar", crate::textfield::live_source(look));
    let size = look.size();
    if size != crate::RadixSize::Two {
        binding.attr("size", size.key())
    } else {
        binding
    }
}

/// Toolbar shell colors resolve from the current mode and palette on render.
/// Notify affected views after mutating the look; see [`Look`].
pub fn toolbar_theme(look: &Look) -> Arc<dyn ToolbarTheme> {
    Arc::new(RadixToolbarTheme { binding: toolbar_binding(look), look: look.clone(), style: None })
}

/// Override shell geometry without replacing palette resolution or SDK behavior.
pub fn toolbar_theme_with(look: &Look, style: ToolbarStyle) -> Arc<dyn ToolbarTheme> {
    Arc::new(RadixToolbarTheme { binding: toolbar_binding(look), look: look.clone(), style: Some(style) })
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
    use crate::ScaleFamily;
    use xgpui_luma::theme::ThemeMode;

    #[test]
    fn custom_geometry_keeps_palette_resolution_and_defaults_independent() {
        let look = Look::built_in();
        let custom = toolbar_theme_with(&look, ToolbarStyle { gap: 12.0, padding_x: 9.0, ..Default::default() });
        let normal = toolbar_theme(&look);
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let custom = custom.resolve(true, ToolbarVariant::Outline);
            let normal = normal.resolve(true, ToolbarVariant::Outline);
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
        let light = theme.resolve(true, ToolbarVariant::Outline);
        look.set_mode(ThemeMode::Dark);
        let dark = theme.resolve(true, ToolbarVariant::Outline);
        assert_ne!(light.background, dark.background);
        crate::control_group::assert_close(dark.border, look.resolve_step(ScaleFamily::Gray, 6).hsla());
        let ghost = theme.resolve(true, ToolbarVariant::Ghost);
        assert_eq!(ghost.background.a, 0.0);
        assert_eq!(ghost.border.a, 0.0);
    }
}
