use gpui::{
    anchored, deferred, point, Anchor, App, Context, DragMoveEvent, EventEmitter, FocusHandle, Focusable, IntoElement,
    KeyDownEvent, MouseButton, MouseDownEvent, MouseUpEvent, Pixels, Point, Render, SharedString, Size, Subscription,
    Window, div, prelude::*, px,
};

use super::{
    OverlayWindowBuilder, OverlayWindowEvent, OverlayWindowMode, OverlayWindowPosition, OverlayWindowRenderModel,
    OverlayWindowTemplateHandlers, OverlayWindowTemplateParts,
};
use super::model::ThemeInvalidator;
use crate::infra::popup_dismiss::PopupDismiss;
use crate::theme::observe_theme_revision;

#[derive(Clone)]
struct OverlayWindowHeaderDrag {
    id: SharedString,
}

impl Render for OverlayWindowHeaderDrag {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}

pub struct OverlayWindowControl {
    model: super::OverlayWindowModel,
    focus_handle: FocusHandle,
    restore_focus: Option<FocusHandle>,
    pending_focus: bool,
    pending_focus_restore: bool,
    dragging: Option<OverlayWindowDragState>,
    theme_children: Vec<ThemeInvalidator>,
    _subscriptions: Vec<Subscription>,
}

#[derive(Clone, Copy)]
struct OverlayWindowDragState {
    pointer_origin: Point<Pixels>,
    overlay_origin: Point<Pixels>,
}

type OverlayWindowMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
type OverlayWindowMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;

impl EventEmitter<OverlayWindowEvent> for OverlayWindowControl {}

impl OverlayWindowControl {
    pub(crate) fn from_builder(builder: OverlayWindowBuilder, cx: &mut Context<Self>) -> Self {
        observe_theme_revision(cx, |this, cx| {
            this.invalidate_theme_children(cx);
            cx.notify();
        })
        .detach();

        Self {
            focus_handle: cx.focus_handle().tab_stop(true),
            model: builder.model,
            restore_focus: None,
            pending_focus: false,
            pending_focus_restore: false,
            dragging: None,
            theme_children: builder.theme_children,
            _subscriptions: Vec::new(),
        }
    }

    fn invalidate_theme_children(&self, cx: &mut Context<Self>) {
        for child in &self.theme_children {
            child.invalidate(cx);
        }
    }

    pub fn open(&mut self, cx: &mut Context<Self>) {
        self.open_from(None, cx);
    }

    pub fn open_from(&mut self, opener: Option<FocusHandle>, cx: &mut Context<Self>) {
        if let Some(opener) = opener {
            self.restore_focus = Some(opener);
        }
        if self.model.open {
            return;
        }
        self.model.open = true;
        self.pending_focus = true;
        self.pending_focus_restore = false;
        cx.emit(OverlayWindowEvent::Opened);
        cx.notify();
    }

    pub fn dismiss(&mut self, cx: &mut Context<Self>) {
        if !self.model.open {
            return;
        }
        self.model.open = false;
        self.pending_focus = false;
        self.pending_focus_restore = self.restore_focus.is_some();
        self.dragging = None;
        cx.emit(OverlayWindowEvent::Dismissed);
        cx.notify();
    }

    pub fn set_mode(&mut self, mode: OverlayWindowMode, cx: &mut Context<Self>) {
        self.model.mode = mode;
        cx.notify();
    }

    pub fn set_position(&mut self, position: OverlayWindowPosition, cx: &mut Context<Self>) {
        self.model.position = position;
        cx.notify();
    }

    pub fn set_dismiss(&mut self, dismiss: PopupDismiss, cx: &mut Context<Self>) {
        self.model.dismiss = Some(dismiss);
        cx.notify();
    }

    pub fn set_draggable(&mut self, draggable: bool, cx: &mut Context<Self>) {
        self.model.draggable = draggable;
        cx.notify();
    }

    pub fn is_open(&self) -> bool {
        self.model.open
    }

