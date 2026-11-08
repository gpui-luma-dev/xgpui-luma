//! Inspect metadata for `context_menu`.

use xgpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use crate::ShadcnModeTokens;

pub struct ContextMenuInspectPalette {
    pub target_background: crate::ResolvedColor,
    pub target_foreground: crate::ResolvedColor,
    pub target_border: crate::ResolvedColor,
    pub menu: crate::inspect::controls::floating_menu::FloatingMenuInspectPalette,
}

#[derive(Clone, Debug)]
pub struct ContextMenuInspectMetrics {
    pub target_padding_x: crate::ResolvedMetric,
    pub target_padding_y: crate::ResolvedMetric,
    pub target_radius: crate::ResolvedMetric,
    pub target_min_width: crate::ResolvedMetric,
    pub menu: crate::inspect::controls::floating_menu::FloatingMenuInspectMetrics,
}

pub fn inspect_context_menu_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
    size: ControlSize,
) -> ContextMenuInspectPalette {
    let colors = crate::tables::resolve_context_menu_colors(mode, theme_mode, state);
    let menu = crate::inspect::controls::floating_menu::inspect_floating_menu_color_palette(mode, theme_mode, size);
    ContextMenuInspectPalette {
        target_background: colors.background,
        target_foreground: colors.foreground,
        target_border: colors.border,
        menu,
    }
}

pub fn inspect_context_menu_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> ContextMenuInspectMetrics {
    let table = crate::tables::metrics::resolve_context_menu_metrics(mode, theme_mode, size);
    table.into()
}

impl From<crate::tables::metrics::ContextMenuMetricTable> for ContextMenuInspectMetrics {
    fn from(table: crate::tables::metrics::ContextMenuMetricTable) -> Self {
        Self {
            target_padding_x: table.target_padding_x,
            target_padding_y: table.target_padding_y,
            target_radius: table.target_radius,
            target_min_width: table.target_min_width,
            menu: table.menu.into(),
        }
    }
}
