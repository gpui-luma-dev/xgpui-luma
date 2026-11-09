//! Persistent state and bindings for look-owned ListBox composition.

crate::motion_profile!(ListBoxMotion => listbox {
    scroll: crate::motion::Transition,
    drop_layout: crate::motion::Transition,
});

use std::hash::Hash;
use gpui::{App, AppContext, Context, ElementId, SharedString, Window};

use super::{
    ListBoxBinding, ListBoxError, ListBoxInput, ListBoxScrollHandle, ListBoxState, ListBoxUpdate,
    ListBoxVirtualization, SelectionPolicy,
};

/// Host callback for semantic input. Apply it to the control to queue callbacks.
pub type ListBoxInputHandler<M, K> = fn(&mut M, ListBoxInput<K>, &mut Window, &mut Context<M>);

/// Persistent SDK objects shared by successive renders. This is host-owned,
/// not a separately spawned entity; the look supplies the visual composition.
pub struct ListBoxControl<M: 'static, T, K: 'static> {
    motion_config: crate::motion::MotionConfig<ListBoxMotion>,
    /// Collection and selection state. Deliver updates from direct mutations
    /// through [`Self::handle_update`] to deliver effects and named callbacks.
    pub state: ListBoxState<T, K>,
    binding: ListBoxBinding,
    scroll: ListBoxScrollHandle<K>,
    on_input: ListBoxInputHandler<M, K>,
    item_id: fn(&K) -> ElementId,
    item_label: fn(&T) -> SharedString,
    virtualization: ListBoxVirtualization,
    pub(super) event_source: gpui::Entity<super::drag::ListBoxEventSource<K>>,
    drag_drop: Option<super::ListBoxDragDrop<M, K>>,
    drop_layout: crate::motion::DropLayoutMotion<K>,
}

/// Borrowed integration point for look-owned builders. The binding is mutable
/// only to register its persistent focus subscriptions during rendering.
pub struct ListBoxRenderParts<'a, M: 'static, T, K: 'static> {
    pub motion_config: &'a mut crate::motion::MotionConfig<ListBoxMotion>,
    /// Optional persistent SDK drag/drop binding; Looks attach native geometry.
    pub drag_drop: Option<super::ListBoxDragDropBinding<M, K>>,
    /// Persistent SDK geometry for post-drop transitions.
    pub drop_layout: crate::motion::DropLayoutMotion<K>,
    /// Current collection and selection state.
    pub state: &'a ListBoxState<T, K>,
    /// Stable keyboard and focus binding.
    pub binding: &'a mut ListBoxBinding,
    /// Stable wheel, reveal, and auto-scroll adapter.
    pub scroll: &'a ListBoxScrollHandle<K>,
    /// Host callback used by the input binding.
    pub on_input: ListBoxInputHandler<M, K>,
    /// Stable, unique element ID within this list for each key.
    pub item_id: fn(&K) -> ElementId,
    /// Accessible name of each item, independent of its visual template.
    pub item_label: fn(&T) -> SharedString,
    /// Eager or opt-in virtualized rendering.
    pub virtualization: ListBoxVirtualization,
}

