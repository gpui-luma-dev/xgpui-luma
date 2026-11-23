//! Source metadata for canonical runtime metric values.

use crate::size::ShadcnSize;

use crate::catalog::{CssTokenMap, SpacingField};
use crate::{MetricSource, ResolvedMetric};

pub fn control_size_key(size: ShadcnSize) -> &'static str {
    match size {
        ShadcnSize::Sm => "sm",
        ShadcnSize::Md => "md",
        ShadcnSize::Lg => "lg",
    }
}

pub fn derived_metric(note: impl Into<String>, value_px: f32) -> ResolvedMetric {
    ResolvedMetric { value_px, source: MetricSource::Derived { note: note.into() } }
}

/// A control's padding or gap, with where it comes from: the look's `.lss` (`metrics.control`, scoped
/// by size), which derives it from `--spacing` when the catalog has one.
pub fn spacing_control_metric(
    _catalog: &CssTokenMap,
    size: ShadcnSize,
    field: SpacingField,
    value_px: f32,
) -> ResolvedMetric {
    let field_label = match field {
        SpacingField::PaddingX => "padding_x",
        SpacingField::PaddingY => "padding_y",
        SpacingField::Gap => "gap",
    };
    lss_control_metric(size, field_label, value_px)
}

/// A control's corner radius, with where it comes from: the look's `.lss` (`metrics.control.radius`,
/// scoped by size), which derives it from `--radius` when the catalog has one.
pub fn radius_metric(_catalog: &CssTokenMap, size: ShadcnSize, value_px: f32) -> ResolvedMetric {
    lss_control_metric(size, "radius", value_px)
}

fn lss_control_metric(size: ShadcnSize, field: &str, value_px: f32) -> ResolvedMetric {
    let size_key = control_size_key(size);
    ResolvedMetric {
        value_px,
        source: MetricSource::Derived {
            note: format!("shadcn tokens.lss · metrics.control.{field} (size {size_key})"),
        },
    }
}

pub fn pill_radius_metric(_catalog: &CssTokenMap, value_px: f32) -> ResolvedMetric {
    ResolvedMetric { value_px, source: MetricSource::Scaffold { path: "ShadcnRadii.full".into() } }
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
