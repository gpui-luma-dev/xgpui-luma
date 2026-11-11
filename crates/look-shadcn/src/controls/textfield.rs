//! Text field property mappings for shadcn-inspired inputs:
//!
//! **Outline** (shadcn Input): `border-border`, light transparent fill, dark `input/30`,
//! `selection:bg-primary`, `selection:text-primary-foreground`, `placeholder:text-muted-foreground`.
//!
//! **Input** (selector / combobox triggers): same fill as Outline (`transparent` light,
//! `input/30` dark; hover `input/50` in selector palette). Opaque controls use `border-border`.
//! Used by selector, combobox, search_selector, and autocomplete — not standalone text fields.
//!
//! **Primary**: bordered control with opaque `background` fill and `shadow-xs` elevation.
//!
//! **Surface**: filled `muted` background, no border, same text/selection tokens.

use serde::Deserialize;

use xgpui_luma::controls::textfield::TextFieldLook;
use xgpui_luma::controls::textfield::TextFieldPalette;
use xgpui_luma::controls::textfield::TextFieldState;
use xgpui_luma::controls::textfield::compose_textfield_look;
use xgpui_luma::theme::ControlSize;
use xgpui_luma::theme::InteractionState;
use xgpui_luma::theme::LumaTextStyle;
use xgpui_luma::theme::StandardBoxScale;
use xgpui_luma::theme::ThemeMode;

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::LookResolver;
use crate::provenance::ResolvedColor;
use crate::shadow::parse_shadow_token;
use crate::stylesheet::StylesheetConfig;
use crate::stylesheet::find_textfield_color_rule;
use crate::stylesheet::find_textfield_elevation_rule;
use crate::stylesheet::resolve_textfield_color_rule;
use crate::stylesheet::resolve_stylesheet_shadow_token;

fn textfield_style_key(style: ShadcnTextFieldStyle) -> &'static str {
    match style {
        ShadcnTextFieldStyle::Outline => "outline",
        ShadcnTextFieldStyle::Input => "input",
        ShadcnTextFieldStyle::Primary => "primary",
        ShadcnTextFieldStyle::Surface => "surface",
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ShadcnTextFieldStyle {
    #[default]
    Outline,
    Input,
    Primary,
    Surface,
}

#[derive(Clone, Debug)]
pub struct TextFieldColorTable {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
    pub placeholder: ResolvedColor,
    pub icon: ResolvedColor,
    pub selection_background: ResolvedColor,
    pub selection_foreground: ResolvedColor,
    pub caret: ResolvedColor,
}

impl TextFieldColorTable {
    pub fn fallback() -> Self {
        Self {
            background: ResolvedColor::transparent(),
            foreground: ResolvedColor::fallback_foreground(),
            border: ResolvedColor::fallback_foreground(),
            placeholder: ResolvedColor::fallback_foreground(),
            icon: ResolvedColor::fallback_foreground(),
            selection_background: ResolvedColor::fallback_foreground(),
            selection_foreground: ResolvedColor::fallback_foreground(),
            caret: ResolvedColor::fallback_foreground(),
        }
    }
}

pub fn resolve_textfield_colors(
    resolver: &LookResolver<'_>,
    style: ShadcnTextFieldStyle,
    enabled: bool,
    invalid: bool,
    theme_mode: ThemeMode,
) -> anyhow::Result<TextFieldColorTable> {
    resolve_textfield_colors_with_stylesheet(resolver, resolver.stylesheet(), style, enabled, invalid, theme_mode)
}

pub fn resolve_textfield_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    style: ShadcnTextFieldStyle,
    enabled: bool,
    invalid: bool,
    theme_mode: ThemeMode,
) -> anyhow::Result<TextFieldColorTable> {
    let rule = find_textfield_color_rule(stylesheet, textfield_style_key(style), enabled, invalid, theme_mode)
        .ok_or_else(|| anyhow::anyhow!("no matching textfield color rule"))?;
    let colors = resolve_textfield_color_rule(resolver, rule)?;
    Ok(TextFieldColorTable {
        background: colors.background,
        foreground: colors.foreground,
        border: colors.border,
        placeholder: colors.placeholder,
        icon: colors.icon,
        selection_background: colors.selection_background,
        selection_foreground: colors.selection_foreground,
        caret: colors.caret,
    })
}

pub(crate) fn textfield_elevation_shadow(
    ctx: &LookContext,
    stylesheet: &StylesheetConfig,
    style: ShadcnTextFieldStyle,
) -> Option<Vec<gpui::BoxShadow>> {
    let rule = find_textfield_elevation_rule(stylesheet, style)?;
    let token = resolve_stylesheet_shadow_token(&rule.shadow)?;
    let shadows = parse_shadow_token(ctx.catalog(), &token).ok()?;
    if shadows.is_empty() { None } else { Some(shadows) }
}

