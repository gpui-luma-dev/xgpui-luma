crate::motion_profile!(SliderMotion => slider {
    thumb: crate::motion::Transition,
});

use std::sync::atomic::{AtomicU64, Ordering};
use std::ops::RangeInclusive;
use std::sync::Arc;

use gpui::{AbsoluteLength, AppContext, Bounds, Entity, Hsla, Pixels, Point, SharedString};

use super::constraints::{clamp_and_snap_value, normalize_intervals};
use super::control::SliderControl;
use super::domain::DomainTrackRenderer;
use super::input::{SliderInputStrategy, wrap_and_snap};
use super::segments::build_track_segments;
use super::template::{SliderTemplate, default_slider_template, modified_slider_template};
use crate::theme::InteractionState;
use crate::infra::value::{ControlRange, normalized_step, value_from_input};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ThumbId(u64);

impl ThumbId {
    pub fn next() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SliderThumbRole {
    #[default]
    Value,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderThumbPolicy {
    pub min_count: usize,
    pub max_count: usize,
    pub min_distance: f32,
    pub allow_insert: bool,
    pub allow_remove: bool,
    /// When true, thumbs may share the same position while dragging (crossover / stack).
    pub allow_overlap: bool,
}

impl Default for SliderThumbPolicy {
    fn default() -> Self {
        Self {
            min_count: 1,
            max_count: 1,
            min_distance: 0.0,
            allow_insert: false,
            allow_remove: false,
            allow_overlap: false,
        }
    }
}

impl SliderThumbPolicy {
    pub fn multi_stop() -> Self {
        Self {
            min_count: 2,
            max_count: 8,
            min_distance: 0.02,
            allow_insert: true,
            allow_remove: true,
            allow_overlap: true,
        }
    }

    pub fn is_multi_thumb(self) -> bool {
        self.max_count > 1
    }

    pub fn supports_click_to_add(self) -> bool {
        self.is_multi_thumb() && self.allow_insert
    }

    pub fn supports_delete_to_remove(self) -> bool {
        self.is_multi_thumb() && self.allow_remove
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SliderThumbValue {
    pub id: ThumbId,
    pub position: f32,
    pub preview: Option<Hsla>,
    pub role: SliderThumbRole,
}

pub trait SliderValueMapping: Send + Sync + 'static {
    fn value_to_position(&self, value: f32, range: ControlRange) -> f32;

    fn position_to_value(&self, position: f32, range: ControlRange) -> f32;

    /// Whether scalar values wrap when the track wraps (default: true).
    /// Mirrored mappings can return false to preserve their maximum value on a circular track.
    fn wraps_value(&self) -> bool {
        true
    }
}

pub trait RadialHitTarget: Send + Sync + 'static {
    fn accepts_pointer(
        &self,
        bounds: Bounds<Pixels>,
        pointer: Point<Pixels>,
        thumb_position: f32,
        reversed: bool,
    ) -> bool;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrackSegmentKind {
    Domain,
    Active,
    Inactive,
    Blocked,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrackSegment {
    pub start: f32,
    pub end: f32,
    pub kind: TrackSegmentKind,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TrackPresentation {
    #[default]
    Fill,
    Domain,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SliderOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone)]
pub struct SliderModel {
    pub(crate) motion: crate::motion::MotionConfig<SliderMotion>,
    pub(crate) id: SharedString,
    pub(crate) strategy: SliderInputStrategy,
    pub(crate) presentation: TrackPresentation,
    pub(crate) range: ControlRange,
    pub(crate) step: f32,
    pub(crate) keyboard_position_step: Option<f32>,
    pub(crate) thumbs: Vec<SliderThumbValue>,
    pub(crate) allowed_intervals: Vec<RangeInclusive<f32>>,
    /// Optional intervals used only for blocked-track rendering.
    /// When unset, [`Self::allowed_intervals`] is used for both interaction and display.
    pub(crate) track_intervals: Option<Vec<RangeInclusive<f32>>>,
    pub(crate) reversed: bool,
    pub(crate) wrapping: bool,
    pub(crate) enabled: bool,
    pub(crate) corner_radius: Option<AbsoluteLength>,
    pub(crate) thumb_radius: Option<AbsoluteLength>,
    pub(crate) template: Arc<dyn SliderTemplate>,
    pub(crate) thumb_policy: SliderThumbPolicy,
    pub(crate) domain_track: Option<Arc<dyn DomainTrackRenderer>>,
    pub(crate) value_map: Option<Arc<dyn SliderValueMapping>>,
    pub(crate) radial_hit_target: Option<Arc<dyn RadialHitTarget>>,
}

pub struct SliderRenderModel<'a> {
    pub id: &'a SharedString,
    pub strategy: SliderInputStrategy,
    pub orientation: SliderOrientation,
    pub presentation: TrackPresentation,
    pub range: ControlRange,
    pub step: f32,
    pub thumbs: &'a [SliderThumbValue],
    pub track_segments: Vec<TrackSegment>,
    pub reversed: bool,
    pub wrapping: bool,
    pub enabled: bool,
    pub corner_radius: Option<AbsoluteLength>,
    pub thumb_radius: Option<AbsoluteLength>,
    pub thumb_policy: SliderThumbPolicy,
    pub active_thumb_id: Option<ThumbId>,
    pub state: InteractionState,
    pub domain_track: Option<Arc<dyn DomainTrackRenderer>>,
}

pub struct SliderBuilder {
    callbacks: crate::infra::callbacks::EventCallbacks<crate::controls::slider::SliderEvent>,

    pub(crate) model: SliderModel,
    pub(crate) motion_enabled: bool,
}

impl SliderBuilder {
    crate::motion_methods!(SliderMotion, model.motion);

    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::slider::SliderEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `SliderEvent::Change` through the existing event stream.
    pub fn on_value_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::slider::ThumbId, f32, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::slider::SliderEvent::Change { thumb_id, value } = event {
                callback(owner, thumb_id, *value, cx);
            }
        })
    }

