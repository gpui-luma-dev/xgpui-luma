//! Scrollbar property mappings for shadcn-inspired controls:
//!
//! | Part  | Token              |
//! |-------|--------------------|
//! | Track | transparent (`Ghost`) / muted (`Soft`) |
//! | Thumb | `border`           |
//! | Hover | `border` (no change) |
//! | Press | `border` (darkened)  |

use crate::stylesheet::config::control_size_key;
use crate::stylesheet::config::matches_optional_bool;
use crate::stylesheet::config::matches_optional_layer;
use crate::stylesheet::config::matches_optional_str;
use serde::Deserialize;
use std::collections::HashMap;

use xgpui_luma::controls::scrollbar::ScrollbarLook;
use xgpui_luma::controls::scrollbar::ScrollbarOrientation;
use xgpui_luma::controls::scrollbar::ScrollbarStyle;
use xgpui_luma::theme::ControlSize;
use xgpui_luma::theme::InteractionLayer;
use xgpui_luma::theme::InteractionState;

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;
use crate::provenance::LookResolver;
use crate::provenance::ResolvedColor;
use crate::stylesheet::StylesheetConfig;
use crate::stylesheet::find_scrollbar_color_rule;
use crate::stylesheet::resolve_scrollbar_color_rule;

#[derive(Clone, Debug)]
pub struct ScrollbarColorTable {
    pub track_background: ResolvedColor,
    pub thumb_background: ResolvedColor,
}

impl ScrollbarColorTable {
    pub fn fallback() -> Self {
        Self {
            track_background: ResolvedColor::transparent(),
            thumb_background: ResolvedColor::fallback_foreground(),
        }
    }
}

