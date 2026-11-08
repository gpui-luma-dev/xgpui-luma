//! Studio-only presentation and event logging for the independent examples.

use std::{fmt::Debug, sync::Arc};

use gpui::{App, Context, Div, Entity, FontWeight, IntoElement, RenderOnce, Stateful, Window, div, prelude::*, px};
use xgpui_luma::controls::listbox::ListBoxEvent;
use xgpui_luma::infra::icon::lucide_icon;
use xgpui_luma::{hstack, vstack};
use xgpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextSize};
use lucide_svg_static::Icon as LucideIcon;

use super::super::event_stream::ControlEventStream;

/// Always occupies the same space so toggling selection does not move the label.
#[derive(IntoElement)]
pub(super) struct SelectionMark {
    pub selected: bool,
}

impl RenderOnce for SelectionMark {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .size(px(16.0))
            .flex_shrink_0()
            .opacity(if self.selected { 1.0 } else { 0.0 })
            .child(lucide_icon(LucideIcon::Check, window.text_style().color, 16.0))
    }
}

pub(super) struct ExamplePresentation {
    pub look: Arc<ShadcnLook>,
    title: &'static str,
    event_stream: Entity<ControlEventStream>,
}

impl ExamplePresentation {
    pub fn new(look: Arc<ShadcnLook>, title: &'static str, event_stream: Entity<ControlEventStream>) -> Self {
        Self { look, title, event_stream }
    }

    pub fn sync_look<M: 'static>(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<M>) {
        self.look = look;
        cx.notify();
    }

    pub fn record<K: Debug, M: 'static>(&self, events: &[ListBoxEvent<K>], cx: &mut Context<M>) {
        for event in events {
            self.record_message(&format!("ListBoxEvent::{event:?}"), cx);
        }
    }

    /// Record a host interaction alongside SDK state-change events.
    pub fn record_message<M: 'static>(&self, message: &str, cx: &mut Context<M>) {
        let message = format!("{} · {message}", self.title);
        self.event_stream.update(cx, |stream, cx| stream.append_line(&message, cx));
    }

    pub fn section(&self, selected_count: usize, surface: Stateful<Div>) -> Div {
        let chrome = self.look.chrome();
        let title_style = self.look.typography_scale(ShadcnTextSize::Sm);
        let body_style = self.look.typography_scale(ShadcnTextSize::Xs);
        vstack! { gap=8.0;
            hstack! { justify=between;
                div().typography_style(title_style).font_weight(FontWeight::SEMIBOLD)
                    .text_color(chrome.title_text).child(self.title),
                div().typography_style(body_style).text_color(chrome.muted_text)
                    .child(format!("Selected: {selected_count}")),
            },
            surface,
        }
        .w_full()
        .min_w(px(0.0))
    }
}
