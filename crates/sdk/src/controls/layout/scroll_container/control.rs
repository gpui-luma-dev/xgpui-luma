use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use gpui::{
    AnyElement, App, AppContext, Bounds, Context, Entity, EntityId, Pixels, ScrollHandle, ScrollEvent, SharedString,
    Stateful, Task, Window, div, point, prelude::*, px,
};

use crate::controls::scrollbar::{Scrollbar, ScrollbarTemplate};

use super::model::{ScrollbarAutoHideActivate, ScrollbarPlacement, ScrollbarVisibility};
use super::template::{notify_host, scrollbar_chrome_visible, viewport_right_inset, wake_on_move};

type ScrollWheelHandler = Box<dyn Fn(&ScrollEvent, &mut Window, &mut App) + 'static>;

#[derive(Clone)]
pub struct ScrollContainer {
    id: SharedString,
    policy: Rc<Cell<crate::interaction::ScrollInteraction>>,
    focus_owner: Option<gpui::FocusHandle>,
    scroll_handle: ScrollHandle,
    motion: super::ScrollMotion,
    motion_config: Rc<RefCell<crate::motion::MotionConfig<super::ScrollContainerMotion>>>,
    pending_smooth: Rc<Cell<Option<f32>>>,
    scrollbar: Entity<Scrollbar>,
    scrollbar_width: Pixels,
    placement: ScrollbarPlacement,
    visibility: ScrollbarVisibility,
    auto_hide_activate: ScrollbarAutoHideActivate,
    last_max_scroll: Rc<Cell<f32>>,
    host_view: Rc<Cell<Option<EntityId>>>,
    hovered: Rc<Cell<bool>>,
    active_until: Rc<Cell<Option<Instant>>>,
    hide_task: Rc<RefCell<Option<Task<()>>>>,
}

impl ScrollContainer {
    pub fn new(
        id: impl Into<SharedString>,
        scrollbar_template: Arc<dyn ScrollbarTemplate>,
        cx: &mut impl AppContext,
    ) -> Self {
        let id = id.into();
        let scrollbar = Scrollbar::new(format!("{id}-scrollbar")).vertical().template(scrollbar_template).spawn(cx);

        Self {
            id,
            policy: Rc::new(Cell::new(crate::interaction::ScrollInteraction::DOCUMENT)),
            focus_owner: None,
            scroll_handle: ScrollHandle::new(),
            motion: super::ScrollMotion::default(),
            motion_config: Rc::new(RefCell::new(Default::default())),
            pending_smooth: Rc::default(),
            scrollbar,
            scrollbar_width: px(crate::theme::stylesheet::SharedGeometry::sdk().scrollbar_width),
            placement: ScrollbarPlacement::Inset,
            visibility: ScrollbarVisibility::AlwaysVisible,
            auto_hide_activate: ScrollbarAutoHideActivate::HoverOrMove,
            last_max_scroll: Rc::new(Cell::new(0.0)),
            host_view: Rc::new(Cell::new(None)),
            hovered: Rc::new(Cell::new(false)),
            active_until: Rc::new(Cell::new(None)),
            hide_task: Rc::new(RefCell::new(None)),
        }
    }

    pub fn motion(self, configure: impl Fn(&mut super::ScrollContainerMotion) + Send + Sync + 'static) -> Self {
        {
            let mut config = self.motion_config.borrow_mut();
            config.configure(configure);
            self.motion.configure(config.resolve().scroll.enabled(config.enabled()));
        }
        self
    }
    pub fn motion_defaults(self, defaults: super::ScrollContainerMotion) -> Self {
        {
            let mut config = self.motion_config.borrow_mut();
            config.defaults(defaults);
            self.motion.configure(config.resolve().scroll.enabled(config.enabled()));
        }
        self
    }
    pub fn motion_enabled(self, enabled: bool) -> Self {
        self.set_motion_enabled(enabled);
        self
    }
    pub fn set_motion_enabled(&self, enabled: bool) {
        let mut config = self.motion_config.borrow_mut();
        config.set_enabled(enabled);
        self.motion.cancel();
        self.motion.configure(config.resolve().scroll.enabled(enabled));
    }

    /// Explicit owner for RequireFocus. Without one, RequireFocus passes through.
    pub fn wheel_focus_owner(mut self, owner: gpui::FocusHandle) -> Self {
        self.focus_owner = Some(owner);
        self
    }

    /// Configure wheel independently.
    pub fn wheel_scroll_policy(self, policy: crate::interaction::WheelScrollPolicy) -> Self {
        self.set_wheel_scroll_policy(policy);
        self
    }
    /// Change wheel routing without discarding position or motion.
    pub fn set_wheel_scroll_policy(&self, policy: crate::interaction::WheelScrollPolicy) {
        let mut current = self.policy.get();
        current.wheel = policy;
        self.policy.set(current);
    }

