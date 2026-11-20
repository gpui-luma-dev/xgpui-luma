//! The autocomplete's status line and clear button read from `.lss` (issue #285): `autocomplete.lss`.

use std::sync::Arc;

use gpui::Hsla;
use xgpui_luma_style::{PartView, Value};

use super::theme::{AutocompleteLook, AutocompleteTheme};
use crate::controls::floating_menu::FloatingMenuLook;
use crate::style::StyleBinding;
use crate::theme::LumaTextStyle;

fn color(part: PartView<'_>, name: &str) -> Hsla {
    part.value(name).and_then(Value::as_color).map(Hsla::from).unwrap_or_else(gpui::transparent_black)
}

/// The autocomplete for `binding` (the `autocomplete` control), with `menu` as its popup.
pub fn styled_autocomplete_look(binding: &StyleBinding, menu: FloatingMenuLook) -> AutocompleteLook {
    let resolved = binding.resolve(&[], 1.0);
    let root = resolved.root();
    AutocompleteLook {
        status_color: color(root, "status"),
        status_typography: LumaTextStyle {
            size: root.font_size().unwrap_or(12.5),
            line_height: root.line_height().unwrap_or(18.0),
            weight: gpui::FontWeight(root.value("font_weight").and_then(Value::as_number).unwrap_or(400.0)),
        },
        muted_text_color: color(root, "muted_text"),
        clear_icon_color: color(root, "clear_icon"),
        clear_icon_hover_color: color(root, "clear_icon_hover"),
        menu,
    }
}

/// An autocomplete theme read from `.lss`, resolved on every presentation.
#[derive(Clone)]
pub struct StyledAutocompleteTheme {
    binding: StyleBinding,
    menu: crate::controls::popup_menu::MenuSource,
}

impl StyledAutocompleteTheme {
    pub fn new(binding: StyleBinding, menu: crate::controls::popup_menu::MenuSource) -> Self {
        Self { binding, menu }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self {
            binding: StyleBinding::sdk("autocomplete"),
            menu: Arc::new(|| crate::controls::floating_menu::default_floating_menu_theme().resolve()),
        }
    }
}

impl AutocompleteTheme for StyledAutocompleteTheme {
    fn resolve(&self) -> AutocompleteLook {
        styled_autocomplete_look(&self.binding, (self.menu)())
    }
}