pub fn resolve_scrollbar_colors(
    resolver: &LookResolver<'_>,
    style: ScrollbarStyle,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<ScrollbarColorTable> {
    resolve_scrollbar_colors_with_stylesheet(resolver, resolver.stylesheet(), style, disabled, layer)
}

pub fn resolve_scrollbar_colors_with_stylesheet(
    resolver: &LookResolver<'_>,
    stylesheet: &StylesheetConfig,
    style: ScrollbarStyle,
    disabled: bool,
    layer: InteractionLayer,
) -> anyhow::Result<ScrollbarColorTable> {
    let rule = find_scrollbar_color_rule(stylesheet, scrollbar_style_key(style), disabled, layer)
        .ok_or_else(|| anyhow::anyhow!("no matching scrollbar color rule"))?;
    let colors = resolve_scrollbar_color_rule(resolver, rule, layer)?;
    Ok(ScrollbarColorTable { track_background: colors.track_background, thumb_background: colors.thumb_background })
}

pub fn scrollbar_look(
    mode: &ShadcnModeTokens,
    state: InteractionState,
    orientation: ScrollbarOrientation,
    size: ControlSize,
    style: ScrollbarStyle,
) -> ScrollbarLook {
    let ctx = LookContext::new(mode, mode.theme_mode, state);
    let catalog = ctx.catalog();
    let metrics = ctx.metrics();
    let layer = state.layer();
    let resolver = LookResolver::new(catalog, ctx.theme_mode, "scrollbar").with_stylesheet(mode.stylesheet());
    let colors = resolve_scrollbar_colors(&resolver, style, state.disabled, layer)
        .unwrap_or_else(|_| ScrollbarColorTable::fallback());

    let baseline = xgpui_luma::theme::stylesheet::CommonStylesheet::sdk_default()
        .scrollbar
        .resolve_geometry(size.stylesheet_key(), Default::default());
    let length = match orientation {
        ScrollbarOrientation::Horizontal => baseline.length_h,
        ScrollbarOrientation::Vertical => baseline.length_v,
    };
    let stylesheet = mode.stylesheet();
    let (thickness, track_thickness, thumb_thickness, min_thumb_length) = stylesheet
        .scrollbar
        .metrics_for_size(size)
        .map(crate::stylesheet::resolve_scrollbar_metrics)
        .map(|metrics| (metrics.thickness, metrics.track_thickness, metrics.thumb_thickness, metrics.min_thumb_length))
        .unwrap_or((baseline.thickness, baseline.track_thickness, baseline.thumb_thickness, baseline.min_thumb_length));
    let mut look = ScrollbarLook {
        disabled_opacity: mode
            .stylesheet()
            .common
            .scrollbar
            .resolve_geometry(size.stylesheet_key(), Default::default())
            .disabled_opacity,
        track_background: colors.track_background.hsla(),
        thumb_background: colors.thumb_background.hsla(),
        length,
        thickness,
        track_thickness,
        thumb_thickness,
        min_thumb_length,
        radius: metrics.radius.pill,
    };
    let geometry = mode.stylesheet().common.scrollbar.resolve_geometry(
        crate::tables::metrics::helpers::control_size_key(size),
        xgpui_luma::theme::stylesheet::ScrollbarGeometry {
            thickness: look.thickness,
            track_thickness: look.track_thickness,
            thumb_thickness: look.thumb_thickness,
            min_thumb_length: look.min_thumb_length,
            length_h: baseline.length_h,
            length_v: baseline.length_v,
            ..Default::default()
        },
    );
    look.thickness = geometry.thickness;
    look.track_thickness = geometry.track_thickness;
    look.thumb_thickness = geometry.thumb_thickness;
    look.min_thumb_length = geometry.min_thumb_length;
    look.length = match orientation {
        ScrollbarOrientation::Horizontal => geometry.length_h,
        ScrollbarOrientation::Vertical => geometry.length_v,
    };
    look
}

fn scrollbar_style_key(style: ScrollbarStyle) -> &'static str {
    match style {
        ScrollbarStyle::Ghost => "ghost",
        ScrollbarStyle::Soft => "soft",
    }
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;
    use xgpui_luma::theme::ThemeMode;

    use xgpui_luma::controls::scrollbar::ScrollbarOrientation;
    use xgpui_luma::controls::scrollbar::ScrollbarStyle;
    use xgpui_luma::theme::InteractionState;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use super::scrollbar_look;

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
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.8 0.02 200)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
        ]))
    }

    #[test]
    fn default_scrollbar_uses_border_thumb() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = scrollbar_look(
            &mode,
            InteractionState::default(),
            ScrollbarOrientation::Vertical,
            xgpui_luma::theme::ControlSize::Md,
            ScrollbarStyle::Ghost,
        );

        assert_eq!(look.thumb_background, catalog.color("border").expect("border"));
        assert_eq!(look.track_background.a, 0.0);
    }

    #[test]
    fn scrollbar_size_changes_metrics() {
        let mode = ShadcnModeTokens::from_catalog(sample_catalog(), ThemeMode::Light).expect("catalog");
        let small = scrollbar_look(
            &mode,
            InteractionState::default(),
            ScrollbarOrientation::Vertical,
            xgpui_luma::theme::ControlSize::Sm,
            ScrollbarStyle::Ghost,
        );
        let medium = scrollbar_look(
            &mode,
            InteractionState::default(),
            ScrollbarOrientation::Vertical,
            xgpui_luma::theme::ControlSize::Md,
            ScrollbarStyle::Ghost,
        );
        let large = scrollbar_look(
            &mode,
            InteractionState::default(),
            ScrollbarOrientation::Vertical,
            xgpui_luma::theme::ControlSize::Lg,
            ScrollbarStyle::Ghost,
        );

        assert!(small.thickness < medium.thickness);
        assert!(medium.thickness < large.thickness);
        assert!(small.thumb_thickness < medium.thumb_thickness);
        assert!(medium.thumb_thickness < large.thumb_thickness);
    }

    #[test]
    fn hovered_and_focused_scrollbar_thumb_stay_border() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let border = catalog.color("border").expect("border");

        let hovered = scrollbar_look(
            &mode,
            InteractionState { hovered: true, ..InteractionState::default() },
            ScrollbarOrientation::Vertical,
            xgpui_luma::theme::ControlSize::Md,
            ScrollbarStyle::Ghost,
        );
        let focused = scrollbar_look(
            &mode,
            InteractionState { focused: true, ..InteractionState::default() },
            ScrollbarOrientation::Vertical,
            xgpui_luma::theme::ControlSize::Md,
            ScrollbarStyle::Ghost,
        );

        assert_eq!(hovered.thumb_background, border);
        assert_eq!(focused.thumb_background, border);
    }

    #[test]
    fn soft_scrollbar_uses_visible_track() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let look = scrollbar_look(
            &mode,
            InteractionState::default(),
            ScrollbarOrientation::Vertical,
            xgpui_luma::theme::ControlSize::Md,
            ScrollbarStyle::Soft,
        );

        assert_eq!(look.track_background, catalog.color("muted").expect("muted"));
        assert_eq!(look.thumb_background, catalog.color("border").expect("border"));
    }
}