/// Final textfield colors, including visible focus and dark input hover.
pub fn resolve_textfield_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnTextFieldStyle,
    state: TextFieldState,
    enabled: bool,
) -> TextFieldColorTable {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let catalog = ctx.catalog();
    let resolver = LookResolver::new(catalog, ctx.theme_mode, "textfield").with_stylesheet(mode.stylesheet());
    let mut colors = resolve_textfield_colors(&resolver, style, enabled, state.invalid, ctx.theme_mode)
        .unwrap_or_else(|_| TextFieldColorTable::fallback());
    if style == ShadcnTextFieldStyle::Input
        && theme_mode == ThemeMode::Dark
        && enabled
        && state.hovered
        && let Ok(hover_fill) = resolver.resolve_decl("input/50")
    {
        colors.background = hover_fill;
    }

    colors
}

pub fn textfield_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnTextFieldStyle,
    state: TextFieldState,
    enabled: bool,
) -> TextFieldPalette {
    textfield_palette_for_size(mode, theme_mode, style, state, enabled, ControlSize::Md)
}

pub fn textfield_palette_for_size(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnTextFieldStyle,
    state: TextFieldState,
    enabled: bool,
    size: ControlSize,
) -> TextFieldPalette {
    let mut palette = textfield_palette_base_for_size(mode, theme_mode, style, state, enabled, size);
    let scale = StandardBoxScale::compute(size, &mode.metrics, 1.0);
    let geometry = textfield_geometry(mode, size, &scale, &palette.typography);
    crate::tables::typography::apply_resolved_geometry_typography(
        &mut palette.typography,
        &geometry.font_size,
        &geometry.line_height,
    );
    palette
}

pub(crate) fn textfield_geometry(
    mode: &ShadcnModeTokens,
    size: ControlSize,
    scale: &StandardBoxScale,
    typography: &LumaTextStyle,
) -> xgpui_luma::theme::stylesheet::ResolvedTextFieldGeometry {
    mode.stylesheet().common.textfield.inspect_geometry(
        crate::tables::metrics::helpers::control_size_key(size),
        xgpui_luma::theme::stylesheet::TextFieldGeometry {
            min_height: scale.height,
            padding_x: scale.padding_x,
            padding_y: scale.padding_y,
            gap: scale.gap,
            icon_size: scale.icon_size,
            font_size: typography.size,
            line_height: typography.line_height,
            ..Default::default()
        },
    )
}

pub(crate) fn textfield_palette_base_for_size(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnTextFieldStyle,
    state: TextFieldState,
    enabled: bool,
    size: ControlSize,
) -> TextFieldPalette {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let stylesheet = mode.stylesheet();
    let typography = ctx.typography();
    let colors = resolve_textfield_palette(mode, theme_mode, style, state, enabled);
    // Keep elevation in the look when disabled so SDK hosts can reserve projection
    // space; templates gate paint with `enabled` / `should_paint_shadow`.
    let shadow = textfield_elevation_shadow(&ctx, stylesheet, style);

    let mut text_style = typography.text.body;
    apply_textfield_control_size_typography(&mut text_style, mode, size);

    TextFieldPalette {
        focus_border: if state.invalid {
            gpui::transparent_black()
        } else {
            crate::focus::focus_ring_or_fallback(&mode.catalog)
        },
        background: colors.background.hsla(),
        foreground: colors.foreground.hsla(),
        border: colors.border.hsla(),
        placeholder: colors.placeholder.hsla(),
        icon: colors.icon.hsla(),
        selection_background: colors.selection_background.hsla(),
        selection_foreground: colors.selection_foreground.hsla(),
        caret: colors.caret.hsla(),
        shadow,
        typography: text_style,
        font_family: typography.font.sans.family.clone().into(),
    }
}

/// Scales text-field typography from shared Button font sizes (same curve as buttons/selectors).
pub fn apply_textfield_control_size_typography(
    typography: &mut LumaTextStyle,
    mode: &ShadcnModeTokens,
    size: ControlSize,
) {
    super::apply_button_metrics_typography(typography, mode, size);
}

