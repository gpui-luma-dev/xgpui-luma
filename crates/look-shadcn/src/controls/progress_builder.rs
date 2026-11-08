//! Look-owned progress builder. Spawn synthesizes the SDK [`xgpui_luma::controls::progress::Progress`].

use xgpui_luma::controls::progress::{ProgressBuilder, ProgressDirection, ProgressOrientation};
use xgpui_luma::infra::value::ControlRange;
use gpui::{Context, SharedString};

use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

/// Builder in the guise of a progress control: Shadcn template plus SDK options, until `.spawn(cx)`.
pub struct Progress {
    look: Option<ShadcnLook>,
    linear: bool,
    builder: ProgressBuilder,
    size: ShadcnSize,
}

impl Progress {
    xgpui_luma::forward_motion!(xgpui_luma::controls::progress::ProgressMotion, builder builder);

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self { look: None, linear: false, builder: xgpui_luma::controls::progress::new(id), size: ShadcnSize::Md }
    }

    pub fn linear(id: impl Into<SharedString>) -> Self {
        Self {
            look: None,
            linear: true,
            builder: xgpui_luma::controls::progress::new(id).linear(),
            size: ShadcnSize::Md,
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn range(mut self, range: impl Into<ControlRange>) -> Self {
        self.builder = self.builder.range(range);
        self
    }

    pub fn value(mut self, value: impl Into<f64>) -> Self {
        self.builder = self.builder.value(value);
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

    pub fn direction(mut self, direction: ProgressDirection) -> Self {
        self.builder = self.builder.direction(direction);
        self
    }

    pub fn orientation(mut self, orientation: ProgressOrientation) -> Self {
        self.builder = self.builder.orientation(orientation);
        self
    }

    pub fn show_thumb(mut self, show_thumb: bool) -> Self {
        self.builder = self.builder.show_thumb(show_thumb);
        self
    }

    pub fn motion_enabled(mut self, motion_enabled: bool) -> Self {
        self.builder = self.builder.motion_enabled(motion_enabled);
        self
    }

    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.builder = self.builder.indeterminate(indeterminate);
        self
    }

    pub fn as_linear(mut self) -> Self {
        self.linear = true;
        self.builder = self.builder.linear();
        self
    }

    pub fn circular(mut self) -> Self {
        self.linear = false;
        self.builder = self.builder.circular();
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> xgpui_luma::controls::progress::Progress {
        let look = resolve_look_from(self.look.as_ref(), cx);
        self.into_sdk_builder(look).spawn(cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> ProgressBuilder {
        let self_builder = self.builder.motion_defaults(look.motion_defaults().progress);
        let template = if self.linear {
            look.linear_progress_template()
        } else {
            look.progress_template()
        };
        self_builder.size(self.size.control_size()).template(template)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = Progress::linear("ok").look(&look).range(0..100).value(40).into_sdk_builder(look);
    }
}
