use gpui::SharedString;
use crate::controls::textfield::{self, TextField, TextFieldBuilder, TextFieldEvent};

pub type TextSelection = TextField;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum TextSelectionEvent {
    Change { value: String },
    Submit { value: String },
    FocusEnter,
    FocusLeave,
}

pub fn new(id: impl Into<SharedString>) -> TextFieldBuilder {
    textfield::new(id)
}

pub fn map_event(event: &TextFieldEvent) -> Option<TextSelectionEvent> {
    match event {
        TextFieldEvent::Change { value } => Some(TextSelectionEvent::Change { value: value.clone() }),
        TextFieldEvent::Submit { value } => Some(TextSelectionEvent::Submit { value: value.clone() }),
        TextFieldEvent::FocusChanged { focused: true } => Some(TextSelectionEvent::FocusEnter),
        TextFieldEvent::FocusChanged { focused: false } => Some(TextSelectionEvent::FocusLeave),
        TextFieldEvent::EnabledChanged { .. } | TextFieldEvent::HoverChanged { .. } => None,
    }
}
