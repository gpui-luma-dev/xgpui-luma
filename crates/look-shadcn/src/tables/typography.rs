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
    {
        // The control's own label size, as it draws: its `.lss` (#287).
        let control = if toggle { "toggle" } else { "button" };
        let theme = mode.style_theme();
        let key = super::metrics::helpers::control_size_key(size);
        if let Ok(key) = theme.key_named(control, &[("size", key)], &[]) {
            let resolved = theme.resolve(&key);
            let number = |name| resolved.root().value(name).and_then(xgpui_luma::style::lss::Value::as_number);
            if let (Some(font), Some(line)) = (number("font_size"), number("line_height")) {
                style.size = font;
                style.line_height = line;
                font_size_source = TypographySource::Constant { label: format!("{control}.lss · font_size") };
                line_height_source = TypographySource::Constant { label: format!("{control}.lss · line_height") };
            }
        }
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

#[cfg(test)]
mod tests {
    use xgpui_luma::style::lss::Value;

    use super::*;

    /// The button's and toggle's label typography is what each draws: its `.lss` font size and line
    /// height at each size (#287).
    #[test]
    fn control_typography_reads_the_control_lss() {
        let look = crate::ShadcnLook::from_css_str(crate::FALLBACK_CSS).expect("catalog");
        let mode = look.mode_tokens();
        let theme = mode.style_theme();
        for (control, toggle) in [("button", false), ("toggle", true)] {
            for size in [ShadcnSize::Sm, ShadcnSize::Md, ShadcnSize::Lg] {
                let key = theme
                    .key_named(control, &[("size", super::super::metrics::helpers::control_size_key(size))], &[])
                    .expect("key");
                let resolved = theme.resolve(&key);
                let number = |name| resolved.root().value(name).and_then(Value::as_number).expect(name);
                let table = resolve_control_typography(&mode, size, toggle);
                assert_eq!(
                    (table.style.size, table.style.line_height),
                    (number("font_size"), number("line_height")),
                    "{control} {size:?}"
                );
            }
        }
    }
}
