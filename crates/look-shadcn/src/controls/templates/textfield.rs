//! Shadcn text fields, drawn from `.lss` (`style/textfield.lss`) following the look's mode and catalog.

use std::sync::Arc;

use xgpui_luma::controls::text::StyledTextFieldTheme;
use xgpui_luma::controls::textfield::{TextFieldTemplate, TextFieldTheme, ThemedTextFieldTemplate};
use xgpui_luma::style::ThemeSource;

use crate::look::ShadcnLook;

/// The `.lss` text field for a Shadcn style (`None` is the base, outline) at the look's size.
pub(crate) fn styled_textfield_theme(theme: ShadcnLook, variant: Option<&str>) -> StyledTextFieldTheme {
    let size = theme.size();
    let look = theme.clone();
    StyledTextFieldTheme::new(ThemeSource::Live(Arc::new(move || look.style_theme()))).with(|binding| {
        let binding = match variant {
            Some(variant) => binding.variant(variant),
            None => binding,
        };
        if size != crate::ShadcnSize::Md {
            binding.attr("size", size.key())
        } else {
            binding
        }
    })
}

fn theme_for(theme: ShadcnLook, variant: Option<&str>) -> Arc<dyn TextFieldTheme> {
    Arc::new(styled_textfield_theme(theme, variant))
}

fn template_for(theme: ShadcnLook, variant: Option<&str>) -> Arc<dyn TextFieldTemplate> {
    Arc::new(ThemedTextFieldTemplate::new(theme_for(theme, variant)))
}

pub fn textfield_template(theme: ShadcnLook) -> Arc<dyn TextFieldTemplate> {
    template_for(theme, None)
}

pub fn textfield_theme(theme: ShadcnLook) -> Arc<dyn TextFieldTheme> {
    theme_for(theme, None)
}

pub fn input_textfield_theme(theme: ShadcnLook) -> Arc<dyn TextFieldTheme> {
    theme_for(theme, Some("input"))
}

pub fn input_textfield_template(theme: ShadcnLook) -> Arc<dyn TextFieldTemplate> {
    template_for(theme, Some("input"))
}

pub fn surface_textfield_theme(theme: ShadcnLook) -> Arc<dyn TextFieldTheme> {
    theme_for(theme, Some("surface"))
}

pub fn primary_textfield_theme(theme: ShadcnLook) -> Arc<dyn TextFieldTheme> {
    theme_for(theme, Some("primary"))
}

pub fn primary_textfield_template(theme: ShadcnLook) -> Arc<dyn TextFieldTemplate> {
    template_for(theme, Some("primary"))
}

#[cfg(test)]
mod tests {
    use gpui::Hsla;
    use xgpui_luma::controls::textarea::TextAreaState;
    use xgpui_luma::controls::textfield::TextFieldState;
    use xgpui_luma::theme::{ThemeMode};

    use super::*;

    fn close(a: Hsla, b: Hsla) -> bool {
        let (a, b) = (a.to_rgb(), b.to_rgb());
        [(a.r, b.r), (a.g, b.g), (a.b, b.b), (a.a, b.a)].iter().all(|(x, y)| (x - y).abs() < 2e-3)
    }

    fn look(mode: ThemeMode) -> ShadcnLook {
        let look = ShadcnLook::from_css_str(crate::FALLBACK_CSS).unwrap();
        look.set_mode(mode);
        look
    }

    fn css(look: &ShadcnLook, name: &str) -> Hsla {
        look.mode_tokens().catalog.color(name).unwrap()
    }

    /// A field's resolved root, read the way the old look read it (#433).
    struct Probe {
        background: Hsla,
        border: Hsla,
        foreground: Hsla,
        selection_foreground: Hsla,
        focus_border: Hsla,
        shadow: Option<Vec<gpui::BoxShadow>>,
        opacity: f32,
        typography: xgpui_luma::theme::LumaTextStyle,
    }

