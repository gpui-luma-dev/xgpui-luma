use serde::Deserialize;
use std::collections::HashMap;

use xgpui_luma::controls::button_family::ButtonFamilyLook;
use xgpui_luma::controls::button_family::ButtonFamilyPalette;
use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::controls::button_family::compose_button_family_look;
use crate::size::ShadcnSize;
use xgpui_luma::theme::InteractionLayer;
use xgpui_luma::theme::InteractionState;
use xgpui_luma::theme::StandardBoxScale;
use xgpui_luma::theme::ThemeMode;
use xgpui_luma::theme::snap_to_pixel;

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::LookResolver;
use crate::provenance::ResolvedColor;
use crate::shadow::parse_shadow_token;
use crate::stylesheet::StylesheetConfig;
use crate::stylesheet::find_button_color_rule;
use crate::stylesheet::find_button_elevation_rule;
use crate::stylesheet::resolve_button_color_rule;
use crate::stylesheet::resolve_button_metrics_rule;
use crate::stylesheet::resolve_layered_elevation_shadow;
use crate::stylesheet::resolve_stylesheet_shadow_token;

/// Button-local corner radius presets (shadcn/Shadcn `radius` prop).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ButtonRadiusPreset {
    None,
    Small,
    Medium,
    Large,
    Full,
}

impl ButtonRadiusPreset {
    pub const ALL: [Self; 5] = [Self::None, Self::Small, Self::Medium, Self::Large, Self::Full];

    /// The `.lss` `radius` attribute value.
    pub fn key(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
            Self::Full => "full",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::None => "No radius",
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
            Self::Full => "Full",
        }
    }
}

