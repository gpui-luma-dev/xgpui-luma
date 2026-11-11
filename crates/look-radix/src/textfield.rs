//! Radix text-field theme adapter.
//!
//! Variants match Radix Themes Text Field: `classic` | `surface` | `soft`.

use std::sync::Arc;

use gpui::{BoxShadow, FontWeight, SharedString, point, px};
use xgpui_luma::controls::textfield::{
    TextFieldLook, TextFieldPalette, TextFieldState, TextFieldTemplate, TextFieldTheme, ThemedTextFieldTemplate,
    compose_textfield_look,
};
use xgpui_luma::theme::{ControlSize, LumaTextStyle, MetricTokens, StandardBoxScale};

use crate::look::Look;
use crate::scale::ScaleFamily;
use crate::semantic::SemanticRole;

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

    pub fn control_size(self) -> ControlSize {
        match self {
            Self::One => ControlSize::Sm,
            Self::Two => ControlSize::Md,
            Self::Three => ControlSize::Lg,
        }
    }
}

struct TextFieldThemeAdapter {
    look: Look,
    variant: TextFieldVariant,
}

impl TextFieldTheme for TextFieldThemeAdapter {
    fn resolve(&self, state: TextFieldState, enabled: bool) -> TextFieldPalette {
        resolve_text_chrome_palette(&self.look, self.variant, state.hovered, state.invalid, enabled)
    }

    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }

    fn resolve_look(
        &self,
        state: TextFieldState,
        enabled: bool,
        size: ControlSize,
        scale: &StandardBoxScale,
    ) -> TextFieldLook {
        let mut palette = self.resolve(state, enabled);
        palette.typography = typography_for_size(size);
        let mut look = compose_textfield_look(&palette, scale, self.metrics().border_width.default);
        let stylesheet = self.look.stylesheet_snapshot();
        look.helper_geometry = stylesheet.shared_geometry();
        let geometry = stylesheet.textfield.resolve_geometry(
            crate::look::sdk_size_key(size),
            xgpui_luma::theme::stylesheet::TextFieldGeometry {
                min_height: look.min_height,
                padding_x: look.padding_x,
                padding_y: look.padding_y,
                gap: look.gap,
                icon_size: look.icon_size,
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
        look.gap = geometry.gap;
        look.icon_size = geometry.icon_size;
        look.typography.size = geometry.font_size;
        look.typography.line_height = geometry.line_height;
        if matches!(self.variant, TextFieldVariant::Soft | TextFieldVariant::Classic) {
            // Soft has no rim; Classic draws its edge via shadow-1.
            look.border_width = 0.0;
        }
        look
    }
}

pub fn textfield_theme(look: &Look) -> Arc<dyn TextFieldTheme> {
    textfield_theme_with(look, TextFieldVariant::default())
}

pub fn textfield_theme_with(look: &Look, variant: TextFieldVariant) -> Arc<dyn TextFieldTheme> {
    Arc::new(TextFieldThemeAdapter { look: look.clone(), variant })
}

pub fn textfield_template(look: &Look, variant: TextFieldVariant) -> Arc<dyn TextFieldTemplate> {
    Arc::new(ThemedTextFieldTemplate::new(textfield_theme_with(look, variant)))
}

pub(crate) fn resolve_text_chrome_palette(
    look: &Look,
    variant: TextFieldVariant,
    hovered: bool,
    invalid: bool,
    enabled: bool,
) -> TextFieldPalette {
    let foreground = look.resolve_role(SemanticRole::Foreground).hsla();
    let placeholder = look.resolve_role(SemanticRole::MutedForeground).hsla();
    let selection_background = look.resolve_role(SemanticRole::Soft).hsla();
    let selection_foreground = look.resolve_role(SemanticRole::SoftForeground).hsla();
    let gray = |n| look.resolve_step(ScaleFamily::Gray, n).hsla();
    let accent = |n| look.resolve_step(ScaleFamily::Color, n).hsla();
    let surface = look.resolve_role(SemanticRole::Surface).hsla();

    if !enabled {
        return TextFieldPalette {
            focus_border: if invalid {
                gpui::transparent_black()
            } else {
                look.resolve_role(SemanticRole::Focus).hsla()
            },
            background: match variant {
                TextFieldVariant::Soft => alpha(gray(3), 0.55),
                TextFieldVariant::Classic | TextFieldVariant::Surface => surface,
            },
            foreground: placeholder,
            border: match variant {
                TextFieldVariant::Soft => gpui::transparent_black(),
                TextFieldVariant::Classic | TextFieldVariant::Surface => gray(6),
            },
            placeholder,
            icon: placeholder,
            selection_background,
            selection_foreground,
            caret: placeholder,
            shadow: None,
            typography: typography_for_size(ControlSize::Md),
            font_family: SharedString::from("System UI"),
        };
    }

    // Radix: surface = inset gray rim; classic = raised shadow-1; soft = accent tint, no border.
    let (background, border, shadow) = if invalid {
        let bg = match variant {
            TextFieldVariant::Soft => accent(3),
            TextFieldVariant::Classic | TextFieldVariant::Surface => surface,
        };
        (bg, look.resolve_role(SemanticRole::Destructive).hsla(), None)
    } else {
        match variant {
            TextFieldVariant::Surface => {
                let border = if hovered { gray(8) } else { gray(7) };
                (surface, border, None)
            }
            TextFieldVariant::Classic => {
                // Classic draws its edge via `--shadow-1`, not a gray stroke.
                (surface, gpui::transparent_black(), Some(classic_field_shadow(gray(5), alpha(gray(2), 0.45))))
            }
            TextFieldVariant::Soft => {
                let bg = if hovered { accent(4) } else { accent(3) };
                (bg, gpui::transparent_black(), None)
            }
        }
    };

    TextFieldPalette {
        focus_border: if invalid {
            gpui::transparent_black()
        } else {
            look.resolve_role(SemanticRole::Focus).hsla()
        },
        background,
        foreground,
        border,
        placeholder,
        icon: placeholder,
        selection_background,
        selection_foreground,
        caret: foreground,
        shadow,
        typography: typography_for_size(ControlSize::Md),
        font_family: SharedString::from("System UI"),
    }
}

fn alpha(color: gpui::Hsla, a: f32) -> gpui::Hsla {
    gpui::Hsla { a, ..color }
}

/// Radix `--shadow-1` approximation: hairline rim + soft top fade.
fn classic_field_shadow(edge: gpui::Hsla, fade: gpui::Hsla) -> Vec<BoxShadow> {
    vec![
        BoxShadow {
            offset: point(px(0.0), px(0.0)),
            blur_radius: px(0.0),
            spread_radius: px(1.0),
            color: edge,
            inset: true,
        },
        BoxShadow {
            offset: point(px(0.0), px(1.5)),
            blur_radius: px(2.0),
            spread_radius: px(0.0),
            color: fade,
            inset: true,
        },
    ]
}

fn typography_for_size(size: ControlSize) -> LumaTextStyle {
    let geometry = crate::look::embedded_common_stylesheet()
        .textfield
        .resolve_geometry(crate::look::sdk_size_key(size), Default::default());
    LumaTextStyle { size: geometry.font_size, line_height: geometry.line_height, weight: FontWeight::NORMAL }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn existing_theme_resolves_replaced_geometry_in_both_modes() {
        let look = Look::built_in();
        let fork = look.fork();
        let theme = textfield_theme(&look);
        let fork_theme = textfield_theme(&fork);
        let scale = StandardBoxScale::compute(ControlSize::Md, &look.metrics(), 1.0);
        let original = fork_theme.resolve_look(TextFieldState::default(), true, ControlSize::Md, &scale);
        let mut sheet = look.common_stylesheet();
        sheet.textfield.geometry.min_height = Some(73.0);
        look.set_common_stylesheet(sheet);
        for mode in [xgpui_luma::theme::ThemeMode::Light, xgpui_luma::theme::ThemeMode::Dark] {
            look.set_mode(mode);
            assert_eq!(theme.resolve_look(TextFieldState::default(), true, ControlSize::Md, &scale).min_height, 73.0);
            assert_eq!(
                fork_theme.resolve_look(TextFieldState::default(), true, ControlSize::Md, &scale).min_height,
                original.min_height
            );
        }
    }

    #[test]
    fn focus_adorner_uses_focus_role_without_replacing_face_border() {
        let look = Look::built_in();
        let theme = textfield_theme(&look);
        let palette = theme.resolve(TextFieldState { focused: true, focus_visible: true, ..Default::default() }, true);
        let expected = look.resolve_role(SemanticRole::Focus).hsla();
        assert_eq!(palette.focus_border, expected);
        assert_eq!(palette.border, theme.resolve(TextFieldState::default(), true).border);
    }

    #[test]
    fn soft_uses_accent_tint_background() {
        let look = Look::built_in();
        let theme = textfield_theme_with(&look, TextFieldVariant::Soft);
        let palette = theme.resolve(TextFieldState::default(), true);
        assert_eq!(palette.background, look.resolve_step(ScaleFamily::Color, 3).hsla());
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

/// Native common geometry owned by this control.
pub(crate) fn native_textfield_stylesheet() -> xgpui_luma::theme::stylesheet::TextFieldStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    TextFieldStylesheet {
        sizes: [("1", 12.5, 18.0), ("2", 14.0, 20.0), ("3", 16.0, 22.0)]
            .into_iter()
            .map(|(key, font, line)| {
                (
                    key.into(),
                    TextFieldGeometryStylesheet {
                        font_size: Some(font),
                        line_height: Some(line),
                        ..Default::default()
                    },
                )
            })
            .collect(),
        ..Default::default()
    }
}
