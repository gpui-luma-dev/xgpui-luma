//! Inspect metadata for `button`.

use gpui::{BoxShadow, Hsla};
use xgpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use crate::{
    LookContext, MetricSource, ResolvedColor, ResolvedMetric, ResolvedTypography, ShadcnButtonStyle, ShadcnModeTokens,
};
use crate::stylesheet::{find_button_elevation_rule, resolve_stylesheet_shadow_token};

use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::infra::shadow_layout::shadow_projection_extent;

pub struct ButtonInspectPalette {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
}

pub fn inspect_button_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    state: InteractionState,
) -> ButtonInspectPalette {
    inspect_button_color_palette_with_stylesheet(mode, mode.stylesheet(), theme_mode, style, role, state)
}

pub(crate) fn inspect_button_color_palette_with_stylesheet(
    mode: &ShadcnModeTokens,
    stylesheet: &crate::stylesheet::StylesheetConfig,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    state: InteractionState,
) -> ButtonInspectPalette {
    let colors = crate::controls::button::resolve_button_palette_with_stylesheet(
        &LookContext::new(mode, theme_mode, state),
        stylesheet,
        style,
        role,
    );

    let border = effective_border_resolved(&colors);
    ButtonInspectPalette { background: colors.background, foreground: colors.foreground, border }
}

#[derive(Clone, Debug)]
pub struct ButtonInspectMetrics {
    pub height: ResolvedMetric,
    pub icon_size: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub gap: ResolvedMetric,
    pub radius: ResolvedMetric,
    pub border_width: ResolvedMetric,
    pub focus_ring_width: ResolvedMetric,
    pub focus_ring_offset: ResolvedMetric,
}

pub fn inspect_button_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    size: ControlSize,
    state: InteractionState,
) -> ButtonInspectMetrics {
    inspect_button_metrics_with_stylesheet(mode, mode.stylesheet(), theme_mode, style, role, size, state)
}

pub(crate) fn inspect_button_metrics_with_stylesheet(
    mode: &ShadcnModeTokens,
    stylesheet: &crate::stylesheet::StylesheetConfig,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    size: ControlSize,
    state: InteractionState,
) -> ButtonInspectMetrics {
    let table = crate::tables::metrics::resolve_button_metrics_with_stylesheet(
        mode, stylesheet, theme_mode, style, role, size, state,
    );
    table.into()
}

#[derive(Clone, Debug)]
pub struct ButtonInspectTypography {
    pub font_family: ResolvedTypography,
    pub font_size: ResolvedTypography,
    pub font_weight: ResolvedTypography,
    pub line_height: ResolvedTypography,
}

#[derive(Clone, Debug)]
pub struct ButtonInspectElevationLayer {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Hsla,
    pub css: String,
}

#[derive(Clone, Debug)]
pub struct ButtonInspectElevation {
    pub applied: bool,
    pub rule_shadow: String,
    pub style_key: String,
    pub token: Option<String>,
    pub catalog_value: Option<String>,
    pub layers: Vec<ButtonInspectElevationLayer>,
    pub shadows: Option<Vec<BoxShadow>>,
    pub reserved_shadow_extent: ResolvedMetric,
}

pub fn inspect_button_elevation(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    state: InteractionState,
) -> ButtonInspectElevation {
    inspect_button_elevation_with_stylesheet(mode, mode.stylesheet(), theme_mode, style, role, state)
}

pub(crate) fn inspect_button_elevation_with_stylesheet(
    mode: &ShadcnModeTokens,
    stylesheet: &crate::stylesheet::StylesheetConfig,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    state: InteractionState,
) -> ButtonInspectElevation {
    let effective_style = effective_button_style(style, role);
    let look = crate::controls::button::button_look_with_stylesheet(
        mode,
        stylesheet,
        theme_mode,
        style,
        role,
        ControlSize::Md,
        None,
        state,
    );
    let ctx = LookContext::new(mode, theme_mode, state);
    let style_key = button_style_key(effective_style);
    let rule_shadow = if matches!(role, ButtonFamilyRole::Toggle { .. }) {
        match style {
            ShadcnButtonStyle::ContentOnly => "none".into(),
            ShadcnButtonStyle::Ghost | ShadcnButtonStyle::Outline => find_button_elevation_rule(stylesheet, style)
                .map(|rule| rule.shadow.clone())
                .unwrap_or_else(|| "none".into()),
            _ => crate::stylesheet::resolve_layered_elevation_shadow(&stylesheet.toggle.elevation_rules, state.layer())
                .unwrap_or_else(|| "none".into()),
        }
    } else {
        find_button_elevation_rule(stylesheet, effective_style)
            .map(|rule| rule.shadow.clone())
            .unwrap_or_else(|| "none".into())
    };
    let token = resolve_stylesheet_shadow_token(&rule_shadow);
    let catalog_value = token.as_ref().and_then(|token| ctx.catalog().get(token).map(|value| value.to_string()));
    let layers: Vec<ButtonInspectElevationLayer> = look
        .shadow
        .as_ref()
        .map(|shadows| shadows.iter().enumerate().map(|(index, shadow)| elevation_layer(index, shadow)).collect())
        .unwrap_or_default();
    let applied = !state.disabled && !layers.is_empty();
    let reserved_shadow_extent = resolved_shadow_extent(look.shadow.as_ref());

    ButtonInspectElevation {
        applied,
        rule_shadow,
        style_key: style_key.to_string(),
        token,
        catalog_value,
        layers,
        shadows: look.shadow.clone(),
        reserved_shadow_extent,
    }
}

