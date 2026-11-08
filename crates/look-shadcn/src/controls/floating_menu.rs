//! Floating menu property mappings (shadcn / tweakcn):
//!
//! | Part          | Token                              |
//! |---------------|------------------------------------|
//! | Surface       | `popover` (fallback `card`)        |
//! | Foreground    | `popover-foreground`               |
//! | Border        | `border`                           |
//! | Item hover bg | `accent`                           |
//! | Item hover fg | `accent-foreground`                |
//! | Item disabled | `muted-foreground`               |

use crate::stylesheet::config::matches_optional_layer;
use serde::Deserialize;
use std::collections::HashMap;

use xgpui_luma::controls::floating_menu::FloatingMenuLook;
use xgpui_luma::theme::ControlSize;
use xgpui_luma::theme::InteractionLayer;
use xgpui_luma::theme::InteractionState;
use xgpui_luma::theme::ThemeMode;

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::LookResolver;
use crate::provenance::ResolvedColor;
use crate::shadow::parse_shadow_token;
use crate::stylesheet::StylesheetConfig;
use crate::stylesheet::find_floating_menu_surface_color_rule;
use crate::stylesheet::find_floating_menu_surface_elevation_rule;
use crate::stylesheet::find_floating_menu_trigger_color_rule;
use crate::stylesheet::resolve_floating_menu_surface_color_rule;
use crate::stylesheet::resolve_floating_menu_trigger_color_rule;
use crate::stylesheet::resolve_stylesheet_shadow_token;

#[derive(Clone, Debug)]
pub struct FloatingMenuColorTable {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
    pub item_hover_background: ResolvedColor,
    pub item_hover_foreground: ResolvedColor,
    pub item_disabled_foreground: ResolvedColor,
}

impl FloatingMenuColorTable {
    pub fn fallback() -> Self {
        Self {
            background: ResolvedColor::transparent(),
            foreground: ResolvedColor::fallback_foreground(),
            border: ResolvedColor::fallback_foreground(),
            item_hover_background: ResolvedColor::transparent(),
            item_hover_foreground: ResolvedColor::fallback_foreground(),
            item_disabled_foreground: ResolvedColor::fallback_foreground(),
        }
    }
}

pub fn resolve_floating_menu_colors(
    resolver: &LookResolver<'_>,
    present: bool,
) -> anyhow::Result<FloatingMenuColorTable> {
    resolve_floating_menu_colors_with_stylesheet(resolver, resolver.stylesheet(), present)
}

pub fn resolve_floating_menu_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    _present: bool,
) -> anyhow::Result<FloatingMenuColorTable> {
    let rule = find_floating_menu_surface_color_rule(stylesheet)
        .ok_or_else(|| anyhow::anyhow!("no matching floating menu surface color rule"))?;
    let colors = resolve_floating_menu_surface_color_rule(resolver, rule)?;
    Ok(FloatingMenuColorTable {
        background: colors.background,
        foreground: colors.foreground,
        border: colors.border,
        item_hover_background: colors.item_hover_background,
        item_hover_foreground: colors.item_hover_foreground,
        item_disabled_foreground: colors.item_disabled_foreground,
    })
}

#[derive(Clone, Debug)]
pub struct GhostTriggerColorTable {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
}

impl GhostTriggerColorTable {
    pub fn fallback() -> Self {
        Self { background: ResolvedColor::transparent(), foreground: ResolvedColor::fallback_foreground() }
    }
}

pub fn resolve_ghost_trigger_colors(
    resolver: &LookResolver<'_>,
    layer: InteractionLayer,
    disabled: bool,
) -> anyhow::Result<GhostTriggerColorTable> {
    resolve_ghost_trigger_colors_with_stylesheet(resolver, resolver.stylesheet(), layer, disabled)
}

