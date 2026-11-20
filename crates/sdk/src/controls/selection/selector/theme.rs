use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla};

use crate::controls::selector_list::SelectorItemsPanelLook;
use crate::theme::{InteractionState, LumaTextStyle, MetricTokens, StandardBoxScale};

use super::model::SelectorTriggerStyle;

#[derive(Clone, Debug)]
pub struct SelectorPalette {
    /// Independent focus paint; transparency retains the configured focus space.
    pub trigger_focus_border: Hsla,
    pub trigger_background: Hsla,
    pub trigger_foreground: Hsla,
    pub trigger_icon: Hsla,
    pub trigger_border: Option<Hsla>,
    pub trigger_shadow: Option<Vec<BoxShadow>>,
    pub trigger_typography: LumaTextStyle,
    pub items_panel: SelectorItemsPanelLook,
}

#[derive(Clone, Debug)]
pub struct SelectorLook {
    pub trigger_background: Hsla,
    pub trigger_foreground: Hsla,
    pub trigger_icon: Hsla,
    pub trigger_border: Option<Hsla>,
    pub trigger_shadow: Option<Vec<BoxShadow>>,
    pub trigger_typography: LumaTextStyle,
    pub trigger_radius: f32,
    pub trigger_padding_x: f32,
    pub trigger_padding_y: f32,
    pub trigger_gap: f32,
    pub trigger_height: f32,
    pub trigger_icon_size: f32,
    /// Independent adorner paint; transparent retains focus space.
    pub trigger_focus_border: Hsla,
    pub menu_offset_y: f32,
    pub items_panel: SelectorItemsPanelLook,
}

/// Selector-owned state that is relevant to trigger presentation in addition to
/// generic pointer/focus interaction state.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SelectorVisualState {
    pub interaction: InteractionState,
    pub open: bool,
    pub selected: bool,
    pub invalid: bool,
}

impl SelectorVisualState {
    pub fn resolved_interaction(self) -> InteractionState {
        InteractionState {
            // Open is an active trigger state, but never overrides disabled or pressed.
            hovered: self.open || self.interaction.hovered,
            invalid: self.invalid || self.interaction.invalid,
            ..self.interaction
        }
    }
}

pub trait SelectorTheme: Send + Sync {
    fn resolve(
        &self,
        trigger_style: SelectorTriggerStyle,
        state: InteractionState,
        without_elevation: bool,
    ) -> SelectorPalette;

    fn metrics(&self) -> MetricTokens;

    fn resolve_look(
        &self,
        trigger_style: SelectorTriggerStyle,
        state: InteractionState,
        scale: &StandardBoxScale,
        without_elevation: bool,
    ) -> SelectorLook {
        compose_selector_look(&self.resolve(trigger_style, state, without_elevation), scale)
    }

    fn resolve_visual_look(
        &self,
        trigger_style: SelectorTriggerStyle,
        visual_state: SelectorVisualState,
        scale: &StandardBoxScale,
        without_elevation: bool,
    ) -> SelectorLook {
        self.resolve_look(trigger_style, visual_state.resolved_interaction(), scale, without_elevation)
    }
}

/// The SDK's selector theme: `selector.lss` alone.
pub fn default_selector_theme() -> Arc<dyn SelectorTheme> {
    static THEME: OnceLock<Arc<dyn SelectorTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledSelectorTheme::sdk())).clone()
}

