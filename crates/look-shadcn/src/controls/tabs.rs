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

use xgpui_luma::theme::InteractionLayer;

use crate::provenance::ResolvedColor;

#[derive(Clone, Debug)]
pub struct TabsListColorTable {
    pub disabled_background: ResolvedColor,
}

impl TabsListColorTable {
    pub fn fallback() -> Self {
        Self { disabled_background: ResolvedColor::transparent() }
    }
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

#[cfg(test)]
mod tests {

    use xgpui_luma::theme::{InteractionState, ThemeMode};

    #[test]
    fn embedded_tabs_preserve_geometry_across_modes_sizes_and_states() {
        let look = crate::ShadcnLook::built_in();
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            look.set_mode(mode);
            let tokens = look.mode_tokens();
            for size in [crate::ShadcnSize::Sm, crate::ShadcnSize::Md, crate::ShadcnSize::Lg] {
                let theme = look.with_size(size).tabs_theme();
                for enabled in [true, false] {
                    let list = theme.resolve_list(enabled);
                    assert_eq!(list.padding, 0.0);
                    assert_eq!(list.gap, tokens.control_sizes.gap(size));
                    assert_eq!(list.radius, tokens.control_sizes.radius(size));
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
                        let item = theme.resolve_item(active, state);
                        assert_eq!(item.indicator_height, 3.0);
                        assert_eq!(item.padding_x, tokens.control_sizes.padding_x(size));
                        assert_eq!(item.radius, tokens.control_sizes.radius(size));
                        assert_eq!(
                            item.height,
                            item.label_typography.line_height + tokens.control_sizes.padding_y(size) * 2.0
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn custom_stylesheets_validate_tab_geometry() {
        for value in ["-1", "nan", "inf"] {
            assert!(
                crate::stylesheet::StylesheetConfig::parse(&format!("[common.tabs.geometry]\nlist_padding = {value}"))
                    .is_err()
            );
        }
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
