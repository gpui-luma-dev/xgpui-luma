mod styled;

pub use styled::{StyledProgressTheme, resolve_progress};
mod control;
mod direction;
mod model;
mod template;
pub(crate) mod theme;

pub use direction::{ProgressDirection, ProgressOrientation, display_position};
pub use model::{ProgressBuilder, ProgressModel, ProgressRenderModel};
pub use template::{
    CircularProgressTemplate, LinearProgressTemplate, ProgressTemplate, ThemedLinearProgressTemplate,
    ThemedProgressTemplate, default_circular_progress_template, default_linear_progress_template,
    default_progress_template,
};

pub use theme::{ProgressTheme, default_progress_theme};

use gpui::{Entity, SharedString};

use self::control::ProgressControl;

pub type Progress = Entity<ProgressControl>;

pub fn new(id: impl Into<SharedString>) -> ProgressBuilder {
    ProgressBuilder::new(id)
}

pub use model::ProgressMotion;
