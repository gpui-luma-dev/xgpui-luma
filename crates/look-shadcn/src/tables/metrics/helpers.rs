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
