use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::controls::floating_menu::{FloatingMenuLook, default_floating_menu_look};
use crate::theme::{LumaTextStyle, ThemeTokens};

#[derive(Clone, Debug)]
pub struct AutocompleteLook {
    pub status_color: Hsla,
    /// Status text comes from this theme, rather than a template-local default.
    pub status_typography: crate::theme::LumaTextStyle,
    pub muted_text_color: Hsla,
    pub clear_icon_color: Hsla,
    pub clear_icon_hover_color: Hsla,
    pub menu: FloatingMenuLook,
}

pub trait AutocompleteTheme: Send + Sync {
    fn resolve(&self) -> AutocompleteLook;
}

#[derive(Clone, Debug, Default)]
pub struct DefaultAutocompleteTheme {
    tokens: ThemeTokens,
}

pub fn default_autocomplete_theme() -> Arc<dyn AutocompleteTheme> {
    static THEME: OnceLock<Arc<dyn AutocompleteTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(DefaultAutocompleteTheme::default())).clone()
}

impl DefaultAutocompleteTheme {
    pub fn new(tokens: ThemeTokens) -> Self {
        Self { tokens }
    }
}

impl AutocompleteTheme for DefaultAutocompleteTheme {
    fn resolve(&self) -> AutocompleteLook {
        default_autocomplete_look(&self.tokens)
    }
}

pub(crate) fn default_autocomplete_look(tokens: &ThemeTokens) -> AutocompleteLook {
    let palette = tokens.stylesheet.recipes.palette("autocomplete", &tokens.palette);
    let mut look = AutocompleteLook {
        status_color: palette.state.selected.background,
        // Body weight; pixel size and line height come from autocomplete geometry.
        status_typography: LumaTextStyle { size: 12.5, line_height: 18.0, ..tokens.typography.text.body },
        muted_text_color: palette.surface.floating.foreground,
        clear_icon_color: palette.surface.floating.foreground,
        clear_icon_hover_color: palette.form.input.foreground,
        menu: default_floating_menu_look(tokens),
    };
    let geometry = tokens.stylesheet.autocomplete.resolve_geometry(crate::theme::stylesheet::AutocompleteGeometry {
        status_font_size: look.status_typography.size,
        status_line_height: look.status_typography.line_height,
        ..Default::default()
    });
    look.status_typography.size = geometry.status_font_size;
    look.status_typography.line_height = geometry.status_line_height;
    look
}

#[cfg(test)]
mod stylesheet_tests {
    use super::*;
    use crate::{
        color::GamutMapping,
        theme::{ThemeMode, stylesheet::CommonStylesheet},
    };

    #[test]
    fn status_typography_comes_from_theme_tokens_then_common_overrides() {
        let sheet = CommonStylesheet::resolve_layers(&[
            "[common.autocomplete.geometry]\nstatus_font_size=19\nstatus_line_height=27",
        ])
        .unwrap();
        let mut tokens = sheet.theme_tokens(ThemeMode::Light, GamutMapping::CssLocalMinde).unwrap();
        tokens.typography.text.body.weight = gpui::FontWeight::BOLD;
        let theme = DefaultAutocompleteTheme::new(tokens);
        let look = theme.resolve();
        assert_eq!(
            (look.status_typography.size, look.status_typography.line_height, look.status_typography.weight),
            (19.0, 27.0, gpui::FontWeight::BOLD)
        );
    }
}

crate::theme::stylesheet::geometry::geometry_family!(
    AutocompleteStylesheet,
    AutocompleteGeometryStylesheet,
    AutocompleteGeometry,
    ResolvedAutocompleteGeometry,
    "autocomplete",
    [
        gap,
        status_gap,
        clear_inset,
        clear_width,
        clear_icon_size,
        status_font_size,
        status_line_height,
        trigger_width,
        popup_extra_width
    ]
);

impl AutocompleteStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: AutocompleteGeometryStylesheet {
                gap: Some(10.0),
                status_gap: Some(10.0),
                clear_inset: Some(10.0),
                clear_width: Some(18.0),
                clear_icon_size: Some(12.0),
                trigger_width: Some(240.0),
                popup_extra_width: Some(24.0),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