    /// Configure boundary independently.
    pub fn scroll_boundary_policy(self, policy: crate::interaction::ScrollBoundaryPolicy) -> Self {
        self.set_scroll_boundary_policy(policy);
        self
    }
    /// Change wheel routing without discarding position or motion.
    pub fn set_scroll_boundary_policy(&self, policy: crate::interaction::ScrollBoundaryPolicy) {
        let mut current = self.policy.get();
        current.boundary = policy;
        self.policy.set(current);
    }

    /// Configure focus_scope independently.
    pub fn wheel_focus_scope(self, policy: crate::interaction::WheelFocusScope) -> Self {
        self.set_wheel_focus_scope(policy);
        self
    }
    /// Change wheel routing without discarding position or motion.
    pub fn set_wheel_focus_scope(&self, policy: crate::interaction::WheelFocusScope) {
        let mut current = self.policy.get();
        current.focus_scope = policy;
        self.policy.set(current);
    }

    pub fn placement(mut self, placement: ScrollbarPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn overlay(mut self, overlay: bool) -> Self {
        self.placement = if overlay {
            ScrollbarPlacement::Overlay
        } else {
            ScrollbarPlacement::Inset
        };
        self
    }

    pub fn visibility(mut self, visibility: ScrollbarVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    pub fn auto_hide(mut self, auto_hide: bool) -> Self {
        self.visibility = if auto_hide {
            ScrollbarVisibility::AutoHide
        } else {
            ScrollbarVisibility::AlwaysVisible
        };
        self
    }

    pub fn auto_hide_activate(mut self, activate: ScrollbarAutoHideActivate) -> Self {
        self.auto_hide_activate = activate;
        self
    }

    pub fn scrollbar_placement(&self) -> ScrollbarPlacement {
        self.placement
    }

    pub fn scrollbar_visibility(&self) -> ScrollbarVisibility {
        self.visibility
    }

    pub fn scrollbar_auto_hide_activate(&self) -> ScrollbarAutoHideActivate {
        self.auto_hide_activate
    }

    /// Reveal timed chrome when a composite owner handles wheel movement itself.
    pub(crate) fn reveal_on_scroll(&self, cx: &mut App) {
        if self.visibility == ScrollbarVisibility::AutoHide && self.auto_hide_activate.listens_to_move() {
            wake_on_move(&self.active_until, &self.hide_task, &self.host_view, cx);
        }
    }

    pub fn scrollbar(&self) -> Entity<Scrollbar> {
        self.scrollbar.clone()
    }

    pub fn set_vertical_offset<T: 'static>(&self, value: f32, cx: &mut Context<T>) {
        self.motion.cancel();
        self.pending_smooth.set(None);
        self.scroll_handle.set_offset(point(px(0.0), px(-value.max(0.0))));
        cx.notify();
    }

    /// Animate to a vertical offset in logical pixels, clamped to the content.
    /// Uses the shared 200ms ease-out scroll motion. Wheel, pointer or keyboard
    /// input and subsequent positioning requests interrupt it.
    pub fn set_vertical_offset_smooth<T: 'static>(&self, value: f32, cx: &mut Context<T>) {
        self.motion.cancel();
        self.pending_smooth.set(value.is_finite().then_some(value.max(0.0)));
        cx.notify();
    }

    pub(crate) fn drag_scroll_handle(&self) -> ScrollHandle {
        self.scroll_handle.clone()
    }

    pub fn vertical_offset(&self) -> Pixels {
        px((-self.scroll_handle.offset().y.as_f32()).max(0.0))
    }

    pub fn max_vertical_offset(&self) -> Pixels {
        px(self.scroll_handle.max_offset().y.as_f32().max(0.0))
    }

    pub fn scroll_vertical_by<T: 'static>(&self, delta: Pixels, cx: &mut Context<T>) -> bool {
        let current = self.vertical_offset().as_f32();
        let target = (current + delta.as_f32()).clamp(0.0, self.max_vertical_offset().as_f32());
        if (target - current).abs() <= 0.5 {
            return false;
        }