pub fn resolve_button_radius_preset(preset: ButtonRadiusPreset, radii: &crate::ShadcnRadii, height: f32) -> f32 {
    match preset {
        ButtonRadiusPreset::Full => height / 2.0,
        preset => radii.get(preset),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShadcnButtonStyle {
    Primary,
    Secondary,
    Outline,
    Ghost,
    ContentOnly,
}

#[derive(Clone, Debug)]
pub struct ButtonColorPalette {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: Option<ResolvedColor>,
}

impl ButtonColorPalette {
    pub fn fallback() -> Self {
        Self {
            background: ResolvedColor::transparent(),
            foreground: ResolvedColor::fallback_foreground(),
            border: None,
        }
    }
}

pub fn resolve_button_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
    layer: InteractionLayer,
    theme_mode: ThemeMode,
    selected: bool,
) -> anyhow::Result<ButtonColorPalette> {
    let rule = find_button_color_rule(stylesheet, style, layer, theme_mode, selected)
        .ok_or_else(|| anyhow::anyhow!("no matching button color rule"))?;

    let colors = resolve_button_color_rule(resolver, rule)?;
    Ok(ButtonColorPalette { background: colors.background, foreground: colors.foreground, border: colors.border })
}

// Mirrors the public semantic resolver with an additional selected stylesheet.
#[allow(clippy::too_many_arguments)]
pub(crate) fn button_look_with_stylesheet(
    mode: &ShadcnModeTokens,
    stylesheet: &StylesheetConfig,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    size: ShadcnSize,
    radius: Option<ButtonRadiusPreset>,
    state: InteractionState,
) -> ButtonFamilyLook {
    if let ButtonFamilyRole::Toggle { selected } = role {
        return super::toggle::toggle_look_with_stylesheet(
            mode, stylesheet, theme_mode, style, selected, size, radius, state,
        );
    }

    let ctx = LookContext::new(mode, theme_mode, state);
    let palette = button_palette(&ctx, stylesheet, style, role, size);
    let scale = button_box_scale(&ctx, stylesheet, size, 1.0);
    let effective_style = effective_button_style(style, role);
    let mut look = compose_button_family_look(&palette, role, &scale, ctx.radii().full);
    if has_family_icon_override(stylesheet, size, false) {
        look.icon_size = scale.icon_size;
    }
    if let Some(radius) = radius {
        look.radius = resolve_button_radius_preset(radius, ctx.radii(), look.height);
    }
    look.shadow = button_elevation_shadow(&ctx, stylesheet, effective_style);
    look
}

pub(crate) fn toggle_elevation_shadow(
    catalog: &crate::catalog::CssTokenMap,
    stylesheet: &StylesheetConfig,
    layer: InteractionLayer,
) -> Option<Vec<gpui::BoxShadow>> {
    let token = resolve_layered_elevation_shadow(&stylesheet.toggle.elevation_rules, layer)?;
    let shadows = parse_shadow_token(catalog, &token).ok()?;
    if shadows.is_empty() { None } else { Some(shadows) }
}

pub(crate) fn button_elevation_shadow(
    ctx: &LookContext,
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
) -> Option<Vec<gpui::BoxShadow>> {
    let rule = find_button_elevation_rule(stylesheet, style)?;
    let token = resolve_stylesheet_shadow_token(&rule.shadow)?;
    let shadows = parse_shadow_token(ctx.catalog(), &token).ok()?;
    if shadows.is_empty() { None } else { Some(shadows) }
}

pub fn button_box_scale(
    ctx: &LookContext,
    stylesheet: &StylesheetConfig,
    size: ShadcnSize,
    scale_factor: f32,
) -> StandardBoxScale {
    let (geometry, radius) = resolve_family_geometry(ctx, stylesheet, size, false, scale_factor);
    StandardBoxScale {
        height: geometry.height.value_px,
        padding_x: geometry.padding_x.value_px,
        padding_y: geometry.padding_y.value_px,
        gap: geometry.gap.value_px,
        radius,
        icon_size: geometry.icon_size.value_px,
    }
}

pub(crate) fn has_family_icon_override(stylesheet: &StylesheetConfig, size: ShadcnSize, toggle: bool) -> bool {
    let key = crate::tables::metrics::helpers::control_size_key(size);
    if toggle {
        stylesheet.toggle.metrics_for_size(size).is_some()
            || stylesheet.common.toggle.geometry.icon_size.is_some()
            || stylesheet.common.toggle.sizes.get(key).is_some_and(|value| value.icon_size.is_some())
    } else {
        stylesheet.button.metrics_for_size(size).is_some()
            || stylesheet.common.button.geometry.icon_size.is_some()
            || stylesheet.common.button.sizes.get(key).is_some_and(|value| value.icon_size.is_some())
    }
}

pub(crate) fn resolve_family_geometry(
    ctx: &LookContext,
    stylesheet: &StylesheetConfig,
    size: ShadcnSize,
    toggle: bool,
    scale_factor: f32,
) -> (xgpui_luma::theme::stylesheet::ResolvedButtonGeometry, f32) {
    use xgpui_luma::theme::stylesheet::{ButtonGeometry, ResolvedButtonGeometry, ToggleGeometry};
    let scale = StandardBoxScale::compute(&ctx.tokens.metrics_for(size), scale_factor);
    let typography = ctx.tokens.typography.text.label;
    let mut fallback = ButtonGeometry {
        height: scale.height,
        padding_x: scale.padding_x,
        padding_y: scale.padding_y,
        gap: scale.gap,
        icon_size: scale.icon_size,
        font_size: typography.size,
        line_height: typography.line_height,
    };
    let mut radius = scale.radius;
    let rule = if toggle {
        stylesheet.toggle.metrics_for_size(size)
    } else {
        stylesheet.button.metrics_for_size(size)
    };
    if let Some(rule) = rule {
        let metrics = resolve_button_metrics_rule(rule, ctx.controls(), size);
        fallback.height = metrics.height;
        fallback.padding_x = metrics.padding_horizontal;
        fallback.icon_size = metrics.icon_size;
        fallback.font_size = metrics.font_size;
        radius = metrics.corner_radius;
    }
    let key = crate::tables::metrics::helpers::control_size_key(size);
    let tokens = if toggle {
        &stylesheet.toggle.tokens
    } else {
        &stylesheet.button.tokens
    };
    let mut height_source = None;
    if let Some(tokens) = tokens.get(key) {
        if let Some(height) = &tokens.height {
            if let Some(value) = crate::stylesheet::resolve_stylesheet_metric(height, ctx.controls(), size) {
                fallback.height = value;
                height_source = Some(height);
            }
        }
        if let Some(token) = &tokens.corner_radius {
            radius = crate::stylesheet::resolve_stylesheet_metric(token, ctx.controls(), size).unwrap_or(radius);
        }
    }
    let mut geometry = if toggle {
        let value = stylesheet.common.toggle.inspect_geometry_for(
            key,
            ToggleGeometry {
                height: fallback.height,
                padding_x: fallback.padding_x,
                padding_y: fallback.padding_y,
                gap: fallback.gap,
                icon_size: fallback.icon_size,
                font_size: fallback.font_size,
                line_height: fallback.line_height,
            },
        );
        ResolvedButtonGeometry {
            height: value.height,
            padding_x: value.padding_x,
            padding_y: value.padding_y,
            gap: value.gap,
            icon_size: value.icon_size,
            font_size: value.font_size,
            line_height: value.line_height,
        }
    } else {
        stylesheet.common.button.inspect_geometry_for(key, fallback)
    };
    let family = if toggle { "toggle" } else { "button" };
    if matches!(geometry.height.source, xgpui_luma::theme::provenance::MetricSource::Constant { .. }) {
        if let Some(token) = height_source {
            geometry.height.source = xgpui_luma::theme::provenance::MetricSource::Derived {
                note: format!("stylesheet · {family}.tokens.{key}.height → {token}"),
            };
        }
    }
    for metric in [
        &mut geometry.height,
        &mut geometry.padding_x,
        &mut geometry.padding_y,
        &mut geometry.gap,
        &mut geometry.icon_size,
    ] {
        metric.value_px = snap_to_pixel(metric.value_px, scale_factor);
    }
    (geometry, radius)
}

pub(crate) fn resolve_button_palette_with_stylesheet(
    ctx: &LookContext,
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
) -> ButtonColorPalette {
    let style = effective_button_style(style, role);
    let layer = ctx.state.layer();
    let theme_mode = ctx.theme_mode;
    let selected =
        matches!(role, ButtonFamilyRole::Toggle { selected: true }) && style != ShadcnButtonStyle::ContentOnly;

    let resolver = LookResolver::new(ctx.catalog(), theme_mode, "button_resolver").with_stylesheet(stylesheet);
    let colors = resolve_button_colors_with_stylesheet(&resolver, stylesheet, style, layer, theme_mode, selected)
        .unwrap_or_else(|_| ButtonColorPalette::fallback());
    colors
}

pub fn button_palette(
    ctx: &LookContext,
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    size: ShadcnSize,
) -> ButtonFamilyPalette {
    let colors = resolve_button_palette_with_stylesheet(ctx, stylesheet, style, role);

    let background = colors.background.hsla();
    let foreground = colors.foreground.hsla();
    let border = colors.border.map(|color| color.hsla());

    let typography = crate::tables::typography::resolve_control_typography_with_stylesheet(
        ctx.tokens,
        stylesheet,
        size,
        matches!(role, ButtonFamilyRole::Toggle { .. }),
    );

    ButtonFamilyPalette {
        focus_border: if !matches!(role, ButtonFamilyRole::Toggle { .. })
            && effective_button_style(style, role) == ShadcnButtonStyle::ContentOnly
        {
            gpui::transparent_black()
        } else {
            crate::focus::focus_ring_or_fallback(ctx.catalog())
        },
        background,
        foreground,
        muted_foreground: ctx.palette().app_muted_foreground,
        border,
        typography: typography.style,
        font_family: typography.font_family,
    }
}

fn effective_button_style(style: ShadcnButtonStyle, role: ButtonFamilyRole) -> ShadcnButtonStyle {
    effective_button_style_for_role(style, role)
}

pub(crate) fn effective_button_style_for_role(style: ShadcnButtonStyle, role: ButtonFamilyRole) -> ShadcnButtonStyle {
    if matches!(role, ButtonFamilyRole::Toggle { selected: false }) {
        match style {
            ShadcnButtonStyle::Ghost | ShadcnButtonStyle::Outline | ShadcnButtonStyle::ContentOnly => style,
            _ => ShadcnButtonStyle::Outline,
        }
    } else {
        style
    }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use super::*;
    use crate::stylesheet::embedded_stylesheet;
    use xgpui_luma::controls::button_family::ButtonFamilyRole;
    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use xgpui_luma::theme::ThemeMode;

    #[cfg(feature = "test-support")]
    #[test]
    fn text_and_popup_focus_geometry_is_stable() {
        use xgpui_luma::infra::focus_adorner::testing::*;
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let look = crate::ShadcnLook::from_css_str(crate::FALLBACK_CSS).unwrap();
            look.set_mode(mode);
            for theme in [
                look.textfield_theme(),
                look.input_textfield_theme(),
                look.surface_textfield_theme(),
                look.primary_textfield_theme(),
            ] {
                assert_textfield_layout(theme);
            }
            for theme in [
                look.textarea_theme(),
                look.input_textarea_theme(),
                look.surface_textarea_theme(),
                look.primary_textarea_theme(),
            ] {
                assert_textarea_layout(theme);
            }
            assert_selector_layout(look.selector_theme());
            assert_search_selector_layout(look.selector_theme());
            assert_popup_layout(look.popup_menu_theme());
        }
    }

    #[cfg(feature = "test-support")]
    #[test]
    fn choice_focus_geometry_is_stable_across_styles() {
        use xgpui_luma::controls::button::ButtonRenderModel;
        use xgpui_luma::infra::focus_adorner::testing::assert_stable_layout;
        use gpui::IntoElement;
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let look = crate::ShadcnLook::from_css_str(crate::FALLBACK_CSS).unwrap();
            look.set_mode(mode);
            for style in [
                ShadcnButtonStyle::Primary,
                ShadcnButtonStyle::Secondary,
                ShadcnButtonStyle::Outline,
                ShadcnButtonStyle::Ghost,
                ShadcnButtonStyle::ContentOnly,
            ] {
                let template = super::super::templates::checkbox_template(look.clone(), style);
                assert_stable_layout(move |state, selected, window, cx| {
                    template
                        .render(
                            &ButtonRenderModel {
                                data: xgpui_luma::controls::checkbox::CheckboxData::new(selected),
                                state,
                                ..Default::default()
                            },
                            window,
                            cx,
                        )
                        .into_any_element()
                });
                let template = super::super::templates::radio_button_template(look.clone(), style);
                assert_stable_layout(move |state, selected, window, cx| {
                    template
                        .render(
                            &ButtonRenderModel {
                                data: xgpui_luma::controls::radio_button::RadioButtonData::new(selected),
                                state,
                                ..Default::default()
                            },
                            window,
                            cx,
                        )
                        .into_any_element()
                });
                let template = super::super::templates::switch_template(look.clone(), style);
                assert_stable_layout(move |state, selected, window, cx| {
                    template
                        .render(
                            &ButtonRenderModel {
                                data: xgpui_luma::controls::switch::SwitchData::new(selected),
                                state,
                                ..Default::default()
                            },
                            window,
                            cx,
                        )
                        .into_any_element()
                });
            }
        }
    }

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
    fn palette_typography_uses_stylesheet_font_size() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let ctx = LookContext::new(&mode, ThemeMode::Light, InteractionState::default());
        let palette = button_palette(
            &ctx,
            embedded_stylesheet(),
            ShadcnButtonStyle::Primary,
            ButtonFamilyRole::Text,
            ShadcnSize::Sm,
        );

        assert!((palette.typography.size - 12.0).abs() < f32::EPSILON);
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct ButtonStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, ButtonMetricsRule>,
    /// Look-owned metric-token references; literals live in the common contract.
    #[serde(default)]
    pub tokens: HashMap<String, ButtonMetricTokensRule>,
    #[serde(default)]
    pub elevation_rules: Vec<ButtonElevationRule>,
    #[serde(default)]
    pub color_rules: Vec<ButtonColorRule>,
}

