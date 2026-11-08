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

use xgpui_luma::controls::resizable_panels::ResizablePanelsLook;
use xgpui_luma::theme::InteractionLayer;
use xgpui_luma::theme::InteractionState;
use xgpui_luma::theme::ThemeMode;

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
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

pub fn resizable_panels_look(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
) -> ResizablePanelsLook {
    let ctx = LookContext::new(mode, theme_mode, state);
    let state = ctx.state;
    let resolver =
        LookResolver::new(ctx.catalog(), ctx.theme_mode, "resizable_panels").with_stylesheet(mode.stylesheet());
    let colors = resolve_resizable_panels_colors(&resolver, state.disabled, state.layer())
        .unwrap_or_else(|_| ResizablePanelsColorTable::fallback());

    let geometry = mode.stylesheet().common.resizable_panels.resolve_geometry("md", Default::default());

    ResizablePanelsLook {
        border: colors.border.hsla(),
        border_width: geometry.border_width,
        divider_width: geometry.divider_width,
        divider: colors.divider.hsla(),
        grip: colors.grip.hsla(),
        grip_emphasis: colors.grip_emphasis.hsla(),
        disabled_opacity: if state.disabled { geometry.disabled_opacity } else { 1.0 },
    }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use xgpui_luma::theme::ThemeMode;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::resizable_panels_look;

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
        ]))
    }

    #[test]
    fn enabled_resizable_panels_use_border_and_accent_grip() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = resizable_panels_look(&mode, ThemeMode::Light, Default::default());
        assert_eq!(look.border, catalog.color("border").expect("border"));
        assert_eq!(look.grip_emphasis, catalog.color("accent").expect("accent"));
    }
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
    handle_size: xgpui_luma::controls::resizable_panels::ResizeHandleSize,
    stylesheet: &xgpui_luma::theme::stylesheet::CommonStylesheet,
) -> ResizablePanelsMetricTable {
    use crate::tables::metrics::helpers::inspect_shared_metric;

    let metrics = handle_size.metrics_with_stylesheet(stylesheet);
    let size_label = match handle_size {
        xgpui_luma::controls::resizable_panels::ResizeHandleSize::Sm => "sm",
        xgpui_luma::controls::resizable_panels::ResizeHandleSize::Md => "md",
        xgpui_luma::controls::resizable_panels::ResizeHandleSize::Lg => "lg",
    };

    let geometry = stylesheet.resizable_panels.inspect_geometry(
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
