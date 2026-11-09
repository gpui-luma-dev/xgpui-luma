use gpui::Focusable;
use std::{ops::Range, time::Duration};

use gpui::{
    Bounds, ClipboardItem, Context, DragMoveEvent, EntityInputHandler, KeyDownEvent, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, Pixels, Point, ScrollEvent, Task, UTF16Selection, Window, px,
};

use super::super::TextAreaTemplateHandlers;
use super::{TextArea, TextAreaDrag};
use crate::controls::text::{EditableTextPolicy, FocusNavigation, handle_key_down, select_all, word_cluster_range};

impl TextArea {
    pub(super) fn template_handlers(&self, cx: &mut Context<Self>) -> TextAreaTemplateHandlers {
        TextAreaTemplateHandlers {
            hover: Box::new(cx.listener(Self::handle_hover_changed)),
            mouse_down: Box::new(cx.listener(Self::handle_mouse_down)),
            mouse_move: Box::new(cx.listener(Self::handle_mouse_move)),
            mouse_up: Box::new(cx.listener(Self::handle_mouse_up)),
            mouse_up_out: Box::new(cx.listener(Self::handle_mouse_up)),
            click: Box::new(cx.listener(Self::handle_click)),
            key_down: Box::new(cx.listener(Self::handle_key_down)),
            drag_move: Box::new(cx.listener(Self::handle_drag_move)),
        }
    }

    pub(super) fn next_selection_scroll_epoch(&mut self) -> usize {
        self.selection_scroll_epoch += 1;
        self.selection_scroll_epoch
    }

    pub(super) fn stop_selection_autoscroll(&mut self) {
        self.selection_drag_position = None;
        self.next_selection_scroll_epoch();
        self.selection_scroll_task = Task::ready(());
    }

    pub(super) fn update_drag_selection_at(&mut self, position: Point<Pixels>, cx: &mut Context<Self>) {
        let hit_position = self.selection_hit_position(position);
        let index = self.char_offset_for_point(hit_position).min(self.model.value.chars().count());
        self.state.set_cursor(index, true);
        self.state.preferred_column = None;
        self.pause_caret_blink(cx);
        cx.notify();
    }

