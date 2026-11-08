//! Inspect metadata for sidebar flush theme tokens.

use xgpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use crate::{LookContext, LookResolver, ResolvedColor, ResolvedMetric, ShadcnModeTokens};

pub struct SidebarContainerInspectPalette {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct SidebarSectionInspectPalette {
    pub label_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct SidebarItemInspectPalette {
    pub background: Option<ResolvedColor>,
    pub focus_border: Option<ResolvedColor>,
    pub foreground: ResolvedColor,
    pub icon_color: ResolvedColor,
}

#[derive(Clone, Debug)]
pub struct SidebarInspectMetrics {
    pub section_height: ResolvedMetric,
    pub item_height: ResolvedMetric,
    pub item_padding_x: ResolvedMetric,
    pub item_gap: ResolvedMetric,
    pub item_radius: ResolvedMetric,
    pub item_icon_size: ResolvedMetric,
}

pub fn inspect_sidebar_container_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
) -> SidebarContainerInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let resolver =
        LookResolver::new(ctx.catalog(), theme_mode, "sidebar_container_inspect").with_stylesheet(mode.stylesheet());
    let colors = crate::tables::resolve_sidebar_container_colors(&resolver, true)
        .unwrap_or_else(|_| crate::tables::SidebarContainerColorTable::fallback());
    SidebarContainerInspectPalette {
        background: colors.background,
        foreground: colors.foreground,
        border: colors.border,
    }
}

pub fn inspect_sidebar_section_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
) -> SidebarSectionInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let resolver =
        LookResolver::new(ctx.catalog(), theme_mode, "sidebar_section_inspect").with_stylesheet(mode.stylesheet());
    let colors = crate::tables::resolve_sidebar_section_colors(&resolver, true)
        .unwrap_or_else(|_| crate::tables::SidebarSectionColorTable::fallback());
    SidebarSectionInspectPalette { label_color: colors.label_color }
}

pub fn inspect_sidebar_branch_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> SidebarItemInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, state);
    let resolver =
        LookResolver::new(ctx.catalog(), theme_mode, "sidebar_branch_inspect").with_stylesheet(mode.stylesheet());
    let colors = crate::tables::resolve_sidebar_branch_colors(&resolver, state.disabled, state.layer())
        .unwrap_or_else(|_| crate::tables::SidebarBranchColorTable::fallback());
    SidebarItemInspectPalette {
        background: colors.background,
        focus_border: crate::tables::resolve_sidebar_focus_border(ctx.catalog(), theme_mode, state),
        foreground: colors.foreground,
        icon_color: colors.icon_color,
    }
}

pub fn inspect_sidebar_item_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    selected: bool,
    state: InteractionState,
) -> SidebarItemInspectPalette {
    let ctx = LookContext::new(mode, theme_mode, state);
    let resolver =
        LookResolver::new(ctx.catalog(), theme_mode, "sidebar_item_inspect").with_stylesheet(mode.stylesheet());
    let colors = crate::tables::resolve_sidebar_item_colors(&resolver, selected, state.disabled, state.layer())
        .unwrap_or_else(|_| crate::tables::SidebarItemColorTable::fallback());
    SidebarItemInspectPalette {
        background: colors.background,
        focus_border: crate::tables::resolve_sidebar_focus_border(ctx.catalog(), theme_mode, state),
        foreground: colors.foreground,
        icon_color: colors.icon_color,
    }
}

pub fn inspect_sidebar_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: ControlSize,
) -> SidebarInspectMetrics {
    let table = crate::tables::metrics::resolve_sidebar_metrics(mode, theme_mode, size);
    table.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspect::test_support::sample_catalog;

    #[test]
    fn sidebar_container_metadata_matches_table() {
        assert_eq!(crate::stylesheet::resolve_sidebar_container_colors_metadata(crate::embedded_stylesheet()).len(), 1);
    }

    #[test]
    fn sidebar_item_metadata_matches_table() {
        assert_eq!(crate::stylesheet::resolve_sidebar_item_colors_metadata(crate::embedded_stylesheet()).len(), 10);
    }

    #[test]
    fn inspect_selected_item_uses_sidebar_primary() {
        let catalog = sample_catalog();
        let sidebar_primary = catalog.color("sidebar-primary").expect("sidebar-primary");
        let mode = ShadcnModeTokens::from_catalog(catalog, ThemeMode::Light).expect("catalog");
        let palette = inspect_sidebar_item_color_palette(&mode, ThemeMode::Light, true, InteractionState::default());
        assert_eq!(palette.background.as_ref().map(|color| color.value), Some(sidebar_primary));
    }

    #[test]
    fn inspect_container_palette_uses_sidebar_tokens() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let palette = inspect_sidebar_container_color_palette(&mode, ThemeMode::Light);
        assert_eq!(palette.background.value, catalog.color("sidebar").expect("sidebar"));
    }
}

impl From<crate::tables::metrics::SidebarMetricTable> for SidebarInspectMetrics {
    fn from(table: crate::tables::metrics::SidebarMetricTable) -> Self {
        Self {
            section_height: table.section_height,
            item_height: table.item_height,
            item_padding_x: table.item_padding_x,
            item_gap: table.item_gap,
            item_radius: table.item_radius,
            item_icon_size: table.item_icon_size,
        }
    }
}