#[cfg(all(test, feature = "test-support"))]
#[path = "scroll_container_tests.rs"]
mod scroll_container_tests;

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct ScrollbarStylesheet {
    #[serde(default)]
    pub metrics: HashMap<String, ScrollbarMetricsRule>,
    #[serde(default)]
    pub color_rules: Vec<ScrollbarColorRule>,
}

impl ScrollbarStylesheet {
    /// Native bundled rules, retaining disabled-first matching.
    pub fn native_default() -> Self {
        Self {
            color_rules: [
                (None, true, None, "muted", "muted-foreground"),
                (None, false, Some("disabled"), "transparent", "muted-foreground"),
                (Some("ghost"), false, Some("default"), "transparent", "border"),
                (Some("ghost"), false, Some("hover"), "transparent", "border"),
                (Some("ghost"), false, Some("pressed"), "transparent", "@darken_border"),
                (Some("soft"), false, Some("default"), "muted", "border"),
                (Some("soft"), false, Some("hover"), "muted", "border"),
                (Some("soft"), false, Some("pressed"), "muted", "@darken_border"),
            ]
            .into_iter()
            .map(|(style, disabled, layer, track, thumb)| ScrollbarColorRule {
                style: style.map(Into::into),
                disabled: Some(disabled),
                layer: layer.map(Into::into),
                track_background: track.into(),
                thumb_background: thumb.into(),
            })
            .collect(),
            ..Default::default()
        }
    }

    pub fn metrics_for_size(&self, size: ControlSize) -> Option<&ScrollbarMetricsRule> {
        self.metrics.get(control_size_key(size))
    }

