//! Inspect metadata for `slider`.

#[derive(Clone, Debug)]
pub struct SliderInspectMetrics {
    pub width: crate::ResolvedMetric,
    pub height: crate::ResolvedMetric,
    pub track_height: crate::ResolvedMetric,
    pub thumb_size: crate::ResolvedMetric,
    pub radius: crate::ResolvedMetric,
}

impl From<crate::tables::metrics::SliderMetricTable> for SliderInspectMetrics {
    fn from(table: crate::tables::metrics::SliderMetricTable) -> Self {
        Self {
            width: table.width,
            height: table.height,
            track_height: table.track_height,
            thumb_size: table.thumb_size,
            radius: table.radius,
        }
    }
}
