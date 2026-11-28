use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla};
use xgpui_luma_style::Resolved;

use crate::style::StyleBinding;

use crate::theme::{LumaTextStyle, MetricTokens};
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
}

/// The SDK's textarea theme: `textarea.lss` alone.
pub fn default_textarea_theme() -> Arc<dyn TextAreaTheme> {
    static THEME: OnceLock<Arc<dyn TextAreaTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(crate::controls::text::StyledTextAreaTheme::sdk())).clone()
}
