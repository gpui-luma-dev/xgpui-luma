//! Lab page: Parley text fields over left-to-right, right-to-left, mixed and long text.
use std::sync::Arc;

use gpui::{Context, Entity, Font, FontFallbacks, Render, Subscription, Window, div, prelude::*, px};
use xgpui_luma_look_radix::{self as radix, Look};

use super::{ParleyTextField, ParleyTextFieldEvent};
use crate::theme::ColorVizLookExt;

/// (label, initial value, placeholder)
pub(crate) const FIELDS: [(&str, &str, &str); 5] = [
    ("Empty", "", "Type here; try an IME, emoji or Hebrew"),
    ("Latin, combining marks and emoji", "office café e\u{301} 👍🏽 🇮🇹 done", ""),
    ("Hebrew (right-to-left base)", "שלום עולם 123 office", ""),
    ("Arabic (right-to-left base)", "مرحبا بالعالم 456", ""),
    (
        "Long text (scrolls to follow the caret)",
        "The quick brown fox jumps over the lazy dog, then keeps going past the end of the field so the caret has to scroll.",
        "",
    ),
];

pub(crate) fn font() -> Font {
    let mut font = gpui::font("IBM Plex Sans");
    font.fallbacks = Some(FontFallbacks::from_fonts(vec!["Noto Sans Hebrew".into(), "Noto Sans Arabic".into()]));
    font
}

pub struct TextFieldPage {
    look: Arc<Look>,
    pub(crate) fields: Vec<Entity<ParleyTextField>>,
    last_event: String,
    _subscriptions: Vec<Subscription>,
}

impl TextFieldPage {
    pub fn new(look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let theme = radix::textfield_theme_with(&look, radix::TextFieldVariant::Surface);
        let fields: Vec<_> = FIELDS
            .iter()
            .enumerate()
            .map(|(index, (_, value, placeholder))| {
                cx.new(|cx| {
                    ParleyTextField::new(format!("parley-text-field-{index}"), theme.clone(), font(), cx)
                        .with_value(value)
                        .with_placeholder(*placeholder)
                        .with_inverted_colors()
                })
            })
            .collect();
        let subscriptions = fields
            .iter()
            .enumerate()
            .map(|(index, field)| {
                cx.subscribe(field, move |this: &mut Self, _, event: &ParleyTextFieldEvent, cx| {
                    let label = FIELDS[index].0;
                    this.last_event = match event {
                        ParleyTextFieldEvent::Change(value) => format!("{label}: changed to {value:?}"),
                        ParleyTextFieldEvent::Submit(value) => format!("{label}: submitted {value:?}"),
                    };
                    cx.notify();
                })
            })
            .collect();
        Self { look, fields, last_event: "No events yet".into(), _subscriptions: subscriptions }
    }
}

impl Render for TextFieldPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let rows = self.fields.iter().zip(FIELDS).map(|(field, (label, _, _))| {
            let state = field.read(cx);
            let (anchor, caret) = state.selection();
            let status = match state.layout_error() {
                Some(error) => format!("Layout error: {error}"),
                None => format!(
                    "{} bytes · anchor {} {:?} · caret {} {:?} · scroll {:.0}px",
                    state.value().len(),
                    anchor.byte_index,
                    anchor.affinity,
                    caret.byte_index,
                    caret.affinity,
                    f32::from(state.scroll()),
                ),
            };
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(div().text_size(px(13.0)).child(label))
                .child(div().w(px(360.0)).child(field.clone()))
                .child(div().text_size(px(11.0)).text_color(chrome.muted_text).child(status))
        });
        div()
            .id("parley-text-field-page")
            .size_full()
            .overflow_y_scroll()
            .p_4()
            .flex()
            .flex_col()
            .gap_4()
            .font_family("IBM Plex Sans")
            .text_color(chrome.body_text)
            .child(div().text_size(px(22.0)).child("Parley text field"))
            .child(div().text_size(px(13.0)).child(
                "Left/Right move visually; Alt moves by word; Cmd/Ctrl-Left/Right and Home/End go to the ends. \
                 Backspace removes whole characters. Shift extends; double-click selects a word, triple-click all. \
                 Enter submits.",
            ))
            .children(rows)
            .child(div().text_size(px(12.0)).text_color(chrome.muted_text).child(self.last_event.clone()))
    }
}
