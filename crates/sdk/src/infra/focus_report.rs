//! One way for a control to report focus changes and to subscribe to its
//! focus handle (#433). Each control used to carry its own copy of both.

use gpui::{Context, EventEmitter, FocusHandle, FocusOutEvent, Subscription, Window};

/// What a control last reported about its focus.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct FocusReport {
    reported: bool,
}

impl FocusReport {
    /// Record `focused` and say whether it should be reported: only a change
    /// is, and a disabled control never reports gaining focus (it still
    /// reports losing it, so no listener is left believing it is focused).
    pub fn change(&mut self, focused: bool, enabled: bool) -> bool {
        if (focused && !enabled) || self.reported == focused {
            return false;
        }
        self.reported = focused;
        true
    }
}

/// A control that emits a `FocusChanged`-style event.
///
/// Implementors supply the event and their [`FocusReport`];
/// [`ReportsFocus::emit_focus_changed`] is the one place the rules live: a
/// change is reported once, and a disabled control never reports gaining
/// focus (it still reports losing it, so a listener is never left believing
/// it is focused).
pub(crate) trait ReportsFocus: EventEmitter<Self::FocusEvent> + Sized + 'static {
    type FocusEvent: 'static;

    fn focus_report(&mut self) -> &mut FocusReport;

    fn focus_event(focused: bool) -> Self::FocusEvent;

    /// Whether the control can take focus now.
    fn focus_enabled(&self) -> bool {
        true
    }

    /// Emit the focus event if `focused` differs from the last report.
    /// Returns whether it did.
    fn emit_focus_changed(&mut self, focused: bool, cx: &mut Context<Self>) -> bool {
        let enabled = self.focus_enabled();
        if !self.focus_report().change(focused, enabled) {
            return false;
        }
        cx.emit(Self::focus_event(focused));
        true
    }
}

/// A control's focus-in and focus-out subscriptions, made once on first render.
#[derive(Default)]
pub(crate) struct FocusSubscriptions(Option<[Subscription; 2]>);

impl FocusSubscriptions {
    /// Subscribe to `handle` itself gaining and losing focus, if not yet subscribed.
    pub fn ensure<T: 'static>(
        &mut self,
        handle: &FocusHandle,
        window: &mut Window,
        cx: &mut Context<T>,
        on_focus: impl FnMut(&mut T, &mut Window, &mut Context<T>) + 'static,
        on_blur: impl FnMut(&mut T, FocusOutEvent, &mut Window, &mut Context<T>) + 'static,
    ) {
        if self.0.is_none() {
            self.0 = Some([cx.on_focus(handle, window, on_focus), cx.on_focus_out(handle, window, on_blur)]);
        }
    }

    /// Like [`FocusSubscriptions::ensure`], but focus moving into any of
    /// `handle`'s descendants counts as gaining focus.
    pub fn ensure_within<T: 'static>(
        &mut self,
        handle: &FocusHandle,
        window: &mut Window,
        cx: &mut Context<T>,
        on_focus: impl FnMut(&mut T, &mut Window, &mut Context<T>) + 'static,
        on_blur: impl FnMut(&mut T, FocusOutEvent, &mut Window, &mut Context<T>) + 'static,
    ) {
        if self.0.is_none() {
            self.0 = Some([cx.on_focus_in(handle, window, on_focus), cx.on_focus_out(handle, window, on_blur)]);
        }
    }

    /// Drop the subscriptions, so the next `ensure` subscribes again (for a
    /// control whose focus handle was replaced).
    pub fn reset(&mut self) {
        self.0 = None;
    }
}

#[cfg(test)]
mod tests {
    use gpui::{AppContext as _, TestAppContext};

    use super::*;

    struct Probe {
        report: FocusReport,
        enabled: bool,
        events: Vec<bool>,
    }

    struct Focused(bool);

    impl EventEmitter<Focused> for Probe {}

    impl ReportsFocus for Probe {
        type FocusEvent = Focused;

        fn focus_report(&mut self) -> &mut FocusReport {
            &mut self.report
        }

        fn focus_event(focused: bool) -> Focused {
            Focused(focused)
        }

        fn focus_enabled(&self) -> bool {
            self.enabled
        }
    }

    #[test]
    fn reports_each_change_once_and_never_focus_while_disabled() {
        let cx = TestAppContext::single();
        let probe =
            cx.update(|cx| cx.new(|_| Probe { report: FocusReport::default(), enabled: true, events: Vec::new() }));
        cx.update(|cx| {
            cx.subscribe(&probe, |probe, event: &Focused, cx| probe.update(cx, |probe, _| probe.events.push(event.0)))
                .detach()
        });
        cx.update(|cx| {
            probe.update(cx, |probe, cx| {
                assert!(probe.emit_focus_changed(true, cx));
                assert!(!probe.emit_focus_changed(true, cx), "a repeat is not reported");
                probe.enabled = false;
                assert!(probe.emit_focus_changed(false, cx), "losing focus is reported while disabled");
                assert!(!probe.emit_focus_changed(true, cx), "gaining focus is not reported while disabled");
            })
        });
        cx.run_until_parked();
        cx.read(|cx| assert_eq!(probe.read(cx).events, [true, false]));
    }
}
