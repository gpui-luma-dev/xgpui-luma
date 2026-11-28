use std::sync::{Arc, OnceLock};

pub trait AutocompleteTheme: Send + Sync {
    /// The status line and clear button's style (#433): an `autocomplete` binding the controls resolve and read.
    fn style(&self) -> crate::style::StyleBinding {
        crate::style::StyleBinding::sdk("autocomplete")
    }

    /// The popup menu's style: a `floating_menu` binding.
    fn menu(&self) -> crate::style::StyleBinding {
        crate::style::StyleBinding::sdk("floating_menu")
    }
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
