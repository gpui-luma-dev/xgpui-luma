use gpui::{Context, FocusHandle, Window};

use crate::infra::state::focus_visible;
use crate::theme::InteractionState;

pub(crate) struct ControlInteraction {
    state: InteractionState,
    focus_handle: FocusHandle,
    tab_stop: bool,
}

impl ControlInteraction {
    pub fn new<T>(enabled: bool, cx: &mut Context<T>) -> Self {
        Self::new_with_tab_stop(enabled, true, cx)
    }

    pub fn new_with_tab_stop<T>(enabled: bool, tab_stop: bool, cx: &mut Context<T>) -> Self {
        Self {
            state: InteractionState { disabled: !enabled, ..InteractionState::default() },
            focus_handle: cx.focus_handle().tab_stop(enabled && tab_stop),
            tab_stop,
        }
    }

    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    pub fn enabled(&self) -> bool {
        !self.state.disabled
    }

    pub fn hovered(&self) -> bool {
        self.state.hovered
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.state.disabled = !enabled;
        self.focus_handle = self.focus_handle.clone().tab_stop(enabled && self.tab_stop);

        if !enabled {
            self.state.hovered = false;
            self.state.pressed = false;
        }
    }

    pub fn set_tab_stop(&mut self, enabled: bool, tab_stop: bool) {
        self.tab_stop = tab_stop;
        self.focus_handle = self.focus_handle.clone().tab_stop(enabled && tab_stop);
    }

    /// The state the control draws. `focused` follows the focus-ring rule ([`focus_visible`]): a clicked
    /// control draws no ring.
    pub fn render_state(&self, enabled: bool, window: &Window) -> InteractionState {
        let state = self.attachment_state(enabled, window);
        InteractionState { focused: focus_visible(state.focused, window), ..state }
    }

    /// The state attachments such as tooltips read: `focused` is the control's real focus, so a tooltip opened
    /// from keyboard focus stays up when the pointer moves.
    pub fn attachment_state(&self, enabled: bool, window: &Window) -> InteractionState {
        InteractionState { focused: enabled && self.focus_handle.is_focused(window), disabled: !enabled, ..self.state }
    }

    /// Track the pointer entering or leaving. A disabled control never shows
    /// hover (#433); it can still lose a hover it had.
    pub fn handle_hover(&mut self, hovered: bool) -> bool {
        let hovered = hovered && !self.state.disabled;
        if self.state.hovered == hovered {
            return false;
        }

        self.state.hovered = hovered;
        true
    }

    pub fn handle_mouse_down<T>(&mut self, enabled: bool, window: &mut Window, cx: &mut Context<T>) -> bool {
        if !enabled {
            return false;
        }

        self.state.pressed = true;
        self.focus_handle.focus(window, cx);
        true
    }

    pub fn handle_secondary_mouse_down(&mut self, enabled: bool) -> bool {
        if !enabled {
            return false;
        }

        self.state.pressed = true;
        true
    }

    pub fn handle_mouse_up(&mut self) -> bool {
        if !self.state.pressed {
            return false;
        }

        self.state.pressed = false;
        true
    }

    pub fn is_pressed(&self) -> bool {
        self.state.pressed
    }
}

#[cfg(test)]
mod tests {
    use gpui::{AppContext as _, TestAppContext};

    use super::*;

    struct Host(Option<ControlInteraction>);

    #[test]
    fn a_disabled_control_never_shows_hover_but_can_lose_it() {
        let cx = TestAppContext::single();
        let host = cx.update(|cx| cx.new(|cx| Host(Some(ControlInteraction::new(true, cx)))));
        cx.update(|cx| {
            host.update(cx, |host, _| {
                let interaction = host.0.as_mut().unwrap();
                assert!(interaction.handle_hover(true));
                assert!(interaction.hovered());

                interaction.set_enabled(false);
                assert!(!interaction.hovered(), "disabling clears hover");
                assert!(!interaction.handle_hover(true), "hover is ignored while disabled");
                assert!(!interaction.hovered());

                interaction.set_enabled(true);
                assert!(interaction.handle_hover(true), "hover returns once enabled");
            })
        });
    }
}
