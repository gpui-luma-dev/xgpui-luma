mod control;
mod model;
mod state;
mod template;
pub(crate) mod theme;

pub use control::{TextFieldDrag, TextFieldEvent};
pub use model::{TextFieldLookOverride, TextFieldBuilder, TextFieldModel, TextFieldRenderModel, Validator};
pub use state::TextFieldState;
pub use template::{
    TextFieldClickHandler, TextFieldHoverHandler, TextFieldKeyDownHandler, TextFieldMouseDownHandler,
    TextFieldMouseMoveHandler, TextFieldMouseUpHandler, TextFieldTemplate, TextFieldTemplateHandlers,
    TextFieldDragMoveHandler, ThemedTextFieldTemplate, default_textfield_template,
};
pub use theme::{
    TextFieldLook, TextFieldPalette, TextFieldTheme, apply_control_typography, compose_textfield_look,
    default_textfield_theme,
};

use gpui::{Entity, SharedString};

use self::control::TextFieldControl;

pub type TextField = Entity<TextFieldControl>;

pub fn new(id: impl Into<SharedString>) -> TextFieldBuilder {
    TextFieldBuilder::new(id)
}

mod callback;
pub use callback::TextFieldEventExt;
