//! One dismissal policy for every popup (#433): which of an outside click,
//! focus leaving the popup, and Escape close it.

/// What closes a popup.
///
/// Build one from a preset ([`PopupDismiss::LIGHT`] and friends) and adjust it
/// with the `with_*` methods. Item selection is not part of the policy: a menu
/// that closes when an item is chosen does so itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub struct PopupDismiss {
    /// A mouse-down outside the popup. Clicks on the popup's own trigger are
    /// the trigger's to handle, so they never count as outside.
    pub click_outside: bool,
    /// Keyboard focus leaving the popup while its window is active.
    pub focus_loss: bool,
    /// The Escape key.
    pub escape: bool,
}

/// Why a popup is being asked to close.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DismissReason {
    ClickOutside,
    FocusLoss,
    Escape,
}

impl PopupDismiss {
    /// Nothing closes it; the app closes it.
    pub const NEVER: Self = Self { click_outside: false, focus_loss: false, escape: false };
    /// Only Escape closes it.
    pub const ESCAPE: Self = Self { click_outside: false, focus_loss: false, escape: true };
    /// An outside click or Escape closes it.
    pub const CLICK_OUTSIDE: Self = Self { click_outside: true, focus_loss: false, escape: true };
    /// Focus loss or Escape closes it.
    pub const FOCUS_LOSS: Self = Self { click_outside: false, focus_loss: true, escape: true };
    /// Light dismiss: an outside click, focus loss or Escape closes it.
    pub const LIGHT: Self = Self { click_outside: true, focus_loss: true, escape: true };

    pub const fn with_click_outside(mut self, enabled: bool) -> Self {
        self.click_outside = enabled;
        self
    }

    pub const fn with_focus_loss(mut self, enabled: bool) -> Self {
        self.focus_loss = enabled;
        self
    }

    pub const fn with_escape(mut self, enabled: bool) -> Self {
        self.escape = enabled;
        self
    }

    /// Whether `reason` closes the popup under this policy.
    pub const fn allows(self, reason: DismissReason) -> bool {
        match reason {
            DismissReason::ClickOutside => self.click_outside,
            DismissReason::FocusLoss => self.focus_loss,
            DismissReason::Escape => self.escape,
        }
    }
}

impl Default for PopupDismiss {
    fn default() -> Self {
        Self::LIGHT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_allow_their_reasons() {
        use DismissReason::*;
        let table = [
            (PopupDismiss::NEVER, [false, false, false]),
            (PopupDismiss::ESCAPE, [false, false, true]),
            (PopupDismiss::CLICK_OUTSIDE, [true, false, true]),
            (PopupDismiss::FOCUS_LOSS, [false, true, true]),
            (PopupDismiss::LIGHT, [true, true, true]),
        ];
        for (policy, expected) in table {
            assert_eq!([policy.allows(ClickOutside), policy.allows(FocusLoss), policy.allows(Escape)], expected);
        }
    }

    #[test]
    fn with_methods_adjust_one_reason() {
        let policy = PopupDismiss::LIGHT.with_escape(false).with_focus_loss(false);
        assert_eq!(policy, PopupDismiss::CLICK_OUTSIDE.with_escape(false));
        assert_eq!(PopupDismiss::default(), PopupDismiss::LIGHT);
    }
}
