use xgpui_luma_look_shadcn::inspect::{
    ResolvedColor, ResolvedMetric, format_inspect_css_key, format_inspect_metric_provenance,
    format_inspect_metric_source, format_inspect_provenance, format_metric_px,
};

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

pub fn metric_row(label: &str, metric: &ResolvedMetric) -> InspectPropertyRow {
    InspectPropertyRow {
        label: label.to_owned(),
        value: format_metric_px(metric.value_px),
        source: format_inspect_metric_source(&metric.source),
        detail: format_inspect_metric_provenance(&metric.source),
    }
}

pub fn metric_properties(fields: &[(&'static str, &ResolvedMetric)]) -> Vec<InspectPropertyRow> {
    fields.iter().map(|(label, metric)| metric_row(label, metric)).collect()
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
