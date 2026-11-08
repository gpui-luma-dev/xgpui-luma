//! Inspect metadata for `tree_view`.

use xgpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use crate::{LookContext, LookResolver, ResolvedColor, ResolvedMetric, ShadcnModeTokens};

pub struct TreeViewRowInspectPalette {
    pub background: Option<ResolvedColor>,
    pub foreground: ResolvedColor,
    pub icon_color: ResolvedColor,
    pub chevron_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct TreeViewInspectMetrics {
    pub row_height: ResolvedMetric,
    pub base_padding_x: ResolvedMetric,
    pub indentation_width: ResolvedMetric,
    pub inner_gap: ResolvedMetric,
    pub radius: ResolvedMetric,
    pub icon_size: ResolvedMetric,
    pub chevron_size: ResolvedMetric,
}

pub fn inspect_tree_view_row_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> TreeViewRowInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, state);
    let resolver =
        LookResolver::new(ctx.catalog(), theme_mode, "tree_view_row_inspect").with_stylesheet(mode.stylesheet());
    let colors = crate::tables::resolve_tree_view_row_colors(&resolver, state.disabled, state.layer())
        .unwrap_or_else(|_| crate::tables::TreeViewRowColorTable::fallback());
    TreeViewRowInspectPalette {
        background: colors.background,
        foreground: colors.foreground,
        icon_color: colors.icon_color,
        chevron_color: colors.chevron_color,
    }
}

pub fn inspect_tree_view_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> TreeViewInspectMetrics {
    let table = crate::tables::metrics::resolve_tree_view_metrics(mode, theme_mode, size);
    table.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspect::test_support::sample_catalog;

    #[test]
    fn tree_view_metadata_covers_row_table() {
        assert_eq!(crate::stylesheet::resolve_tree_view_row_colors_metadata(crate::embedded_stylesheet()).len(), 5);
    }

    #[test]
    fn inspect_disabled_row_uses_muted_foreground() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let palette = inspect_tree_view_row_color_palette(
            &mode,
            ThemeMode::Light,
            xgpui_luma::theme::InteractionState { disabled: true, ..Default::default() },
        );
        assert!(palette.background.is_none());
        assert!(matches!(
            palette.foreground.source,
            crate::ColorSource::CssVar { ref token } if token == "muted-foreground"
        ));
    }
}

impl From<crate::tables::metrics::TreeViewMetricTable> for TreeViewInspectMetrics {
    fn from(table: crate::tables::metrics::TreeViewMetricTable) -> Self {
        Self {
            row_height: table.row_height,
            base_padding_x: table.base_padding_x,
            indentation_width: table.indentation_width,
            inner_gap: table.inner_gap,
            radius: table.radius,
            icon_size: table.icon_size,
            chevron_size: table.chevron_size,
        }
    }
}
