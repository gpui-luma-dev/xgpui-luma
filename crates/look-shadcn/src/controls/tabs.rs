//! Tabs navigation property mappings:
//!
//! | Part              | Token                         |
//! |-------------------|-------------------------------|
//! | Inactive label    | `foreground`                  |
//! | Active label      | `foreground`                 |
//! | Active indicator  | `primary` / `ring` on focus   |
//! | Disabled label    | `muted-foreground`            |
//! | Disabled list bg  | `muted`                       |

use crate::stylesheet::config::EnabledColorRule;
use crate::stylesheet::config::find_enabled_color_rule;
use crate::stylesheet::config::matches_optional_bool;
use crate::stylesheet::config::matches_optional_layer;
use serde::Deserialize;

use xgpui_luma::controls::tabs::TabsItemLook;
use xgpui_luma::controls::tabs::TabsListLook;
use xgpui_luma::theme::ControlSize;
use xgpui_luma::theme::InteractionLayer;
use xgpui_luma::theme::InteractionState;

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::LookResolver;
use crate::provenance::ResolvedColor;
use crate::stylesheet::StylesheetConfig;
use crate::stylesheet::find_tabs_item_color_rule;
use crate::stylesheet::find_tabs_list_color_rule;
use crate::stylesheet::resolve_tabs_item_color_rule;
use crate::stylesheet::resolve_tabs_list_color_rule;

#[derive(Clone, Debug)]
pub struct TabsListColorTable {
    pub disabled_background: ResolvedColor,
}

impl TabsListColorTable {
    pub fn fallback() -> Self {
        Self { disabled_background: ResolvedColor::transparent() }
    }
}

pub fn resolve_tabs_list_colors(resolver: &LookResolver<'_>, enabled: bool) -> anyhow::Result<TabsListColorTable> {
    resolve_tabs_list_colors_with_stylesheet(resolver, resolver.stylesheet(), enabled)
}

pub fn resolve_tabs_list_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    enabled: bool,
) -> anyhow::Result<TabsListColorTable> {
    let rule = find_tabs_list_color_rule(stylesheet, enabled)
        .ok_or_else(|| anyhow::anyhow!("no matching tabs navigation list color rule"))?;
    let colors = resolve_tabs_list_color_rule(resolver, rule)?;
    Ok(TabsListColorTable { disabled_background: colors.disabled_background })
}

#[derive(Clone, Debug)]
pub struct TabsItemColorTable {
    pub label_color: ResolvedColor,
    pub indicator: Option<ResolvedColor>,
}

impl TabsItemColorTable {
    pub fn fallback() -> Self {
        Self { label_color: ResolvedColor::fallback_foreground(), indicator: None }
    }
}

pub fn resolve_tabs_item_colors(
    resolver: &LookResolver<'_>,
    active: bool,
    layer: InteractionLayer,
    focused: bool,
) -> anyhow::Result<TabsItemColorTable> {
    resolve_tabs_item_colors_with_stylesheet(resolver, resolver.stylesheet(), active, layer, focused)
}

pub fn resolve_tabs_item_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    active: bool,
    layer: InteractionLayer,
    focused: bool,
) -> anyhow::Result<TabsItemColorTable> {
    let rule = find_tabs_item_color_rule(stylesheet, active, layer, focused)
        .ok_or_else(|| anyhow::anyhow!("no matching tabs navigation item color rule"))?;
    let colors = resolve_tabs_item_color_rule(resolver, rule, layer)?;
    Ok(TabsItemColorTable { label_color: colors.label_color, indicator: colors.indicator })
}

pub fn tabs_list_look(mode: &ShadcnModeTokens, enabled: bool, size: ControlSize) -> TabsListLook {
    tabs_list_look_with_stylesheet(mode, mode.stylesheet(), enabled, size)
}

pub fn tabs_list_look_with_stylesheet(
    mode: &ShadcnModeTokens,
    stylesheet: &StylesheetConfig,
    enabled: bool,
    size: ControlSize,
) -> TabsListLook {
    let ctx = LookContext::new(mode, mode.theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "tabs_list").with_stylesheet(stylesheet);
    let colors = resolve_tabs_list_colors_with_stylesheet(&resolver, stylesheet, enabled)
        .unwrap_or_else(|_| TabsListColorTable::fallback());

    let geometry = crate::tables::metrics::resolve_common_tabs_geometry(mode, stylesheet, size);
    TabsListLook {
        border_width: metrics.border_width.default,
        background: if enabled {
            None
        } else {
            Some(colors.disabled_background.hsla())
        },
        border: None,
        radius: metrics.radius(size),
        padding: geometry.list_padding.value_px,
        gap: geometry.list_gap.value_px,
    }
}

