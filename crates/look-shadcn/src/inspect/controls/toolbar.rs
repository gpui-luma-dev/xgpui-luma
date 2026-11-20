//! Inspect metadata for `toolbar`.

use crate::ResolvedMetric;

#[derive(Clone, Debug)]
pub struct ToolbarInspectMetrics {
    pub radius: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub gap: ResolvedMetric,
    pub separator_height: ResolvedMetric,
}

impl From<crate::tables::metrics::ToolbarMetricTable> for ToolbarInspectMetrics {
    fn from(table: crate::tables::metrics::ToolbarMetricTable) -> Self {
        Self {
            radius: table.radius,
            padding_x: table.padding_x,
            padding_y: table.padding_y,
            gap: table.gap,
            separator_height: table.separator_height,
        }
    }
}
