//! Inspect metadata for `checkbox`.

use xgpui_luma::theme::{InteractionState, ThemeMode};
use crate::{ResolvedColor, ResolvedMetric, ShadcnButtonStyle, ShadcnModeTokens};

pub struct CheckboxInspectPalette {
    pub indicator_background: ResolvedColor,
    pub indicator_border: ResolvedColor,
    pub checkmark_color: ResolvedColor,
    pub label_color: ResolvedColor,
}

pub fn inspect_checkbox_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    checked: bool,
    state: InteractionState,
) -> CheckboxInspectPalette {
    inspect_checkbox_color_palette_with_stylesheet(
        &crate::LookContext::new(mode, theme_mode, state),
        mode.stylesheet(),
        style,
        checked,
    )
}

pub(crate) fn inspect_checkbox_color_palette_with_stylesheet(
    ctx: &crate::LookContext,
    stylesheet: &crate::stylesheet::StylesheetConfig,
    style: ShadcnButtonStyle,
    checked: bool,
) -> CheckboxInspectPalette {
    let colors = crate::controls::checkbox::resolve_checkbox_palette_with_stylesheet(ctx, stylesheet, style, checked);
    CheckboxInspectPalette {
        indicator_background: colors.indicator_background,
        indicator_border: colors.indicator_border,
        checkmark_color: colors.checkmark_color,
        label_color: colors.label_color,
    }
}

#[derive(Clone, Debug)]
pub struct CheckboxInspectMetrics {
    pub height: ResolvedMetric,
    pub gap: ResolvedMetric,
    pub indicator_size: ResolvedMetric,
    pub indicator_radius: ResolvedMetric,
    pub glyph_size: ResolvedMetric,
    pub control_radius: ResolvedMetric,
    pub border_width: ResolvedMetric,
    pub focus_ring_width: ResolvedMetric,
    pub focus_ring_offset: ResolvedMetric,
}

pub fn inspect_checkbox_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    size: xgpui_luma::theme::ControlSize,
) -> CheckboxInspectMetrics {
    let table = crate::tables::metrics::resolve_checkbox_metrics(mode, theme_mode, size);
    table.into()
}

pub fn inspect_checkbox_elevation(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    checked: bool,
    state: InteractionState,
) -> crate::inspect::controls::button::ButtonInspectElevation {
    inspect_checkbox_elevation_with_stylesheet(
        &crate::LookContext::new(mode, theme_mode, state),
        mode.stylesheet(),
        style,
        checked,
    )
}

pub(crate) fn inspect_checkbox_elevation_with_stylesheet(
    ctx: &crate::LookContext,
    stylesheet: &crate::stylesheet::StylesheetConfig,
    style: ShadcnButtonStyle,
    checked: bool,
) -> crate::inspect::controls::button::ButtonInspectElevation {
    use xgpui_luma::theme::ControlSize;
    use crate::stylesheet::resolve_stylesheet_shadow_token;
    use crate::inspect::controls::button::{button_style_key, inspect_layered_elevation};

    let look =
        crate::controls::checkbox::checkbox_look_with_stylesheet(ctx, stylesheet, style, checked, ControlSize::Md);
    let layer = crate::controls::choice_indicator::choice_indicator_color_layer(ctx.state);
    let rule = (style != ShadcnButtonStyle::ContentOnly)
        .then(|| stylesheet.checkbox.elevation_rule_for_layer(layer))
        .flatten();
    let rule_shadow = rule.map(|rule| rule.shadow.clone()).unwrap_or_else(|| "none".to_string());
    let token = rule.and_then(|rule| resolve_stylesheet_shadow_token(&rule.shadow));
    inspect_layered_elevation(
        ctx.tokens,
        ctx.theme_mode,
        ctx.state,
        rule_shadow,
        token,
        look.indicator_shadow.as_ref(),
        button_style_key(style),
    )
}

impl From<crate::tables::metrics::CheckboxMetricTable> for CheckboxInspectMetrics {
    fn from(table: crate::tables::metrics::CheckboxMetricTable) -> Self {
        Self {
            height: table.height,
            gap: table.gap,
            indicator_size: table.indicator_size,
            indicator_radius: table.indicator_radius,
            glyph_size: table.glyph_size,
            control_radius: table.control_radius,
            border_width: table.border_width,
            focus_ring_width: table.focus_ring_width,
            focus_ring_offset: table.focus_ring_offset,
        }
    }
}
