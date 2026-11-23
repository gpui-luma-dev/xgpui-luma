//! Resolved runtime text style and its source metadata.
use gpui::SharedString;
use xgpui_luma::theme::{LumaTextStyle};
use crate::{ShadcnModeTokens, TypographySource};
use crate::stylesheet::{StylesheetConfig, resolve_button_metrics_rule};
use crate::size::ShadcnSize;

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
pub fn resolve_control_typography(mode: &ShadcnModeTokens, size: ShadcnSize, toggle: bool) -> ControlTypographyTable {
    resolve_control_typography_with_stylesheet(mode, mode.stylesheet(), size, toggle)
}

pub(crate) fn resolve_control_typography_with_stylesheet(
    mode: &ShadcnModeTokens,
    stylesheet: &StylesheetConfig,
    size: ShadcnSize,
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
        let metrics = resolve_button_metrics_rule(rule, &mode.control_sizes, size);
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
    if toggle {
        // The toggle's own label size, as it draws: its `.lss` (#287).
        let theme = mode.style_theme();
        let key = super::metrics::helpers::control_size_key(size);
        if let Ok(key) = theme.key_named("toggle", &[("size", key)], &[]) {
            let resolved = theme.resolve(&key);
            let number = |name| resolved.root().value(name).and_then(xgpui_luma::style::lss::Value::as_number);
            if let (Some(font), Some(line)) = (number("font_size"), number("line_height")) {
                style.size = font;
                style.line_height = line;
                font_size_source = TypographySource::Constant { label: "toggle.lss · font_size".into() };
                line_height_source = TypographySource::Constant { label: "toggle.lss · line_height".into() };
            }
        }
    }
    let (font, line, font_path, line_path) = if toggle {
        (None, None, String::new(), String::new())
    } else {
        common_typography_overrides(stylesheet, size)
    };
    if let Some(font) = font {
        let ratio = if style.size > 0.0 {
            style.line_height / style.size
        } else {
            1.0
        };
        style.size = font;
        style.line_height = font * ratio;
        font_size_source =
            TypographySource::Constant { label: format!("stylesheet · common.button.{font_path}.font_size") };
        line_height_source = TypographySource::Derived { note: "font_size × label line-height ratio".into() };
    }
    if let Some(line) = line {
        style.line_height = line;
        line_height_source =
            TypographySource::Constant { label: format!("stylesheet · common.button.{line_path}.line_height") };
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
    size: ShadcnSize,
) -> (Option<f32>, Option<f32>, String, String) {
    let key = super::metrics::helpers::control_size_key(size);
    let specific = stylesheet.common.button.sizes.get(key);
    let (specific_font, specific_line, general_font, general_line) = (
        specific.and_then(|value| value.font_size),
        specific.and_then(|value| value.line_height),
        stylesheet.common.button.geometry.font_size,
        stylesheet.common.button.geometry.line_height,
    );
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

#[cfg(test)]
mod tests {
    use xgpui_luma::style::lss::Value;

    use super::*;

    /// The toggle's label typography is what the toggle draws: its `.lss` font size and line height
    /// at each size (#287).
    #[test]
    fn toggle_typography_reads_the_toggle_lss() {
        let look = crate::ShadcnLook::from_css_str(crate::FALLBACK_CSS).expect("catalog");
        let mode = look.mode_tokens();
        let theme = mode.style_theme();
        for size in [ShadcnSize::Sm, ShadcnSize::Md, ShadcnSize::Lg] {
            let key = theme
                .key_named("toggle", &[("size", super::super::metrics::helpers::control_size_key(size))], &[])
                .expect("key");
            let resolved = theme.resolve(&key);
            let number = |name| resolved.root().value(name).and_then(Value::as_number).expect(name);
            let table = resolve_control_typography(&mode, size, true);
            assert_eq!((table.style.size, table.style.line_height), (number("font_size"), number("line_height")));
        }
    }
}
