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
    pub(super) dismiss: crate::infra::popup_dismiss::PopupDismiss,
    pub(super) explicit_offset_y: bool,
    pub(super) explicit_window_margin: bool,
    pub(super) offset_y: Pixels,
    pub(super) window_margin: Pixels,
    pub(super) initial_content_size: Option<gpui::Size<Pixels>>,
    pub(super) focus_on_open: bool,
    pub(super) motion_enabled: bool,
    pub(super) trigger_on_pointer_down: bool,
    pub(super) measure_trigger: bool,
    pub(super) panel_look: super::panel::PopoverPanelLook,
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
        let geometry = crate::theme::stylesheet::PopoverGeometry::sdk();
        Self {
            motion: Default::default(),
            callbacks: Default::default(),

            id: id.clone(),
            data: D::default(),
            trigger: Arc::new(|_, _| div().into_any_element()),
            content: Arc::new(|_, _, _| div().into_any_element()),
            placement: PopoverPlacement::default(),
            dismiss: crate::infra::popup_dismiss::PopupDismiss::CLICK_OUTSIDE,
            explicit_offset_y: false,
            explicit_window_margin: false,
            offset_y: px(geometry.button_offset_y),
            window_margin: px(geometry.window_margin),
            initial_content_size: None,
            focus_on_open: true,
            motion_enabled: true,
            trigger_on_pointer_down: true,
            measure_trigger: true,
            panel_look: super::panel::PopoverPanelLook::sdk(),
        }
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
    /// What closes the popover; an outside click or Escape by default.
    pub fn dismiss(mut self, value: crate::infra::popup_dismiss::PopupDismiss) -> Self {
        self.dismiss = value;
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
    /// Where the popover opens (a look's `popover.lss`); the SDK's by default. An explicit
    /// [`offset_y`](Self::offset_y) or [`window_margin`](Self::window_margin) still wins.
    pub fn with_geometry(mut self, geometry: crate::theme::stylesheet::PopoverGeometry) -> Self {
        if !self.explicit_offset_y {
            self.offset_y = px(geometry.button_offset_y);
        }
        if !self.explicit_window_margin {
            self.window_margin = px(geometry.window_margin);
        }
        self
    }
    /// The panel drawn around the content (a look's `popover.lss` `panel` part); off in the SDK.
    pub fn panel_look(mut self, look: super::panel::PopoverPanelLook) -> Self {
        self.panel_look = look;
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
    /// Placement defaults are `popover.lss`'s (#287), and explicit builder values replace them.
    #[test]
    fn button_placement_reads_popover_lss_and_explicit_values_win() {
        let sdk = |name| crate::style::sdk_number("popover", name);
        let baseline = PopoverButtonBuilder::<()>::new("baseline");
        assert_eq!((baseline.offset_y, baseline.window_margin), (px(sdk("button_offset_y")), px(sdk("window_margin"))));
        let explicit = PopoverButtonBuilder::<()>::new("explicit").offset_y(px(0.0)).window_margin(px(0.0));
        assert_eq!((explicit.offset_y, explicit.window_margin), (px(0.0), px(0.0)));
    }

    /// A look's geometry replaces the SDK's, but not an explicit builder value (#397).
    #[test]
    fn with_geometry_rebinds_placement_except_explicit_values() {
        let geometry = crate::theme::stylesheet::PopoverGeometry {
            button_offset_y: 6.0,
            panel_offset_y: 7.0,
            window_margin: 12.0,
        };
        let themed = PopoverButtonBuilder::<()>::new("themed").with_geometry(geometry);
        assert_eq!((themed.offset_y, themed.window_margin), (px(6.0), px(12.0)));
        let explicit = PopoverButtonBuilder::<()>::new("explicit").offset_y(px(1.0)).with_geometry(geometry);
        assert_eq!((explicit.offset_y, explicit.window_margin), (px(1.0), px(12.0)));
    }
}
