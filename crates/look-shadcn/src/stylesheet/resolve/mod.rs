use std::collections::HashMap;

use gpui::{FontWeight, px};
use xgpui_luma::controls::sidebar::SidebarMetricScale;
use xgpui_luma::theme::{ControlSize, InteractionLayer, LumaTextStyle, MetricTokens};

use crate::controls::ShadcnButtonStyle;
use crate::provenance::{LookResolver, ResolvedColor};

use super::config::{
    ButtonMetricsRule, ProgressMetricsRule, ScrollbarMetricsRule, SidebarMetricsRule, SliderMetricsRule,
    StepperMetricsRule, SwitchMetricsRule, TypographyRule,
};

/// Context for resolving derived stylesheet tokens (`@outline_layer`, `@action_layer`, etc.).
#[derive(Clone, Copy, Debug)]
pub struct ResolveContext {
    pub style: ShadcnButtonStyle,
    pub layer: InteractionLayer,
    pub disabled: bool,
}

impl Default for ResolveContext {
    fn default() -> Self {
        Self { style: ShadcnButtonStyle::Primary, layer: InteractionLayer::Default, disabled: false }
    }
}

/// Resolved field values for a single color rule, used for `@field` interpolation.
#[derive(Clone, Debug, Default)]
pub struct ResolvedFields {
    colors: HashMap<String, ResolvedColor>,
}

impl ResolvedFields {
    pub fn insert(&mut self, field: impl Into<String>, color: ResolvedColor) {
        self.colors.insert(field.into(), color);
    }

    pub fn get(&self, field: &str) -> Option<&ResolvedColor> {
        self.colors.get(field)
    }
}

pub fn resolve_color_ref(
    resolver: &LookResolver<'_>,
    raw: &str,
    fields: &ResolvedFields,
) -> anyhow::Result<ResolvedColor> {
    resolve_stylesheet_color(resolver, raw, fields, &ResolveContext::default())
}

pub fn resolve_stylesheet_color(
    resolver: &LookResolver<'_>,
    raw: &str,
    fields: &ResolvedFields,
    ctx: &ResolveContext,
) -> anyhow::Result<ResolvedColor> {
    let raw = raw.trim();
    if let Some(field) = raw.strip_prefix('@') {
        if let Some(color) = fields.get(field) {
            return Ok(color.clone());
        }
        return resolve_derived_token(resolver, field, ctx);
    }
    if let Some(inner) = parse_first_fn(raw) {
        return resolver.resolve_first_decl(&inner);
    }
    if let Some(inner) = parse_first_layer_fn(raw) {
        return resolver.resolve_first_layer_decl(&inner, ctx.layer);
    }
    resolver.resolve_decl(raw)
}

pub fn resolve_optional_stylesheet_color(
    resolver: &LookResolver<'_>,
    raw: Option<&str>,
    fields: &ResolvedFields,
    ctx: &ResolveContext,
) -> anyhow::Result<Option<ResolvedColor>> {
    raw.map(|value| resolve_stylesheet_color(resolver, value, fields, ctx)).transpose()
}

fn resolve_derived_token(
    resolver: &LookResolver<'_>,
    token: &str,
    ctx: &ResolveContext,
) -> anyhow::Result<ResolvedColor> {
    match token {
        "outline_layer" => resolver.resolve_outline_layer_decl(ctx.layer),
        "action_layer" => resolver.resolve_action_layer_decl(ctx.style, ctx.layer),
        "action_foreground" => resolver.resolve_action_foreground_decl(ctx.style),
        "action_default" => resolver.resolve_action_layer_decl(ctx.style, InteractionLayer::Default),
        "primary_layer" => resolver.resolve_action_layer_decl(ShadcnButtonStyle::Primary, ctx.layer),
        "primary_default" => resolver.resolve_action_layer_decl(ShadcnButtonStyle::Primary, InteractionLayer::Default),
        "darken_border" => resolver.resolve_darken_border_decl(0.08),
        "accent_whisper_40" => resolver.resolve_accent_whisper_decl(40),
        "accent_whisper_pressed_40" => resolver.resolve_accent_whisper_pressed_decl(40),
        "label" => resolver.resolve_label_decl(ctx.disabled),
        other => anyhow::bail!("unknown stylesheet field reference `@{other}`"),
    }
}

