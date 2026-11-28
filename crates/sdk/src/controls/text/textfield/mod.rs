mod control;
mod model;
mod state;
mod template;
pub(crate) mod theme;

pub use control::{TextFieldDrag, TextFieldEvent};
#[allow(deprecated)]
pub use model::TextFieldLookOverride;
pub use model::{TextFieldBuilder, TextFieldModel, TextFieldRenderModel, Validator};
pub use state::TextFieldState;
pub use template::{
    TextFieldClickHandler, TextFieldHoverHandler, TextFieldKeyDownHandler, TextFieldMouseDownHandler,
    TextFieldMouseMoveHandler, TextFieldMouseUpHandler, TextFieldTemplate, TextFieldTemplateHandlers,
    TextFieldDragMoveHandler, ThemedTextFieldTemplate, default_textfield_template,
};
#[allow(deprecated)]
pub use theme::{TextFieldLook, compose_textfield_look};
pub use theme::{TextFieldPalette, TextFieldTheme, apply_control_typography, default_textfield_theme};

use gpui::{Entity, SharedString};

use self::control::TextFieldControl;

pub type TextField = Entity<TextFieldControl>;

pub fn new(id: impl Into<SharedString>) -> TextFieldBuilder {
    TextFieldBuilder::new(id)
}

mod callback;
pub use callback::TextFieldEventExt;
