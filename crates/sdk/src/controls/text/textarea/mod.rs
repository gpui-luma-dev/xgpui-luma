mod control;
mod model;
mod state;
mod template;
pub(crate) mod theme;

pub use control::{TextArea, TextAreaDrag, TextAreaEvent};
pub use model::{TextAreaBuilder, TextAreaLineMetric, TextAreaModel, TextAreaRenderModel, Validator};
pub use state::TextAreaState;
pub use template::{
    TextAreaClickHandler, TextAreaHoverHandler, TextAreaKeyDownHandler, TextAreaMouseDownHandler,
    TextAreaMouseMoveHandler, TextAreaMouseUpHandler, TextAreaDragMoveHandler, TextAreaTemplate,
    TextAreaTemplateHandlers, ThemedTextAreaTemplate, default_textarea_template,
};
pub use theme::{TextAreaLook, TextAreaPalette, TextAreaTheme, compose_textarea_look, default_textarea_theme};

mod callback;
pub use callback::TextAreaEventExt;
