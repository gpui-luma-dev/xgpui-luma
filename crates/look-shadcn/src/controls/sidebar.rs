//! Navigation sidebar property mappings (shadcn / tweakcn sidebar tokens):
//!
//! | Part              | Token                              |
//! |-------------------|------------------------------------|
//! | Container bg      | `sidebar`                          |
//! | Container fg      | `sidebar-foreground`               |
//! | Container border  | `sidebar-border`                   |
//! | Section label     | `muted-foreground`                 |
//! | Item fg           | `sidebar-foreground`               |
//! | Item hover bg     | `accent`                           |
//! | Item hover fg     | `accent-foreground`                |
//! | Selected bg       | `sidebar-primary`                  |
//! | Selected fg       | `sidebar-primary-foreground`       |
//! | Focus ring        | `sidebar-ring`                     |

use crate::mode::ShadcnModeTokens;

/// A standalone sidebar frame's colors, from the look's `sidebar.lss` (`container_background`,
/// `container_border`), and the large radius.
pub fn sidebar_container_look(mode: &ShadcnModeTokens) -> SidebarContainerLook {
    let resolved =
        xgpui_luma::style::StyleBinding::new("sidebar", xgpui_luma::style::ThemeSource::Fixed(mode.style_theme()))
            .resolve(&[], 1.0);
    let color = |name| {
        resolved
            .root()
            .value(name)
            .and_then(xgpui_luma::style::lss::Value::as_color)
            .map(gpui::Hsla::from)
            .unwrap_or_else(gpui::transparent_black)
    };
    SidebarContainerLook {
        background: color("container_background"),
        border: color("container_border"),
        radius: mode.radii.large,
    }
}

/// Theme colors used by standalone sidebar frames, separate from navigation items.
#[derive(Clone, Copy, Debug)]
pub struct SidebarContainerLook {
    pub background: gpui::Hsla,
    pub border: gpui::Hsla,
    pub radius: f32,
}

#[derive(Clone, Debug)]
pub struct SidebarMetricTable {
    pub section_height: crate::ResolvedMetric,
    pub item_height: crate::ResolvedMetric,
    pub item_padding_x: crate::ResolvedMetric,
    pub item_gap: crate::ResolvedMetric,
    pub item_radius: crate::ResolvedMetric,
    pub item_icon_size: crate::ResolvedMetric,
}

pub fn resolve_sidebar_metrics(
    mode: &crate::ShadcnModeTokens,
    theme_mode: xgpui_luma::theme::ThemeMode,
    size: crate::ShadcnSize,
) -> SidebarMetricTable {
    use crate::tables::metrics::helpers::{derived_metric, radius_metric, spacing_control_metric};

    let ctx = crate::LookContext::new(mode, theme_mode, xgpui_luma::theme::InteractionState::default());
    let catalog = ctx.catalog();
    let size_metrics = ctx.controls().for_size(size);

    SidebarMetricTable {
        section_height: derived_metric("navigation sidebar section height", 20.0),
        item_height: derived_metric("navigation sidebar item height", 30.0),
        item_padding_x: derived_metric("navigation sidebar item padding x", 8.0),
        item_gap: spacing_control_metric(catalog, size, crate::catalog::SpacingField::Gap, size_metrics.gap),
        item_radius: radius_metric(catalog, size, ctx.controls().radius(size)),
        item_icon_size: derived_metric("navigation sidebar icon size", 16.0),
    }
}