fn parse_first_fn(raw: &str) -> Option<Vec<&str>> {
    let inner = raw.strip_prefix("first(")?.strip_suffix(')')?;
    Some(inner.split(',').map(str::trim).collect())
}

fn parse_first_layer_fn(raw: &str) -> Option<Vec<&str>> {
    let inner = raw.strip_prefix("first_layer(")?.strip_suffix(')')?;
    Some(inner.split(',').map(str::trim).collect())
}

mod color_rules;
pub use color_rules::*;

pub fn resolve_slider_metrics(
    rule: &SliderMetricsRule,
    metrics: &MetricTokens,
    size: ControlSize,
) -> ResolvedSliderMetrics {
    ResolvedSliderMetrics {
        width: rule.width,
        height: rule.height,
        track_height: rule.track_height,
        thumb_size: rule.thumb_size,
        radius: resolve_stylesheet_metric(&rule.radius, metrics, size).unwrap_or(metrics.radius.pill),
    }
}

pub fn resolve_switch_metrics(rule: &SwitchMetricsRule) -> ResolvedSwitchMetrics {
    ResolvedSwitchMetrics { width: rule.width, height: rule.height, thumb_size: rule.thumb_size }
}

pub fn resolve_scrollbar_metrics(rule: &ScrollbarMetricsRule) -> ResolvedScrollbarMetrics {
    ResolvedScrollbarMetrics {
        thickness: rule.thickness,
        track_thickness: rule.track_thickness,
        thumb_thickness: rule.thumb_thickness,
        min_thumb_length: rule.min_thumb_length,
    }
}

pub fn resolve_progress_metrics(rule: &ProgressMetricsRule) -> ResolvedProgressMetrics {
    ResolvedProgressMetrics {
        size: rule.size,
        stroke_width: rule.stroke_width,
        track_height: rule.track_height,
        thumb_size: rule.thumb_size,
    }
}

pub fn resolve_stepper_metrics(rule: &StepperMetricsRule) -> ResolvedStepperMetrics {
    ResolvedStepperMetrics { step_badge_size: rule.step_badge_size, track_thickness: rule.track_thickness }
}
pub fn resolve_button_metrics_rule(
    rule: &ButtonMetricsRule,
    metrics: &MetricTokens,
    size: ControlSize,
) -> ResolvedButtonMetrics {
    ResolvedButtonMetrics {
        height: resolve_stylesheet_metric(&rule.height, metrics, size).unwrap_or_else(|| metrics.control_height(size)),
        padding_horizontal: rule.padding_horizontal,
        font_size: rule.font_size,
        icon_size: rule.icon_size,
        corner_radius: resolve_stylesheet_metric(&rule.corner_radius, metrics, size)
            .unwrap_or_else(|| metrics.radius(size)),
    }
}

pub fn resolve_sidebar_metrics(rule: &SidebarMetricsRule, metrics: &MetricTokens) -> SidebarMetricScale {
    let defaults = SidebarMetricScale::default();
    let size = ControlSize::Md;
    let resolve =
        |raw: &str, fallback: gpui::Pixels| resolve_stylesheet_metric(raw, metrics, size).map(px).unwrap_or(fallback);

    SidebarMetricScale {
        width_expanded: resolve(&rule.width_expanded, defaults.width_expanded),
        width_icon_rail: resolve(&rule.width_icon_rail, defaults.width_icon_rail),
        width_mobile: resolve(&rule.width_mobile, defaults.width_mobile),
        item_height: resolve(&rule.item_height, defaults.item_height),
        icon_size: resolve(&rule.icon_size, defaults.icon_size),
        rail_hit_width: resolve(&rule.rail_hit_width, defaults.rail_hit_width),
        popover_offset: resolve(&rule.popover_offset, defaults.popover_offset),
    }
}

pub fn resolve_stylesheet_shadow_token(raw: &str) -> Option<String> {
    match raw.trim() {
        "" | "none" => None,
        token => Some(token.to_string()),
    }
}

