mod styled;

pub use styled::{StyledStepperTheme, resolve_stepper};
mod control;
mod model;
mod template;
pub(crate) mod theme;

pub use model::{
    StepState, StepperBuilder, StepperContentRenderer, StepperItem, StepperLabelPlacement, StepperModel,
    StepperRenderModel,
};
pub use template::{StepperTemplate, ThemedStepperTemplate, default_stepper_template};

pub use theme::{StepperTheme, default_stepper_theme};

use gpui::{Entity, SharedString};

use self::control::StepperControl;

pub type Stepper = Entity<StepperControl>;

pub fn stepper(id: impl Into<SharedString>, step_count: usize) -> StepperBuilder {
    StepperBuilder::new(id, step_count)
}

pub use model::StepperMotion;
