//! Inspect metadata for `control_group`.

use xgpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use crate::{LookContext, LookResolver, ResolvedColor, ResolvedMetric, ShadcnModeTokens};

pub struct ControlGroupListInspectPalette {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct ControlGroupInspectMetrics {
    pub radius: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub gap: ResolvedMetric,
}

pub fn inspect_control_group_list_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    enabled: bool,
) -> ControlGroupListInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let resolver =
        LookResolver::new(ctx.catalog(), theme_mode, "control_group_list_inspect").with_stylesheet(mode.stylesheet());
    let colors = crate::tables::resolve_control_group_list_colors(&resolver, enabled)
        .unwrap_or_else(|_| crate::tables::ControlGroupListColorTable::fallback());
    ControlGroupListInspectPalette { background: colors.background, border: colors.border }
}

pub fn inspect_control_group_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> ControlGroupInspectMetrics {
    let table = crate::tables::metrics::resolve_control_group_metrics(mode, theme_mode, size);
    table.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspect::test_support::sample_catalog;

    #[test]
    fn control_group_metadata_covers_enabled_and_disabled() {
        assert_eq!(
            crate::stylesheet::resolve_control_group_list_colors_metadata(crate::embedded_stylesheet()).len(),
            2
        );
    }

    #[test]
    fn inspect_disabled_list_uses_muted_foreground_background() {
        let catalog = sample_catalog();
        let muted_foreground = catalog.color("muted-foreground").expect("muted-foreground");
        let mode = ShadcnModeTokens::from_catalog(catalog, ThemeMode::Light).expect("catalog");
        let palette = inspect_control_group_list_color_palette(&mode, ThemeMode::Light, false);
        assert_eq!(palette.background.value, muted_foreground);
    }
}

impl From<crate::tables::metrics::ControlGroupMetricTable> for ControlGroupInspectMetrics {
    fn from(table: crate::tables::metrics::ControlGroupMetricTable) -> Self {
        Self { radius: table.radius, padding_x: table.padding_x, padding_y: table.padding_y, gap: table.gap }
    }
}
