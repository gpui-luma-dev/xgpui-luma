//! Inspect metadata for `table`.

use crate::ResolvedMetric;

#[derive(Clone, Debug)]
pub struct TableInspectMetrics {
    pub radius: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub row_min_height: ResolvedMetric,
    pub row_padding_x: ResolvedMetric,
    pub row_padding_y: ResolvedMetric,
}

impl From<crate::tables::metrics::TableMetricTable> for TableInspectMetrics {
    fn from(table: crate::tables::metrics::TableMetricTable) -> Self {
        Self {
            radius: table.radius,
            padding_x: table.padding_x,
            padding_y: table.padding_y,
            row_min_height: table.row_min_height,
            row_padding_x: table.row_padding_x,
            row_padding_y: table.row_padding_y,
        }
    }
}
