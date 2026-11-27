use gpui::{
    anchored, deferred, App, Bounds, Context, EventEmitter, FocusHandle, FocusOutEvent, Focusable, IntoElement,
    MouseButton, MouseDownEvent, Pixels, Render, SharedString, Size, Subscription, Window, div, prelude::*, px,
};

use super::model::{PopoverPanelBuilder, PopoverPanelPlacement, PopoverPanelRenderModel};
use crate::infra::popup_dismiss::DismissReason;
use crate::infra::ElementExt;
use crate::infra::popup_placement::{PopupAlign, PopupPlacementRequest, PopupSide, PopupSpace, ResolvedPopupPlacement};
use crate::motion::overlay_presence::OverlayPresence;
use crate::focus::EscapeFocus;

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PopoverPanelEvent {
    OpenChanged { open: bool },
    Dismiss,
    FocusChanged { focused: bool },
}

pub struct PopoverPanel {
    model: super::model::PopoverPanelModel,
    focus_handle: FocusHandle,
    content_bounds: Option<Bounds<Pixels>>,
    emitted_focused: bool,
    focus_out_subscription: Option<Subscription>,
    pending_focus: bool,
    restore_focus: Option<FocusHandle>,
    pending_focus_restore: bool,
    dismiss_guard: bool,
    presence: OverlayPresence,
}

impl EventEmitter<PopoverPanelEvent> for PopoverPanel {}

impl PopoverPanel {
    crate::motion_snapshot!(super::PopoverMotion, model.motion);

    pub(crate) fn from_builder(builder: PopoverPanelBuilder, cx: &mut Context<Self>) -> Self {
        let mut builder = builder;
        builder.model.motion.freeze();
        let motion = builder.model.motion.profile().clone();
        let presence =
            OverlayPresence::with_motion(builder.model.open, builder.model.motion_enabled, motion.presence.clone());
        Self {
            pending_focus: builder.model.open && builder.model.focus_on_open,
            presence,
            model: builder.model,
            focus_handle: cx.focus_handle().tab_stop(true),
            content_bounds: None,
            emitted_focused: false,
            focus_out_subscription: None,
            restore_focus: None,
            pending_focus_restore: false,
            dismiss_guard: false,
        }
    }

    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> PopoverPanelBuilder {
        PopoverPanelBuilder::new(id)
    }

    pub fn is_open(&self) -> bool {
        self.model.open
    }

    pub fn open(&mut self, cx: &mut Context<Self>) {
        self.open_from(None, cx);
    }

    pub fn open_from(&mut self, opener: Option<FocusHandle>, cx: &mut Context<Self>) {
        if let Some(opener) = opener {
            self.restore_focus = Some(opener);
        }
        self.set_open(true, cx);
    }

    pub fn open_guarded(&mut self, cx: &mut Context<Self>) {
        self.open_guarded_from(None, cx);
    }

    pub fn open_guarded_from(&mut self, opener: Option<FocusHandle>, cx: &mut Context<Self>) {
        if let Some(opener) = opener {
            self.restore_focus = Some(opener);
        }
        self.dismiss_guard = true;
        self.set_open(true, cx);
    }

    pub fn close(&mut self, cx: &mut Context<Self>) {
        self.set_open(false, cx);
    }

    pub fn dismiss(&mut self, cx: &mut Context<Self>) {
        if self.close_internal(cx) {
            cx.emit(PopoverPanelEvent::Dismiss);
            cx.notify();
        }
    }

    pub fn toggle_guarded(&mut self, cx: &mut Context<Self>) {
        self.toggle_guarded_from(None, cx);
    }

    pub fn toggle_guarded_from(&mut self, opener: Option<FocusHandle>, cx: &mut Context<Self>) {
        if self.model.open {
            self.dismiss(cx);
        } else {
            self.open_guarded_from(opener, cx);
        }
    }

