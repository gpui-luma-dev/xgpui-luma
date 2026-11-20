//! Optional inspection data and formatting for Shadcn looks.
//!
//! Enable the `inspect` feature for the color-control chrome sections and provenance formatting Studio
//! shows. Control styling is inspected from `.lss`.

mod controls;
mod format;

#[cfg(test)]
mod test_support;

pub use crate::{ColorSource, MetricSource, ResolvedColor, ResolvedMetric, ResolvedTypography, TypographySource};
pub use crate::{ShadcnButtonStyle, ShadcnTextFieldStyle};
pub use format::{
    format_css_style_ref, format_inspect_css_key, format_inspect_metric_provenance, format_inspect_metric_source,
    format_inspect_provenance, format_metric_px,
};

pub use controls::{
    ColorChromeInspectSection, ColorChromeProfile, COLOR_ARC_CHROME_PROFILES, COLOR_FIELD_CHROME_PROFILES,
    COLOR_RING_CHROME_PROFILES, COLOR_SLIDER_CHROME_PROFILES, inspect_color_chrome_sections,
};