    /// Handle `SliderEvent::Release` through the existing event stream.
    pub fn on_release<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::slider::ThumbId, f32, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::slider::SliderEvent::Release { thumb_id, value } = event {
                callback(owner, thumb_id, *value, cx);
            }
        })
    }

    /// Handle `SliderEvent::DragStart` through the existing event stream.
    pub fn on_drag_start<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::slider::ThumbId, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::slider::SliderEvent::DragStart { thumb_id } = event {
                callback(owner, thumb_id, cx);
            }
        })
    }

    /// Handle `SliderEvent::DragEnd` through the existing event stream.
    pub fn on_drag_end<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::slider::ThumbId, f32, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::slider::SliderEvent::DragEnd { thumb_id, value } = event {
                callback(owner, thumb_id, *value, cx);
            }
        })
    }

    /// Handle `SliderEvent::ThumbAdded` through the existing event stream.
    pub fn on_thumb_added<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::slider::ThumbId, f32, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::slider::SliderEvent::ThumbAdded { thumb_id, value } = event {
                callback(owner, thumb_id, *value, cx);
            }
        })
    }

    /// Handle `SliderEvent::ThumbRemoved` through the existing event stream.
    pub fn on_thumb_removed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::slider::ThumbId, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::slider::SliderEvent::ThumbRemoved { thumb_id } = event {
                callback(owner, thumb_id, cx);
            }
        })
    }

    /// Handle `SliderEvent::ThumbSelected` through the existing event stream.
    pub fn on_thumb_selected<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::slider::ThumbId, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::slider::SliderEvent::ThumbSelected { thumb_id } = event {
                callback(owner, thumb_id, cx);
            }
        })
    }

    /// Handle `SliderEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::slider::SliderEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `SliderEvent::HoverChanged` through the existing event stream.
    pub fn on_hover_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::slider::SliderEvent::HoverChanged { hovered } = event {
                callback(owner, *hovered, cx);
            }
        })
    }

    /// Handle `SliderEvent::EnabledChanged` through the existing event stream.
    pub fn on_enabled_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::slider::SliderEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        let thumb_id = ThumbId::next();
        Self {
            callbacks: Default::default(),

            motion_enabled: true,
            model: SliderModel {
                motion: Default::default(),
                id: id.into(),
                strategy: SliderInputStrategy::Horizontal,
                presentation: TrackPresentation::Fill,
                range: ControlRange::default(),
                step: 1.0,
                keyboard_position_step: None,
                thumbs: vec![SliderThumbValue {
                    id: thumb_id,
                    position: 0.0,
                    preview: None,
                    role: SliderThumbRole::Value,
                }],
                allowed_intervals: Vec::new(),
                track_intervals: None,
                reversed: false,
                wrapping: false,
                enabled: true,
                corner_radius: None,
                thumb_radius: None,
                template: default_slider_template(),
                thumb_policy: SliderThumbPolicy::default(),
                domain_track: None,
                value_map: None,
                radial_hit_target: None,
            },
        }
    }

    pub fn strategy(mut self, strategy: SliderInputStrategy) -> Self {
        self.model.strategy = strategy;
        self
    }

    pub fn set_strategy(&mut self, strategy: SliderInputStrategy) {
        self.model.strategy = strategy;
    }

    pub fn set_radial_hit_target(&mut self, target: Arc<dyn RadialHitTarget>) {
        self.model.radial_hit_target = Some(target);
    }

    pub fn orientation(mut self, orientation: SliderOrientation) -> Self {
        self.model.strategy = orientation.into();
        self
    }

    pub fn horizontal(mut self) -> Self {
        self.model.strategy = SliderInputStrategy::Horizontal;
        self
    }

    pub fn vertical(mut self) -> Self {
        self.model.strategy = SliderInputStrategy::Vertical;
        self
    }

    pub fn angular(mut self, min_angle: f32, max_angle: f32) -> Self {
        self.model.strategy = SliderInputStrategy::Angular { min_angle, max_angle };
        self
    }

    pub fn wrapping(mut self, wrapping: bool) -> Self {
        self.model.wrapping = wrapping;
        self.sync_primary_thumb_position();
        self
    }

    pub fn fill(mut self) -> Self {
        self.model.presentation = TrackPresentation::Fill;
        self
    }

    pub fn domain(mut self) -> Self {
        self.model.presentation = TrackPresentation::Domain;
        self
    }

    pub fn domain_track(mut self, renderer: Arc<dyn DomainTrackRenderer>) -> Self {
        self.model.presentation = TrackPresentation::Domain;
        self.model.domain_track = Some(renderer);
        self
    }

    pub fn value_map(mut self, mapping: Arc<dyn SliderValueMapping>) -> Self {
        self.model.value_map = Some(mapping);
        self.sync_primary_thumb_position();
        self
    }

    pub fn radial_hit_target(mut self, target: Arc<dyn RadialHitTarget>) -> Self {
        self.model.radial_hit_target = Some(target);
        self
    }

    pub fn range(mut self, range: impl Into<ControlRange>) -> Self {
        self.model.range = range.into();
        self.model.allowed_intervals = normalize_intervals(&self.model.allowed_intervals, self.model.range);
        self.sync_primary_thumb_position();
        self
    }

    pub fn allowed_intervals(mut self, intervals: Vec<RangeInclusive<f32>>) -> Self {
        self.model.allowed_intervals = normalize_intervals(&intervals, self.model.range);
        self.sync_primary_thumb_position();
        self
    }

    pub fn step(mut self, step: impl Into<f64>) -> Self {
        self.model.step = normalized_step(value_from_input(step));
        self.sync_primary_thumb_position();
        self
    }

    /// Opt into immediate arrow steps in normalized track position instead of value space.
    /// Useful for rings with multiple positions for the same value. Shift uses ten steps.
    /// Finite fractions in (0, 1] are accepted; invalid values retain the previous setting.
    /// Controls with allowed value intervals retain value-based stepping.
    pub fn keyboard_position_step(mut self, step: f32) -> Self {
        if step.is_finite() && step > 0.0 && step <= 1.0 {
            self.model.keyboard_position_step = Some(step);
        }
        self
    }

    pub fn value(mut self, value: impl Into<f64>) -> Self {
        let value = constrain_primary_value(value_from_input(value), &self.model);
        self.set_primary_position(position_for_value(&self.model, value));
        self
    }

    pub fn reversed(mut self, reversed: bool) -> Self {
        self.model.reversed = reversed;
        self
    }

    pub fn motion_enabled(mut self, motion_enabled: bool) -> Self {
        self.motion_enabled = motion_enabled;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn corner_radius(mut self, radius: AbsoluteLength) -> Self {
        self.model.corner_radius = Some(radius);
        self
    }

    pub fn clear_corner_radius(mut self) -> Self {
        self.model.corner_radius = None;
        self
    }

    pub fn thumb_radius(mut self, radius: AbsoluteLength) -> Self {
        self.model.thumb_radius = Some(radius);
        self
    }

    pub fn clear_thumb_radius(mut self) -> Self {
        self.model.thumb_radius = None;
        self
    }

    pub fn template(mut self, template: Arc<dyn SliderTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &SliderRenderModel<'_>) -> gpui::Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.model.template = modified_slider_template(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn thumb_policy(mut self, policy: SliderThumbPolicy) -> Self {
        self.model.thumb_policy = policy;
        self.ensure_thumb_count();
        self
    }

    /// Click empty track space to insert a new thumb (multi-thumb mode).
    pub fn allow_insert(mut self, allow: bool) -> Self {
        self.model.thumb_policy.allow_insert = allow;
        self
    }

    /// Delete/Backspace removes the selected thumb (multi-thumb mode).
    pub fn allow_remove(mut self, allow: bool) -> Self {
        self.model.thumb_policy.allow_remove = allow;
        self
    }

    pub fn multi_stop(self) -> Self {
        self.thumb_policy(SliderThumbPolicy::multi_stop()).domain()
    }

    pub fn thumb_values(mut self, values: impl IntoIterator<Item = (impl Into<f64>, Option<Hsla>)>) -> Self {
        self.model.thumbs = values
            .into_iter()
            .map(|(value, preview)| {
                let value = value_from_input(value);
                let position = position_for_value(&self.model, constrain_primary_value(value, &self.model));
                SliderThumbValue { id: ThumbId::next(), position, preview, role: SliderThumbRole::Value }
            })
            .collect();
        self.ensure_thumb_count();
        self
    }

    pub fn spawn(mut self, cx: &mut impl AppContext) -> Entity<SliderControl> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = { cx.new(|cx| SliderControl::from_builder(self, cx)) };
        callbacks.bind(&control, cx);
        control
    }

    fn sync_primary_thumb_position(&mut self) {
        let value = primary_value(&self.model);
        self.set_primary_position(position_for_value(&self.model, value));
    }

    fn set_primary_position(&mut self, position: f32) {
        if let Some(thumb) = self.model.thumbs.first_mut() {
            thumb.position = position.clamp(0.0, 1.0);
        }
    }

    fn ensure_thumb_count(&mut self) {
        let policy = self.model.thumb_policy;
        while self.model.thumbs.len() < policy.min_count {
            self.model.thumbs.push(SliderThumbValue {
                id: ThumbId::next(),
                position: 0.0,
                preview: None,
                role: SliderThumbRole::Value,
            });
        }

        if self.model.thumbs.len() > policy.max_count {
            self.model.thumbs.truncate(policy.max_count);
        }
    }
}

