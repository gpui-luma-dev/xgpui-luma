//! Popup menu property mappings:
//!
//! | Part    | Variant                          |
//! |---------|----------------------------------|
//! | Trigger | outline or ghost command button  |
//! | Menu    | floating menu surface            |

use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::controls::popup_menu::{PopupMenuPalette, PopupMenuTriggerStyle};
use xgpui_luma::theme::{InteractionState, ThemeMode};

use crate::look_context::LookContext;
use super::button::{ShadcnButtonStyle, button_box_scale, button_elevation_shadow, button_palette};
use super::floating_menu::floating_menu_look;
use crate::mode::ShadcnModeTokens;

fn shadcn_button_style(trigger_style: PopupMenuTriggerStyle) -> ShadcnButtonStyle {
    match trigger_style {
        PopupMenuTriggerStyle::Primary => ShadcnButtonStyle::Primary,
        PopupMenuTriggerStyle::Secondary => ShadcnButtonStyle::Secondary,
        PopupMenuTriggerStyle::Outline => ShadcnButtonStyle::Outline,
        PopupMenuTriggerStyle::Ghost => ShadcnButtonStyle::Ghost,
    }
}

pub fn popup_menu_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    trigger_style: PopupMenuTriggerStyle,
    metrics: xgpui_luma::controls::popup_menu::PopupMenuTriggerMetrics,
    size: crate::ShadcnSize,
    menu_size: crate::ShadcnSize,
    state: InteractionState,
) -> PopupMenuPalette {
    let ctx = LookContext::new(mode, theme_mode, state);
    let stylesheet = mode.stylesheet();
    let button_style = shadcn_button_style(trigger_style);
    let role = if metrics.icon_only {
        ButtonFamilyRole::Icon
    } else {
        ButtonFamilyRole::Text
    };
    let mut button = button_palette(&ctx, stylesheet, button_style, role, size);
    let scale = button_box_scale(&ctx, stylesheet, size, 1.0);
    let geometry = popup_menu_geometry(mode, size, &scale, &button.typography);
    crate::tables::typography::apply_resolved_geometry_typography(
        &mut button.typography,
        &geometry.font_size,
        &geometry.line_height,
    );
    let trigger_shadow = if metrics.without_elevation {
        None
    } else {
        button_elevation_shadow(&ctx, stylesheet, button_style)
    };

    PopupMenuPalette {
        trigger_focus_border: button.focus_border,
        trigger_background: button.background,
        trigger_foreground: button.foreground,
        trigger_border: button.border,
        trigger_shadow,
        trigger_typography: button.typography,
        floating_menu: floating_menu_look(mode, theme_mode, menu_size),
    }
}

pub(crate) fn popup_menu_geometry(
    mode: &ShadcnModeTokens,
    size: crate::ShadcnSize,
    scale: &xgpui_luma::theme::StandardBoxScale,
    typography: &xgpui_luma::theme::LumaTextStyle,
) -> xgpui_luma::theme::stylesheet::ResolvedPopupMenuGeometry {
    mode.stylesheet().common.popup_menu.inspect_geometry_for(
        crate::tables::metrics::helpers::control_size_key(size),
        xgpui_luma::theme::stylesheet::PopupMenuGeometry {
            height: scale.height,
            padding_x: scale.padding_x,
            padding_y: scale.padding_y,
            gap: scale.gap,
            icon_size: scale.icon_size,
            font_size: typography.size,
            line_height: typography.line_height,
            radius: scale.radius,
            ..Default::default()
        },
    )
}

#[cfg(test)]
mod tests {

    use std::collections::BTreeMap;
    use xgpui_luma::theme::ThemeMode;

    use xgpui_luma::controls::popup_menu::{PopupMenuTriggerMetrics, PopupMenuTriggerStyle};
    use xgpui_luma::theme::InteractionState;

