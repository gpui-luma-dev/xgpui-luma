use crate::infra::focus_report::ReportsFocus as _;
use gpui::{ClickEvent, Context, FocusOutEvent, MouseDownEvent, MouseUpEvent, ScrollEvent, TouchPhase, Window, px};

use super::selection::{TableDirection, next_enabled_index};
use super::{TableControl, TableEvent, TableSelectionMode};
use crate::key_handling::{
    ActivateControl, DecreaseValueLarge, IncreaseValueLarge, SelectFirstItem, SelectLastItem, SelectNextItem,
    SelectPreviousItem,
};

impl<T> TableControl<T>
where
    T: 'static,
{
    /// Only table-owned keys while the table itself has focus. Embedded controls
    /// keep their shortcuts and Space activation.
    pub(super) fn handle_key_down(&mut self, event: &gpui::KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.focus_handle.is_focused(window) {
            return;
        }
        if event.keystroke.key == "escape" && self.column_resize.is_some() {
            self.cancel_column_resize(cx);
            cx.stop_active_drag(window);
            cx.stop_propagation();
            return;
        }
        if event.keystroke.key == "escape" && self.active_row_drag.is_some() {
            self.cancel_row_drag(cx);
            cx.stop_active_drag(window);
            cx.stop_propagation();
            return;
        }
        let modifiers = event.keystroke.modifiers;
        let key = event.keystroke.key.as_str();
        let toggle = modifiers.platform || modifiers.control;
        let extended = self.model.selection_mode == TableSelectionMode::Extended;
        if modifiers.alt || modifiers.function {
            return;
        }
        if toggle
            && key.eq_ignore_ascii_case("a")
            && matches!(self.model.selection_mode, TableSelectionMode::Multiple | TableSelectionMode::Extended)
        {
            self.select_all(modifiers.shift, cx);
        } else if extended && modifiers.shift && matches!(key, "up" | "down" | "home" | "end") {
            self.clear_pointer_interaction(cx);
            let next = match key {
                "home" => super::selection::first_enabled_index(&self.model.items, self.model.row_enabled.as_ref()),
                "end" => super::selection::last_enabled_index(&self.model.items, self.model.row_enabled.as_ref()),
                _ => next_enabled_index(
                    &self.model.items,
                    self.model.row_enabled.as_ref(),
                    self.model.active_index,
                    if key == "up" {
                        TableDirection::Previous
                    } else {
                        TableDirection::Next
                    },
                ),
            };
            if let Some(index) = next {
                self.select_index(index, toggle, true, cx);
            }
        } else if extended && key == "space" {
            if let Some(index) = self.model.active_index {
                self.select_index(index, toggle, modifiers.shift, cx);
            }
        } else if modifiers.shift {
            return;
        } else if modifiers.platform && matches!(key, "up" | "down") {
            self.move_active_to_boundary(key == "up", cx);
        } else if modifiers.control && matches!(key, "a" | "e") {
            self.move_active_to_boundary(key == "a", cx);
        } else if toggle && !(extended && matches!(key, "up" | "down" | "home" | "end")) {
            return;
        } else {
            match key {
                "up" => self.move_active(TableDirection::Previous, cx),
                "down" => self.move_active(TableDirection::Next, cx),
                "home" => self.move_active_to_boundary(true, cx),
                "end" => self.move_active_to_boundary(false, cx),
                "space" | "enter" => self.handle_activate_control(&ActivateControl, window, cx),
                "pageup" => self.handle_decrease_value_large(&DecreaseValueLarge, window, cx),
                "pagedown" => self.handle_increase_value_large(&IncreaseValueLarge, window, cx),
                _ => return,
            }
        }
        cx.stop_propagation();
    }

    pub(super) fn clear_pointer_interaction(&mut self, cx: &mut Context<Self>) {
        if self.hovered_index.is_none() && self.pressed_index.is_none() {
            return;
        }

        self.clear_hovered_index(cx);
        self.pressed_index = None;
        cx.notify();
    }

    pub(super) fn clear_hovered_index(&mut self, cx: &mut Context<Self>) {
        if let Some(index) = self.hovered_index.take() {
            cx.emit(TableEvent::RowHoverChanged { index, hovered: false });
        }
    }

    pub(super) fn handle_item_hover(&mut self, index: usize, hovered: bool, cx: &mut Context<Self>) {
        if !self.can_use_item(index) {
            return;
        }

        if hovered {
            if self.hovered_index != Some(index) {
                self.clear_hovered_index(cx);
                self.hovered_index = Some(index);
                cx.emit(TableEvent::RowHoverChanged { index, hovered: true });
                cx.notify();
            }
        } else if self.hovered_index == Some(index) {
            self.clear_hovered_index(cx);
            if self.pressed_index == Some(index) {
                self.pressed_index = None;
            }
            cx.notify();
        }
    }

    pub(super) fn handle_item_mouse_down(
        &mut self,
        index: usize,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.can_use_item(index) {
            self.pressed_index = Some(index);
            self.model.pointer_focus.apply(&self.focus_handle, window, cx);
            cx.notify();
        }
    }

    pub(super) fn handle_item_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.pressed_index.is_some() {
            self.pressed_index = None;
            cx.notify();
        }
    }

    pub(super) fn handle_item_click(&mut self, index: usize, event: &ClickEvent, cx: &mut Context<Self>) {
        if event.is_keyboard() {
            return;
        }

        if !self.can_use_item(index) {
            return;
        }
        if self.model.select_on_row_click {
            let modifiers = event.modifiers();
            self.select_index(index, modifiers.platform || modifiers.control, modifiers.shift, cx);
        } else {
            self.set_active_index_internal(Some(index), None, cx);
        }
    }

    pub(super) fn move_active(&mut self, direction: TableDirection, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }
        self.clear_pointer_interaction(cx);
        if let Some(next_index) = next_enabled_index(
            self.model.items.as_slice(),
            self.model.row_enabled.as_ref(),
            self.model.active_index,
            direction,
        ) {
            self.set_active_index_internal(Some(next_index), Some(direction), cx);
        }
    }

    pub(super) fn move_active_to_boundary(&mut self, first: bool, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }
        self.clear_pointer_interaction(cx);
        let len = self.model.items.len();
        if len == 0 {
            return;
        }

        let next_index = if first { 0 } else { len - 1 };
        self.set_active_index_internal(Some(next_index), None, cx);
    }

    pub(super) fn handle_select_previous_item(
        &mut self,
        _: &SelectPreviousItem,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.move_active(TableDirection::Previous, cx);
    }

    pub(super) fn handle_select_next_item(&mut self, _: &SelectNextItem, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_active(TableDirection::Next, cx);
    }

    pub(super) fn handle_select_first_item(
        &mut self,
        _: &SelectFirstItem,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.move_active_to_boundary(true, cx);
    }

    pub(super) fn handle_select_last_item(&mut self, _: &SelectLastItem, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_active_to_boundary(false, cx);
    }

    pub(super) fn handle_activate_control(
        &mut self,
        _: &ActivateControl,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.clear_pointer_interaction(cx);
        if let Some(index) = self.model.active_index {
            self.select_index(index, false, false, cx);
        }
    }

    pub(super) fn handle_decrease_value_large(
        &mut self,
        _: &DecreaseValueLarge,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.model.enabled {
            return;
        }

        self.clear_pointer_interaction(cx);

        if self.is_paged() {
            self.prev_page(cx);
        } else {
            self.list_state.scroll_by(-self.page_scroll_distance());
            if self.is_scroll_snap() {
                self.snap_scroll_position();
            }
            self.emit_scroll_changed_if_needed(cx);
            cx.notify();
        }
    }

    pub(super) fn handle_increase_value_large(
        &mut self,
        _: &IncreaseValueLarge,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.model.enabled {
            return;
        }

        self.clear_pointer_interaction(cx);

        if self.is_paged() {
            self.next_page(cx);
        } else {
            self.list_state.scroll_by(self.page_scroll_distance());
            if self.is_scroll_snap() {
                self.snap_scroll_position();
            }
            self.emit_scroll_changed_if_needed(cx);
            cx.notify();
        }
    }

    pub(super) fn handle_scroll_wheel(&mut self, event: &ScrollEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.is_paged() || !self.model.enabled {
            return;
        }

        let delta_y = event.delta.pixel_delta(px(20.0)).y;
        let is_scroll_end = matches!(event.touch_phase, TouchPhase::Ended);
        if is_scroll_end {
            if !self.wheel_gesture_active {
                return;
            }
            self.wheel_gesture_active = false;
        } else if delta_y != px(0.0) {
            self.wheel_gesture_active = event.delta.precise();
        }
        if delta_y == px(0.0) && !is_scroll_end {
            return;
        }

        if !self.is_scroll_snap() {
            if self.emit_scroll_changed_if_needed(cx) {
                cx.notify();
            }
            return;
        }

        if event.delta.precise() {
            // Trackpad: let GPUI's list scroll smoothly during the gesture, then snap once
            // when the gesture ends. Snapping on every delta fought partial scroll progress
            // and felt erratic (especially with natural scrolling).
            if is_scroll_end {
                self.snap_scroll_position();
                self.emit_scroll_changed_if_needed(cx);
                cx.notify();
            }
            return;
        }

        // Mouse wheel (line deltas): GPUI scrolls first in bubble order; align to the nearest row.
        self.snap_scroll_position();
        self.emit_scroll_changed_if_needed(cx);
        cx.notify();
    }

    pub(super) fn handle_focus_in(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.model.enabled && self.emit_focus_changed(true, cx) {
            cx.notify();
        }
    }

    pub(super) fn handle_focus_out(&mut self, _: FocusOutEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.cancel_column_resize(cx);
        if self.emit_focus_changed(false, cx) {
            cx.notify();
        }
    }
}