    fn render_model<'a>(&'a self, window: &Window) -> OverlayWindowRenderModel<'a> {
        OverlayWindowRenderModel {
            id: &self.model.id,
            mode: self.model.mode,
            position: self.model.position,
            dismiss: self.model.dismiss(),
            draggable: self.model.draggable,
            width: self.model.width,
            focused: self.focus_handle.is_focused(window),
            content: &self.model.content,
        }
    }

    fn has_internal_focus(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }

    fn handle_shell_mouse_down(&mut self, _event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_handle.focus(window, cx);
        cx.stop_propagation();
        if self.model.dismiss().focus_loss {
            self.pending_focus = false;
        }
    }

    fn handle_shell_mouse_down_out(&mut self, _event: &MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.model.mode == OverlayWindowMode::Modeless && self.model.dismiss().click_outside {
            self.dismiss(cx);
        }
    }

    fn handle_backdrop_mouse_down(&mut self, _event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.model.dismiss().click_outside {
            window.prevent_default();
            cx.stop_propagation();
            self.dismiss(cx);
        }
    }

    fn handle_header_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_handle.focus(window, cx);
        if !self.model.draggable {
            return;
        }

        let viewport = window.viewport_size();
        let look = self.model.template.resolve_look(self.model.mode);
        let origin = resolve_overlay_window_origin(self.model.position, viewport, &look, self.model.width);
        self.dragging = Some(OverlayWindowDragState { pointer_origin: event.position, overlay_origin: origin });
    }

    fn handle_drag_move(
        &mut self,
        event: &DragMoveEvent<OverlayWindowHeaderDrag>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.drag(cx).id != self.model.id {
            return;
        }
        let Some(dragging) = self.dragging else {
            return;
        };
        let delta = event.event.position - dragging.pointer_origin;
        let origin = point(dragging.overlay_origin.x + delta.x, dragging.overlay_origin.y + delta.y);
        let look = self.model.template.resolve_look(self.model.mode);
        self.model.position = OverlayWindowPosition::Absolute(clamp_overlay_window_origin(
            origin,
            _window.viewport_size(),
            &look,
            self.model.width,
        ));
        cx.notify();
    }

    fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, _cx: &mut Context<Self>) {
        self.dragging = None;
    }

    fn handle_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if event.keystroke.key.as_str() == "escape" && self.model.dismiss().escape {
            window.prevent_default();
            cx.stop_propagation();
            self.dismiss(cx);
        }
    }

    fn sync_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.pending_focus && self.model.open {
            self.pending_focus = false;
            let focus = self.focus_handle.clone();
            cx.on_next_frame(window, move |_this, window, cx| {
                focus.focus(window, cx);
            });
        }

        if self.pending_focus_restore && !self.model.open {
            self.pending_focus_restore = false;
            if let Some(focus) = self.restore_focus.clone() {
                cx.on_next_frame(window, move |_this, window, cx| {
                    focus.focus(window, cx);
                });
            }
        }
    }
}

impl Focusable for OverlayWindowControl {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for OverlayWindowControl {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_focus(window, cx);

        if self.model.open
            && self.model.mode == OverlayWindowMode::Modeless
            && self.model.dismiss().focus_loss
            && !self.pending_focus
            && !self.has_internal_focus(window)
        {
            self.dismiss(cx);
        }

        if !self.model.open {
            return div().into_any_element();
        }

        let look = self.model.template.resolve_look(self.model.mode);
        let model = self.render_model(window);
        let header_drag_handle = self.model.draggable.then(|| {
            div()
                .id("drag-header")
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .h(px(look.drag_header_height))
                .occlude()
                .cursor_grab()
                .on_mouse_down(MouseButton::Left, cx.listener(Self::handle_header_mouse_down))
                .on_drag(OverlayWindowHeaderDrag { id: self.model.id.clone() }, |drag, _, _, cx| {
                    cx.stop_propagation();
                    cx.new(|_| drag.clone())
                })
                .into_any_element()
        });

        let shell = self.model.template.render(
            &model,
            &look,
            OverlayWindowTemplateParts {
                header_drag_handle,
                handlers: OverlayWindowTemplateHandlers {
                    key_down: Box::new(cx.listener(Self::handle_key_down)),
                    header_mouse_down: Box::new(cx.listener(Self::handle_header_mouse_down)),
                    shell_mouse_down: Box::new(cx.listener(Self::handle_shell_mouse_down)),
                    shell_mouse_down_out: Box::new(cx.listener(Self::handle_shell_mouse_down_out)),
                },
            },
            window,
            cx,
        );

