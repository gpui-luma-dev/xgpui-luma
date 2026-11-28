use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla, SharedString};
use xgpui_luma_style::Resolved;

use crate::style::StyleBinding;

use crate::theme::{LumaTextStyle, MetricTokens};
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

pub trait TextFieldTheme: Send + Sync {
    /// The `textfield` binding the field is read from.
    fn style(&self) -> StyleBinding {
        StyleBinding::sdk("textfield")
    }

    /// The field's resolved style in `state` (#433): templates draw from its root.
    fn resolve_style(&self, state: TextFieldState, enabled: bool) -> Arc<Resolved> {
        crate::controls::text::resolve_text_field(&self.style(), state, enabled)
    }

    fn resolve(&self, state: TextFieldState, enabled: bool) -> TextFieldPalette;

    fn metrics(&self) -> MetricTokens;
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

#[cfg(test)]
mod override_tests {
    use super::*;
    use crate::style::PartRead;
    use crate::style::lss::{Overrides, PartId, Prop};

    /// Style overrides set over a field's style reach the resolved root.
    #[test]
    fn overrides_set_values_over_the_resolved_style() {
        let theme = default_textfield_theme();
        let overrides = Overrides::new().set(PartId::ROOT, Prop::<f32>::new("font_size"), 21.0).set(
            PartId::ROOT,
            Prop::<f32>::new("padding_x"),
            5.0,
        );
        let style = crate::controls::text::resolve_text_field(
            &theme.style().with_overrides(&overrides),
            TextFieldState::default(),
            true,
        );
        assert_eq!(style.root().typography(14.0, 20.0).size, 21.0);
        assert_eq!(style.root().number("padding_x"), 5.0);
    }

    #[test]
    fn default_theme_uses_body_typography_and_control_icon_size() {
        let tokens = crate::theme::ThemeTokens::default();
        let style = default_textfield_theme().resolve_style(TextFieldState::default(), true);
        assert!((style.root().typography(14.0, 20.0).size - tokens.typography.text.body.size).abs() < f32::EPSILON);
        assert_eq!(style.root().number("icon_size"), tokens.metrics.control.icon_size);
    }
}
