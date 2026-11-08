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

use xgpui_luma::controls::accordion::AccordionContentPalette;
use xgpui_luma::controls::accordion::AccordionPalette;
use xgpui_luma::theme::ControlSize;
use xgpui_luma::theme::InteractionLayer;
use xgpui_luma::theme::InteractionState;
use xgpui_luma::theme::ThemeMode;

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::LookResolver;
use crate::provenance::ResolvedColor;
use crate::stylesheet::StylesheetConfig;
use crate::stylesheet::find_accordion_content_color_rule;
use crate::stylesheet::find_accordion_trigger_color_rule;
use crate::stylesheet::resolve_accordion_content_color_rule;
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

pub fn resolve_accordion_content_colors(
    resolver: &LookResolver<'_>,
    expanded: bool,
) -> anyhow::Result<AccordionContentColorTable> {
    resolve_accordion_content_colors_with_stylesheet(resolver, resolver.stylesheet(), expanded)
}

pub fn resolve_accordion_content_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    expanded: bool,
) -> anyhow::Result<AccordionContentColorTable> {
    let rule = find_accordion_content_color_rule(stylesheet, expanded)
        .ok_or_else(|| anyhow::anyhow!("no matching accordion content color rule"))?;
    let colors = resolve_accordion_content_color_rule(resolver, rule)?;
    Ok(AccordionContentColorTable { foreground: colors.foreground, background: colors.background })
}

pub fn accordion_trigger_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
    size: ControlSize,
) -> AccordionPalette {
    let ctx = LookContext::new(mode, theme_mode, state);
    let state = ctx.state;
    let typography = crate::tables::typography::resolve_control_typography(mode, size, false);
    let resolver =
        LookResolver::new(ctx.catalog(), ctx.theme_mode, "accordion_trigger").with_stylesheet(mode.stylesheet());
    let colors = resolve_accordion_trigger_colors(&resolver, state.disabled, state.layer())
        .unwrap_or_else(|_| AccordionTriggerColorTable::fallback());

    AccordionPalette {
        background: colors.background.map(|color| color.hsla()),
        foreground: colors.foreground.hsla(),
        border_color: colors.border_color.hsla(),
        icon_color: colors.icon_color.hsla(),
        chevron_color: colors.chevron_color.hsla(),
        typography: typography.style,
        font_family: typography.font_family,
    }
}

pub fn accordion_content_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    expanded: bool,
) -> AccordionContentPalette {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let resolver =
        LookResolver::new(ctx.catalog(), ctx.theme_mode, "accordion_content").with_stylesheet(mode.stylesheet());
    let colors = resolve_accordion_content_colors(&resolver, expanded)
        .unwrap_or_else(|_| AccordionContentColorTable::fallback());

    AccordionContentPalette {
        background: colors.background.map(|color| color.hsla()),
        foreground: colors.foreground.hsla(),
    }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use xgpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::accordion_trigger_palette;

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
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
        ]))
    }

    #[test]
    fn trigger_uses_paired_accent_hover_like_sidebar() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let default = accordion_trigger_palette(&mode, ThemeMode::Light, InteractionState::default(), ControlSize::Md);
        let hovered = accordion_trigger_palette(
            &mode,
            ThemeMode::Light,
            InteractionState { hovered: true, ..InteractionState::default() },
            ControlSize::Md,
        );

        assert_eq!(default.foreground, catalog.color("foreground").expect("foreground"));
        assert_eq!(default.chevron_color, catalog.color("muted-foreground").expect("muted"));
        assert!(default.background.is_none());
        assert_eq!(hovered.background, Some(catalog.color("accent").expect("accent")));
        assert_eq!(hovered.foreground, catalog.color("accent-foreground").expect("accent-foreground"));
        assert_eq!(hovered.icon_color, hovered.foreground);
        assert_eq!(hovered.chevron_color, hovered.foreground);
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