    use crate::catalog::CssTokenMap;
    use crate::mode::ShadcnModeTokens;
    use crate::controls::button::{ShadcnButtonStyle, button_palette};
    use super::{popup_menu_palette, shadcn_button_style};
    use crate::controls::floating_menu::floating_menu_look;
    use xgpui_luma::controls::button_family::ButtonFamilyRole;
    use crate::size::ShadcnSize;
    use crate::look_context::LookContext;
    use crate::stylesheet::embedded_stylesheet;

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
            ("popover".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("popover-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("shadow-xs".into(), "0 1px 2px 0px hsl(0 0% 0% / 0.05)".into()),
        ]))
    }

    #[test]
    fn outline_trigger_matches_outline_command_button() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let state = InteractionState::default();
        let metrics = PopupMenuTriggerMetrics::default();
        let popup = popup_menu_palette(
            &mode,
            ThemeMode::Light,
            PopupMenuTriggerStyle::Outline,
            metrics,
            ShadcnSize::Md,
            ShadcnSize::Md,
            state,
        );
        let ctx = LookContext::new(&mode, ThemeMode::Light, state);
        let button = button_palette(
            &ctx,
            embedded_stylesheet(),
            ShadcnButtonStyle::Outline,
            ButtonFamilyRole::Text,
            ShadcnSize::Md,
        );

        assert_eq!(popup.trigger_background, button.background);
        assert_eq!(popup.trigger_foreground, button.foreground);
        assert_eq!(popup.trigger_border, button.border);
        assert!(popup.trigger_shadow.as_ref().is_some_and(|shadows| !shadows.is_empty()));
    }

    #[test]
    fn ghost_trigger_matches_ghost_command_button() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let state = InteractionState::default();
        let metrics = PopupMenuTriggerMetrics::default();
        let popup = popup_menu_palette(
            &mode,
            ThemeMode::Light,
            PopupMenuTriggerStyle::Ghost,
            metrics,
            ShadcnSize::Md,
            ShadcnSize::Md,
            state,
        );
        let ctx = LookContext::new(&mode, ThemeMode::Light, state);
        let button = button_palette(
            &ctx,
            embedded_stylesheet(),
            ShadcnButtonStyle::Ghost,
            ButtonFamilyRole::Text,
            ShadcnSize::Md,
        );

        assert_eq!(popup.trigger_background, button.background);
        assert_eq!(popup.trigger_foreground, button.foreground);
        assert_eq!(popup.trigger_border, button.border);
        assert!(popup.trigger_shadow.is_none());
    }

    #[test]
    fn ghost_hovered_trigger_uses_accent_fill_like_command_button() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let metrics = PopupMenuTriggerMetrics::default();
        let hovered = InteractionState { hovered: true, ..InteractionState::default() };
        let popup = popup_menu_palette(
            &mode,
            ThemeMode::Light,
            PopupMenuTriggerStyle::Ghost,
            metrics,
            ShadcnSize::Md,
            ShadcnSize::Md,
            hovered,
        );
        let ctx = LookContext::new(&mode, ThemeMode::Light, hovered);
        let button = button_palette(
            &ctx,
            embedded_stylesheet(),
            shadcn_button_style(PopupMenuTriggerStyle::Ghost),
            ButtonFamilyRole::Text,
            ShadcnSize::Md,
        );

        assert_eq!(popup.trigger_background, button.background);
        assert_eq!(popup.trigger_foreground, button.foreground);
        assert_eq!(popup.trigger_background, catalog.color("accent").expect("accent"));
    }

    #[test]
    fn menu_size_is_independent_from_trigger_size() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Light).expect("catalog");
        let metrics = PopupMenuTriggerMetrics::default();
        let state = InteractionState::default();
        let popup = popup_menu_palette(
            &mode,
            ThemeMode::Light,
            PopupMenuTriggerStyle::Outline,
            metrics,
            ShadcnSize::Sm,
            ShadcnSize::Lg,
            state,
        );
        let ctx = LookContext::new(&mode, ThemeMode::Light, state);
        let small_trigger = button_palette(
            &ctx,
            embedded_stylesheet(),
            ShadcnButtonStyle::Outline,
            ButtonFamilyRole::Text,
            ShadcnSize::Sm,
        );
        let large_menu = floating_menu_look(&mode, ThemeMode::Light, ShadcnSize::Lg);

        assert_eq!(popup.trigger_typography.size, small_trigger.typography.size);
        assert_eq!(popup.floating_menu.item_height, large_menu.item_height);
        assert_eq!(popup.floating_menu.item_typography.size, large_menu.item_typography.size);
    }
}
