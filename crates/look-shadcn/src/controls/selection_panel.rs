//! Selection panel — same popover list surface as selector items panel.
#![allow(deprecated)] // Still reads a deprecated Look until its family has a style-override replacement (#433).

use xgpui_luma::controls::selection_panel::SelectionPanelLook;
use xgpui_luma::theme::{ThemeMode};

use crate::mode::ShadcnModeTokens;
use crate::size::ShadcnSize;

/// The selection panel's style (#433): the `selection_panel` control (the Shadcn floating menu unless restyled) at
/// `size`, which the SDK's template resolves and draws from.
pub fn selection_panel_style(mode: &ShadcnModeTokens, size: ShadcnSize) -> xgpui_luma::style::StyleBinding {
    super::selector_items_panel::mode_binding(mode, "selection_panel", size)
}

/// The selection panel, built from [`selection_panel_style`].
#[deprecated(note = "use `selection_panel_style`; the SDK's template draws from the style (#433)")]
pub fn selection_panel_look(mode: &ShadcnModeTokens, _theme_mode: ThemeMode, size: ShadcnSize) -> SelectionPanelLook {
    SelectionPanelLook::from_style(&selection_panel_style(mode, size))
}
