//! Inspect metadata for `pager`.

use crate::{ResolvedMetric, ResolvedTypography};

#[derive(Clone, Debug)]
pub struct PagerInspectMetrics {
    pub control_height: ResolvedMetric,
    pub button_size: ResolvedMetric,
    pub button_min_width: ResolvedMetric,
    pub radius: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub gap: ResolvedMetric,
    pub group_gap: ResolvedMetric,
    pub font_family: ResolvedTypography,
    pub reserved_shadow_extent: ResolvedMetric,
}

impl From<crate::tables::metrics::PagerMetricTable> for PagerInspectMetrics {
    fn from(table: crate::tables::metrics::PagerMetricTable) -> Self {
        Self {
            control_height: table.control_height,
            button_size: table.button_size,
            button_min_width: table.button_min_width,
            radius: table.radius,
            padding_x: table.padding_x,
            padding_y: table.padding_y,
            gap: table.gap,
            group_gap: table.group_gap,
            font_family: table.font_family,
            reserved_shadow_extent: table.reserved_shadow_extent,
        }
    }
}
