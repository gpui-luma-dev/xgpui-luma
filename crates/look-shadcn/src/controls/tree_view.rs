//! Tree view row tokens (shadcn / tweakcn sidebar branch styling):
//!
//! | Part           | Token                              |
//! |----------------|------------------------------------|
//! | Row label      | `sidebar-foreground`               |
//! | Row hover bg   | `sidebar-accent` (layer)         |
//! | Row pressed bg | `accent` (layer)                   |
//! | Selected row bg / fg | `primary` / `primary-foreground` |
//! | Disabled label | `muted-foreground`                 |
//! | Icon / chevron | `sidebar-foreground`               |

use crate::stylesheet::config::matches_optional_layer;
use serde::Deserialize;

use xgpui_luma::controls::tree_view::TreeViewPalette;
use xgpui_luma::theme::ControlSize;
use xgpui_luma::theme::InteractionLayer;
use xgpui_luma::theme::InteractionState;

use super::apply_button_metrics_typography;

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::LookResolver;
use crate::provenance::ResolvedColor;
use crate::stylesheet::StylesheetConfig;
use crate::stylesheet::find_tree_view_row_color_rule;
use crate::stylesheet::resolve_tree_view_row_color_rule;

#[derive(Clone, Debug)]
pub struct TreeViewRowColorTable {
    pub foreground: ResolvedColor,
    pub icon_color: ResolvedColor,
    pub chevron_color: ResolvedColor,
    pub background: Option<ResolvedColor>,
}

impl TreeViewRowColorTable {
    pub fn fallback() -> Self {
        Self {
            foreground: ResolvedColor::fallback_foreground(),
            icon_color: ResolvedColor::fallback_foreground(),
            chevron_color: ResolvedColor::fallback_foreground(),
            background: None,
        }
    }
}

pub fn resolve_tree_view_row_colors(
    resolver: &LookResolver<'_>,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<TreeViewRowColorTable> {
    resolve_tree_view_row_colors_with_stylesheet(resolver, resolver.stylesheet(), disabled, layer)
}

