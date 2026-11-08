//! Table — input surface + accent whisper hover, muted selected rows.
//!
//! | Row state   | Token / effect        |
//! |-------------|-----------------------|
//! | Default     | transparent           |
//! | Hover       | `accent` at 40% alpha |
//! | Selected    | `muted`               |
//! | Keyboard active | `muted`           |

use crate::stylesheet::config::EnabledColorRule;
use crate::stylesheet::config::find_enabled_color_rule;
use crate::stylesheet::config::matches_optional_layer;
use serde::Deserialize;
use std::collections::HashMap;

use xgpui_luma::controls::table::TableLook;
use xgpui_luma::controls::table::TableRowPalette;
use xgpui_luma::theme::ControlSize;
use xgpui_luma::theme::InteractionLayer;
use xgpui_luma::theme::InteractionState;

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::LookResolver;
use crate::provenance::ResolvedColor;
use crate::stylesheet::StylesheetConfig;
use crate::stylesheet::find_table_row_color_rule;
use crate::stylesheet::find_table_surface_color_rule;
use crate::stylesheet::resolve_table_row_color_rule;
use crate::stylesheet::resolve_table_surface_color_rule;
use crate::stylesheet::resolve_stylesheet_metric;

#[derive(Clone, Debug)]
pub struct TableSurfaceColorTable {
    pub background: ResolvedColor,
    pub border: ResolvedColor,
    pub header_background: ResolvedColor,
    pub header_label_color: ResolvedColor,
}

impl TableSurfaceColorTable {
    pub fn fallback() -> Self {
        Self {
            background: ResolvedColor::transparent(),
            border: ResolvedColor::fallback_foreground(),
            header_background: ResolvedColor::transparent(),
            header_label_color: ResolvedColor::fallback_foreground(),
        }
    }
}

pub fn resolve_table_surface_colors(
    resolver: &LookResolver<'_>,
    enabled: bool,
) -> anyhow::Result<TableSurfaceColorTable> {
    resolve_table_surface_colors_with_stylesheet(resolver, resolver.stylesheet(), enabled)
}

pub fn resolve_table_surface_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    enabled: bool,
) -> anyhow::Result<TableSurfaceColorTable> {
    let rule = find_table_surface_color_rule(stylesheet, enabled)
        .ok_or_else(|| anyhow::anyhow!("no matching table surface color rule"))?;
    let colors = resolve_table_surface_color_rule(resolver, rule)?;
    Ok(TableSurfaceColorTable {
        background: colors.background,
        border: colors.border,
        header_background: colors.header_background,
        header_label_color: colors.header_label_color,
    })
}

#[derive(Clone, Debug)]
pub struct TableRowColorTable {
    pub background: ResolvedColor,
    pub label_color: ResolvedColor,
    pub divider: ResolvedColor,
}

impl TableRowColorTable {
    pub fn fallback() -> Self {
        Self {
            background: ResolvedColor::transparent(),
            label_color: ResolvedColor::fallback_foreground(),
            divider: ResolvedColor::fallback_foreground(),
        }
    }
}

