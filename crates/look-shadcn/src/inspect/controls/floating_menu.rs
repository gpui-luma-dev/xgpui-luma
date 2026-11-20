//! Inspect metadata for `floating_menu`.

use xgpui_luma::theme::ThemeMode;
use crate::ShadcnModeTokens;
use crate::size::ShadcnSize;

#[derive(Clone, Debug)]
pub struct FloatingMenuInspectMetrics {
    pub radius: crate::ResolvedMetric,
    pub padding: crate::ResolvedMetric,
    pub min_width: crate::ResolvedMetric,
    pub item_height: crate::ResolvedMetric,
    pub item_padding_x: crate::ResolvedMetric,
    pub item_gap: crate::ResolvedMetric,
    pub item_icon_size: crate::ResolvedMetric,
    pub item_radius: crate::ResolvedMetric,
    pub submenu_offset_x: crate::ResolvedMetric,
}

pub fn inspect_floating_menu_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ShadcnSize,
) -> FloatingMenuInspectMetrics {
    let table = crate::tables::metrics::resolve_floating_menu_metrics(mode, theme_mode, size);
    table.into()
}

impl From<crate::tables::metrics::FloatingMenuMetricTable> for FloatingMenuInspectMetrics {
    fn from(table: crate::tables::metrics::FloatingMenuMetricTable) -> Self {
        Self {
            radius: table.radius,
            padding: table.padding,
            min_width: table.min_width,
            item_height: table.item_height,
            item_padding_x: table.item_padding_x,
            item_gap: table.item_gap,
            item_icon_size: table.item_icon_size,
            item_radius: table.item_radius,
            submenu_offset_x: table.submenu_offset_x,
        }
    }
}
