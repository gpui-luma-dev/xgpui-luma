//! Inspect metadata for `overlay_window`.

use crate::ResolvedMetric;

#[derive(Clone, Debug)]
pub struct OverlayWindowInspectMetrics {
    pub radius: ResolvedMetric,
    pub padding: ResolvedMetric,
    pub min_width: ResolvedMetric,
    pub max_width: ResolvedMetric,
    pub estimated_height: ResolvedMetric,
}

impl From<crate::tables::metrics::OverlayWindowMetricTable> for OverlayWindowInspectMetrics {
    fn from(table: crate::tables::metrics::OverlayWindowMetricTable) -> Self {
        Self {
            radius: table.radius,
            padding: table.padding,
            min_width: table.min_width,
            max_width: table.max_width,
            estimated_height: table.estimated_height,
        }
    }
}
