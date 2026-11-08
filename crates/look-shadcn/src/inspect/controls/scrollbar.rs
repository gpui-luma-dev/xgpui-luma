//! Inspect metadata for `scrollbar`.

use xgpui_luma::theme::{InteractionState, ThemeMode};
use crate::{LookContext, LookResolver, ResolvedColor, ShadcnModeTokens};

use xgpui_luma::controls::scrollbar::{ScrollbarOrientation, ScrollbarStyle};

pub struct ScrollbarInspectPalette {
    pub track_background: ResolvedColor,
    pub thumb_background: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ScrollbarInspectMetrics {
    pub length: crate::ResolvedMetric,
    pub thickness: crate::ResolvedMetric,
    pub track_thickness: crate::ResolvedMetric,
    pub thumb_thickness: crate::ResolvedMetric,
    pub min_thumb_length: crate::ResolvedMetric,
    pub radius: crate::ResolvedMetric,
}

pub fn inspect_scrollbar_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ScrollbarStyle,
    state: InteractionState,
) -> ScrollbarInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, state);
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "scrollbar_inspect").with_stylesheet(mode.stylesheet());
    let colors = crate::tables::resolve_scrollbar_colors(&resolver, style, state.disabled, state.layer())
        .unwrap_or_else(|_| crate::tables::ScrollbarColorTable::fallback());

    ScrollbarInspectPalette { track_background: colors.track_background, thumb_background: colors.thumb_background }
}

pub fn inspect_scrollbar_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    orientation: ScrollbarOrientation,
    style: ScrollbarStyle,
) -> ScrollbarInspectMetrics {
    let table = crate::tables::metrics::resolve_scrollbar_metrics(mode, theme_mode, orientation, style);
    table.into()
}

impl From<crate::tables::metrics::ScrollbarMetricTable> for ScrollbarInspectMetrics {
    fn from(table: crate::tables::metrics::ScrollbarMetricTable) -> Self {
        Self {
            length: table.length,
            thickness: table.thickness,
            track_thickness: table.track_thickness,
            thumb_thickness: table.thumb_thickness,
            min_thumb_length: table.min_thumb_length,
            radius: table.radius,
        }
    }
}
