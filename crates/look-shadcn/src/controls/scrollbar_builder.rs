//! Look-owned scrollbar builder. Spawn synthesizes the SDK [`xgpui_luma::controls::scrollbar::Scrollbar`].

use gpui::{Context, Entity, SharedString};
use xgpui_luma::controls::scrollbar::{ScrollbarBuilder, ScrollbarOrientation, ScrollbarStyle};
use xgpui_luma::infra::value::ControlRange;
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

/// Builder in the guise of a scrollbar: Shadcn template plus SDK options, until `.spawn(cx)`.
pub struct Scrollbar {
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::scrollbar::ScrollbarEvent>,

    look: Option<ShadcnLook>,
    builder: ScrollbarBuilder,
    custom_template: bool,
    size: ShadcnSize,
}

impl Scrollbar {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::scrollbar::ScrollbarEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `ScrollbarEvent::Change` through the existing event stream.
    pub fn on_value_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, f32, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::scrollbar::ScrollbarEvent::Change { value } = event {
                callback(owner, *value, cx);
            }
        })
    }

    /// Handle `ScrollbarEvent::DragStart` through the existing event stream.
    pub fn on_drag_start<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::scrollbar::ScrollbarEvent::DragStart = event {
                callback(owner, cx);
            }
        })
    }

    /// Handle `ScrollbarEvent::DragEnd` through the existing event stream.
    pub fn on_drag_end<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, f32, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::scrollbar::ScrollbarEvent::DragEnd { value } = event {
                callback(owner, *value, cx);
            }
        })
    }

    /// Handle `ScrollbarEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::scrollbar::ScrollbarEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `ScrollbarEvent::HoverChanged` through the existing event stream.
    pub fn on_hover_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::scrollbar::ScrollbarEvent::HoverChanged { hovered } = event {
                callback(owner, *hovered, cx);
            }
        })
    }

    /// Handle `ScrollbarEvent::EnabledChanged` through the existing event stream.
    pub fn on_enabled_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::scrollbar::ScrollbarEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            look: None,
            builder: xgpui_luma::controls::scrollbar::Scrollbar::new(id),
            custom_template: false,
            size: ShadcnSize::Md,
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn orientation(mut self, orientation: ScrollbarOrientation) -> Self {
        self.builder = self.builder.orientation(orientation);
        self
    }

    pub fn horizontal(mut self) -> Self {
        self.builder = self.builder.horizontal();
        self
    }

    pub fn vertical(mut self) -> Self {
        self.builder = self.builder.vertical();
        self
    }

    pub fn range(mut self, range: impl Into<ControlRange>) -> Self {
        self.builder = self.builder.range(range);
        self
    }

    pub fn step(mut self, step: impl Into<f64>) -> Self {
        self.builder = self.builder.step(step);
        self
    }

    pub fn page_step(mut self, page_step: impl Into<f64>) -> Self {
        self.builder = self.builder.page_step(page_step);
        self
    }

    pub fn value(mut self, value: impl Into<f64>) -> Self {
        self.builder = self.builder.value(value);
        self
    }

    pub fn thumb_fraction(mut self, thumb_fraction: impl Into<f64>) -> Self {
        self.builder = self.builder.thumb_fraction(thumb_fraction);
        self
    }

    pub fn length(mut self, length: impl Into<f64>) -> Self {
        self.builder = self.builder.length(length);
        self
    }

    pub fn size(mut self, size: ShadcnSize) -> Self {
        self.size = size;
        self
    }

    pub fn style(mut self, style: ScrollbarStyle) -> Self {
        self.builder = self.builder.style(style);
        self
    }

    pub fn ghost(mut self) -> Self {
        self.builder = self.builder.ghost();
        self
    }

    pub fn soft(mut self) -> Self {
        self.builder = self.builder.soft();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }

    pub fn template(
        mut self,
        template: std::sync::Arc<dyn xgpui_luma::controls::scrollbar::ScrollbarTemplate>,
    ) -> Self {
        self.custom_template = true;
        self.builder = self.builder.template(template);
        self
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> Entity<xgpui_luma::controls::scrollbar::Scrollbar> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = {
            let look = resolve_look_from(self.look.as_ref(), cx);
            self.into_sdk_builder(look).spawn(cx)
        };
        callbacks.bind(&control, cx);
        control
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> ScrollbarBuilder {
        let builder = self.builder.size(self.size.control_size());
        if self.custom_template {
            builder
        } else {
            builder.template(look.scrollbar_template())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = Scrollbar::new("ok").look(&look).vertical().range(0..100).into_sdk_builder(look);
    }
}