pub fn textfield_look(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnTextFieldStyle,
    state: TextFieldState,
    enabled: bool,
    size: ControlSize,
    scale: &StandardBoxScale,
) -> TextFieldLook {
    let palette = textfield_palette_for_size(mode, theme_mode, style, state, enabled, size);
    let geometry = textfield_geometry(mode, size, scale, &palette.typography);
    let mut scale = *scale;
    scale.height = geometry.min_height.value_px;
    scale.padding_x = geometry.padding_x.value_px;
    scale.padding_y = geometry.padding_y.value_px;
    scale.gap = geometry.gap.value_px;
    scale.icon_size = geometry.icon_size.value_px;
    let mut look = compose_textfield_look(&palette, &scale, mode.metrics.border_width.default);
    look.helper_geometry = mode.stylesheet().common.shared_geometry();
    look.caret_width = geometry.caret_width.value_px;
    look.caret_height_extra = geometry.caret_height_extra.value_px;
    look.disabled_opacity = geometry.disabled_opacity.value_px;
    look
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;

    use xgpui_luma::controls::textfield::TextFieldState;
    use xgpui_luma::theme::{ControlSize, ThemeMode};

    use crate::catalog::CssTokenMap;
    use crate::color::with_alpha;
    use crate::mode::ShadcnModeTokens;

    use super::{ShadcnTextFieldStyle, textfield_palette, textfield_palette_for_size};

    fn sample_catalog() -> CssTokenMap {
        CssTokenMap::from_map(BTreeMap::from([
            ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("primary-foreground".into(), "oklch(1 0 0)".into()),
            ("secondary".into(), "oklch(0.6437 0.1019 187.3840)".into()),
            ("secondary-foreground".into(), "oklch(1 0 0)".into()),
            ("background".into(), "oklch(0.9735 0.0261 90.0953)".into()),
            ("card".into(), "oklch(0.98 0.02 90.0953)".into()),
            ("foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("muted".into(), "oklch(0.6979 0.0159 196.7940)".into()),
            ("muted-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("accent-foreground".into(), "oklch(1 0 0)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.7200 0.0120 205.0000)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("shadow-xs".into(), "0 1px 2px 0px hsl(0 0% 0% / 0.05)".into()),
        ]))
    }

    #[test]
    fn input_textfield_light_uses_transparent_fill_and_border_token() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look =
            textfield_palette(&mode, ThemeMode::Light, ShadcnTextFieldStyle::Input, TextFieldState::default(), true);

        assert_eq!(look.background, gpui::hsla(0.0, 0.0, 0.0, 0.0));
        assert_eq!(look.border, catalog.color("border").expect("border"));
        assert!(look.shadow.is_none());
    }

    #[test]
    fn input_textfield_dark_uses_input_fill_and_border_token() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Dark).expect("catalog");
        let look =
            textfield_palette(&mode, ThemeMode::Dark, ShadcnTextFieldStyle::Input, TextFieldState::default(), true);
        let input = catalog.color("input").expect("input");

        assert_eq!(look.background, with_alpha(input, 0.30));
        assert_eq!(look.border, catalog.color("border").expect("border"));
        assert!(look.shadow.is_none());
    }

    #[test]
    fn input_textfield_dark_hover_uses_input_half_fill() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Dark).expect("catalog");
        let default =
            textfield_palette(&mode, ThemeMode::Dark, ShadcnTextFieldStyle::Input, TextFieldState::default(), true);
        let mut hovered = TextFieldState::default();
        hovered.hovered = true;
        let look = textfield_palette(&mode, ThemeMode::Dark, ShadcnTextFieldStyle::Input, hovered, true);
        let input = catalog.color("input").expect("input");

        assert_eq!(default.background, with_alpha(input, 0.30));
        assert_eq!(look.background, with_alpha(input, 0.50));
    }

    #[test]
    fn outline_textfield_light_uses_transparent_fill_and_border_token() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look =
            textfield_palette(&mode, ThemeMode::Light, ShadcnTextFieldStyle::Outline, TextFieldState::default(), true);

        assert_eq!(look.background, gpui::hsla(0.0, 0.0, 0.0, 0.0));
        assert_eq!(look.border, catalog.color("border").expect("border"));
        assert_eq!(look.foreground, catalog.color("foreground").expect("foreground"));
        assert_eq!(look.selection_foreground, catalog.color("primary-foreground").expect("primary-foreground"));
        assert!(look.shadow.is_none());
    }

    #[test]
    fn primary_textfield_light_uses_background_fill_and_shadow() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look =
            textfield_palette(&mode, ThemeMode::Light, ShadcnTextFieldStyle::Primary, TextFieldState::default(), true);

        assert_eq!(look.background, catalog.color("background").expect("background"));
        assert_eq!(look.border, catalog.color("border").expect("border"));
        assert!(look.shadow.as_ref().is_some_and(|shadows| !shadows.is_empty()));
    }

    #[test]
    fn primary_textfield_disabled_retains_elevation_shadow_for_layout() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look =
            textfield_palette(&mode, ThemeMode::Light, ShadcnTextFieldStyle::Primary, TextFieldState::default(), false);

        assert_eq!(look.background, catalog.color("muted").expect("muted"));
        assert!(look.shadow.as_ref().is_some_and(|shadows| !shadows.is_empty()));
    }

    #[test]
    fn outline_textfield_dark_uses_input_fill_and_border_token() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Dark).expect("catalog");
        let look =
            textfield_palette(&mode, ThemeMode::Dark, ShadcnTextFieldStyle::Outline, TextFieldState::default(), true);
        let input = catalog.color("input").expect("input");

        assert_eq!(look.background, with_alpha(input, 0.30));
        assert_eq!(look.border, catalog.color("border").expect("border"));
    }

    #[test]
    fn surface_textfield_uses_muted_fill_and_no_border() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look =
            textfield_palette(&mode, ThemeMode::Light, ShadcnTextFieldStyle::Surface, TextFieldState::default(), true);

        assert_eq!(look.background, catalog.color("muted").expect("muted"));
        assert_eq!(look.border, gpui::hsla(0.0, 0.0, 0.0, 0.0));
        assert!(look.shadow.is_none());
    }

    #[test]
    fn outline_textfield_hover_does_not_change_background() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let default =
            textfield_palette(&mode, ThemeMode::Light, ShadcnTextFieldStyle::Outline, TextFieldState::default(), true);
        let mut hovered = TextFieldState::default();
        hovered.hovered = true;
        let look = textfield_palette(&mode, ThemeMode::Light, ShadcnTextFieldStyle::Outline, hovered, true);

        assert_eq!(look.background, default.background);
    }

    #[test]
    fn palette_typography_uses_stylesheet_font_size() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog, ThemeMode::Light).expect("catalog");
        let sm = textfield_palette_for_size(
            &mode,
            ThemeMode::Light,
            ShadcnTextFieldStyle::Outline,
            TextFieldState::default(),
            true,
            ControlSize::Sm,
        );
        let md = textfield_palette_for_size(
            &mode,
            ThemeMode::Light,
            ShadcnTextFieldStyle::Outline,
            TextFieldState::default(),
            true,
            ControlSize::Md,
        );
        let lg = textfield_palette_for_size(
            &mode,
            ThemeMode::Light,
            ShadcnTextFieldStyle::Outline,
            TextFieldState::default(),
            true,
            ControlSize::Lg,
        );

        assert!((sm.typography.size - 12.0).abs() < f32::EPSILON);
        assert!((md.typography.size - 14.0).abs() < f32::EPSILON);
        assert!((lg.typography.size - 16.0).abs() < f32::EPSILON);
        assert!(lg.typography.line_height > md.typography.line_height);
        assert!(md.typography.line_height > sm.typography.line_height);
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct TextfieldStylesheet {
    #[serde(default)]
    pub elevation_rules: Vec<TextfieldElevationRule>,
    #[serde(default)]
    pub color_rules: Vec<TextfieldColorRule>,
}

