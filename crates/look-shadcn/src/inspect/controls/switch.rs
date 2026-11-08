//! Inspect metadata for `switch`.

use xgpui_luma::theme::{InteractionState, ThemeMode};
use crate::{ResolvedColor, ResolvedMetric, ShadcnButtonStyle, ShadcnModeTokens};

pub struct SwitchInspectPalette {
    pub track_background: ResolvedColor,
    pub track_border: ResolvedColor,
    pub thumb_background: ResolvedColor,
    pub thumb_border: ResolvedColor,
    pub label_color: ResolvedColor,
}

pub fn inspect_switch_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    on: bool,
    state: InteractionState,
) -> SwitchInspectPalette {
    let colors = crate::tables::resolve_switch_palette(mode, theme_mode, style, on, state);
    SwitchInspectPalette {
        track_background: colors.track_background,
        track_border: colors.track_border,
        thumb_background: colors.thumb_background,
        thumb_border: colors.thumb_border,
        label_color: colors.label_color,
    }
}

#[derive(Clone, Debug)]
pub struct SwitchInspectMetrics {
    pub track_width: ResolvedMetric,
    pub track_height: ResolvedMetric,
    pub track_padding: ResolvedMetric,
    pub thumb_size: ResolvedMetric,
    pub gap: ResolvedMetric,
    pub track_radius: ResolvedMetric,
    pub border_width: ResolvedMetric,
    pub focus_ring_width: ResolvedMetric,
    pub focus_ring_offset: ResolvedMetric,
}

pub fn inspect_switch_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    size: xgpui_luma::theme::ControlSize,
) -> SwitchInspectMetrics {
    let table = crate::tables::metrics::resolve_switch_metrics(mode, theme_mode, style, size);
    table.into()
}

pub fn inspect_switch_elevation(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    style: ShadcnButtonStyle,
    on: bool,
    state: InteractionState,
) -> crate::inspect::controls::button::ButtonInspectElevation {
    use xgpui_luma::theme::ControlSize;
    use crate::stylesheet::{resolve_stylesheet_shadow_token};
    use crate::inspect::controls::button::{button_style_key, inspect_layered_elevation};

    let look = crate::paint::switch_look(mode, theme_mode, style, on, state, ControlSize::Md);
    let layer = state.layer();
    let rule = mode.stylesheet().switch.elevation_rule_for_layer(layer);
    let rule_shadow = rule.map(|rule| rule.shadow.clone()).unwrap_or_else(|| "none".to_string());
    let token = rule.and_then(|rule| resolve_stylesheet_shadow_token(&rule.shadow));
    let shadows = if look.thumb_shadow.is_empty() {
        None
    } else {
        Some(&look.thumb_shadow)
    };
    inspect_layered_elevation(mode, theme_mode, state, rule_shadow, token, shadows, button_style_key(style))
}

impl From<crate::tables::metrics::SwitchMetricTable> for SwitchInspectMetrics {
    fn from(table: crate::tables::metrics::SwitchMetricTable) -> Self {
        Self {
            track_width: table.track_width,
            track_height: table.track_height,
            track_padding: table.track_padding,
            thumb_size: table.thumb_size,
            gap: table.gap,
            track_radius: table.track_radius,
            border_width: table.border_width,
            focus_ring_width: table.focus_ring_width,
            focus_ring_offset: table.focus_ring_offset,
        }
    }
}
