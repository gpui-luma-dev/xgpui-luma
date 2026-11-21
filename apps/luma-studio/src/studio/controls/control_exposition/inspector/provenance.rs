use xgpui_luma_look_shadcn::inspect::{ResolvedColor, format_inspect_css_key, format_inspect_provenance};

use super::occupation::InspectMetricPropertyData;
use super::schema::{InspectColorRow, InspectPropertyRow};

pub fn color_row(label: &'static str, color: &ResolvedColor) -> InspectColorRow {
    InspectColorRow {
        label: label.into(),
        value: color.value,
        source: format_inspect_css_key(&color.source),
        detail: format_inspect_provenance(&color.source),
    }
}

pub fn property_rows_from_metric_data(rows: &[InspectMetricPropertyData]) -> Vec<InspectPropertyRow> {
    rows.iter()
        .map(|row| InspectPropertyRow {
            label: row.name.to_string(),
            value: row.value.to_string(),
            source: row.source.to_string(),
            detail: row.provenance.as_ref().map(|value| value.to_string()),
        })
        .collect()
}
