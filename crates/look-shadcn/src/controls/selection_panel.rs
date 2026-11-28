//! Selection panel — same popover list surface as selector items panel.

use crate::mode::ShadcnModeTokens;
use crate::size::ShadcnSize;

/// The selection panel's style (#433): the `selection_panel` control (the Shadcn floating menu unless restyled) at
/// `size`, which the SDK's template resolves and draws from.
pub fn selection_panel_style(mode: &ShadcnModeTokens, size: ShadcnSize) -> xgpui_luma::style::StyleBinding {
    super::selector_items_panel::mode_binding(mode, "selection_panel", size)
}
