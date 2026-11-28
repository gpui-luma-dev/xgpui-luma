#![allow(deprecated)] // AutocompleteLook is this module's compatibility type.

use std::sync::{Arc, OnceLock};

use gpui::Hsla;

use crate::controls::floating_menu::FloatingMenuLook;

/// The autocomplete's values, copied out of its style (#433). A compatibility type: the controls read the
/// resolved style itself.
#[deprecated(note = "read `AutocompleteTheme::style()` and `menu()` instead (#433)")]
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
    /// The status line and clear button's style (#433): an `autocomplete` binding the controls resolve and read.
    fn style(&self) -> crate::style::StyleBinding {
        crate::style::StyleBinding::sdk("autocomplete")
    }

    /// The popup menu's style: a `floating_menu` binding.
    fn menu(&self) -> crate::style::StyleBinding {
        crate::style::StyleBinding::sdk("floating_menu")
    }

    /// The look built from [`Self::style`] and [`Self::menu`], for code that still reads an [`AutocompleteLook`].
    #[deprecated(note = "read `style()` and `menu()` instead (#433)")]
    fn resolve(&self) -> AutocompleteLook {
        super::styled::styled_autocomplete_look(
            &self.style(),
            crate::controls::floating_menu::styled_floating_menu_look(&self.menu()),
        )
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