pub fn resolve_tree_view_row_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<TreeViewRowColorTable> {
    let rule = find_tree_view_row_color_rule(stylesheet, disabled, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching tree view row color rule"))?;
    let colors = resolve_tree_view_row_color_rule(resolver, rule, layer)?;
    Ok(TreeViewRowColorTable {
        foreground: colors.foreground,
        icon_color: colors.icon_color,
        chevron_color: colors.chevron_color,
        background: colors.background,
    })
}

pub fn tree_view_row_palette(
    mode: &ShadcnModeTokens,
    selected: bool,
    state: InteractionState,
    size: ControlSize,
) -> TreeViewPalette {
    let ctx = LookContext::new(mode, mode.theme_mode, state);
    let typography = ctx.typography();
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "tree_view_row").with_stylesheet(mode.stylesheet());
    let colors = resolve_tree_view_row_colors(&resolver, state.disabled, state.layer())
        .unwrap_or_else(|_| TreeViewRowColorTable::fallback());

    let mut row_typography = typography.text.label;
    apply_button_metrics_typography(&mut row_typography, mode, size);

    // Selection persists independently of hover and the active keyboard row.
    // Keep the matching foreground on labels and icons for themed contrast.
    let selected = selected && !state.disabled;
    let foreground = if selected {
        mode.palette.selected_foreground
    } else {
        colors.foreground.hsla()
    };

    TreeViewPalette {
        background: if selected {
            Some(mode.palette.selected_background)
        } else {
            colors.background.map(|color| color.hsla())
        },
        foreground,
        icon_color: if selected { foreground } else { colors.icon_color.hsla() },
        chevron_color: if selected {
            foreground
        } else {
            colors.chevron_color.hsla()
        },
        typography: row_typography,
        font_family: typography.font.sans.family.clone().into(),
    }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use xgpui_luma::theme::{ControlSize, InteractionLayer, ThemeMode};

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use crate::provenance::LookResolver;
    use super::{resolve_tree_view_row_colors, tree_view_row_palette};

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("secondary".into(), "oklch(0.6437 0.1019 187.3840)".into()),
            ("secondary-foreground".into(), "oklch(1 0 0)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("accent-foreground".into(), "oklch(1 0 0)".into()),
            ("background".into(), "oklch(0.9735 0.0261 90.0953)".into()),
            ("foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("muted".into(), "oklch(0.6979 0.0159 196.7940)".into()),
            ("muted-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("sidebar-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("sidebar-accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
        ]))
    }

    #[test]
    fn tree_view_hovered_row_uses_sidebar_accent_background() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let palette = tree_view_row_palette(
            &mode,
            false,
            xgpui_luma::theme::InteractionState { hovered: true, ..Default::default() },
            ControlSize::Md,
        );
        let resolver = LookResolver::new(&catalog, ThemeMode::Light, "test");
        let expected = resolve_tree_view_row_colors(&resolver, false, InteractionLayer::Hovered)
            .expect("row colors")
            .background
            .map(|color| color.hsla());
        assert_eq!(palette.background, expected);
    }

    #[test]
    fn tree_view_selection_remains_visible_without_hover_or_focus_in_both_modes() {
        for theme_mode in [ThemeMode::Light, ThemeMode::Dark] {
            let mode = ShadcnModeTokens::from_catalog(sample_catalog(), theme_mode).expect("catalog");
            for state in [
                xgpui_luma::theme::InteractionState::default(),
                xgpui_luma::theme::InteractionState { hovered: true, ..Default::default() },
                xgpui_luma::theme::InteractionState { pressed: true, ..Default::default() },
            ] {
                let selected = tree_view_row_palette(&mode, true, state, ControlSize::Md);
                assert_eq!(selected.background, Some(mode.palette.selected_background));
                assert_eq!(selected.foreground, mode.palette.selected_foreground);
                assert_eq!(selected.icon_color, selected.foreground);
                assert_eq!(selected.chevron_color, selected.foreground);
            }
            let idle = tree_view_row_palette(&mode, false, Default::default(), ControlSize::Md);
            assert_eq!(idle.background, None);
            let disabled = xgpui_luma::theme::InteractionState { disabled: true, ..Default::default() };
            let selected = tree_view_row_palette(&mode, true, disabled, ControlSize::Md);
            let unselected = tree_view_row_palette(&mode, false, disabled, ControlSize::Md);
            assert_eq!(selected.background, unselected.background);
            assert_eq!(selected.foreground, unselected.foreground);
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct TreeViewStylesheet {
    #[serde(default)]
    pub row: TreeViewRowStylesheet,
}

impl TreeViewStylesheet {
    /// Native bundled row rules in matching order.
    pub fn native_default() -> Self {
        Self {
            row: TreeViewRowStylesheet {
                color_rules: [
                    (true, None, "muted-foreground", None),
                    (false, Some("default"), "first(sidebar-foreground,foreground)", None),
                    (false, Some("disabled"), "first(sidebar-foreground,foreground)", None),
                    (
                        false,
                        Some("hover"),
                        "first(sidebar-foreground,foreground)",
                        Some("first_layer(sidebar-accent,accent)"),
                    ),
                    (false, Some("pressed"), "first(sidebar-foreground,foreground)", Some("first_layer(accent)")),
                ]
                .into_iter()
                .map(|(disabled, layer, color, background)| TreeViewRowColorRule {
                    disabled: Some(disabled),
                    layer: layer.map(Into::into),
                    foreground: color.into(),
                    icon_color: color.into(),
                    chevron_color: color.into(),
                    background: background.map(Into::into),
                })
                .collect(),
            },
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct TreeViewRowStylesheet {
    #[serde(default)]
    pub color_rules: Vec<TreeViewRowColorRule>,
}

impl TreeViewRowStylesheet {
    pub fn find_color_rule(&self, disabled: bool, layer: InteractionLayer) -> Option<&TreeViewRowColorRule> {
        self.color_rules.iter().find(|rule| {
            if rule.disabled == Some(true) {
                return disabled;
            }
            if disabled {
                return false;
            }
            matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct TreeViewRowColorRule {
    pub disabled: Option<bool>,
    pub layer: Option<String>,
    pub foreground: String,
    pub icon_color: String,
    pub chevron_color: String,
    pub background: Option<String>,
}

#[derive(Clone, Debug)]
pub struct TreeViewMetricTable {
    pub row_height: crate::ResolvedMetric,
    pub base_padding_x: crate::ResolvedMetric,
    pub indentation_width: crate::ResolvedMetric,
    pub inner_gap: crate::ResolvedMetric,
    pub radius: crate::ResolvedMetric,
    pub icon_size: crate::ResolvedMetric,
    pub chevron_size: crate::ResolvedMetric,
}

pub fn resolve_tree_view_metrics(
    mode: &crate::ShadcnModeTokens,
    theme_mode: xgpui_luma::theme::ThemeMode,
    size: xgpui_luma::theme::ControlSize,
) -> TreeViewMetricTable {
    use xgpui_luma::controls::tree_view::TreeViewScale;

    use crate::catalog::SpacingField;
    use crate::tables::metrics::helpers::{control_size_key, derived_metric, radius_metric, spacing_control_metric};

    let ctx = crate::LookContext::new(mode, theme_mode, xgpui_luma::theme::InteractionState::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let scale = TreeViewScale::compute_with_stylesheet(size, metrics, 1.0, &mode.stylesheet().common);
    let size_key = control_size_key(size);

    let mut table = TreeViewMetricTable {
        row_height: derived_metric(format!("{size_key} tree row height from configured scale"), scale.row_height),
        base_padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, scale.base_padding_x),
        indentation_width: derived_metric("tree indentation width", scale.indentation_width),
        inner_gap: spacing_control_metric(catalog, size, SpacingField::Gap, scale.inner_gap),
        radius: radius_metric(catalog, size, scale.radius),
        icon_size: derived_metric("tree icon size", scale.icon_size),
        chevron_size: derived_metric("tree chevron size", scale.chevron_size),
    };
    let geometry = mode.stylesheet().common.tree_view.inspect_geometry(
        size_key,
        xgpui_luma::theme::stylesheet::TreeViewGeometry {
            row_height: scale.row_height,
            base_padding_x: scale.base_padding_x,
            indentation_width: scale.indentation_width,
            inner_gap: scale.inner_gap,
            radius: scale.radius,
            icon_size: scale.icon_size,
            chevron_size: scale.chevron_size,
            ..Default::default()
        },
    );
    table.row_height = crate::tables::metrics::helpers::prefer_shared_metric(geometry.row_height, table.row_height);
    table.row_height.value_px = scale.row_height;
    table.base_padding_x =
        crate::tables::metrics::helpers::prefer_shared_metric(geometry.base_padding_x, table.base_padding_x);
    table.base_padding_x.value_px = scale.base_padding_x;
    table.indentation_width =
        crate::tables::metrics::helpers::prefer_shared_metric(geometry.indentation_width, table.indentation_width);
    table.indentation_width.value_px = scale.indentation_width;
    table.inner_gap = crate::tables::metrics::helpers::prefer_shared_metric(geometry.inner_gap, table.inner_gap);
    table.inner_gap.value_px = scale.inner_gap;
    table.radius = crate::tables::metrics::helpers::prefer_shared_metric(geometry.radius, table.radius);
    table.radius.value_px = scale.radius;
    table.icon_size = crate::tables::metrics::helpers::prefer_shared_metric(geometry.icon_size, table.icon_size);
    table.icon_size.value_px = scale.icon_size;
    table.chevron_size =
        crate::tables::metrics::helpers::prefer_shared_metric(geometry.chevron_size, table.chevron_size);
    table.chevron_size.value_px = scale.chevron_size;
    table
}
