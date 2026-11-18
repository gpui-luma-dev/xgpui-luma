use crate::infra::attachments::{AttachmentHost, AttachmentTarget};
mod motion;
mod pointer;
mod render;
mod values;

use std::sync::Arc;

use gpui::{
    App, Bounds, Context, Empty, EventEmitter, Focusable, IntoElement, Pixels, Render, SharedString, Subscription,
    Window,
};

use super::constraints::normalize_intervals;
use super::domain::DomainTrackRenderer;
use super::input::SliderInputStrategy;
use super::model::{primary_value, SliderBuilder, SliderOrientation, SliderThumbValue, ThumbId, TrackPresentation};
use super::thumbs::{insert_thumb, thumb_value};
use super::model::SliderModel;
use crate::infra::interaction::ControlInteraction;
use crate::infra::value::{ControlRange, value_from_input};
use crate::motion::VisualTransition;
use crate::theme::observe_theme_revision;

use self::motion::{sync_thumb_preview, sync_thumb_previews};
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum SliderEvent {
    Change { thumb_id: ThumbId, value: f32 },
    Release { thumb_id: ThumbId, value: f32 },
    DragStart { thumb_id: ThumbId },
    DragEnd { thumb_id: ThumbId, value: f32 },
    ThumbAdded { thumb_id: ThumbId, value: f32 },
    ThumbRemoved { thumb_id: ThumbId },
    ThumbSelected { thumb_id: ThumbId },
    FocusChanged { focused: bool },
    HoverChanged { hovered: bool },
    EnabledChanged { enabled: bool },
}

#[derive(Clone, Debug)]
pub struct SliderDrag {
    pub(super) id: SharedString,
    pub(super) thumb_id: ThumbId,
}
impl SliderDrag {
    pub fn new(id: SharedString, thumb_id: ThumbId) -> Self {
        Self { id, thumb_id }
    }
}

impl Render for SliderDrag {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

pub struct SliderControl {
    model: SliderModel,
    interaction: ControlInteraction,
    attachments: AttachmentHost,
    track_bounds: Option<Bounds<Pixels>>,
    active_thumb_id: Option<ThumbId>,
    angular_drag_angle_offset: Option<f32>,
    angular_drag_pointer_angle: Option<f32>,
    focus_in_subscription: Option<Subscription>,
    focus_out_subscription: Option<Subscription>,
    emitted_focused: bool,
    motion_enabled: bool,
    thumb_transitions: Vec<VisualTransition>,
    display_thumbs: Vec<SliderThumbValue>,
}

impl EventEmitter<SliderEvent> for SliderControl {}

impl SliderControl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> SliderBuilder {
        SliderBuilder::new(id)
    }

    crate::motion_snapshot!(super::SliderMotion, model.motion);

    pub(crate) fn from_builder(builder: SliderBuilder, cx: &mut Context<Self>) -> Self {
        let mut builder = builder;
        builder.model.motion.freeze();
        let motion = builder.model.motion.profile().clone();
        let enabled = builder.model.enabled;
        let motion_enabled = builder.motion_enabled;
        let active_thumb_id = builder.model.thumbs.first().map(|thumb| thumb.id);
        let mut model = builder.model;
        sync_thumb_previews(&mut model);
        let definition = motion.thumb.enabled(motion_enabled);
        let thumb_transitions = model
            .thumbs
            .iter()
            .map(|thumb| VisualTransition::with_transition(thumb.position, definition.clone()))
            .collect();
        let display_thumbs = model.thumbs.clone();

        observe_theme_revision(cx, |_, cx| cx.notify()).detach();
        Self {
            model,
            attachments: AttachmentHost::with_accessible_role(gpui::Role::Slider),
            interaction: ControlInteraction::new(enabled, cx),
            track_bounds: None,
            active_thumb_id,
            angular_drag_angle_offset: None,
            angular_drag_pointer_angle: None,
            focus_in_subscription: None,
            focus_out_subscription: None,
            emitted_focused: false,
            motion_enabled,
            thumb_transitions,
            display_thumbs,
        }
    }

    pub fn value(&self) -> f32 {
        primary_value(&self.model)
    }

    pub fn thumbs(&self) -> &[super::model::SliderThumbValue] {
        &self.model.thumbs
    }

    pub fn active_thumb_id(&self) -> Option<ThumbId> {
        self.active_thumb_id
    }

    pub fn thumb_value(&self, thumb_id: ThumbId) -> Option<f32> {
        self.model
            .thumbs
            .iter()
            .find(|thumb| thumb.id == thumb_id)
            .map(|thumb| thumb_value(thumb, self.model.range, &self.model))
    }

    pub fn primary_thumb_id(&self) -> ThumbId {
        self.model.thumbs.first().map(|thumb| thumb.id).expect("slider always has a primary thumb")
    }