pub fn resolve_ghost_trigger_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    layer: InteractionLayer,
    disabled: bool,
) -> anyhow::Result<GhostTriggerColorTable> {
    let rule = find_floating_menu_trigger_color_rule(stylesheet, disabled, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching ghost trigger color rule"))?;
    let colors = resolve_floating_menu_trigger_color_rule(resolver, rule, layer)?;
    Ok(GhostTriggerColorTable { background: colors.background, foreground: colors.foreground })
}

pub fn floating_menu_look(mode: &ShadcnModeTokens, theme_mode: ThemeMode, size: ControlSize) -> FloatingMenuLook {
    let ctx = LookContext::new(mode, theme_mode, InteractionState::default());
    let catalog = ctx.catalog();
    let metrics = ctx.metrics();
    let typography = ctx.typography();
    let shadow = floating_menu_elevation_shadow(catalog, mode.stylesheet());
    let resolver = LookResolver::new(catalog, ctx.theme_mode, "floating_menu").with_stylesheet(mode.stylesheet());
    let colors = resolve_floating_menu_colors(&resolver, true).unwrap_or_else(|_| FloatingMenuColorTable::fallback());
    let mut item_typography = typography.text.label;
    super::typography::apply_button_metrics_typography(&mut item_typography, mode, size);
    let item_icon_size = super::button::button_box_scale(&ctx, mode.stylesheet(), size, 1.0).icon_size;

    let base_recipe = xgpui_luma::theme::stylesheet::CommonStylesheet::sdk_default()
        .floating_menu
        .authored_geometry(crate::tables::metrics::helpers::control_size_key(size));
    let recipe = mode
        .stylesheet()
        .common
        .floating_menu
        .authored_geometry(crate::tables::metrics::helpers::control_size_key(size))
        .with_fallback(&base_recipe);
    let mut look = FloatingMenuLook {
        window_margin: mode
            .stylesheet()
            .common
            .with_sdk_defaults()
            .popover
            .authored_geometry("panel")
            .window_margin
            .unwrap_or_default(),
        positive: mode.data_positive,
        border_width: mode.metrics.border_width.default,
        background: colors.background.hsla(),
        foreground: colors.foreground.hsla(),
        border: colors.border.hsla(),
        shadow,
        radius: metrics.radius.lg,
        padding: metrics.padding_y(size) * recipe.padding_scale.unwrap_or_default(),
        min_width: recipe.min_width.unwrap_or_default(),
        item_disabled_foreground: colors.item_disabled_foreground.hsla(),
        item_hover_background: colors.item_hover_background.hsla(),
        item_hover_foreground: colors.item_hover_foreground.hsla(),
        item_typography,
        item_height: metrics.control_height(size) * recipe.item_height_scale.unwrap_or_default(),
        item_padding_x: metrics.padding_x(size) * recipe.item_padding_scale.unwrap_or_default(),
        item_gap: metrics.gap(size),
        item_icon_size: item_icon_size,
        item_radius: metrics.radius.sm,
        disabled_opacity: recipe.disabled_opacity.unwrap_or_default(),
        separator_color: colors.border.hsla(),
        separator_thickness: recipe.separator_thickness.unwrap_or_default(),
        separator_spacing: metrics.gap(size) * recipe.submenu_offset_scale.unwrap_or_default(),
        separator_inset: recipe.separator_inset.unwrap_or_default(),
        submenu_offset_x: metrics.gap(size) * recipe.submenu_offset_scale.unwrap_or_default(),
    };
    if let Some(rule) =
        mode.stylesheet().floating_menu.metrics.get(crate::tables::metrics::helpers::control_size_key(size))
    {
        look.min_width = rule.min_width;
        look.radius = crate::stylesheet::resolve_stylesheet_metric(&rule.radius, metrics, size).unwrap_or(look.radius);
        look.item_radius =
            crate::stylesheet::resolve_stylesheet_metric(&rule.item_radius, metrics, size).unwrap_or(look.item_radius);
        look.item_height = metrics.control_height(size) * rule.item_height_factor;
        look.item_padding_x = metrics.padding_x(size) * rule.item_padding_x_factor;
        look.submenu_offset_x = metrics.gap(size) * rule.submenu_offset_x_factor;
    }
    let geometry = mode.stylesheet().common.floating_menu.inspect_geometry(
        crate::tables::metrics::helpers::control_size_key(size),
        xgpui_luma::theme::stylesheet::FloatingMenuGeometry {
            border_width: look.border_width,
            padding: look.padding,
            min_width: look.min_width,
            item_height: look.item_height,
            item_padding_x: look.item_padding_x,
            item_gap: look.item_gap,
            item_icon_size: look.item_icon_size,
            submenu_offset_x: look.submenu_offset_x,
            font_size: look.item_typography.size,
            line_height: look.item_typography.line_height,
            separator_thickness: look.separator_thickness,
            separator_spacing: look.separator_spacing,
            separator_inset: look.separator_inset,
            disabled_opacity: look.disabled_opacity,
            ..Default::default()
        },
    );
    look.border_width = geometry.border_width.value_px;
    look.padding = geometry.padding.value_px;
    look.min_width = geometry.min_width.value_px;
    look.item_height = geometry.item_height.value_px;
    look.item_padding_x = geometry.item_padding_x.value_px;
    look.item_gap = geometry.item_gap.value_px;
    look.item_icon_size = geometry.item_icon_size.value_px;
    look.submenu_offset_x = geometry.submenu_offset_x.value_px;
    look.separator_thickness = geometry.separator_thickness.value_px;
    look.separator_spacing = geometry.separator_spacing.value_px;
    look.separator_inset = geometry.separator_inset.value_px;
    look.disabled_opacity = geometry.disabled_opacity.value_px;

    crate::tables::typography::apply_resolved_geometry_typography(
        &mut look.item_typography,
        &geometry.font_size,
        &geometry.line_height,
    );

    look
}

fn floating_menu_elevation_shadow(
    catalog: &crate::catalog::CssTokenMap,
    stylesheet: &StylesheetConfig,
) -> Vec<gpui::BoxShadow> {
    let Some(rule) = find_floating_menu_surface_elevation_rule(stylesheet) else {
        return Vec::new();
    };
    let Some(token) = resolve_stylesheet_shadow_token(&rule.shadow) else {
        return Vec::new();
    };
    parse_shadow_token(catalog, &token).unwrap_or_default()
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;
    use xgpui_luma::theme::ThemeMode;

    use xgpui_luma::theme::ControlSize;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::floating_menu_look;

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
            ("accent-foreground".into(), "oklch(1 0 0)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("popover".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("popover-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("shadow-md".into(), "0 1px 3px 0px hsl(0 0% 0% / 0.10), 0 2px 4px -1px hsl(0 0% 0% / 0.10)".into()),
        ]))
    }

    #[test]
    fn floating_menu_resolves_stylesheet_shadow() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let look = floating_menu_look(&mode, xgpui_luma::theme::ThemeMode::Light, ControlSize::Md);

        assert!(!look.shadow.is_empty());
    }

    #[test]
    fn floating_menu_uses_popover_surface_and_accent_item_hover() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = floating_menu_look(&mode, xgpui_luma::theme::ThemeMode::Light, ControlSize::Md);

        assert_eq!(look.background, catalog.color("popover").expect("popover"));
        assert_eq!(look.foreground, catalog.color("popover-foreground").expect("popover-foreground"));
        assert_eq!(look.item_hover_background, catalog.color("accent").expect("accent"));
        assert_eq!(look.item_hover_foreground, catalog.color("accent-foreground").expect("accent-foreground"));
    }

    #[test]
    fn floating_menu_item_typography_scales_with_size() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let sm = floating_menu_look(&mode, xgpui_luma::theme::ThemeMode::Light, ControlSize::Sm);
        let md = floating_menu_look(&mode, xgpui_luma::theme::ThemeMode::Light, ControlSize::Md);
        let lg = floating_menu_look(&mode, xgpui_luma::theme::ThemeMode::Light, ControlSize::Lg);

        assert!((sm.item_typography.size - 12.0).abs() < f32::EPSILON);
        assert!((md.item_typography.size - 14.0).abs() < f32::EPSILON);
        assert!((lg.item_typography.size - 16.0).abs() < f32::EPSILON);
        assert!(sm.item_typography.size < md.item_typography.size);
        assert!(md.item_typography.size < lg.item_typography.size);
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct FloatingMenuStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, FloatingMenuMetricsRule>,
    #[serde(default)]
    pub surface: FloatingMenuSurfaceStylesheet,
    #[serde(default)]
    pub trigger: FloatingMenuTriggerStylesheet,
}

