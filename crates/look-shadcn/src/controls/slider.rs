//! Slider property mappings (primary / secondary action styling):
//!
//! | Part  | Token |
//! |-------|-------|
//! | Track | `border` (neutral track rail) |
//! | Fill  | primary: `@action_default` / `@action_layer`; secondary: `@action_foreground` |
//! | Thumb | `first(background,card)` |
//! | Ring  | primary: `@action_default`; secondary: `@action_foreground` |
//!
//! Track uses `border` rather than `muted` because many tweakcn light themes
//! set `--muted` near white (e.g. 98% lightness), which disappears on card panels.

use crate::stylesheet::config::{LayeredElevationRule, matches_optional_layer};
use serde::Deserialize;

use xgpui_luma::theme::{ControlSizes, InteractionLayer, ThemeMode};

use crate::controls::button::ButtonRadiusPreset;
use crate::mode::ShadcnModeTokens;
use crate::provenance::ResolvedColor;
use crate::size::ShadcnSize;
use crate::stylesheet::StylesheetConfig;

#[derive(Clone, Debug)]
pub struct SliderColorTable {
    pub track_background: ResolvedColor,
    pub fill_background: ResolvedColor,
    pub thumb_background: ResolvedColor,
    pub thumb_border: ResolvedColor,
}

