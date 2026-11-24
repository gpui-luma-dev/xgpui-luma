use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::controls::floating_menu::FloatingMenuLook;

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

pub fn default_autocomplete_theme() -> Arc<dyn AutocompleteTheme> {
    static THEME: OnceLock<Arc<dyn AutocompleteTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledAutocompleteTheme::sdk())).clone()
}

crate::theme::stylesheet::geometry::lss_geometry!(
    /// The autocomplete's layout, from `autocomplete.lss`. The status text size is the theme's.
    AutocompleteGeometry,
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