    pub fn set_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if open {
            self.open_internal(cx);
        } else {
            self.close_internal(cx);
        }
        cx.notify();
    }

    pub fn set_motion_enabled(&mut self, motion_enabled: bool, cx: &mut Context<Self>) {
        if self.model.motion_enabled == motion_enabled {
            return;
        }
        self.model.motion_enabled = motion_enabled;
        self.presence.set_motion_enabled(motion_enabled);
        cx.notify();
    }

    pub fn set_anchor_bounds(&mut self, bounds: Bounds<Pixels>, cx: &mut Context<Self>) {
        if self.model.anchor_bounds == Some(bounds) {
            return;
        }
        self.model.anchor_bounds = Some(bounds);
        cx.notify();
    }

    pub fn clear_anchor_bounds(&mut self, cx: &mut Context<Self>) {
        if self.model.anchor_bounds.take().is_some() {
            cx.notify();
        }
    }

    pub fn set_placement(&mut self, placement: PopoverPanelPlacement, cx: &mut Context<Self>) {
        if self.model.placement == placement {
            return;
        }
        self.model.placement = placement;
        cx.notify();
    }

    pub fn set_offset_y(&mut self, offset_y: Pixels, cx: &mut Context<Self>) {
        if self.model.offset_y == offset_y {
            return;
        }
        self.model.offset_y = offset_y;
        cx.notify();
    }

    /// Rebind the panel drawn around the content (a look's `popover.lss`).
    pub fn set_look(&mut self, look: super::PopoverPanelLook, cx: &mut Context<Self>) {
        if self.model.look != look {
            self.model.look = look;
            cx.notify();
        }
    }

    pub fn set_window_margin(&mut self, window_margin: Pixels, cx: &mut Context<Self>) {
        if self.model.window_margin == window_margin {
            return;
        }
        self.model.window_margin = window_margin;
        cx.notify();
    }

    pub fn set_initial_content_size(&mut self, size: Size<Pixels>, cx: &mut Context<Self>) {
        if self.model.initial_content_size == Some(size) {
            return;
        }
        self.model.initial_content_size = Some(size);
        cx.notify();
    }

    fn render_model<'a>(&'a self, window: &Window) -> PopoverPanelRenderModel<'a> {
        PopoverPanelRenderModel {
            id: &self.model.id,
            open: self.model.open,
            anchor_bounds: self.model.anchor_bounds,
            placement: self.model.placement,
            dismiss: self.model.dismiss,
            focused: self.focus_handle.is_focused(window),
            content_bounds: self.content_bounds,
        }
    }

    fn open_internal(&mut self, cx: &mut Context<Self>) -> bool {
        let was_open = self.model.open;
        self.model.open = true;
        self.pending_focus_restore = false;
        self.presence.set_open(true);
        if self.model.focus_on_open {
            self.pending_focus = true;
        }
        if !was_open {
            cx.emit(PopoverPanelEvent::OpenChanged { open: true });
        }
        !was_open
    }

    fn close_internal(&mut self, cx: &mut Context<Self>) -> bool {
        let was_open = self.model.open;
        self.model.open = false;
        self.pending_focus = false;
        self.pending_focus_restore = self.restore_focus.is_some();
        self.dismiss_guard = false;
        self.presence.set_open(false);
        if was_open {
            cx.emit(PopoverPanelEvent::OpenChanged { open: false });
        }
        was_open
    }

    fn can_dismiss_on_click_away(&self) -> bool {
        self.model.dismiss.allows(DismissReason::ClickOutside)
    }

    fn can_dismiss_on_focus_loss(&self) -> bool {
        self.model.dismiss.allows(DismissReason::FocusLoss)
    }

    fn emit_focus_changed(&mut self, focused: bool, cx: &mut Context<Self>) {
        if self.emitted_focused == focused {
            return;
        }
        self.emitted_focused = focused;
        cx.emit(PopoverPanelEvent::FocusChanged { focused });
    }

    fn handle_focus_out(&mut self, _: FocusOutEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.emit_focus_changed(false, cx);
        if self.dismiss_guard {
            self.dismiss_guard = false;
            cx.notify();
            return;
        }
        if !window.is_window_active() || !self.can_dismiss_on_focus_loss() {
            cx.notify();
            return;
        }
        self.dismiss(cx);
    }

    fn handle_shell_mouse_down(&mut self, _event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_handle.focus(window, cx);
        self.emit_focus_changed(true, cx);
        cx.stop_propagation();
    }

    fn handle_mouse_move(&mut self, _: &gpui::MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        // An open anchored panel owns hover tracking while the pointer is over
        // its content. Do not let underlying controls react to the same move.
        cx.stop_propagation();
    }

    fn handle_mouse_down_out(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let inside_anchor = self.model.anchor_bounds.is_some_and(|bounds| bounds.contains(&event.position));
        let inside_content = self.content_bounds.is_some_and(|bounds| bounds.contains(&event.position));
        if inside_anchor || inside_content {
            return;
        }
        if self.dismiss_guard {
            self.dismiss_guard = false;
            cx.notify();
            return;
        }
        if !window.is_window_active() || !self.can_dismiss_on_click_away() {
            return;
        }
        self.dismiss(cx);
    }

    fn handle_content_bounds(&mut self, bounds: Bounds<Pixels>, cx: &mut Context<Self>) {
        if bounds.size.width <= px(0.0) || self.content_bounds == Some(bounds) {
            return;
        }
        self.content_bounds = Some(bounds);
        cx.notify();
    }

    fn handle_escape_focus(&mut self, _: &EscapeFocus, _window: &mut Window, cx: &mut Context<Self>) {
        if self.model.open && self.model.dismiss.allows(DismissReason::Escape) {
            self.dismiss(cx);
        } else {
            cx.propagate();
        }
    }

    fn sync_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.pending_focus && self.model.open && self.model.anchor_bounds.is_some() {
            self.pending_focus = false;
            let focus = self.focus_handle.clone();
            cx.on_next_frame(window, move |this, window, cx| {
                if this.model.open {
                    focus.focus(window, cx);
                    this.emit_focus_changed(true, cx);
                }
            });
            self.dismiss_guard = false;
        }

        // Restore opener focus only after the close presence has finished painting.
        if self.pending_focus_restore && !self.presence.should_paint() {
            self.pending_focus_restore = false;
            if let Some(focus) = self.restore_focus.clone() {
                cx.on_next_frame(window, move |_this, window, cx| {
                    focus.focus(window, cx);
                });
            }
        }
    }
}

