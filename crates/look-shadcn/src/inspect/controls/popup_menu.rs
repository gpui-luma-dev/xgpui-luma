//! Inspect metadata for `popup_menu`.

use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::controls::popup_menu::PopupMenuTriggerStyle;
use xgpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use crate::{ResolvedColor, ShadcnButtonStyle, ShadcnModeTokens};

use super::button::{inspect_button_color_palette, inspect_button_metrics};

pub struct PopupMenuInspectPalette {
    pub trigger_style: PopupMenuTriggerStyle,
    pub trigger_background: ResolvedColor,
    pub trigger_foreground: ResolvedColor,
    pub trigger_border: ResolvedColor,
    pub menu: crate::inspect::controls::floating_menu::FloatingMenuInspectPalette,
}

#[derive(Clone, Debug)]
pub struct PopupMenuInspectMetrics {
    pub trigger: crate::inspect::controls::button::ButtonInspectMetrics,
    pub menu: crate::inspect::controls::floating_menu::FloatingMenuInspectMetrics,
}

fn button_style(trigger_style: PopupMenuTriggerStyle) -> ShadcnButtonStyle {
    match trigger_style {
        PopupMenuTriggerStyle::Primary => ShadcnButtonStyle::Primary,
        PopupMenuTriggerStyle::Secondary => ShadcnButtonStyle::Secondary,
        PopupMenuTriggerStyle::Outline => ShadcnButtonStyle::Outline,
        PopupMenuTriggerStyle::Ghost => ShadcnButtonStyle::Ghost,
    }
}

pub fn inspect_popup_menu_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    trigger_style: PopupMenuTriggerStyle,
    state: InteractionState,
    size: ControlSize,
) -> PopupMenuInspectPalette {
    let trigger =
        inspect_button_color_palette(mode, theme_mode, button_style(trigger_style), ButtonFamilyRole::Text, state);
    let menu = crate::inspect::controls::floating_menu::inspect_floating_menu_color_palette(mode, theme_mode, size);

    PopupMenuInspectPalette {
        trigger_style,
        trigger_background: trigger.background,
        trigger_foreground: trigger.foreground,
        trigger_border: trigger.border,
        menu,
    }
}

pub fn inspect_popup_menu_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    trigger_style: PopupMenuTriggerStyle,
    size: ControlSize,
) -> PopupMenuInspectMetrics {
    let mut result = PopupMenuInspectMetrics {
        trigger: inspect_button_metrics(
            mode,
            theme_mode,
            button_style(trigger_style),
            ButtonFamilyRole::Text,
            size,
            InteractionState::default(),
        ),
        menu: crate::inspect::controls::floating_menu::inspect_floating_menu_metrics(mode, theme_mode, size),
    };
    let scale = crate::controls::button::button_box_scale(
        &crate::LookContext::new(mode, theme_mode, InteractionState::default()),
        mode.stylesheet(),
        size,
        1.0,
    );
    let geometry = crate::controls::popup_menu::popup_menu_geometry(mode, size, &scale, &mode.typography.text.label);
    result.trigger.height =
        crate::tables::metrics::helpers::prefer_shared_metric(geometry.height, result.trigger.height);
    result.trigger.padding_x =
        crate::tables::metrics::helpers::prefer_shared_metric(geometry.padding_x, result.trigger.padding_x);
    result.trigger.padding_y =
        crate::tables::metrics::helpers::prefer_shared_metric(geometry.padding_y, result.trigger.padding_y);
    result.trigger.gap = crate::tables::metrics::helpers::prefer_shared_metric(geometry.gap, result.trigger.gap);
    result.trigger.icon_size =
        crate::tables::metrics::helpers::prefer_shared_metric(geometry.icon_size, result.trigger.icon_size);
    result.trigger.radius =
        crate::tables::metrics::helpers::prefer_shared_metric(geometry.radius, result.trigger.radius);
    result
}
