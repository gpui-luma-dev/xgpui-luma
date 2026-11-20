//! Control group list chrome — muted surface + border.
//!
//! | Part            | Token                |
//! |-----------------|----------------------|
//! | Enabled bg      | `muted`              |
//! | Disabled bg     | `muted-foreground`   |
//! | Border          | `border`             |

use crate::stylesheet::config::EnabledColorRule;
use crate::stylesheet::config::find_enabled_color_rule;
use serde::Deserialize;

use crate::provenance::LookResolver;
use crate::provenance::ResolvedColor;
use crate::stylesheet::StylesheetConfig;
use crate::stylesheet::find_control_group_list_color_rule;
use crate::stylesheet::resolve_control_group_list_color_rule;

#[derive(Clone, Debug)]
pub struct ControlGroupListColorTable {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
}

impl ControlGroupListColorTable {
    pub fn fallback() -> Self {
        Self { background: ResolvedColor::transparent(), border: ResolvedColor::fallback_foreground() }
    }
}

pub fn resolve_control_group_list_colors(
    resolver: &LookResolver<'_>,
    enabled: bool,
) -> anyhow::Result<ControlGroupListColorTable> {
    resolve_control_group_list_colors_with_stylesheet(resolver, resolver.stylesheet(), enabled)
}

pub fn resolve_control_group_list_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    enabled: bool,
) -> anyhow::Result<ControlGroupListColorTable> {
    let rule = find_control_group_list_color_rule(stylesheet, enabled)
        .ok_or_else(|| anyhow::anyhow!("no matching control group list color rule"))?;
    let colors = resolve_control_group_list_color_rule(resolver, rule)?;
    Ok(ControlGroupListColorTable { background: colors.background, border: colors.border })
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct ControlGroupStylesheet {
    #[serde(default)]
    pub list: ControlGroupListStylesheet,
}

impl ControlGroupStylesheet {
    /// Native bundled list rules.
    pub fn native_default() -> Self {
        Self {
            list: ControlGroupListStylesheet {
                color_rules: [(true, "muted"), (false, "muted-foreground")]
                    .into_iter()
                    .map(|(enabled, background)| ControlGroupListColorRule {
                        enabled: Some(enabled),
                        background: background.into(),
                        border: "border".into(),
                    })
                    .collect(),
            },
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct ControlGroupListStylesheet {
    #[serde(default)]
    pub color_rules: Vec<ControlGroupListColorRule>,
}

impl ControlGroupListStylesheet {
    pub fn find_color_rule(&self, enabled: bool) -> Option<&ControlGroupListColorRule> {
        find_enabled_color_rule(&self.color_rules, enabled)
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ControlGroupListColorRule {
    pub enabled: Option<bool>,
    pub background: String,
    pub border: String,
}

impl EnabledColorRule for ControlGroupListColorRule {
    fn enabled(&self) -> Option<bool> {
        self.enabled
    }
}

#[derive(Clone, Debug)]
pub struct ControlGroupMetricTable {
    pub radius: crate::ResolvedMetric,
    pub padding_x: crate::ResolvedMetric,
    pub padding_y: crate::ResolvedMetric,
    pub gap: crate::ResolvedMetric,
}
