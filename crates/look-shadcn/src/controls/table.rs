//! Table — input surface + accent whisper hover, muted selected rows.
//!
//! | Row state   | Token / effect        |
//! |-------------|-----------------------|
//! | Default     | transparent           |
//! | Hover       | `accent` at 40% alpha |
//! | Selected    | `muted`               |
//! | Keyboard active | `muted`           |

use crate::stylesheet::config::EnabledColorRule;
use crate::stylesheet::config::find_enabled_color_rule;
use crate::stylesheet::config::matches_optional_layer;
use serde::Deserialize;
use std::collections::HashMap;

use crate::size::ShadcnSize;
use xgpui_luma::theme::InteractionLayer;

use crate::provenance::ResolvedColor;

#[derive(Clone, Debug)]
pub struct TableSurfaceColorTable {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
    pub header_background: ResolvedColor,
    pub header_label_color: ResolvedColor,
}

impl TableSurfaceColorTable {
    pub fn fallback() -> Self {
        Self {
            background: ResolvedColor::transparent(),
            border: ResolvedColor::fallback_foreground(),
            header_background: ResolvedColor::transparent(),
            header_label_color: ResolvedColor::fallback_foreground(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TableRowColorTable {
    pub background: ResolvedColor,
    pub label_color: ResolvedColor,
    pub divider: ResolvedColor,
}

impl TableRowColorTable {
    pub fn fallback() -> Self {
        Self {
            background: ResolvedColor::transparent(),
            label_color: ResolvedColor::fallback_foreground(),
            divider: ResolvedColor::fallback_foreground(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct TableStylesheet {
    #[serde(default)]
    pub surface: TableSurfaceStylesheet,
    #[serde(default)]
    pub row: TableRowStylesheet,
}

impl TableStylesheet {
    /// Native bundled surface and row rules in priority order.
    pub fn native_default() -> Self {
        Self {
            surface: TableSurfaceStylesheet {
                metrics: HashMap::from([(
                    "md".into(),
                    TableSurfaceMetricsRule {
                        padding_y_factor: 0.5,
                        radius: "radius.md".into(),
                        max_radius: Some("1rem".into()),
                    },
                )]),
                color_rules: [(true, "background"), (false, "muted")]
                    .into_iter()
                    .map(|(enabled, background)| TableSurfaceColorRule {
                        enabled: Some(enabled),
                        background: background.into(),
                        border: "input".into(),
                        header_background: "muted".into(),
                        header_label_color: "muted-foreground".into(),
                    })
                    .collect(),
            },
            row: TableRowStylesheet {
                color_rules: [
                    (Some(true), None, None, None, "transparent", "muted-foreground"),
                    (None, Some(true), None, None, "muted", "foreground"),
                    (None, None, Some(true), None, "muted", "foreground"),
                    (None, None, None, Some("hover"), "@accent_whisper_40", "foreground"),
                    (None, None, None, Some("pressed"), "@accent_whisper_pressed_40", "foreground"),
                    (None, None, None, Some("default"), "transparent", "foreground"),
                ]
                .into_iter()
                .map(|(disabled, selected, focused, layer, background, label)| TableRowColorRule {
                    disabled,
                    selected,
                    focused,
                    layer: layer.map(Into::into),
                    background: background.into(),
                    label_color: label.into(),
                    divider: "border".into(),
                })
                .collect(),
            },
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct TableSurfaceStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, TableSurfaceMetricsRule>,
    #[serde(default)]
    pub color_rules: Vec<TableSurfaceColorRule>,
}

impl TableSurfaceStylesheet {
    pub fn find_color_rule(&self, enabled: bool) -> Option<&TableSurfaceColorRule> {
        find_enabled_color_rule(&self.color_rules, enabled)
    }

    pub fn metrics_for_size(&self, size: ShadcnSize) -> Option<&TableSurfaceMetricsRule> {
        let key = match size {
            ShadcnSize::Sm => "sm",
            ShadcnSize::Md => "md",
            ShadcnSize::Lg => "lg",
        };
        self.metrics.get(key)
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct TableSurfaceMetricsRule {
    pub padding_y_factor: f32,
    pub radius: String,
    #[serde(default)]
    pub max_radius: Option<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct TableSurfaceColorRule {
    pub enabled: Option<bool>,
    pub background: String,
    pub border: String,
    pub header_background: String,
    pub header_label_color: String,
}

impl EnabledColorRule for TableSurfaceColorRule {
    fn enabled(&self) -> Option<bool> {
        self.enabled
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct TableRowStylesheet {
    #[serde(default)]
    pub color_rules: Vec<TableRowColorRule>,
}

impl TableRowStylesheet {
    pub fn find_color_rule(
        &self,
        selected: bool,
        focused: bool,
        disabled: bool,
        layer: InteractionLayer,
    ) -> Option<&TableRowColorRule> {
        self.color_rules.iter().find(|rule| {
            if rule.disabled == Some(true) {
                return disabled;
            }
            if disabled {
                return false;
            }
            if rule.selected == Some(true) {
                return selected;
            }
            if rule.focused == Some(true) {
                return focused;
            }
            if selected || focused {
                return false;
            }
            matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct TableRowColorRule {
    pub disabled: Option<bool>,
    pub selected: Option<bool>,
    pub focused: Option<bool>,
    pub layer: Option<String>,
    pub background: String,
    pub label_color: String,
    pub divider: String,
}
