use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, IntoElement, Pixels, SharedString, Window, div, prelude::*, px};

use super::control::PopoverButton;

pub type PopoverTrigger<D> = Arc<dyn for<'a> Fn(&PopoverRenderModel<'a, D>, &mut App) -> AnyElement + 'static>;
pub type PopoverContent<D> = Arc<dyn Fn(&PopoverRenderModel<'_, D>, &mut Window, &mut App) -> AnyElement + 'static>;

#[derive(Clone, Debug)]
pub enum PopoverButtonEvent {
    OpenChanged { open: bool },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PopoverPlacement {
    #[default]
    Smart,
    BelowStart,
    BelowCenter,
    AboveStart,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PopoverDismissPolicy {
    #[default]
    CloseOnClickAway,
    CloseOnFocusLoss,
    CloseOnClickAwayOrFocusLoss,
    KeepOpen,
}

pub struct PopoverRenderModel<'a, D> {
    pub id: &'a SharedString,
    pub data: &'a D,
    pub open: bool,
}

pub struct PopoverButtonBuilder<D = ()> {
    callbacks: crate::infra::callbacks::EventCallbacks<crate::controls::popover_button::PopoverButtonEvent>,

    pub(super) motion: crate::motion::MotionConfig<super::PopoverMotion>,
    pub(super) id: SharedString,
    pub(super) data: D,
    pub(super) trigger: PopoverTrigger<D>,
    pub(super) content: PopoverContent<D>,
    pub(super) placement: PopoverPlacement,
    pub(super) dismiss_policy: PopoverDismissPolicy,
    explicit_offset_y: bool,
    explicit_window_margin: bool,
    pub(super) offset_y: Pixels,
    pub(super) window_margin: Pixels,
    pub(super) initial_content_size: Option<gpui::Size<Pixels>>,
    pub(super) focus_on_open: bool,
    pub(super) motion_enabled: bool,
    pub(super) trigger_on_pointer_down: bool,
    pub(super) measure_trigger: bool,
}

impl<D: Default + 'static> PopoverButtonBuilder<D> {
    crate::motion_methods!(super::PopoverMotion, motion);
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::popover_button::PopoverButtonEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `PopoverButtonEvent::OpenChanged` through the existing event stream.
    pub fn on_open_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            let crate::controls::popover_button::PopoverButtonEvent::OpenChanged { open } = event;
            callback(owner, *open, cx);
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();
        let geometry =
            crate::theme::stylesheet::CommonStylesheet::sdk_default().popover.authored_geometry_for("button");
        Self {
            motion: Default::default(),
            callbacks: Default::default(),

            id: id.clone(),
            data: D::default(),
            trigger: Arc::new(|_, _| div().into_any_element()),
            content: Arc::new(|_, _, _| div().into_any_element()),
            placement: PopoverPlacement::default(),
            dismiss_policy: PopoverDismissPolicy::default(),
            explicit_offset_y: false,
            explicit_window_margin: false,
            offset_y: px(geometry.offset_y.unwrap_or_default()),
            window_margin: px(geometry.window_margin.unwrap_or_default()),
            initial_content_size: None,
            focus_on_open: true,
            motion_enabled: true,
            trigger_on_pointer_down: true,
            measure_trigger: true,
        }
    }
    /// Capture placement defaults; explicit builder offsets and margins win in either order.
    pub fn stylesheet(mut self, stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        let baseline =
            crate::theme::stylesheet::CommonStylesheet::sdk_default().popover.authored_geometry_for("button");
        let geometry = stylesheet.popover.authored_geometry_for("button").with_fallback(&baseline);
        if !self.explicit_offset_y {
            self.offset_y = px(geometry.offset_y.unwrap_or_default());
        }
        if !self.explicit_window_margin {
            self.window_margin = px(geometry.window_margin.unwrap_or_default());
        }
        self
    }

    pub fn typed(mut self, data: D) -> Self {
        self.data = data;
        self
    }
    pub fn trigger<F>(mut self, trigger: F) -> Self
    where
        F: for<'a> Fn(&PopoverRenderModel<'a, D>, &mut App) -> AnyElement + 'static,
    {
        self.trigger = Arc::new(trigger);
        self
    }
    pub fn content<F>(mut self, content: F) -> Self
    where
        F: Fn(&PopoverRenderModel<'_, D>, &mut Window, &mut App) -> AnyElement + 'static,
    {
        self.content = Arc::new(content);
        self
    }
    pub fn placement(mut self, value: PopoverPlacement) -> Self {
        self.placement = value;
        self
    }
    pub fn dismiss_policy(mut self, value: PopoverDismissPolicy) -> Self {
        self.dismiss_policy = value;
        self
    }
    pub fn offset_y(mut self, value: impl Into<Pixels>) -> Self {
        self.explicit_offset_y = true;
        self.offset_y = value.into();
        self
    }
    pub fn window_margin(mut self, value: impl Into<Pixels>) -> Self {
        self.explicit_window_margin = true;
        self.window_margin = value.into();
        self
    }
    pub fn initial_content_size(mut self, value: gpui::Size<Pixels>) -> Self {
        self.initial_content_size = Some(value);
        self
    }
    pub fn focus_on_open(mut self, value: bool) -> Self {
        self.focus_on_open = value;
        self
    }
    pub fn motion_enabled(mut self, value: bool) -> Self {
        self.motion_enabled = value;
        self
    }
    pub fn trigger_on_pointer_down(mut self, value: bool) -> Self {
        self.trigger_on_pointer_down = value;
        self
    }
    /// Uses bounds supplied by the host instead of measuring the trigger child.
    pub fn measure_trigger(mut self, value: bool) -> Self {
        self.measure_trigger = value;
        self
    }
    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> Entity<PopoverButton<D>> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = { cx.new(|cx| PopoverButton::from_builder(self, cx)) };
        callbacks.bind(&control, cx);
        control
    }
}

#[cfg(test)]
mod stylesheet_tests {
    use super::*;
    use crate::theme::stylesheet::CommonStylesheet;

    #[test]
    fn button_placement_captures_defaults_and_explicit_values_win_in_either_order() {
        let baseline = PopoverButtonBuilder::<()>::new("baseline");
        assert_eq!((baseline.offset_y, baseline.window_margin), (px(0.0), px(8.0)));
        let mut sheet = CommonStylesheet::resolve_layers(&[
            "[common.popover.geometry]\nwindow_margin=19\n[common.popover.sizes.button]\noffset_y=6",
        ])
        .unwrap();
        let captured = PopoverButtonBuilder::<()>::new("captured").stylesheet(&sheet);
        sheet.popover.geometry.window_margin = Some(37.0);
        assert_eq!((captured.offset_y, captured.window_margin), (px(6.0), px(19.0)));
        for builder in [
            PopoverButtonBuilder::<()>::new("before")
                .offset_y(px(0.0))
                .window_margin(px(0.0))
                .stylesheet(&sheet),
            PopoverButtonBuilder::<()>::new("after").stylesheet(&sheet).offset_y(px(0.0)).window_margin(px(0.0)),
        ] {
            assert_eq!((builder.offset_y, builder.window_margin), (px(0.0), px(0.0)));
        }
    }
}
