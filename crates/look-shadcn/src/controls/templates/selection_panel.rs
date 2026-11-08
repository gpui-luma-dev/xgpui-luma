use std::sync::Arc;

use xgpui_luma::controls::selection_panel::SelectionPanelLookProvider;

use crate::controls::selection_panel::selection_panel_look;
use crate::look::ShadcnLook;

pub fn selection_panel_look_provider(theme: ShadcnLook) -> SelectionPanelLookProvider {
    Arc::new(move |size| {
        let tokens = theme.mode_tokens();
        selection_panel_look(tokens.as_ref(), theme.mode(), size)
    })
}
