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

use xgpui_luma::controls::slider::{SliderLook, SliderThumbSize};
use xgpui_luma::theme::{ControlSize, ControlSizes, InteractionLayer, InteractionState, ThemeMode};

use crate::controls::button::{ButtonRadiusPreset, ShadcnButtonStyle};
use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::shadow::parse_shadow_token;
use crate::stylesheet::{
    StylesheetConfig, find_slider_color_rule, resolve_layered_elevation_shadow, resolve_slider_color_rule,
};

/// Shared corner-radius preset resolution for slider parts.
///
/// Medium/large/full resolve to a pill/circle for the given cross-axis size.
fn resolve_slider_part_radius_preset(
    preset: ButtonRadiusPreset,
    metrics: &xgpui_luma::theme::MetricTokens,
    cross_axis_size: f32,
) -> f32 {
    match preset {
        ButtonRadiusPreset::None => metrics.radius.none,
        ButtonRadiusPreset::Small => metrics.radius.sm,
        ButtonRadiusPreset::Medium | ButtonRadiusPreset::Large | ButtonRadiusPreset::Full => cross_axis_size / 2.0,
    }
}

/// Thumb corner radius presets for slider controls.
pub fn resolve_slider_thumb_radius_preset(
    preset: ButtonRadiusPreset,
    metrics: &xgpui_luma::theme::MetricTokens,
    thumb_size: f32,
) -> f32 {
    resolve_slider_part_radius_preset(preset, metrics, thumb_size)
}

/// Track end-cap radius presets for slider controls.
pub fn resolve_slider_track_radius_preset(
    preset: ButtonRadiusPreset,
    metrics: &xgpui_luma::theme::MetricTokens,
    track_height: f32,
) -> f32 {
    resolve_slider_part_radius_preset(preset, metrics, track_height)
}

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

pub fn resolve_slider_colors(
    resolver: &LookResolver<'_>,
    style: ShadcnButtonStyle,
    layer: InteractionLayer,
) -> anyhow::Result<SliderColorTable> {
    resolve_slider_colors_with_stylesheet(resolver, resolver.stylesheet(), style, layer)
}