    pub fn find_color_rule(&self, style: &str, disabled: bool, layer: InteractionLayer) -> Option<&ScrollbarColorRule> {
        self.color_rules.iter().find(|rule| {
            matches_optional_str(rule.style.as_deref(), style)
                && matches_optional_bool(rule.disabled, disabled)
                && matches_optional_layer(rule.layer.as_deref(), layer)
        })
    }
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
pub struct ScrollbarMetricsRule {
    pub thickness: f32,
    pub track_thickness: f32,
    pub thumb_thickness: f32,
    pub min_thumb_length: f32,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ScrollbarColorRule {
    pub style: Option<String>,
    pub disabled: Option<bool>,
    pub layer: Option<String>,
    pub track_background: String,
    pub thumb_background: String,
}

/// Native common geometry owned by this control.
pub(crate) fn native_scrollbar_stylesheet() -> xgpui_luma::theme::stylesheet::ScrollbarStylesheet {
    use xgpui_luma::theme::stylesheet::*;
    ScrollbarStylesheet {
        geometry: ScrollbarGeometryStylesheet { length_h: Some(260.0), length_v: Some(180.0), ..Default::default() },
        sizes: ["sm", "md", "lg"]
            .into_iter()
            .map(|key| (key.into(), ScrollbarGeometryStylesheet::default()))
            .collect(),
    }
}

#[derive(Clone, Debug)]
pub struct ScrollbarMetricTable {
    pub length: crate::ResolvedMetric,
    pub thickness: crate::ResolvedMetric,
    pub track_thickness: crate::ResolvedMetric,
    pub thumb_thickness: crate::ResolvedMetric,
    pub min_thumb_length: crate::ResolvedMetric,
    pub radius: crate::ResolvedMetric,
}

pub fn resolve_scrollbar_metrics(
    mode: &crate::ShadcnModeTokens,
    theme_mode: xgpui_luma::theme::ThemeMode,
    orientation: xgpui_luma::controls::scrollbar::ScrollbarOrientation,
    style: xgpui_luma::controls::scrollbar::ScrollbarStyle,
) -> ScrollbarMetricTable {
    resolve_scrollbar_metrics_for_size(mode, theme_mode, orientation, style, xgpui_luma::theme::ControlSize::Md)
}

/// Resolve the selected look's Scrollbar dimensions and their sources for a size.
pub fn resolve_scrollbar_metrics_for_size(
    mode: &crate::ShadcnModeTokens,
    theme_mode: xgpui_luma::theme::ThemeMode,
    orientation: xgpui_luma::controls::scrollbar::ScrollbarOrientation,
    style: xgpui_luma::controls::scrollbar::ScrollbarStyle,
    size: xgpui_luma::theme::ControlSize,
) -> ScrollbarMetricTable {
    use crate::tables::metrics::helpers::{derived_metric, pill_radius_metric};

    let ctx = crate::LookContext::new(mode, theme_mode, xgpui_luma::theme::InteractionState::default());
    let look =
        crate::paint::scrollbar_look(mode, xgpui_luma::theme::InteractionState::default(), orientation, size, style);
    let catalog = ctx.catalog();

    let mut table = ScrollbarMetricTable {
        length: derived_metric("gallery scrollbar demo length", look.length),
        thickness: derived_metric("scrollbar chrome thickness", look.thickness),
        track_thickness: derived_metric("track hit target", look.track_thickness),
        thumb_thickness: derived_metric("thumb visual size", look.thumb_thickness),
        min_thumb_length: derived_metric("minimum draggable thumb", look.min_thumb_length),
        radius: pill_radius_metric(catalog, look.radius),
    };
    let geometry = mode.stylesheet().common.scrollbar.inspect_geometry(
        crate::tables::metrics::helpers::control_size_key(size),
        xgpui_luma::theme::stylesheet::ScrollbarGeometry {
            thickness: table.thickness.value_px,
            track_thickness: table.track_thickness.value_px,
            thumb_thickness: table.thumb_thickness.value_px,
            min_thumb_length: table.min_thumb_length.value_px,
            length_h: table.length.value_px,
            length_v: table.length.value_px,
            ..Default::default()
        },
    );
    table.thickness = crate::tables::metrics::helpers::prefer_shared_metric(geometry.thickness, table.thickness);
    table.track_thickness =
        crate::tables::metrics::helpers::prefer_shared_metric(geometry.track_thickness, table.track_thickness);
    table.thumb_thickness =
        crate::tables::metrics::helpers::prefer_shared_metric(geometry.thumb_thickness, table.thumb_thickness);
    table.min_thumb_length =
        crate::tables::metrics::helpers::prefer_shared_metric(geometry.min_thumb_length, table.min_thumb_length);
    table.length = crate::tables::metrics::helpers::prefer_shared_metric(
        match orientation {
            xgpui_luma::controls::scrollbar::ScrollbarOrientation::Horizontal => geometry.length_h,
            xgpui_luma::controls::scrollbar::ScrollbarOrientation::Vertical => geometry.length_v,
        },
        table.length,
    );
    table
}
