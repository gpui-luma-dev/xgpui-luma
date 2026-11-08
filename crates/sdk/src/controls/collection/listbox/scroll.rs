use crate::interaction::{ScrollInteraction, WheelScrollPolicy, ScrollBoundaryPolicy, WheelFocusScope};
use std::cell::{Cell, RefCell};
use std::hash::Hash;
use std::rc::Rc;

use gpui::{Context, Div, DragMoveEvent, Pixels, ScrollHandle, Size, Stateful, div, prelude::*, px};

use crate::controls::scroll_container::ScrollMotion;

use super::{
    ListBoxAxis, ListBoxState, ListBoxUpdate, ListBoxVirtualWindow,
    virtualization::UniformMetrics,
    measured::MeasuredGeometry,
    drag_scroll::{DragAutoScroll, DEFAULT_MAX_SPEED},
};

/// Optional scrolling adapter for a host-composed list. The host supplies the
/// styled surface and scrolling stack, with one direct child per visible item
/// in `visible_items()` order. GPUI measures the children, so either axis and
/// variable item sizes work without host-written offset calculations. Uniform
/// virtualization uses `virtual_window`; content-sized vertical rows use
/// `content_window`. Both bind through `bind_virtualized` on this same handle.
///
/// Keep one handle per list. This adapter contains wheel events at the surface
/// and keeps the focused active item visible when the viewport resizes. It does
/// not choose the axis, dimensions, spacing, item content, or appearance.
pub struct ListBoxScrollHandle<K> {
    scroll: ScrollHandle,
    pending_reveal: RefCell<Option<K>>,
    pending_position: RefCell<Option<(K, bool)>>,
    positioned: Cell<bool>,
    smooth_requested: Cell<bool>,
    pending_animation: RefCell<Option<(K, bool)>>,
    motion: ScrollMotion,
    viewport_size: Rc<Cell<Size<Pixels>>>,
    drag_scroll: Rc<DragAutoScroll>,
    policy: Rc<Cell<ScrollInteraction>>,
    focus_owner: Rc<RefCell<Option<gpui::FocusHandle>>>,
    rendered_window: RefCell<Option<ListBoxVirtualWindow>>,
    measured: Rc<RefCell<MeasuredGeometry<K>>>,
}

impl<K> Default for ListBoxScrollHandle<K> {
    fn default() -> Self {
        Self {
            scroll: ScrollHandle::new(),
            pending_reveal: RefCell::new(None),
            pending_position: RefCell::new(None),
            positioned: Cell::new(false),
            smooth_requested: Cell::new(false),
            pending_animation: RefCell::new(None),
            motion: ScrollMotion::default(),
            viewport_size: Rc::default(),
            drag_scroll: Rc::default(),
            policy: Rc::new(Cell::new(ScrollInteraction::VIEWPORT)),
            focus_owner: Rc::default(),
            rendered_window: RefCell::new(None),
            measured: Rc::default(),
        }
    }
}

