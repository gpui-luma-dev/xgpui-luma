//! Radix text-area theme adapter (Classic / Surface / Soft).

use std::sync::Arc;

use xgpui_luma::controls::textarea::{
    TextAreaLook, TextAreaPalette, TextAreaState, TextAreaTemplate, TextAreaTheme, ThemedTextAreaTemplate,
    compose_textarea_look,
};
use xgpui_luma::theme::{ControlSize, MetricTokens, StandardBoxScale};

use crate::look::Look;
use crate::textfield::{TextFieldVariant, resolve_text_chrome_palette};

/// Radix `size` prop on Text Area. Default is [`Self::Two`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextAreaSize {
    One,
    #[default]
    Two,
    Three,
}

impl TextAreaSize {
    pub const ALL: [Self; 3] = [Self::One, Self::Two, Self::Three];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::One => "1",
            Self::Two => "2",
            Self::Three => "3",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::One => "Size 1",
            Self::Two => "Size 2",
            Self::Three => "Size 3",
        }
    }

    pub fn control_size(self) -> ControlSize {
        match self {
            Self::One => ControlSize::Sm,
            Self::Two => ControlSize::Md,
            Self::Three => ControlSize::Lg,
        }
    }
}

struct TextAreaThemeAdapter {
    look: Look,
    variant: TextFieldVariant,
}

impl TextAreaTheme for TextAreaThemeAdapter {
    fn resolve(&self, state: TextAreaState, enabled: bool) -> TextAreaPalette {
        let field =
            resolve_text_chrome_palette(&self.look, self.variant, state.hovered, state.focused, state.invalid, enabled);
        TextAreaPalette {
            background: field.background,
            foreground: field.foreground,
            border: field.border,
            placeholder: field.placeholder,
            selection_background: field.selection_background,
            selection_foreground: field.selection_foreground,
            caret: field.caret,
            shadow: field.shadow,
            typography: field.typography,
            font_family: field.font_family.to_string(),
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }

    fn resolve_look(
        &self,
        state: TextAreaState,
        enabled: bool,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> TextAreaLook {
        let mut look = compose_textarea_look(&self.resolve(state, enabled), scale, self.metrics().border_width.default);
        let stylesheet = self.look.stylesheet_snapshot();
        look.helper_geometry = stylesheet.shared_geometry();
        let geometry = stylesheet.textarea.resolve_geometry(
            crate::look::sdk_size_key(size),
            xgpui_luma::theme::stylesheet::TextAreaGeometry {
                min_height: look.min_height,
                padding_x: look.padding_x,
                padding_y: look.padding_y,
                font_size: look.typography.size,
                line_height: look.typography.line_height,
                ..Default::default()
            },
        );
        look.caret_width = geometry.caret_width;
        look.caret_height_extra = geometry.caret_height_extra;
        look.disabled_opacity = geometry.disabled_opacity;
        look.min_height = geometry.min_height;
        look.padding_x = geometry.padding_x;
        look.padding_y = geometry.padding_y;
        look.typography.size = geometry.font_size;
        look.typography.line_height = geometry.line_height;
        if matches!(self.variant, TextFieldVariant::Soft | TextFieldVariant::Classic) {
            look.border_width = 0.0;
        }
        look
    }
}

pub fn textarea_theme(look: &Look) -> Arc<dyn TextAreaTheme> {
    textarea_theme_with(look, TextFieldVariant::default())
}

pub fn textarea_theme_with(look: &Look, variant: TextFieldVariant) -> Arc<dyn TextAreaTheme> {
    Arc::new(TextAreaThemeAdapter { look: look.clone(), variant })
}

pub fn textarea_template(look: &Look, variant: TextFieldVariant) -> Arc<dyn TextAreaTemplate> {
    Arc::new(ThemedTextAreaTemplate::new(textarea_theme_with(look, variant)))
}

/// Native common geometry owned by this control.
pub(crate) fn native_textarea_stylesheet() -> xgpui_luma::theme::stylesheet::TextAreaStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    TextAreaStylesheet {
        geometry: TextAreaGeometryStylesheet { font_size: Some(14.0), line_height: Some(20.0), ..Default::default() },
        ..Default::default()
    }
}