impl ButtonStylesheet {
    /// Native bundled rules in their original priority order.
    pub fn native_default() -> Self {
        let mut color_rules = Vec::new();
        for (style, foreground, base, hover, pressed) in [
            ("primary", "primary-foreground", "primary", "primary-hover", "primary-pressed"),
            ("secondary", "secondary-foreground", "secondary", "secondary-hover", "secondary-pressed"),
        ] {
            for (layer, background, label) in [
                ("default", base, foreground),
                ("hover", hover, foreground),
                ("pressed", pressed, foreground),
                ("disabled", "muted", "muted-foreground"),
            ] {
                color_rules.push(ButtonColorRule {
                    style: Some(style.into()),
                    layer: Some(layer.into()),
                    mode: None,
                    selected: None,
                    background: background.into(),
                    foreground: label.into(),
                    border: Some("@background".into()),
                });
            }
        }
        for selected in [true, false] {
            for layer in ["default", "hover", "pressed"] {
                for mode in ["light", "dark"] {
                    let background = match (selected, layer, mode) {
                        (true, "default", _) | (_, _, "light") if selected || layer != "default" => "accent",
                        (false, "default", "light") => "background",
                        (false, "default", _) => "input/30",
                        _ => "input/50",
                    };
                    color_rules.push(ButtonColorRule {
                        style: Some("outline".into()),
                        layer: Some(layer.into()),
                        mode: Some(mode.into()),
                        selected: Some(selected),
                        background: background.into(),
                        foreground: if selected || layer != "default" {
                            "accent-foreground"
                        } else {
                            "foreground"
                        }
                        .into(),
                        border: Some(if mode == "light" { "border" } else { "input" }.into()),
                    });
                }
            }
        }
        color_rules.push(ButtonColorRule {
            style: Some("outline".into()),
            layer: Some("disabled".into()),
            mode: None,
            selected: None,
            background: "transparent".into(),
            foreground: "muted-foreground".into(),
            border: Some("border".into()),
        });
        for selected in [true, false] {
            color_rules.push(ButtonColorRule {
                style: Some("ghost".into()),
                layer: Some("default".into()),
                mode: None,
                selected: Some(selected),
                background: if selected { "accent" } else { "transparent" }.into(),
                foreground: if selected { "accent-foreground" } else { "foreground" }.into(),
                border: None,
            });
            for layer in ["hover", "pressed"] {
                for mode in ["light", "dark"] {
                    color_rules.push(ButtonColorRule {
                        style: Some("ghost".into()),
                        layer: Some(layer.into()),
                        mode: Some(mode.into()),
                        selected: Some(selected),
                        background: if mode == "light" { "accent" } else { "accent/50" }.into(),
                        foreground: "accent-foreground".into(),
                        border: None,
                    });
                }
            }
        }
        color_rules.push(ButtonColorRule {
            style: Some("ghost".into()),
            layer: Some("disabled".into()),
            mode: None,
            selected: None,
            background: "transparent".into(),
            foreground: "muted-foreground".into(),
            border: None,
        });
        for layer in ["default", "hover", "pressed", "disabled"] {
            color_rules.push(ButtonColorRule {
                style: Some("content-only".into()),
                layer: Some(layer.into()),
                mode: None,
                selected: None,
                background: "transparent".into(),
                foreground: "foreground".into(),
                border: Some("transparent".into()),
            });
        }
        color_rules.push(ButtonColorRule {
            style: None,
            layer: None,
            mode: None,
            selected: None,
            background: "transparent".into(),
            foreground: "foreground".into(),
            border: None,
        });
        Self {
            tokens: [("sm", "metrics.control.sm"), ("md", "metrics.control.md"), ("lg", "metrics.control.lg")]
                .into_iter()
                .map(|(key, height)| {
                    (
                        key.into(),
                        ButtonMetricTokensRule { height: Some(height.into()), corner_radius: Some("radius".into()) },
                    )
                })
                .collect(),
            elevation_rules: [
                ("primary", "none"),
                ("secondary", "none"),
                ("ghost", "none"),
                ("outline", "shadow-xs"),
                ("content-only", "none"),
            ]
            .into_iter()
            .map(|(style, shadow)| ButtonElevationRule { style: style.into(), shadow: shadow.into() })
            .collect(),
            color_rules,
            ..Default::default()
        }
    }