        let shell = shell
            .track_focus(&self.focus_handle)
            .on_drag_move(cx.listener(Self::handle_drag_move))
            .into_any_element();

        let overlay = match self.model.mode {
            OverlayWindowMode::Modal => render_modal_overlay(
                &self.model.id,
                shell,
                &look,
                self.model.position,
                window.viewport_size(),
                Box::new(cx.listener(Self::handle_backdrop_mouse_down)),
            ),
            OverlayWindowMode::Modeless => render_modeless_overlay(
                &self.model.id,
                shell,
                self.model.position,
                &look,
                self.model.width,
                window.viewport_size(),
                self.dragging.is_some(),
                Box::new(cx.listener(Self::handle_mouse_up)),
                Box::new(cx.listener(Self::handle_mouse_up)),
            ),
        };

        div().child(deferred(overlay).with_priority(10)).into_any_element()
    }
}

fn render_modal_overlay(
    id: &SharedString,
    shell: gpui::AnyElement,
    look: &super::OverlayWindowLook,
    position: OverlayWindowPosition,
    viewport: Size<Pixels>,
    backdrop_mouse_down: OverlayWindowMouseDownHandler,
) -> gpui::AnyElement {
    let content = render_modal_content(position, shell, look.viewport_padding);

    anchored()
        .snap_to_window_with_margin(px(0.0))
        .anchor(Anchor::TopLeft)
        .position(point(px(0.0), px(0.0)))
        .child(
            div()
                .id((id.clone(), 0usize))
                .relative()
                .w(viewport.width)
                .h(viewport.height)
                .on_scroll(|_event, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                })
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .bg(look.backdrop_background)
                        .on_mouse_down(MouseButton::Left, backdrop_mouse_down),
                )
                .child(content),
        )
        .into_any_element()
}

fn render_modal_content(
    position: OverlayWindowPosition,
    shell: gpui::AnyElement,
    viewport_padding: f32,
) -> gpui::AnyElement {
    match position {
        OverlayWindowPosition::Absolute(point) => div()
            .relative()
            .size_full()
            .child(div().absolute().left(point.x).top(point.y).child(shell))
            .into_any_element(),
        OverlayWindowPosition::Center => div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .p(px(viewport_padding))
            .child(shell)
            .into_any_element(),
        OverlayWindowPosition::Top => div()
            .size_full()
            .flex()
            .items_center()
            .justify_start()
            .p(px(viewport_padding))
            .child(shell)
            .into_any_element(),
        OverlayWindowPosition::Bottom => div()
            .size_full()
            .flex()
            .items_center()
            .justify_end()
            .p(px(viewport_padding))
            .child(shell)
            .into_any_element(),
        OverlayWindowPosition::Left => div()
            .size_full()
            .flex()
            .items_start()
            .justify_center()
            .p(px(viewport_padding))
            .child(shell)
            .into_any_element(),
        OverlayWindowPosition::Right => div()
            .size_full()
            .flex()
            .items_end()
            .justify_center()
            .p(px(viewport_padding))
            .child(shell)
            .into_any_element(),
        OverlayWindowPosition::TopLeft => div()
            .size_full()
            .flex()
            .items_start()
            .justify_start()
            .p(px(viewport_padding))
            .child(shell)
            .into_any_element(),
        OverlayWindowPosition::TopRight => div()
            .size_full()
            .flex()
            .items_end()
            .justify_start()
            .p(px(viewport_padding))
            .child(shell)
            .into_any_element(),
        OverlayWindowPosition::BottomLeft => div()
            .size_full()
            .flex()
            .items_start()
            .justify_end()
            .p(px(viewport_padding))
            .child(shell)
            .into_any_element(),
        OverlayWindowPosition::BottomRight => div()
            .size_full()
            .flex()
            .items_end()
            .justify_end()
            .p(px(viewport_padding))
            .child(shell)
            .into_any_element(),
    }
}