impl Focusable for PopoverPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for PopoverPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.focus_out_subscription.is_none() {
            let focus_handle = self.focus_handle.clone();
            self.focus_out_subscription = Some(cx.on_focus_out(&focus_handle, window, Self::handle_focus_out));
        }

        let was_animating = self.presence.is_animating();
        let is_animating = self.presence.sync();
        self.presence.schedule_frame(window, cx);
        if was_animating || is_animating {
            cx.notify();
        }

        self.sync_focus(window, cx);

        let model = self.render_model(window);
        if !self.presence.should_paint() {
            return div().into_any_element();
        }

        let Some(anchor_bounds) = model.anchor_bounds else {
            return div().into_any_element();
        };

        let panel = cx.entity();
        let content = (self.model.content)(&model, window, cx);
        let content_size = self.content_bounds.map(|bounds| bounds.size).or(self.model.initial_content_size);
        let placement = resolve_popover_panel_placement(
            anchor_bounds,
            content_size,
            self.model.placement,
            self.model.offset_y,
            PopupSpace::of(window, cx),
            self.model.window_margin,
        );
        let scale_size = content_size.unwrap_or(Size { width: px(0.0), height: px(0.0) });
        let offset = self.presence.adjust_offset(placement.offset, scale_size);
        let opacity = self.presence.opacity();

        let content = div()
            // The overlay owns pointer hit-testing while open. Without this,
            // deferred anchored content can still leave underlying controls
            // hovered as the pointer moves across the popup.
            .block_mouse_except_scroll()
            .on_prepaint(move |bounds, _, cx| {
                panel.update(cx, |panel, cx| panel.handle_content_bounds(bounds, cx));
            })
            .opacity(opacity)
            .child(self.model.look.wrap(content))
            .into_any_element();

        let overlay = anchored()
            .snap_to_window_with_margin(self.model.window_margin)
            .anchor(placement.anchor)
            .position(placement.position)
            .offset(offset)
            .child(content);

        div()
            .id(self.model.id.clone())
            .track_focus(&self.focus_handle)
            .key_context("LumaFocus")
            .occlude()
            .on_mouse_down(MouseButton::Left, cx.listener(Self::handle_shell_mouse_down))
            .on_mouse_move(cx.listener(Self::handle_mouse_move))
            .on_mouse_down_out(cx.listener(Self::handle_mouse_down_out))
            .on_action(cx.listener(Self::handle_escape_focus))
            .child(deferred(overlay).with_priority(1))
            .into_any_element()
    }
}

fn resolve_popover_panel_placement(
    anchor_bounds: Bounds<Pixels>,
    content_size: Option<Size<Pixels>>,
    placement: PopoverPanelPlacement,
    offset_y: Pixels,
    viewport_size: impl Into<PopupSpace>,
    window_margin: Pixels,
) -> ResolvedPopupPlacement {
    let (side, align, flip) = match placement {
        PopoverPanelPlacement::BelowStart => (PopupSide::Below, PopupAlign::Start, false),
        PopoverPanelPlacement::BelowCenter => (PopupSide::Below, PopupAlign::Center, false),
        PopoverPanelPlacement::AboveStart => (PopupSide::Above, PopupAlign::Start, false),
        PopoverPanelPlacement::SmartStart => (PopupSide::Below, PopupAlign::Start, true),
    };
    PopupPlacementRequest::new(anchor_bounds, side, viewport_size)
        .align(align)
        .flip(flip)
        .gap(offset_y)
        .content_size(content_size)
        .window_margin(window_margin)
        .resolve()
}

#[cfg(test)]
mod tests {
    use gpui::{Anchor, point, px, size, Bounds};

    use super::*;

    #[test]
    fn below_center_uses_content_width_for_horizontal_offset() {
        let placement = resolve_popover_panel_placement(
            Bounds::new(point(px(20.0), px(10.0)), size(px(40.0), px(20.0))),
            Some(size(px(100.0), px(50.0))),
            PopoverPanelPlacement::BelowCenter,
            px(4.0),
            size(px(500.0), px(500.0)),
            px(8.0),
        );

        assert_eq!(placement.anchor, Anchor::TopLeft);
        assert_eq!(placement.position, point(px(40.0), px(30.0)));
        assert_eq!(placement.offset, point(px(-50.0), px(4.0)));
    }

    #[test]
    fn smart_start_flips_above_when_below_does_not_fit() {
        let placement = resolve_popover_panel_placement(
            Bounds::new(point(px(20.0), px(450.0)), size(px(40.0), px(20.0))),
            Some(size(px(100.0), px(80.0))),
            PopoverPanelPlacement::SmartStart,
            px(4.0),
            size(px(500.0), px(500.0)),
            px(8.0),
        );

        assert_eq!(placement.anchor, Anchor::BottomLeft);
        assert_eq!(placement.position, point(px(20.0), px(450.0)));
        assert_eq!(placement.offset, point(px(0.0), px(-4.0)));
    }
}