    pub fn find_color_rule(&self, style: &str, layer: &str, mode: &str, selected: bool) -> Option<&ButtonColorRule> {
        self.color_rules.iter().find(|rule| rule.matches(style, layer, mode, selected))
    }

    pub fn elevation_rule_for_style(&self, style: &str) -> Option<&ButtonElevationRule> {
        self.elevation_rules.iter().find(|rule| rule.style == style)
    }

    pub fn metrics_for_size(&self, size: ShadcnSize) -> Option<&ButtonMetricsRule> {
        let key = match size {
            ShadcnSize::Sm => "sm",
            ShadcnSize::Md => "md",
            ShadcnSize::Lg => "lg",
        };
        self.metrics.get(key)
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ButtonMetricsRule {
    pub height: String,
    pub padding_horizontal: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub corner_radius: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ButtonElevationRule {
    pub style: String,
    pub shadow: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ButtonColorRule {
    pub style: Option<String>,
    pub layer: Option<String>,
    pub mode: Option<String>,
    pub selected: Option<bool>,

    pub background: String,
    pub foreground: String,
    pub border: Option<String>,
}

impl ButtonColorRule {
    pub fn matches(&self, style: &str, layer: &str, mode: &str, selected: bool) -> bool {
        self.style.as_deref().is_none_or(|value| value == "any" || value == style)
            && self.layer.as_deref().is_none_or(|value| value == "any" || value == layer)
            && self.mode.as_deref().is_none_or(|value| value == "any" || value == mode)
            && self.selected.is_none_or(|value| value == selected)
    }
}

/// Shadcn token references retained independently of common pixel dimensions.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ButtonMetricTokensRule {
    pub height: Option<String>,
    pub corner_radius: Option<String>,
}

/// Native common geometry owned by this control.
pub(crate) fn native_button_stylesheet() -> xgpui_luma::theme::stylesheet::ButtonStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    ButtonStylesheet {
        sizes: [("sm", 12.0, 12.0, 14.0), ("md", 16.0, 14.0, 16.0), ("lg", 20.0, 16.0, 18.0)]
            .into_iter()
            .map(|(key, padding, font, icon)| {
                (
                    key.into(),
                    ButtonGeometryStylesheet {
                        padding_x: Some(padding),
                        font_size: Some(font),
                        icon_size: Some(icon),
                        ..Default::default()
                    },
                )
            })
            .collect(),
        ..Default::default()
    }
}
