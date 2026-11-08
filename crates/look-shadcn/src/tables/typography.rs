//! Resolved runtime text style and its source metadata.
use gpui::SharedString;
use xgpui_luma::theme::{ControlSize, LumaTextStyle};
use crate::{ShadcnModeTokens, TypographySource};
use crate::stylesheet::{StylesheetConfig, resolve_button_metrics_rule};

#[derive(Clone, Debug)]
pub struct ControlTypographyTable {
    pub style: LumaTextStyle,
    pub font_family: SharedString,
    pub font_family_source: TypographySource,
    pub font_size_source: TypographySource,
    pub font_weight_source: TypographySource,
    pub line_height_source: TypographySource,
}

/// Label typography for button-sized controls, including Accordion.
pub fn resolve_control_typography(mode: &ShadcnModeTokens, size: ControlSize, toggle: bool) -> ControlTypographyTable {
    resolve_control_typography_with_stylesheet(mode, mode.stylesheet(), size, toggle)
}

pub(crate) fn resolve_control_typography_with_stylesheet(
    mode: &ShadcnModeTokens,
    stylesheet: &StylesheetConfig,
    size: ControlSize,
    toggle: bool,
) -> ControlTypographyTable {
    let mut style = mode.typography.text.label;
    let mut font_size_source = TypographySource::Scaffold { path: "LumaTypography.text.label.size".into() };
    let mut line_height_source = TypographySource::Scaffold { path: "LumaTypography.text.label.line_height".into() };
    let rule = if toggle {
        stylesheet.toggle.metrics_for_size(size)
    } else {
        stylesheet.button.metrics_for_size(size)
    };
    if let Some(rule) = rule {
        let metrics = resolve_button_metrics_rule(rule, &mode.metrics, size);
        let base_size = style.size;
        style.size = metrics.font_size;
        let family = if toggle { "toggle" } else { "button" };
        let size_key = super::metrics::helpers::control_size_key(size);
        font_size_source =
            TypographySource::Constant { label: format!("stylesheet [{family}.metrics.{size_key}].font_size") };
        if base_size > 0.0 {
            style.line_height = style.size * (style.line_height / base_size);
            line_height_source = TypographySource::Derived { note: "font_size × label line-height ratio".into() };
        }
    }
    let family = if toggle { "toggle" } else { "button" };
    let (font, line, font_path, line_path) = common_typography_overrides(stylesheet, size, toggle);
    if let Some(font) = font {
        let ratio = if style.size > 0.0 {
            style.line_height / style.size
        } else {
            1.0
        };
        style.size = font;
        style.line_height = font * ratio;
        font_size_source =
            TypographySource::Constant { label: format!("stylesheet · common.{family}.{font_path}.font_size") };
        line_height_source = TypographySource::Derived { note: "font_size × label line-height ratio".into() };
    }
    if let Some(line) = line {
        style.line_height = line;
        line_height_source =
            TypographySource::Constant { label: format!("stylesheet · common.{family}.{line_path}.line_height") };
    }
    ControlTypographyTable {
        style,
        font_family: mode.typography.font.sans.family.clone().into(),
        font_family_source: if mode.catalog.get("font-sans").is_some() {
            TypographySource::CssVar { token: "font-sans".into() }
        } else {
            TypographySource::Scaffold { path: "LumaTypography.font.sans.family".into() }
        },
        font_size_source,
        font_weight_source: TypographySource::Scaffold { path: "LumaTypography.text.label.weight".into() },
        line_height_source,
    }
}

pub(crate) fn common_typography_overrides(
    stylesheet: &StylesheetConfig,
    size: ControlSize,
    toggle: bool,
) -> (Option<f32>, Option<f32>, String, String) {
    let key = super::metrics::helpers::control_size_key(size);
    let (specific_font, specific_line, general_font, general_line) = if toggle {
        let specific = stylesheet.common.toggle.sizes.get(key);
        (
            specific.and_then(|value| value.font_size),
            specific.and_then(|value| value.line_height),
            stylesheet.common.toggle.geometry.font_size,
            stylesheet.common.toggle.geometry.line_height,
        )
    } else {
        let specific = stylesheet.common.button.sizes.get(key);
        (
            specific.and_then(|value| value.font_size),
            specific.and_then(|value| value.line_height),
            stylesheet.common.button.geometry.font_size,
            stylesheet.common.button.geometry.line_height,
        )
    };
    (
        specific_font.or(general_font),
        specific_line.or(general_line),
        if specific_font.is_some() {
            format!("sizes.{key}")
        } else {
            "geometry".into()
        },
        if specific_line.is_some() {
            format!("sizes.{key}")
        } else {
            "geometry".into()
        },
    )
}

/// Apply common typography while preserving the existing ratio for an unset line height.
pub(crate) fn apply_resolved_geometry_typography(
    style: &mut LumaTextStyle,
    font: &xgpui_luma::theme::provenance::ResolvedMetric,
    line: &xgpui_luma::theme::provenance::ResolvedMetric,
) {
    let ratio = if style.size > 0.0 {
        style.line_height / style.size
    } else {
        1.0
    };
    style.size = font.value_px;
    style.line_height = if matches!(line.source, xgpui_luma::theme::provenance::MetricSource::Constant { .. })
        && !matches!(font.source, xgpui_luma::theme::provenance::MetricSource::Constant { .. })
    {
        font.value_px * ratio
    } else {
        line.value_px
    };
}