pub fn inspect_layered_elevation(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    state: InteractionState,
    rule_shadow: String,
    token: Option<String>,
    shadows: Option<&Vec<BoxShadow>>,
    style_key: &str,
) -> ButtonInspectElevation {
    let ctx = LookContext::new(mode, theme_mode, state);
    let catalog_value = token.as_ref().and_then(|t| ctx.catalog().get(t).map(|value| value.to_string()));
    let layers: Vec<ButtonInspectElevationLayer> = shadows
        .map(|shadows| shadows.iter().enumerate().map(|(index, shadow)| elevation_layer(index, shadow)).collect())
        .unwrap_or_default();
    let applied = !state.disabled && !layers.is_empty();
    let reserved_shadow_extent = resolved_shadow_extent(shadows);

    ButtonInspectElevation {
        applied,
        rule_shadow,
        style_key: style_key.to_string(),
        token,
        catalog_value,
        layers,
        shadows: shadows.cloned(),
        reserved_shadow_extent,
    }
}

fn resolved_shadow_extent(shadows: Option<&Vec<BoxShadow>>) -> ResolvedMetric {
    ResolvedMetric {
        value_px: shadow_projection_extent(shadows.map(Vec::as_slice), 1.0, true),
        source: MetricSource::Derived { note: "shadow projection extent at scale 1.0".into() },
    }
}

pub fn format_inspect_box_shadow_layer(layer: &ButtonInspectElevationLayer) -> String {
    layer.css.clone()
}

/// Default medium text-button typography. Use the sized variant for other controls.
pub fn inspect_button_typography(mode: &ShadcnModeTokens, theme_mode: ThemeMode) -> ButtonInspectTypography {
    inspect_button_typography_for_size(mode, theme_mode, ControlSize::Md, ButtonFamilyRole::Text)
}

pub fn inspect_button_typography_for_size(
    mode: &ShadcnModeTokens,
    _theme_mode: ThemeMode,
    size: ControlSize,
    role: ButtonFamilyRole,
) -> ButtonInspectTypography {
    crate::tables::typography::resolve_control_typography(mode, size, matches!(role, ButtonFamilyRole::Toggle { .. }))
        .into()
}

impl From<crate::tables::typography::ControlTypographyTable> for ButtonInspectTypography {
    fn from(table: crate::tables::typography::ControlTypographyTable) -> Self {
        Self {
            font_family: ResolvedTypography { value: table.font_family.to_string(), source: table.font_family_source },
            font_size: ResolvedTypography { value: table.style.size.to_string(), source: table.font_size_source },
            font_weight: ResolvedTypography {
                value: table.style.weight.0.to_string(),
                source: table.font_weight_source,
            },
            line_height: ResolvedTypography {
                value: table.style.line_height.to_string(),
                source: table.line_height_source,
            },
        }
    }
}

fn effective_button_style(style: ShadcnButtonStyle, role: ButtonFamilyRole) -> ShadcnButtonStyle {
    if matches!(role, ButtonFamilyRole::Toggle { selected: false }) {
        match style {
            ShadcnButtonStyle::Ghost | ShadcnButtonStyle::Outline | ShadcnButtonStyle::ContentOnly => style,
            _ => ShadcnButtonStyle::Outline,
        }
    } else {
        style
    }
}

pub(crate) fn button_style_key(style: ShadcnButtonStyle) -> &'static str {
    match style {
        ShadcnButtonStyle::Primary => "primary",
        ShadcnButtonStyle::Secondary => "secondary",
        ShadcnButtonStyle::Outline => "outline",
        ShadcnButtonStyle::Ghost => "ghost",
        ShadcnButtonStyle::ContentOnly => "content-only",
    }
}

fn elevation_layer(_index: usize, shadow: &BoxShadow) -> ButtonInspectElevationLayer {
    let offset_x = shadow.offset.x.as_f32();
    let offset_y = shadow.offset.y.as_f32();
    let blur = shadow.blur_radius.as_f32();
    let spread = shadow.spread_radius.as_f32();
    let color = shadow.color;
    let css = format!(
        "{}px {}px {}px {}px hsl({} {}% {}% / {})",
        format_shadow_number(offset_x),
        format_shadow_number(offset_y),
        format_shadow_number(blur),
        format_shadow_number(spread),
        format_shadow_number(color.h * 360.0),
        format_shadow_number(color.s * 100.0),
        format_shadow_number(color.l * 100.0),
        format_shadow_alpha(color.a),
    );

    ButtonInspectElevationLayer { offset_x, offset_y, blur, spread, color, css }
}

