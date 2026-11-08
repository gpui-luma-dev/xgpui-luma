//! Inspect metadata for `progress`.

use xgpui_luma::theme::{InteractionState, ThemeMode};
use crate::{LookContext, LookResolver, ResolvedColor, ShadcnModeTokens};

pub struct ProgressInspectPalette {
    pub track_color: ResolvedColor,
    pub progress_color: ResolvedColor,
    pub thumb_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ProgressInspectMetrics {
    pub size: crate::ResolvedMetric,
    pub stroke_width: crate::ResolvedMetric,
    pub track_height: crate::ResolvedMetric,
    pub thumb_size: crate::ResolvedMetric,
}

pub fn inspect_progress_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    enabled: bool,
) -> ProgressInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "progress_inspect").with_stylesheet(mode.stylesheet());
    let colors = crate::tables::resolve_progress_colors(&resolver, enabled)
        .unwrap_or_else(|_| crate::tables::ProgressColorTable::fallback());
    ProgressInspectPalette {
        track_color: colors.track_color,
        progress_color: colors.progress_color.clone(),
        thumb_color: colors.progress_color,
    }
}

pub fn inspect_progress_metrics(mode: &ShadcnModeTokens, _theme_mode: ThemeMode) -> ProgressInspectMetrics {
    let table = crate::tables::metrics::resolve_progress_metrics(mode, _theme_mode);
    table.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspect::test_support::sample_catalog;
    use crate::ColorSource;

    #[test]
    fn progress_metadata_has_enabled_and_disabled_rows() {
        assert_eq!(crate::stylesheet::resolve_progress_colors_metadata(crate::embedded_stylesheet()).len(), 2);
    }

    #[test]
    fn inspect_enabled_progress_uses_accent_foreground_fill() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let palette = inspect_progress_color_palette(&mode, ThemeMode::Light, true);
        assert!(matches!(
            palette.progress_color.source,
            ColorSource::CssVar { ref token } if token == "accent-foreground"
        ));
        assert!(matches!(
            palette.track_color.source,
            ColorSource::CssVar { ref token } if token == "accent"
        ));
    }

    #[test]
    fn inspect_progress_metrics_include_linear_fields() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let metrics = inspect_progress_metrics(&mode, ThemeMode::Light);
        assert!(metrics.track_height.value_px > 0.0);
        assert!(metrics.thumb_size.value_px > 0.0);
    }
}

impl From<crate::tables::metrics::ProgressMetricTable> for ProgressInspectMetrics {
    fn from(table: crate::tables::metrics::ProgressMetricTable) -> Self {
        Self {
            size: table.size,
            stroke_width: table.stroke_width,
            track_height: table.track_height,
            thumb_size: table.thumb_size,
        }
    }
}
