use crate::infra::focus_report::ReportsFocus as _;
use gpui::{
    App, Context, DragMoveEvent, EventEmitter, FocusHandle, FocusOutEvent, Focusable, IntoElement, KeyDownEvent,
    MouseButton, MouseDownEvent, MouseUpEvent, Render, SharedString, Window, div, prelude::*,
};

use super::{
    DockSplitterBuilder, DockSplitterModel, DockSplitterRenderModel, DockSplitterTemplateHandlers, SplitterOrientation,
};
use crate::infra::state::ControlFocusState;
use crate::theme::observe_theme_revision;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum DockSplitterEvent {
    ResizeStart,
    Resize { total_delta: f32 },
    ResizeEnd,
    FocusChanged { focused: bool },
    HoverChanged { hovered: bool },
    EnabledChanged { enabled: bool },
}

#[derive(Clone, Debug)]
pub struct DockSplitterDrag {
    pub id: SharedString,
}

impl Render for DockSplitterDrag {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}

pub struct DockSplitter {
    model: DockSplitterModel,
    focus_handle: FocusHandle,
    hovered: bool,
    dragging: bool,
    focus_report: crate::infra::focus_report::FocusReport,
    focus_subscriptions: crate::infra::focus_report::FocusSubscriptions,
    drag_start_axis_px: f32,
}

impl EventEmitter<DockSplitterEvent> for DockSplitter {}

impl DockSplitter {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>, orientation: SplitterOrientation) -> DockSplitterBuilder {
        DockSplitterBuilder::new(id, orientation)
    }

    pub(crate) fn from_builder(builder: DockSplitterBuilder, cx: &mut Context<Self>) -> Self {
        let enabled = builder.model.enabled;

        observe_theme_revision(cx, |_, cx| cx.notify()).detach();
        Self {
            model: builder.model,
            focus_handle: cx.focus_handle().tab_stop(enabled),
            hovered: false,
            dragging: false,
            focus_report: Default::default(),
            focus_subscriptions: Default::default(),
            drag_start_axis_px: 0.0,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }
        self.model.enabled = enabled;
        self.focus_subscriptions.reset();
        if !enabled {
            if self.hovered {
                cx.emit(DockSplitterEvent::HoverChanged { hovered: false });
            }
            self.hovered = false;
            if self.dragging {
                cx.emit(DockSplitterEvent::ResizeEnd);
            }
            self.dragging = false;
            self.emit_focus_changed(false, cx);
        }
        cx.emit(DockSplitterEvent::EnabledChanged { enabled });
        cx.notify();
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn super::DockSplitterTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    pub fn set_theme(&mut self, theme: std::sync::Arc<dyn super::DockSplitterTheme>, cx: &mut Context<Self>) {
        self.model.theme = theme;
        cx.notify();
    }

    fn render_model<'a>(&'a self, window: &Window) -> DockSplitterRenderModel<'a> {
        let focus = ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window);

        DockSplitterRenderModel {
            id: &self.model.id,
            orientation: self.model.orientation,
            enabled: self.model.enabled,
            hovered: self.hovered,
            dragging: self.dragging,
            focused: focus.focused,
            focus_handle: &self.focus_handle,
        }
    }

    fn style(&self) -> std::sync::Arc<xgpui_luma_style::Resolved> {
        super::resolve_dock_splitter(&self.model.theme.style(), self.model.enabled)
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> DockSplitterTemplateHandlers {
        DockSplitterTemplateHandlers {
            hover: Box::new(cx.listener(Self::handle_hover)),
            mouse_down: Box::new(cx.listener(Self::handle_mouse_down)),
            mouse_up: Box::new(cx.listener(Self::handle_mouse_up)),
            mouse_up_out: Box::new(cx.listener(Self::handle_mouse_up)),
            key_down: Box::new(cx.listener(Self::handle_key_down)),
        }
    }

    fn axis_position(&self, position: gpui::Point<gpui::Pixels>) -> f32 {
        match self.model.orientation {
            SplitterOrientation::Vertical => position.x.as_f32(),
            SplitterOrientation::Horizontal => position.y.as_f32(),
        }
    }

    fn handle_hover(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if self.hovered == *hovered {
            return;
        }
        self.hovered = *hovered;
        cx.emit(DockSplitterEvent::HoverChanged { hovered: *hovered });
        cx.notify();
    }

    fn handle_focus_in(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.emit_focus_changed(true, cx) {
            cx.notify();
        }
    }

    fn handle_focus_out(&mut self, _: FocusOutEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.emit_focus_changed(false, cx) {
            cx.notify();
        }
    }

    fn handle_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || event.button != MouseButton::Left {
            return;
        }

        self.focus_handle.focus(window, cx);
        self.dragging = true;
        self.drag_start_axis_px = self.axis_position(event.position);
        cx.emit(DockSplitterEvent::ResizeStart);
        cx.notify();
    }

    fn handle_drag_move(
        &mut self,
        event: &DragMoveEvent<DockSplitterDrag>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.model.enabled || !self.dragging {
            return;
        }

        let drag = event.drag(cx);
        if drag.id != self.model.id {
            return;
        }

        let total_delta = self.axis_position(event.event.position) - self.drag_start_axis_px;
        if total_delta.abs() <= f32::EPSILON {
            return;
        }

        if let Some(on_resize) = &self.model.on_resize {
            on_resize(&total_delta, window, cx);
        }
        cx.emit(DockSplitterEvent::Resize { total_delta });
    }

    fn handle_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        let step_px = if event.keystroke.modifiers.shift {
            self.model.keyboard_shift_step
        } else {
            self.model.keyboard_step
        };

        let total_delta = match event.keystroke.key.as_str() {
            "left" | "up" => -step_px,
            "right" | "down" => step_px,
            _ => return,
        };

        self.focus_handle.focus(window, cx);
        cx.emit(DockSplitterEvent::ResizeStart);
        if let Some(on_resize) = &self.model.on_resize {
            on_resize(&total_delta, window, cx);
        }
        cx.emit(DockSplitterEvent::Resize { total_delta });
        cx.emit(DockSplitterEvent::ResizeEnd);
        window.prevent_default();
        cx.stop_propagation();
        cx.notify();
    }

    fn handle_mouse_up(&mut self, event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if event.button != MouseButton::Left || !self.dragging {
            return;
        }

        self.dragging = false;
        cx.emit(DockSplitterEvent::ResizeEnd);
        cx.notify();
    }
}

impl Focusable for DockSplitter {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DockSplitter {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle.clone();
        self.focus_subscriptions
            .ensure(&focus_handle, window, cx, Self::handle_focus_in, Self::handle_focus_out);

        let model = self.render_model(window);
        let resolved = self.style();
        let look = resolved.root();
        let handlers = self.template_handlers(cx);

        div()
            .relative()
            .on_drag_move(cx.listener(Self::handle_drag_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::handle_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::handle_mouse_up))
            .child(self.model.template.render(&model, look, handlers, window, cx))
            .into_any_element()
    }
}

impl crate::infra::focus_report::ReportsFocus for DockSplitter {
    type FocusEvent = DockSplitterEvent;

    fn focus_report(&mut self) -> &mut crate::infra::focus_report::FocusReport {
        &mut self.focus_report
    }

    fn focus_event(focused: bool) -> DockSplitterEvent {
        DockSplitterEvent::FocusChanged { focused }
    }

    fn focus_enabled(&self) -> bool {
        self.model.enabled
    }
}
