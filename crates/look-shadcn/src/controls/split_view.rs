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

use xgpui_luma::controls::split_view::SplitViewLook;
use xgpui_luma::theme::InteractionState;
use xgpui_luma::theme::ThemeMode;

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
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

pub fn split_view_look(mode: &ShadcnModeTokens, theme_mode: ThemeMode, _hovered: bool, enabled: bool) -> SplitViewLook {
    let state = if enabled {
        InteractionState::default()
    } else {
        InteractionState { disabled: true, ..InteractionState::default() }
    };
    let ctx = LookContext::new(mode, theme_mode, state);
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "split_view").with_stylesheet(mode.stylesheet());
    let colors = resolve_split_view_colors(&resolver, enabled).unwrap_or_else(|_| SplitViewColorTable::fallback());

    SplitViewLook {
        geometry: mode.stylesheet().common.split_view.resolve_geometry("", Default::default()),
        separator: colors.separator.hsla(),
        separator_hover: colors.separator_hover.hsla(),
    }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use xgpui_luma::theme::ThemeMode;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::split_view_look;

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
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
        ]))
    }

    #[test]
    fn enabled_split_view_uses_border_tokens() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = split_view_look(&mode, ThemeMode::Light, false, true);
        assert_eq!(look.separator, catalog.color("border").expect("border"));
    }
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
    let geometry = stylesheet.split_view.resolve_geometry("", Default::default());

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