impl<M: 'static, T: 'static, K: Clone + Eq + Hash + 'static> ListBoxControl<M, T, K> {
    crate::motion_methods!(ListBoxMotion, motion_config);
    pub fn motion_enabled(mut self, enabled: bool) -> Self {
        self.set_motion_enabled(enabled);
        self
    }
    /// Control-wide policy. `false` settles motion immediately and keeps [`Self::motion_profile`].
    pub fn is_motion_enabled(&self) -> bool {
        self.motion_config.enabled()
    }

    pub fn set_motion_enabled(&mut self, enabled: bool) {
        self.motion_config.set_enabled(enabled);
        self.scroll.set_motion(self.motion_config.resolve().scroll.enabled(enabled));
        self.drop_layout.set_motion_enabled(enabled);
    }

    /// Create once per list. IDs must remain stable across reorder/replacement;
    /// `on_input` must apply SDK input. Named callbacks are queued automatically;
    /// returned events remain available for manual delivery.
    pub fn new(
        state: ListBoxState<T, K>,
        on_input: ListBoxInputHandler<M, K>,
        item_id: fn(&K) -> ElementId,
        item_label: fn(&T) -> SharedString,
        cx: &mut App,
    ) -> Self {
        let motion = crate::motion::DropLayoutMotion::default();
        Self {
            state,
            motion_config: Default::default(),
            drop_layout: motion.clone(),
            event_source: cx.new(|_| super::drag::ListBoxEventSource { motion }),
            drag_drop: None,
            on_input,
            item_id,
            item_label,
            virtualization: ListBoxVirtualization::default(),
            binding: ListBoxBinding::new(cx),
            scroll: ListBoxScrollHandle::default(),
        }
    }

    /// Enable scoped, host-owned drag/drop. Register lifecycle handlers directly
    /// on this control; Look builders consume this configuration at render time.
    pub fn drag_drop(mut self, config: super::ListBoxDragDrop<M, K>) -> Self {
        self.drag_drop = Some(config);
        self
    }

    /// Retained keyed post-drop geometry and running transitions.
    pub fn drop_layout_motion(&self) -> crate::motion::DropLayoutMotion<K> {
        self.drop_layout.clone()
    }

    /// Stable source/target identity used by ListBox drop proposals and events.
    pub fn drag_drop_id(&self) -> gpui::EntityId {
        self.event_source.entity_id()
    }

    /// Opt into focus-required wheel routing. Defaults to hover scrolling.
    pub fn require_focus_for_scroll(mut self, required: bool) -> Self {
        self.scroll = self.scroll.require_focus_for_scroll(required);
        self
    }

    /// Override focus behavior independently of wheel routing.
    pub fn pointer_focus_policy(mut self, policy: crate::interaction::PointerFocusPolicy) -> Self {
        self.binding.set_pointer_focus_policy(policy);
        self
    }

    /// Configure wheel eligibility without changing boundary behavior.
    pub fn wheel_scroll_policy(self, policy: crate::interaction::WheelScrollPolicy) -> Self {
        self.scroll.set_wheel_scroll_policy(policy);
        self
    }
    /// Change policy without resetting list state.
    pub fn set_wheel_scroll_policy(&self, policy: crate::interaction::WheelScrollPolicy) {
        self.scroll.set_wheel_scroll_policy(policy);
    }

    /// Configure the retained scroll adapter.
    pub fn scroll_boundary_policy(self, policy: crate::interaction::ScrollBoundaryPolicy) -> Self {
        self.scroll.set_scroll_boundary_policy(policy);
        self
    }
    /// Change policy without resetting list state.
    pub fn set_scroll_boundary_policy(&self, policy: crate::interaction::ScrollBoundaryPolicy) {
        self.scroll.set_scroll_boundary_policy(policy);
    }

    /// Configure the retained scroll adapter.
    pub fn wheel_focus_scope(self, policy: crate::interaction::WheelFocusScope) -> Self {
        self.scroll.set_wheel_focus_scope(policy);
        self
    }
    /// Change policy without resetting list state.
    pub fn set_wheel_focus_scope(&self, policy: crate::interaction::WheelFocusScope) {
        self.scroll.set_wheel_focus_scope(policy);
    }

    /// Opt into uniform or measured virtualization; eager rendering remains the default.
    /// Item templates must keep durable state in their host/entities, since
    /// off-screen element instances are not retained.
    pub fn virtualization(mut self, policy: ListBoxVirtualization) -> Self {
        self.virtualization = policy;
        self
    }

    /// Bring the item with this stable key into view on the next layout.
    /// An already fully visible item does not move. Oversized items show their
    /// leading edge. Does not change selection, active item, or focus; works
    /// without focus. Unknown/removed keys are ignored. The latest request wins.
    pub fn scroll_to(&self, item_key: K, cx: &mut Context<M>) {
        self.scroll.scroll_to(item_key, cx);
    }

    /// Center this item's midpoint in the visible viewport on the next layout.
    /// Uses the list's scrolling axis and clamps at collection boundaries, where
    /// exact centering may be impossible. Does not change selection or focus.
    /// This is a one-time request, not a scrolling or selection policy.
    pub fn scroll_to_center(&self, item_key: K, cx: &mut Context<M>) {
        self.scroll.scroll_to_center(item_key, cx);
    }

    /// Smoothly bring this item into view using shared scroll motion (200ms).
    /// Does not change selection/focus. New input or another request cancels it.
    pub fn scroll_to_smooth(&self, item_key: K, cx: &mut Context<M>) {
        self.scroll.scroll_to_smooth(item_key, cx);
    }

    /// Smoothly center this item in the viewport, clamped at collection boundaries.
    /// Measured/virtualized destinations track the item's actual layout.
    pub fn scroll_to_center_smooth(&self, item_key: K, cx: &mut Context<M>) {
        self.scroll.scroll_to_center_smooth(item_key, cx);
    }

    /// Invalidate one or all content-sized measurements after external content
    /// or theme changes. Snapshot replacement and width changes are automatic.
    pub fn invalidate_measurements(&self, key: Option<&K>, cx: &mut Context<M>) {
        self.scroll.invalidate_measurements(key);
        cx.notify();
    }

    /// Configure the existing basic linear edge-scrolling function, in logical
    /// pixels per second. See [`ListBoxScrollHandle::set_drag_auto_scroll_speed`].
    pub fn set_drag_auto_scroll_speed(&self, pixels_per_second: f32) {
        self.scroll.set_drag_auto_scroll_speed(pixels_per_second);
    }

    /// Apply semantic input, queue rendering/reveal effects and named callbacks.
    /// Events are also returned for optional manual handling.
    pub fn apply(&mut self, input: ListBoxInput<K>, cx: &mut Context<M>) -> ListBoxUpdate<K> {
        if !matches!(input, ListBoxInput::Focus(_)) {
            self.drop_layout.cancel();
        }
        let update = self.state.apply(input);
        self.handle_update(&update, cx);
        update
    }

    /// Reconcile a runtime policy change and queue its rendering/reveal effects.
    pub fn set_selection_policy(&mut self, policy: SelectionPolicy, cx: &mut Context<M>) -> ListBoxUpdate<K> {
        let update = self.state.set_selection_policy(policy);
        self.handle_update(&update, cx);
        update
    }

    /// Display an ordered subset of source keys and deliver rendering effects.
    /// Hidden selections are preserved. Queues named callbacks and returns events.
    pub fn set_projection(
        &mut self,
        keys: impl IntoIterator<Item = K>,
        cx: &mut Context<M>,
    ) -> Result<ListBoxUpdate<K>, ListBoxError> {
        let update = self.state.set_projection(keys)?;
        self.drop_layout.cancel();
        self.handle_update(&update, cx);
        Ok(update)
    }

    /// Restore source order and visibility, preserving keyed selection.
    pub fn reset_projection(&mut self, cx: &mut Context<M>) -> ListBoxUpdate<K> {
        self.drop_layout.cancel();
        let update = self.state.reset_projection();
        self.handle_update(&update, cx);
        update
    }

    /// Deliver effects and queue named callbacks after a committed state mutation.
    /// Called automatically by control methods. After direct `state` mutation,
    /// call this exactly once with the returned update. Callbacks run after the
    /// current owner borrow ends, in event and registration order.
    pub fn handle_update(&self, update: &ListBoxUpdate<K>, cx: &mut Context<M>) {
        self.scroll.handle_update(update, cx);
        super::drag::emit_batch(self.event_source.downgrade(), update.events.clone(), cx);
    }

    /// Borrow existing state/handles for composition without recreating them.
    pub fn render_parts(&mut self) -> ListBoxRenderParts<'_, M, T, K> {
        self.scroll.set_wheel_focus_owner(self.binding.focus_handle().clone());
        let drag_drop = self.drag_drop.clone().map(|config| super::ListBoxDragDropBinding {
            source: self.event_source.downgrade(),
            config,
            preview_style: None,
        });
        ListBoxRenderParts {
            motion_config: &mut self.motion_config,
            drop_layout: self.drop_layout.clone(),
            drag_drop,
            state: &self.state,
            binding: &mut self.binding,
            scroll: &self.scroll,
            on_input: self.on_input,
            item_id: self.item_id,
            item_label: self.item_label,
            virtualization: self.virtualization,
        }
    }
}
