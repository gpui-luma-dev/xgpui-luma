use gpui::{
    App, Bounds, ClickEvent, Context, FocusOutEvent, Modifiers, MouseDownEvent, MouseUpEvent, Pixels, SharedString,
    Window,
};

use super::super::model::{
    ControlGroupArrowAxis, ControlGroupFocusStrategy, ControlGroupFocusTarget, ControlGroupItemLike,
    ControlGroupStateMode, ControlSelectionMode,
};
use super::super::template::ControlGroupTemplateHandlers;
use super::selection::{
    ControlGroupDirection, compute_next_selected_ids, enabled_item_index_by_id, first_enabled_index,
    last_enabled_index, next_enabled_index, next_enabled_index_no_wrap, normalize_model, selected_ids_contain,
};
use super::{ControlGroupControl, ControlGroupEvent};
use crate::key_handling::{
    ActivateControl, SelectFirstItem, SelectLastItem, SelectNextItem, SelectNextRow, SelectPreviousItem,
    SelectPreviousRow,
};

impl<T> ControlGroupControl<T>
where
    T: ControlGroupItemLike + 'static,
{
    pub(super) fn template_handlers(&self, cx: &mut Context<Self>) -> ControlGroupTemplateHandlers {
        ControlGroupTemplateHandlers {
            item_bounds: (0..self.model.items.len())
                .map(|index| {
                    Box::new(cx.listener(move |this, bounds, _window, cx| {
                        this.handle_item_bounds(index, *bounds, cx);
                    })) as _
                })
                .collect(),
            item_hovers: (0..self.model.items.len())
                .map(|index| {
                    Box::new(cx.listener(move |this, hovered, _window, cx| {
                        this.handle_item_hover(index, *hovered, cx);
                    })) as _
                })
                .collect(),
            item_mouse_downs: (0..self.model.items.len())
                .map(|index| {
                    Box::new(cx.listener(move |this, event, window, cx| {
                        this.handle_item_mouse_down(index, event, window, cx);
                    })) as _
                })
                .collect(),
            item_mouse_ups: (0..self.model.items.len())
                .map(|_| Box::new(cx.listener(Self::handle_item_mouse_up)) as _)
                .collect(),
            item_mouse_up_outs: (0..self.model.items.len())
                .map(|_| Box::new(cx.listener(Self::handle_item_mouse_up)) as _)
                .collect(),
            item_clicks: (0..self.model.items.len())
                .map(|index| {
                    Box::new(cx.listener(move |this, event, _window, cx| {
                        this.handle_item_click(index, event, cx);
                    })) as _
                })
                .collect(),
        }
    }

    pub(super) fn can_use_item(&self, index: usize) -> bool {
        self.model.enabled && self.model.items.get(index).is_some_and(ControlGroupItemLike::is_enabled)
    }

    pub(super) fn active_index(&self) -> Option<usize> {
        enabled_item_index_by_id(&self.model.items, self.model.active_id.as_ref())
    }

    pub(super) fn commit_toggle_index(&mut self, index: usize, cx: &mut Context<Self>) -> bool {
        self.commit_toggle_index_with_modifiers(index, Modifiers::default(), cx)
    }

    pub(super) fn commit_toggle_index_with_modifiers(
        &mut self,
        index: usize,
        modifiers: Modifiers,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.can_use_item(index) {
            return false;
        }

        let Some(item) = self.model.items.get(index) else {
            return false;
        };

        let item_id = item.id().clone();
        let current = self.model.effective_selected_ids();
        let next_selected_ids = if self.model.selection_mode == ControlSelectionMode::Multiple
            && !self.model.independent_toggles
            && !(modifiers.secondary(cx) || modifiers.shift)
        {
            vec![item.id().clone()]
        } else {
            compute_next_selected_ids(&self.model.items, current, self.model.selection_mode, index)
        };
        let selected = selected_ids_contain(&next_selected_ids, &item_id);

        if next_selected_ids == current {
            cx.emit(ControlGroupEvent::Activate { activated_id: item_id });
            return false;
        }

        if self.model.state_mode == ControlGroupStateMode::Unmanaged {
            self.model.default_selected_ids = next_selected_ids.clone();
            normalize_model(&mut self.model);
        }

        self.sync_selection_transitions_to(&next_selected_ids);

        cx.emit(ControlGroupEvent::Change { changed_id: item_id.clone(), selected, selected_ids: next_selected_ids });
        cx.emit(ControlGroupEvent::Activate { activated_id: item_id });
        cx.notify();
        true
    }

    pub(super) fn focus_target_for_index(
        &self,
        index: usize,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<ControlGroupFocusTarget> {
        let item = self.model.items.get(index)?;
        let provider = self.model.focus_target_provider.as_ref()?;
        provider(item, window, cx)
    }

    pub(super) fn focus_item_for_strategy(&self, index: usize, window: &mut Window, cx: &mut App) {
        match self.model.focus_strategy {
            ControlGroupFocusStrategy::ActiveDescendant => {
                self.focus_handle.focus(window, cx);
            }
            ControlGroupFocusStrategy::RovingItemFocus => {
                if let Some(target) = self.focus_target_for_index(index, window, cx) {
                    target.focus_handle.focus(window, cx);
                } else {
                    self.focus_handle.focus(window, cx);
                }
            }
        }
    }

    pub(super) fn active_focus_target(&self, window: &mut Window, cx: &mut App) -> Option<ControlGroupFocusTarget> {
        let index = self.active_index()?;
        self.focus_target_for_index(index, window, cx)
    }

    pub(super) fn group_handles_arrow_action(
        &self,
        axis: ControlGroupArrowAxis,
        window: &mut Window,
        cx: &mut App,
    ) -> bool {
        if self.model.focus_strategy == ControlGroupFocusStrategy::ActiveDescendant {
            return true;
        }

        let Some(target) = self.active_focus_target(window, cx) else {
            return true;
        };

        if !target.focus_handle.is_focused(window) {
            return true;
        }

        !target.arrow_policy.child_owns_axis(axis)
    }

    pub(super) fn set_active_id_from_focus(&mut self, item_id: SharedString, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        if enabled_item_index_by_id(&self.model.items, Some(&item_id)).is_none() {
            return;
        }

        if self.model.active_id.as_ref() == Some(&item_id) {
            return;
        }

        self.model.active_id = Some(item_id.clone());
        cx.emit(ControlGroupEvent::ItemFocused { item_id });
        cx.notify();
    }

    pub(super) fn emit_focus_changed(&mut self, focused: bool, cx: &mut Context<Self>) -> bool {
        let focused = self.model.enabled && focused;
        if self.emitted_focused == focused {
            return false;
        }

        self.emitted_focused = focused;
        cx.emit(ControlGroupEvent::FocusChanged { focused });
        true
    }

    pub(super) fn clear_focus_subscriptions(&mut self) {
        self.focus_in_subscription = None;
        self.focus_out_subscription = None;
    }

    pub(super) fn clear_item_focus_subscriptions(&mut self) {
        self.item_focus_subscriptions.clear();
        self.item_focus_subscription_keys.clear();
    }

    pub(super) fn any_item_focus_target_focused(&self, window: &Window) -> bool {
        self.item_focus_subscription_keys.iter().any(|(_, focus_handle)| focus_handle.is_focused(window))
    }

    pub(super) fn ensure_item_focus_subscriptions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.model.focus_strategy != ControlGroupFocusStrategy::RovingItemFocus
            || self.model.focus_target_provider.is_none()
        {
            self.clear_item_focus_subscriptions();
            return;
        }

        let mut desired_keys = Vec::new();
        for index in 0..self.model.items.len() {
            let Some(item) = self.model.items.get(index) else {
                continue;
            };
            if !item.is_enabled() {
                continue;
            }
            if let Some(target) = self.focus_target_for_index(index, window, cx) {
                desired_keys.push((item.id().clone(), target.focus_handle));
            }
        }

        if desired_keys == self.item_focus_subscription_keys {
            return;
        }

        self.item_focus_subscriptions.clear();
        self.item_focus_subscription_keys = desired_keys;

        for (item_id, focus_handle) in self.item_focus_subscription_keys.clone() {
            self.item_focus_subscriptions.push(cx.on_focus_in(&focus_handle, window, {
                let item_id = item_id.clone();
                move |this, _window, cx| {
                    let focus_changed = this.emit_focus_changed(true, cx);
                    this.set_active_id_from_focus(item_id.clone(), cx);
                    if focus_changed {
                        cx.notify();
                    }
                }
            }));
            self.item_focus_subscriptions.push(cx.on_focus_out(
                &focus_handle,
                window,
                move |this, _: FocusOutEvent, window, cx| {
                    if !this.focus_handle.is_focused(window)
                        && !this.any_item_focus_target_focused(window)
                        && this.emit_focus_changed(false, cx)
                    {
                        cx.notify();
                    }
                },
            ));
        }
    }

    pub(super) fn set_active_index(
        &mut self,
        index: usize,
        window: Option<&mut Window>,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.can_use_item(index) {
            return false;
        }

        let Some(item) = self.model.items.get(index) else {
            return false;
        };

        if self.model.active_id.as_ref().is_some_and(|active_id| active_id == item.id()) {
            return false;
        }

        let item_id = item.id().clone();
        self.model.active_id = Some(item_id.clone());
        cx.emit(ControlGroupEvent::ItemFocused { item_id });
        if let Some(window) = window {
            self.focus_item_for_strategy(index, window, cx);
        }
        cx.notify();
        true
    }

    pub(super) fn handle_item_hover(&mut self, index: usize, hovered: bool, cx: &mut Context<Self>) {
        if !self.can_use_item(index) {
            return;
        }

        if hovered {
            if self.hovered_item != Some(index) {
                self.hovered_item = Some(index);
                cx.notify();
            }
        } else if self.hovered_item == Some(index) {
            self.hovered_item = None;
            if self.pressed_item == Some(index) {
                self.pressed_item = None;
            }
            cx.notify();
        }
    }

    pub(super) fn handle_item_bounds(&mut self, index: usize, bounds: Bounds<Pixels>, cx: &mut Context<Self>) {
        let Some(item) = self.model.items.get(index) else {
            return;
        };
        if self.item_bounds.len() != self.model.items.len() {
            self.item_bounds.resize(self.model.items.len(), None);
        }
        if self.item_bounds.get(index).copied().flatten() == Some(bounds) {
            return;
        }
        self.item_bounds[index] = Some(bounds);
        cx.emit(ControlGroupEvent::ItemBoundsChanged { item_id: item.id().clone(), bounds });
    }

    pub(super) fn handle_item_mouse_down(
        &mut self,
        index: usize,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.can_use_item(index) {
            self.pressed_item = Some(index);
            let changed = self.set_active_index(index, Some(window), cx);
            if !changed {
                self.focus_item_for_strategy(index, window, cx);
            }
            cx.notify();
        }
    }

    pub(super) fn handle_item_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.pressed_item.is_some() {
            self.pressed_item = None;
            cx.notify();
        }
    }

    pub(super) fn handle_item_click(&mut self, index: usize, event: &ClickEvent, cx: &mut Context<Self>) {
        if event.is_keyboard() {
            return;
        }

        self.commit_toggle_index_with_modifiers(index, event.modifiers(), cx);
    }

    pub(super) fn move_active(
        &mut self,
        direction: ControlGroupDirection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.model.enabled {
            return;
        }

        if let Some(next_index) = next_enabled_index(&self.model.items, self.active_index(), direction) {
            let changed = self.set_active_index(next_index, Some(window), cx);
            if changed && self.model.selection_follows_active {
                self.commit_toggle_index(next_index, cx);
            }
        }
    }

    pub(super) fn move_active_to_boundary(&mut self, first: bool, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        let next_index = if first {
            first_enabled_index(&self.model.items)
        } else {
            last_enabled_index(&self.model.items)
        };

        if let Some(next_index) = next_index {
            let changed = self.set_active_index(next_index, Some(window), cx);
            if changed && self.model.selection_follows_active {
                self.commit_toggle_index(next_index, cx);
            }
        }
    }

    pub(super) fn handle_select_previous_item(
        &mut self,
        _: &SelectPreviousItem,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.group_handles_arrow_action(ControlGroupArrowAxis::Horizontal, window, cx) {
            self.move_active(ControlGroupDirection::Previous, window, cx);
        }
    }

    pub(super) fn handle_select_next_item(&mut self, _: &SelectNextItem, window: &mut Window, cx: &mut Context<Self>) {
        if self.group_handles_arrow_action(ControlGroupArrowAxis::Horizontal, window, cx) {
            self.move_active(ControlGroupDirection::Next, window, cx);
        }
    }

    pub(super) fn handle_select_previous_row(
        &mut self,
        _: &SelectPreviousRow,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.group_handles_arrow_action(ControlGroupArrowAxis::Vertical, window, cx) {
            self.move_active(ControlGroupDirection::Previous, window, cx);
        }
    }

    pub(super) fn handle_select_next_row(&mut self, _: &SelectNextRow, window: &mut Window, cx: &mut Context<Self>) {
        if self.group_handles_arrow_action(ControlGroupArrowAxis::Vertical, window, cx) {
            self.move_active(ControlGroupDirection::Next, window, cx);
        }
    }

    pub(super) fn handle_select_first_item(
        &mut self,
        _: &SelectFirstItem,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.group_handles_arrow_action(ControlGroupArrowAxis::Horizontal, window, cx) {
            self.move_active_to_boundary(true, window, cx);
        }
    }

    pub(super) fn handle_select_last_item(&mut self, _: &SelectLastItem, window: &mut Window, cx: &mut Context<Self>) {
        if self.group_handles_arrow_action(ControlGroupArrowAxis::Horizontal, window, cx) {
            self.move_active_to_boundary(false, window, cx);
        }
    }

    pub(super) fn handle_activate_control(
        &mut self,
        _: &ActivateControl,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.model.enabled {
            return;
        }

        if let Some(index) = self.active_index() {
            self.commit_toggle_index(index, cx);
        }
    }

    pub(super) fn handle_group_focus_entry(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.emit_focus_changed(true, cx) {
            cx.notify();
        }
    }

    pub(super) fn handle_group_focus_out(&mut self, _: FocusOutEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.any_item_focus_target_focused(window) && self.emit_focus_changed(false, cx) {
            cx.notify();
        }
    }

    pub(super) fn handle_group_next_focus(
        &mut self,
        _: &crate::focus::NextFocus,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.model.focus_strategy != ControlGroupFocusStrategy::RovingItemFocus {
            return;
        }

        if let Some(next_index) =
            next_enabled_index_no_wrap(&self.model.items, self.active_index(), ControlGroupDirection::Next)
        {
            let changed = self.set_active_index(next_index, Some(window), cx);
            if changed && self.model.selection_follows_active {
                self.commit_toggle_index(next_index, cx);
            }
            cx.stop_propagation();
        }
    }

    pub(super) fn handle_group_prev_focus(
        &mut self,
        _: &crate::focus::PreviousFocus,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.model.focus_strategy != ControlGroupFocusStrategy::RovingItemFocus {
            return;
        }

        if let Some(prev_index) =
            next_enabled_index_no_wrap(&self.model.items, self.active_index(), ControlGroupDirection::Previous)
        {
            let changed = self.set_active_index(prev_index, Some(window), cx);
            if changed && self.model.selection_follows_active {
                self.commit_toggle_index(prev_index, cx);
            }
            cx.stop_propagation();
        }
    }
}