    pub fn range(&self) -> ControlRange {
        self.model.range
    }

    pub fn strategy(&self) -> SliderInputStrategy {
        self.model.strategy
    }

    pub fn orientation(&self) -> SliderOrientation {
        self.model.strategy.orientation()
    }

    pub fn motion_enabled(&self) -> bool {
        self.motion_enabled
    }

    pub fn set_motion_enabled(&mut self, motion_enabled: bool, cx: &mut Context<Self>) {
        if self.motion_enabled == motion_enabled {
            return;
        }
        self.motion_enabled = motion_enabled;
        let definition = self.model.motion.profile().thumb.enabled(motion_enabled);
        self.thumb_transitions = self
            .model
            .thumbs
            .iter()
            .zip(self.thumb_transitions.iter())
            .map(|(thumb, transition)| {
                let mut next = VisualTransition::with_transition(transition.progress(), definition.clone());
                next.set_target(thumb.position);
                next
            })
            .collect();
        if self.thumb_transitions.len() != self.model.thumbs.len() {
            self.rebuild_thumb_transitions();
        }
        cx.notify();
    }

    pub fn set_value(&mut self, value: impl Into<f64>, cx: &mut Context<Self>) {
        let value = value_from_input(value);
        let thumb_id = self.active_thumb_id.or_else(|| self.model.thumbs.first().map(|thumb| thumb.id));
        if let Some(thumb_id) = thumb_id {
            self.set_thumb_value_internal(thumb_id, value, false, true, cx);
        }
    }

    pub fn set_range(&mut self, range: impl Into<ControlRange>, cx: &mut Context<Self>) {
        self.model.range = range.into();
        self.model.allowed_intervals = normalize_intervals(&self.model.allowed_intervals, self.model.range);
        sync_thumb_previews(&mut self.model);
        cx.notify();
    }

    pub fn set_allowed_intervals(&mut self, intervals: Vec<std::ops::RangeInclusive<f32>>, cx: &mut Context<Self>) {
        self.model.allowed_intervals = normalize_intervals(&intervals, self.model.range);
        sync_thumb_previews(&mut self.model);
        cx.notify();
    }

    pub fn clear_allowed_intervals(&mut self, cx: &mut Context<Self>) {
        if self.model.allowed_intervals.is_empty() {
            return;
        }
        self.model.allowed_intervals.clear();
        sync_thumb_previews(&mut self.model);
        cx.notify();
    }

    pub fn set_track_intervals(&mut self, intervals: Vec<std::ops::RangeInclusive<f32>>, cx: &mut Context<Self>) {
        self.model.track_intervals = Some(normalize_intervals(&intervals, self.model.range));
        cx.notify();
    }

    pub fn clear_track_intervals(&mut self, cx: &mut Context<Self>) {
        if self.model.track_intervals.is_none() {
            return;
        }
        self.model.track_intervals = None;
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }

        let was_hovered = self.interaction.hovered();
        let was_pressed = self.interaction.is_pressed();
        let active_thumb_id = self.active_thumb_id;
        self.model.enabled = enabled;
        self.interaction.set_enabled(enabled);
        if !enabled {
            self.active_thumb_id = None;
            if was_pressed && let Some(thumb_id) = active_thumb_id {
                let value = self.thumb_value(thumb_id).unwrap_or(self.model.range.start);
                cx.emit(SliderEvent::DragEnd { thumb_id, value });
            }
            if was_hovered {
                cx.emit(SliderEvent::HoverChanged { hovered: false });
            }
            self.emit_focus_changed(false, cx);
        }
        cx.emit(SliderEvent::EnabledChanged { enabled });
        cx.notify();
    }

    pub fn set_template(&mut self, template: Arc<dyn super::SliderTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    pub fn set_domain_track(&mut self, renderer: Arc<dyn DomainTrackRenderer>, cx: &mut Context<Self>) {
        self.model.presentation = TrackPresentation::Domain;
        self.model.domain_track = Some(renderer);
        sync_thumb_previews(&mut self.model);
        cx.notify();
    }

    pub fn set_thumb_position(&mut self, thumb_id: ThumbId, percentage: f32, cx: &mut Context<Self>) -> bool {
        self.set_thumb_position_internal(thumb_id, percentage.clamp(0.0, 1.0), true, true, cx)
    }

    pub fn select_thumb_id(&mut self, thumb_id: ThumbId, cx: &mut Context<Self>) -> bool {
        let changed = self.select_thumb(thumb_id, true, cx);
        if changed {
            cx.notify();
        }
        changed
    }

    pub fn insert_thumb_at(&mut self, percentage: f32, cx: &mut Context<Self>) -> Option<ThumbId> {
        let percentage = percentage.clamp(0.0, 1.0);
        let thumb_id = insert_thumb(&mut self.model, percentage)?;
        sync_thumb_preview(&mut self.model, thumb_id);
        self.sync_transitions_with_thumbs();
        if let Some(transition) = self.transition_for_thumb_mut(thumb_id) {
            transition.snap_to(percentage);
        }
        self.select_thumb(thumb_id, true, cx);
        let value = self.thumb_value(thumb_id).unwrap_or(self.model.range.start);
        cx.emit(SliderEvent::ThumbAdded { thumb_id, value });
        cx.emit(SliderEvent::Change { thumb_id, value });
        cx.notify();
        Some(thumb_id)
    }

    pub fn remove_thumb_id(&mut self, thumb_id: ThumbId, cx: &mut Context<Self>) -> bool {
        self.remove_thumb(thumb_id, cx)
    }

    pub fn sync_domain_thumb_previews(&mut self) {
        sync_thumb_previews(&mut self.model);
    }
}