impl<K: Clone + Eq + Hash + 'static> ListBoxScrollHandle<K> {
    pub fn set_motion(&self, definition: crate::motion::Transition) {
        self.motion.configure(definition);
    }
    /// Request a one-time reveal of this stable key, even without focus.
    /// Fully visible items stay in place; oversized items show their leading
    /// edge. Missing keys are ignored at layout. Selection is unchanged.
    /// Compose a drop-motion viewport using the same retained scroll offset.
    #[doc(hidden)]
    pub fn drop_layout_scope(
        &self,
        motion: &crate::motion::DropLayoutMotion<K>,
        content: impl gpui::IntoElement,
    ) -> gpui::AnyElement {
        let scroll = self.scroll.clone();
        motion.scope(content, move || scroll.offset())
    }
    pub fn scroll_to<M: 'static>(&self, item_key: K, cx: &mut Context<M>) {
        self.request_position(item_key, false, cx);
    }

    /// Center this item in the viewport, clamped at collection boundaries.
    /// The latest request wins; layout resolves keys against the current data.
    /// This does not select/focus the item or change subsequent wheel behavior.
    pub fn scroll_to_center<M: 'static>(&self, item_key: K, cx: &mut Context<M>) {
        self.request_position(item_key, true, cx);
    }

    /// Smooth counterpart of `scroll_to`, using the shared scroll animator.
    /// New input or another positioning request interrupts the movement.
    pub fn scroll_to_smooth<M: 'static>(&self, item_key: K, cx: &mut Context<M>) {
        self.request_position(item_key, false, cx);
        self.smooth_requested.set(true);
    }

    /// Smoothly center this specific item in the viewport. Selection is unchanged.
    pub fn scroll_to_center_smooth<M: 'static>(&self, item_key: K, cx: &mut Context<M>) {
        self.request_position(item_key, true, cx);
        self.smooth_requested.set(true);
    }

    fn request_position<M: 'static>(&self, key: K, center: bool, cx: &mut Context<M>) {
        self.motion.cancel();
        self.smooth_requested.set(false);
        self.pending_animation.borrow_mut().take();
        self.pending_reveal.borrow_mut().take();
        self.measured.borrow_mut().reveal = None;
        *self.pending_position.borrow_mut() = Some((key, center));
        cx.notify();
    }

    /// Read-only geometry from the most recent virtualized or content-sized
    /// composition, or `None` for fixed-size eager rendering. Read after building the list for diagnostics;
    /// this neither measures layout nor requests a frame or consumes a reveal.
    pub fn rendered_window(&self) -> Option<ListBoxVirtualWindow> {
        self.rendered_window.borrow().clone()
    }

    /// Invalidate cached content heights after a host-owned content or typography
    /// change. `None` invalidates all rows; otherwise only the supplied key.
    /// Existing heights remain estimates until measured again. Notify the owner
    /// after calling this. Snapshot replacement and viewport width changes are automatic.
    pub fn invalidate_measurements(&self, key: Option<&K>) {
        self.motion.cancel();
        self.measured.borrow_mut().invalidate(key);
    }

    /// Compose content-sized vertical rows. `overscan: None` constructs all rows;
    /// `Some(n)` uses measured/estimated heights to construct a viewport window.
    /// Pass the result to `bind_virtualized`; templates remain ordinary GPUI content.
    pub fn content_window<T>(
        &self,
        state: &ListBoxState<T, K>,
        estimated_height: f32,
        gap: f32,
        overscan: Option<usize>,
    ) -> ListBoxVirtualWindow {
        let mut geometry = self.measured.borrow_mut();
        let offset = -f32::from(self.scroll.offset().y) + std::mem::take(&mut geometry.correction);
        let offset = geometry.prepare(state, estimated_height, gap, overscan, offset);
        let viewport = f32::from(self.viewport_size.get().height);
        if viewport > 0.0
            && let Some((key, center)) = self.pending_position.borrow_mut().take()
        {
            if self.smooth_requested.replace(false) {
                *self.pending_animation.borrow_mut() = state.visible_index(&key).map(|_| (key, center));
            } else {
                geometry.reveal = state.visible_index(&key).map(|_| key);
                geometry.reveal_center = center;
                geometry.reveal_unfocused = true;
            }
            self.positioned.set(true);
        } else if state.is_focused() {
            if let Some(key) = self.pending_reveal.borrow_mut().take() {
                geometry.reveal = Some(key);
                geometry.reveal_center = false;
                geometry.reveal_unfocused = false;
            }
        } else if !geometry.reveal_unfocused {
            geometry.reveal = None;
        }
        let offset = geometry.reveal_offset(offset, viewport);
        let mut position = self.scroll.offset();
        position.y = px(-offset);
        self.scroll.set_offset(position);
        geometry.window(offset, viewport)
    }

    /// Compatibility alias for `wheel_scroll_policy`; leaves boundary behavior unchanged.
    pub fn require_focus_for_scroll(self, required: bool) -> Self {
        self.wheel_scroll_policy(required.into())
    }

    /// Supply the real owner when composing a list manually. Look builders do this automatically.
    pub fn set_wheel_focus_owner(&self, owner: gpui::FocusHandle) {
        *self.focus_owner.borrow_mut() = Some(owner);
    }

    /// Current wheel settings.
    pub fn scroll_interaction(&self) -> ScrollInteraction {
        self.policy.get()
    }

    /// Configure wheel independently of other settings.
    pub fn wheel_scroll_policy(self, policy: WheelScrollPolicy) -> Self {
        self.set_wheel_scroll_policy(policy);
        self
    }
    /// Update wheel for the next wheel event; retained selection/position are unchanged.
    pub fn set_wheel_scroll_policy(&self, policy: WheelScrollPolicy) {
        let mut current = self.policy.get();
        current.wheel = policy;
        self.policy.set(current);
    }

    /// Configure boundary independently of other settings.
    pub fn scroll_boundary_policy(self, policy: ScrollBoundaryPolicy) -> Self {
        self.set_scroll_boundary_policy(policy);
        self
    }
    /// Update boundary for the next wheel event; retained selection/position are unchanged.
    pub fn set_scroll_boundary_policy(&self, policy: ScrollBoundaryPolicy) {
        let mut current = self.policy.get();
        current.boundary = policy;
        self.policy.set(current);
    }

    /// Configure focus_scope independently of other settings.
    pub fn wheel_focus_scope(self, policy: WheelFocusScope) -> Self {
        self.set_wheel_focus_scope(policy);
        self
    }
    /// Update focus_scope for the next wheel event; retained selection/position are unchanged.
    pub fn set_wheel_focus_scope(&self, policy: WheelFocusScope) {
        let mut current = self.policy.get();
        current.focus_scope = policy;
        self.policy.set(current);
    }

    /// Set maximum drag auto-scroll speed in logical pixels per second, per list.
    /// Defaults to 540. The basic auto-scrolling function increases speed linearly
    /// with pointer depth into the edge zone, without temporal easing or inertia.
    /// Zero disables movement; negative or non-finite values restore the default.
    /// Changes are read on the next drag-scroll frame, including during a drag.
    ///
    /// ```
    /// use xgpui_luma::controls::listbox::ListBoxScrollHandle;
    /// let scroll = ListBoxScrollHandle::<u32>::default();
    /// scroll.set_drag_auto_scroll_speed(720.0);
    /// ```
    pub fn set_drag_auto_scroll_speed(&self, pixels_per_second: f32) {
        let speed = if pixels_per_second.is_finite() && pixels_per_second >= 0.0 {
            pixels_per_second
        } else {
            DEFAULT_MAX_SPEED
        };
        self.drag_scroll.max_speed.set(speed);
    }

    /// Opt into continuous edge scrolling for an accepted drag payload. Attach
    /// to the same viewport passed to `bind`. The host retains drop policy;
    /// scrolling stops outside the viewport, at its limits, or when the drag ends.
    /// Configure speed with [`Self::set_drag_auto_scroll_speed`].
    /// Currently uses a basic linear auto-scrolling function; custom speed/easing
    /// functions are a future extension.
    pub fn bind_drag_auto_scroll<D: 'static>(
        &self,
        viewport: Stateful<Div>,
        axis: ListBoxAxis,
        accepts: impl Fn(&D) -> bool + 'static,
    ) -> Stateful<Div> {
        let controller = self.drag_scroll.clone();
        let motion = self.motion.clone();
        let scroll = self.scroll.clone();
        viewport.on_drag_move(move |event: &DragMoveEvent<D>, window, cx| {
            motion.cancel();
            controller.accepted.set(accepts(event.drag(cx)));
            controller.schedule(scroll.clone(), axis, window);
        })
    }

    /// Deliver rendering and reveal effects after committing an update to the
    /// model. The host still owns event delivery and application reactions.
    pub fn handle_update<M: 'static>(&self, update: &ListBoxUpdate<K>, cx: &mut Context<M>) {
        if update.changed || update.reveal.is_some() {
            self.motion.cancel();
        }
        self.queue_reveal(update);
        if update.changed || update.reveal.is_some() {
            cx.notify();
        }
    }

    /// Compute a uniform-item render window using the retained viewport and offset.
    /// Invalid dimensions return `None`; callers should use eager composition.
    /// Render only `range`, with the returned spacers as direct stack children.
    /// Call [`Self::bind_virtualized`] with this window instead of `bind`.
    pub fn virtual_window<T>(
        &self,
        state: &ListBoxState<T, K>,
        axis: ListBoxAxis,
        item_extent: f32,
        gap: f32,
        overscan: usize,
    ) -> Option<ListBoxVirtualWindow> {
        let metrics = UniformMetrics::new(axis, state.visible_items().len(), item_extent, gap, overscan)?;
        let viewport = metrics.viewport_extent(self.viewport_size.get());
        let reveal = if viewport > 0.0 && (state.is_focused()) {
            self.take_reveal_index(state)
        } else {
            None
        };
        metrics.update_scroll(&self.scroll, viewport, reveal);
        if viewport > 0.0
            && let Some((key, center)) = self.pending_position.borrow_mut().take()
            && let Some(index) = state.visible_index(&key)
        {
            if self.smooth_requested.replace(false) {
                *self.pending_animation.borrow_mut() = Some((key, center));
            } else {
                metrics.position_item(&self.scroll, viewport, index, center);
            }
            self.positioned.set(true);
        }
        Some(metrics.window(metrics.offset(&self.scroll), viewport))
    }

    /// Attach a uniform or measured stack. Reveals use collection geometry,
    /// not spacer/child indices; wheel policy and drag scrolling are unchanged.
    pub fn bind_virtualized<T>(
        &self,
        surface: Stateful<Div>,
        viewport: Stateful<Div>,
        state: &ListBoxState<T, K>,
        rendered: ListBoxVirtualWindow,
    ) -> Stateful<Div> {
        self.bind_inner(surface, viewport, state, Some(rendered))
    }

    /// Attach the scrolling stack inside its surface. Resolve reveal keys at
    /// render time against the current snapshot, not against stale indices.
    pub fn bind<T>(
        &self,
        surface: Stateful<Div>,
        viewport: Stateful<Div>,
        state: &ListBoxState<T, K>,
    ) -> Stateful<Div> {
        self.bind_inner(surface, viewport, state, None)
    }

    fn bind_inner<T>(
        &self,
        surface: Stateful<Div>,
        mut viewport: Stateful<Div>,
        state: &ListBoxState<T, K>,
        rendered: Option<ListBoxVirtualWindow>,
    ) -> Stateful<Div> {
        let axis = if viewport.interactivity().base_style.overflow.x == Some(gpui::Overflow::Scroll) {
            ListBoxAxis::Horizontal
        } else {
            ListBoxAxis::Vertical
        };
        let explicit = if rendered.is_none() {
            self.pending_position.borrow_mut().take().and_then(|(key, center)| {
                let index = state.visible_index(&key)?;
                if self.smooth_requested.replace(false) {
                    *self.pending_animation.borrow_mut() = Some((key, center));
                    None
                } else {
                    Some((index, center))
                }
            })
        } else {
            None
        };
        let animation_request = self
            .pending_animation
            .borrow_mut()
            .take()
            .and_then(|(key, center)| state.visible_index(&key).map(|index| (key, index, center)));
        let motion = self.motion.clone();
        let wheel_motion = motion.clone();
        let pointer_motion = motion.clone();
        let animation_metrics = rendered.as_ref().and_then(|window| window.metrics);
        let animation_geometry =
            rendered.as_ref().filter(|window| window.metrics.is_none()).map(|_| self.measured.clone());
        let positioned =
            self.positioned.replace(false) || explicit.is_some() || animation_request.is_some() || motion.is_active();
        self.rendered_window.replace(rendered.clone());
        let (viewport, _accepts_wheel) = if rendered.is_some() {
            self.track_viewport(viewport, state)
        } else {
            self.bind_viewport(viewport, state)
        };
        let active_index = state.active_key().filter(|_| state.is_focused()).and_then(|key| state.visible_index(key));
        let scroll = self.scroll.clone();
        let viewport_size = self.viewport_size.clone();
        let content_geometry = rendered.as_ref().filter(|window| window.metrics.is_none()).map(|window| {
            let geometry = self.measured.clone();
            let keys = geometry.borrow().keys_in(window.range.clone());
            (geometry, keys, usize::from(window.leading_space.is_some()), -f32::from(scroll.offset().y))
        });
        let measured_viewport = div()
            .w_full()
            .min_w(px(0.0))
            .on_children_prepainted(move |bounds, window, cx| {
                let Some(bounds) = bounds.first() else { return };
                let previous = viewport_size.replace(bounds.size);
                let reveal = resize_reveal(previous, bounds.size, active_index.filter(|_| !positioned));
                let needs_frame = if let Some(rendered) = &rendered
                    && let Some(metrics) = rendered.metrics
                {
                    let extent = metrics.viewport_extent(bounds.size);
                    let previous_offset = scroll.offset();
                    metrics.update_scroll(&scroll, extent, reveal);
                    // Measurement is retained for the next render. Wheel input
                    // already notifies the owning view; this also covers first
                    // layout, resizing, and GPUI's post-layout offset clamping.
                    previous != bounds.size
                        || previous_offset != scroll.offset()
                        || metrics.window(metrics.offset(&scroll), extent).range != rendered.range
                } else if let Some((geometry, keys, skip, render_offset)) = &content_geometry {
                    let actual_offset = -f32::from(scroll.offset().y);
                    let mut geometry = geometry.borrow_mut();
                    let changed = geometry.measure(
                        keys.iter().enumerate().filter_map(|(i, key)| {
                            scroll.bounds_for_item(i + skip).map(|row| (key.clone(), f32::from(row.size.height)))
                        }),
                        f32::from(bounds.size.width),
                        *render_offset,
                        f32::from(bounds.size.height),
                    );
                    // GPUI may already have clamped a shrinking collection.
                    // Anchor against the pre-layout geometry, then apply the
                    // correction relative to the actual post-layout offset.
                    geometry.correction += *render_offset - actual_offset;
                    previous != bounds.size || changed || geometry.correction.abs() > 0.01
                } else if let Some((index, center)) = explicit {
                    let previous_offset = scroll.offset();
                    if let Some(row) = scroll.bounds_for_item(index) {
                        let viewport = scroll.bounds();
                        let mut offset = previous_offset;
                        let (start, extent, visible, maximum, value) = match axis {
                            ListBoxAxis::Vertical => (
                                row.top() - viewport.top(),
                                row.size.height,
                                viewport.size.height,
                                scroll.max_offset().y,
                                &mut offset.y,
                            ),
                            ListBoxAxis::Horizontal => (
                                row.left() - viewport.left(),
                                row.size.width,
                                viewport.size.width,
                                scroll.max_offset().x,
                                &mut offset.x,
                            ),
                        };
                        *value = px(-position_offset(
                            -f32::from(*value),
                            f32::from(visible),
                            f32::from(start),
                            f32::from(extent),
                            center,
                        )
                        .clamp(0.0, f32::from(maximum).max(0.0)));
                        scroll.set_offset(offset);
                    }
                    scroll.offset() != previous_offset
                } else if let Some(index) = reveal {
                    position_eager_item(&scroll, axis, index)
                } else {
                    false
                };
                if let Some((key, index, center)) = &animation_request {
                    let (key, index, center) = (key.clone(), *index, *center);
                    let geometry = animation_geometry.clone();
                    let original = scroll.offset();
                    motion.animate(
                        scroll.clone(),
                        move |scroll| {
                            let viewport = scroll.bounds();
                            let (offset, visible) = match axis {
                                ListBoxAxis::Vertical => (-original.y.as_f32(), viewport.size.height.as_f32()),
                                ListBoxAxis::Horizontal => (-original.x.as_f32(), viewport.size.width.as_f32()),
                            };
                            let target = if let Some(geometry) = &geometry {
                                geometry.borrow().position_for_key(&key, offset, visible, center)?
                            } else if let Some(metrics) = animation_metrics {
                                metrics.position_offset(offset, visible, index, center)
                            } else {
                                let row = scroll.bounds_for_item(index)?;
                                let (start, extent) = match axis {
                                    ListBoxAxis::Vertical => {
                                        ((row.top() - viewport.top()).as_f32(), row.size.height.as_f32())
                                    }
                                    ListBoxAxis::Horizontal => {
                                        ((row.left() - viewport.left()).as_f32(), row.size.width.as_f32())
                                    }
                                };
                                position_offset(offset, visible, start, extent, center)
                            };
                            let mut position = scroll.offset();
                            match axis {
                                ListBoxAxis::Vertical => position.y = px(-target),
                                ListBoxAxis::Horizontal => position.x = px(-target),
                            }
                            Some(position)
                        },
                        window,
                        cx,
                    );
                }
                if needs_frame {
                    let view = window.current_view();
                    window.on_next_frame(move |_, cx| cx.notify(view));
                }
            })
            .child(viewport);
        let wheel_scroll = self.scroll.clone();
        let wheel_policy = self.policy.clone();
        let wheel_owner = self.focus_owner.clone();
        let focused = state.is_focused();
        surface
            .on_any_mouse_down(move |_, _, _| pointer_motion.cancel())
            .on_scroll(move |event, window, cx| {
                let policy = wheel_policy.get();
                let focused = wheel_owner
                    .borrow()
                    .as_ref()
                    .map_or(focused, |owner| policy.focus_scope.focused(owner, None, window, cx));
                if !policy.wheel.accepts(focused) {
                    return;
                }
                let Some(delta) =
                    crate::interaction::wheel_delta(event, window.line_height(), axis == ListBoxAxis::Horizontal)
                else {
                    return;
                };
                wheel_motion.cancel();
                let moved = crate::interaction::scroll_handle_by(&wheel_scroll, delta, axis == ListBoxAxis::Horizontal);
                if moved {
                    window.refresh();
                }
                if policy.boundary.consumes(moved) {
                    cx.stop_propagation();
                }
            })
            .child(measured_viewport)
    }

    fn bind_viewport<T>(&self, mut viewport: Stateful<Div>, state: &ListBoxState<T, K>) -> (Stateful<Div>, bool) {
        if state.is_focused()
            && let Some(index) = self.take_reveal_index(state)
        {
            let horizontal = viewport.interactivity().base_style.overflow.x == Some(gpui::Overflow::Scroll);
            position_eager_item(
                &self.scroll,
                if horizontal {
                    ListBoxAxis::Horizontal
                } else {
                    ListBoxAxis::Vertical
                },
                index,
            );
        }
        self.track_viewport(viewport, state)
    }

    fn track_viewport<T>(&self, viewport: Stateful<Div>, state: &ListBoxState<T, K>) -> (Stateful<Div>, bool) {
        (
            viewport.overflow_hidden().track_scroll(&self.scroll),
            self.policy.get().wheel.accepts(state.is_focused()),
        )
    }

    fn queue_reveal(&self, update: &ListBoxUpdate<K>) {
        if let Some(key) = &update.reveal {
            self.motion.cancel();
            self.pending_animation.borrow_mut().take();
            self.pending_position.borrow_mut().take();
            self.measured.borrow_mut().reveal = None;
            *self.pending_reveal.borrow_mut() = Some(key.clone());
        }
    }

    fn take_reveal_index<T>(&self, state: &ListBoxState<T, K>) -> Option<usize> {
        self.pending_reveal.borrow_mut().take().and_then(|key| state.visible_index(&key))
    }
}