        self.set_vertical_offset(target, cx);
        true
    }

    pub fn ensure_vertical_bounds_visible<T: 'static>(&self, top: Pixels, bottom: Pixels, cx: &mut Context<T>) -> bool {
        let viewport_height = self.scroll_handle.bounds().size.height.as_f32().max(0.0);
        let max_scroll = self.scroll_handle.max_offset().y.as_f32().max(0.0);
        let current = self.vertical_offset().as_f32();
        let mut target = current;

        let visible_top = current;
        let visible_bottom = current + viewport_height;

        let top = top.as_f32();
        let bottom = bottom.as_f32();

        if top < visible_top {
            target -= visible_top - top;
        } else if bottom > visible_bottom {
            target += bottom - visible_bottom;
        }

        target = target.clamp(0.0, max_scroll);
        if (target - current).abs() <= 0.5 {
            return false;
        }

        self.set_vertical_offset(target, cx);
        true
    }

    pub fn sync_scrollbar<T: 'static>(&self, cx: &mut Context<T>) {
        let viewport_height = self.scroll_handle.bounds().size.height.as_f32().max(0.0);
        let max_scroll = self.scroll_handle.max_offset().y.as_f32().max(0.0);
        let value = (-self.scroll_handle.offset().y.as_f32()).clamp(0.0, max_scroll);
        let content_height = viewport_height + max_scroll;

        self.scrollbar.update(cx, |scrollbar, cx| {
            scrollbar.set_length(viewport_height.max(1.0), cx);
            scrollbar.set_step(24.0, cx);
            scrollbar.set_page_step((viewport_height * 0.85).max(1.0), cx);
            scrollbar.set_viewport(0.0, content_height.max(1.0), value, value + viewport_height, cx);
            scrollbar.set_enabled(max_scroll > 0.5, cx);
        });
    }

    pub fn render(&self, content: AnyElement) -> Stateful<gpui::Div> {
        self.render_internal(content, None, true)
    }

    pub fn render_with_scroll_wheel(
        &self,
        content: AnyElement,
        on_scroll_wheel: impl Fn(&ScrollEvent, &mut Window, &mut App) + 'static,
    ) -> Stateful<gpui::Div> {
        self.render_internal(content, Some(Box::new(on_scroll_wheel)), true)
    }

    /// Keep clipping, positioning and scrollbar interaction, but pass wheel input
    /// to ancestors without moving the viewport or cancelling pending motion.
    pub fn render_without_wheel(&self, content: AnyElement) -> Stateful<gpui::Div> {
        self.render_internal(content, None, false)
    }

    fn render_internal(
        &self,
        content: AnyElement,
        on_scroll_wheel: Option<ScrollWheelHandler>,
        accepts_wheel: bool,
    ) -> Stateful<gpui::Div> {
        let scrollable = self.scroll_handle.max_offset().y.as_f32() > 0.5;
        let viewport_right = viewport_right_inset(scrollable, self.placement, self.visibility, self.scrollbar_width);
        let scroll_handle = self.scroll_handle.clone();
        let last_max_scroll = self.last_max_scroll.clone();
        let host_view = self.host_view.clone();
        let auto_hide = self.visibility == ScrollbarVisibility::AutoHide;
        let activate = self.auto_hide_activate;
        let track_hover = auto_hide && activate.listens_to_hover();
        let track_move = auto_hide && activate.listens_to_move();
        let show_scrollbar = scrollable
            && self.visibility != ScrollbarVisibility::Hidden
            && scrollbar_chrome_visible(
                self.visibility,
                activate,
                self.hovered.get(),
                self.active_until.get(),
                Instant::now(),
            );

        let mut viewport = div()
            .on_children_prepainted({
                let host_view = host_view.clone();
                let motion = self.motion.clone();
                let pending = self.pending_smooth.clone();
                let scrollbar = self.scrollbar.clone();
                let focus_owner = self.focus_owner.clone();
                move |_: Vec<Bounds<Pixels>>, window: &mut Window, cx: &mut App| {
                    use gpui::Focusable;
                    if !show_scrollbar && scrollbar.read(cx).focus_handle(cx).is_focused(window) {
                        if let Some(owner) = &focus_owner {
                            owner.focus(window, cx);
                        } else {
                            window.blur(cx);
                        }
                    }
                    // `current_view` is only valid during layout/prepaint/paint.
                    host_view.set(Some(window.current_view()));
                    if scroll_handle.bounds().size.height > px(0.0)
                        && let Some(value) = pending.take()
                    {
                        motion.animate(scroll_handle.clone(), move |_| Some(point(px(0.0), px(-value))), window, cx);
                    }
                    let max_scroll = scroll_handle.max_offset().y.as_f32().max(0.0);
                    if (last_max_scroll.get() - max_scroll).abs() > 0.5 {
                        last_max_scroll.set(max_scroll);
                        notify_host(&host_view, cx);
                    }
                }
            })
            .id((self.id.clone(), 0usize))
            .absolute()
            .top(px(0.0))
            .bottom(px(0.0))
            .left(px(0.0))
            .right(viewport_right)
            .overflow_hidden()
            .scrollbar_width(px(0.0))
            .track_scroll(&self.scroll_handle)
            .child(content);

        if accepts_wheel {
            let active_until = self.active_until.clone();
            let hide_task = self.hide_task.clone();
            let host_view = host_view.clone();
            let motion = self.motion.clone();
            let pending = self.pending_smooth.clone();
            let scroll = self.scroll_handle.clone();
            let policy = self.policy.clone();
            let focus_owner = self.focus_owner.clone();
            let scrollbar = self.scrollbar.clone();
            viewport = viewport.on_scroll(move |event, window, cx| {
                use gpui::Focusable;
                let policy = policy.get();
                let focused = focus_owner.as_ref().is_some_and(|owner| {
                    policy.focus_scope.focused(owner, Some(&scrollbar.read(cx).focus_handle(cx)), window, cx)
                });
                if !policy.wheel.accepts(focused) {
                    return;
                }
                let Some(delta) = crate::interaction::wheel_delta(event, window.line_height(), false) else {
                    return;
                };
                let moved = crate::interaction::scroll_handle_by(&scroll, delta, false);
                if moved {
                    notify_host(&host_view, cx);
                }
                motion.cancel();
                pending.set(None);
                if track_move {
                    wake_on_move(&active_until, &hide_task, &host_view, cx);
                }
                if let Some(handler) = &on_scroll_wheel {
                    handler(event, window, cx);
                }
                if policy.boundary.consumes(moved) {
                    cx.stop_propagation();
                }
            });
        }

        let pointer_motion = self.motion.clone();
        let pointer_pending = self.pending_smooth.clone();
        let key_motion = self.motion.clone();
        let key_pending = self.pending_smooth.clone();
        let mut root = div()
            .id(self.id.clone())
            .relative()
            .size_full()
            .on_any_mouse_down(move |_, _, _| {
                pointer_motion.cancel();
                pointer_pending.set(None);
            })
            .on_key_down(move |_, _, _| {
                key_motion.cancel();
                key_pending.set(None);
            });

        if track_hover {
            let hovered = self.hovered.clone();
            let active_until = self.active_until.clone();
            let hide_task = self.hide_task.clone();
            let host_view = host_view.clone();
            root = root.on_hover(move |is_hovered, _window, cx| {
                let is_hovered = *is_hovered;
                if hovered.get() == is_hovered {
                    return;
                }
                hovered.set(is_hovered);
                if is_hovered {
                    // Hover keeps chrome visible; cancel a pending move-idle hide.
                    *hide_task.borrow_mut() = None;
                } else if activate == ScrollbarAutoHideActivate::Hover {
                    // Hover-only: hide as soon as the pointer leaves.
                    active_until.set(None);
                    *hide_task.borrow_mut() = None;
                }
                notify_host(&host_view, cx);
            });
        }

        root.child(viewport).when(show_scrollbar, |root| {
            root.child(
                div()
                    .absolute()
                    .top(px(0.0))
                    .right(px(0.0))
                    .bottom(px(0.0))
                    .w(self.scrollbar_width)
                    .flex()
                    .justify_center()
                    .child(self.scrollbar.clone()),
            )
        })
    }
}

