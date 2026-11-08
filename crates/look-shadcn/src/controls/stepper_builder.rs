//! Look-owned stepper builder. Spawn synthesizes the SDK [`xgpui_luma::controls::stepper::Stepper`].

use gpui::{AnyElement, App, Context, SharedString, Window};
use xgpui_luma::controls::progress::ProgressDirection;
use xgpui_luma::controls::stepper::{StepperBuilder, StepperLabelPlacement, StepState};
use xgpui_luma::infra::icon::SelectionStatusIcons;
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

/// Builder in the guise of a stepper: Shadcn template plus SDK options, until `.spawn(cx)`.
pub struct Stepper {
    look: Option<ShadcnLook>,
    builder: StepperBuilder,
    custom_template: bool,
    size: ShadcnSize,
}

impl Stepper {
    xgpui_luma::forward_motion!(xgpui_luma::controls::stepper::StepperMotion, builder builder);

    pub fn new(id: impl Into<SharedString>, step_count: usize) -> Self {
        Self {
            look: None,
            builder: xgpui_luma::controls::stepper::stepper(id, step_count),
            custom_template: false,
            size: ShadcnSize::Md,
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn current_step(mut self, current: usize) -> Self {
        self.builder = self.builder.current_step(current);
        self
    }

    pub fn icons(mut self, icons: SelectionStatusIcons) -> Self {
        self.builder = self.builder.icons(icons);
        self
    }

    pub fn step_state(mut self, step_index: usize, state: StepState) -> Self {
        self.builder = self.builder.step_state(step_index, state);
        self
    }

    pub fn labels(mut self, labels: Vec<impl Into<SharedString>>) -> Self {
        self.builder = self.builder.labels(labels);
        self
    }

    pub fn label_placement(mut self, placement: StepperLabelPlacement) -> Self {
        self.builder = self.builder.label_placement(placement);
        self
    }

    pub fn complete(mut self) -> Self {
        self.builder = self.builder.complete();
        self
    }

    pub fn direction(mut self, direction: ProgressDirection) -> Self {
        self.builder = self.builder.direction(direction);
        self
    }

    pub fn size(mut self, size: ShadcnSize) -> Self {
        self.size = size;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }

    pub fn motion_enabled(mut self, motion_enabled: bool) -> Self {
        self.builder = self.builder.motion_enabled(motion_enabled);
        self
    }

    pub fn content_height(mut self, height: f32) -> Self {
        self.builder = self.builder.content_height(height);
        self
    }

    pub fn step_content<F>(mut self, step_index: usize, content: F) -> Self
    where
        F: Fn(&mut Window, &mut App) -> AnyElement + Send + Sync + 'static,
    {
        self.builder = self.builder.step_content(step_index, content);
        self
    }

    pub fn template(mut self, template: std::sync::Arc<dyn xgpui_luma::controls::stepper::StepperTemplate>) -> Self {
        self.custom_template = true;
        self.builder = self.builder.template(template);
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> xgpui_luma::controls::stepper::Stepper {
        let look = resolve_look_from(self.look.as_ref(), cx);
        self.into_sdk_builder(look).spawn(cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> StepperBuilder {
        let self_builder =
            self.builder.stylesheet(&look.stylesheet().common).motion_defaults(look.motion_defaults().stepper);
        let builder = self_builder.size(self.size.control_size());
        if self.custom_template {
            builder
        } else {
            builder.template(look.stepper_template())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = Stepper::new("ok", 3).look(&look).current_step(1).into_sdk_builder(look);
    }
}
