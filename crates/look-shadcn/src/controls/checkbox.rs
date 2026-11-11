//! Checkbox property mappings (tweakcn / shadcn):
//!
//! | State    | Indicator token | Checkmark token        |
//! |----------|-----------------|------------------------|
//! | Unchecked| outline surface | `foreground`           |
//! | Checked  | `{style}`       | `{style}-foreground`   |
//! | Disabled | `muted`         | `muted-foreground`     |

use crate::stylesheet::config::LayeredElevationRule;
use crate::stylesheet::config::matches_optional_bool;
use serde::Deserialize;

use xgpui_luma::controls::checkbox::CheckboxPalette;
use xgpui_luma::theme::ControlSize;
use xgpui_luma::theme::InteractionLayer;
use xgpui_luma::theme::InteractionState;
use xgpui_luma::theme::ThemeMode;

use crate::look_context::LookContext;
use crate::provenance::LookResolver;
use crate::provenance::ResolvedColor;
use super::ShadcnButtonStyle;
use super::choice_indicator::choice_indicator_color_layer;
use crate::mode::ShadcnModeTokens;
use crate::shadow::parse_shadow_token;
use crate::stylesheet::StylesheetConfig;
use crate::stylesheet::find_checkbox_color_rule;
use crate::stylesheet::resolve_checkbox_color_rule;
use crate::stylesheet::resolve_layered_elevation_shadow;

#[derive(Clone, Debug)]
pub struct CheckboxColorTable {
    pub indicator_background: ResolvedColor,
    pub checkmark_color: ResolvedColor,
    pub label_color: ResolvedColor,
}

impl CheckboxColorTable {
    pub fn fallback() -> Self {
        Self {
            indicator_background: ResolvedColor::transparent(),
            checkmark_color: ResolvedColor::fallback_foreground(),
            label_color: ResolvedColor::fallback_foreground(),
        }
    }
}

pub fn resolve_checkbox_colors(
    resolver: &LookResolver<'_>,
    style: ShadcnButtonStyle,
    checked: bool,
    layer: InteractionLayer,
) -> anyhow::Result<CheckboxColorTable> {
    resolve_checkbox_colors_with_stylesheet(resolver, resolver.stylesheet(), style, checked, layer)
}

pub fn resolve_checkbox_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
    checked: bool,
    layer: InteractionLayer,
) -> anyhow::Result<CheckboxColorTable> {
    let rule = find_checkbox_color_rule(stylesheet, checked, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching checkbox color rule"))?;
    let colors = resolve_checkbox_color_rule(resolver, rule, style, layer)?;
    Ok(CheckboxColorTable {
        indicator_background: colors.indicator_background,
        checkmark_color: colors.checkmark_color,
        label_color: colors.label_color,
    })
}

/// Face colors after style normalization, interaction policy, and fallbacks.
/// Focus-adorner paint is resolved independently in the control palette.
#[derive(Clone, Debug)]
pub struct CheckboxResolvedColors {
    pub indicator_background: ResolvedColor,
    pub indicator_border: ResolvedColor,
    pub checkmark_color: ResolvedColor,
    pub label_color: ResolvedColor,
}

/// Resolves the same final colors used by rendering and inspection.
pub fn resolve_checkbox_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    checked: bool,
    state: InteractionState,
) -> CheckboxResolvedColors {
    resolve_checkbox_palette_with_stylesheet(
        &LookContext::new(mode, theme_mode, state),
        mode.stylesheet(),
        style,
        checked,
    )
}

pub(crate) fn resolve_checkbox_palette_with_stylesheet(
    ctx: &LookContext,
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
    checked: bool,
) -> CheckboxResolvedColors {
    let mode = ctx.tokens;
    let theme_mode = ctx.theme_mode;
    let state = ctx.state;
    let indicator_style = if style == ShadcnButtonStyle::ContentOnly {
        ShadcnButtonStyle::Primary
    } else {
        style
    };
    let resolver = LookResolver::new(&mode.catalog, theme_mode, "checkbox").with_stylesheet(stylesheet);
    let colors = resolve_checkbox_colors_with_stylesheet(
        &resolver,
        stylesheet,
        indicator_style,
        checked,
        choice_indicator_color_layer(state),
    )
    .unwrap_or_else(|_| CheckboxColorTable::fallback());
    let indicator_border = super::choice_indicator::resolve_indicator_border(
        &resolver,
        style,
        checked,
        state,
        &colors.indicator_background,
    );
    CheckboxResolvedColors {
        indicator_background: colors.indicator_background,
        indicator_border,
        checkmark_color: colors.checkmark_color,
        label_color: colors.label_color,
    }
}

