#![allow(deprecated)] // SelectorLook, SelectorPalette and the panel look are this module's compatibility types.

use std::sync::{Arc, OnceLock};

use gpui::{BoxShadow, Hsla};

use crate::controls::selector_list::SelectorItemsPanelLook;
use crate::theme::{InteractionState, LumaTextStyle, MetricTokens, StandardBoxScale};

use super::model::SelectorTriggerStyle;

/// A compatibility type (#433): the template draws from the trigger's resolved style.
#[deprecated(
    note = "the template draws from `SelectorTheme::style()`; resolve it with `resolve_selector_trigger` (#433)"
)]
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

/// A compatibility type (#433): the template draws from the trigger's resolved style. Build one with
/// [`SelectorLook::from_resolved`]; it may gain fields.
#[deprecated(
    note = "the template draws from `SelectorTheme::style()`; resolve it with `resolve_selector_trigger` (#433)"
)]
#[derive(Clone, Debug)]
#[non_exhaustive]
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
    /// The trigger's fade in this state: the root part's resolved `opacity` (`:disabled { opacity }`), 1 when unset.
    pub trigger_opacity: f32,
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

impl SelectorLook {
    /// The look `resolved` (a trigger's resolved style) describes, with `items_panel` as the popup: the
    /// compatibility path for code that still reads a `SelectorLook`. Templates read the resolved style
    /// directly (#433).
    pub fn from_resolved(
        resolved: &xgpui_luma_style::Resolved,
        items_panel: SelectorItemsPanelLook,
        without_elevation: bool,
    ) -> Self {
        use crate::style::PartRead;
        let root = resolved.root();
        Self {
            trigger_background: root.color("background"),
            trigger_foreground: root.color("foreground"),
            trigger_icon: root.color("icon"),
            trigger_border: root.color_opt("border"),
            trigger_shadow: if without_elevation { None } else { root.box_shadow_opt() },
            trigger_typography: root.typography(14.0, 20.0),
            trigger_radius: root.number("radius"),
            trigger_padding_x: root.number("padding_x"),
            trigger_padding_y: root.number("padding_y"),
            trigger_gap: root.number("gap"),
            trigger_height: root.number("height"),
            trigger_icon_size: root.number("icon_size"),
            trigger_focus_border: root.color("focus_border"),
            trigger_opacity: root.opacity(),
            menu_offset_y: root.number("menu_offset_y"),
            items_panel,
        }
    }
}

pub trait SelectorTheme: Send + Sync {
    /// The trigger's style (#433): the binding the template resolves in the trigger's style and state and draws
    /// from (see [`resolve_selector_trigger`](super::resolve_selector_trigger)).
    fn style(&self) -> crate::style::StyleBinding {
        crate::style::StyleBinding::sdk("selector")
    }

    /// The popup panel's style (#433): a `floating_menu` binding the panel template resolves and draws from.
    fn menu(&self) -> crate::style::StyleBinding {
        crate::style::StyleBinding::sdk("floating_menu")
    }

    /// The popup panel built from [`Self::menu`], for code that still reads a [`SelectorItemsPanelLook`].
    #[deprecated(note = "the panel template draws from `menu()`; resolve it and read its parts instead (#433)")]
    fn items_panel(&self) -> SelectorItemsPanelLook {
        SelectorItemsPanelLook::from_style(&self.menu())
    }

    /// The trigger's colors, built from [`Self::style`] for code that still reads a [`SelectorPalette`].
    #[deprecated(note = "the template draws from `style()`; resolve it with `resolve_selector_trigger` (#433)")]
    fn resolve(
        &self,
        trigger_style: SelectorTriggerStyle,
        state: InteractionState,
        without_elevation: bool,
    ) -> SelectorPalette {
        let visual = SelectorVisualState { interaction: state, ..Default::default() };
        let look = SelectorLook::from_resolved(
            &super::styled::resolve_trigger(&self.style(), trigger_style, visual, 1.0),
            self.items_panel(),
            without_elevation,
        );
        SelectorPalette {
            trigger_focus_border: look.trigger_focus_border,
            trigger_background: look.trigger_background,
            trigger_foreground: look.trigger_foreground,
            trigger_icon: look.trigger_icon,
            trigger_border: look.trigger_border,
            trigger_shadow: look.trigger_shadow,
            trigger_typography: look.trigger_typography,
            items_panel: look.items_panel,
        }
    }

    /// The trigger's border width and focus geometry at rest.
    fn metrics(&self) -> MetricTokens {
        super::styled::metrics(&self.style())
    }

    /// The trigger's look in `state`, built from [`Self::style`] for code that still reads a [`SelectorLook`].
    #[deprecated(note = "the template draws from `style()`; resolve it with `resolve_selector_trigger` (#433)")]
    fn resolve_look(
        &self,
        trigger_style: SelectorTriggerStyle,
        state: InteractionState,
        _scale: &StandardBoxScale,
        without_elevation: bool,
    ) -> SelectorLook {
        let visual = SelectorVisualState { interaction: state, ..Default::default() };
        SelectorLook::from_resolved(
            &super::styled::resolve_trigger(&self.style(), trigger_style, visual, 1.0),
            self.items_panel(),
            without_elevation,
        )
    }

    /// The trigger as drawn (an open trigger looks hovered), built from [`Self::style`] for code that still reads
    /// a [`SelectorLook`].
    #[deprecated(note = "the template draws from `style()`; resolve it with `resolve_selector_trigger` (#433)")]
    fn resolve_visual_look(
        &self,
        trigger_style: SelectorTriggerStyle,
        visual_state: SelectorVisualState,
        _scale: &StandardBoxScale,
        without_elevation: bool,
    ) -> SelectorLook {
        SelectorLook::from_resolved(
            &super::styled::resolve_trigger(&self.style(), trigger_style, visual_state, 1.0),
            self.items_panel(),
            without_elevation,
        )
    }
}

/// The SDK's selector theme: `selector.lss` alone.
pub fn default_selector_theme() -> Arc<dyn SelectorTheme> {
    static THEME: OnceLock<Arc<dyn SelectorTheme>> = OnceLock::new();

    THEME.get_or_init(|| Arc::new(super::styled::StyledSelectorTheme::sdk())).clone()
}

/// The trigger's button face (#430): a selector look with its theme's border width and focus geometry.
/// These test the compatibility methods themselves, which build their looks from `style()`.
#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;

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
        use crate::style::{ThemeSource, fixture};
        for lss in fixture::themes() {
            let theme = super::super::styled::StyledSelectorTheme::new(
                ThemeSource::Fixed(lss.clone()),
                Arc::new(|| crate::style::StyleBinding::sdk("floating_menu")),
            );
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
            let token = |path: &str| fixture::token(&lss, path, "color").as_color().map(Hsla::from).expect("a color");
            assert_eq!(look.trigger_border, Some(token("palette.form.input.invalid_border")), "{}", lss.mode());
            assert_eq!(look.trigger_focus_border, token("palette.focus.ring"), "{}", lss.mode());
        }
    }
}
