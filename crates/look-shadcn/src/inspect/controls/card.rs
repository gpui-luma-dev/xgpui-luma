//! Inspect metadata for `card`.

use crate::ResolvedMetric;

#[derive(Clone, Debug)]
pub struct CardInspectMetrics {
    pub padding: ResolvedMetric,
    pub section_gap: ResolvedMetric,
    pub header_gap: ResolvedMetric,
    pub body_gap: ResolvedMetric,
    pub radius: ResolvedMetric,
}

impl From<crate::tables::metrics::CardMetricTable> for CardInspectMetrics {
    fn from(table: crate::tables::metrics::CardMetricTable) -> Self {
        Self {
            padding: table.padding,
            section_gap: table.section_gap,
            header_gap: table.header_gap,
            body_gap: table.body_gap,
            radius: table.radius,
        }
    }
}
