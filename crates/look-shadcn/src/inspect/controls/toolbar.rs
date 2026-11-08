//! Inspect metadata for `toolbar`.

use xgpui_luma::controls::toolbar::ToolbarVariant;
use xgpui_luma::theme::{ControlSize, ThemeMode};
use crate::{ResolvedColor, ResolvedMetric, ShadcnModeTokens};

#[derive(Clone, Debug)]
pub struct ToolbarInspectPalette {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
    pub separator: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ToolbarInspectMetrics {
    pub radius: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub gap: ResolvedMetric,
    pub separator_height: ResolvedMetric,
}

pub fn inspect_toolbar_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    enabled: bool,
    variant: ToolbarVariant,
) -> ToolbarInspectPalette {
    let colors = crate::tables::resolve_toolbar_colors(mode, theme_mode, enabled, variant);
    ToolbarInspectPalette { background: colors.background, border: colors.border, separator: colors.separator }
}

pub fn inspect_toolbar_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> ToolbarInspectMetrics {
    let table = crate::tables::metrics::resolve_toolbar_metrics(mode, theme_mode, size);
    table.into()
}

impl From<crate::tables::metrics::ToolbarMetricTable> for ToolbarInspectMetrics {
    fn from(table: crate::tables::metrics::ToolbarMetricTable) -> Self {
        Self {
            radius: table.radius,
            padding_x: table.padding_x,
            padding_y: table.padding_y,
            gap: table.gap,
            separator_height: table.separator_height,
        }
    }
}