impl Focusable for SliderControl {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.interaction.focus_handle().clone()
    }
}

impl AttachmentTarget for SliderControl {
    fn attachments(&self) -> &AttachmentHost {
        &self.attachments
    }
    fn attachments_mut(&mut self) -> &mut AttachmentHost {
        &mut self.attachments
    }
    fn attachment_anchor(&self, bounds: Bounds<Pixels>) -> Bounds<Pixels> {
        let Some(track) = self.track_bounds else {
            return bounds;
        };
        // Linear sliders anchor to their current active display thumb; angular
        // sliders retain the control anchor until their templates expose a region.
        if self.model.strategy.is_angular() {
            return bounds;
        }
        let thumb = self
            .display_thumbs
            .iter()
            .find(|thumb| Some(thumb.id) == self.active_thumb_id)
            .or_else(|| self.display_thumbs.first());
        let Some(thumb) = thumb else {
            return bounds;
        };
        let position = super::layout::display_position(thumb.position, self.model.reversed);
        match self.model.strategy.orientation() {
            super::model::SliderOrientation::Horizontal => Bounds::new(
                gpui::point(track.origin.x + track.size.width * position, bounds.origin.y),
                gpui::Size { width: gpui::px(0.0), height: bounds.size.height },
            ),
            super::model::SliderOrientation::Vertical => Bounds::new(
                gpui::point(bounds.origin.x, track.origin.y + track.size.height * (1.0 - position)),
                gpui::Size { width: bounds.size.width, height: gpui::px(0.0) },
            ),
        }
    }
}

#[cfg(all(test, feature = "test-support"))]
mod attachment_tests {
    use super::*;
    use gpui::{AppContext, TestAppContext, point, px, Size};

    #[test]
    fn configured_thumb_spring_retargets_without_losing_velocity() {
        use std::time::Duration;
        let app = TestAppContext::single();
        app.update(|cx| {
            let slider = cx.new(|cx| {
                SliderControl::from_builder(
                    SliderBuilder::new("spring-thumb")
                        .range(0..100)
                        .value(0)
                        .motion(|m| m.thumb = crate::motion::Transition::spring(200., 18.)),
                    cx,
                )
            });
            slider.update(cx, |slider, cx| {
                slider.thumb_transitions[0].elapse_for_test(Duration::ZERO);
                slider.set_value(100, cx);
                slider.thumb_transitions[0].elapse_for_test(Duration::from_millis(100));
                slider.thumb_transitions[0].sync();
                let before = slider.thumb_transitions[0].state();
                assert!(before.velocity > 0.);
                slider.set_value(0, cx);
                assert_eq!(slider.thumb_transitions[0].state(), before);
                slider.set_motion_enabled(false, cx);
                assert_eq!(slider.thumb_transitions[0].progress(), 0.);
                assert!(!slider.thumb_transitions[0].is_animating());
            });
        });
    }

    #[test]
    fn attachment_anchor_tracks_active_thumb_and_reversal() {
        let app = TestAppContext::single();
        app.update(|cx| {
            let slider = cx.new(|cx| SliderControl::from_builder(SliderBuilder::new("anchor").value(50.0), cx));
            let bounds = Bounds::new(point(px(100.0), px(200.0)), Size { width: px(220.0), height: px(32.0) });
            slider.update(cx, |slider, _| {
                slider.track_bounds =
                    Some(Bounds::new(point(px(110.0), px(212.0)), Size { width: px(200.0), height: px(8.0) }));
                let half = slider.attachment_anchor(bounds);
                assert_eq!(half.origin.x, px(210.0));
                slider.display_thumbs[0].position = 0.8;
                assert_eq!(slider.attachment_anchor(bounds).origin.x, px(270.0));
                slider.model.reversed = true;
                assert!((f32::from(slider.attachment_anchor(bounds).origin.x) - 150.0).abs() < 0.01);
                assert_eq!(half.size.height, bounds.size.height);
            });
        });
    }
}
