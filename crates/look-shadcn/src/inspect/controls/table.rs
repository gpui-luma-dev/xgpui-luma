//! Inspect metadata for `table`.

use xgpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use crate::{LookContext, LookResolver, ResolvedColor, ResolvedMetric, ShadcnModeTokens};

pub struct TableInspectPalette {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
    pub header_background: ResolvedColor,
    pub header_label_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct TableRowInspectPalette {
    pub background: ResolvedColor,
    pub label_color: ResolvedColor,
    pub divider: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct TableInspectMetrics {
    pub radius: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub row_min_height: ResolvedMetric,
    pub row_padding_x: ResolvedMetric,
    pub row_padding_y: ResolvedMetric,
}

pub fn inspect_table_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    enabled: bool,
) -> TableInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "table_inspect").with_stylesheet(mode.stylesheet());
    let colors = crate::tables::resolve_table_surface_colors(&resolver, enabled)
        .unwrap_or_else(|_| crate::tables::TableSurfaceColorTable::fallback());
    TableInspectPalette {
        background: colors.background,
        border: colors.border,
        header_background: colors.header_background,
        header_label_color: colors.header_label_color,
    }
}

pub fn inspect_table_row_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    selected: bool,
    state: InteractionState,
) -> TableRowInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, state);
    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "table_row_inspect").with_stylesheet(mode.stylesheet());
    let colors =
        crate::tables::resolve_table_row_colors(&resolver, selected, state.focused, state.disabled, state.layer())
            .unwrap_or_else(|_| crate::tables::TableRowColorTable::fallback());
    TableRowInspectPalette { background: colors.background, label_color: colors.label_color, divider: colors.divider }
}

pub fn inspect_table_metrics(mode: &ShadcnModeTokens, theme_mode: ThemeMode, size: ControlSize) -> TableInspectMetrics {
    let table = crate::tables::metrics::resolve_table_metrics(mode, theme_mode, size);
    table.into()
}

impl From<crate::tables::metrics::TableMetricTable> for TableInspectMetrics {
    fn from(table: crate::tables::metrics::TableMetricTable) -> Self {
        Self {
            radius: table.radius,
            padding_x: table.padding_x,
            padding_y: table.padding_y,
            row_min_height: table.row_min_height,
            row_padding_x: table.row_padding_x,
            row_padding_y: table.row_padding_y,
        }
    }
}