    pub(super) fn update_selection_autoscroll(&mut self, position: Point<Pixels>, cx: &mut Context<Self>) {
        self.selection_drag_position = Some(position);
        if self.selection_autoscroll_delta_for_point(position) == px(0.0) {
            self.next_selection_scroll_epoch();
            return;
        }

        let epoch = self.next_selection_scroll_epoch();
        self.selection_scroll_task = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_millis(33)).await;
            if let Some(this) = this.upgrade() {
                this.update(cx, |this, cx| this.tick_selection_autoscroll(epoch, cx));
            }
        });
    }

    pub(super) fn tick_selection_autoscroll(&mut self, epoch: usize, cx: &mut Context<Self>) {
        if epoch != self.selection_scroll_epoch || !self.model.enabled || !self.mouse_selecting {
            return;
        }

        let Some(position) = self.selection_drag_position else {
            return;
        };

        let delta = self.selection_autoscroll_delta_for_point(position);
        if delta == px(0.0) {
            return;
        }

        if self.scroll_by(delta) {
            self.update_drag_selection_at(position, cx);
        }

        let epoch = self.next_selection_scroll_epoch();
        self.selection_scroll_task = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_millis(33)).await;
            if let Some(this) = this.upgrade() {
                this.update(cx, |this, cx| this.tick_selection_autoscroll(epoch, cx));
            }
        });
    }

    pub(super) fn handle_hover_changed(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        self.state.set_hovered(*hovered);
        cx.notify();
    }

    pub(super) fn handle_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            cx.stop_propagation();
            return;
        }

        self.suppress_select_all_on_next_focus = true;
        self.model.pointer_focus.apply(&self.focus_handle, window, cx);
        let chars = self.model.value.chars().collect::<Vec<_>>();
        let len = chars.len();
        let index = self.char_offset_for_point(event.position).min(len);

        if event.click_count >= 3 {
            select_all(&mut self.state, len);
            self.mouse_selecting = false;
            self.stop_selection_autoscroll();
            self.pause_caret_blink(cx);
            cx.notify();
            return;
        }

        if event.click_count == 2 {
            if let Some((start, end)) = word_cluster_range(&chars, index) {
                self.state.selection_anchor = Some(start);
                self.state.cursor = end;
                self.state.preferred_column = None;
                self.mouse_selecting = false;
                self.stop_selection_autoscroll();
                self.pause_caret_blink(cx);
                cx.notify();
            }
            return;
        }

        self.state.set_cursor(index, event.modifiers.shift);
        self.state.preferred_column = None;
        self.mouse_selecting = true;
        self.selection_drag_position = Some(event.position);
        self.pause_caret_blink(cx);
        cx.notify();
    }

    pub(super) fn handle_mouse_move(&mut self, event: &MouseMoveEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.mouse_selecting {
            return;
        }

        self.update_drag_selection_at(event.position, cx);
        self.update_selection_autoscroll(event.position, cx);
    }

    pub(super) fn handle_drag_move(
        &mut self,
        event: &DragMoveEvent<TextAreaDrag>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.drag(cx).id != self.model.id || !self.model.enabled {
            return;
        }

        self.mouse_selecting = true;
        self.update_drag_selection_at(event.event.position, cx);
        self.update_selection_autoscroll(event.event.position, cx);
    }

    pub(super) fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.resize_dragging {
            self.stop_resize_drag();
            cx.notify();
            return;
        }

        self.mouse_selecting = false;
        self.stop_selection_autoscroll();
        if self.state.selection_range().is_none() {
            self.state.clear_selection();
        }
        cx.notify();
    }

    pub(super) fn handle_click(&mut self, _event: &gpui::ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            cx.stop_propagation();
        }
    }

    pub(super) fn handle_scroll_wheel(&mut self, event: &ScrollEvent, window: &mut Window, cx: &mut Context<Self>) {
        let policy = self.model.scroll_interaction;
        let bar = self.scrollbar.read(cx).focus_handle(cx);
        if !self.model.enabled
            || !policy.wheel.accepts(policy.focus_scope.focused(&self.focus_handle, Some(&bar), window, cx))
        {
            return;
        }
        let line_height = self
            .layout_cache
            .as_ref()
            .map(|cache| cache.line_height)
            .unwrap_or_else(|| px(crate::theme::LumaTypography::default().text.body.line_height));
        let Some(delta) = crate::interaction::wheel_delta(event, line_height, false) else {
            return;
        };
        let moved = self.scroll_by(-delta);
        if moved {
            self.pause_caret_blink(cx);
            cx.notify();
        }
        if policy.boundary.consumes(moved) {
            cx.stop_propagation();
        }
    }

    pub(super) fn handle_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.focus_handle.is_focused(window) {
            return;
        }
        if !self.model.enabled {
            cx.stop_propagation();
            return;
        }

        let clipboard_text = if event.keystroke.modifiers.secondary(cx) && event.keystroke.key == "v" {
            cx.read_from_clipboard().and_then(|item| item.text())
        } else {
            None
        };
        let result = handle_key_down(
            &mut self.state,
            self.model.value.as_ref(),
            event,
            clipboard_text.as_deref(),
            event.keystroke.modifiers.secondary(cx),
            EditableTextPolicy {
                multiline: true,
                submit_on_enter: false,
                strip_newlines_on_paste: false,
                allow_tab_character: false,
                clear_on_escape: self.model.clean_on_escape,
                max_clipboard_paste_bytes: self.model.max_clipboard_paste_bytes,
            },
        );

        if let Some(text) = result.clipboard_write.clone() {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }

        if let Some(navigation) = result.focus_navigation {
            match navigation {
                FocusNavigation::Next => window.focus_next(cx),
                FocusNavigation::Prev => window.focus_prev(cx),
            }
        }

        if result.handled {
            window.prevent_default();
            cx.stop_propagation();
        }

        if result.changed {
            self.model.value = result.value.into();
            self.marked_range = None;
            self.recompute_invalid();
            self.emit_change(cx);
        }

        if result.changed || result.handled {
            self.ensure_cursor_visible();
            self.pause_caret_blink(cx);
            cx.notify();
        }
    }

    pub(super) fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut char_offset = 0usize;
        let mut utf16_count = 0usize;

        for ch in self.model.value.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            char_offset += 1;
        }

        char_offset
    }

    pub(super) fn offset_to_utf16(&self, offset: usize) -> usize {
        self.model.value.chars().take(offset).map(char::len_utf16).sum()
    }

    pub(super) fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    pub(super) fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    pub(super) fn selected_range(&self) -> Range<usize> {
        self.state
            .selection_range()
            .map(|(start, end)| start..end)
            .unwrap_or(self.state.cursor..self.state.cursor)
    }

    pub(super) fn replace_char_range(
        &mut self,
        range: Range<usize>,
        new_text: &str,
        selected_range_utf16: Option<Range<usize>>,
        mark_text: bool,
        cx: &mut Context<Self>,
    ) {
        let mut chars = self.model.value.chars().collect::<Vec<_>>();
        chars.splice(range.clone(), new_text.chars());
        self.model.value = chars.into_iter().collect::<String>().into();

        let inserted_len = new_text.chars().count();
        if let Some(range_utf16) = selected_range_utf16 {
            let mut rel_start = 0usize;
            let mut rel_end = inserted_len;
            let mut utf16_count = 0usize;
            for (ix, ch) in new_text.chars().enumerate() {
                if utf16_count < range_utf16.start {
                    rel_start = ix + 1;
                }
                if utf16_count < range_utf16.end {
                    rel_end = ix + 1;
                }
                utf16_count += ch.len_utf16();
            }
            self.state.selection_anchor = Some(range.start + rel_start);
            self.state.cursor = range.start + rel_end;
        } else {
            self.state.clear_selection();
            self.state.cursor = range.start + inserted_len;
        }

        self.marked_range = if mark_text && inserted_len > 0 {
            Some(range.start..range.start + inserted_len)
        } else {
            None
        };
        self.state.preferred_column = None;
        self.recompute_invalid();
        self.emit_change(cx);
        self.ensure_cursor_visible();
        cx.notify();
    }
}

impl EntityInputHandler for TextArea {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        adjusted_range.replace(self.range_to_utf16(&range));
        Some(self.model.value.chars().skip(range.start).take(range.end.saturating_sub(range.start)).collect())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection { range: self.range_to_utf16(&self.selected_range()), reversed: false })
    }

    fn marked_text_range(&self, _window: &mut Window, _cx: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked_range.as_ref().map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.marked_range = None;
        cx.notify();
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range| self.range_from_utf16(range))
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.selected_range());
        self.replace_char_range(range, text, None, false, cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range| self.range_from_utf16(range))
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.selected_range());
        self.replace_char_range(range, new_text, new_selected_range, true, cx);
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        element_bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let range = self.range_from_utf16(&range_utf16);
        self.layout_cache
            .as_ref()
            .and_then(|cache| {
                Self::range_bounds_from_cache(cache, self.model.value.as_ref(), self.vertical_scroll, range)
            })
            .or(Some(element_bounds))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        Some(self.offset_to_utf16(self.char_offset_for_point(point)))
    }
}