pub fn resolve_table_row_colors(
    resolver: &LookResolver<'_>,
    selected: bool,
    focused: bool,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<TableRowColorTable> {
    resolve_table_row_colors_with_stylesheet(resolver, resolver.stylesheet(), selected, focused, disabled, layer)
}

pub fn resolve_table_row_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    selected: bool,
    focused: bool,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<TableRowColorTable> {
    let rule = find_table_row_color_rule(stylesheet, selected, focused, disabled, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching table row color rule"))?;
    let colors = resolve_table_row_color_rule(resolver, rule, layer)?;
    Ok(TableRowColorTable { background: colors.background, label_color: colors.label_color, divider: colors.divider })
}

pub fn table_look(mode: &ShadcnModeTokens, enabled: bool, _focused: bool, size: ControlSize) -> TableLook {
    let ctx = LookContext::new(mode, mode.theme_mode, InteractionState::default());
    let metrics = ctx.metrics();
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "table").with_stylesheet(mode.stylesheet());
    let colors =
        resolve_table_surface_colors(&resolver, enabled).unwrap_or_else(|_| TableSurfaceColorTable::fallback());
    let surface_metrics = mode.stylesheet().table.surface.metrics_for_size(size);
    let radius = surface_metrics
        .and_then(|rule| resolve_stylesheet_metric(&rule.radius, metrics, size))
        .unwrap_or(metrics.radius(size));
    let radius = surface_metrics
        .and_then(|rule| rule.max_radius.as_deref())
        .and_then(|max| resolve_stylesheet_metric(max, metrics, size))
        .map_or(radius, |max| radius.min(max));

    let baseline = xgpui_luma::theme::stylesheet::CommonStylesheet::sdk_default()
        .table
        .authored_geometry(size.stylesheet_key());
    let recipe = mode.stylesheet().common.table.authored_geometry(size.stylesheet_key()).with_fallback(&baseline);
    let mut look = TableLook {
        row_divider_width: recipe.row_divider_width.unwrap_or_default(),
        column_padding_x: recipe.column_padding_x.unwrap_or_default(),
        column_resize_hit_width: recipe.column_resize_hit_width.unwrap_or_default(),
        column_resize_line_width: recipe.column_resize_line_width.unwrap_or_default(),
        column_resize_vertical_extension: recipe.column_resize_vertical_extension.unwrap_or_default(),

        background: colors.background.hsla(),
        border: colors.border.hsla(),
        border_width: recipe.border_width.unwrap_or_default(),
        header_divider_width: recipe.header_divider_width.unwrap_or_default(),
        header_padding_bottom_scale: recipe.header_padding_bottom_scale.unwrap_or_default(),
        header_background: colors.header_background.hsla(),
        header_label_color: colors.header_label_color.hsla(),
        header_typography: ctx.typography().text.caption,
        radius,
        padding_x: recipe.padding_x.unwrap_or_default(),
        padding_y: metrics.padding_y(size) * recipe.padding_y_scale.unwrap_or_default(),
    };
    let geometry = mode.stylesheet().common.table.resolve_geometry(
        crate::tables::metrics::helpers::control_size_key(size),
        xgpui_luma::theme::stylesheet::TableGeometry {
            padding_x: look.padding_x,
            padding_y: look.padding_y,
            ..Default::default()
        },
    );
    look.padding_x = geometry.padding_x;
    look.padding_y = geometry.padding_y;

    look
}

pub fn table_row_palette(
    mode: &ShadcnModeTokens,
    selected: bool,
    state: InteractionState,
    size: ControlSize,
) -> TableRowPalette {
    let ctx = LookContext::new(mode, mode.theme_mode, state);
    let typography = ctx.typography();
    let resolver = LookResolver::new(ctx.catalog(), ctx.theme_mode, "table_row").with_stylesheet(mode.stylesheet());
    let colors = resolve_table_row_colors(&resolver, selected, state.focused, state.disabled, state.layer())
        .unwrap_or_else(|_| TableRowColorTable::fallback());

    let mut label_typography = typography.text.label;
    super::apply_button_metrics_typography(&mut label_typography, mode, size);

    TableRowPalette {
        background: colors.background.hsla(),
        label_color: colors.label_color.hsla(),
        divider: colors.divider.hsla(),
        label_typography,
    }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;
    use xgpui_luma::theme::ThemeMode;

    use xgpui_luma::theme::{ControlSize, InteractionState};

    use crate::catalog::CssTokenMap;
    use crate::color::with_alpha;
    use crate::mode::ShadcnModeTokens;
    use crate::resolve::resolve_color;
    use super::table_row_palette;

    const ROW_HOVER_ACCENT_ALPHA: f32 = 0.4;

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
        ]))
    }

    #[test]
    fn table_row_hover_uses_accent_whisper_and_selected_uses_muted() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let hover =
            table_row_palette(&mode, false, InteractionState { hovered: true, ..Default::default() }, ControlSize::Md);
        let selected =
            table_row_palette(&mode, true, InteractionState { hovered: true, ..Default::default() }, ControlSize::Md);

        let accent = catalog.color("accent").expect("accent");
        assert_eq!(hover.background, with_alpha(accent, ROW_HOVER_ACCENT_ALPHA));
        assert_eq!(selected.background, resolve_color(&catalog, "muted").expect("muted"));
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct TableStylesheet {
    #[serde(default)]
    pub surface: TableSurfaceStylesheet,
    #[serde(default)]
    pub row: TableRowStylesheet,
}

