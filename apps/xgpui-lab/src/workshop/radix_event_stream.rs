use std::sync::Arc;

use gpui::{Context, Entity, Render, Window};
use crate::theme::{Look};

use super::event_log_view::{EventLogView, EventLogViewLookExt, radix_event_log_theme};

pub struct RadixEventStream {
    look: Arc<Look>,
    log_id: &'static str,
    intro: &'static str,
    event_log: Entity<EventLogView>,
    next_index: usize,
}

impl RadixEventStream {
    pub fn new(cx: &mut Context<Self>, look: Arc<Look>, log_id: &'static str, intro: &'static str) -> Self {
        let event_log = look
            .event_log_view(log_id)
            .placeholder("Event history will appear here…")
            .full_width(true)
            .rows(6)
            .font_family(crate::theme::MONO_FAMILY)
            .spawn(cx);

        Self { look, log_id, intro, event_log, next_index: 1 }
    }

    pub fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.event_log.update(cx, |log, cx| {
            log.set_theme(radix_event_log_theme(look), cx);
            log.set_font_family(crate::theme::MONO_FAMILY, cx);
        });
        cx.notify();
    }

    pub fn append_line(&mut self, line: &str, cx: &mut Context<Self>) {
        let line = format!("[{}] {line}", self.next_index);
        self.next_index += 1;
        self.event_log.update(cx, |log, cx| log.append_line(&line, cx));
        cx.notify();
    }
}

impl Render for RadixEventStream {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        super::sample_layout::card(&self.look, self.log_id, "Events", self.intro, 1120., self.event_log.clone())
    }
}