pub fn resolve_layered_elevation_shadow(
    rules: &[crate::stylesheet::config::LayeredElevationRule],
    layer: InteractionLayer,
) -> Option<String> {
    let rule = if layer == InteractionLayer::Disabled {
        rules.iter().find(|rule| rule.layer.as_deref() == Some("disabled"))
    } else {
        rules.iter().find(|rule| rule.layer.is_none())
    }?;
    resolve_stylesheet_shadow_token(&rule.shadow)
}

pub fn resolve_typography_rule(rule: &TypographyRule) -> LumaTextStyle {
    LumaTextStyle { size: rule.size, line_height: rule.line_height, weight: FontWeight(rule.weight) }
}

pub fn resolve_stylesheet_metric(raw: &str, metrics: &MetricTokens, size: ControlSize) -> Option<f32> {
    match raw.trim() {
        "metrics.control.sm" => Some(metrics.control_height(ControlSize::Sm)),
        "metrics.control.md" => Some(metrics.control_height(ControlSize::Md)),
        "metrics.control.lg" => Some(metrics.control_height(ControlSize::Lg)),
        "radius" => Some(metrics.radius(size)),
        "radius.pill" => Some(metrics.radius.pill),
        value => resolve_f32_literal(value).ok(),
    }
}

fn resolve_f32_literal(raw: &str) -> anyhow::Result<f32> {
    let value = raw.trim();
    if let Some(rem) = value.strip_suffix("rem") {
        return rem.trim().parse::<f32>().map(|n| n * 16.0).map_err(Into::into);
    }
    if let Some(px) = value.strip_suffix("px") {
        return px.trim().parse().map_err(Into::into);
    }
    value.parse().map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use gpui::px;
    use xgpui_luma::theme::{ControlSize, MetricTokens, ThemeMode};

    use crate::catalog::CssTokenMap;
    use crate::provenance::LookResolver;

    use super::*;

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "hsl(330 64% 52%)".into()),
            ("primary-foreground".into(), "hsl(0 0% 100%)".into()),
            ("background".into(), "hsl(0 0% 100%)".into()),
        ]))
    }

    #[test]
    fn field_reference_uses_prior_resolved_color() {
        let catalog = sample_catalog();
        let resolver = LookResolver::new(&catalog, ThemeMode::Light, "test");
        let mut fields = ResolvedFields::default();
        fields.insert("background", resolver.resolve_decl("primary").expect("primary"));

        let referenced = resolve_color_ref(&resolver, "@background", &fields).expect("reference");
        assert_eq!(referenced.value, fields.get("background").expect("background").value);
    }

    #[test]
    fn f32_literal_parses_px_and_rem() {
        assert!((resolve_f32_literal("12.0").expect("literal") - 12.0).abs() < f32::EPSILON);
        assert!((resolve_f32_literal("1rem").expect("rem") - 16.0).abs() < f32::EPSILON);
    }

    #[test]
    fn resolve_sidebar_metrics_from_rem_literals() {
        use crate::stylesheet::config::SidebarMetricsRule;

        let metrics = MetricTokens::default();
        let rule = SidebarMetricsRule::default();
        let scale = resolve_sidebar_metrics(&rule, &metrics);
        assert_eq!(scale.width_expanded, px(256.0));
        assert_eq!(scale.width_icon_rail, px(48.0));
        assert_eq!(scale.width_mobile, px(288.0));
        assert_eq!(scale.item_height, px(32.0));
        assert_eq!(scale.icon_size, px(16.0));
    }

    #[test]
    fn metric_token_resolves_control_height_and_radius() {
        let metrics = MetricTokens::default();
        assert_eq!(
            resolve_stylesheet_metric("metrics.control.md", &metrics, ControlSize::Md),
            Some(metrics.control_height(ControlSize::Md))
        );
        assert_eq!(
            resolve_stylesheet_metric("radius", &metrics, ControlSize::Sm),
            Some(metrics.radius(ControlSize::Sm))
        );
    }

    #[test]
    fn typography_rule_resolves_to_text_style() {
        let style = resolve_typography_rule(&TypographyRule { size: 16.0, line_height: 22.0, weight: 600.0 });
        assert_eq!(style.size, 16.0);
        assert_eq!(style.line_height, 22.0);
        assert_eq!(style.weight, FontWeight(600.0));
    }
}
