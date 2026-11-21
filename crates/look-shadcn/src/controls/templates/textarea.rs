//! Shadcn textareas, drawn from `.lss`: the textarea extends the text field (`style/textfield.lss`).

use std::sync::Arc;

use xgpui_luma::controls::text::StyledTextAreaTheme;
use xgpui_luma::controls::textarea::{TextAreaTemplate, TextAreaTheme, ThemedTextAreaTemplate};
use xgpui_luma::style::ThemeSource;

use crate::look::ShadcnLook;

fn theme_for(theme: ShadcnLook, variant: Option<&str>) -> Arc<dyn TextAreaTheme> {
    let size = theme.size();
    let look = theme.clone();
    Arc::new(StyledTextAreaTheme::new(ThemeSource::Live(Arc::new(move || look.style_theme()))).with(|binding| {
        let binding = match variant {
            Some(variant) => binding.variant(variant),
            None => binding,
        };
        if size != crate::ShadcnSize::Md {
            binding.attr("size", size.key())
        } else {
            binding
        }
    }))
}

fn template_for(theme: ShadcnLook, variant: Option<&str>) -> Arc<dyn TextAreaTemplate> {
    Arc::new(ThemedTextAreaTemplate::new(theme_for(theme, variant)))
}

pub fn textarea_template(theme: ShadcnLook) -> Arc<dyn TextAreaTemplate> {
    template_for(theme, None)
}

pub fn textarea_theme(theme: ShadcnLook) -> Arc<dyn TextAreaTheme> {
    theme_for(theme, None)
}

pub fn surface_textarea_theme(theme: ShadcnLook) -> Arc<dyn TextAreaTheme> {
    theme_for(theme, Some("surface"))
}

pub fn input_textarea_theme(theme: ShadcnLook) -> Arc<dyn TextAreaTheme> {
    theme_for(theme, Some("input"))
}

pub fn input_textarea_template(theme: ShadcnLook) -> Arc<dyn TextAreaTemplate> {
    template_for(theme, Some("input"))
}

pub fn primary_textarea_theme(theme: ShadcnLook) -> Arc<dyn TextAreaTheme> {
    theme_for(theme, Some("primary"))
}

pub fn primary_textarea_template(theme: ShadcnLook) -> Arc<dyn TextAreaTemplate> {
    template_for(theme, Some("primary"))
}
