use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla};
use xgpui_luma_style::Resolved;

use crate::style::StyleBinding;

use crate::theme::{LumaTextStyle, MetricTokens, StandardBoxScale};
use crate::controls::textarea::TextAreaState;

#[derive(Clone, Debug)]
pub struct TextAreaPalette {
    /// Independent focus-adorner paint. Transparent paint retains the configured geometry.
    pub focus_border: gpui::Hsla,
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub placeholder: Hsla,
    pub selection_background: Hsla,
    pub selection_foreground: Hsla,
    pub caret: Hsla,
    pub shadow: Option<Vec<BoxShadow>>,
    pub typography: LumaTextStyle,
    pub font_family: String,
}

#[deprecated(note = "templates draw from the resolved style; use `TextAreaTheme::resolve_style` (#433)")]
#[derive(Clone, Debug)]
pub struct TextAreaLook {
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
    pub selection_background: Hsla,
    pub selection_foreground: Hsla,
    pub caret: Hsla,
    pub shadow: Option<Vec<BoxShadow>>,
    pub typography: LumaTextStyle,
    pub font_family: String,
    pub min_height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub radius: f32,
    pub border_width: f32,
}

pub trait TextAreaTheme: Send + Sync {
    /// The `textarea` binding the area is read from.
    fn style(&self) -> StyleBinding {
        StyleBinding::sdk("textarea")
    }

    /// The area's resolved style in `state` (#433): templates draw from its root.
    fn resolve_style(&self, state: TextAreaState, enabled: bool) -> Arc<Resolved> {
        crate::controls::text::resolve_text_area(&self.style(), state, enabled)
    }

    fn resolve(&self, state: TextAreaState, enabled: bool) -> TextAreaPalette;

    fn metrics(&self) -> MetricTokens;

    #[deprecated(note = "templates draw from the resolved style; use `resolve_style` (#433)")]
    #[allow(deprecated)]
    fn resolve_look(&self, state: TextAreaState, enabled: bool, scale: &StandardBoxScale) -> TextAreaLook {
        compose_textarea_look(&self.resolve(state, enabled), scale, self.metrics().border_width)
    }
}

/// The SDK's textarea theme: `textarea.lss` alone.
pub fn default_textarea_theme() -> Arc<dyn TextAreaTheme> {
    static THEME: OnceLock<Arc<dyn TextAreaTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(crate::controls::text::StyledTextAreaTheme::sdk())).clone()
}

#[deprecated(note = "templates draw from the resolved style (#433)")]
#[allow(deprecated)]
pub fn compose_textarea_look(palette: &TextAreaPalette, scale: &StandardBoxScale, border_width: f32) -> TextAreaLook {
    TextAreaLook {
        helper_geometry: crate::theme::stylesheet::SharedGeometry::sdk(),
        caret_width: crate::style::sdk_number("textarea", "caret_width"),
        caret_height_extra: crate::style::sdk_number("textarea", "caret_height_extra"),
        // Palette themes draw their own disabled colors; only `.lss` themes fade.
        opacity: 1.0,
        background: palette.background,
        foreground: palette.foreground,
        border: palette.border,
        focus_border: palette.focus_border,
        placeholder: palette.placeholder,
        selection_background: palette.selection_background,
        selection_foreground: palette.selection_foreground,
        caret: palette.caret,
        shadow: palette.shadow.clone(),
        typography: palette.typography,
        font_family: palette.font_family.clone(),
        min_height: scale.height,
        padding_x: scale.padding_x,
        padding_y: scale.padding_y,
        radius: scale.radius,
        border_width,
    }
}