impl TextfieldStylesheet {
    /// Native bundled field rules, preserving the surface-invalid exception.
    pub fn native_default() -> Self {
        Self {
            elevation_rules: [("primary", "shadow-xs"), ("outline", "none"), ("surface", "none"), ("input", "none")]
                .into_iter()
                .map(|(style, shadow)| TextfieldElevationRule { style: style.into(), shadow: shadow.into() })
                .collect(),
            color_rules: [
                ("input", false, None, None, "muted", "border"),
                ("input", true, Some(false), Some("light"), "transparent", "border"),
                ("input", true, Some(false), Some("dark"), "input/30", "border"),
                ("input", true, Some(true), Some("light"), "transparent", "destructive"),
                ("input", true, Some(true), Some("dark"), "input/30", "destructive"),
                ("primary", false, None, None, "muted", "border"),
                ("primary", true, Some(false), None, "background", "border"),
                ("primary", true, Some(true), None, "background", "destructive"),
                ("outline", false, None, None, "muted", "border"),
                ("surface", false, None, None, "muted", "transparent"),
                ("surface", true, Some(true), None, "muted", "destructive"),
                ("surface", true, None, None, "muted", "transparent"),
                ("outline", true, Some(false), Some("light"), "transparent", "border"),
                ("outline", true, Some(false), Some("dark"), "input/30", "border"),
                ("outline", true, Some(true), Some("light"), "transparent", "destructive"),
                ("outline", true, Some(true), Some("dark"), "input/30", "destructive"),
            ]
            .into_iter()
            .map(|(style, enabled, invalid, mode, background, border)| TextfieldColorRule {
                style: Some(style.into()),
                enabled: Some(enabled),
                invalid,
                mode: mode.map(Into::into),
                background: background.into(),
                foreground: if enabled { "foreground" } else { "muted-foreground" }.into(),
                border: border.into(),
                placeholder: "muted-foreground".into(),
                icon: "muted-foreground".into(),
                selection_background: if enabled { "primary" } else { "muted-foreground" }.into(),
                selection_foreground: if enabled {
                    "primary-foreground"
                } else {
                    "muted-foreground"
                }
                .into(),
                caret: if enabled { "foreground" } else { "muted-foreground" }.into(),
            })
            .collect(),
        }
    }

