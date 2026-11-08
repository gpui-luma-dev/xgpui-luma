//! Inspect metadata for `slider`.

use xgpui_luma::theme::{InteractionState, ThemeMode};
use crate::{LookContext, LookResolver, ResolvedColor, ShadcnButtonStyle, ShadcnModeTokens};

pub struct SliderInspectPalette {
    pub track_background: ResolvedColor,
    pub fill_background: ResolvedColor,
    pub thumb_background: ResolvedColor,
    pub thumb_border: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct SliderInspectMetrics {
    pub width: crate::ResolvedMetric,
    pub height: crate::ResolvedMetric,
    pub track_height: crate::ResolvedMetric,
    pub thumb_size: crate::ResolvedMetric,
    pub radius: crate::ResolvedMetric,
}

pub fn inspect_slider_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> SliderInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, state);
    let layer = state.layer();
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "slider_inspect").with_stylesheet(mode.stylesheet());
    let colors = crate::tables::resolve_slider_colors(&resolver, ShadcnButtonStyle::Primary, layer)
        .unwrap_or_else(|_| crate::tables::SliderColorTable::fallback());

    SliderInspectPalette {
        track_background: colors.track_background,
        fill_background: colors.fill_background,
        thumb_background: colors.thumb_background,
        thumb_border: colors.thumb_border,
    }
}

pub fn inspect_slider_metrics(mode: &ShadcnModeTokens, theme_mode: ThemeMode) -> SliderInspectMetrics {
    let table = crate::tables::metrics::resolve_slider_metrics(mode, theme_mode);
    table.into()
}

impl From<crate::tables::metrics::SliderMetricTable> for SliderInspectMetrics {
    fn from(table: crate::tables::metrics::SliderMetricTable) -> Self {
        Self {
            width: table.width,
            height: table.height,
            track_height: table.track_height,
            thumb_size: table.thumb_size,
            radius: table.radius,
        }
    }
}
