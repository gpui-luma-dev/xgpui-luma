//! Inspect metadata for `tabs`.

#[derive(Clone, Debug)]
pub struct TabsInspectMetrics {
    pub list_radius: crate::ResolvedMetric,
    pub list_gap: crate::ResolvedMetric,
    pub list_padding: crate::ResolvedMetric,
    pub item_padding_x: crate::ResolvedMetric,
    pub item_height: crate::ResolvedMetric,
    pub item_radius: crate::ResolvedMetric,
    pub indicator_height: crate::ResolvedMetric,
}

impl From<crate::tables::metrics::TabsMetricTable> for TabsInspectMetrics {
    fn from(table: crate::tables::metrics::TabsMetricTable) -> Self {
        Self {
            list_radius: table.list_radius,
            list_gap: table.list_gap,
            list_padding: table.list_padding,
            item_padding_x: table.item_padding_x,
            item_height: table.item_height,
            item_radius: table.item_radius,
            indicator_height: table.indicator_height,
        }
    }
}