/// A selector look from a palette at a box scale (themes that resolve colors only).
pub(crate) fn compose_selector_look(palette: &SelectorPalette, scale: &StandardBoxScale) -> SelectorLook {
    SelectorLook {
        trigger_background: palette.trigger_background,
        trigger_foreground: palette.trigger_foreground,
        trigger_icon: palette.trigger_icon,
        trigger_border: palette.trigger_border,
        trigger_shadow: palette.trigger_shadow.clone(),
        trigger_typography: palette.trigger_typography,
        trigger_radius: scale.radius,
        trigger_padding_x: scale.padding_x,
        trigger_padding_y: scale.padding_y,
        trigger_gap: scale.gap,
        trigger_height: scale.height,
        trigger_icon_size: scale.icon_size,
        trigger_focus_border: palette.trigger_focus_border,
        menu_offset_y: scale.gap
            * crate::theme::stylesheet::CommonStylesheet::sdk_default()
                .selector
                .geometry
                .menu_offset_scale
                .unwrap_or_default(),
        items_panel: palette.items_panel.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeTokens;

    #[test]
    fn disabled_state_takes_precedence_over_hover_and_pressed() {
        let theme = default_selector_theme();
        let state = InteractionState { hovered: true, pressed: true, disabled: true, ..InteractionState::default() };

        let disabled = theme.resolve(SelectorTriggerStyle::Outline, state, false);
        let baseline = theme.resolve(
            SelectorTriggerStyle::Outline,
            InteractionState { disabled: true, ..InteractionState::default() },
            false,
        );

        assert_eq!(disabled.trigger_background, baseline.trigger_background);
        assert_eq!(disabled.trigger_foreground, baseline.trigger_foreground);
        assert_eq!(disabled.trigger_border, baseline.trigger_border);
    }

    #[test]
    fn pressed_state_takes_precedence_over_hover() {
        let theme = default_selector_theme();
        let pressed = theme.resolve(
            SelectorTriggerStyle::Outline,
            InteractionState { pressed: true, hovered: true, ..InteractionState::default() },
            false,
        );
        let baseline = theme.resolve(
            SelectorTriggerStyle::Outline,
            InteractionState { pressed: true, ..InteractionState::default() },
            false,
        );

        assert_eq!(pressed.trigger_background, baseline.trigger_background);
        assert_eq!(pressed.trigger_foreground, baseline.trigger_foreground);
        assert_eq!(pressed.trigger_border, baseline.trigger_border);
    }

    #[test]
    fn default_theme_does_not_add_focus_visuals() {
        let theme = default_selector_theme();
        let focused = theme.resolve(
            SelectorTriggerStyle::Outline,
            InteractionState { focused: true, ..InteractionState::default() },
            false,
        );
        let baseline = theme.resolve(SelectorTriggerStyle::Outline, InteractionState::default(), false);

        assert_eq!(focused.trigger_background, baseline.trigger_background);
        assert_eq!(focused.trigger_foreground, baseline.trigger_foreground);
        assert_eq!(focused.trigger_border, baseline.trigger_border);
        assert_eq!(focused.trigger_shadow, baseline.trigger_shadow);

        let scale = StandardBoxScale::compute(&theme.metrics(), 1.0);
        let focused_look = theme.resolve_visual_look(
            SelectorTriggerStyle::Outline,
            SelectorVisualState {
                interaction: InteractionState { focused: true, ..InteractionState::default() },
                ..Default::default()
            },
            &scale,
            false,
        );
        assert!(focused_look.trigger_focus_border.a > 0.0);
    }

    #[test]
    fn open_state_matches_hover_and_disabled_open_matches_disabled() {
        let theme = default_selector_theme();
        let scale = StandardBoxScale::compute(&theme.metrics(), 1.0);
        let open = theme.resolve_visual_look(
            SelectorTriggerStyle::Outline,
            SelectorVisualState { open: true, ..Default::default() },
            &scale,
            false,
        );
        let hover = theme.resolve_look(
            SelectorTriggerStyle::Outline,
            InteractionState { hovered: true, ..Default::default() },
            &scale,
            false,
        );
        let disabled_open = theme.resolve_visual_look(
            SelectorTriggerStyle::Outline,
            SelectorVisualState {
                open: true,
                interaction: InteractionState { disabled: true, ..Default::default() },
                ..Default::default()
            },
            &scale,
            false,
        );
        let disabled = theme.resolve_look(
            SelectorTriggerStyle::Outline,
            InteractionState { disabled: true, ..Default::default() },
            &scale,
            false,
        );

        assert_eq!(open.trigger_background, hover.trigger_background);
        assert_eq!(open.trigger_foreground, hover.trigger_foreground);
        assert_eq!(disabled_open.trigger_background, disabled.trigger_background);
        assert_eq!(disabled_open.trigger_foreground, disabled.trigger_foreground);
    }

    #[test]
    fn invalid_state_uses_destructive_border_and_keeps_it_when_focused() {
        let theme = default_selector_theme();
        let scale = StandardBoxScale::compute(&theme.metrics(), 1.0);
        let look = theme.resolve_visual_look(
            SelectorTriggerStyle::Outline,
            SelectorVisualState {
                invalid: true,
                interaction: InteractionState { focused: true, ..Default::default() },
                ..Default::default()
            },
            &scale,
            false,
        );

        let tokens = ThemeTokens::default();
        let close =
            |a: Hsla, b: Hsla| (a.to_rgb().r - b.to_rgb().r).abs() < 2e-3 && (a.to_rgb().g - b.to_rgb().g).abs() < 2e-3;
        assert!(close(look.trigger_border.unwrap(), tokens.palette.form.input.invalid_border));
        assert!(close(look.trigger_focus_border, tokens.palette.focus.ring));
    }
}

crate::theme::stylesheet::geometry::geometry_family!(
    SelectorStylesheet,
    SelectorGeometryStylesheet,
    SelectorGeometry,
    ResolvedSelectorGeometry,
    "selector",
    [
        height,
        padding_x,
        padding_y,
        gap,
        icon_size,
        font_size,
        line_height,
        radius,
        menu_offset_y,
        menu_offset_scale
    ]
);

impl SelectorStylesheet {
    /// Native SDK geometry, equivalent to the retained stylesheet.
    pub fn native_default() -> Self {
        Self {
            geometry: SelectorGeometryStylesheet { menu_offset_scale: Some(1.0 / 2.0), ..Default::default() },
            ..Default::default()
        }
    }
}
