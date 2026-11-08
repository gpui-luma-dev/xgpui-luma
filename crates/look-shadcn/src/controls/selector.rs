//! Selector property mappings — outline/ghost command-button trigger + accent item panel.

use xgpui_luma::controls::button_family::ButtonFamilyRole;
use xgpui_luma::controls::selector::{SelectorLook, SelectorPalette, SelectorTriggerStyle, SelectorVisualState};
use xgpui_luma::theme::{ControlSize, InteractionState, StandardBoxScale, ThemeMode};

use crate::look_context::LookContext;
use crate::mode::ShadcnModeTokens;

use super::button::{ShadcnButtonStyle, button_elevation_shadow, button_palette};
use super::selector_items_panel::selector_items_panel_look;

fn shadcn_button_style(trigger_style: SelectorTriggerStyle) -> ShadcnButtonStyle {
    match trigger_style {
        SelectorTriggerStyle::Outline => ShadcnButtonStyle::Outline,
        SelectorTriggerStyle::Ghost => ShadcnButtonStyle::Ghost,
    }
}

pub fn selector_palette(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    trigger_style: SelectorTriggerStyle,
    state: InteractionState,
    without_elevation: bool,
) -> SelectorPalette {
    let ctx = LookContext::new(mode, theme_mode, state);
    let stylesheet = mode.stylesheet();
    let button_style = shadcn_button_style(trigger_style);
    let button = button_palette(&ctx, stylesheet, button_style, ButtonFamilyRole::Text, ControlSize::Md);
    let trigger_shadow = if without_elevation {
        None
    } else {
        button_elevation_shadow(&ctx, stylesheet, button_style)
    };
    let icon = ctx.catalog().color("muted-foreground").unwrap_or(button.foreground);

    SelectorPalette {
        trigger_background: button.background,
        trigger_foreground: button.foreground,
        trigger_icon: icon,
        trigger_border: button.border,
        trigger_shadow,
        trigger_typography: button.typography,
        items_panel: selector_items_panel_look(mode, theme_mode, ControlSize::Md),
    }
}

pub fn selector_look(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    trigger_style: SelectorTriggerStyle,
    state: InteractionState,
    size: ControlSize,
    scale: &StandardBoxScale,
    without_elevation: bool,
) -> SelectorLook {
    let palette = selector_palette(mode, theme_mode, trigger_style, state, without_elevation);
    let mut typography = palette.trigger_typography;
    super::typography::apply_button_metrics_typography(&mut typography, mode, size);

    let geometry = selector_geometry(mode, size, scale, &typography);
    crate::tables::typography::apply_resolved_geometry_typography(
        &mut typography,
        &geometry.font_size,
        &geometry.line_height,
    );
    let trigger_border = (!state.disabled && state.invalid)
        .then_some(mode.palette.destructive_background)
        .or(palette.trigger_border);
    SelectorLook {
        trigger_background: palette.trigger_background,
        trigger_foreground: palette.trigger_foreground,
        trigger_icon: palette.trigger_icon,
        trigger_border,
        trigger_shadow: palette.trigger_shadow,
        trigger_typography: typography,
        trigger_radius: geometry.radius.value_px,
        trigger_padding_x: geometry.padding_x.value_px,
        trigger_padding_y: geometry.padding_y.value_px,
        trigger_gap: geometry.gap.value_px,
        trigger_height: geometry.height.value_px,
        trigger_icon_size: geometry.icon_size.value_px,
        trigger_focus_border: None,
        menu_offset_y: geometry.menu_offset_y.value_px,
        items_panel: selector_items_panel_look(mode, theme_mode, size),
    }
}