pub fn resolve_accordion_metrics(
    mode: &crate::ShadcnModeTokens,
    theme_mode: xgpui_luma::theme::ThemeMode,
    size: xgpui_luma::theme::ControlSize,
    scale_factor: f32,
) -> AccordionMetricTable {
    use xgpui_luma::controls::accordion::AccordionScale;

    use crate::catalog::SpacingField;
    use crate::tables::metrics::helpers::{
        control_size_key, derived_metric, radius_metric, scaffold_control_metric, spacing_control_metric,
    };

    let ctx = crate::LookContext::new(mode, theme_mode, xgpui_luma::theme::InteractionState::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let scale = AccordionScale::compute_with_stylesheet(size, metrics, scale_factor, &mode.stylesheet().common);
    let size_key = control_size_key(size);

    let mut table = AccordionMetricTable {
        disabled_opacity: derived_metric("configured disabled trigger opacity", scale.disabled_opacity),
        drag_handle_padding: derived_metric("configured drag handle padding", scale.drag_handle_padding),
        trigger_height: scaffold_control_metric(size_key, "control_height", scale.trigger_height),
        padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, scale.padding_x),
        padding_y: spacing_control_metric(catalog, size, SpacingField::PaddingY, scale.padding_y),
        content_padding_y: derived_metric(
            format!("{size_key} content padding y from configured scale"),
            scale.content_padding_y,
        ),
        radius: radius_metric(catalog, size, scale.radius),
        item_gap: derived_metric("accordion item gap", scale.item_gap),
        inner_gap: spacing_control_metric(catalog, size, SpacingField::Gap, scale.inner_gap),
        icon_size: derived_metric("accordion icon size", scale.icon_size),
        chevron_size: derived_metric("accordion chevron size", scale.chevron_size),
    };
    let geometry = mode.stylesheet().common.accordion.inspect_geometry(
        size_key,
        xgpui_luma::theme::stylesheet::AccordionGeometry {
            trigger_height: scale.trigger_height,
            padding_x: scale.padding_x,
            padding_y: scale.padding_y,
            content_padding_y: scale.content_padding_y,
            radius: scale.radius,
            item_gap: scale.item_gap,
            inner_gap: scale.inner_gap,
            icon_size: scale.icon_size,
            chevron_size: scale.chevron_size,
            ..Default::default()
        },
    );
    table.trigger_height =
        crate::tables::metrics::helpers::prefer_shared_metric(geometry.trigger_height, table.trigger_height);
    table.trigger_height.value_px = scale.trigger_height;
    table.padding_x = crate::tables::metrics::helpers::prefer_shared_metric(geometry.padding_x, table.padding_x);
    table.padding_x.value_px = scale.padding_x;
    table.padding_y = crate::tables::metrics::helpers::prefer_shared_metric(geometry.padding_y, table.padding_y);
    table.padding_y.value_px = scale.padding_y;
    table.content_padding_y =
        crate::tables::metrics::helpers::prefer_shared_metric(geometry.content_padding_y, table.content_padding_y);
    table.content_padding_y.value_px = scale.content_padding_y;
    table.radius = crate::tables::metrics::helpers::prefer_shared_metric(geometry.radius, table.radius);
    table.radius.value_px = scale.radius;
    table.item_gap = crate::tables::metrics::helpers::prefer_shared_metric(geometry.item_gap, table.item_gap);
    table.item_gap.value_px = scale.item_gap;
    table.inner_gap = crate::tables::metrics::helpers::prefer_shared_metric(geometry.inner_gap, table.inner_gap);
    table.inner_gap.value_px = scale.inner_gap;
    table.icon_size = crate::tables::metrics::helpers::prefer_shared_metric(geometry.icon_size, table.icon_size);
    table.icon_size.value_px = scale.icon_size;
    table.chevron_size =
        crate::tables::metrics::helpers::prefer_shared_metric(geometry.chevron_size, table.chevron_size);
    table.chevron_size.value_px = scale.chevron_size;
    table
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
