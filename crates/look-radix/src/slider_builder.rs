//! Look-owned slider builder. Spawn synthesizes the SDK [`xgpui_luma::controls::slider::Slider`].

use gpui::{App, Context, Div, SharedString, Stateful};
use xgpui_luma::controls::slider::{
    SliderBuilder, SliderOrientation, SliderRenderModel, SliderTemplateModifier, SliderThumbPolicy, TrackPresentation,
};
use xgpui_luma::infra::value::ControlRange;

use crate::look::{Look, resolve_look};
use crate::slider::{SliderSize, SliderVariant, slider_template_for};

/// Builder in the guise of a slider: Radix axes plus SDK options, until `.spawn(cx)`.
pub struct Slider {
    motion: xgpui_luma::motion::MotionOverrides<xgpui_luma::controls::slider::SliderMotion>,
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::slider::SliderEvent>,

    id: SharedString,
    look: Option<Look>,
    variant: SliderVariant,
    size: SliderSize,
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
            variant: SliderVariant::default(),
            size: SliderSize::default(),
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
            modifiers: Vec::new(),
        }
    }

    xgpui_luma::forward_motion!(xgpui_luma::controls::slider::SliderMotion, edits motion);

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn variant(mut self, variant: SliderVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn classic(self) -> Self {
        self.variant(SliderVariant::Classic)
    }

    pub fn surface(self) -> Self {
        self.variant(SliderVariant::Surface)
    }

    pub fn soft(self) -> Self {
        self.variant(SliderVariant::Soft)
    }

    pub fn size(mut self, size: SliderSize) -> Self {
        self.size = size;
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
            let theme = crate::tooltip::tooltip_theme(&look);
            let entity = self.into_sdk_builder(look).spawn(cx);
            entity.update(cx, |control, _| {
                xgpui_luma::infra::attachments::AttachmentTarget::attachments_mut(control).set_tooltip_theme(theme);
            });
            entity
        };
        callbacks.bind(&control, cx);
        control
    }

    fn resolve_look(&self, cx: &App) -> Look {
        resolve_look(self.look.as_ref(), cx.try_global::<Look>())
    }

    fn into_sdk_builder(self, look: Look) -> SliderBuilder {
        let template = slider_template_for(&look, self.variant, self.size);
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
        if let Some(orientation) = self.orientation {
            builder = builder.orientation(orientation);
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
        for modifier in self.modifiers {
            builder = builder.with_template_modifier(move |root, model| (modifier)(root, model));
        }
        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xgpui_luma::theme::ControlSize;

    #[test]
    fn size_maps_to_sdk_control_size() {
        assert_eq!(Slider::new("s").size(SliderSize::One).size.control_size(), ControlSize::Sm);
        assert_eq!(Slider::new("s").size(SliderSize::Two).size.control_size(), ControlSize::Md);
        assert_eq!(Slider::new("s").size(SliderSize::Three).size.control_size(), ControlSize::Lg);
    }

    #[test]
    fn value_keeps_radix_axes() {
        let slider = Slider::new("gain").soft().size(SliderSize::Three).range(0.0f32..10.0).step(0.5).value(2.0);
        assert!(matches!(slider.variant, SliderVariant::Soft));
        assert_eq!(slider.size, SliderSize::Three);
        assert_eq!(slider.range, ControlRange::new(0.0, 10.0));
        assert_eq!(slider.step, 0.5);
        assert_eq!(slider.value, Some(2.0));
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = Look::built_in();
        let _builder = Slider::new("ok")
            .look(&look)
            .classic()
            .range(0..100)
            .value(40)
            .with_template_modifier(|root, _| root)
            .into_sdk_builder(look);
    }
}