pub(crate) fn primary_value(model: &SliderModel) -> f32 {
    let position = model.thumbs.first().map(|thumb| thumb.position).unwrap_or(0.0);
    let value = value_for_position(model, position);
    constrain_primary_value(value, model)
}

pub(crate) fn value_for_position(model: &SliderModel, position: f32) -> f32 {
    model
        .value_map
        .as_ref()
        .map(|mapping| mapping.position_to_value(position, model.range))
        .unwrap_or_else(|| model.range.value_at(position))
}

pub(crate) fn position_for_value(model: &SliderModel, value: f32) -> f32 {
    model
        .value_map
        .as_ref()
        .map(|mapping| mapping.value_to_position(value, model.range))
        .unwrap_or_else(|| model.range.percentage(value))
        .clamp(0.0, 1.0)
}

pub(crate) fn constrain_primary_value(value: f32, model: &SliderModel) -> f32 {
    if model.wrapping
        && model.allowed_intervals.is_empty()
        && model.value_map.as_ref().is_none_or(|map| map.wraps_value())
    {
        wrap_and_snap(value, model.range, model.step)
    } else {
        clamp_and_snap_value(value, &model.allowed_intervals, model.range, model.step)
    }
}

pub(crate) fn build_render_segments_at(model: &SliderModel, thumb_position: f32) -> Vec<TrackSegment> {
    let presentation = if model.thumb_policy.is_multi_thumb() {
        TrackPresentation::Domain
    } else {
        model.presentation
    };
    let track_intervals = model.track_intervals.as_ref().unwrap_or(&model.allowed_intervals);
    build_track_segments(presentation, thumb_position, track_intervals, model.range)
}

#[cfg(test)]
mod animated_tests {
    use super::*;

    #[test]
    fn animated_defaults_true() {
        assert!(SliderBuilder::new("slider-animated").motion_enabled);
        assert!(!SliderBuilder::new("slider-animated-off").motion_enabled(false).motion_enabled);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_slider_template();
        let builder = SliderBuilder::new("slider-test")
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }
}
