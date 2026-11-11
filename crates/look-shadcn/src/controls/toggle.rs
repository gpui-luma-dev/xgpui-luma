use crate::controls::button::{ButtonMetricsRule, ButtonMetricTokensRule};
use crate::stylesheet::config::LayeredElevationRule;
use serde::Deserialize;
use xgpui_luma::theme::InteractionLayer;
use std::collections::HashMap;
use xgpui_luma::controls::button_family::{
    ButtonFamilyLook, ButtonFamilyPalette, ButtonFamilyRole, compose_button_family_look,
};
use xgpui_luma::theme::{ControlSize, InteractionState, StandardBoxScale, ThemeMode};

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::stylesheet::{StylesheetConfig};

use super::button::{
    ButtonRadiusPreset, ShadcnButtonStyle, button_box_scale, button_elevation_shadow, resolve_button_radius_preset,
    toggle_elevation_shadow,
};

/// Inline padded toggle (shadcn default). Square geometry requires explicit `.round(true)` on the builder.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ToggleLayout {
    #[default]
    Inline,
    Square,
}

impl ToggleLayout {
    pub fn from_round(round: bool) -> Self {
        if round { Self::Square } else { Self::Inline }
    }
}

pub fn toggle_look(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    selected: bool,
    size: ControlSize,
    state: InteractionState,
) -> ButtonFamilyLook {
    toggle_look_semantic(mode, theme_mode, style, selected, size, None, state)
}

pub fn toggle_look_semantic(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    selected: bool,
    size: ControlSize,
    radius: Option<ButtonRadiusPreset>,
    state: InteractionState,
) -> ButtonFamilyLook {
    toggle_look_with_stylesheet(mode, mode.stylesheet(), theme_mode, style, selected, size, radius, state)
}

// Mirrors the public semantic resolver with an additional selected stylesheet.
#[allow(clippy::too_many_arguments)]
pub(crate) fn toggle_look_with_stylesheet(
    mode: &ShadcnModeTokens,
    stylesheet: &StylesheetConfig,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    selected: bool,
    size: ControlSize,
    radius: Option<ButtonRadiusPreset>,
    state: InteractionState,
) -> ButtonFamilyLook {
    let role = ButtonFamilyRole::Toggle { selected };
    let ctx = LookContext::new(mode, theme_mode, state);
    let palette = toggle_palette(&ctx, stylesheet, style, selected, size);
    let scale = toggle_box_scale(&ctx, stylesheet, size, 1.0);
    let mut look = compose_button_family_look(&palette, role, &scale, ctx.metrics().radius.pill);
    if super::button::has_family_icon_override(stylesheet, size, true) {
        look.icon_size = scale.icon_size;
    }
    if let Some(radius) = radius {
        look.radius = resolve_button_radius_preset(radius, ctx.metrics(), look.height);
    }
    look.shadow = toggle_shadow(&ctx, stylesheet, style);
    look
}

fn toggle_shadow(
    ctx: &LookContext,
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
) -> Option<Vec<gpui::BoxShadow>> {
    match style {
        ShadcnButtonStyle::Outline | ShadcnButtonStyle::Ghost => button_elevation_shadow(ctx, stylesheet, style),
        ShadcnButtonStyle::ContentOnly => None,
        _ => toggle_elevation_shadow(ctx.catalog(), stylesheet, ctx.state.layer()),
    }
}

/// Square icon toggle sizing (matches icon-button metrics); radius preset controls corners.
pub fn toggle_icon_look_semantic(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    selected: bool,
    size: ControlSize,
    radius: Option<ButtonRadiusPreset>,
    state: InteractionState,
) -> ButtonFamilyLook {
    toggle_icon_look_with_stylesheet(mode, mode.stylesheet(), theme_mode, style, selected, size, radius, state)
}

