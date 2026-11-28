//! The autocomplete's status line and clear button read from `.lss` (issue #285): `autocomplete.lss`.

use std::sync::Arc;

use xgpui_luma_style::Value;

use super::theme::{AutocompleteTheme};

use crate::style::StyleBinding;
use crate::theme::LumaTextStyle;

/// The status line's text in a resolved `autocomplete` root (#433).
pub fn status_typography(root: xgpui_luma_style::PartView<'_>) -> LumaTextStyle {
    LumaTextStyle {
        size: root.font_size().unwrap_or(12.5),
        line_height: root.line_height().unwrap_or(18.0),
        weight: gpui::FontWeight(root.value("font_weight").and_then(Value::as_number).unwrap_or(400.0)),
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
            menu: Arc::new(|| crate::style::StyleBinding::sdk("floating_menu")),
        }
    }
}

impl AutocompleteTheme for StyledAutocompleteTheme {
    fn style(&self) -> StyleBinding {
        self.binding.clone()
    }

    fn menu(&self) -> StyleBinding {
        (self.menu)()
    }
}