pub fn selector_look_with_visual_state(
    mode: &ShadcnModeTokens,
    theme_mode: ThemeMode,
    trigger_style: SelectorTriggerStyle,
    visual_state: SelectorVisualState,
    size: ControlSize,
    scale: &StandardBoxScale,
    without_elevation: bool,
) -> SelectorLook {
    let state = visual_state.resolved_interaction();
    let mut look = selector_look(mode, theme_mode, trigger_style, state, size, scale, without_elevation);
    if visual_state.invalid && !visual_state.interaction.disabled {
        look.trigger_border = Some(mode.palette.destructive_background);
        look.trigger_focus_border = Some(mode.palette.destructive_background);
    } else if visual_state.interaction.focused && !visual_state.interaction.disabled && look.trigger_border.is_some() {
        look.trigger_focus_border = crate::focus::focus_ring_color(&mode.catalog).ok();
    }
    look
}

pub(crate) fn selector_geometry(
    mode: &ShadcnModeTokens,
    size: ControlSize,
    scale: &StandardBoxScale,
    typography: &xgpui_luma::theme::LumaTextStyle,
) -> xgpui_luma::theme::stylesheet::ResolvedSelectorGeometry {
    let icon = super::button::button_box_scale(
        &LookContext::new(mode, mode.theme_mode, InteractionState::default()),
        mode.stylesheet(),
        size,
        1.0,
    )
    .icon_size;
    let baseline = xgpui_luma::theme::stylesheet::CommonStylesheet::sdk_default()
        .selector
        .authored_geometry(size.stylesheet_key());
    let recipe = mode.stylesheet().common.selector.authored_geometry(size.stylesheet_key()).with_fallback(&baseline);
    mode.stylesheet().common.selector.inspect_geometry(
        crate::tables::metrics::helpers::control_size_key(size),
        xgpui_luma::theme::stylesheet::SelectorGeometry {
            height: scale.height,
            padding_x: scale.padding_x,
            padding_y: scale.padding_y,
            gap: scale.gap,
            icon_size: icon,
            font_size: typography.size,
            line_height: typography.line_height,
            radius: scale.radius,
            menu_offset_y: recipe.gap.unwrap_or(scale.gap) * recipe.menu_offset_scale.unwrap_or_default(),
            ..Default::default()
        },
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use xgpui_luma::controls::button_family::ButtonFamilyRole;
    use xgpui_luma::controls::selector::{SelectorTriggerStyle, SelectorVisualState};
    use xgpui_luma::theme::{ControlSize, InteractionState, ThemeMode};

    use crate::catalog::CssTokenMap;
    use crate::controls::button::{ShadcnButtonStyle, button_palette};
    use crate::look_context::LookContext;
    use crate::mode::ShadcnModeTokens;
    use crate::stylesheet::embedded_stylesheet;
    use super::{selector_look, selector_look_with_visual_state, selector_palette};

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
            ("accent-foreground".into(), "oklch(1 0 0)".into()),
            ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
            ("input".into(), "oklch(0.7200 0.0120 205.0000)".into()),
            ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
            ("card".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("popover".into(), "oklch(0.9306 0.0260 92.4020)".into()),
            ("popover-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
            ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
            ("shadow-xs".into(), "0 1px 2px 0px hsl(0 0% 0% / 0.05)".into()),
            ("shadow-sm".into(), "0 1px 2px 0px hsl(0 0% 0% / 0.10)".into()),
        ]))
    }

    #[test]
    fn outline_trigger_matches_outline_command_button() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog, ThemeMode::Light).expect("catalog");
        let state = InteractionState::default();
        let selector = selector_palette(&mode, ThemeMode::Light, SelectorTriggerStyle::Outline, state, false);
        let ctx = LookContext::new(&mode, ThemeMode::Light, state);
        let button = button_palette(
            &ctx,
            embedded_stylesheet(),
            ShadcnButtonStyle::Outline,
            ButtonFamilyRole::Text,
            ControlSize::Md,
        );

        assert_eq!(selector.trigger_background, button.background);
        assert_eq!(selector.trigger_foreground, button.foreground);
        assert_eq!(selector.trigger_border, button.border);
        assert!(selector.trigger_shadow.as_ref().is_some_and(|shadows| !shadows.is_empty()));
    }

    #[test]
    fn ghost_trigger_matches_ghost_command_button() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog, ThemeMode::Light).expect("catalog");
        let state = InteractionState::default();
        let selector = selector_palette(&mode, ThemeMode::Light, SelectorTriggerStyle::Ghost, state, false);
        let ctx = LookContext::new(&mode, ThemeMode::Light, state);
        let button = button_palette(
            &ctx,
            embedded_stylesheet(),
            ShadcnButtonStyle::Ghost,
            ButtonFamilyRole::Text,
            ControlSize::Md,
        );

        assert_eq!(selector.trigger_background, button.background);
        assert_eq!(selector.trigger_foreground, button.foreground);
        assert_eq!(selector.trigger_border, button.border);
        assert!(selector.trigger_shadow.is_none());
    }

    #[test]
    fn outline_without_elevation_clears_shadow() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog, ThemeMode::Light).expect("catalog");
        let selector =
            selector_palette(&mode, ThemeMode::Light, SelectorTriggerStyle::Outline, InteractionState::default(), true);
        assert!(selector.trigger_shadow.is_none());
    }

    #[test]
    fn open_trigger_uses_hover_palette_but_disabled_still_wins() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog, ThemeMode::Light).expect("catalog");
        let scale = xgpui_luma::theme::StandardBoxScale::compute(ControlSize::Md, &mode.metrics, 1.0);
        let open = selector_look_with_visual_state(
            &mode,
            ThemeMode::Light,
            SelectorTriggerStyle::Outline,
            SelectorVisualState { open: true, ..Default::default() },
            ControlSize::Md,
            &scale,
            false,
        );
        let hover = selector_look(
            &mode,
            ThemeMode::Light,
            SelectorTriggerStyle::Outline,
            InteractionState { hovered: true, ..Default::default() },
            ControlSize::Md,
            &scale,
            false,
        );
        let disabled_open = selector_look_with_visual_state(
            &mode,
            ThemeMode::Light,
            SelectorTriggerStyle::Outline,
            SelectorVisualState {
                open: true,
                interaction: InteractionState { disabled: true, ..Default::default() },
                ..Default::default()
            },
            ControlSize::Md,
            &scale,
            false,
        );
        let disabled = selector_look(
            &mode,
            ThemeMode::Light,
            SelectorTriggerStyle::Outline,
            InteractionState { disabled: true, ..Default::default() },
            ControlSize::Md,
            &scale,
            false,
        );

        assert_eq!(open.trigger_background, hover.trigger_background);
        assert_eq!(open.trigger_foreground, hover.trigger_foreground);
        assert_eq!(disabled_open.trigger_background, disabled.trigger_background);
        assert_eq!(disabled_open.trigger_foreground, disabled.trigger_foreground);
    }

    #[test]
    fn selector_trigger_chevron_uses_muted_icon_color() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog.clone(), ThemeMode::Dark).expect("catalog");
        let palette =
            selector_palette(&mode, ThemeMode::Dark, SelectorTriggerStyle::Outline, InteractionState::default(), false);

        assert_eq!(palette.trigger_icon, catalog.color("muted-foreground").expect("muted-foreground"));
    }

    #[test]
    fn selector_trigger_typography_uses_stylesheet_font_size() {
        let catalog = sample_catalog();
        let mode = ShadcnModeTokens::from_catalog(catalog, ThemeMode::Light).expect("catalog");
        let sm_scale = xgpui_luma::theme::StandardBoxScale::compute(ControlSize::Sm, &mode.metrics, 1.0);
        let lg_scale = xgpui_luma::theme::StandardBoxScale::compute(ControlSize::Lg, &mode.metrics, 1.0);

        let sm = selector_look(
            &mode,
            ThemeMode::Light,
            SelectorTriggerStyle::Outline,
            InteractionState::default(),
            ControlSize::Sm,
            &sm_scale,
            false,
        );
        let lg = selector_look(
            &mode,
            ThemeMode::Light,
            SelectorTriggerStyle::Outline,
            InteractionState::default(),
            ControlSize::Lg,
            &lg_scale,
            false,
        );

        assert_eq!(sm.trigger_typography.size, 12.0);
        assert_eq!(lg.trigger_typography.size, 16.0);
    }
}
