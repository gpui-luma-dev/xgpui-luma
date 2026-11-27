//! Shared activation core for button-family controls.
//!
//! [`CommandCore`] owns focus/hover/enabled interaction and emits [`CommandEvent`]
//! (re-exported as [`super::ButtonEvent`] from the button control). It does not
//! render UI — presentations live in [`super::template`].

use gpui::{ClickEvent, Context, EventEmitter, MouseDownEvent, MouseUpEvent, Window};

use crate::controls::button_family::ButtonInteractionState;
use crate::infra::interaction::ControlInteraction;
use crate::key_handling::ActivateControl;

/// Semantic command event emitted by command-like controls.
///
/// This is intentionally presentation-agnostic: it models user activation
/// and observable command interaction-state transitions.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CommandEvent {
    Click,
    FocusChanged { focused: bool },
    EnabledChanged { enabled: bool },
    HoverChanged { hovered: bool },
}

impl CommandEvent {
    pub fn is_click(&self) -> bool {
        matches!(self, Self::Click)
    }
}

/// Shared behavior core for command-like controls.
///
/// `CommandCore` owns interaction state and input/event semantics, but does not
/// render any UI. Concrete controls (for example text-command and icon-command
/// presentations) can reuse this behavior and provide their own templates.
pub struct CommandCore {
    interaction: ControlInteraction,
    focus_report: crate::infra::focus_report::FocusReport,
}

impl CommandCore {
    /// Create a new command behavior core with explicit tab-stop participation.
    pub fn new_with_tab_stop<T>(enabled: bool, tab_stop: bool, cx: &mut Context<T>) -> Self {
        Self {
            interaction: ControlInteraction::new_with_tab_stop(enabled, tab_stop, cx),
            focus_report: Default::default(),
        }
    }

    /// Propagate enabled changes into interaction state.
    pub fn set_enabled<T>(&mut self, enabled: bool, cx: &mut Context<T>) -> bool
    where
        T: EventEmitter<CommandEvent>,
    {
        let old_enabled = self.interaction.enabled();
        let old_hovered = self.interaction.hovered();
        self.interaction.set_enabled(enabled);

        let mut changed = old_enabled != enabled;
        if changed {
            cx.emit(CommandEvent::EnabledChanged { enabled });
        }

        if old_hovered != self.interaction.hovered() {
            changed = true;
            cx.emit(CommandEvent::HoverChanged { hovered: self.interaction.hovered() });
        }

        if !enabled && self.focus_report.change(false, false) {
            changed = true;
            cx.emit(CommandEvent::FocusChanged { focused: false });
        }

        changed
    }

    /// Compute render-time interaction state for templates/presenters.
    pub fn render_state(&self, enabled: bool, window: &Window) -> ButtonInteractionState {
        self.interaction.render_state(enabled, window)
    }

    /// Access the focus handle used by this command behavior core.
    pub fn focus_handle(&self) -> &gpui::FocusHandle {
        self.interaction.focus_handle()
    }

    /// Emit a semantic click if enabled.
    pub fn activate<T>(&mut self, enabled: bool, cx: &mut Context<T>) -> bool
    where
        T: EventEmitter<CommandEvent>,
    {
        if !enabled {
            return false;
        }

        cx.emit(CommandEvent::Click);
        true
    }

    /// Handle pointer click activation (ignores keyboard-generated click events).
    pub fn handle_click<T>(&mut self, enabled: bool, event: &ClickEvent, cx: &mut Context<T>) -> bool
    where
        T: EventEmitter<CommandEvent>,
    {
        if event.is_keyboard() {
            return false;
        }

        if !enabled {
            return false;
        }

        cx.stop_propagation();
        self.activate(enabled, cx)
    }

    /// Handle keyboard action activation.
    pub fn handle_activate_control<T>(&mut self, enabled: bool, _event: &ActivateControl, cx: &mut Context<T>) -> bool
    where
        T: EventEmitter<CommandEvent>,
    {
        self.activate(enabled, cx)
    }

    /// Update hover state. Returns `true` when caller should notify.
    pub fn handle_hover<T>(&mut self, enabled: bool, hovered: bool, cx: &mut Context<T>) -> bool
    where
        T: EventEmitter<CommandEvent>,
    {
        let hovered = enabled && hovered;
        if !self.interaction.handle_hover(hovered) {
            return false;
        }

        cx.emit(CommandEvent::HoverChanged { hovered });
        true
    }

    /// Update focus state. Returns `true` when caller should notify.
    pub fn handle_focus_changed<T>(&mut self, enabled: bool, focused: bool, cx: &mut Context<T>) -> bool
    where
        T: EventEmitter<CommandEvent>,
    {
        let focused = enabled && focused;
        if !self.focus_report.change(focused, enabled) {
            return false;
        }

        cx.emit(CommandEvent::FocusChanged { focused });
        true
    }

    /// Update pressed state on mouse down. Returns `true` when caller should notify.
    pub fn handle_mouse_down<T>(
        &mut self,
        enabled: bool,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<T>,
    ) -> bool {
        if !enabled {
            return false;
        }

        cx.stop_propagation();
        self.interaction.handle_mouse_down(enabled, window, cx)
    }

    /// Update pressed state on mouse up. Returns `true` when caller should notify.
    pub fn handle_mouse_up(&mut self, _event: &MouseUpEvent) -> bool {
        self.interaction.handle_mouse_up()
    }
}
