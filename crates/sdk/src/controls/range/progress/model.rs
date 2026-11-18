crate::motion_profile!(ProgressMotion => progress {
    value: crate::motion::Transition,
    period: std::time::Duration,
});

use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::direction::{ProgressDirection, ProgressOrientation};
use super::template::{default_circular_progress_template, default_linear_progress_template, template_with_modifier};
use super::{ProgressTemplate, default_progress_template};
use super::control::ProgressControl;
use crate::infra::value::{ControlRange, value_from_input};

#[derive(Clone)]
pub struct ProgressModel {
    pub(crate) motion: crate::motion::MotionConfig<ProgressMotion>,
    pub(crate) id: SharedString,
    pub(crate) range: ControlRange,
    pub(crate) value: f32,
    pub(crate) enabled: bool,
    pub(crate) direction: ProgressDirection,
    pub(crate) show_thumb: bool,
    pub(crate) indeterminate: bool,
    pub(crate) template: Arc<dyn ProgressTemplate>,
}

pub struct ProgressRenderModel<'a> {
    pub id: &'a SharedString,
    pub range: ControlRange,
    pub value: f32,
    pub percentage: f32,
    pub enabled: bool,
    pub direction: ProgressDirection,
    pub show_thumb: bool,
    pub indeterminate: bool,
    pub phase: f32,
}

pub struct ProgressBuilder {
    pub(crate) model: ProgressModel,
    pub(crate) motion_enabled: bool,
}

impl ProgressBuilder {
    crate::motion_methods!(ProgressMotion, model.motion);

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: ProgressModel {
                motion: Default::default(),
                id: id.into(),
                range: ControlRange::default(),
                value: 0.0,
                enabled: true,
                direction: ProgressDirection::default(),
                show_thumb: false,
                indeterminate: false,
                template: default_progress_template(),
            },
            motion_enabled: true,
        }
    }

    pub fn range(mut self, range: impl Into<ControlRange>) -> Self {
        self.model.range = range.into();
        self.model.value = self.model.range.clamp(self.model.value);
        self
    }

    pub fn value(mut self, value: impl Into<f64>) -> Self {
        self.model.value = self.model.range.clamp(value_from_input(value));
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn direction(mut self, direction: ProgressDirection) -> Self {
        self.model.direction = direction;
        self
    }

    pub fn orientation(mut self, orientation: ProgressOrientation) -> Self {
        let reversed = self.model.direction.is_reversed();
        self.model.direction = match orientation {
            ProgressOrientation::Horizontal => {
                if reversed {
                    ProgressDirection::RightToLeft
                } else {
                    ProgressDirection::LeftToRight
                }
            }
            ProgressOrientation::Vertical => {
                if reversed {
                    ProgressDirection::TopToBottom
                } else {
                    ProgressDirection::BottomToTop
                }
            }
        };
        self
    }

    pub fn show_thumb(mut self, show_thumb: bool) -> Self {
        self.model.show_thumb = show_thumb;
        self
    }

    pub fn motion_enabled(mut self, motion_enabled: bool) -> Self {
        self.motion_enabled = motion_enabled;
        self
    }

    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.model.indeterminate = indeterminate;
        self
    }

    pub fn linear(mut self) -> Self {
        self.model.template = default_linear_progress_template();
        self
    }

    pub fn circular(mut self) -> Self {
        self.model.template = default_circular_progress_template();
        self
    }

    pub fn template(mut self, template: Arc<dyn ProgressTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &ProgressRenderModel<'_>) -> gpui::Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.model.template = template_with_modifier(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<ProgressControl> {
        cx.new(|cx| ProgressControl::from_builder(self, cx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::progress::template::default_linear_progress_template;

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_progress_template();
        let builder = ProgressBuilder::new("progress-test")
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }

    #[test]
    fn linear_assigns_linear_template() {
        let builder = ProgressBuilder::new("progress-linear").linear();
        assert!(Arc::ptr_eq(&builder.model.template, &default_linear_progress_template()));
    }

    #[test]
    fn value_clamps_to_range() {
        let builder = ProgressBuilder::new("progress-clamp").range(0..100).value(150);
        assert_eq!(builder.model.value, 100.0);
    }

    #[test]
    fn animated_defaults_true() {
        assert!(ProgressBuilder::new("progress-animated").motion_enabled);
        assert!(!ProgressBuilder::new("progress-animated-off").motion_enabled(false).motion_enabled);
    }

    #[test]
    fn indeterminate_builder_flag() {
        assert!(!ProgressBuilder::new("progress-det").model.indeterminate);
        assert!(ProgressBuilder::new("progress-indet").indeterminate(true).model.indeterminate);
    }
}