// Native scroll_to_item only reveals axes with Overflow::Scroll. Wheel routing
// deliberately uses Hidden, so reveal explicitly using the retained geometry.
fn position_eager_item(scroll: &ScrollHandle, axis: ListBoxAxis, index: usize) -> bool {
    let Some(row) = scroll.bounds_for_item(index) else {
        return false;
    };
    let viewport = scroll.bounds();
    let old = scroll.offset();
    let mut next = old;
    let (start, extent, visible, maximum, value) = match axis {
        ListBoxAxis::Vertical => (
            (row.top() - viewport.top()).as_f32(),
            row.size.height.as_f32(),
            viewport.size.height.as_f32(),
            scroll.max_offset().y.as_f32(),
            &mut next.y,
        ),
        ListBoxAxis::Horizontal => (
            (row.left() - viewport.left()).as_f32(),
            row.size.width.as_f32(),
            viewport.size.width.as_f32(),
            scroll.max_offset().x.as_f32(),
            &mut next.x,
        ),
    };
    *value = px(-position_offset(-value.as_f32(), visible, start, extent, false).clamp(0.0, maximum.max(0.0)));
    scroll.set_offset(next);
    old != next
}

pub(super) fn position_offset(offset: f32, viewport: f32, start: f32, extent: f32, center: bool) -> f32 {
    if center {
        start + (extent - viewport) / 2.0
    } else if start < offset || extent > viewport {
        start
    } else if start + extent > offset + viewport {
        start + extent - viewport
    } else {
        offset
    }
}

