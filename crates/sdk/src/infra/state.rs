use gpui::{FocusHandle, Window};

use crate::theme::InteractionState;

/// When a focused control draws its focus ring (#433): one rule for every control. The ring shows while the
/// control has focus and the last input was the keyboard, like GPUI's own `focus_visible` style, so clicking a
/// control focuses it without a ring and Tab shows one. Text entry is the one exception: a field with a caret
/// shows focus however it got it.
pub fn focus_visible(focused: bool, window: &Window) -> bool {
    focused && window.last_input_was_keyboard()
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ControlFocusState {
    /// The GPUI focus handle associated with the control is focused.
    pub focused: bool,
    /// The control draws its focus ring: see [`focus_visible`].
    pub focus_visible: bool,
}

impl ControlFocusState {
    pub fn from_focus_handle(enabled: bool, focus_handle: &FocusHandle, window: &Window) -> Self {
        let focused = enabled && focus_handle.is_focused(window);

        Self { focused, focus_visible: focus_visible(focused, window) }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CompositeItemState {
    pub disabled: bool,
    pub hovered: bool,
    pub pressed: bool,
    /// The item is semantically selected, checked, or current.
    pub selected: bool,
    /// The item is the active descendant inside its focused composite control.
    pub active: bool,
    /// The item should draw a keyboard focus affordance.
    pub focus_visible: bool,
}

impl CompositeItemState {
    pub fn interaction_state(self) -> InteractionState {
        InteractionState {
            hovered: self.hovered,
            pressed: self.pressed,
            focused: self.active,
            disabled: self.disabled,
            invalid: false,
        }
    }
}

/// Maps each item's state before its template draws it (#397): the item's id and the state input gives it, to the
/// state drawn. A preview, a doc page or a snapshot can draw one row hovered, pressed, focused, selected or
/// disabled among rows at rest. Only the look changes: input, selection and events still follow the real state.
pub type ItemStateOverride<Id> = std::sync::Arc<dyn Fn(&Id, CompositeItemState) -> CompositeItemState + Send + Sync>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuPath {
    Root(usize),
    Submenu { parent: usize, child: usize },
}

impl MenuPath {
    pub fn is_root(self, index: usize) -> bool {
        matches!(self, Self::Root(active) if active == index)
    }

    pub fn is_submenu(self, parent: usize, child: usize) -> bool {
        matches!(
            self,
            Self::Submenu {
                parent: active_parent,
                child: active_child,
            } if active_parent == parent && active_child == child
        )
    }
}
