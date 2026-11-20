//! Split view property mappings:
//!
//! | Part              | Token              |
//! |-------------------|--------------------|
//! | Separator         | `border`           |
//! | Separator (hover) | `border-hover`     |
//! | Disabled          | `muted-foreground` |

use crate::stylesheet::config::EnabledColorRule;
use crate::stylesheet::config::find_enabled_color_rule;
use serde::Deserialize;

use crate::provenance::LookResolver;
use crate::provenance::ResolvedColor;
use crate::stylesheet::StylesheetConfig;
use crate::stylesheet::find_split_view_color_rule;
use crate::stylesheet::resolve_split_view_color_rule;

#[derive(Clone, Debug)]
pub struct SplitViewColorTable {
    pub separator: ResolvedColor,
    pub separator_hover: ResolvedColor,
}

impl SplitViewColorTable {
    pub fn fallback() -> Self {
        Self { separator: ResolvedColor::transparent(), separator_hover: ResolvedColor::fallback_foreground() }
    }
}

pub fn resolve_split_view_colors(resolver: &LookResolver<'_>, enabled: bool) -> anyhow::Result<SplitViewColorTable> {
    resolve_split_view_colors_with_stylesheet(resolver, resolver.stylesheet(), enabled)
}

pub fn resolve_split_view_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    enabled: bool,
) -> anyhow::Result<SplitViewColorTable> {
    let rule = find_split_view_color_rule(stylesheet, enabled)
        .ok_or_else(|| anyhow::anyhow!("no matching split view color rule"))?;
    let colors = resolve_split_view_color_rule(resolver, rule)?;
    Ok(SplitViewColorTable { separator: colors.separator, separator_hover: colors.separator_hover })
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct SplitViewStylesheet {
    #[serde(default)]
    pub color_rules: Vec<SplitViewColorRule>,
}

impl SplitViewStylesheet {
    /// Native bundled separator colors.
    pub fn native_default() -> Self {
        Self {
            color_rules: [(true, "border", "border-hover"), (false, "muted-foreground", "muted-foreground")]
                .into_iter()
                .map(|(enabled, separator, hover)| SplitViewColorRule {
                    enabled: Some(enabled),
                    separator: separator.into(),
                    separator_hover: hover.into(),
                })
                .collect(),
        }
    }

    pub fn find_color_rule(&self, enabled: bool) -> Option<&SplitViewColorRule> {
        find_enabled_color_rule(&self.color_rules, enabled)
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct SplitViewColorRule {
    pub enabled: Option<bool>,
    pub separator: String,
    pub separator_hover: String,
}

impl EnabledColorRule for SplitViewColorRule {
    fn enabled(&self) -> Option<bool> {
        self.enabled
    }
}

#[derive(Clone, Debug)]
pub struct SplitViewMetricTable {
    pub separator_hitbox_width: crate::ResolvedMetric,
    pub separator_cue_width: crate::ResolvedMetric,
    pub separator_cue_hovered_width: crate::ResolvedMetric,
    pub separator_cue_radius: crate::ResolvedMetric,
    pub separator_cue_inset_y: crate::ResolvedMetric,
}

pub fn resolve_split_view_metrics() -> SplitViewMetricTable {
    resolve_split_view_metrics_with_stylesheet(xgpui_luma::theme::stylesheet::CommonStylesheet::sdk_default())
}

/// Resolve separator inspection from the same selected stylesheet used for paint.
pub fn resolve_split_view_metrics_with_stylesheet(
    stylesheet: &xgpui_luma::theme::stylesheet::CommonStylesheet,
) -> SplitViewMetricTable {
    use crate::tables::metrics::helpers::derived_metric;
    let geometry = stylesheet.split_view.resolve_geometry(Default::default());

    SplitViewMetricTable {
        separator_hitbox_width: derived_metric("split view separator hitbox width", geometry.hitbox_width),
        separator_cue_width: derived_metric("split view separator cue width", geometry.cue_width),
        separator_cue_hovered_width: derived_metric(
            "split view separator cue hovered width",
            geometry.cue_hovered_width,
        ),
        separator_cue_radius: derived_metric("split view separator cue radius", geometry.cue_radius),
        separator_cue_inset_y: derived_metric("split view separator cue inset y", geometry.cue_inset_y),
    }
}