fn resize_reveal(previous: Size<Pixels>, current: Size<Pixels>, active_index: Option<usize>) -> Option<usize> {
    (previous != current).then_some(active_index).flatten()
}

#[cfg(test)]
mod tests {
    use gpui::size;
    use super::*;
    use super::super::{ListBoxInput, ListBoxNavigation, ListBoxSnapshot, SelectionMode};

    #[test]
    fn focus_policy_controls_both_axes_and_preserves_offset_and_pending_reveal() {
        for horizontal in [false, true] {
            for require_focus in [false, true] {
                let scroll = ListBoxScrollHandle::<u32>::default().require_focus_for_scroll(require_focus);
                let mut state = ListBoxState::try_new([1, 2, 3], |item| *item, SelectionMode::Multiple).unwrap();
                let offset = gpui::point(px(-40.0), px(-80.0));
                scroll.scroll.set_offset(offset);
                scroll.queue_reveal(&state.apply(ListBoxInput::Navigate(ListBoxNavigation::Last)));

                // Enter, leave, and re-enter focus without changing the host's
                // scroll axis or discarding the tracked position.
                for focused in [false, true, false, true] {
                    state.apply(ListBoxInput::Focus(focused));
                    let viewport = div().id("viewport");
                    let viewport = if horizontal {
                        viewport.overflow_x_scroll()
                    } else {
                        viewport.overflow_y_scroll()
                    };
                    let (mut viewport, accepts_wheel) = scroll.bind_viewport(viewport, &state);
                    assert_eq!(accepts_wheel, !require_focus || focused);
                    let overflow = &viewport.interactivity().base_style.overflow;
                    assert_eq!(overflow.x, Some(gpui::Overflow::Hidden));
                    assert_eq!(overflow.y, Some(gpui::Overflow::Hidden));
                    if focused {
                        assert!(scroll.pending_reveal.borrow().is_none());
                    }
                    assert_eq!(scroll.scroll.offset(), offset);
                }
            }
        }
    }

