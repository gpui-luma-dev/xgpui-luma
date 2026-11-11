//! Source metadata for canonical runtime metric values.

use xgpui_luma::theme::ControlSize;

use crate::catalog::{CssTokenMap, SpacingField};
use crate::{MetricSource, ResolvedMetric};

pub fn control_size_key(size: ControlSize) -> &'static str {
    match size {
        ControlSize::Sm => "sm",
        ControlSize::Md => "md",
        ControlSize::Lg => "lg",
    }
}

pub fn scaffold_control_metric(size_key: &str, field: &str, value_px: f32) -> ResolvedMetric {
    ResolvedMetric {
        value_px,
        source: MetricSource::Scaffold { path: format!("MetricTokens.control.{size_key}.{field}") },
    }
}

pub fn derived_metric(note: impl Into<String>, value_px: f32) -> ResolvedMetric {
    ResolvedMetric { value_px, source: MetricSource::Derived { note: note.into() } }
}

pub fn spacing_control_metric(
    catalog: &CssTokenMap,
    size: ControlSize,
    field: SpacingField,
    value_px: f32,
) -> ResolvedMetric {
    if catalog.get("spacing").and_then(crate::catalog::parse_length_px).is_some() {
        let size_key = control_size_key(size);
        let field_label = match field {
            SpacingField::PaddingX => "padding_x",
            SpacingField::PaddingY => "padding_y",
            SpacingField::Gap => "gap",
        };
        let multiplier = crate::catalog::spacing_multiplier(size, field);
        let multiplier_label = if (multiplier - multiplier.round()).abs() < f32::EPSILON {
            format!("{}", multiplier.round() as i32)
        } else {
            format!("{multiplier}")
        };
        ResolvedMetric {
            value_px,
            source: MetricSource::Derived {
                note: format!("{size_key} {field_label} = --spacing × {multiplier_label}"),
            },
        }
    } else {
        let field_label = match field {
            SpacingField::PaddingX => "padding_x",
            SpacingField::PaddingY => "padding_y",
            SpacingField::Gap => "gap",
        };
        scaffold_control_metric(control_size_key(size), field_label, value_px)
    }
}

pub fn radius_metric(catalog: &CssTokenMap, size: ControlSize, value_px: f32) -> ResolvedMetric {
    if catalog.get("radius").and_then(crate::catalog::parse_length_px).is_some() {
        let (size_label, offset) = match size {
            ControlSize::Sm => ("sm", 4.0_f32),
            ControlSize::Md => ("md", 2.0_f32),
            ControlSize::Lg => ("lg", 0.0_f32),
        };
        let offset_label = if (offset - offset.round()).abs() < f32::EPSILON {
            format!("{}px", offset.round() as i32)
        } else {
            format!("{offset}px")
        };
        ResolvedMetric {
            value_px,
            source: MetricSource::Derived { note: format!("{size_label} = max(--radius − {offset_label}, 0px)") },
        }
    } else {
        scaffold_control_metric(control_size_key(size), "radius", value_px)
    }
}

pub fn pill_radius_metric(_catalog: &CssTokenMap, value_px: f32) -> ResolvedMetric {
    ResolvedMetric { value_px, source: MetricSource::Scaffold { path: "MetricTokens.radius.pill".into() } }
}

pub fn focus_ring_width_metric(metrics: &xgpui_luma::theme::MetricTokens) -> ResolvedMetric {
    ResolvedMetric {
        value_px: xgpui_luma::infra::focus_adorner::FocusAdorner::new(metrics.focus).width,
        source: MetricSource::Scaffold { path: "MetricTokens.focus.width".into() },
    }
}

pub fn focus_ring_offset_metric(metrics: &xgpui_luma::theme::MetricTokens) -> ResolvedMetric {
    derived_metric(
        "max(focus.gap + focus.offset, 0) + max(focus.width, 0)",
        xgpui_luma::infra::focus_adorner::FocusAdorner::new(metrics.focus).extent(),
    )
}

pub fn border_width_metric(metrics: &xgpui_luma::theme::MetricTokens) -> ResolvedMetric {
    ResolvedMetric {
        value_px: metrics.border_width.default,
        source: MetricSource::Scaffold { path: "MetricTokens.border_width.default".into() },
    }
}

// Preserve the existing Shadcn inspector API while reporting shared authored paths.
pub fn inspect_shared_metric(value: xgpui_luma::theme::provenance::ResolvedMetric) -> crate::ResolvedMetric {
    use xgpui_luma::theme::provenance::MetricSource;
    let source = match value.source {
        MetricSource::Authored { key } => crate::MetricSource::Derived { note: format!("stylesheet · {key}") },
        MetricSource::Derived { note } => crate::MetricSource::Derived { note },
        MetricSource::Scaffold { path } => crate::MetricSource::Scaffold { path },
        MetricSource::Constant { label } => crate::MetricSource::Constant { label },
        MetricSource::ScaleStep { family, step } => {
            crate::MetricSource::Derived { note: format!("{family} step {step}") }
        }
    };
    crate::ResolvedMetric { value_px: value.value_px, source }
}

/// Preserve token/recipe provenance when a common field is unset.
pub(crate) fn prefer_shared_metric(
    metric: xgpui_luma::theme::provenance::ResolvedMetric,
    fallback: ResolvedMetric,
) -> ResolvedMetric {
    if matches!(metric.source, xgpui_luma::theme::provenance::MetricSource::Constant { .. }) {
        fallback
    } else {
        inspect_shared_metric(metric)
    }
}