// Mirrors the public semantic resolver with an additional selected stylesheet.
#[allow(clippy::too_many_arguments)]
pub(crate) fn toggle_icon_look_with_stylesheet(
    mode: &ShadcnModeTokens,
    stylesheet: &StylesheetConfig,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    selected: bool,
    size: ControlSize,
    radius: Option<ButtonRadiusPreset>,
    state: InteractionState,
) -> ButtonFamilyLook {
    let role = ButtonFamilyRole::Toggle { selected };
    let ctx = LookContext::new(mode, theme_mode, state);
    let palette = toggle_palette(&ctx, stylesheet, style, selected, size);
    let mut scale = button_box_scale(&ctx, stylesheet, size, 1.0);
    scale.padding_x = 0.0;
    scale.padding_y = 0.0;
    let mut look = compose_button_family_look(&palette, role, &scale, ctx.metrics().radius.pill);
    if super::button::has_family_icon_override(stylesheet, size, false) {
        look.icon_size = scale.icon_size;
    }
    if let Some(radius) = radius {
        look.radius = resolve_button_radius_preset(radius, ctx.metrics(), look.height);
    }
    look.shadow = toggle_shadow(&ctx, stylesheet, style);
    look
}

pub fn toggle_box_scale(
    ctx: &LookContext,
    stylesheet: &StylesheetConfig,
    size: ControlSize,
    scale_factor: f32,
) -> StandardBoxScale {
    let (geometry, radius) = super::button::resolve_family_geometry(ctx, stylesheet, size, true, scale_factor);
    StandardBoxScale {
        height: geometry.height.value_px,
        padding_x: geometry.padding_x.value_px,
        padding_y: geometry.padding_y.value_px,
        gap: geometry.gap.value_px,
        radius,
        icon_size: geometry.icon_size.value_px,
    }
}

pub fn toggle_palette(
    ctx: &LookContext,
    stylesheet: &StylesheetConfig,
    style: ShadcnButtonStyle,
    selected: bool,
    size: ControlSize,
) -> ButtonFamilyPalette {
    let role = ButtonFamilyRole::Toggle { selected };
    let colors = super::button::resolve_button_palette_with_stylesheet(ctx, stylesheet, style, role);

    let background = colors.background.hsla();
    let foreground = colors.foreground.hsla();
    let border = colors.border.map(|color| color.hsla());

    let typography =
        crate::tables::typography::resolve_control_typography_with_stylesheet(ctx.tokens, stylesheet, size, true);

    ButtonFamilyPalette {
        focus_border: super::button::button_palette(ctx, stylesheet, style, role, size).focus_border,
        background,
        foreground,
        muted_foreground: ctx.palette().app_muted_foreground,
        border,
        typography: typography.style,
        font_family: typography.font_family,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::controls::button::button_look;
    use crate::catalog::CssTokenMap;
    use crate::controls::button::ButtonRadiusPreset;
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
    fn toggle_look_uses_toggle_metrics_not_button_metrics() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let sm = toggle_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Secondary,
            true,
            ControlSize::Sm,
            InteractionState::default(),
        );
        let lg = toggle_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Secondary,
            true,
            ControlSize::Lg,
            InteractionState::default(),
        );

        assert!((sm.height - 36.0).abs() < f32::EPSILON);
        assert!((sm.padding_x - 10.0).abs() < f32::EPSILON);
        assert!((lg.height - 44.0).abs() < f32::EPSILON);
        assert!((lg.padding_x - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn toggle_inline_look_has_horizontal_padding() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let look = toggle_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Secondary,
            false,
            ControlSize::Md,
            InteractionState::default(),
        );

        assert!(look.padding_x > 0.0);
    }

    #[test]
    fn focused_toggle_uses_independent_ring_paint() {
        let catalog = retro_arcade_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let focused = toggle_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::ContentOnly,
            false,
            ControlSize::Md,
            InteractionState { focused: true, ..InteractionState::default() },
        );

        assert_eq!(focused.focus_border, catalog.color("ring").expect("ring"));
    }

    #[test]
    fn outline_toggle_selected_uses_selected_button_colors() {
        let catalog = retro_arcade_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let icon = button_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Outline,
            ButtonFamilyRole::Icon,
            ControlSize::Md,
            InteractionState::default(),
        );
        let toggle = toggle_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Outline,
            true,
            ControlSize::Md,
            InteractionState::default(),
        );

        assert_ne!(toggle.background, icon.background);
        assert_ne!(toggle.foreground, icon.foreground);
        assert_eq!(toggle.shadow, icon.shadow);
    }

    #[test]
    fn ghost_toggle_selected_uses_selected_button_colors() {
        let catalog = retro_arcade_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let icon = button_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Ghost,
            ButtonFamilyRole::Icon,
            ControlSize::Md,
            InteractionState::default(),
        );
        let toggle = toggle_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Ghost,
            true,
            ControlSize::Md,
            InteractionState::default(),
        );

        assert_ne!(toggle.background, icon.background);
        assert_ne!(toggle.foreground, icon.foreground);
        assert_eq!(toggle.border, icon.border);
        assert!(toggle.shadow.is_none());
        assert!(icon.shadow.is_none());
    }

    #[test]
    fn toggle_icon_look_uses_button_metrics_not_toggle_inline_metrics() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let inline = toggle_look(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Secondary,
            true,
            ControlSize::Md,
            InteractionState::default(),
        );
        let icon = toggle_icon_look_semantic(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Primary,
            true,
            ControlSize::Md,
            None,
            InteractionState::default(),
        );

        assert!(inline.padding_x > 0.0);
        assert_eq!(icon.padding_x, 0.0);
        assert_eq!(icon.padding_y, 0.0);
        assert_ne!(inline.height, icon.height);
    }

    #[test]
    fn toggle_look_resolves_radius_preset() {
        let mode = ShadcnModeTokens::from_catalog(retro_arcade_catalog(), ThemeMode::Light).expect("catalog");
        let full = toggle_look_semantic(
            &mode,
            ThemeMode::Light,
            ShadcnButtonStyle::Secondary,
            true,
            ControlSize::Md,
            Some(ButtonRadiusPreset::Full),
            InteractionState::default(),
        );

        assert!((full.radius - full.height / 2.0).abs() < f32::EPSILON);
    }
}

