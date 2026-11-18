crate::motion_profile!(StepperMotion => stepper {
    selection: crate::motion::Transition,
});

use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Entity, SharedString, Window};

use super::control::StepperControl;
use super::template::{default_stepper_template, template_with_modifier};
use super::StepperTemplate;
use crate::controls::progress::ProgressDirection;
use crate::infra::icon::SelectionStatusIcons;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StepState {
    Complete,
    #[default]
    InProgress,
    Incomplete,
}

/// Label position relative to the step badge. Side placements apply to vertical steppers; horizontal steppers keep labels below the badge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StepperLabelPlacement {
    #[default]
    Below,
    /// Left of the badge in vertical layouts.
    Start,
    /// Right of the badge in vertical layouts.
    End,
}

pub type StepperContentRenderer = Arc<dyn Fn(&mut Window, &mut App) -> AnyElement + Send + Sync>;

#[derive(Clone, Debug)]
pub struct StepperItem {
    pub index: usize,
    pub state: StepState,
    pub label: Option<SharedString>,
}

#[derive(Clone)]
pub struct StepperModel {
    pub(crate) motion: crate::motion::MotionConfig<StepperMotion>,
    pub(crate) id: SharedString,
    pub(crate) step_count: usize,
    pub(crate) current_step: usize,
    pub(crate) step_state_overrides: Vec<Option<StepState>>,
    pub(crate) labels: Vec<SharedString>,
    pub(crate) label_placement: StepperLabelPlacement,
    pub(crate) direction: ProgressDirection,
    pub(crate) enabled: bool,
    pub(crate) workflow_complete: bool,
    pub(crate) motion_enabled: bool,
    pub(crate) icons_explicit: bool,
    pub(crate) icons: SelectionStatusIcons,
    pub(crate) step_contents: Vec<Option<StepperContentRenderer>>,
    pub(crate) content_height: Option<f32>,
    pub(crate) template: Arc<dyn StepperTemplate>,
}

pub struct StepperRenderModel<'a> {
    pub id: &'a SharedString,
    pub step_count: usize,
    pub current_step: usize,
    /// Interpolated step position for track fill / panel slide (`0 .. step_count-1`).
    pub display_step: f32,
    /// Transition progress for the active from→to hop (`0..1`).
    pub transition_progress: f32,
    pub from_step: f32,
    pub to_step: usize,
    pub step_states: &'a [StepState],
    pub labels: &'a [SharedString],
    pub label_placement: StepperLabelPlacement,
    pub direction: ProgressDirection,
    pub enabled: bool,
    pub step_contents: &'a [Option<StepperContentRenderer>],
    pub content_height: Option<f32>,
    pub icons: &'a SelectionStatusIcons,
}

impl StepperRenderModel<'_> {
    pub fn has_content_panel(&self) -> bool {
        self.step_contents.iter().any(|slot| slot.is_some())
    }
}

pub struct StepperBuilder {
    pub(crate) model: StepperModel,
}

impl StepperBuilder {
    crate::motion_methods!(StepperMotion, model.motion);

    pub fn new(id: impl Into<SharedString>, step_count: usize) -> Self {
        let step_count = step_count.max(1);
        Self {
            model: StepperModel {
                motion: Default::default(),
                id: id.into(),
                step_count,
                current_step: 0,
                step_state_overrides: vec![None; step_count],
                labels: Vec::new(),
                label_placement: StepperLabelPlacement::default(),
                direction: ProgressDirection::default(),
                enabled: true,
                workflow_complete: false,
                motion_enabled: true,
                icons_explicit: false,
                icons: SelectionStatusIcons::default(),
                step_contents: vec![None; step_count],
                content_height: None,
                template: default_stepper_template(),
            },
        }
    }

    pub fn current_step(mut self, current: usize) -> Self {
        self.model.current_step = current.min(self.model.step_count.saturating_sub(1));
        self
    }

    /// Capture shared completion defaults; explicit glyphs retain precedence.
    pub fn stylesheet(mut self, stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        if !self.model.icons_explicit {
            self.model.icons = SelectionStatusIcons::from_stylesheet(stylesheet);
        }
        self
    }

    pub fn icons(mut self, icons: SelectionStatusIcons) -> Self {
        self.model.icons_explicit = true;
        self.model.icons = icons;
        self
    }