#[allow(clippy::too_many_arguments)]
fn render_modeless_overlay(
    id: &SharedString,
    shell: gpui::AnyElement,
    position: OverlayWindowPosition,
    look: &super::OverlayWindowLook,
    width: Option<f32>,
    viewport: Size<Pixels>,
    dragging: bool,
    mouse_up: OverlayWindowMouseUpHandler,
    mouse_up_out: OverlayWindowMouseUpHandler,
) -> gpui::AnyElement {
    let origin = resolve_overlay_window_origin(position, viewport, look, width);
    anchored()
        .snap_to_window_with_margin(px(look.window_margin))
        .anchor(Anchor::TopLeft)
        .position(point(px(0.0), px(0.0)))
        .child(
            div()
                .id((id.clone(), 1usize))
                .relative()
                .w(viewport.width)
                .h(viewport.height)
                .when(dragging, |root| {
                    root.child(div().absolute().inset_0().occlude().on_mouse_down(
                        MouseButton::Left,
                        |_event, window, cx| {
                            window.prevent_default();
                            cx.stop_propagation();
                        },
                    ))
                })
                .on_mouse_up(MouseButton::Left, move |event, window, cx| mouse_up(event, window, cx))
                .on_mouse_up_out(MouseButton::Left, move |event, window, cx| mouse_up_out(event, window, cx))
                .child(div().absolute().left(origin.x).top(origin.y).child(shell)),
        )
        .into_any_element()
}

fn resolve_overlay_window_origin(
    position: OverlayWindowPosition,
    viewport: Size<Pixels>,
    look: &super::OverlayWindowLook,
    width: Option<f32>,
) -> Point<Pixels> {
    let width = px(match width {
        Some(width) => width.max(look.min_width),
        None => look.min_width.clamp(look.min_width, look.max_width),
    });
    let height = px(look.estimated_height);
    let inset = px(look.viewport_padding);

    match position {
        OverlayWindowPosition::Center => point((viewport.width - width) * 0.5, (viewport.height - height) * 0.5),
        OverlayWindowPosition::Top => point((viewport.width - width) * 0.5, inset),
        OverlayWindowPosition::Bottom => point((viewport.width - width) * 0.5, viewport.height - height - inset),
        OverlayWindowPosition::Left => point(inset, (viewport.height - height) * 0.5),
        OverlayWindowPosition::Right => point(viewport.width - width - inset, (viewport.height - height) * 0.5),
        OverlayWindowPosition::TopLeft => point(inset, inset),
        OverlayWindowPosition::TopRight => point(viewport.width - width - inset, inset),
        OverlayWindowPosition::BottomLeft => point(inset, viewport.height - height - inset),
        OverlayWindowPosition::BottomRight => point(viewport.width - width - inset, viewport.height - height - inset),
        OverlayWindowPosition::Absolute(point) => point,
    }
}

fn clamp_overlay_window_origin(
    origin: Point<Pixels>,
    viewport: Size<Pixels>,
    look: &super::OverlayWindowLook,
    width: Option<f32>,
) -> Point<Pixels> {
    let width = px(width.unwrap_or(look.min_width).max(look.min_width));
    let height = px(look.estimated_height);
    let inset = px(look.window_margin);
    let max_x = (viewport.width - width - inset).max(inset);
    let max_y = (viewport.height - height - inset).max(inset);
    point(origin.x.clamp(inset, max_x), origin.y.clamp(inset, max_y))
}

#[cfg(test)]
mod geometry_tests {
    use super::*;
    #[test]
    fn placement_and_drag_clamp_use_the_resolved_look() {
        let mut look = super::super::default_overlay_window_theme().resolve(OverlayWindowMode::Modeless);
        look.min_width = 100.0;
        look.max_width = 100.0;
        look.estimated_height = 80.0;
        look.viewport_padding = 31.0;
        look.window_margin = 5.0;
        let viewport = gpui::size(px(300.0), px(200.0));
        assert_eq!(
            resolve_overlay_window_origin(OverlayWindowPosition::TopLeft, viewport, &look, None),
            point(px(31.0), px(31.0))
        );
        assert_eq!(
            clamp_overlay_window_origin(point(px(999.0), px(999.0)), viewport, &look, None),
            point(px(195.0), px(115.0))
        );
    }
}
