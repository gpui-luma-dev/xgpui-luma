//! Look-owned slider builder. Spawn synthesizes the SDK [`xgpui_luma::controls::slider::Slider`].

use std::ops::RangeInclusive;
use std::sync::Arc;

use gpui::{App, Context, Div, Hsla, SharedString, Stateful, px};
use xgpui_luma::controls::slider::{
    SliderBuilder, SliderOrientation, SliderRenderModel, SliderTemplate, SliderTemplateModifier, SliderThumbPolicy,
    TrackPresentation,
};
use xgpui_luma::infra::value::ControlRange;
use xgpui_luma::theme::InteractionState;

use super::button::{ButtonRadiusPreset, ShadcnButtonStyle};
use super::slider::{resolve_slider_thumb_radius_preset, resolve_slider_track_radius_preset, slider_look};
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

/// Builder in the guise of a slider: Shadcn axes plus SDK options, until `.spawn(cx)`.
pub struct Slider {
    motion: xgpui_luma::motion::MotionOverrides<xgpui_luma::controls::slider::SliderMotion>,
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::slider::SliderEvent>,

    id: SharedString,
    look: Option<ShadcnLook>,
    style: ShadcnButtonStyle,
    size: ShadcnSize,
    radius: Option<ButtonRadiusPreset>,
    range: ControlRange,
    step: f64,
    value: Option<f64>,
    enabled: bool,
    motion_enabled: bool,
    wrapping: bool,
    reversed: bool,
    orientation: Option<SliderOrientation>,
    presentation: Option<TrackPresentation>,
    thumb_policy: Option<SliderThumbPolicy>,
    angular: Option<(f32, f32)>,
    allowed_intervals: Option<Vec<RangeInclusive<f32>>>,
    thumb_values: Option<Vec<(f64, Option<Hsla>)>>,
    template: Option<Arc<dyn SliderTemplate>>,
    modifiers: Vec<SliderTemplateModifier>,
}