    pub fn step_state(mut self, step_index: usize, state: StepState) -> Self {
        if step_index < self.model.step_count {
            self.model.step_state_overrides[step_index] = Some(state);
        }
        self
    }

    pub fn labels(mut self, labels: Vec<impl Into<SharedString>>) -> Self {
        self.model.labels = labels.into_iter().map(Into::into).collect();
        self
    }

    pub fn label_placement(mut self, placement: StepperLabelPlacement) -> Self {
        self.model.label_placement = placement;
        self
    }

    pub fn complete(mut self) -> Self {
        self.model.workflow_complete = true;
        self.model.current_step = self.model.step_count.saturating_sub(1);
        self
    }

    pub fn direction(mut self, direction: ProgressDirection) -> Self {
        self.model.direction = direction;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn motion_enabled(mut self, motion_enabled: bool) -> Self {
        self.model.motion_enabled = motion_enabled;
        self
    }

    pub fn content_height(mut self, height: f32) -> Self {
        self.model.content_height = Some(height.max(0.0));
        self
    }

    pub fn step_content<F>(mut self, step_index: usize, content: F) -> Self
    where
        F: Fn(&mut Window, &mut App) -> AnyElement + Send + Sync + 'static,
    {
        if step_index < self.model.step_count {
            self.model.step_contents[step_index] = Some(Arc::new(content));
        }
        self
    }

    pub fn template(mut self, template: Arc<dyn StepperTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &StepperRenderModel<'_>) -> gpui::Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.model.template = template_with_modifier(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<StepperControl> {
        cx.new(|cx| StepperControl::from_builder(self, cx))
    }
}

pub(crate) fn derive_step_states(model: &StepperModel) -> Vec<StepState> {
    if model.workflow_complete {
        return vec![StepState::Complete; model.step_count];
    }

    (0..model.step_count)
        .map(|index| {
            if let Some(state) = model.step_state_overrides[index] {
                return state;
            }

            if index < model.current_step {
                StepState::Complete
            } else if index == model.current_step {
                StepState::InProgress
            } else {
                StepState::Incomplete
            }
        })
        .collect()
}

/// Badge visual state from interpolated `display_step` (snaps at half-step thresholds).
pub(crate) fn visual_step_state(model: &StepperModel, display_step: f32, index: usize) -> StepState {
    if let Some(state) = model.step_state_overrides.get(index).copied().flatten() {
        return state;
    }
    if model.workflow_complete {
        return StepState::Complete;
    }

    let index_f = index as f32;
    if display_step >= index_f + 0.5 {
        StepState::Complete
    } else if (display_step - index_f).abs() < 0.5 {
        StepState::InProgress
    } else {
        StepState::Incomplete
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_step_clamps_to_last_index() {
        let builder = StepperBuilder::new("stepper", 4).current_step(99);
        assert_eq!(builder.model.current_step, 3);
    }

    #[test]
    fn derive_step_states_at_start() {
        let model = StepperBuilder::new("stepper", 4).current_step(0).model;
        let states = derive_step_states(&model);
        assert_eq!(
            states,
            vec![StepState::InProgress, StepState::Incomplete, StepState::Incomplete, StepState::Incomplete]
        );
    }

    #[test]
    fn derive_step_states_at_end() {
        let model = StepperBuilder::new("stepper", 4).current_step(3).model;
        let states = derive_step_states(&model);
        assert_eq!(states, vec![StepState::Complete, StepState::Complete, StepState::Complete, StepState::InProgress]);
    }

    #[test]
    fn derive_step_states_when_workflow_complete() {
        let model = StepperBuilder::new("stepper", 4).complete().model;
        let states = derive_step_states(&model);
        assert!(states.iter().all(|state| *state == StepState::Complete));
    }

    #[test]
    fn explicit_step_state_override() {
        let model = StepperBuilder::new("stepper", 3).current_step(0).step_state(2, StepState::Complete).model;
        let states = derive_step_states(&model);
        assert_eq!(states[2], StepState::Complete);
    }

    #[test]
    fn builder_animated_defaults_true() {
        assert!(StepperBuilder::new("stepper", 3).model.motion_enabled);
        assert!(!StepperBuilder::new("stepper", 3).motion_enabled(false).model.motion_enabled);
    }
}
