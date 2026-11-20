//! Resizable panels property mappings:
//!
//! | Part          | Token              |
//! |---------------|--------------------|
//! | Divider line  | `border`           |
//! | Panel border  | `border`           |
//! | Handle grip   | `border`           |
//! | Grip emphasis | `accent` (layer)   |
//! | Disabled      | `muted-foreground` |

use crate::stylesheet::config::matches_optional_bool;
use crate::stylesheet::config::matches_optional_layer;
use serde::Deserialize;

use xgpui_luma::theme::InteractionLayer;

use crate::provenance::LookResolver;
use crate::provenance::ResolvedColor;
use crate::stylesheet::StylesheetConfig;
use crate::stylesheet::find_resizable_panels_color_rule;
use crate::stylesheet::resolve_resizable_panels_color_rule;

#[derive(Clone, Debug)]
pub struct ResizablePanelsColorTable {
    pub border: ResolvedColor,
    pub divider: ResolvedColor,
    pub grip: ResolvedColor,
    pub grip_emphasis: ResolvedColor,
}

impl ResizablePanelsColorTable {
    pub fn fallback() -> Self {
        Self {
            border: ResolvedColor::fallback_foreground(),
            divider: ResolvedColor::fallback_foreground(),
            grip: ResolvedColor::fallback_foreground(),
            grip_emphasis: ResolvedColor::fallback_foreground(),
        }
    }
}

pub fn resolve_resizable_panels_colors(
    resolver: &LookResolver<'_>,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<ResizablePanelsColorTable> {
    resolve_resizable_panels_colors_with_stylesheet(resolver, resolver.stylesheet(), disabled, layer)
}

pub fn resolve_resizable_panels_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<ResizablePanelsColorTable> {
    let rule = find_resizable_panels_color_rule(stylesheet, disabled, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching resizable panels color rule"))?;
    let colors = resolve_resizable_panels_color_rule(resolver, rule, layer)?;
    Ok(ResizablePanelsColorTable {
        border: colors.border,
        divider: colors.divider,
        grip: colors.grip,
        grip_emphasis: colors.grip_emphasis,
    })
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct ResizablePanelsStylesheet {
    #[serde(default)]
    pub color_rules: Vec<ResizablePanelsColorRule>,
}

impl ResizablePanelsStylesheet {
    /// Native bundled border and grip rules.
    pub fn native_default() -> Self {
        Self {
            color_rules: [
                (true, None, "muted-foreground", "muted-foreground", "muted-foreground"),
                (false, Some("default"), "border", "border", "accent"),
                (false, Some("hover"), "border", "border", "first_layer(accent)"),
                (false, Some("pressed"), "border", "border", "first_layer(accent)"),
                (false, Some("disabled"), "border", "border", "accent"),
            ]
            .into_iter()
            .map(|(disabled, layer, divider, grip, emphasis)| ResizablePanelsColorRule {
                disabled: Some(disabled),
                layer: layer.map(Into::into),
                border: "border".into(),
                divider: divider.into(),
                grip: grip.into(),
                grip_emphasis: emphasis.into(),
            })
            .collect(),
        }
    }

    pub fn find_color_rule(&self, disabled: bool, layer: InteractionLayer) -> Option<&ResizablePanelsColorRule> {
        self.color_rules.iter().find(|rule| {
            matches_optional_bool(rule.disabled, disabled) && matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ResizablePanelsColorRule {
    pub disabled: Option<bool>,
    pub layer: Option<String>,
    pub border: String,
    pub divider: String,
    pub grip: String,
    pub grip_emphasis: String,
}

#[derive(Clone, Debug)]
pub struct ResizablePanelsMetricTable {
    pub lane_px: crate::ResolvedMetric,
    pub hit_target_px: crate::ResolvedMetric,
    pub grip_cross_axis_px: crate::ResolvedMetric,
    pub grip_main_axis_px: crate::ResolvedMetric,
}

pub fn resolve_resizable_panels_metrics(
    handle_size: crate::ShadcnSize,
    stylesheet: &xgpui_luma::theme::stylesheet::CommonStylesheet,
) -> ResizablePanelsMetricTable {
    use crate::tables::metrics::helpers::inspect_shared_metric;

    let size_label = handle_size.key();
    let metrics = xgpui_luma::controls::resizable_panels::ResizeHandleMetrics::from_stylesheet(
        &stylesheet.with_size_key(size_label),
    );

    let geometry = stylesheet.resizable_panels.inspect_geometry_for(
        size_label,
        xgpui_luma::theme::stylesheet::ResizablePanelsGeometry {
            lane_width: metrics.lane_px,
            hit_target: metrics.hit_target_px,
            grip_cross_axis: metrics.grip_cross_axis_px,
            grip_main_axis: metrics.grip_main_axis_px,
            ..Default::default()
        },
    );
    ResizablePanelsMetricTable {
        lane_px: inspect_shared_metric(geometry.lane_width),
        hit_target_px: inspect_shared_metric(geometry.hit_target),
        grip_cross_axis_px: inspect_shared_metric(geometry.grip_cross_axis),
        grip_main_axis_px: inspect_shared_metric(geometry.grip_main_axis),
    }
}
