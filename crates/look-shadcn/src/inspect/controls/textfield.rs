//! Inspect metadata for `textfield`.

use xgpui_luma::theme::ThemeMode;
use crate::{ResolvedColor, ResolvedMetric, ShadcnModeTokens};

use crate::ShadcnTextFieldStyle;
use xgpui_luma::controls::textfield::TextFieldState;

pub struct TextFieldInspectPalette {
    pub background: ResolvedColor,
    pub foreground: ResolvedColor,
    pub border: ResolvedColor,
    pub placeholder: ResolvedColor,
    pub icon: ResolvedColor,
    pub selection_background: ResolvedColor,
    pub selection_foreground: ResolvedColor,
    pub caret: ResolvedColor,
}

pub fn inspect_textfield_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnTextFieldStyle,
    state: TextFieldState,
    enabled: bool,
) -> TextFieldInspectPalette {
    let colors = crate::tables::resolve_textfield_palette(mode, theme_mode, style, state, enabled);

    TextFieldInspectPalette {
        background: colors.background,
        foreground: colors.foreground,
        border: colors.border,
        placeholder: colors.placeholder,
        icon: colors.icon,
        selection_background: colors.selection_background,
        selection_foreground: colors.selection_foreground,
        caret: colors.caret,
    }
}

#[derive(Clone, Debug)]
pub struct TextFieldInspectMetrics {
    pub min_height: ResolvedMetric,
    pub icon_size: ResolvedMetric,
    pub padding_x: ResolvedMetric,
    pub padding_y: ResolvedMetric,
    pub radius: ResolvedMetric,
    pub border_width: ResolvedMetric,
    pub focus_ring_width: ResolvedMetric,
    pub focus_ring_offset: ResolvedMetric,
}

pub fn inspect_textfield_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: xgpui_luma::theme::ControlSize,
) -> TextFieldInspectMetrics {
    let table = crate::tables::metrics::resolve_textfield_metrics(mode, theme_mode, size);
    table.into()
}

pub fn inspect_textfield_elevation(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnTextFieldStyle,
    enabled: bool,
) -> crate::inspect::controls::button::ButtonInspectElevation {
    use xgpui_luma::theme::InteractionState;
    use crate::paint::textfield_palette;
    use crate::stylesheet::{find_textfield_elevation_rule, resolve_stylesheet_shadow_token};
    use crate::inspect::controls::button::inspect_layered_elevation;

    let palette = textfield_palette(mode, theme_mode, style, TextFieldState::default(), enabled);
    let rule = find_textfield_elevation_rule(mode.stylesheet(), style);
    let rule_shadow = rule.map(|rule| rule.shadow.clone()).unwrap_or_else(|| "none".to_string());
    let token = rule.and_then(|rule| resolve_stylesheet_shadow_token(&rule.shadow));
    inspect_layered_elevation(
        mode,
        theme_mode,
        InteractionState { disabled: !enabled, ..InteractionState::default() },
        rule_shadow,
        token,
        palette.shadow.as_ref(),
        textfield_style_key(style),
    )
}

fn textfield_style_key(style: ShadcnTextFieldStyle) -> &'static str {
    match style {
        ShadcnTextFieldStyle::Outline => "outline",
        ShadcnTextFieldStyle::Input => "input",
        ShadcnTextFieldStyle::Primary => "primary",
        ShadcnTextFieldStyle::Surface => "surface",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inspect::test_support::sample_catalog;

    #[test]
    fn inspect_surface_textfield_uses_muted_token() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let palette = super::inspect_textfield_color_palette(
            &mode,
            ThemeMode::Light,
            ShadcnTextFieldStyle::Surface,
            TextFieldState::default(),
            true,
        );
        assert!(matches!(
            palette.background.source,
            crate::ColorSource::CssVar { ref token } if token == "muted"
        ));
        assert!(matches!(palette.border.source, crate::ColorSource::Transparent));
    }

    #[test]
    fn textfield_metadata_covers_style_and_state_rows() {
        assert_eq!(crate::stylesheet::resolve_textfield_colors_metadata(crate::embedded_stylesheet()).len(), 16);
    }
}

impl From<crate::tables::metrics::TextFieldMetricTable> for TextFieldInspectMetrics {
    fn from(table: crate::tables::metrics::TextFieldMetricTable) -> Self {
        Self {
            min_height: table.min_height,
            icon_size: table.icon_size,
            padding_x: table.padding_x,
            padding_y: table.padding_y,
            radius: table.radius,
            border_width: table.border_width,
            focus_ring_width: table.focus_ring_width,
            focus_ring_offset: table.focus_ring_offset,
        }
    }
}
