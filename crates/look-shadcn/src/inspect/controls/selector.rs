//! Inspect metadata for `selector`.

use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::controls::selector::SelectorTriggerStyle;
use xgpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use crate::{ResolvedColor, ShadcnButtonStyle, ShadcnModeTokens};

use super::button::{inspect_button_color_palette, inspect_button_metrics};
use super::floating_menu::{FloatingMenuInspectMetrics, FloatingMenuInspectPalette};

pub struct SelectorInspectPalette {
    pub trigger_style: SelectorTriggerStyle,
    pub trigger_background: ResolvedColor,
    pub trigger_foreground: ResolvedColor,
    pub trigger_border: ResolvedColor,
    pub items_panel: FloatingMenuInspectPalette,
}

#[derive(Clone, Debug)]
pub struct SelectorInspectMetrics {
    pub trigger: crate::inspect::controls::button::ButtonInspectMetrics,
    pub items_panel: FloatingMenuInspectMetrics,
}

fn button_style(trigger_style: SelectorTriggerStyle) -> ShadcnButtonStyle {
    match trigger_style {
        SelectorTriggerStyle::Outline => ShadcnButtonStyle::Outline,
        SelectorTriggerStyle::Ghost => ShadcnButtonStyle::Ghost,
    }
}

pub fn inspect_selector_color_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    trigger_style: SelectorTriggerStyle,
    state: InteractionState,
    size: ControlSize,
) -> SelectorInspectPalette {
    let menu = crate::inspect::controls::floating_menu::inspect_floating_menu_color_palette(mode, theme_mode, size);
    let trigger =
        inspect_button_color_palette(mode, theme_mode, button_style(trigger_style), ButtonFamilyRole::Text, state);

    SelectorInspectPalette {
        trigger_style,
        trigger_background: trigger.background,
        trigger_foreground: trigger.foreground,
        trigger_border: trigger.border,
        items_panel: menu,
    }
}

pub fn inspect_selector_metrics(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    trigger_style: SelectorTriggerStyle,
    size: ControlSize,
) -> SelectorInspectMetrics {
    let mut result = SelectorInspectMetrics {
        trigger: inspect_button_metrics(
            mode,
            theme_mode,
            button_style(trigger_style),
            ButtonFamilyRole::Text,
            size,
            InteractionState::default(),
        ),
        items_panel: crate::inspect::controls::floating_menu::inspect_floating_menu_metrics(mode, theme_mode, size),
    };
    let scale = xgpui_luma::theme::StandardBoxScale::compute(size, &mode.metrics, 1.0);
    let geometry = crate::controls::selector::selector_geometry(mode, size, &scale, &mode.typography.text.label);
    use crate::catalog::SpacingField;
    use crate::tables::metrics::helpers::{
        control_size_key, radius_metric, scaffold_control_metric, spacing_control_metric,
    };
    result.trigger.height = scaffold_control_metric(control_size_key(size), "control_height", scale.height);
    result.trigger.padding_x = spacing_control_metric(&mode.catalog, size, SpacingField::PaddingX, scale.padding_x);
    result.trigger.padding_y = spacing_control_metric(&mode.catalog, size, SpacingField::PaddingY, scale.padding_y);
    result.trigger.gap = spacing_control_metric(&mode.catalog, size, SpacingField::Gap, scale.gap);
    result.trigger.radius = radius_metric(&mode.catalog, size, scale.radius);
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
    let panel = mode.stylesheet().common.selector_list.inspect_geometry(
        size.stylesheet_key(),
        xgpui_luma::theme::stylesheet::SelectorListGeometry {
            padding: result.items_panel.padding.value_px,
            min_width: result.items_panel.min_width.value_px,
            item_height: result.items_panel.item_height.value_px,
            item_padding_x: result.items_panel.item_padding_x.value_px,
            item_gap: result.items_panel.item_gap.value_px,
            item_icon_size: result.items_panel.item_icon_size.value_px,
            radius: result.items_panel.radius.value_px,
            item_radius: result.items_panel.item_radius.value_px,
            ..Default::default()
        },
    );
    result.items_panel.padding =
        crate::tables::metrics::helpers::prefer_shared_metric(panel.padding, result.items_panel.padding);
    result.items_panel.min_width =
        crate::tables::metrics::helpers::prefer_shared_metric(panel.min_width, result.items_panel.min_width);
    result.items_panel.item_height =
        crate::tables::metrics::helpers::prefer_shared_metric(panel.item_height, result.items_panel.item_height);
    result.items_panel.item_padding_x =
        crate::tables::metrics::helpers::prefer_shared_metric(panel.item_padding_x, result.items_panel.item_padding_x);
    result.items_panel.item_gap =
        crate::tables::metrics::helpers::prefer_shared_metric(panel.item_gap, result.items_panel.item_gap);
    result.items_panel.item_icon_size =
        crate::tables::metrics::helpers::prefer_shared_metric(panel.item_icon_size, result.items_panel.item_icon_size);
    result.items_panel.radius =
        crate::tables::metrics::helpers::prefer_shared_metric(panel.radius, result.items_panel.radius);
    result.items_panel.item_radius =
        crate::tables::metrics::helpers::prefer_shared_metric(panel.item_radius, result.items_panel.item_radius);
    result
}