fn format_shadow_number(value: f32) -> String {
    if (value - value.round()).abs() < f32::EPSILON {
        format!("{}", value.round() as i32)
    } else {
        format!("{value}")
    }
}

fn format_shadow_alpha(value: f32) -> String {
    let formatted = format!("{:.3}", value.clamp(0.0, 1.0));
    formatted.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn effective_border_resolved(colors: &crate::tables::ButtonColorPalette) -> ResolvedColor {
    use crate::ColorSource;

    if let Some(color) = &colors.border {
        return color.clone();
    }

    ResolvedColor { value: gpui::hsla(0.0, 0.0, 0.0, 0.0), source: ColorSource::Transparent }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspect::test_support::retro_arcade_catalog;

    #[test]
    fn button_color_table_metadata_is_populated() {
        let metadata = crate::stylesheet::resolve_button_colors_metadata(crate::embedded_stylesheet());
        assert!(!metadata.is_empty());
        assert!(metadata[0].inputs.len() == 4);
    }

    #[test]
    fn inspect_palette_inherited_border_references_background_token() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let palette = inspect_button_color_palette(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            InteractionState::default(),
        );
        assert!(matches!(
            palette.border.source,
            crate::ColorSource::CssVar { ref token } if token == "primary"
        ));
        assert_eq!(palette.border.value, palette.background.value);
    }

    #[test]
    fn inspect_metrics_match_button_look_for_primary_default_md() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let look = crate::paint::button_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState::default(),
        );
        let metrics = inspect_button_metrics(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            ControlSize::Md,
            InteractionState::default(),
        );

        assert_eq!(metrics.height.value_px, look.height);
        assert_eq!(metrics.padding_x.value_px, look.padding_x);
        assert_eq!(metrics.padding_y.value_px, look.padding_y);
        assert_eq!(metrics.gap.value_px, look.gap);
        assert_eq!(metrics.radius.value_px, look.radius);
    }

    #[test]
    fn inspect_elevation_outline_resolves_shadow_token_and_layers() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let elevation = inspect_button_elevation(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Outline,
            ButtonFamilyRole::Text,
            InteractionState::default(),
        );

        assert!(elevation.applied);
        assert_eq!(elevation.rule_shadow, "shadow-xs");
        assert_eq!(elevation.token.as_deref(), Some("shadow-xs"));
        assert!(elevation.catalog_value.as_ref().is_some_and(|value| !value.is_empty()));
        assert!(elevation.layers.len() >= 1);
        assert!(elevation.shadows.as_ref().is_some_and(|shadows| !shadows.is_empty()));
        assert!(elevation.reserved_shadow_extent.value_px > 0.0);
    }

    #[test]
    fn inspect_elevation_primary_has_no_shadow() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let elevation = inspect_button_elevation(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            InteractionState::default(),
        );

        assert!(!elevation.applied);
        assert_eq!(elevation.rule_shadow, "none");
        assert!(elevation.token.is_none());
        assert!(elevation.layers.is_empty());
        assert_eq!(elevation.reserved_shadow_extent.value_px, 0.0);
    }

    #[test]
    fn inspect_elevation_disabled_suppresses_applied_shadow() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let elevation = inspect_button_elevation(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Outline,
            ButtonFamilyRole::Text,
            InteractionState { disabled: true, ..InteractionState::default() },
        );

        assert!(!elevation.applied);
        assert_eq!(elevation.rule_shadow, "shadow-xs");
        assert!(!elevation.layers.is_empty());
    }

    #[test]
    fn inspect_typography_uses_font_sans_catalog_and_resolved_button_size() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let typography = inspect_button_typography(&mode, ThemeMode::Light);

        assert!(matches!(
            typography.font_family.source,
            crate::TypographySource::CssVar { ref token } if token == "font-sans"
        ));
        assert_eq!(typography.font_family.value, "Outfit");
        assert!(matches!(
            typography.font_size.source,
            crate::TypographySource::Constant { ref label } if label.contains("common.button.sizes.md")
        ));
        assert_eq!(typography.font_size.value, "14");
        assert_eq!(typography.line_height.value.parse::<f32>().unwrap(), 14.0 * (18.0 / 12.5));
    }
}

impl From<crate::tables::metrics::ButtonMetricTable> for ButtonInspectMetrics {
    fn from(table: crate::tables::metrics::ButtonMetricTable) -> Self {
        Self {
            height: table.height,
            icon_size: table.icon_size,
            padding_x: table.padding_x,
            padding_y: table.padding_y,
            gap: table.gap,
            radius: table.radius,
            border_width: table.border_width,
            focus_ring_width: table.focus_ring_width,
            focus_ring_offset: table.focus_ring_offset,
        }
    }
}
