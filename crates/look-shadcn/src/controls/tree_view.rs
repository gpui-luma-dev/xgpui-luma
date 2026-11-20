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

use xgpui_luma::theme::InteractionLayer;

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
