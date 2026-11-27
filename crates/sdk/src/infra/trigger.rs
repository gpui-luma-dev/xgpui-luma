//! When a popup's trigger opens and closes it (#433): one rule for every
//! trigger, instead of a click handler written per control.

use gpui::ClickEvent;

/// What on the trigger opens (and closes) its popup.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub enum TriggerOpenOn {
    /// A completed click: press and release on the trigger.
    #[default]
    Click,
    /// The mouse button going down on the trigger.
    Press,
    /// Nothing on the trigger; the app opens and closes the popup itself.
    Never,
}

/// What a trigger event does to its popup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TriggerToggle {
    Open,
    Close,
}

impl TriggerToggle {
    fn from_open(open: bool) -> Self {
        if open { Self::Close } else { Self::Open }
    }
}

/// The toggle for a mouse-down on an `enabled` trigger whose popup is `open`.
pub(crate) fn toggle_on_press(open_on: TriggerOpenOn, enabled: bool, open: bool) -> Option<TriggerToggle> {
    (enabled && open_on == TriggerOpenOn::Press).then(|| TriggerToggle::from_open(open))
}

/// The toggle for a click on an `enabled` trigger whose popup is `open`.
/// Keyboard clicks never toggle here: Enter and Space reach the control as
/// its activate action, which opens the popup with an item active.
pub(crate) fn toggle_on_click(
    open_on: TriggerOpenOn,
    event: &ClickEvent,
    enabled: bool,
    open: bool,
) -> Option<TriggerToggle> {
    (enabled && open_on == TriggerOpenOn::Click && !event.is_keyboard()).then(|| TriggerToggle::from_open(open))
}

#[cfg(test)]
mod tests {
    use gpui::{KeyboardClickEvent, MouseClickEvent};

    use super::*;

    fn mouse_click() -> ClickEvent {
        ClickEvent::Mouse(MouseClickEvent::default())
    }

    #[test]
    fn click_toggles_only_when_opening_on_click() {
        assert_eq!(toggle_on_click(TriggerOpenOn::Click, &mouse_click(), true, false), Some(TriggerToggle::Open));
        assert_eq!(toggle_on_click(TriggerOpenOn::Click, &mouse_click(), true, true), Some(TriggerToggle::Close));
        assert_eq!(toggle_on_click(TriggerOpenOn::Press, &mouse_click(), true, false), None);
        assert_eq!(toggle_on_click(TriggerOpenOn::Never, &mouse_click(), true, false), None);
    }

    #[test]
    fn press_toggles_only_when_opening_on_press() {
        assert_eq!(toggle_on_press(TriggerOpenOn::Press, true, false), Some(TriggerToggle::Open));
        assert_eq!(toggle_on_press(TriggerOpenOn::Press, true, true), Some(TriggerToggle::Close));
        assert_eq!(toggle_on_press(TriggerOpenOn::Click, true, false), None);
        assert_eq!(toggle_on_press(TriggerOpenOn::Never, true, false), None);
    }

    #[test]
    fn disabled_triggers_and_keyboard_clicks_never_toggle() {
        assert_eq!(toggle_on_click(TriggerOpenOn::Click, &mouse_click(), false, false), None);
        assert_eq!(toggle_on_press(TriggerOpenOn::Press, false, false), None);
        let keyboard = ClickEvent::Keyboard(KeyboardClickEvent::default());
        assert_eq!(toggle_on_click(TriggerOpenOn::Click, &keyboard, true, false), None);
    }
}