impl Slider {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::slider::SliderEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `SliderEvent::Change` through the existing event stream.
    pub fn on_value_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::slider::ThumbId, f32, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::slider::SliderEvent::Change { thumb_id, value } = event {
                callback(owner, thumb_id, *value, cx);
            }
        })
    }

    /// Handle `SliderEvent::Release` through the existing event stream.
    pub fn on_release<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::slider::ThumbId, f32, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::slider::SliderEvent::Release { thumb_id, value } = event {
                callback(owner, thumb_id, *value, cx);
            }
        })
    }

    /// Handle `SliderEvent::DragStart` through the existing event stream.
    pub fn on_drag_start<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::slider::ThumbId, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::slider::SliderEvent::DragStart { thumb_id } = event {
                callback(owner, thumb_id, cx);
            }
        })
    }

    /// Handle `SliderEvent::DragEnd` through the existing event stream.
    pub fn on_drag_end<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::slider::ThumbId, f32, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::slider::SliderEvent::DragEnd { thumb_id, value } = event {
                callback(owner, thumb_id, *value, cx);
            }
        })
    }

    /// Handle `SliderEvent::ThumbAdded` through the existing event stream.
    pub fn on_thumb_added<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::slider::ThumbId, f32, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::slider::SliderEvent::ThumbAdded { thumb_id, value } = event {
                callback(owner, thumb_id, *value, cx);
            }
        })
    }

    /// Handle `SliderEvent::ThumbRemoved` through the existing event stream.
    pub fn on_thumb_removed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::slider::ThumbId, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::slider::SliderEvent::ThumbRemoved { thumb_id } = event {
                callback(owner, thumb_id, cx);
            }
        })
    }

    /// Handle `SliderEvent::ThumbSelected` through the existing event stream.
    pub fn on_thumb_selected<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::slider::ThumbId, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::slider::SliderEvent::ThumbSelected { thumb_id } = event {
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
            if let xgpui_luma::controls::slider::SliderEvent::FocusChanged { focused } = event {
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
            if let xgpui_luma::controls::slider::SliderEvent::HoverChanged { hovered } = event {
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
            if let xgpui_luma::controls::slider::SliderEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            id: id.into(),
            look: None,
            motion: Default::default(),
            style: ShadcnButtonStyle::Primary,
            size: ShadcnSize::Md,
            radius: None,
            range: ControlRange::default(),
            step: 1.0,
            value: None,
            enabled: true,
            motion_enabled: true,
            wrapping: false,
            reversed: false,
            orientation: None,
            presentation: None,
            thumb_policy: None,
            angular: None,
            allowed_intervals: None,
            thumb_values: None,
            template: None,
            modifiers: Vec::new(),
        }
    }

    xgpui_luma::forward_motion!(xgpui_luma::controls::slider::SliderMotion, edits motion);

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn style(mut self, style: ShadcnButtonStyle) -> Self {
        self.style = style;
        self
    }

    pub fn primary(self) -> Self {
        self.style(ShadcnButtonStyle::Primary)
    }

    pub fn secondary(self) -> Self {
        self.style(ShadcnButtonStyle::Secondary)
    }

    pub fn size(mut self, size: ShadcnSize) -> Self {
        self.size = size;
        self
    }

    pub fn radius(mut self, radius: ButtonRadiusPreset) -> Self {
        self.radius = Some(radius);
        self
    }

    pub fn range(mut self, range: impl Into<ControlRange>) -> Self {
        self.range = range.into();
        self
    }

    pub fn step(mut self, step: impl Into<f64>) -> Self {
        self.step = step.into();
        self
    }

    pub fn value(mut self, value: impl Into<f64>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn motion_enabled(mut self, motion_enabled: bool) -> Self {
        self.motion_enabled = motion_enabled;
        self
    }

    pub fn wrapping(mut self, wrapping: bool) -> Self {
        self.wrapping = wrapping;
        self
    }

    pub fn reversed(mut self, reversed: bool) -> Self {
        self.reversed = reversed;
        self
    }

    pub fn orientation(mut self, orientation: SliderOrientation) -> Self {
        self.orientation = Some(orientation);
        self
    }

    pub fn horizontal(self) -> Self {
        self.orientation(SliderOrientation::Horizontal)
    }

    pub fn vertical(self) -> Self {
        self.orientation(SliderOrientation::Vertical)
    }

    pub fn fill(mut self) -> Self {
        self.presentation = Some(TrackPresentation::Fill);
        self
    }

    pub fn domain(mut self) -> Self {
        self.presentation = Some(TrackPresentation::Domain);
        self
    }

    pub fn thumb_policy(mut self, policy: SliderThumbPolicy) -> Self {
        self.thumb_policy = Some(policy);
        self
    }

    pub fn multi_stop(self) -> Self {
        self.thumb_policy(SliderThumbPolicy::multi_stop()).domain()
    }

    pub fn angular(mut self, min_angle: f32, max_angle: f32) -> Self {
        self.angular = Some((min_angle, max_angle));
        self
    }

    pub fn allowed_intervals(mut self, intervals: Vec<RangeInclusive<f32>>) -> Self {
        self.allowed_intervals = Some(intervals);
        self
    }

    pub fn thumb_values(mut self, values: impl IntoIterator<Item = (impl Into<f64>, Option<Hsla>)>) -> Self {
        self.thumb_values = Some(values.into_iter().map(|(value, preview)| (value.into(), preview)).collect());
        self
    }

    pub fn template(mut self, template: Arc<dyn SliderTemplate>) -> Self {
        self.template = Some(template);
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &SliderRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> xgpui_luma::controls::slider::Slider {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = {
            let look = self.resolve_look(cx);
            let theme = crate::tooltip_theme(&look);
            let entity = self.into_sdk_builder(look).spawn(cx);
            entity.update(cx, |control, _| {
                xgpui_luma::infra::attachments::AttachmentTarget::attachments_mut(control).set_tooltip_theme(theme);
            });
            entity
        };
        callbacks.bind(&control, cx);
        control
    }

    fn resolve_look(&self, cx: &App) -> ShadcnLook {
        resolve_look_from(self.look.as_ref(), cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> SliderBuilder {
        let template = self.template.unwrap_or_else(|| look.slider_template_with_style(self.style));
        let mut builder = xgpui_luma::controls::slider::new(self.id)
            .motion_defaults(look.motion_defaults().slider)
            .with_motion_overrides(self.motion)
            .template(template)
            .size(self.size.control_size())
            .range(self.range)
            .step(self.step)
            .enabled(self.enabled)
            .motion_enabled(self.motion_enabled)
            .wrapping(self.wrapping)
            .reversed(self.reversed);
        if let Some(preset) = self.radius {
            let tokens = look.mode_tokens();
            let resolved = slider_look(
                tokens.as_ref(),
                look.mode(),
                self.style,
                self.size.control_size(),
                None,
                InteractionState::default(),
            );
            builder = builder
                .corner_radius(
                    px(resolve_slider_track_radius_preset(preset, &tokens.metrics, resolved.track_height)).into(),
                )
                .thumb_radius(
                    px(resolve_slider_thumb_radius_preset(preset, &tokens.metrics, resolved.thumb_size)).into(),
                );
        }
        if let Some(orientation) = self.orientation {
            builder = builder.orientation(orientation);
        }
        if let Some((min_angle, max_angle)) = self.angular {
            builder = builder.angular(min_angle, max_angle);
        }
        if let Some(intervals) = self.allowed_intervals {
            builder = builder.allowed_intervals(intervals);
        }
        match self.presentation {
            Some(TrackPresentation::Fill) => builder = builder.fill(),
            Some(TrackPresentation::Domain) => builder = builder.domain(),
            None => {}
        }
        if let Some(policy) = self.thumb_policy {
            builder = builder.thumb_policy(policy);
        }
        if let Some(value) = self.value {
            builder = builder.value(value);
        }
        if let Some(values) = self.thumb_values {
            builder = builder.thumb_values(values);
        }
        for modifier in self.modifiers {
            builder = builder.with_template_modifier(move |root, model| (modifier)(root, model));
        }
        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_keeps_shadcn_axes() {
        let slider = Slider::new("gain").secondary().size(ShadcnSize::Lg).range(0.0f32..10.0).step(0.5).value(2.0);
        assert!(matches!(slider.style, ShadcnButtonStyle::Secondary));
        assert_eq!(slider.size, ShadcnSize::Lg);
        assert_eq!(slider.range, ControlRange::new(0.0, 10.0));
        assert_eq!(slider.step, 0.5);
        assert_eq!(slider.value, Some(2.0));
    }

    #[test]
    fn radius_keeps_shadcn_axis() {
        let slider = Slider::new("gain").radius(ButtonRadiusPreset::None);
        assert!(matches!(slider.radius, Some(ButtonRadiusPreset::None)));
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = Slider::new("ok")
            .look(&look)
            .primary()
            .radius(ButtonRadiusPreset::None)
            .range(0..100)
            .value(40)
            .with_template_modifier(|root, _| root)
            .into_sdk_builder(look);
    }
}
