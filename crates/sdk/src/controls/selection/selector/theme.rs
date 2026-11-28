use std::sync::{Arc, OnceLock};

use crate::theme::{InteractionState, MetricTokens};

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
    /// The trigger's style (#433): the binding the template resolves in the trigger's style and state and draws
    /// from (see [`resolve_selector_trigger`](super::resolve_selector_trigger)).
    fn style(&self) -> crate::style::StyleBinding {
        crate::style::StyleBinding::sdk("selector")
    }

    /// The popup panel's style (#433): a `floating_menu` binding the panel template resolves and draws from.
    fn menu(&self) -> crate::style::StyleBinding {
        crate::style::StyleBinding::sdk("floating_menu")
    }

    /// The trigger's border width and focus geometry at rest.
    fn metrics(&self) -> MetricTokens {
        super::styled::metrics(&self.style())
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
mod tests {}
