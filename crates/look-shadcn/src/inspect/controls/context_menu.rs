//! Inspect metadata for `context_menu`.

#[derive(Clone, Debug)]
pub struct ContextMenuInspectMetrics {
    pub target_padding_x: crate::ResolvedMetric,
    pub target_padding_y: crate::ResolvedMetric,
    pub target_radius: crate::ResolvedMetric,
    pub target_min_width: crate::ResolvedMetric,
    pub menu: crate::inspect::controls::floating_menu::FloatingMenuInspectMetrics,
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
