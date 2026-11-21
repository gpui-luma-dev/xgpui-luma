//! Selection panel — same popover list surface as selector items panel.

use xgpui_luma::controls::selection_panel::SelectionPanelLook;
use xgpui_luma::theme::{ThemeMode};

use crate::mode::ShadcnModeTokens;
use crate::size::ShadcnSize;

/// The selection panel from `.lss` (the `selection_panel` control: the Shadcn floating menu unless restyled).
pub fn selection_panel_look(mode: &ShadcnModeTokens, _theme_mode: ThemeMode, size: ShadcnSize) -> SelectionPanelLook {
    xgpui_luma::controls::selection_panel::styled_selection_panel_look(
        &super::selector_items_panel::mode_binding(mode, "selection_panel", size),
        &mode.metrics_for(size),
        mode.common_for(size),
    )
}