/// Native common geometry owned by this control.
pub(crate) fn native_toggle_stylesheet() -> xgpui_luma::theme::stylesheet::ToggleStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    ToggleStylesheet {
        sizes: [("sm", 10.0, 12.0, 14.0, 36.0), ("md", 12.0, 14.0, 16.0, 40.0), ("lg", 20.0, 16.0, 18.0, 44.0)]
            .into_iter()
            .map(|(key, padding, font, icon, height)| {
                (
                    key.into(),
                    ToggleGeometryStylesheet {
                        padding_x: Some(padding),
                        font_size: Some(font),
                        icon_size: Some(icon),
                        height: Some(height),
                        ..Default::default()
                    },
                )
            })
            .collect(),
        ..Default::default()
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct ToggleStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, ButtonMetricsRule>,
    /// Look-owned metric-token references; literals live in the common contract.
    #[serde(default)]
    pub tokens: HashMap<String, ButtonMetricTokensRule>,
    #[serde(default)]
    pub elevation_rules: Vec<LayeredElevationRule>,
}

impl ToggleStylesheet {
    /// Native bundled toggle token references and elevation.
    pub fn native_default() -> Self {
        Self {
            tokens: ["sm", "md", "lg"]
                .into_iter()
                .map(|key| (key.into(), ButtonMetricTokensRule { height: None, corner_radius: Some("radius".into()) }))
                .collect(),
            elevation_rules: vec![
                LayeredElevationRule { layer: None, shadow: "shadow-sm".into() },
                LayeredElevationRule { layer: Some("disabled".into()), shadow: "none".into() },
            ],
            ..Default::default()
        }
    }

    pub fn elevation_rule_for_layer(&self, layer: InteractionLayer) -> Option<&LayeredElevationRule> {
        if layer == InteractionLayer::Disabled {
            return self.elevation_rules.iter().find(|rule| rule.layer.as_deref() == Some("disabled"));
        }
        self.elevation_rules.iter().find(|rule| rule.layer.is_none())
    }

    pub fn metrics_for_size(&self, size: ControlSize) -> Option<&ButtonMetricsRule> {
        let key = match size {
            ControlSize::Sm => "sm",
            ControlSize::Md => "md",
            ControlSize::Lg => "lg",
        };
        self.metrics.get(key)
    }
}
