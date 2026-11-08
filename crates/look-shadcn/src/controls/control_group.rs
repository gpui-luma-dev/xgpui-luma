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

use xgpui_luma::controls::control_group::ControlGroupListLook;
use xgpui_luma::theme::ControlSize;
use xgpui_luma::theme::InteractionState;

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
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

pub fn control_group_list_look(mode: &ShadcnModeTokens, enabled: bool) -> ControlGroupListLook {
    let ctx = LookContext::new(mode, mode.theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let resolver =
        LookResolver::new(ctx.catalog(), ctx.theme_mode, "control_group_list").with_stylesheet(mode.stylesheet());
    let colors = resolve_control_group_list_colors(&resolver, enabled)
        .unwrap_or_else(|_| ControlGroupListColorTable::fallback());

    let baseline = xgpui_luma::theme::stylesheet::CommonStylesheet::sdk_default().control_group.authored_geometry("md");
    let recipe = mode.stylesheet().common.control_group.authored_geometry("md").with_fallback(&baseline);
    ControlGroupListLook {
        background: colors.background.hsla(),
        border: colors.border.hsla(),
        radius: recipe.radius.unwrap_or(metrics.radius(ControlSize::Md)),
        padding_x: recipe.padding_x.unwrap_or_default(),
        padding_y: recipe.padding_y.unwrap_or_default(),
        gap: recipe.gap.unwrap_or_default(),
        vertical_gap: recipe.vertical_gap.unwrap_or_default(),
        border_width: recipe.border_width.unwrap_or_default(),
        disabled_opacity: recipe.disabled_opacity.unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use xgpui_luma::theme::ThemeMode;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::control_group_list_look;

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("secondary".into(), "oklch(0.6437 0.1019 187.3840)".into()),
            ("secondary-foreground".into(), "oklch(1 0 0)".into()),
            ("background".into(), "oklch(0.9735 0.0261 90.0953)".into()),
            ("foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("muted".into(), "oklch(0.6979 0.0159 196.7940)".into()),
            ("muted-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
        ]))
    }

    #[test]
    fn control_group_uses_muted_surface_and_border() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = control_group_list_look(&mode, true);

        assert_eq!(look.background, catalog.color("muted").expect("muted"));
        assert_eq!(look.border, catalog.color("border").expect("border"));
    }
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

pub fn resolve_control_group_metrics(
    mode: &crate::ShadcnModeTokens,
    theme_mode: xgpui_luma::theme::ThemeMode,
    size: xgpui_luma::theme::ControlSize,
) -> ControlGroupMetricTable {
    use crate::tables::metrics::helpers::{derived_metric, radius_metric};

    let ctx = crate::LookContext::new(mode, theme_mode, xgpui_luma::theme::InteractionState::default());
    let catalog = ctx.catalog();
    let look = crate::paint::control_group_list_look(mode, true);

    let mut table = ControlGroupMetricTable {
        radius: radius_metric(catalog, size, look.radius),
        padding_x: derived_metric("control group padding x", look.padding_x),
        padding_y: derived_metric("control group padding y", look.padding_y),
        gap: derived_metric("control group gap", look.gap),
    };
    let geometry = mode.stylesheet().common.control_group.inspect_geometry(
        "md",
        xgpui_luma::theme::stylesheet::ControlGroupGeometry {
            padding_x: table.padding_x.value_px,
            padding_y: table.padding_y.value_px,
            gap: table.gap.value_px,
            radius: table.radius.value_px,
            ..Default::default()
        },
    );
    table.padding_x = crate::tables::metrics::helpers::prefer_shared_metric(geometry.padding_x, table.padding_x);
    table.padding_y = crate::tables::metrics::helpers::prefer_shared_metric(geometry.padding_y, table.padding_y);
    table.gap = crate::tables::metrics::helpers::prefer_shared_metric(geometry.gap, table.gap);
    table.radius = crate::tables::metrics::helpers::prefer_shared_metric(geometry.radius, table.radius);

    table
}
