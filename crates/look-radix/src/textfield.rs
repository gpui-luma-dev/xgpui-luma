//! Radix text fields, drawn from `.lss` (`style/textfield.lss`).
//!
//! Variants match Radix Themes Text Field: `classic` | `surface` | `soft`.

use std::sync::Arc;

use xgpui_luma::controls::text::StyledTextFieldTheme;
use xgpui_luma::controls::textfield::{TextFieldTemplate, TextFieldTheme, ThemedTextFieldTemplate};

use crate::look::Look;

/// Radix Themes text-field / text-area visual variants.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextFieldVariant {
    Classic,
    #[default]
    Surface,
    Soft,
}

impl TextFieldVariant {
    pub const ALL: [Self; 3] = [Self::Classic, Self::Surface, Self::Soft];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Surface => "surface",
            Self::Soft => "soft",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::Surface => "Surface",
            Self::Soft => "Soft",
        }
    }
}

/// Radix `size` prop on Text Field. Default is [`Self::Two`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextFieldSize {
    One,
    #[default]
    Two,
    Three,
}

impl TextFieldSize {
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

/// The binding for a Radix text input: the variant (surface is the base) at the look's size.
pub(crate) fn text_input_binding(
    look: &Look,
    variant: TextFieldVariant,
) -> impl FnOnce(xgpui_luma::style::StyleBinding) -> xgpui_luma::style::StyleBinding {
    let size = look.size();
    move |binding| {
        let binding = match variant {
            TextFieldVariant::Surface => binding,
            TextFieldVariant::Classic => binding.variant("classic"),
            TextFieldVariant::Soft => binding.variant("soft"),
        };
        if size != crate::RadixSize::Two {
            binding.attr("size", size.key())
        } else {
            binding
        }
    }
}

/// The live `.lss` theme source for a look.
pub(crate) fn live_source(look: &Look) -> xgpui_luma::style::ThemeSource {
    let look = look.clone();
    xgpui_luma::style::ThemeSource::Live(Arc::new(move || look.style_theme()))
}

/// A live binding of `control` to the look's `.lss` at the look's size.
pub(crate) fn sized_binding(look: &Look, control: &'static str) -> xgpui_luma::style::StyleBinding {
    let binding = xgpui_luma::style::StyleBinding::new(control, live_source(look));
    let size = look.size();
    if size != crate::RadixSize::Two {
        binding.attr("size", size.key())
    } else {
        binding
    }
}

pub fn textfield_theme(look: &Look) -> Arc<dyn TextFieldTheme> {
    textfield_theme_with(look, TextFieldVariant::default())
}

/// The Radix text field drawn from `.lss` (`style/textfield.lss`) at the look's size.
pub fn textfield_theme_with(look: &Look, variant: TextFieldVariant) -> Arc<dyn TextFieldTheme> {
    Arc::new(StyledTextFieldTheme::new(live_source(look)).with(text_input_binding(look, variant)))
}

pub fn textfield_template(look: &Look, variant: TextFieldVariant) -> Arc<dyn TextFieldTemplate> {
    Arc::new(ThemedTextFieldTemplate::new(textfield_theme_with(look, variant)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scale::ScaleFamily;
    use crate::semantic::SemanticRole;
    use xgpui_luma::controls::textfield::TextFieldState;

    #[test]
    fn focus_adorner_uses_focus_role_without_replacing_face_border() {
        let look = Look::built_in();
        let theme = textfield_theme(&look);
        let palette = theme.resolve(TextFieldState { focused: true, focus_visible: true, ..Default::default() }, true);
        let expected = look.resolve_role(SemanticRole::Focus).hsla();
        crate::control_group::assert_close(palette.focus_border, expected);
        assert_eq!(palette.border, theme.resolve(TextFieldState::default(), true).border);
    }

    #[test]
    fn soft_uses_accent_tint_background() {
        let look = Look::built_in();
        let theme = textfield_theme_with(&look, TextFieldVariant::Soft);
        let palette = theme.resolve(TextFieldState::default(), true);
        crate::control_group::assert_close(palette.background, look.resolve_step(ScaleFamily::Color, 3).hsla());
    }

    #[test]
    fn classic_uses_shadow_not_gray_border() {
        let look = Look::built_in();
        let classic = textfield_theme_with(&look, TextFieldVariant::Classic).resolve(TextFieldState::default(), true);
        let surface = textfield_theme_with(&look, TextFieldVariant::Surface).resolve(TextFieldState::default(), true);

        assert_eq!(classic.background, surface.background);
        assert!(classic.shadow.is_some(), "classic carries shadow-1");
        assert!(surface.shadow.is_none(), "surface stays flat");
        assert_ne!(classic.border.a, surface.border.a);
    }
}
