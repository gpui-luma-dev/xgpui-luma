use gpui::{Context, IntoElement, Render, SharedString, Window, div, prelude::*};

use super::model::{
    StepperBuilder, StepperLabelPlacement, StepperModel, StepperRenderModel, derive_step_states, visual_step_state,
};
use super::{StepState, StepperTemplate};
use crate::motion::VisualTransition;
use crate::controls::progress::ProgressDirection;
use crate::theme::observe_theme_revision;

pub struct StepperControl {
    model: StepperModel,
    step_states: Vec<StepState>,
    visual_step_states: Vec<StepState>,
    from_step: f32,
    to_step: usize,
    transition: VisualTransition,
}

impl StepperControl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>, step_count: usize) -> StepperBuilder {
        StepperBuilder::new(id, step_count)
    }

    crate::motion_snapshot!(super::StepperMotion, model.motion);

    pub(crate) fn from_builder(builder: StepperBuilder, cx: &mut Context<Self>) -> Self {
        let mut builder = builder;
        builder.model.motion.freeze();
        let motion = builder.model.motion.profile().clone();
        observe_theme_revision(cx, |_, cx| cx.notify()).detach();
        let step_states = derive_step_states(&builder.model);
        let current = builder.model.current_step;
        let definition = motion.selection.enabled(builder.model.motion_enabled);
        let mut control = Self {
            model: builder.model,
            step_states,
            visual_step_states: Vec::new(),
            from_step: current as f32,
            to_step: current,
            transition: VisualTransition::with_transition(current as f32, definition),
        };
        control.refresh_visual_step_states();
        control
    }

    pub fn step_count(&self) -> usize {
        self.model.step_count
    }

    pub fn current_step(&self) -> usize {
        self.model.current_step
    }

    pub fn display_step(&self) -> f32 {
        self.transition.progress()
    }

    pub fn is_complete(&self) -> bool {
        self.model.workflow_complete
    }

    pub fn label_placement(&self) -> StepperLabelPlacement {
        self.model.label_placement
    }

    pub fn is_enabled(&self) -> bool {
        self.model.enabled
    }

    pub fn direction(&self) -> ProgressDirection {
        self.model.direction
    }

    pub fn set_current_step(&mut self, current: usize, cx: &mut Context<Self>) {
        let current = current.min(self.model.step_count.saturating_sub(1));
        let changed = self.model.current_step != current || self.model.workflow_complete;

        self.model.workflow_complete = false;
        if !changed {
            return;
        }

        self.retarget_step(current);
        self.model.current_step = current;
        self.refresh_step_states();
        cx.notify();
    }

    pub fn complete(&mut self, cx: &mut Context<Self>) {
        if self.model.workflow_complete {
            return;
        }

        let last = self.model.step_count.saturating_sub(1);
        self.model.workflow_complete = true;
        self.retarget_step(last);
        self.model.current_step = last;
        self.refresh_step_states();
        cx.notify();
    }

    pub fn set_step_state(&mut self, step_index: usize, state: StepState, cx: &mut Context<Self>) {
        if step_index >= self.model.step_count {
            return;
        }

        self.model.step_state_overrides[step_index] = Some(state);
        self.refresh_step_states();
        cx.notify();
    }

    pub fn clear_step_state(&mut self, step_index: usize, cx: &mut Context<Self>) {
        if step_index >= self.model.step_count {
            return;
        }

        self.model.step_state_overrides[step_index] = None;
        self.refresh_step_states();
        cx.notify();
    }

    pub fn set_labels(&mut self, labels: Vec<impl Into<SharedString>>, cx: &mut Context<Self>) {
        self.model.labels = labels.into_iter().map(Into::into).collect();
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }

        self.model.enabled = enabled;
        cx.notify();
    }

    pub fn set_direction(&mut self, direction: ProgressDirection, cx: &mut Context<Self>) {
        if self.model.direction == direction {
            return;
        }

        self.model.direction = direction;
        cx.notify();
    }

    pub fn set_label_placement(&mut self, placement: StepperLabelPlacement, cx: &mut Context<Self>) {
        if self.model.label_placement == placement {
            return;
        }

        self.model.label_placement = placement;
        cx.notify();
    }

    /// Control-wide policy. `false` settles motion immediately and keeps [`Self::motion_profile`].
    pub fn motion_enabled(&self) -> bool {
        self.model.motion_enabled
    }

    pub fn set_motion_enabled(&mut self, motion_enabled: bool, cx: &mut Context<Self>) {
        if self.model.motion_enabled == motion_enabled {
            return;
        }

        self.model.motion_enabled = motion_enabled;
        self.transition.set_transition(self.model.motion.profile().selection.enabled(motion_enabled));
        if !motion_enabled {
            self.transition.snap_to(self.to_step as f32);
        }
        cx.notify();
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn StepperTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    fn retarget_step(&mut self, next: usize) {
        self.from_step = self.display_step();
        self.to_step = next;
        self.transition.set_target(next as f32);
    }

    fn refresh_step_states(&mut self) {
        self.step_states = derive_step_states(&self.model);
        self.refresh_visual_step_states();
    }

    fn refresh_visual_step_states(&mut self) {
        let display_step = self.display_step();
        self.visual_step_states = (0..self.model.step_count)
            .map(|index| visual_step_state(&self.model, display_step, index))
            .collect();
    }

    fn render_model(&self) -> StepperRenderModel<'_> {
        StepperRenderModel {
            id: &self.model.id,
            step_count: self.model.step_count,
            current_step: self.model.current_step,
            display_step: self.display_step(),
            transition_progress: if (self.to_step as f32 - self.from_step).abs() <= f32::EPSILON {
                1.0
            } else {
                (self.display_step() - self.from_step) / (self.to_step as f32 - self.from_step)
            },
            from_step: self.from_step,
            to_step: self.to_step,
            step_states: &self.visual_step_states,
            labels: &self.model.labels,
            label_placement: self.model.label_placement,
            direction: self.model.direction,
            enabled: self.model.enabled,
            step_contents: &self.model.step_contents,
            content_height: self.model.content_height,
            icons: &self.model.icons,
        }
    }
}

impl Render for StepperControl {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let was_animating = self.transition.is_animating();
        let _ = self.transition.sync();
        self.refresh_visual_step_states();
        let model = self.render_model();

        self.transition.schedule_frame(window, cx);
        if was_animating || self.transition.is_animating() {
            cx.notify();
        }

        div().child(self.model.template.render(&model, window, cx)).into_any_element()
    }
}
