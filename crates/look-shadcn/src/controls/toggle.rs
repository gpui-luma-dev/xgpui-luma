use crate::controls::button::{ButtonMetricsRule, ButtonMetricTokensRule};
use crate::stylesheet::config::LayeredElevationRule;
use serde::Deserialize;
use xgpui_luma::theme::InteractionLayer;
use std::collections::HashMap;
use xgpui_luma::controls::button_family::{
    ButtonFamilyLook, ButtonFamilyPalette, ButtonFamilyRole, compose_button_family_look,
};
use xgpui_luma::theme::{InteractionState, StandardBoxScale, ThemeMode};

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::stylesheet::{StylesheetConfig};
use crate::size::ShadcnSize;

use super::button::{
    ButtonRadiusPreset, ShadcnButtonStyle, button_elevation_shadow, resolve_button_radius_preset,
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

// Mirrors the public semantic resolver with an additional selected stylesheet.
#[allow(clippy::too_many_arguments)]
pub(crate) fn toggle_look_with_stylesheet(
    mode: &ShadcnModeTokens,
    stylesheet: &StylesheetConfig,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    selected: bool,
    size: ShadcnSize,
    radius: Option<ButtonRadiusPreset>,
    state: InteractionState,
) -> ButtonFamilyLook {
    let role = ButtonFamilyRole::Toggle { selected };
    let ctx = LookContext::new(mode, theme_mode, state);
    let palette = toggle_palette(&ctx, stylesheet, style, selected, size);
    let scale = toggle_box_scale(&ctx, stylesheet, size, 1.0);
    let mut look = compose_button_family_look(&palette, role, &scale, ctx.radii().full);
    if super::button::has_family_icon_override(stylesheet, size, true) {
        look.icon_size = scale.icon_size;
    }
    if let Some(radius) = radius {
        look.radius = resolve_button_radius_preset(radius, ctx.radii(), look.height);
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

// Mirrors the public semantic resolver with an additional selected stylesheet.

pub fn toggle_box_scale(
    ctx: &LookContext,
    stylesheet: &StylesheetConfig,
    size: ShadcnSize,
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
    size: ShadcnSize,
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

    pub fn metrics_for_size(&self, size: ShadcnSize) -> Option<&ButtonMetricsRule> {
        let key = match size {
            ShadcnSize::Sm => "sm",
            ShadcnSize::Md => "md",
            ShadcnSize::Lg => "lg",
        };
        self.metrics.get(key)
    }
}
