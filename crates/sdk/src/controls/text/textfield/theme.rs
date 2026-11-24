use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla, SharedString};

use crate::theme::{LumaTextStyle, MetricTokens, StandardBoxScale};
use crate::controls::textfield::TextFieldState;

#[derive(Clone, Debug)]
pub struct TextFieldPalette {
    /// Independent focus-adorner paint. Transparent paint retains the configured geometry.
    pub focus_border: gpui::Hsla,
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub placeholder: Hsla,
    pub icon: Hsla,
    pub selection_background: Hsla,
    pub selection_foreground: Hsla,
    pub caret: Hsla,
    pub shadow: Option<Vec<BoxShadow>>,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
}

#[derive(Clone, Debug)]
pub struct TextFieldLook {
    /// Helper chrome resolved through the selected common stylesheet.
    pub helper_geometry: crate::theme::stylesheet::SharedGeometry,
    /// Painted caret width and extra height in pixels.
    pub caret_width: f32,
    pub caret_height_extra: f32,
    /// Whole-input opacity when disabled.
    pub disabled_opacity: f32,
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    /// Independent focus paint; transparency does not remove focus space.
    pub focus_border: Hsla,
    pub placeholder: Hsla,
    pub icon: Hsla,
    pub selection_background: Hsla,
    pub selection_foreground: Hsla,
    pub caret: Hsla,
    pub shadow: Option<Vec<BoxShadow>>,
    pub typography: LumaTextStyle,
    pub font_family: SharedString,
    pub min_height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub radius: f32,
    pub border_width: f32,
    pub icon_size: f32,
}

pub trait TextFieldTheme: Send + Sync {
    fn resolve(&self, state: TextFieldState, enabled: bool) -> TextFieldPalette;

    fn metrics(&self) -> MetricTokens;

    fn resolve_look(&self, state: TextFieldState, enabled: bool, scale: &StandardBoxScale) -> TextFieldLook {
        compose_textfield_look(&self.resolve(state, enabled), scale, self.metrics().border_width)
    }
}

/// The SDK's text-field theme: `textfield.lss` alone.
pub fn default_textfield_theme() -> Arc<dyn TextFieldTheme> {
    static THEME: OnceLock<Arc<dyn TextFieldTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(crate::controls::text::StyledTextFieldTheme::sdk())).clone()
}

/// Scales text-field typography from `LumaTypography` size roles for lookless fallbacks.
pub fn apply_control_typography(typography: &mut LumaTextStyle, tokens: &crate::theme::LumaTypography) {
    let font_size = tokens.text.body.size;
    let base_size = typography.size;
    typography.size = font_size;
    if base_size > 0.0 {
        typography.line_height = font_size * (typography.line_height / base_size);
    }
}

pub fn compose_textfield_look(
    palette: &TextFieldPalette,
    scale: &StandardBoxScale,
    border_width: f32,
) -> TextFieldLook {
    TextFieldLook {
        helper_geometry: crate::theme::stylesheet::SharedGeometry::sdk(),
        caret_width: crate::style::sdk_number("textfield", "caret_width"),
        caret_height_extra: crate::style::sdk_number("textfield", "caret_height_extra"),
        disabled_opacity: crate::style::sdk_number("textfield", "disabled_opacity"),
        background: palette.background,
        foreground: palette.foreground,
        border: palette.border,
        focus_border: palette.focus_border,
        placeholder: palette.placeholder,
        icon: palette.icon,
        selection_background: palette.selection_background,
        selection_foreground: palette.selection_foreground,
        caret: palette.caret,
        shadow: palette.shadow.clone(),
        typography: palette.typography,
        font_family: palette.font_family.clone(),
        min_height: scale.height,
        padding_x: scale.padding_x,
        padding_y: scale.padding_y,
        gap: scale.gap,
        radius: scale.radius,
        border_width,
        icon_size: scale.icon_size,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_uses_body_typography_and_control_icon_size() {
        let tokens = crate::theme::ThemeTokens::default();
        let look = default_textfield_theme().resolve_look(
            TextFieldState::default(),
            true,
            &StandardBoxScale::compute(&tokens.metrics, 1.0),
        );

        assert!((look.typography.size - tokens.typography.text.body.size).abs() < f32::EPSILON);
        assert_eq!(look.icon_size, tokens.metrics.control.icon_size);
    }
}
