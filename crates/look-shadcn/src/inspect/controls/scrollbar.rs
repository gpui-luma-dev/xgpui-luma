//! Inspect metadata for `scrollbar`.

#[derive(Clone, Debug)]
pub struct ScrollbarInspectMetrics {
    pub length: crate::ResolvedMetric,
    pub thickness: crate::ResolvedMetric,
    pub track_thickness: crate::ResolvedMetric,
    pub thumb_thickness: crate::ResolvedMetric,
    pub min_thumb_length: crate::ResolvedMetric,
    pub radius: crate::ResolvedMetric,
}

impl From<crate::tables::metrics::ScrollbarMetricTable> for ScrollbarInspectMetrics {
    fn from(table: crate::tables::metrics::ScrollbarMetricTable) -> Self {
        Self {
            length: table.length,
            thickness: table.thickness,
            track_thickness: table.track_thickness,
            thumb_thickness: table.thumb_thickness,
            min_thumb_length: table.min_thumb_length,
            radius: table.radius,
        }
    }
}