pub fn resolve_slider_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
    layer: InteractionLayer,
) -> anyhow::Result<SliderColorTable> {
    let style_key = match style {
        ShadcnButtonStyle::Primary => "primary",
        ShadcnButtonStyle::Secondary => "secondary",
        ShadcnButtonStyle::Outline => "outline",
        ShadcnButtonStyle::Ghost => "ghost",
        ShadcnButtonStyle::ContentOnly => "primary",
    };
    let rule = find_slider_color_rule(stylesheet, style_key, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching slider color rule"))?;
    let colors = resolve_slider_color_rule(resolver, rule, style, layer)?;
    Ok(SliderColorTable {
        track_background: colors.track_background,
        fill_background: colors.fill_background,
        thumb_background: colors.thumb_background,
        thumb_border: colors.thumb_border,
    })
}

pub fn slider_look(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    size: ControlSize,
    thumb_size: Option<SliderThumbSize>,
    state: InteractionState,
) -> SliderLook {
    slider_look_with_stylesheet(mode, mode.stylesheet(), theme_mode, style, size, thumb_size, state)
}

pub(crate) fn slider_look_with_stylesheet(
    mode: &ShadcnModeTokens,
    stylesheet: &StylesheetConfig,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    size: ControlSize,
    thumb_size: Option<SliderThumbSize>,
    state: InteractionState,
) -> SliderLook {
    let ctx = LookContext::new(mode, theme_mode, state);
    let state = ctx.state;
    let catalog = ctx.catalog();
    let layer = state.layer();
    let thumb_shadow = slider_elevation_shadow(catalog, stylesheet, layer);
    let resolver = LookResolver::new(catalog, ctx.theme_mode, "slider").with_stylesheet(stylesheet);
    let colors = resolve_slider_colors(&resolver, style, layer).unwrap_or_else(|_| SliderColorTable::fallback());
    let (geometry, _, radius) = slider_geometry_with_stylesheet(mode, stylesheet, size, thumb_size);

    SliderLook {
        thumb_border_width: mode.metrics.border_width.default,
        decoration: stylesheet.common.slider.decoration(size),
        track_background: colors.track_background.hsla(),
        fill_background: colors.fill_background.hsla(),
        thumb_background: colors.thumb_background.hsla(),
        thumb_border: colors.thumb_border.hsla(),
        thumb_shadow,
        width: geometry.width,
        height: geometry.height,
        track_height: geometry.track_height,
        thumb_size: geometry.thumb_size,
        radius,
    }
}

fn slider_elevation_shadow(
    catalog: &crate::catalog::CssTokenMap,
    stylesheet: &StylesheetConfig,
    layer: InteractionLayer,
) -> Vec<gpui::BoxShadow> {
    let Some(token) = resolve_layered_elevation_shadow(&stylesheet.slider.elevation_rules, layer) else {
        return Vec::new();
    };
    parse_shadow_token(catalog, &token).unwrap_or_default()
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;
    use xgpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

    use super::slider_look;
    use crate::catalog::CssTokenMap;
    use crate::controls::button::{ButtonRadiusPreset, ShadcnButtonStyle};
    use crate::mode::ShadcnModeTokens;
    use xgpui_luma::controls::slider::SliderThumbSize;

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
            ("input".into(), "oklch(0.8 0.02 200)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("shadow-sm".into(), "0 1px 3px 0px hsl(0 0% 0% / 0.10), 0 1px 2px -1px hsl(0 0% 0% / 0.10)".into()),
        ]))
    }

    fn astrovista_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "hsl(15.1765 72.6496% 54.1176%)".into()),
            ("primary-foreground".into(), "hsl(0 0% 100%)".into()),
            ("secondary".into(), "hsl(217.2973 44.0476% 32.9412%)".into()),
            ("secondary-foreground".into(), "hsl(0 0% 100%)".into()),
            ("background".into(), "hsl(204.0000 12.1951% 91.9608%)".into()),
            ("foreground".into(), "hsl(0 0% 20%)".into()),
            ("muted".into(), "hsl(210 20.0000% 98.0392%)".into()),
            ("muted-foreground".into(), "hsl(220 8.9362% 46.0784%)".into()),
            ("accent".into(), "hsl(207.6923 46.4286% 89.0196%)".into()),
            ("border".into(), "hsl(0 0% 80%)".into()),
            ("input".into(), "hsl(220 15.7895% 96.2745%)".into()),
            ("ring".into(), "hsl(13.2143 73.0435% 54.9020%)".into()),
            ("card".into(), "hsl(0 0% 100%)".into()),
            ("shadow-sm".into(), "0 1px 3px 0px hsl(0 0% 0% / 0.10), 0 1px 2px -1px hsl(0 0% 0% / 0.10)".into()),
        ]))
    }

    #[test]
    fn slider_look_resolves_stylesheet_shadow() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let look = slider_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ControlSize::Md,
            None,
            InteractionState::default(),
        );

        assert!(!look.thumb_shadow.is_empty());
    }

    #[test]
    fn disabled_slider_look_has_no_shadow() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let look = slider_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ControlSize::Md,
            None,
            InteractionState { disabled: true, ..InteractionState::default() },
        );

        assert!(look.thumb_shadow.is_empty());
    }

    #[test]
    fn primary_slider_uses_border_track_and_primary_fill() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = slider_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ControlSize::Md,
            None,
            InteractionState::default(),
        );

        assert_eq!(look.track_background, catalog.color("border").expect("border"));
        assert_eq!(look.fill_background, catalog.color("primary").expect("primary"));
        assert_eq!(look.thumb_background, catalog.color("background").expect("background"));
        assert_ne!(look.track_background, catalog.color("secondary").expect("secondary"));
    }

    #[test]
    fn secondary_slider_uses_secondary_foreground_fill() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = slider_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Secondary,
            ControlSize::Md,
            None,
            InteractionState::default(),
        );

        assert_eq!(look.track_background, catalog.color("border").expect("border"));
        assert_eq!(look.fill_background, catalog.color("secondary-foreground").expect("secondary-foreground"));
        assert_eq!(look.thumb_border, catalog.color("secondary-foreground").expect("secondary-foreground"));
        assert_eq!(look.thumb_background, catalog.color("background").expect("background"));
        assert_ne!(look.fill_background, catalog.color("secondary").expect("secondary"));
    }

    #[test]
    fn disabled_slider_track_background_matches_enabled() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let enabled = slider_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ControlSize::Md,
            None,
            InteractionState::default(),
        );
        let disabled = slider_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ControlSize::Md,
            None,
            InteractionState { disabled: true, ..InteractionState::default() },
        );

        assert_eq!(disabled.track_background, enabled.track_background);
        assert_eq!(disabled.track_background, catalog.color("border").expect("border"));
    }

    #[test]
    fn astrovista_light_track_is_border_not_white_muted() {
        let catalog = astrovista_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = slider_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ControlSize::Md,
            None,
            InteractionState::default(),
        );

        let border = catalog.color("border").expect("border");
        let muted = catalog.color("muted").expect("muted");
        let card = catalog.color("card").expect("card");

        assert_eq!(look.track_background, border);
        assert_eq!(look.fill_background, catalog.color("primary").expect("primary"));
        assert_ne!(look.track_background, catalog.color("secondary").expect("secondary"));
        assert!(border.l > muted.l || (border.l - muted.l).abs() > 0.05);
        assert!(border.l < card.l - 0.05);
    }

    #[test]
    fn slider_size_changes_metrics() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog, ThemeMode::Light).expect("catalog");

        let sm = slider_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ControlSize::Sm,
            None,
            InteractionState::default(),
        );
        let md = slider_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ControlSize::Md,
            None,
            InteractionState::default(),
        );
        let lg = slider_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ControlSize::Lg,
            None,
            InteractionState::default(),
        );

        assert!(sm.height < md.height);
        assert!(md.height < lg.height);
        assert!(sm.track_height < md.track_height);
        assert!(md.track_height < lg.track_height);
        assert!(sm.thumb_size < md.thumb_size);
        assert!(md.thumb_size < lg.thumb_size);
    }

    #[test]
    fn slider_radius_presets_use_square_and_pill_shapes() {
        use super::{resolve_slider_thumb_radius_preset, resolve_slider_track_radius_preset};
        use xgpui_luma::theme::MetricTokens;

        let metrics = MetricTokens::default();
        let thumb_size = 16.0;
        let track_height = 6.0;
        let thumb_circle = thumb_size / 2.0;
        let track_pill = track_height / 2.0;

        assert_eq!(resolve_slider_thumb_radius_preset(ButtonRadiusPreset::None, &metrics, thumb_size), 0.0);
        assert_eq!(resolve_slider_track_radius_preset(ButtonRadiusPreset::None, &metrics, track_height), 0.0);
        assert!(resolve_slider_thumb_radius_preset(ButtonRadiusPreset::Small, &metrics, thumb_size) < thumb_circle);
        assert!(resolve_slider_track_radius_preset(ButtonRadiusPreset::Small, &metrics, track_height) <= track_pill);
        assert!(
            resolve_slider_track_radius_preset(ButtonRadiusPreset::Small, &metrics, track_height * 2.0) < track_height
        );
        assert_eq!(resolve_slider_thumb_radius_preset(ButtonRadiusPreset::Medium, &metrics, thumb_size), thumb_circle);
        assert_eq!(resolve_slider_track_radius_preset(ButtonRadiusPreset::Medium, &metrics, track_height), track_pill);
    }

    #[test]
    fn slider_thumb_override_changes_thumb_metrics_only() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog, ThemeMode::Light).expect("catalog");

        let md_default = slider_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ControlSize::Md,
            None,
            InteractionState::default(),
        );
        let md_large = slider_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ControlSize::Md,
            Some(SliderThumbSize::Lg),
            InteractionState::default(),
        );

        assert_eq!(md_default.height, md_large.height);
        assert_eq!(md_default.track_height, md_large.track_height);
        assert!(md_large.thumb_size > md_default.thumb_size);
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

    pub fn metrics_for_size(&self, size: ControlSize) -> Option<&SliderMetricsRule> {
        self.metrics.get(size).and_then(Option::as_ref)
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
        sizes: ControlSizes {
            sm: SliderGeometryStylesheet {
                width: Some(260.0),
                height: Some(24.0),
                track_height: Some(4.0),
                thumb_size: Some(12.0),
                ..Default::default()
            },
            md: SliderGeometryStylesheet {
                width: Some(260.0),
                height: Some(32.0),
                track_height: Some(6.0),
                thumb_size: Some(16.0),
                ..Default::default()
            },
            lg: SliderGeometryStylesheet {
                width: Some(260.0),
                height: Some(40.0),
                track_height: Some(8.0),
                thumb_size: Some(20.0),
                ..Default::default()
            },
            ..Default::default()
        },
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
    resolve_slider_metrics_for_size(mode, theme_mode, ControlSize::Md)
}

