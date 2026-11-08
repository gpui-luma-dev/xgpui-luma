//! Inspect metadata for `floating_menu`.

use xgpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use crate::{LookContext, LookResolver, ResolvedColor, ShadcnModeTokens};

pub struct FloatingMenuInspectPalette {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
    pub item_hover_background: ResolvedColor,
    pub item_hover_foreground: ResolvedColor,
    pub item_disabled_foreground: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct FloatingMenuInspectMetrics {
    pub radius: crate::ResolvedMetric,
    pub padding: crate::ResolvedMetric,
    pub min_width: crate::ResolvedMetric,
    pub item_height: crate::ResolvedMetric,
    pub item_padding_x: crate::ResolvedMetric,
    pub item_gap: crate::ResolvedMetric,
    pub item_icon_size: crate::ResolvedMetric,
    pub item_radius: crate::ResolvedMetric,
    pub submenu_offset_x: crate::ResolvedMetric,
}

pub fn inspect_floating_menu_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    _size: ControlSize,
) -> FloatingMenuInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let resolver =
        LookResolver::new(ctx.catalog(), theme_mode, "floating_menu_inspect").with_stylesheet(mode.stylesheet());
    let colors = crate::tables::resolve_floating_menu_colors(&resolver, true)
        .unwrap_or_else(|_| crate::tables::FloatingMenuColorTable::fallback());
    FloatingMenuInspectPalette {
        background: colors.background,
        foreground: colors.foreground,
        border: colors.border,
        item_hover_background: colors.item_hover_background,
        item_hover_foreground: colors.item_hover_foreground,
        item_disabled_foreground: colors.item_disabled_foreground,
    }
}

pub fn inspect_floating_menu_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> FloatingMenuInspectMetrics {
    let table = crate::tables::metrics::resolve_floating_menu_metrics(mode, theme_mode, size);
    table.into()
}

impl From<crate::tables::metrics::FloatingMenuMetricTable> for FloatingMenuInspectMetrics {
    fn from(table: crate::tables::metrics::FloatingMenuMetricTable) -> Self {
        Self {
            radius: table.radius,
            padding: table.padding,
            min_width: table.min_width,
            item_height: table.item_height,
            item_padding_x: table.item_padding_x,
            item_gap: table.item_gap,
            item_icon_size: table.item_icon_size,
            item_radius: table.item_radius,
            submenu_offset_x: table.submenu_offset_x,
        }
    }
}
