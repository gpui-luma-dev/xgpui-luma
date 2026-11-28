mod control;
mod model;
mod state;
mod template;
pub(crate) mod theme;
mod values;

pub use control::{TextArea, TextAreaDrag, TextAreaEvent};
pub use model::{TextAreaBuilder, TextAreaLineMetric, TextAreaModel, TextAreaRenderModel, Validator};
pub use state::TextAreaState;
pub use template::{
    TextAreaClickHandler, TextAreaHoverHandler, TextAreaKeyDownHandler, TextAreaMouseDownHandler,
    TextAreaMouseMoveHandler, TextAreaMouseUpHandler, TextAreaDragMoveHandler, TextAreaTemplate,
    TextAreaTemplateHandlers, ThemedTextAreaTemplate, default_textarea_template,
};
#[allow(deprecated)]
pub use theme::{TextAreaLook, compose_textarea_look};
pub use theme::{TextAreaPalette, TextAreaTheme, default_textarea_theme};

mod callback;
pub use callback::TextAreaEventExt;