#[cfg(all(test, feature = "test-support"))]
mod dispatch_tests {
    use super::*;
    use gpui::{FocusHandle, Focusable, Render};
    struct View {
        container: ScrollContainer,
        focus: FocusHandle,
    }
    impl Focusable for View {
        fn focus_handle(&self, _: &App) -> FocusHandle {
            self.focus.clone()
        }
    }
    impl Render for View {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            self.container.sync_scrollbar(cx);
            div()
                .id("container-test")
                .size_full()
                .track_focus(&self.focus)
                .child(self.container.render(div().h(px(3000.0)).w_full().into_any_element()))
        }
    }
    #[test]
    fn wheel_policy_dispatch_matrix() {
        crate::interaction_tests::matrix(
            |policy, cx| {
                cx.new(|cx| {
                    let focus = cx.focus_handle();
                    let container =
                        ScrollContainer::new("container", crate::controls::scrollbar::default_scrollbar_template(), cx)
                            .wheel_focus_owner(focus.clone())
                            .wheel_scroll_policy(policy.wheel)
                            .scroll_boundary_policy(policy.boundary);
                    View { container, focus }
                })
            },
            |view, _| view.container.vertical_offset().as_f32(),
            |view, cx| view.container.set_vertical_offset(view.container.max_vertical_offset().as_f32(), cx),
        );
    }
}
