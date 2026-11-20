//! Accordion property mappings:
//!
//! | Part              | Token                              |
//! |-------------------|------------------------------------|
//! | Trigger label     | `foreground`                       |
//! | Trigger hover bg  | `accent`                           |
//! | Trigger hover fg  | `accent-foreground` (paired)       |
//! | Disabled label    | `muted-foreground`                 |
//! | Chevron           | `muted-foreground` / hover fg      |
//! | Item border       | `border`                           |
//! | Content label     | `foreground`                       |

use crate::stylesheet::config::matches_optional_bool;
use crate::stylesheet::config::matches_optional_layer;
use serde::Deserialize;

use xgpui_luma::theme::InteractionLayer;

use crate::provenance::LookResolver;
use crate::provenance::ResolvedColor;
use crate::stylesheet::StylesheetConfig;
use crate::stylesheet::find_accordion_trigger_color_rule;
use crate::stylesheet::resolve_accordion_trigger_color_rule;

#[derive(Clone, Debug)]
pub struct AccordionTriggerColorTable {
    pub foreground: ResolvedColor,
    pub icon_color: ResolvedColor,
    pub chevron_color: ResolvedColor,
    pub border_color: ResolvedColor,
    pub background: Option<ResolvedColor>,
}

impl AccordionTriggerColorTable {
    pub fn fallback() -> Self {
        Self {
            foreground: ResolvedColor::fallback_foreground(),
            icon_color: ResolvedColor::fallback_foreground(),
            chevron_color: ResolvedColor::fallback_foreground(),
            border_color: ResolvedColor::fallback_foreground(),
            background: None,
        }
    }
}

pub fn resolve_accordion_trigger_colors(
    resolver: &LookResolver<'_>,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<AccordionTriggerColorTable> {
    resolve_accordion_trigger_colors_with_stylesheet(resolver, resolver.stylesheet(), disabled, layer)
}

pub fn resolve_accordion_trigger_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<AccordionTriggerColorTable> {
    let rule = find_accordion_trigger_color_rule(stylesheet, disabled, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching accordion trigger color rule"))?;
    let colors = resolve_accordion_trigger_color_rule(resolver, rule, layer)?;
    Ok(AccordionTriggerColorTable {
        foreground: colors.foreground,
        icon_color: colors.icon_color,
        chevron_color: colors.chevron_color,
        border_color: colors.border_color,
        background: colors.background,
    })
}

#[derive(Clone, Debug)]
pub struct AccordionContentColorTable {
    pub foreground: ResolvedColor,
    pub background: Option<ResolvedColor>,
}

impl AccordionContentColorTable {
    pub fn fallback() -> Self {
        Self { foreground: ResolvedColor::fallback_foreground(), background: None }
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct AccordionStylesheet {
    #[serde(default)]
    pub trigger: AccordionTriggerStylesheet,
    #[serde(default)]
    pub content: AccordionContentStylesheet,
}

impl AccordionStylesheet {
    /// Native bundled trigger and content rules.
    pub fn native_default() -> Self {
        Self {
            trigger: AccordionTriggerStylesheet {
                color_rules: [
                    (true, None, "muted-foreground", "muted-foreground", None),
                    (false, Some("default"), "foreground", "muted-foreground", None),
                    (false, Some("disabled"), "foreground", "muted-foreground", None),
                    (false, Some("hover"), "accent-foreground", "accent-foreground", Some("accent")),
                    (false, Some("pressed"), "accent-foreground", "accent-foreground", Some("accent")),
                ]
                .into_iter()
                .map(|(disabled, layer, color, chevron, background)| AccordionTriggerColorRule {
                    disabled: Some(disabled),
                    layer: layer.map(Into::into),
                    foreground: color.into(),
                    icon_color: color.into(),
                    chevron_color: chevron.into(),
                    border_color: "border".into(),
                    background: background.map(Into::into),
                })
                .collect(),
            },
            content: AccordionContentStylesheet {
                color_rules: [true, false]
                    .into_iter()
                    .map(|expanded| AccordionContentColorRule {
                        expanded: Some(expanded),
                        foreground: "foreground".into(),
                        background: None,
                    })
                    .collect(),
            },
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct AccordionTriggerStylesheet {
    #[serde(default)]
    pub color_rules: Vec<AccordionTriggerColorRule>,
}

impl AccordionTriggerStylesheet {
    pub fn find_color_rule(&self, disabled: bool, layer: InteractionLayer) -> Option<&AccordionTriggerColorRule> {
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
pub struct AccordionTriggerColorRule {
    pub disabled: Option<bool>,
    pub layer: Option<String>,
    pub foreground: String,
    pub icon_color: String,
    pub chevron_color: String,
    pub border_color: String,
    pub background: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct AccordionContentStylesheet {
    #[serde(default)]
    pub color_rules: Vec<AccordionContentColorRule>,
}

impl AccordionContentStylesheet {
    pub fn find_color_rule(&self, expanded: bool) -> Option<&AccordionContentColorRule> {
        self.color_rules.iter().find(|rule| matches_optional_bool(rule.expanded, expanded))
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct AccordionContentColorRule {
    pub expanded: Option<bool>,
    pub foreground: String,
    pub background: Option<String>,
}

#[derive(Clone, Debug)]
pub struct AccordionMetricTable {
    pub disabled_opacity: crate::ResolvedMetric,
    pub drag_handle_padding: crate::ResolvedMetric,
    pub trigger_height: crate::ResolvedMetric,
    pub padding_x: crate::ResolvedMetric,
    pub padding_y: crate::ResolvedMetric,
    pub content_padding_y: crate::ResolvedMetric,
    pub radius: crate::ResolvedMetric,
    pub item_gap: crate::ResolvedMetric,
    pub inner_gap: crate::ResolvedMetric,
    pub icon_size: crate::ResolvedMetric,
    pub chevron_size: crate::ResolvedMetric,
}

impl AccordionMetricTable {
    /// Converts the complete resolved table to the SDK layout contract.
    pub fn scale(&self) -> xgpui_luma::controls::accordion::AccordionScale {
        xgpui_luma::controls::accordion::AccordionScale {
            disabled_opacity: self.disabled_opacity.value_px,
            drag_handle_padding: self.drag_handle_padding.value_px,
            trigger_height: self.trigger_height.value_px,
            padding_x: self.padding_x.value_px,
            padding_y: self.padding_y.value_px,
            content_padding_y: self.content_padding_y.value_px,
            radius: self.radius.value_px,
            item_gap: self.item_gap.value_px,
            inner_gap: self.inner_gap.value_px,
            icon_size: self.icon_size.value_px,
            chevron_size: self.chevron_size.value_px,
        }
    }
}
