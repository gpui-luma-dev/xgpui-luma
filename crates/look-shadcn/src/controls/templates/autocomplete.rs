use std::sync::Arc;

use xgpui_luma::controls::autocomplete::AutocompleteTheme;
use xgpui_luma::theme::ControlSize;

use crate::controls::autocomplete::autocomplete_textbox_look;
use crate::look::ShadcnLook;

struct ShadcnAutocompleteTheme {
    theme: ShadcnLook,
}

impl AutocompleteTheme for ShadcnAutocompleteTheme {
    fn resolve(&self, size: ControlSize) -> xgpui_luma::controls::autocomplete::AutocompleteLook {
        let tokens = self.theme.mode_tokens();
        autocomplete_textbox_look(tokens.as_ref(), self.theme.mode(), size)
    }
}

pub fn autocomplete_theme(theme: ShadcnLook) -> Arc<dyn AutocompleteTheme> {
    Arc::new(ShadcnAutocompleteTheme { theme: theme.clone() })
}