pub fn tabs_item_look(
    mode: &ShadcnModeTokens,
    active: bool,
    state: InteractionState,
    size: ControlSize,
) -> TabsItemLook {
    tabs_item_look_with_stylesheet(mode, mode.stylesheet(), active, state, size)
}

pub fn tabs_item_look_with_stylesheet(
    mode: &ShadcnModeTokens,
    stylesheet: &StylesheetConfig,
    active: bool,
    state: InteractionState,
    size: ControlSize,
) -> TabsItemLook {
    let ctx = LookContext::new(mode, mode.theme_mode, state);
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let layer = state.layer();
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "tabs_item").with_stylesheet(stylesheet);
    let colors = resolve_tabs_item_colors_with_stylesheet(&resolver, stylesheet, active, layer, state.focused)
        .unwrap_or_else(|_| TabsItemColorTable::fallback());
    let label_typography = match size {
        ControlSize::Sm => typography.text.caption,
        ControlSize::Md => typography.text.label,
        ControlSize::Lg => typography.text.body,
    };

    let (accessory_size, accessory_gap) = stylesheet.common.tabs.accessory_metrics("");
    TabsItemLook {
        accessory_size,
        accessory_gap,
        label_color: colors.label_color.hsla(),
        indicator: colors.indicator.map(|color| color.hsla()),
        background: None,
        label_typography,
        radius: metrics.radius(size),
        padding_x: metrics.padding_x(size),
        height: label_typography.line_height + metrics.padding_y(size) * 2.0,
        indicator_height: crate::tables::metrics::resolve_common_tabs_geometry(mode, stylesheet, size)
            .indicator_height
            .value_px,
    }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use xgpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::tabs_item_look;

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
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
        ]))
    }

    #[test]
    fn embedded_tabs_preserve_geometry_across_modes_sizes_and_states() {
        let look = crate::ShadcnLook::built_in();
        let theme = look.tabs_theme();
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let tokens = look.mode_tokens();
            for size in [ControlSize::Sm, ControlSize::Md, ControlSize::Lg] {
                for enabled in [true, false] {
                    let list = theme.resolve_list(enabled, size);
                    assert_eq!(list.padding, 0.0);
                    assert_eq!(list.gap, tokens.metrics.gap(size));
                    assert_eq!(list.radius, tokens.metrics.radius(size));
                }
                for bits in 0..16 {
                    let state = InteractionState {
                        hovered: bits & 1 != 0,
                        pressed: bits & 2 != 0,
                        focused: bits & 4 != 0,
                        disabled: bits & 8 != 0,
                        ..Default::default()
                    };
                    for active in [true, false] {
                        let item = theme.resolve_item(active, state, size);
                        assert_eq!(item.indicator_height, 3.0);
                        assert_eq!(item.padding_x, tokens.metrics.padding_x(size));
                        assert_eq!(item.radius, tokens.metrics.radius(size));
                        assert_eq!(
                            item.height,
                            item.label_typography.line_height + tokens.metrics.padding_y(size) * 2.0
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn custom_stylesheets_validate_geometry_and_missing_fields_use_sdk_fallback() {
        for value in ["-1", "nan", "inf"] {
            assert!(
                crate::stylesheet::StylesheetConfig::parse(&format!("[common.tabs.geometry]\nlist_padding = {value}"))
                    .is_err()
            );
        }
        let stylesheet = crate::stylesheet::StylesheetConfig::default();
        let look = crate::ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, stylesheet).unwrap();
        assert_eq!(look.tabs_theme().resolve_item(true, Default::default(), ControlSize::Md).indicator_height, 2.0);
    }

    #[test]
    fn active_and_inactive_tabs_use_foreground() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let active = tabs_item_look(&mode, true, InteractionState::default(), ControlSize::Md);
        let inactive = tabs_item_look(&mode, false, InteractionState::default(), ControlSize::Md);

        assert_eq!(active.label_color, catalog.color("foreground").expect("foreground"));
        assert_eq!(inactive.label_color, catalog.color("foreground").expect("foreground"));
        assert_eq!(active.indicator, Some(catalog.color("primary").expect("primary")));
        assert!(active.background.is_none());
    }

    #[test]
    fn focused_active_tab_uses_primary_indicator() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let focused = tabs_item_look(
            &mode,
            true,
            InteractionState { focused: true, ..InteractionState::default() },
            ControlSize::Md,
        );

        assert_eq!(focused.indicator, Some(catalog.color("primary").expect("primary")));
    }

    #[test]
    fn tab_typography_changes_with_size() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog, ThemeMode::Light).expect("catalog");
        let small = tabs_item_look(&mode, false, InteractionState::default(), ControlSize::Sm);
        let medium = tabs_item_look(&mode, false, InteractionState::default(), ControlSize::Md);
        let large = tabs_item_look(&mode, false, InteractionState::default(), ControlSize::Lg);

        assert!(small.label_typography.size < medium.label_typography.size);
        assert!(medium.label_typography.size < large.label_typography.size);
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct TabsStylesheet {
    #[serde(default)]
    pub list: TabsListStylesheet,
    #[serde(default)]
    pub item: TabsItemStylesheet,
}

impl TabsStylesheet {
    /// Native bundled tab rules, preserving disabled and active matching order.
    pub fn native_default() -> Self {
        Self {
            list: TabsListStylesheet {
                color_rules: [(true, "transparent"), (false, "muted")]
                    .into_iter()
                    .map(|(enabled, color)| TabsListColorRule {
                        enabled: Some(enabled),
                        disabled_background: color.into(),
                    })
                    .collect(),
            },
            item: TabsItemStylesheet {
                color_rules: [
                    (None, "disabled", None, "muted-foreground", None),
                    (Some(false), "default", None, "foreground", None),
                    (Some(false), "hover", None, "foreground", None),
                    (Some(false), "pressed", None, "foreground", None),
                    (Some(true), "default", Some(false), "foreground", Some("@primary_default")),
                    (Some(true), "hover", Some(false), "foreground", Some("@primary_layer")),
                    (Some(true), "pressed", Some(false), "foreground", Some("@primary_layer")),
                    (Some(true), "default", Some(true), "foreground", Some("@primary_default")),
                    (Some(true), "hover", Some(true), "foreground", Some("@primary_layer")),
                    (Some(true), "pressed", Some(true), "foreground", Some("@primary_layer")),
                ]
                .into_iter()
                .map(|(active, layer, focused, color, indicator)| TabsItemColorRule {
                    active,
                    layer: Some(layer.into()),
                    focused,
                    label_color: color.into(),
                    indicator: indicator.map(Into::into),
                })
                .collect(),
            },
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct TabsListStylesheet {
    #[serde(default)]
    pub color_rules: Vec<TabsListColorRule>,
}

impl TabsListStylesheet {
    pub fn find_color_rule(&self, enabled: bool) -> Option<&TabsListColorRule> {
        find_enabled_color_rule(&self.color_rules, enabled)
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct TabsListColorRule {
    pub enabled: Option<bool>,
    pub disabled_background: String,
}

impl EnabledColorRule for TabsListColorRule {
    fn enabled(&self) -> Option<bool> {
        self.enabled
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct TabsItemStylesheet {
    #[serde(default)]
    pub color_rules: Vec<TabsItemColorRule>,
}

impl TabsItemStylesheet {
    pub fn find_color_rule(&self, active: bool, layer: InteractionLayer, focused: bool) -> Option<&TabsItemColorRule> {
        self.color_rules.iter().find(|rule| {
            if rule.layer.as_deref() == Some("disabled") {
                return layer == InteractionLayer::Disabled;
            }
            if layer == InteractionLayer::Disabled {
                return false;
            }
            matches_optional_bool(rule.active, active)
                && matches_optional_layer(rule.layer.as_deref(), layer)
                && matches_optional_bool(rule.focused, focused)
        })
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct TabsItemColorRule {
    pub active: Option<bool>,
    pub layer: Option<String>,
    pub focused: Option<bool>,
    pub label_color: String,
    pub indicator: Option<String>,
}

/// Native common geometry owned by this control.
pub(crate) fn native_tabs_stylesheet() -> xgpui_luma::theme::stylesheet::TabsStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    TabsStylesheet {
        geometry: TabsGeometryStylesheet { indicator_height: Some(3.0), ..Default::default() },
        ..Default::default()
    }
}

#[derive(Clone, Debug)]
pub struct TabsMetricTable {
    pub list_radius: crate::ResolvedMetric,
    pub list_gap: crate::ResolvedMetric,
    pub list_padding: crate::ResolvedMetric,
    pub item_padding_x: crate::ResolvedMetric,
    pub item_height: crate::ResolvedMetric,
    pub item_radius: crate::ResolvedMetric,
    pub indicator_height: crate::ResolvedMetric,
}

pub(crate) fn resolve_common_tabs_geometry(
    mode: &crate::ShadcnModeTokens,
    stylesheet: &crate::stylesheet::StylesheetConfig,
    size: xgpui_luma::theme::ControlSize,
) -> xgpui_luma::theme::stylesheet::ResolvedTabsGeometry {
    use xgpui_luma::theme::stylesheet::ResolvedTabsGeometry;
    use crate::tables::metrics::helpers::spacing_control_metric;
    let defaults = xgpui_luma::controls::tabs::default_tabs_theme();
    let default_list = defaults.resolve_list(true, size);
    let default_item = defaults.resolve_item(true, xgpui_luma::theme::InteractionState::default(), size);
    stylesheet.common.tabs.inspect_geometry(
        "default",
        ResolvedTabsGeometry {
            list_gap: {
                let value = spacing_control_metric(
                    &mode.catalog,
                    size,
                    crate::catalog::SpacingField::Gap,
                    mode.metrics.gap(size),
                );
                let source = match value.source {
                    crate::MetricSource::Derived { note } => {
                        xgpui_luma::theme::provenance::MetricSource::Derived { note }
                    }
                    crate::MetricSource::Scaffold { path } => {
                        xgpui_luma::theme::provenance::MetricSource::Scaffold { path }
                    }
                    crate::MetricSource::CssVar { token } => {
                        xgpui_luma::theme::provenance::MetricSource::Authored { key: token }
                    }
                    crate::MetricSource::Constant { label } => {
                        xgpui_luma::theme::provenance::MetricSource::Constant { label }
                    }
                };
                xgpui_luma::theme::provenance::ResolvedMetric::new(value.value_px, source)
            },
            list_padding: xgpui_luma::theme::provenance::ResolvedMetric::constant(
                default_list.padding,
                "SDK tabs list padding",
            ),
            indicator_height: xgpui_luma::theme::provenance::ResolvedMetric::constant(
                default_item.indicator_height,
                "SDK tabs indicator height",
            ),
        },
    )
}

pub fn resolve_tabs_metrics(
    mode: &crate::ShadcnModeTokens,
    theme_mode: xgpui_luma::theme::ThemeMode,
    size: xgpui_luma::theme::ControlSize,
) -> TabsMetricTable {
    resolve_tabs_metrics_with_stylesheet(mode, mode.stylesheet(), theme_mode, size)
}

/// Inspect the same selected stylesheet that the tabs theme uses for painting.
pub fn resolve_tabs_metrics_with_stylesheet(
    mode: &crate::ShadcnModeTokens,
    stylesheet: &crate::stylesheet::StylesheetConfig,
    theme_mode: xgpui_luma::theme::ThemeMode,
    size: xgpui_luma::theme::ControlSize,
) -> TabsMetricTable {
    use crate::tables::metrics::helpers::{derived_metric, radius_metric, spacing_control_metric};
    use crate::catalog::SpacingField;

    let ctx = crate::LookContext::new(mode, theme_mode, xgpui_luma::theme::InteractionState::default());
    let catalog = ctx.catalog();
    let list = crate::controls::tabs::tabs_list_look_with_stylesheet(mode, stylesheet, true, size);
    let item = crate::controls::tabs::tabs_item_look_with_stylesheet(
        mode,
        stylesheet,
        true,
        xgpui_luma::theme::InteractionState::default(),
        size,
    );
    let geometry = resolve_common_tabs_geometry(mode, stylesheet, size);

    TabsMetricTable {
        list_radius: radius_metric(catalog, size, list.radius),
        list_gap: crate::tables::metrics::helpers::inspect_shared_metric(geometry.list_gap),
        list_padding: crate::tables::metrics::helpers::inspect_shared_metric(geometry.list_padding),
        item_padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, item.padding_x),
        item_height: derived_metric("label line_height + spacing.s2", item.height),
        item_radius: radius_metric(catalog, size, item.radius),
        indicator_height: crate::tables::metrics::helpers::inspect_shared_metric(geometry.indicator_height),
    }
}
