//! Lab page: Parley text fields and text areas over left-to-right, right-to-left,
//! mixed and long text.
use std::sync::Arc;

use gpui::{Context, Entity, Font, FontFallbacks, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::{button as radix_button, textfield::TextFieldLook};
use xgpui_luma::prelude::HasPresenter;
use xgpui_luma_look_radix::{self as radix, Look};

use super::{ParleyTextField, ParleyTextFieldEvent};
use crate::theme::ColorVizLookExt;

/// Index of the field that starts disabled.
const DISABLED: usize = 1;

/// (label, initial value, placeholder, text area rows)
pub(crate) const FIELDS: [(&str, &str, &str, Option<usize>); 9] = [
    ("Empty", "", "Type here; try an IME, emoji or Hebrew", None),
    ("Disabled (toggle below)", "Not editable while disabled", "", None),
    ("Latin, combining marks and emoji", "office café e\u{301} 👍🏽 🇮🇹 done", "", None),
    ("Hebrew (right-to-left base)", "שלום עולם 123 office", "", None),
    ("Arabic (right-to-left base)", "مرحبا بالعالم 456", "", None),
    (
        "Long text (scrolls to follow the caret)",
        "The quick brown fox jumps over the lazy dog, then keeps going past the end of the field so the caret has to scroll.",
        "",
        None,
    ),
    ("Text area: empty", "", "Several lines; Enter adds a line break", Some(4)),
    (
        "Text area: wrapping and line breaks",
        "The quick brown fox jumps over the lazy dog. Long lines wrap at the area's width, \
         and Up/Down keep the caret's column.\nA hard line break starts this line.\n\n\
         An empty line came before this one; keep typing to scroll the area.",
        "",
        Some(4),
    ),
    (
        "Text area: Hebrew, Arabic and English",
        "שלום עולם, this line mixes directions 123.\nمرحبا بالعالم and some English words that wrap across lines.\n\
         office café 👍🏽",
        "",
        Some(4),
    ),
];

/// Testing aid: light text on the look's text color, for contrast while testing.
fn inverted(mut look: TextFieldLook) -> TextFieldLook {
    std::mem::swap(&mut look.background, &mut look.foreground);
    look.placeholder = look.foreground.opacity(0.6);
    look.caret = look.foreground;
    look
}

pub(crate) fn font() -> Font {
    let mut font = gpui::font("IBM Plex Sans");
    font.fallbacks = Some(FontFallbacks::from_fonts(vec!["Noto Sans Hebrew".into(), "Noto Sans Arabic".into()]));
    font
}

pub struct TextFieldPage {
    look: Arc<Look>,
    pub(crate) fields: Vec<Entity<ParleyTextField>>,
    /// Toggles the "Disabled" field.
    toggle: Entity<radix_button::Button>,
    last_event: String,
}

impl TextFieldPage {
    pub fn new(look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let fields: Vec<_> = FIELDS
            .iter()
            .enumerate()
            .map(|(index, (label, value, placeholder, rows))| {
                let label = *label;
                let builder = ParleyTextField::new(format!("parley-text-field-{index}"))
                    .surface()
                    .font(font())
                    .value(*value)
                    .placeholder(*placeholder)
                    .enabled(index != DISABLED)
                    .look_override(inverted)
                    .on_event(cx, move |this: &mut Self, event, cx| {
                        this.last_event = match event {
                            ParleyTextFieldEvent::Change { value } => format!("{label}: changed to {value:?}"),
                            ParleyTextFieldEvent::Submit { value } => format!("{label}: submitted {value:?}"),
                            ParleyTextFieldEvent::FocusChanged { focused } => format!("{label}: focused {focused}"),
                            ParleyTextFieldEvent::EnabledChanged { enabled } => format!("{label}: enabled {enabled}"),
                        };
                        cx.notify();
                    });
                match rows {
                    Some(rows) => builder.rows(*rows),
                    None => builder,
                }
                .spawn(cx)
            })
            .collect();
        let toggle = radix::Button::new("parley-text-field-toggle-enabled")
            .look(look.as_ref())
            .outline()
            .label("Enable / disable")
            .on_click(cx, |this: &mut Self, cx| {
                let field = this.fields[DISABLED].clone();
                field.update(cx, |field, cx| field.set_enabled(!field.is_enabled(), cx));
            })
            .spawn(cx);
        Self { look, fields, toggle, last_event: "No events yet".into() }
    }
}

impl Render for TextFieldPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let rows = self.fields.iter().zip(FIELDS).map(|(field, (label, _, _, _))| {
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
            .child(div().text_size(px(22.0)).child("Parley text field and text area"))
            .child(div().text_size(px(13.0)).child(
                "Left/Right move visually; Alt moves by word; Cmd/Ctrl-Left/Right and Home/End go to the line's ends. \
                 Backspace removes whole characters. Shift extends; double-click selects a word, triple-click all. \
                 Fields: Enter submits. Text areas: Enter adds a line break, Up/Down move by line, \
                 Cmd/Ctrl-Up/Down go to the start or end, and the wheel scrolls.",
            ))
            .child(self.toggle.clone())
            .children(rows)
            .child(div().text_size(px(12.0)).text_color(chrome.muted_text).child(self.last_event.clone()))
    }
}
