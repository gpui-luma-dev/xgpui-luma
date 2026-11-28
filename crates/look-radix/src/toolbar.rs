//! Radix chrome for the SDK toolbar; hosted controls keep their own Radix looks.

use std::sync::Arc;

use xgpui_luma::controls::toolbar::{ToolbarTemplate, ToolbarTheme, toolbar_template_with_theme};
use xgpui_luma::style::lss::{Overrides, PartId, Prop};

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
    /// The built-in look's toolbar shell, from its `.lss` (`style/toolbar.lss`) at size 2.
    fn default() -> Self {
        let look = Look::built_in();
        let resolved = toolbar_binding(&look).resolve(&[], 1.0);
        let root = resolved.root();
        let number = |name| root.value(name).and_then(xgpui_luma::style::lss::Value::as_number).unwrap_or(0.0);
        let control_height = look.control_sizes().for_size(look.size()).height;
        Self {
            padding_x: number("padding_x"),
            padding_y: number("padding_y"),
            gap: number("gap"),
            radius: None,
            separator_height_ratio: if control_height > 0.0 {
                number("separator_height") / control_height
            } else {
                0.0
            },
        }
    }
}

/// The `.lss` toolbar (`style/toolbar.lss`), with an instance's explicit shell geometry over it.
struct RadixToolbarTheme {
    binding: xgpui_luma::style::StyleBinding,
    look: Look,
    style: Option<ToolbarStyle>,
}

impl ToolbarTheme for RadixToolbarTheme {
    fn style(&self) -> xgpui_luma::style::StyleBinding {
        let Some(style) = self.style else {
            return self.binding.clone();
        };
        let root = PartId::ROOT;
        let mut overrides = Overrides::new()
            .set(root, Prop::<f32>::new("padding_x"), style.padding_x)
            .set(root, Prop::<f32>::new("padding_y"), style.padding_y)
            .set(root, Prop::<f32>::new("gap"), style.gap)
            .set(
                root,
                Prop::<f32>::new("separator_height"),
                self.look.control_sizes().for_size(self.look.size()).height * style.separator_height_ratio,
            );
        if let Some(radius) = style.radius {
            overrides = overrides.set(root, Prop::<f32>::new("radius"), radius);
        }
        self.binding.clone().with_overrides(&overrides)
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
    use xgpui_luma::controls::toolbar::ToolbarVariant;
    use xgpui_luma::style::PartRead;
    use xgpui_luma::theme::ThemeMode;

    #[test]
    fn custom_geometry_keeps_palette_resolution_and_defaults_independent() {
        let look = Look::built_in();
        let custom = toolbar_theme_with(&look, ToolbarStyle { gap: 12.0, padding_x: 9.0, ..Default::default() });
        let normal = toolbar_theme(&look);
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let custom = custom.resolve_style(true, ToolbarVariant::Outline);
            let normal = normal.resolve_style(true, ToolbarVariant::Outline);
            let (custom, normal) = (custom.root(), normal.root());
            assert_eq!(custom.number("gap"), 12.0);
            assert_eq!(custom.number("padding_x"), 9.0);
            assert_eq!(normal.number("gap"), 6.0);
            assert_eq!(custom.color("background"), normal.color("background"));
        }
    }

    #[test]
    fn toolbar_tracks_mode_and_preserves_ghost_transparency() {
        let look = Look::built_in();
        let theme = toolbar_theme(&look);
        let light = theme.resolve_style(true, ToolbarVariant::Outline);
        look.set_mode(ThemeMode::Dark);
        let dark = theme.resolve_style(true, ToolbarVariant::Outline);
        assert_ne!(light.root().color("background"), dark.root().color("background"));
        crate::control_group::assert_close(dark.root().color("border"), look.resolve_step(ScaleFamily::Gray, 6).hsla());
        let ghost = theme.resolve_style(true, ToolbarVariant::Ghost);
        assert_eq!(ghost.root().color("background").a, 0.0);
        assert_eq!(ghost.root().color("border").a, 0.0);
    }
}
