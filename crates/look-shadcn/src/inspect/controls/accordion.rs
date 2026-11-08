//! Inspect metadata for `accordion`.

use xgpui_luma::theme::{InteractionState, ThemeMode};
use crate::{LookContext, LookResolver, ResolvedColor, ResolvedMetric, ShadcnModeTokens};

pub struct AccordionTriggerInspectPalette {
    pub background: Option<ResolvedColor>,
    pub foreground: ResolvedColor,
    pub border_color: ResolvedColor,
    pub icon_color: ResolvedColor,
    pub chevron_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct AccordionContentInspectPalette {
    pub background: Option<ResolvedColor>,
    pub foreground: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct AccordionInspectMetrics {
    pub trigger_height: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub content_padding_y: ResolvedMetric,
    pub radius: ResolvedMetric,
    pub item_gap: ResolvedMetric,
    pub inner_gap: ResolvedMetric,
    pub icon_size: ResolvedMetric,
    pub chevron_size: ResolvedMetric,
}

pub fn inspect_accordion_trigger_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> AccordionTriggerInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, state);
    let resolver =
        LookResolver::new(ctx.catalog(), theme_mode, "accordion_trigger_inspect").with_stylesheet(mode.stylesheet());
    let colors = crate::tables::resolve_accordion_trigger_colors(&resolver, state.disabled, state.layer())
        .unwrap_or_else(|_| crate::tables::AccordionTriggerColorTable::fallback());
    AccordionTriggerInspectPalette {
        background: colors.background,
        foreground: colors.foreground,
        border_color: colors.border_color,
        icon_color: colors.icon_color,
        chevron_color: colors.chevron_color,
    }
}

pub fn inspect_accordion_content_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    expanded: bool,
) -> AccordionContentInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let resolver =
        LookResolver::new(ctx.catalog(), theme_mode, "accordion_content_inspect").with_stylesheet(mode.stylesheet());
    let colors = crate::tables::resolve_accordion_content_colors(&resolver, expanded)
        .unwrap_or_else(|_| crate::tables::AccordionContentColorTable::fallback());
    AccordionContentInspectPalette { background: colors.background, foreground: colors.foreground }
}

pub fn inspect_accordion_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: xgpui_luma::theme::ControlSize,
) -> AccordionInspectMetrics {
    inspect_accordion_metrics_at_scale(mode, theme_mode, size, 1.0)
}

/// Inspect the same pixel-snapped geometry rendered at the window scale factor.
pub fn inspect_accordion_metrics_at_scale(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: xgpui_luma::theme::ControlSize,
    scale_factor: f32,
) -> AccordionInspectMetrics {
    let table = crate::tables::metrics::resolve_accordion_metrics(mode, theme_mode, size, scale_factor);
    table.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspect::test_support::sample_catalog;
    use crate::ColorSource;

    #[test]
    fn accordion_metadata_covers_trigger_and_content_tables() {
        assert_eq!(crate::stylesheet::resolve_accordion_trigger_colors_metadata(crate::embedded_stylesheet()).len(), 5);
        assert_eq!(crate::stylesheet::resolve_accordion_content_colors_metadata(crate::embedded_stylesheet()).len(), 2);
    }

    #[test]
    fn inspect_hovered_trigger_uses_accent_pair() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let palette = inspect_accordion_trigger_color_palette(
            &mode,
            ThemeMode::Light,
            InteractionState { hovered: true, ..InteractionState::default() },
        );
        let background = palette.background.expect("hovered trigger should paint accent background");
        assert!(matches!(background.source, ColorSource::CssVar { ref token } if token == "accent"));
        assert!(matches!(
            palette.foreground.source,
            ColorSource::CssVar { ref token } if token == "accent-foreground"
        ));
    }
}

impl From<crate::tables::metrics::AccordionMetricTable> for AccordionInspectMetrics {
    fn from(table: crate::tables::metrics::AccordionMetricTable) -> Self {
        Self {
            trigger_height: table.trigger_height,
            padding_x: table.padding_x,
            padding_y: table.padding_y,
            content_padding_y: table.content_padding_y,
            radius: table.radius,
            item_gap: table.item_gap,
            inner_gap: table.inner_gap,
            icon_size: table.icon_size,
            chevron_size: table.chevron_size,
        }
    }
}
