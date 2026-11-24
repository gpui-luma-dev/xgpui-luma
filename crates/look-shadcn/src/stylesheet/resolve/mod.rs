use std::collections::HashMap;

use xgpui_luma::theme::InteractionLayer;

use crate::controls::ShadcnButtonStyle;
use crate::provenance::{LookResolver, ResolvedColor};
use crate::size::ShadcnSize;

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

pub fn resolve_stylesheet_metric(raw: &str, controls: &crate::ShadcnControlMetrics, size: ShadcnSize) -> Option<f32> {
    match raw.trim() {
        "metrics.control.sm" => Some(controls.control_height(ShadcnSize::Sm)),
        "metrics.control.md" => Some(controls.control_height(ShadcnSize::Md)),
        "metrics.control.lg" => Some(controls.control_height(ShadcnSize::Lg)),
        "radius" => Some(controls.radius(size)),
        "radius.full" => Some(xgpui_luma::theme::FULL_RADIUS_PX),
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

    use xgpui_luma::theme::ThemeMode;

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
    fn metric_token_resolves_control_height_and_radius() {
        let controls = crate::ShadcnLook::built_in().mode_tokens().control_sizes;
        assert_eq!(
            resolve_stylesheet_metric("metrics.control.md", &controls, ShadcnSize::Md),
            Some(controls.control_height(ShadcnSize::Md))
        );
        assert_eq!(
            resolve_stylesheet_metric("radius", &controls, ShadcnSize::Sm),
            Some(controls.radius(ShadcnSize::Sm))
        );
    }
}
