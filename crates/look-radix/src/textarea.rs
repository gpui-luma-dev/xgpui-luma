//! Radix textareas (Classic / Surface / Soft), drawn from `.lss`.

use std::sync::Arc;

use xgpui_luma::controls::textarea::{TextAreaTemplate, TextAreaTheme, ThemedTextAreaTemplate};

use crate::look::Look;
use crate::textfield::TextFieldVariant;

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

    pub fn control_size(self) -> crate::RadixSize {
        match self {
            Self::One => crate::RadixSize::One,
            Self::Two => crate::RadixSize::Two,
            Self::Three => crate::RadixSize::Three,
        }
    }
}

pub fn textarea_theme(look: &Look) -> Arc<dyn TextAreaTheme> {
    textarea_theme_with(look, TextFieldVariant::default())
}

/// The Radix textarea drawn from `.lss` (it extends the text field in `style/textfield.lss`).
pub fn textarea_theme_with(look: &Look, variant: TextFieldVariant) -> Arc<dyn TextAreaTheme> {
    Arc::new(
        xgpui_luma::controls::text::StyledTextAreaTheme::new(crate::textfield::live_source(look))
            .with(crate::textfield::text_input_binding(look, variant)),
    )
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