pub fn checkbox_look(
    mode: &ShadcnModeTokens,
    style: ShadcnButtonStyle,
    checked: bool,
    state: InteractionState,
    size: ControlSize,
) -> CheckboxPalette {
    checkbox_look_with_stylesheet(
        &LookContext::new(mode, mode.theme_mode, state),
        mode.stylesheet(),
        style,
        checked,
        size,
    )
}

pub(crate) fn checkbox_look_with_stylesheet(
    ctx: &LookContext,
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
    checked: bool,
    size: ControlSize,
) -> CheckboxPalette {
    let content_only = style == ShadcnButtonStyle::ContentOnly;
    let state = ctx.state;
    let catalog = ctx.catalog();
    let typography =
        crate::tables::typography::resolve_control_typography_with_stylesheet(ctx.tokens, stylesheet, size, false);
    let layer = choice_indicator_color_layer(state);
    let colors = resolve_checkbox_palette_with_stylesheet(ctx, stylesheet, style, checked);

    CheckboxPalette {
        focus_border: if style == ShadcnButtonStyle::ContentOnly {
            gpui::transparent_black()
        } else {
            crate::focus::focus_ring_or_fallback(ctx.catalog())
        },
        control_background: None,
        control_border: None,
        indicator_background: colors.indicator_background.hsla(),
        indicator_border: colors.indicator_border.hsla(),
        checkmark_color: colors.checkmark_color.hsla(),
        label_color: colors.label_color.hsla(),
        label_typography: typography.style,
        label_font_family: typography.font_family,
        indicator_shadow: if content_only {
            None
        } else {
            checkbox_elevation_shadow(catalog, stylesheet, layer)
        },
    }
}

fn checkbox_elevation_shadow(
    catalog: &crate::catalog::CssTokenMap,
    stylesheet: &StylesheetConfig,
    layer: InteractionLayer,
) -> Option<Vec<gpui::BoxShadow>> {
    let token = resolve_layered_elevation_shadow(&stylesheet.checkbox.elevation_rules, layer)?;
    let shadows = parse_shadow_token(catalog, &token).ok()?;
    if shadows.is_empty() { None } else { Some(shadows) }
}

