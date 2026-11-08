//! Inspect metadata for `stepper`.

use xgpui_luma::theme::ThemeMode;
use crate::{LookContext, LookResolver, ResolvedColor, ShadcnModeTokens};

pub struct StepperInspectPalette {
    pub complete_bg: ResolvedColor,
    pub complete_fg: ResolvedColor,
    pub in_progress_bg: ResolvedColor,
    pub in_progress_fg: ResolvedColor,
    pub incomplete_bg: ResolvedColor,
    pub incomplete_border: ResolvedColor,
    pub incomplete_fg: ResolvedColor,
    pub track_active: ResolvedColor,
    pub track_muted: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct StepperInspectMetrics {
    pub step_badge_size: crate::ResolvedMetric,
    pub track_thickness: crate::ResolvedMetric,
}

pub fn inspect_stepper_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    enabled: bool,
) -> StepperInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, xgpui_luma::theme::InteractionState::default());
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "stepper_inspect").with_stylesheet(mode.stylesheet());
    let palette = crate::tables::resolve_stepper_colors(&resolver, enabled)
        .unwrap_or_else(|_| crate::tables::StepperColorTable::fallback());
    StepperInspectPalette {
        complete_bg: palette.complete_bg,
        complete_fg: palette.complete_fg,
        in_progress_bg: palette.in_progress_bg,
        in_progress_fg: palette.in_progress_fg,
        incomplete_bg: palette.incomplete_bg,
        incomplete_border: palette.incomplete_border,
        incomplete_fg: palette.incomplete_fg,
        track_active: palette.track_active,
        track_muted: palette.track_muted,
    }
}

pub fn inspect_stepper_metrics(mode: &ShadcnModeTokens, _theme_mode: ThemeMode) -> StepperInspectMetrics {
    let table = crate::tables::metrics::resolve_stepper_metrics(mode, _theme_mode);
    table.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspect::test_support::sample_catalog;

    use crate::ColorSource;

    #[test]
    fn inspect_enabled_stepper_uses_progress_color_elements() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let palette = inspect_stepper_color_palette(&mode, ThemeMode::Light, true);
        assert!(matches!(
            palette.complete_bg.source,
            ColorSource::CssVar { ref token } if token == "accent-foreground"
        ));
        assert!(matches!(
            palette.complete_fg.source,
            ColorSource::CssVar { ref token } if token == "accent"
        ));
        assert!(matches!(
            palette.track_active.source,
            ColorSource::CssVar { ref token } if token == "accent-foreground"
        ));
        assert!(matches!(
            palette.track_muted.source,
            ColorSource::CssVar { ref token } if token == "accent"
        ));
    }

    #[test]
    fn inspect_stepper_metrics_include_badge_and_track() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let metrics = inspect_stepper_metrics(&mode, ThemeMode::Light);
        assert!(metrics.step_badge_size.value_px > 0.0);
        assert!(metrics.track_thickness.value_px > 0.0);
    }
}

impl From<crate::tables::metrics::StepperMetricTable> for StepperInspectMetrics {
    fn from(table: crate::tables::metrics::StepperMetricTable) -> Self {
        Self { step_badge_size: table.step_badge_size, track_thickness: table.track_thickness }
    }
}
