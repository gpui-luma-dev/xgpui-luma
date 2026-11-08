//! Progress — accent track + accent-foreground fill.

use crate::stylesheet::config::EnabledColorRule;
use crate::stylesheet::config::control_size_key;
use crate::stylesheet::config::find_enabled_color_rule;
use serde::Deserialize;
use std::collections::HashMap;

use xgpui_luma::controls::progress::ProgressLook;
use xgpui_luma::theme::ControlSize;
use xgpui_luma::theme::InteractionState;

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::ColorSource;
use crate::provenance::LookResolver;
use crate::provenance::ResolvedColor;
use crate::stylesheet::StylesheetConfig;
use crate::stylesheet::find_progress_color_rule;
use crate::stylesheet::resolve_progress_color_rule;

#[derive(Clone, Debug)]
pub struct ProgressColorTable {
    pub track_color: ResolvedColor,
    pub progress_color: ResolvedColor,
}

impl ProgressColorTable {
    pub fn fallback() -> Self {
        Self {
            track_color: ResolvedColor { value: gpui::hsla(0.0, 0.0, 0.0, 0.0), source: ColorSource::Transparent },
            progress_color: ResolvedColor::fallback_foreground(),
        }
    }
}

pub fn resolve_progress_colors(resolver: &LookResolver<'_>, enabled: bool) -> anyhow::Result<ProgressColorTable> {
    resolve_progress_colors_with_stylesheet(resolver, resolver.stylesheet(), enabled)
}

pub fn resolve_progress_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    enabled: bool,
) -> anyhow::Result<ProgressColorTable> {
    let rule = find_progress_color_rule(stylesheet, enabled)
        .ok_or_else(|| anyhow::anyhow!("no matching progress color rule"))?;
    let colors = resolve_progress_color_rule(resolver, rule)?;
    Ok(ProgressColorTable { track_color: colors.track_color, progress_color: colors.progress_color })
}

pub fn progress_look(mode: &ShadcnModeTokens, enabled: bool, size: ControlSize) -> ProgressLook {
    let ctx = LookContext::new(mode, mode.theme_mode, InteractionState::default());
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "progress").with_stylesheet(mode.stylesheet());
    let colors = resolve_progress_colors(&resolver, enabled).unwrap_or_else(|_| ProgressColorTable::fallback());
    let stylesheet = mode.stylesheet();
    let size_key = crate::tables::metrics::helpers::control_size_key(size);
    let (size, stroke_width, track_height, thumb_size) = stylesheet
        .progress
        .metrics_for_size(size)
        .map(crate::stylesheet::resolve_progress_metrics)
        .map(|metrics| (metrics.size, metrics.stroke_width, metrics.track_height, metrics.thumb_size))
        .unwrap_or_else(|| {
            let baseline = xgpui_luma::theme::stylesheet::CommonStylesheet::sdk_default()
                .progress
                .resolve_geometry(size_key, Default::default());
            (baseline.size, baseline.stroke_width, baseline.track_height, baseline.thumb_size)
        });

    let mut look = ProgressLook {
        geometry: mode.stylesheet().common.progress.resolve_geometry(size_key, Default::default()),
        track_color: colors.track_color.hsla(),
        progress_color: colors.progress_color.hsla(),
        thumb_color: colors.progress_color.hsla(),
        track_height,
        thumb_size,
        size,
        stroke_width,
    };
    let geometry = mode.stylesheet().common.progress.resolve_geometry(
        size_key,
        xgpui_luma::theme::stylesheet::ProgressGeometry {
            size: look.size,
            stroke_width: look.stroke_width,
            track_height: look.track_height,
            thumb_size: look.thumb_size,
            ..Default::default()
        },
    );
    look.size = geometry.size;
    look.stroke_width = geometry.stroke_width;
    look.track_height = geometry.track_height;
    look.thumb_size = geometry.thumb_size;

    look
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;
    use xgpui_luma::theme::ThemeMode;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::progress_look;

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("secondary".into(), "oklch(0.6437 0.1019 187.3840)".into()),
            ("secondary-foreground".into(), "oklch(1 0 0)".into()),
            ("background".into(), "oklch(0.9735 0.0261 90.0953)".into()),
            ("foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("muted".into(), "oklch(0.6979 0.0159 196.7940)".into()),
            ("muted-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("accent-foreground".into(), "oklch(1 0 0)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
        ]))
    }

    #[test]
    fn progress_uses_accent_track_and_accent_foreground_fill() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = progress_look(&mode, true, xgpui_luma::theme::ControlSize::Md);

        assert_eq!(look.track_color, catalog.color("accent").expect("accent"));
        assert_eq!(look.progress_color, catalog.color("accent-foreground").expect("accent-foreground"));
    }

    #[test]
    fn progress_size_changes_metrics() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let small = progress_look(&mode, true, xgpui_luma::theme::ControlSize::Sm);
        let medium = progress_look(&mode, true, xgpui_luma::theme::ControlSize::Md);
        let large = progress_look(&mode, true, xgpui_luma::theme::ControlSize::Lg);

        assert!(small.size < medium.size);
        assert!(medium.size < large.size);
        assert!(small.stroke_width < medium.stroke_width);
        assert!(medium.stroke_width < large.stroke_width);
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct ProgressStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, ProgressMetricsRule>,
    #[serde(default)]
    pub color_rules: Vec<ProgressColorRule>,
}

