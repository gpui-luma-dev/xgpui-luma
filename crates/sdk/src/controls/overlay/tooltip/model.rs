use std::{sync::Arc, time::Duration};
use gpui::{SharedString, AnyElement};
use super::{TooltipRenderModel, TooltipTemplate, TooltipTheme, bubble};

/// Application-wide tooltip preference. Absent settings enable help by default.
#[derive(Clone, Copy)]
pub struct TooltipSettings {
    pub enabled: bool,
}
impl gpui::Global for TooltipSettings {}

/// Preferred side; the opposite side is used when there is insufficient room.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipPlacement {
    #[default]
    Above,
    Below,
    /// Above the target, aligned to its logical start edge.
    AboveStart,
    /// Above the target, aligned to its logical end edge.
    AboveEnd,
    /// Below the target, aligned to its logical start edge.
    BelowStart,
    /// Below the target, aligned to its logical end edge.
    BelowEnd,
}

/// Escape can suppress help until the target is left, or disable this attachment permanently.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipDismissal {
    #[default]
    UntilLeave,
    Permanently,
}

/// Observable presentation lifecycle, independent of the owner's input events.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TooltipEvent {
    Shown,
    Hidden,
}

type TextProvider = Arc<dyn Fn(&gpui::App) -> SharedString + Send + Sync>;

/// Configuration is independent of the target control and its button/slider template.
#[derive(Clone)]
pub struct Tooltip {
    pub(super) text: SharedString,
    pub(super) text_provider: Option<TextProvider>,
    pub(super) shortcut: Option<SharedString>,
    pub(super) delay: Duration,
    pub(super) hide_delay: Duration,
    pub(super) duration: Option<Duration>,
    pub(super) placement: TooltipPlacement,
    pub(super) dismissal: TooltipDismissal,
    pub(super) on_event: Option<Arc<dyn Fn(TooltipEvent) + Send + Sync>>,
    pub(super) theme: Option<Arc<dyn TooltipTheme>>,
    pub(super) once: bool,
    pub(super) enabled: bool,
    pub(super) always_enabled: bool,
    pub(super) template: TooltipTemplate,
}

impl Tooltip {
    /// Create ordinary help using the owner's theme and default timing.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            text_provider: None,
            shortcut: None,
            delay: Duration::from_millis(500),
            hide_delay: Duration::from_millis(180),
            duration: None,
            placement: TooltipPlacement::Above,
            dismissal: TooltipDismissal::UntilLeave,
            on_event: None,
            theme: None,
            once: false,
            enabled: true,
            always_enabled: false,
            template: Arc::new(bubble),
        }
    }

    /// Optional shortcut or hint rendered alongside the help text.
    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Hover show delay. Keyboard focus opens immediately.
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Grace period after hover/focus leaves the target and bubble.
    pub fn hide_delay(mut self, delay: Duration) -> Self {
        self.hide_delay = delay;
        self
    }

    /// Maximum visible time, starting at actual presentation. None keeps help open
    /// while its trigger remains active. Expiry suppresses reopening until leave.
    pub fn duration(mut self, duration: Option<Duration>) -> Self {
        self.duration = duration;
        self
    }

    /// Resolve display and accessible help text from the current app state on render.
    /// Useful for localization without replacing the retained attachment or dismiss history.
    pub fn text_provider(mut self, provider: impl Fn(&gpui::App) -> SharedString + Send + Sync + 'static) -> Self {
        self.text_provider = Some(Arc::new(provider));
        self
    }

    /// Preferred side, with automatic flipping and viewport clamping.
    pub fn placement(mut self, placement: TooltipPlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Escape suppression policy. Other dismissal triggers suppress until leave.
    pub fn dismissal(mut self, dismissal: TooltipDismissal) -> Self {
        self.dismissal = dismissal;
        self
    }

    /// Called once per actual presentation and once when that presentation ends.
    pub fn on_event(mut self, callback: impl Fn(TooltipEvent) + Send + Sync + 'static) -> Self {
        let previous = self.on_event.take();
        self.on_event = Some(Arc::new(move |event| {
            if let Some(previous) = &previous {
                previous(event);
            }
            callback(event);
        }));
        self
    }

    /// Append a callback when help becomes visible.
    pub fn on_shown(self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_event(move |event| {
            if event == TooltipEvent::Shown {
                callback();
            }
        })
    }

    /// Append a callback when visible help is hidden.
    pub fn on_hidden(self, callback: impl Fn() + Send + Sync + 'static) -> Self {
        self.on_event(move |event| {
            if event == TooltipEvent::Hidden {
                callback();
            }
        })
    }

    /// Override the look bound to the owner. Ordinary help uses the owner's look.
    pub fn theme(mut self, theme: Arc<dyn TooltipTheme>) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Show once per attachment lifetime, counting only an actually rendered bubble.
    pub fn show_once(mut self) -> Self {
        self.once = true;
        self
    }

    /// Disabled configurations do not activate or alter the owner's input behavior.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Keep this attachment available regardless of application-wide settings.
    pub fn always_enabled(mut self) -> Self {
        self.always_enabled = true;
        self
    }

    /// Presentation override; lifecycle, anchoring, and theme resolution stay in the SDK.
    pub fn template(mut self, template: impl Fn(&TooltipRenderModel) -> AnyElement + Send + Sync + 'static) -> Self {
        self.template = Arc::new(template);
        self
    }
}

#[cfg(test)]
mod callback_tests {
    use super::*;
    use std::sync::{Mutex, Arc};

    #[test]
    fn lifecycle_callbacks_append_and_filter_actual_presentations() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let all = log.clone();
        let shown = log.clone();
        let hidden = log.clone();
        let tooltip = Tooltip::new("Help")
            .on_event(move |_| all.lock().unwrap().push("all"))
            .on_shown(move || shown.lock().unwrap().push("shown"))
            .on_hidden(move || hidden.lock().unwrap().push("hidden"));
        let handler = tooltip.on_event.as_ref().unwrap();
        handler(TooltipEvent::Shown);
        handler(TooltipEvent::Hidden);
        assert_eq!(*log.lock().unwrap(), ["all", "shown", "all", "hidden"]);
    }
}
