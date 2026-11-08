use xgpui_luma_look_shadcn::inspect::{
    ButtonInspectElevation, ResolvedColor, ResolvedMetric, format_inspect_box_shadow_layer, format_inspect_css_key,
    format_inspect_metric_provenance, format_inspect_metric_source, format_inspect_provenance, format_metric_px,
};

use super::occupation::InspectMetricPropertyData;
use super::schema::{InspectColorRow, InspectElevationLayer, InspectElevationSnapshot, InspectPropertyRow};

pub fn color_row(label: &'static str, color: &ResolvedColor) -> InspectColorRow {
    InspectColorRow {
        label,
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

pub fn elevation_snapshot(
    elevation: &ButtonInspectElevation,
    applied_source: &'static str,
    style_source: &'static str,
) -> InspectElevationSnapshot {
    InspectElevationSnapshot {
        property_rows: vec![
            InspectPropertyRow::new("applied", if elevation.applied { "yes" } else { "no" }, applied_source),
            InspectPropertyRow::new("rule", elevation.rule_shadow.as_str(), "stylesheet - elevation_rules"),
            InspectPropertyRow::new("style", elevation.style_key.as_str(), style_source),
            InspectPropertyRow::new(
                "token",
                elevation.token.as_deref().unwrap_or("none"),
                elevation.catalog_value.as_deref().unwrap_or("catalog token"),
            ),
            metric_row("shadow projection extent", &elevation.reserved_shadow_extent),
        ],
        catalog_value: elevation.catalog_value.clone(),
        layers: elevation
            .layers
            .iter()
            .map(|layer| InspectElevationLayer { color: layer.color, display: format_inspect_box_shadow_layer(layer) })
            .collect(),
        preview_shadows: if elevation.applied {
            elevation.shadows.clone()
        } else {
            None
        },
    }
}

/// Display typography metadata without reconstructing its source in Studio.
pub fn typography_rows(
    typography: &xgpui_luma_look_shadcn::inspect::ButtonInspectTypography,
) -> Vec<InspectPropertyRow> {
    [
        ("font family", &typography.font_family),
        ("font size", &typography.font_size),
        ("font weight", &typography.font_weight),
        ("line height", &typography.line_height),
    ]
    .into_iter()
    .map(|(label, field)| InspectPropertyRow {
        label: label.into(),
        value: field.value.clone(),
        source: xgpui_luma_look_shadcn::inspect::format_inspect_typography_source(&field.source),
        detail: xgpui_luma_look_shadcn::inspect::format_inspect_typography_provenance(&field.source),
    })
    .collect()
}