impl FloatingMenuStylesheet {
    /// Native bundled menu surface and trigger rules.
    pub fn native_default() -> Self {
        Self {
            surface: FloatingMenuSurfaceStylesheet {
                elevation_rules: vec![FloatingMenuSurfaceElevationRule { shadow: "shadow-md".into() }],
                color_rules: vec![FloatingMenuSurfaceColorRule {
                    background: "first(popover,card)".into(),
                    foreground: "popover-foreground".into(),
                    border: "border".into(),
                    item_hover_background: "accent".into(),
                    item_hover_foreground: "accent-foreground".into(),
                    item_disabled_foreground: "muted-foreground".into(),
                }],
            },
            trigger: FloatingMenuTriggerStylesheet {
                color_rules: [
                    (true, None, "muted", "muted-foreground"),
                    (false, Some("hover"), "transparent", "accent-foreground"),
                    (false, Some("pressed"), "transparent", "accent-foreground"),
                    (false, Some("default"), "transparent", "foreground"),
                    (false, Some("disabled"), "transparent", "foreground"),
                ]
                .into_iter()
                .map(|(disabled, layer, background, foreground)| FloatingMenuTriggerColorRule {
                    disabled: Some(disabled),
                    layer: layer.map(Into::into),
                    background: background.into(),
                    foreground: foreground.into(),
                })
                .collect(),
            },
            ..Default::default()
        }
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct FloatingMenuMetricsRule {
    pub min_width: f32,
    pub radius: String,
    pub item_height_factor: f32,
    pub item_padding_x_factor: f32,
    pub item_radius: String,
    pub submenu_offset_x_factor: f32,
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct FloatingMenuSurfaceStylesheet {
    #[serde(default)]
    pub elevation_rules: Vec<FloatingMenuSurfaceElevationRule>,
    #[serde(default)]
    pub color_rules: Vec<FloatingMenuSurfaceColorRule>,
}

impl FloatingMenuSurfaceStylesheet {
    pub fn elevation_rule(&self) -> Option<&FloatingMenuSurfaceElevationRule> {
        self.elevation_rules.first()
    }

    pub fn color_rule(&self) -> Option<&FloatingMenuSurfaceColorRule> {
        self.color_rules.first()
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct FloatingMenuSurfaceElevationRule {
    pub shadow: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct FloatingMenuSurfaceColorRule {
    pub background: String,
    pub foreground: String,
    pub border: String,
    pub item_hover_background: String,
    pub item_hover_foreground: String,
    pub item_disabled_foreground: String,
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct FloatingMenuTriggerStylesheet {
    #[serde(default)]
    pub color_rules: Vec<FloatingMenuTriggerColorRule>,
}

impl FloatingMenuTriggerStylesheet {
    pub fn find_color_rule(&self, disabled: bool, layer: InteractionLayer) -> Option<&FloatingMenuTriggerColorRule> {
        self.color_rules.iter().find(|rule| {
            if rule.disabled == Some(true) {
                return disabled;
            }
            if disabled {
                return false;
            }
            matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct FloatingMenuTriggerColorRule {
    pub disabled: Option<bool>,
    pub layer: Option<String>,
    pub background: String,
    pub foreground: String,
}

#[derive(Clone, Debug)]
pub struct FloatingMenuMetricTable {
    pub radius: crate::ResolvedMetric,
    pub padding: crate::ResolvedMetric,
    pub min_width: crate::ResolvedMetric,
    pub item_height: crate::ResolvedMetric,
    pub item_padding_x: crate::ResolvedMetric,
    pub item_gap: crate::ResolvedMetric,
    pub item_icon_size: crate::ResolvedMetric,
    pub item_radius: crate::ResolvedMetric,
    pub submenu_offset_x: crate::ResolvedMetric,
}

pub fn resolve_floating_menu_metrics(
    mode: &crate::ShadcnModeTokens,
    theme_mode: xgpui_luma::theme::ThemeMode,
    size: xgpui_luma::theme::ControlSize,
) -> FloatingMenuMetricTable {
    use crate::tables::metrics::helpers::{derived_metric, scaffold_control_metric, spacing_control_metric};
    use crate::catalog::SpacingField;

    let ctx = crate::LookContext::new(mode, theme_mode, xgpui_luma::theme::InteractionState::default());
    let look = crate::paint::floating_menu_look(mode, theme_mode, size);
    let catalog = ctx.catalog();

    let mut table = FloatingMenuMetricTable {
        radius: derived_metric("lg = --radius", look.radius),
        padding: derived_metric("padding_y × 0.5", look.padding),
        min_width: derived_metric("floating menu min width", look.min_width),
        item_height: derived_metric("control_height × 0.9", look.item_height),
        item_padding_x: derived_metric("padding_x × 0.75", look.item_padding_x),
        item_gap: spacing_control_metric(catalog, size, SpacingField::Gap, look.item_gap),
        item_icon_size: crate::tables::metrics::helpers::inspect_shared_metric(
            crate::controls::button::resolve_family_geometry(&ctx, mode.stylesheet(), size, false, 1.0)
                .0
                .icon_size,
        ),
        item_radius: scaffold_control_metric("sm", "radius", look.item_radius),
        submenu_offset_x: derived_metric("gap × 0.5", look.submenu_offset_x),
    };
    let geometry = mode.stylesheet().common.floating_menu.inspect_geometry(
        crate::tables::metrics::helpers::control_size_key(size),
        xgpui_luma::theme::stylesheet::FloatingMenuGeometry {
            padding: table.padding.value_px,
            min_width: table.min_width.value_px,
            item_height: table.item_height.value_px,
            item_padding_x: table.item_padding_x.value_px,
            item_gap: table.item_gap.value_px,
            item_icon_size: table.item_icon_size.value_px,
            submenu_offset_x: table.submenu_offset_x.value_px,
            ..Default::default()
        },
    );
    table.padding = crate::tables::metrics::helpers::prefer_shared_metric(geometry.padding, table.padding);
    table.min_width = crate::tables::metrics::helpers::prefer_shared_metric(geometry.min_width, table.min_width);
    table.item_height = crate::tables::metrics::helpers::prefer_shared_metric(geometry.item_height, table.item_height);
    table.item_padding_x =
        crate::tables::metrics::helpers::prefer_shared_metric(geometry.item_padding_x, table.item_padding_x);
    table.item_gap = crate::tables::metrics::helpers::prefer_shared_metric(geometry.item_gap, table.item_gap);
    table.item_icon_size =
        crate::tables::metrics::helpers::prefer_shared_metric(geometry.item_icon_size, table.item_icon_size);
    table.submenu_offset_x =
        crate::tables::metrics::helpers::prefer_shared_metric(geometry.submenu_offset_x, table.submenu_offset_x);

    table
}
