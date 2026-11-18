//! Selection panel — same popover list surface as selector items panel.

use xgpui_luma::controls::selection_panel::SelectionPanelLook;
use xgpui_luma::controls::selector_list::SelectorItemsPanelLook;
use xgpui_luma::theme::{ThemeMode};

use super::selector_items_panel::selector_items_panel_look;
use crate::mode::ShadcnModeTokens;
use crate::size::ShadcnSize;

pub fn selection_panel_look(mode: &ShadcnModeTokens, theme_mode: ThemeMode, size: ShadcnSize) -> SelectionPanelLook {
    selection_panel_from_items_panel(selector_items_panel_look(mode, theme_mode, size))
        .with_stylesheet(&mode.common_for(size).selection_panel)
}

pub fn selection_panel_from_items_panel(panel: SelectorItemsPanelLook) -> SelectionPanelLook {
    SelectionPanelLook {
        background: panel.background,
        foreground: panel.foreground,
        border: panel.border,
        border_width: panel.border_width,
        disabled_opacity: panel.disabled_opacity,
        shadow: panel.shadow,
        radius: panel.radius,
        padding: panel.padding,
        min_width: panel.min_width,
        item_disabled_foreground: panel.item_disabled_foreground,
        item_hover_background: panel.item_hover_background,
        item_hover_foreground: panel.item_hover_foreground,
        item_typography: panel.item_typography,
        item_height: panel.item_height,
        item_padding_x: panel.item_padding_x,
        item_gap: panel.item_gap,
        item_icon_size: panel.item_icon_size,
        item_radius: panel.item_radius,
    }
}