impl TableStylesheet {
    /// Native bundled surface and row rules in priority order.
    pub fn native_default() -> Self {
        Self {
            surface: TableSurfaceStylesheet {
                metrics: HashMap::from([(
                    "md".into(),
                    TableSurfaceMetricsRule {
                        padding_y_factor: 0.5,
                        radius: "radius.md".into(),
                        max_radius: Some("1rem".into()),
                    },
                )]),
                color_rules: [(true, "background"), (false, "muted")]
                    .into_iter()
                    .map(|(enabled, background)| TableSurfaceColorRule {
                        enabled: Some(enabled),
                        background: background.into(),
                        border: "input".into(),
                        header_background: "muted".into(),
                        header_label_color: "muted-foreground".into(),
                    })
                    .collect(),
            },
            row: TableRowStylesheet {
                color_rules: [
                    (Some(true), None, None, None, "transparent", "muted-foreground"),
                    (None, Some(true), None, None, "muted", "foreground"),
                    (None, None, Some(true), None, "muted", "foreground"),
                    (None, None, None, Some("hover"), "@accent_whisper_40", "foreground"),
                    (None, None, None, Some("pressed"), "@accent_whisper_pressed_40", "foreground"),
                    (None, None, None, Some("default"), "transparent", "foreground"),
                ]
                .into_iter()
                .map(|(disabled, selected, focused, layer, background, label)| TableRowColorRule {
                    disabled,
                    selected,
                    focused,
                    layer: layer.map(Into::into),
                    background: background.into(),
                    label_color: label.into(),
                    divider: "border".into(),
                })
                .collect(),
            },
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct TableSurfaceStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, TableSurfaceMetricsRule>,
    #[serde(default)]
    pub color_rules: Vec<TableSurfaceColorRule>,
}

impl TableSurfaceStylesheet {
    pub fn find_color_rule(&self, enabled: bool) -> Option<&TableSurfaceColorRule> {
        find_enabled_color_rule(&self.color_rules, enabled)
    }

    pub fn metrics_for_size(&self, size: ControlSize) -> Option<&TableSurfaceMetricsRule> {
        let key = match size {
            ControlSize::Sm => "sm",
            ControlSize::Md => "md",
            ControlSize::Lg => "lg",
        };
        self.metrics.get(key)
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct TableSurfaceMetricsRule {
    pub padding_y_factor: f32,
    pub radius: String,
    #[serde(default)]
    pub max_radius: Option<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct TableSurfaceColorRule {
    pub enabled: Option<bool>,
    pub background: String,
    pub border: String,
    pub header_background: String,
    pub header_label_color: String,
}

impl EnabledColorRule for TableSurfaceColorRule {
    fn enabled(&self) -> Option<bool> {
        self.enabled
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct TableRowStylesheet {
    #[serde(default)]
    pub color_rules: Vec<TableRowColorRule>,
}

impl TableRowStylesheet {
    pub fn find_color_rule(
        &self,
        selected: bool,
        focused: bool,
        disabled: bool,
        layer: InteractionLayer,
    ) -> Option<&TableRowColorRule> {
        self.color_rules.iter().find(|rule| {
            if rule.disabled == Some(true) {
                return disabled;
            }
            if disabled {
                return false;
            }
            if rule.selected == Some(true) {
                return selected;
            }
            if rule.focused == Some(true) {
                return focused;
            }
            if selected || focused {
                return false;
            }
            matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct TableRowColorRule {
    pub disabled: Option<bool>,
    pub selected: Option<bool>,
    pub focused: Option<bool>,
    pub layer: Option<String>,
    pub background: String,
    pub label_color: String,
    pub divider: String,
}

#[derive(Clone, Debug)]
pub struct TableMetricTable {
    pub radius: crate::ResolvedMetric,
    pub padding_x: crate::ResolvedMetric,
    pub padding_y: crate::ResolvedMetric,
    pub row_min_height: crate::ResolvedMetric,
    pub row_padding_x: crate::ResolvedMetric,
    pub row_padding_y: crate::ResolvedMetric,
}

pub fn resolve_table_metrics(
    mode: &crate::ShadcnModeTokens,
    theme_mode: xgpui_luma::theme::ThemeMode,
    size: xgpui_luma::theme::ControlSize,
) -> TableMetricTable {
    use xgpui_luma::theme::ListRowScale;

    use crate::catalog::SpacingField;
    use crate::tables::metrics::helpers::{
        control_size_key, derived_metric, radius_metric, scaffold_control_metric, spacing_control_metric,
    };

    let ctx = crate::LookContext::new(mode, theme_mode, xgpui_luma::theme::InteractionState::default());
    let metrics = ctx.metrics();
    let catalog = ctx.catalog();
    let look = crate::paint::table_look(mode, true, false, size);
    let row_scale = ListRowScale::compute(size, metrics, 1.0);
    let size_key = control_size_key(size);

    let mut table = TableMetricTable {
        radius: radius_metric(catalog, size, look.radius),
        padding_x: derived_metric("list padding x", look.padding_x),
        padding_y: spacing_control_metric(catalog, size, SpacingField::PaddingY, look.padding_y),
        row_min_height: scaffold_control_metric(size_key, "control_height", row_scale.min_height),
        row_padding_x: spacing_control_metric(catalog, size, SpacingField::PaddingX, row_scale.padding_x),
        row_padding_y: spacing_control_metric(catalog, size, SpacingField::PaddingY, row_scale.padding_y),
    };
    let geometry = mode.stylesheet().common.table.inspect_geometry(
        size_key,
        xgpui_luma::theme::stylesheet::TableGeometry {
            padding_x: table.padding_x.value_px,
            padding_y: table.padding_y.value_px,
            ..Default::default()
        },
    );
    table.padding_x = crate::tables::metrics::helpers::prefer_shared_metric(geometry.padding_x, table.padding_x);
    table.padding_y = crate::tables::metrics::helpers::prefer_shared_metric(geometry.padding_y, table.padding_y);

    table
}