    fn resolve(theme: Arc<dyn TextFieldTheme>, state: TextFieldState, enabled: bool) -> Probe {
        use xgpui_luma::style::PartRead;
        let style = theme.resolve_style(state, enabled);
        let root = style.root();
        Probe {
            background: root.color("background"),
            border: root.color("border"),
            foreground: root.color("foreground"),
            selection_foreground: root.color("selection_foreground"),
            focus_border: root.color("focus_border"),
            shadow: root.box_shadow_opt(),
            opacity: root.opacity(),
            typography: root.typography(14.0, 20.0),
        }
    }

    #[test]
    fn outline_is_transparent_in_light_and_input_tinted_in_dark() {
        let light = look(ThemeMode::Light);
        let field = resolve(light.textfield_theme(), TextFieldState::default(), true);
        assert_eq!(field.background.a, 0.0);
        assert!(close(field.border, css(&light, "border")));
        assert!(close(field.foreground, css(&light, "foreground")));
        assert!(close(field.selection_foreground, css(&light, "primary-foreground")));
        assert!(field.shadow.is_none());

        let dark = look(ThemeMode::Dark);
        let field = resolve(dark.textfield_theme(), TextFieldState::default(), true);
        assert!(close(field.background, Hsla { a: 0.3, ..css(&dark, "input") }));
    }

    #[test]
    fn input_fills_more_when_hovered_in_dark_mode_only() {
        let dark = look(ThemeMode::Dark);
        let hovered = TextFieldState { hovered: true, ..Default::default() };
        let field = resolve(dark.input_textfield_theme(), hovered, true);
        assert!(close(field.background, Hsla { a: 0.5, ..css(&dark, "input") }));
        let outline = resolve(dark.textfield_theme(), hovered, true);
        assert!(close(outline.background, Hsla { a: 0.3, ..css(&dark, "input") }));
        let light = look(ThemeMode::Light);
        assert_eq!(resolve(light.input_textfield_theme(), hovered, true).background.a, 0.0);
    }

    #[test]
    fn primary_is_opaque_with_a_shadow_and_fades_when_disabled() {
        let light = look(ThemeMode::Light);
        let field = resolve(light.primary_textfield_theme(), TextFieldState::default(), true);
        assert!(close(field.background, css(&light, "background")));
        assert!(field.shadow.as_ref().is_some_and(|layers| !layers.is_empty()));
        let disabled = resolve(light.primary_textfield_theme(), TextFieldState::default(), false);
        // Faded over its resting fill (#413), not a muted fill.
        assert!(close(disabled.background, css(&light, "background")));
        assert!(disabled.opacity < 1.0);
        assert!(disabled.shadow.as_ref().is_some_and(|layers| !layers.is_empty()));
    }

    #[test]
    fn surface_is_muted_with_no_edge_unless_invalid() {
        let light = look(ThemeMode::Light);
        let field = resolve(light.surface_textfield_theme(), TextFieldState::default(), true);
        assert!(close(field.background, css(&light, "muted")));
        assert_eq!(field.border.a, 0.0);
        let invalid =
            resolve(light.surface_textfield_theme(), TextFieldState { invalid: true, ..Default::default() }, true);
        assert!(close(invalid.border, css(&light, "destructive")));
        assert_eq!(invalid.focus_border.a, 0.0);
    }

    #[test]
    fn sizes_scale_the_label_font_and_textarea_matches_the_field() {
        let light = look(ThemeMode::Light);
        let size = |s| resolve(light.with_size(s).textfield_theme(), TextFieldState::default(), true).typography.size;
        assert_eq!(
            (size(crate::ShadcnSize::Sm), size(crate::ShadcnSize::Md), size(crate::ShadcnSize::Lg)),
            (12.0, 14.0, 16.0)
        );

        let theme = light.primary_textarea_theme();
        let style = theme.resolve_style(TextAreaState::default(), true);
        let area = style.root();
        use xgpui_luma::style::PartRead;
        let field = resolve(light.primary_textfield_theme(), TextFieldState::default(), true);
        assert_eq!(
            (area.color("background"), area.color("border"), area.box_shadow_opt().is_some()),
            (field.background, field.border, true)
        );
    }
}
