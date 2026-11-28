use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla, SharedString};
use xgpui_luma_style::Resolved;

use crate::style::StyleBinding;

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

#[deprecated(note = "templates draw from the resolved style; use `TextFieldTheme::resolve_style` (#433)")]
#[derive(Clone, Debug)]
pub struct TextFieldLook {
    /// Helper chrome resolved through the selected common stylesheet.
    pub helper_geometry: crate::theme::stylesheet::SharedGeometry,
    /// Painted caret width and extra height in pixels.
    pub caret_width: f32,
    pub caret_height_extra: f32,
    /// Whole-input fade in this state: the root's resolved `opacity` (`:disabled { opacity }`), 1 when unset.
    pub opacity: f32,
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

    #[deprecated(note = "templates draw from the resolved style; use `resolve_style` (#433)")]
    #[allow(deprecated)]
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

#[deprecated(note = "templates draw from the resolved style (#433)")]
#[allow(deprecated)]
pub fn compose_textfield_look(
    palette: &TextFieldPalette,
    scale: &StandardBoxScale,
    border_width: f32,
) -> TextFieldLook {
    TextFieldLook {
        helper_geometry: crate::theme::stylesheet::SharedGeometry::sdk(),
        caret_width: crate::style::sdk_number("textfield", "caret_width"),
        caret_height_extra: crate::style::sdk_number("textfield", "caret_height_extra"),
        // Palette themes draw their own disabled colors; only `.lss` themes fade.
        opacity: 1.0,
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
    #[allow(deprecated)]
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

    /// A look closure's changes round-trip as overrides: re-resolving them gives the closure's values.
    #[test]
    #[allow(deprecated)]
    fn a_look_closure_is_bridged_to_overrides() {
        let theme = default_textfield_theme();
        let base = theme.resolve_style(TextFieldState::default(), true);
        let before = TextFieldLook::from_resolved(&base);
        let mut after = before.clone();
        after.padding_y = before.padding_y + 3.0;
        after.typography.size = before.typography.size + 2.0;
        after.shadow = None;
        let overrides = TextFieldLook::overrides_between(&before, &after);
        let style = crate::controls::text::resolve_text_field(
            &theme.style().with_overrides(&overrides),
            TextFieldState::default(),
            true,
        );
        let look = TextFieldLook::from_resolved(&style);
        assert_eq!((look.padding_y, look.typography.size), (after.padding_y, after.typography.size));
        assert!(look.shadow.is_none());
        assert_eq!(look.background, before.background);
    }
}
