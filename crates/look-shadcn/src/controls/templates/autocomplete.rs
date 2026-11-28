use std::sync::Arc;

use xgpui_luma::controls::autocomplete::AutocompleteTheme;

use crate::controls::autocomplete::autocomplete_style;
use crate::look::ShadcnLook;

struct ShadcnAutocompleteTheme {
    theme: ShadcnLook,
}

impl AutocompleteTheme for ShadcnAutocompleteTheme {
    fn style(&self) -> xgpui_luma::style::StyleBinding {
        autocomplete_style(self.theme.mode_tokens().as_ref(), self.theme.size())
    }

    fn menu(&self) -> xgpui_luma::style::StyleBinding {
        self.theme.selector_items_panel_style(self.theme.size())
    }
}

pub fn autocomplete_theme(theme: ShadcnLook) -> Arc<dyn AutocompleteTheme> {
    Arc::new(ShadcnAutocompleteTheme { theme: theme.clone() })
}