/// Resolve common indicator dimensions over the SDK's existing row scale.
pub(crate) fn checkbox_scale_with_stylesheet(
    ctx: &LookContext,
    stylesheet: &StylesheetConfig,
    size: ControlSize,
    scale_factor: f32,
) -> (
    xgpui_luma::controls::checkbox::CheckboxScale,
    xgpui_luma::theme::stylesheet::ResolvedCheckboxGeometry,
) {
    use xgpui_luma::controls::checkbox::CheckboxScale;
    use xgpui_luma::theme::{snap_to_pixel, stylesheet::CheckboxGeometry};
    let mut scale = CheckboxScale::compute_with_stylesheet(size, ctx.metrics(), scale_factor, &stylesheet.common);
    let mut geometry = stylesheet.common.checkbox.inspect_geometry(
        crate::tables::metrics::helpers::control_size_key(size),
        CheckboxGeometry {
            indicator_size: scale.indicator_size,
            glyph_size: scale.glyph_size,
            gap: scale.gap,
            indicator_radius: scale.indicator_radius,
            control_padding_x: scale.control_padding_x,
            control_padding_y: scale.control_padding_y,
            ..Default::default()
        },
    );
    for metric in [&mut geometry.indicator_size, &mut geometry.glyph_size, &mut geometry.gap] {
        metric.value_px = snap_to_pixel(metric.value_px, scale_factor);
    }
    scale.indicator_size = geometry.indicator_size.value_px;
    scale.glyph_size = geometry.glyph_size.value_px;
    scale.gap = geometry.gap.value_px;
    scale.indicator_radius = snap_to_pixel(geometry.indicator_radius.value_px, scale_factor);
    scale.control_padding_x = snap_to_pixel(geometry.control_padding_x.value_px, scale_factor);
    scale.control_padding_y = snap_to_pixel(geometry.control_padding_y.value_px, scale_factor);

    (scale, geometry)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use xgpui_luma::theme::ThemeMode;

    fn retro_arcade_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "hsl(330.9554 64.0816% 51.9608%)".into()),
            ("primary-foreground".into(), "hsl(0 0% 100%)".into()),
            ("secondary".into(), "hsl(175.4622 58.6207% 39.8039%)".into()),
            ("secondary-foreground".into(), "hsl(0 0% 100%)".into()),
            ("background".into(), "hsl(43.8462 86.6667% 94.1176%)".into()),
            ("foreground".into(), "hsl(192.2034 80.8219% 14.3137%)".into()),
            ("muted".into(), "hsl(180 6.9307% 60.3922%)".into()),
            ("muted-foreground".into(), "hsl(192.2034 80.8219% 14.3137%)".into()),
            ("accent".into(), "hsl(17.5691 80.4444% 44.1176%)".into()),
            ("accent-foreground".into(), "hsl(0 0% 100%)".into()),
            ("destructive".into(), "hsl(1.0405 71.1934% 52.3529%)".into()),
            ("destructive-foreground".into(), "hsl(0 0% 100%)".into()),
            ("border".into(), "hsl(186.3158 8.2969% 55.0980%)".into()),
            ("input".into(), "hsl(186.3158 8.2969% 55.0980%)".into()),
            ("ring".into(), "hsl(330.9554 64.0816% 51.9608%)".into()),
            ("card".into(), "hsl(45.6000 42.3729% 88.4314%)".into()),
            ("radius".into(), "0.25rem".into()),
            ("spacing".into(), "0.25rem".into()),
            ("font-sans".into(), "ui-sans-serif, system-ui, 'Outfit', sans-serif".into()),
            ("shadow-xs".into(), "0 1px 3px 0px hsl(0 0% 0% / 0.05)".into()),
            ("shadow-sm".into(), "0 1px 3px 0px hsl(0 0% 0% / 0.10), 0 1px 2px -1px hsl(0 0% 0% / 0.10)".into()),
        ]))
    }

    #[test]
    fn checkbox_look_resolves_stylesheet_shadow() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let look =
            checkbox_look(&mode, ShadcnButtonStyle::Primary, false, InteractionState::default(), ControlSize::Md);

        assert!(look.indicator_shadow.as_ref().is_some_and(|shadows| !shadows.is_empty()));
    }

    #[test]
    fn disabled_checkbox_look_has_no_shadow() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let look = checkbox_look(
            &mode,
            ShadcnButtonStyle::Primary,
            false,
            InteractionState { disabled: true, ..InteractionState::default() },
            ControlSize::Md,
        );

        assert!(look.indicator_shadow.is_none());
    }

    #[test]
    fn checkbox_hover_does_not_recolor_indicator_or_label() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let default =
            checkbox_look(&mode, ShadcnButtonStyle::Primary, false, InteractionState::default(), ControlSize::Md);
        let hovered = checkbox_look(
            &mode,
            ShadcnButtonStyle::Primary,
            false,
            InteractionState { hovered: true, ..InteractionState::default() },
            ControlSize::Md,
        );

        assert_eq!(default.indicator_background, hovered.indicator_background);
        assert_eq!(default.checkmark_color, hovered.checkmark_color);
        assert_eq!(default.label_color, hovered.label_color);
    }

    #[test]
    fn content_only_checkbox_keeps_primary_indicator_without_shadow() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let primary =
            checkbox_look(&mode, ShadcnButtonStyle::Primary, true, InteractionState::default(), ControlSize::Md);
        let content_only = checkbox_look(
            &mode,
            ShadcnButtonStyle::ContentOnly,
            true,
            InteractionState { focused: true, ..InteractionState::default() },
            ControlSize::Md,
        );

        assert_eq!(content_only.indicator_background, primary.indicator_background);
        assert_eq!(content_only.indicator_border, primary.indicator_border);
        assert_eq!(content_only.checkmark_color, primary.checkmark_color);
        assert!(content_only.indicator_shadow.is_none());
    }

    fn mode_without_border_and_ring() -> ShadcnModeTokens {
        let mut mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        mode.catalog.tokens.remove("ring");
        mode.catalog.tokens.remove("border");
        mode
    }

    #[test]
    fn incomplete_catalog_does_not_panic_on_focus_or_border() {
        let mode = mode_without_border_and_ring();
        let fallback = ResolvedColor::fallback_foreground().hsla();
        let focused = checkbox_look(
            &mode,
            ShadcnButtonStyle::Primary,
            false,
            InteractionState { focused: true, ..InteractionState::default() },
            ControlSize::Md,
        );
        let unfocused =
            checkbox_look(&mode, ShadcnButtonStyle::Primary, false, InteractionState::default(), ControlSize::Md);

        assert_eq!(focused.indicator_border, fallback);
        assert_eq!(unfocused.indicator_border, fallback);
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct CheckboxStylesheet {
    #[serde(default)]
    pub elevation_rules: Vec<LayeredElevationRule>,
    #[serde(default)]
    pub color_rules: Vec<CheckboxColorRule>,
}

impl CheckboxStylesheet {
    pub fn elevation_rule_for_layer(&self, layer: InteractionLayer) -> Option<&LayeredElevationRule> {
        if layer == InteractionLayer::Disabled {
            return self.elevation_rules.iter().find(|rule| rule.layer.as_deref() == Some("disabled"));
        }
        self.elevation_rules.iter().find(|rule| rule.layer.is_none())
    }

    pub fn find_color_rule(&self, checked: bool, layer: InteractionLayer) -> Option<&CheckboxColorRule> {
        self.color_rules.iter().find(|rule| {
            if rule.layer.as_deref() == Some("disabled") {
                return layer == InteractionLayer::Disabled;
            }
            if layer == InteractionLayer::Disabled {
                return false;
            }
            rule.layer.is_none() && matches_optional_bool(rule.checked, checked)
        })
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct CheckboxColorRule {
    pub layer: Option<String>,
    pub checked: Option<bool>,
    pub indicator_background: String,
    pub checkmark_color: String,
    pub label_color: String,
}

/// Construct Shadcn checkbox rules without parsing bundled data.
pub fn native_shadcn_checkbox() -> CheckboxStylesheet {
    CheckboxStylesheet {
        elevation_rules: vec![
            LayeredElevationRule { layer: None, shadow: "shadow-xs".into() },
            LayeredElevationRule { layer: Some("disabled".into()), shadow: "none".into() },
        ],
        color_rules: vec![
            CheckboxColorRule {
                layer: Some("disabled".into()),
                checked: None,
                indicator_background: "muted".into(),
                checkmark_color: "muted-foreground".into(),
                label_color: "muted-foreground".into(),
            },
            CheckboxColorRule {
                layer: None,
                checked: Some(false),
                indicator_background: "@outline_layer".into(),
                checkmark_color: "foreground".into(),
                label_color: "foreground".into(),
            },
            CheckboxColorRule {
                layer: None,
                checked: Some(true),
                indicator_background: "@action_layer".into(),
                checkmark_color: "@action_foreground".into(),
                label_color: "foreground".into(),
            },
        ],
    }
}

/// Install native Shadcn checkbox overrides over the SDK baseline.
pub fn define_shadcn_checkbox(config: &mut StylesheetConfig) {
    config.checkbox = native_shadcn_checkbox();
}

#[derive(Clone, Debug)]
pub struct CheckboxMetricTable {
    pub height: crate::ResolvedMetric,
    pub gap: crate::ResolvedMetric,
    pub indicator_size: crate::ResolvedMetric,
    pub indicator_radius: crate::ResolvedMetric,
    pub glyph_size: crate::ResolvedMetric,
    pub control_radius: crate::ResolvedMetric,
    pub border_width: crate::ResolvedMetric,
    pub focus_ring_width: crate::ResolvedMetric,
    pub focus_ring_offset: crate::ResolvedMetric,
}

pub fn resolve_checkbox_metrics(
    mode: &crate::ShadcnModeTokens,
    theme_mode: xgpui_luma::theme::ThemeMode,
    size: xgpui_luma::theme::ControlSize,
) -> CheckboxMetricTable {
    resolve_checkbox_metrics_with_stylesheet(mode, mode.stylesheet(), theme_mode, size)
}

pub(crate) fn resolve_checkbox_metrics_with_stylesheet(
    mode: &crate::ShadcnModeTokens,
    stylesheet: &crate::stylesheet::StylesheetConfig,
    theme_mode: xgpui_luma::theme::ThemeMode,
    size: xgpui_luma::theme::ControlSize,
) -> CheckboxMetricTable {
    use crate::catalog::SpacingField;
    use crate::tables::metrics::helpers::{
        border_width_metric, control_size_key, derived_metric, focus_ring_offset_metric, focus_ring_width_metric,
        radius_metric, scaffold_control_metric, spacing_control_metric,
    };

    let ctx = crate::LookContext::new(mode, theme_mode, xgpui_luma::theme::InteractionState::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let (scale, geometry) = crate::controls::checkbox::checkbox_scale_with_stylesheet(&ctx, stylesheet, size, 1.0);
    let authored = |metric: xgpui_luma::theme::provenance::ResolvedMetric, fallback: crate::ResolvedMetric| {
        if matches!(metric.source, xgpui_luma::theme::provenance::MetricSource::Constant { .. }) {
            fallback
        } else {
            crate::tables::metrics::helpers::inspect_shared_metric(metric)
        }
    };
    let size_key = control_size_key(size);

    CheckboxMetricTable {
        height: scaffold_control_metric(size_key, "control_height", scale.height),
        gap: authored(geometry.gap, spacing_control_metric(catalog, size, SpacingField::Gap, scale.gap)),
        indicator_size: authored(
            geometry.indicator_size,
            derived_metric(format!("{size_key} CheckboxScale.indicator_size"), scale.indicator_size),
        ),
        indicator_radius: derived_metric(format!("{size_key} CheckboxScale.indicator_radius"), scale.indicator_radius),
        glyph_size: authored(
            geometry.glyph_size,
            derived_metric(format!("{size_key} glyph = SDK indicator − inset"), scale.glyph_size),
        ),
        control_radius: radius_metric(catalog, size, scale.control_radius),
        border_width: border_width_metric(metrics),
        focus_ring_width: focus_ring_width_metric(metrics),
        focus_ring_offset: focus_ring_offset_metric(metrics),
    }
}