    #[test]
    fn drag_scroll_speed_is_per_list_and_invalid_values_restore_default() {
        let left = ListBoxScrollHandle::<u32>::default();
        let right = ListBoxScrollHandle::<u32>::default();
        left.set_drag_auto_scroll_speed(720.0);
        assert_eq!(left.drag_scroll.max_speed.get(), 720.0);
        assert_eq!(right.drag_scroll.max_speed.get(), DEFAULT_MAX_SPEED);
        left.set_drag_auto_scroll_speed(0.0);
        assert_eq!(left.drag_scroll.max_speed.get(), 0.0);
        for invalid in [-1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            left.set_drag_auto_scroll_speed(invalid);
            assert_eq!(left.drag_scroll.max_speed.get(), DEFAULT_MAX_SPEED);
        }
    }

    #[test]
    fn pending_reveal_follows_keys_across_reorder_and_ignores_removed_items() {
        let mut state = ListBoxState::try_new([1, 2, 3], |item| *item, SelectionMode::Multiple).unwrap();
        let scroll = ListBoxScrollHandle::default();
        scroll.queue_reveal(&state.apply(ListBoxInput::Navigate(ListBoxNavigation::Last)));
        state.replace_snapshot(ListBoxSnapshot::try_new([3, 1, 2], |item| *item).unwrap());
        assert_eq!(scroll.take_reveal_index(&state), Some(0));
        assert_eq!(scroll.take_reveal_index(&state), None);
        scroll.queue_reveal(&state.apply(ListBoxInput::Navigate(ListBoxNavigation::Last)));
        state.replace_snapshot(ListBoxSnapshot::try_new([3, 1], |item| *item).unwrap());
        assert_eq!(scroll.take_reveal_index(&state), None);
    }