impl ProgressStylesheet {
    /// Native bundled progress colors.
    pub fn native_default() -> Self {
        Self {
            color_rules: [(true, "accent", "accent-foreground"), (false, "muted-foreground", "muted-foreground")]
                .into_iter()
                .map(|(enabled, track, progress)| ProgressColorRule {
                    enabled: Some(enabled),
                    track_color: track.into(),
                    progress_color: progress.into(),
                })
                .collect(),
            ..Default::default()
        }
    }

    pub fn metrics_for_size(&self, size: ControlSize) -> Option<&ProgressMetricsRule> {
        self.metrics.get(control_size_key(size))
    }

    pub fn find_color_rule(&self, enabled: bool) -> Option<&ProgressColorRule> {
        find_enabled_color_rule(&self.color_rules, enabled)
    }
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
pub struct ProgressMetricsRule {
    pub size: f32,
    pub stroke_width: f32,
    #[serde(default = "default_progress_track_height")]
    pub track_height: f32,
    #[serde(default = "default_progress_thumb_size")]
    pub thumb_size: f32,
}

fn default_progress_track_height() -> f32 {
    6.0
}

fn default_progress_thumb_size() -> f32 {
    16.0
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ProgressColorRule {
    pub enabled: Option<bool>,
    pub track_color: String,
    pub progress_color: String,
}

impl EnabledColorRule for ProgressColorRule {
    fn enabled(&self) -> Option<bool> {
        self.enabled
    }
}

/// Native common geometry owned by this control.
pub(crate) fn native_progress_stylesheet() -> xgpui_luma::theme::stylesheet::ProgressStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    ProgressStylesheet {
        sizes: ["sm", "md", "lg"]
            .into_iter()
            .map(|key| (key.into(), ProgressGeometryStylesheet::default()))
            .collect(),
        ..Default::default()
    }
}

#[derive(Clone, Debug)]
pub struct ProgressMetricTable {
    pub size: crate::ResolvedMetric,
    pub stroke_width: crate::ResolvedMetric,
    pub track_height: crate::ResolvedMetric,
    pub thumb_size: crate::ResolvedMetric,
}

pub fn resolve_progress_metrics(
    mode: &crate::ShadcnModeTokens,
    theme_mode: xgpui_luma::theme::ThemeMode,
) -> ProgressMetricTable {
    resolve_progress_metrics_for_size(mode, theme_mode, xgpui_luma::theme::ControlSize::Md)
}

/// Resolve the selected look's Progress dimensions and their sources for a size.
pub fn resolve_progress_metrics_for_size(
    mode: &crate::ShadcnModeTokens,
    _theme_mode: xgpui_luma::theme::ThemeMode,
    size: xgpui_luma::theme::ControlSize,
) -> ProgressMetricTable {
    use crate::tables::metrics::helpers::derived_metric;

    let look = crate::paint::progress_look(mode, true, size);

    let mut table = ProgressMetricTable {
        size: derived_metric("progress ring diameter", look.size),
        stroke_width: derived_metric("progress stroke width", look.stroke_width),
        track_height: derived_metric("progress track height", look.track_height),
        thumb_size: derived_metric("progress thumb size", look.thumb_size),
    };
    let geometry = mode.stylesheet().common.progress.inspect_geometry(
        crate::tables::metrics::helpers::control_size_key(size),
        xgpui_luma::theme::stylesheet::ProgressGeometry {
            size: table.size.value_px,
            stroke_width: table.stroke_width.value_px,
            track_height: table.track_height.value_px,
            thumb_size: table.thumb_size.value_px,
            ..Default::default()
        },
    );
    table.size = crate::tables::metrics::helpers::prefer_shared_metric(geometry.size, table.size);
    table.stroke_width =
        crate::tables::metrics::helpers::prefer_shared_metric(geometry.stroke_width, table.stroke_width);
    table.track_height =
        crate::tables::metrics::helpers::prefer_shared_metric(geometry.track_height, table.track_height);
    table.thumb_size = crate::tables::metrics::helpers::prefer_shared_metric(geometry.thumb_size, table.thumb_size);

    table
}
