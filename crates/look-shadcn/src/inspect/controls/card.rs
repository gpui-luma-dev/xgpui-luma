//! Inspect metadata for `card`.

use xgpui_luma::theme::ControlSize;
use crate::{ResolvedMetric, ShadcnLook};

#[derive(Clone, Debug)]
pub struct CardInspectMetrics {
    pub padding: ResolvedMetric,
    pub section_gap: ResolvedMetric,
    pub header_gap: ResolvedMetric,
    pub body_gap: ResolvedMetric,
    pub radius: ResolvedMetric,
}

pub fn inspect_card_metrics(look: &ShadcnLook, size: ControlSize) -> CardInspectMetrics {
    let table = crate::tables::metrics::resolve_card_metrics(look, size);
    table.into()
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
