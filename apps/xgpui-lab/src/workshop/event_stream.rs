use std::sync::Arc;
use std::time::Instant;

use gpui::{Context, Entity, Render, Window};
use crate::theme::{Look};

use super::event_log_view::{EventLogView, EventLogViewLookExt, radix_event_log_theme};

pub struct ControlEventStream {
    look: Arc<Look>,
    log_id: &'static str,
    intro: &'static str,
    event_log: Entity<EventLogView>,
    next_index: usize,
    framed: bool,
    /// Whether each line shows its time since the first event (see [`ControlEventStream::with_times`]).
    times: bool,
    /// When the first event was logged.
    started: Option<Instant>,
}

impl ControlEventStream {
    pub fn new(cx: &mut Context<Self>, look: Arc<Look>, log_id: &'static str, intro: &'static str) -> Self {
        let event_log = look
            .event_log_view(log_id)
            .placeholder("Event history will appear here…")
            .full_width(true)
            .rows(6)
            .font_family(crate::theme::MONO_FAMILY)
            .spawn(cx);

        Self { look, log_id, intro, event_log, next_index: 1, framed: true, times: false, started: None }
    }

    pub(super) fn without_card(mut self) -> Self {
        self.framed = false;
        self
    }

    /// Start each line with its time in seconds since the first event, e.g. `[3] 0.712s Change { … }`.
    pub(super) fn with_times(mut self) -> Self {
        self.times = true;
        self
    }

    pub fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.event_log.update(cx, |log, cx| {
            log.set_theme(radix_event_log_theme(look), cx);
            log.set_font_family(crate::theme::MONO_FAMILY, cx);
        });
        cx.notify();
    }

    /// How many events have been logged.
    pub(super) fn count(&self) -> usize {
        self.next_index - 1
    }

    /// Everything logged, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn text(&self, cx: &gpui::App) -> String {
        self.event_log.read(cx).text().to_string()
    }

    pub fn append_line(&mut self, line: &str, cx: &mut Context<Self>) {
        let line = if self.times {
            let seconds = self.started.get_or_insert_with(Instant::now).elapsed().as_secs_f64();
            format!("[{}] {} {line}", self.next_index, event_time(seconds))
        } else {
            format!("[{}] {line}", self.next_index)
        };
        self.next_index += 1;
        self.event_log.update(cx, |log, cx| log.append_line(&line, cx));
        cx.notify();
    }
}

impl Render for ControlEventStream {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let layout = if self.framed {
            super::sample_layout::card
        } else {
            super::sample_layout::section
        };
        layout(&self.look, self.log_id, "Events", self.intro, 1120., self.event_log.clone())
    }
}

/// An event's time since the first event, to the millisecond: `0.000s`, `12.345s`.
fn event_time(seconds: f64) -> String {
    format!("{seconds:.3}s")
}

#[cfg(test)]
mod tests {
    use super::event_time;

    #[test]
    fn event_times_are_seconds_to_the_millisecond() {
        assert_eq!(event_time(0.0), "0.000s");
        assert_eq!(event_time(0.7123), "0.712s");
        assert_eq!(event_time(12.3456), "12.346s");
    }
}
