use std::sync::Arc;

use xgpui_luma::controls::popup_menu::MenuSource;

use crate::controls::selection_panel::selection_panel_style;
use crate::look::ShadcnLook;

/// The selection panel's style, read from the look's current mode each time it is drawn (#433).
pub fn selection_panel_menu(theme: ShadcnLook) -> MenuSource {
    Arc::new(move || selection_panel_style(theme.mode_tokens().as_ref(), theme.size()))
}