    pub fn elevation_rule_for_style(&self, style: &str) -> Option<&TextfieldElevationRule> {
        self.elevation_rules.iter().find(|rule| rule.style == style)
    }

    pub fn find_color_rule(
        &self,
        style: &str,
        enabled: bool,
        invalid: bool,
        mode: &str,
    ) -> Option<&TextfieldColorRule> {
        self.color_rules.iter().find(|rule| {
            rule.style.as_deref().is_none_or(|value| value == style)
                && rule.enabled.is_none_or(|value| value == enabled)
                && rule.invalid.is_none_or(|value| value == invalid)
                && rule.mode.as_deref().is_none_or(|value| value == mode)
        })
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct TextfieldElevationRule {
    pub style: String,
    pub shadow: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct TextfieldColorRule {
    pub style: Option<String>,
    pub enabled: Option<bool>,
    pub invalid: Option<bool>,
    pub mode: Option<String>,
    pub background: String,
    pub foreground: String,
    pub border: String,
    pub placeholder: String,
    pub icon: String,
    pub selection_background: String,
    pub selection_foreground: String,
    pub caret: String,
}

#[derive(Clone, Debug)]
pub struct TextFieldMetricTable {
    pub min_height: crate::ResolvedMetric,
    pub icon_size: crate::ResolvedMetric,
    pub padding_x: crate::ResolvedMetric,
    pub padding_y: crate::ResolvedMetric,
    pub radius: crate::ResolvedMetric,
    pub border_width: crate::ResolvedMetric,
    pub focus_ring_width: crate::ResolvedMetric,
    pub focus_ring_offset: crate::ResolvedMetric,
}

pub fn resolve_textfield_metrics(
    mode: &crate::ShadcnModeTokens,
    theme_mode: xgpui_luma::theme::ThemeMode,
    size: xgpui_luma::theme::ControlSize,
) -> TextFieldMetricTable {
    use xgpui_luma::theme::StandardBoxScale;

    use crate::tables::metrics::helpers::{
        border_width_metric, control_size_key, focus_ring_offset_metric, focus_ring_width_metric, radius_metric,
        scaffold_control_metric, spacing_control_metric,
    };
    use crate::catalog::SpacingField;

    let ctx = crate::LookContext::new(mode, theme_mode, xgpui_luma::theme::InteractionState::default());
    let catalog = ctx.catalog();
    let metrics = ctx.metrics();
    let scale = StandardBoxScale::compute(size, metrics, 1.0);

    let geometry = crate::controls::textfield::textfield_geometry(mode, size, &scale, &mode.typography.text.body);
    let mut table = TextFieldMetricTable {
        min_height: scaffold_control_metric(control_size_key(size), "control_height", scale.height),
        icon_size: scaffold_control_metric(control_size_key(size), "icon_size", scale.icon_size),
        padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, scale.padding_x),
        padding_y: spacing_control_metric(catalog, size, SpacingField::PaddingY, scale.padding_y),
        radius: radius_metric(catalog, size, scale.radius),
        border_width: border_width_metric(metrics),
        focus_ring_width: focus_ring_width_metric(metrics),
        focus_ring_offset: focus_ring_offset_metric(metrics),
    };
    table.min_height = crate::tables::metrics::helpers::prefer_shared_metric(geometry.min_height, table.min_height);
    table.icon_size = crate::tables::metrics::helpers::prefer_shared_metric(geometry.icon_size, table.icon_size);
    table.padding_x = crate::tables::metrics::helpers::prefer_shared_metric(geometry.padding_x, table.padding_x);
    table.padding_y = crate::tables::metrics::helpers::prefer_shared_metric(geometry.padding_y, table.padding_y);
    table
}