pub fn resolve_slider_metrics_for_size(
    mode: &ShadcnModeTokens,
    _theme_mode: ThemeMode,
    size: ControlSize,
) -> SliderMetricTable {
    resolve_slider_metrics_with_stylesheet(mode, mode.stylesheet(), size, None)
}

/// Shared resolved dimensions; legacy metrics remain below common authored fields.
pub(crate) fn slider_geometry_with_stylesheet(
    mode: &ShadcnModeTokens,
    stylesheet: &crate::stylesheet::StylesheetConfig,
    size: ControlSize,
    thumb_size: Option<xgpui_luma::controls::slider::SliderThumbSize>,
) -> (
    xgpui_luma::theme::stylesheet::SliderGeometry,
    xgpui_luma::theme::stylesheet::SliderGeometrySources,
    f32,
) {
    use xgpui_luma::controls::slider::SliderThumbSize;
    use xgpui_luma::theme::stylesheet::{CommonStylesheet, SliderGeometry};
    let mut fallback = SliderGeometry::sdk_default(&mode.metrics, size, thumb_size, CommonStylesheet::sdk_default());
    let mut radius = mode.metrics.radius.pill;
    if let Some(rule) = stylesheet.slider.metrics_for_size(size) {
        let metrics = crate::stylesheet::resolve_slider_metrics(rule, &mode.metrics, size);
        fallback.width = metrics.width;
        fallback.height = metrics.height;
        fallback.track_height = metrics.track_height;
        radius = metrics.radius;
        if thumb_size.is_none() {
            fallback.thumb_size = metrics.thumb_size;
        }
    }
    let mut legacy_thumb_source = None;
    if let Some(thumb_size) = thumb_size {
        let (thumb_control_size, common_thumb) = match thumb_size {
            SliderThumbSize::Sm => (ControlSize::Sm, stylesheet.common.slider.sizes.sm.thumb_size),
            SliderThumbSize::Md => (ControlSize::Md, stylesheet.common.slider.sizes.md.thumb_size),
            SliderThumbSize::Lg => (ControlSize::Lg, stylesheet.common.slider.sizes.lg.thumb_size),
        };
        if let Some(rule) = stylesheet.slider.metrics_for_size(thumb_control_size) {
            fallback.thumb_size = rule.thumb_size;
            if common_thumb.is_none() && stylesheet.common.slider.geometry.thumb_size.is_none() {
                let key = crate::tables::metrics::helpers::control_size_key(thumb_control_size);
                legacy_thumb_source =
                    Some(format!("instance thumb_size {key} → stylesheet · slider.metrics.{key}.thumb_size"));
            }
        }
    }
    let thumb_key = thumb_size.map(|size| match size {
        SliderThumbSize::Sm => ControlSize::Sm,
        SliderThumbSize::Md => ControlSize::Md,
        SliderThumbSize::Lg => ControlSize::Lg,
    });
    let (geometry, mut sources) = stylesheet.common.slider.inspect_geometry(size, thumb_key, &fallback);
    if let Some(note) = legacy_thumb_source {
        sources.thumb_size = xgpui_luma::theme::provenance::MetricSource::Derived { note };
    }
    (geometry, sources, radius)
}

pub fn resolve_slider_metrics_with_stylesheet(
    mode: &ShadcnModeTokens,
    stylesheet: &crate::stylesheet::StylesheetConfig,
    size: ControlSize,
    thumb_size: Option<xgpui_luma::controls::slider::SliderThumbSize>,
) -> SliderMetricTable {
    use crate::tables::metrics::helpers::{derived_metric, inspect_shared_metric, pill_radius_metric};
    let (geometry, sources, radius) = slider_geometry_with_stylesheet(mode, stylesheet, size, thumb_size);
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