impl SliderColorTable {
    pub fn fallback() -> Self {
        Self {
            track_background: ResolvedColor::transparent(),
            fill_background: ResolvedColor::fallback_foreground(),
            thumb_background: ResolvedColor::fallback_foreground(),
            thumb_border: ResolvedColor::fallback_foreground(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct SliderStylesheet {
    #[serde(default)]
    pub metrics: ControlSizes<Option<SliderMetricsRule>>,
    #[serde(default)]
    pub elevation_rules: Vec<LayeredElevationRule>,
    #[serde(default)]
    pub color_rules: Vec<SliderColorRule>,
}

impl SliderStylesheet {
    pub fn find_color_rule(&self, style: &str, layer: InteractionLayer) -> Option<&SliderColorRule> {
        self.color_rules.iter().find(|rule| {
            rule.style.as_deref().is_none_or(|value| value == style)
                && matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }

    pub fn elevation_rule_for_layer(&self, layer: InteractionLayer) -> Option<&LayeredElevationRule> {
        if layer == InteractionLayer::Disabled {
            return self.elevation_rules.iter().find(|rule| rule.layer.as_deref() == Some("disabled"));
        }
        self.elevation_rules.iter().find(|rule| rule.layer.is_none())
    }

    pub fn metrics_for_size(&self, size: ShadcnSize) -> Option<&SliderMetricsRule> {
        self.metrics.get(size.key()).and_then(Option::as_ref)
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct SliderMetricsRule {
    pub width: f32,
    pub height: f32,
    pub track_height: f32,
    pub thumb_size: f32,
    pub radius: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct SliderColorRule {
    pub style: Option<String>,
    pub layer: Option<String>,
    pub track_background: String,
    pub fill_background: String,
    pub thumb_background: String,
    pub thumb_border: String,
}

/// Constructs the native Shadcn common slider sizes in pure Rust without preloading TOML.
pub fn native_shadcn_common_slider() -> xgpui_luma::theme::stylesheet::SliderStylesheet {
    use xgpui_luma::theme::stylesheet::{SliderGeometryStylesheet, SliderStylesheet};
    SliderStylesheet {
        geometry: SliderGeometryStylesheet::default(),
        sizes: [("sm", 24.0, 4.0, 12.0), ("md", 32.0, 6.0, 16.0), ("lg", 40.0, 8.0, 20.0)]
            .into_iter()
            .map(|(key, height, track_height, thumb_size)| {
                (
                    key.to_string(),
                    SliderGeometryStylesheet {
                        width: Some(260.0),
                        height: Some(height),
                        track_height: Some(track_height),
                        thumb_size: Some(thumb_size),
                        ..Default::default()
                    },
                )
            })
            .collect(),
    }
}

/// Constructs native Shadcn slider elevation and color rules in pure Rust without preloading TOML.
pub fn native_shadcn_slider() -> SliderStylesheet {
    let elevation_rules = vec![
        LayeredElevationRule { layer: None, shadow: "shadow-sm".into() },
        LayeredElevationRule { layer: Some("disabled".into()), shadow: "none".into() },
    ];

    let mut color_rules = Vec::with_capacity(8);
    for (style, default_fill) in [("primary", "@action_default"), ("secondary", "@action_foreground")] {
        color_rules.push(SliderColorRule {
            style: Some(style.into()),
            layer: Some("disabled".into()),
            track_background: "border".into(),
            fill_background: "muted-foreground".into(),
            thumb_background: "first(background,card)".into(),
            thumb_border: "muted".into(),
        });
        for (layer, fill) in [
            ("default", default_fill),
            (
                "hover",
                if style == "primary" {
                    "@action_layer"
                } else {
                    default_fill
                },
            ),
            (
                "pressed",
                if style == "primary" {
                    "@action_layer"
                } else {
                    default_fill
                },
            ),
        ] {
            color_rules.push(SliderColorRule {
                style: Some(style.into()),
                layer: Some(layer.into()),
                track_background: "border".into(),
                fill_background: fill.into(),
                thumb_background: "first(background,card)".into(),
                thumb_border: default_fill.into(),
            });
        }
    }

    SliderStylesheet { metrics: ControlSizes::default(), elevation_rules, color_rules }
}

/// Programmatic definition of all Shadcn slider rules.
pub fn define_shadcn_slider(config: &mut StylesheetConfig) {
    native_shadcn_common_slider().apply_to(&mut config.common.slider);
    config.slider = native_shadcn_slider();
}

#[derive(Clone, Debug)]
pub struct SliderMetricTable {
    pub width: crate::ResolvedMetric,
    pub height: crate::ResolvedMetric,
    pub track_height: crate::ResolvedMetric,
    pub thumb_size: crate::ResolvedMetric,
    pub radius: crate::ResolvedMetric,
}

/// Default medium slider geometry retained for existing inspection callers.
pub fn resolve_slider_metrics(mode: &ShadcnModeTokens, theme_mode: ThemeMode) -> SliderMetricTable {
    resolve_slider_metrics_for_size(mode, theme_mode, ShadcnSize::Md)
}

pub fn resolve_slider_metrics_for_size(
    mode: &ShadcnModeTokens,
    _theme_mode: ThemeMode,
    size: ShadcnSize,
) -> SliderMetricTable {
    resolve_slider_metrics_with_stylesheet(mode, mode.stylesheet(), size)
}

/// Shared resolved dimensions; legacy metrics remain below common authored fields.
pub(crate) fn slider_geometry_with_stylesheet(
    mode: &ShadcnModeTokens,
    stylesheet: &crate::stylesheet::StylesheetConfig,
    size: ShadcnSize,
) -> (
    xgpui_luma::theme::stylesheet::SliderGeometry,
    xgpui_luma::theme::stylesheet::SliderGeometrySources,
    f32,
) {
    use xgpui_luma::theme::stylesheet::{CommonStylesheet, SliderGeometry};
    let mut fallback = SliderGeometry::sdk_default(&mode.metrics_for(size), CommonStylesheet::sdk_default());
    let mut radius = mode.radii.full;
    if let Some(rule) = stylesheet.slider.metrics_for_size(size) {
        let metrics = crate::stylesheet::resolve_slider_metrics(rule, &mode.control_sizes, size);
        fallback.width = metrics.width;
        fallback.height = metrics.height;
        fallback.track_height = metrics.track_height;
        fallback.thumb_size = metrics.thumb_size;
        radius = metrics.radius;
    }
    let (geometry, sources) = stylesheet.common.slider.inspect_geometry_for(size.key(), &fallback);
    (geometry, sources, radius)
}

pub fn resolve_slider_metrics_with_stylesheet(
    mode: &ShadcnModeTokens,
    stylesheet: &crate::stylesheet::StylesheetConfig,
    size: ShadcnSize,
) -> SliderMetricTable {
    use crate::tables::metrics::helpers::{derived_metric, inspect_shared_metric, pill_radius_metric};
    let (geometry, sources, radius) = slider_geometry_with_stylesheet(mode, stylesheet, size);
    let size_key = crate::tables::metrics::helpers::control_size_key(size);
    let legacy = stylesheet.slider.metrics_for_size(size).is_some();
    let metric = |field: &str, value: xgpui_luma::theme::provenance::ResolvedMetric| {
        if legacy && matches!(value.source, xgpui_luma::theme::provenance::MetricSource::Constant { .. }) {
            derived_metric(format!("stylesheet · slider.metrics.{size_key}.{field}"), value.value_px)
        } else {
            inspect_shared_metric(value)
        }
    };
    SliderMetricTable {
        width: metric(
            "width",
            xgpui_luma::theme::provenance::ResolvedMetric { value_px: geometry.width, source: sources.width },
        ),
        height: metric(
            "height",
            xgpui_luma::theme::provenance::ResolvedMetric { value_px: geometry.height, source: sources.height },
        ),
        track_height: metric(
            "track_height",
            xgpui_luma::theme::provenance::ResolvedMetric {
                value_px: geometry.track_height,
                source: sources.track_height,
            },
        ),
        thumb_size: metric(
            "thumb_size",
            xgpui_luma::theme::provenance::ResolvedMetric { value_px: geometry.thumb_size, source: sources.thumb_size },
        ),
        radius: if legacy {
            derived_metric(format!("stylesheet · slider.metrics.{size_key}.radius"), radius)
        } else {
            pill_radius_metric(&mode.catalog, radius)
        },
    }
}

/// The track and thumb end radii for a corner preset at `size`, from `.lss` (`style/slider.lss`'s
/// `radius` attribute).
pub(crate) fn slider_preset_corners(
    look: &crate::ShadcnLook,
    size: crate::ShadcnSize,
    preset: ButtonRadiusPreset,
) -> (f32, f32) {
    use xgpui_luma::style::lss::{PartId, Value};
    let source = look.clone();
    let binding = xgpui_luma::style::StyleBinding::new(
        "slider",
        xgpui_luma::style::ThemeSource::Live(std::sync::Arc::new(move || source.style_theme())),
    );
    let binding = if size != crate::ShadcnSize::Md {
        binding.attr("size", size.key())
    } else {
        binding
    };
    let resolved = binding.attr("radius", preset.key()).resolve(&[], 1.0);
    let radius = |part: &'static str| {
        resolved.part(PartId::new(part)).value("radius").and_then(Value::as_number).unwrap_or_default()
    };
    (radius("track"), radius("thumb"))
}
