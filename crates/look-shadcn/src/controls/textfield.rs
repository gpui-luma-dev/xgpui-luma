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

use xgpui_luma::controls::textfield::TextFieldState;
use crate::size::ShadcnSize;
use xgpui_luma::theme::InteractionState;
use xgpui_luma::theme::LumaTextStyle;
use xgpui_luma::theme::StandardBoxScale;
use xgpui_luma::theme::ThemeMode;

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::LookResolver;
use crate::provenance::ResolvedColor;
use crate::stylesheet::StylesheetConfig;
use crate::stylesheet::find_textfield_color_rule;
use crate::stylesheet::resolve_textfield_color_rule;

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

pub(crate) fn textfield_geometry(
    mode: &ShadcnModeTokens,
    size: ShadcnSize,
    scale: &StandardBoxScale,
    typography: &LumaTextStyle,
) -> xgpui_luma::theme::stylesheet::ResolvedTextFieldGeometry {
    mode.stylesheet().common.textfield.inspect_geometry_for(
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
    size: crate::ShadcnSize,
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
    let scale = StandardBoxScale::compute(&mode.metrics_for(size), 1.0);

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