    #[test]
    fn last_navigation_wins_and_selection_only_updates_do_not_cancel_reveal() {
        let mut state = ListBoxState::try_new([1, 2, 3], |item| *item, SelectionMode::Multiple).unwrap();
        let scroll = ListBoxScrollHandle::default();
        scroll.queue_reveal(&state.apply(ListBoxInput::Navigate(ListBoxNavigation::Last)));
        scroll.queue_reveal(&state.apply(ListBoxInput::Navigate(ListBoxNavigation::First)));
        scroll.queue_reveal(&state.apply(ListBoxInput::SelectActive));
        assert_eq!(scroll.take_reveal_index(&state), Some(0));
    }

    #[test]
    fn resize_reveals_on_either_axis_without_resetting_ordinary_scrolling() {
        let initial = size(px(300.0), px(196.0));
        assert_eq!(resize_reveal(initial, size(px(200.0), px(196.0)), Some(8)), Some(8));
        assert_eq!(resize_reveal(initial, size(px(300.0), px(100.0)), Some(8)), Some(8));
        assert_eq!(resize_reveal(initial, initial, Some(8)), None);
        assert_eq!(resize_reveal(initial, size(px(200.0), px(100.0)), None), None);
    }
}

#[cfg(all(test, feature = "test-support"))]
mod dispatch_tests {
    use super::*;
    use super::super::{ListBoxBinding, ListBoxInput, SelectionMode};
    use gpui::{App, AppContext, Focusable, Render, Window};
    struct List {
        state: ListBoxState<u32, u32>,
        binding: ListBoxBinding,
        scroll: ListBoxScrollHandle<u32>,
    }
    impl Focusable for List {
        fn focus_handle(&self, _: &App) -> gpui::FocusHandle {
            self.binding.focus_handle().clone()
        }
    }
    impl List {
        fn input(&mut self, input: ListBoxInput<u32>, _: &mut Window, cx: &mut Context<Self>) {
            let update = self.state.apply(input);
            self.scroll.handle_update(&update, cx);
        }
    }
    impl Render for List {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            self.scroll.set_wheel_focus_owner(self.binding.focus_handle().clone());
            let surface = self.binding.bind_root(
                div().id("matrix-list").w_full().h(px(150.0)),
                ListBoxAxis::Vertical,
                SelectionMode::SingleAllowNone,
                window,
                cx,
                Self::input,
            );
            let viewport = div().id("matrix-list-viewport").w_full().h(px(150.0)).overflow_y_scroll().child(
                div()
                    .flex()
                    .flex_col()
                    .children((0..100).map(|i| div().h(px(30.0)).flex_shrink_0().child(i.to_string()))),
            );
            self.scroll.bind(surface, viewport, &self.state)
        }
    }
    #[test]
    fn wheel_policy_dispatch_matrix() {
        crate::interaction_tests::matrix(
            |policy, cx| {
                cx.new(|cx| List {
                    state: ListBoxState::try_new(0..100, |i| *i, SelectionMode::SingleAllowNone).unwrap(),
                    binding: ListBoxBinding::new(cx),
                    scroll: ListBoxScrollHandle::default()
                        .wheel_scroll_policy(policy.wheel)
                        .scroll_boundary_policy(policy.boundary),
                })
            },
            |view, _| -view.scroll.scroll.offset().y.as_f32(),
            |view, cx| {
                view.scroll.scroll.set_offset(gpui::point(px(0.0), -view.scroll.scroll.max_offset().y));
                cx.notify();
            },
        );
    }
}
