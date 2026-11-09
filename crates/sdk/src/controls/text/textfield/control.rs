use crate::infra::attachments::{AttachmentHost, AttachmentTarget};
use crate::theme::InteractionState;
use std::sync::Arc;
use std::{ops::Range, time::Duration};

use gpui::{
    App, Bounds, ClipboardItem, Context, DragMoveEvent, ElementInputHandler, Empty, EntityInputHandler, EventEmitter,
    FocusHandle, Focusable, IntoElement, KeyDownEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point,
    Render, SharedString, ShapedLine, Task, TextRun, UTF16Selection, Window, canvas, div, font, point, px, prelude::*,
};

use super::{
    TextFieldLook, TextFieldBuilder, TextFieldRenderModel, TextFieldState, TextFieldTemplateHandlers,
    model::TextFieldModel,
};
use crate::controls::text::{
    EditableTextPolicy, FocusNavigation, cap_clipboard_paste, handle_key_down, select_all, word_cluster_range,
};
use crate::theme::observe_theme_revision;

#[derive(Clone)]
struct TextFieldLayoutCache {
    bounds: Bounds<Pixels>,
    text_viewport: Bounds<Pixels>,
    text_origin: Point<Pixels>,
    content_width: Pixels,
    line: ShapedLine,
}

const TEXTFIELD_TRAILING_HITBOX_WIDTH: f32 = 4.0;
const TEXTFIELD_SCROLL_REVEAL_PADDING: f32 = 24.0;

#[derive(Clone)]
struct TextFieldLayoutPreview {
    character_offsets: Vec<f32>,
    look: TextFieldLook,
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum TextFieldEvent {
    Change { value: String },
    Submit { value: String },
    FocusChanged { focused: bool },
    EnabledChanged { enabled: bool },
}

#[derive(Clone, Debug)]
pub struct TextFieldDrag {
    id: SharedString,
}

impl TextFieldDrag {
    pub(crate) fn new(id: SharedString) -> Self {
        Self { id }
    }
}

impl Render for TextFieldDrag {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

pub struct TextFieldControl {
    attachments: AttachmentHost,
    model: TextFieldModel,
    state: TextFieldState,
    focus_handle: FocusHandle,
    theme_epoch: u64,
    change_count: usize,
    submit_count: usize,
    focus_count: usize,
    blur_count: usize,
    caret_visible: bool,
    caret_paused: bool,
    caret_epoch: usize,
    caret_task: Task<()>,
    mouse_selecting: bool,
    suppress_select_all_on_next_focus: bool,
    marked_range: Option<Range<usize>>,
    horizontal_scroll: Pixels,
    layout_cache: Option<TextFieldLayoutCache>,
}

impl EventEmitter<TextFieldEvent> for TextFieldControl {}

impl TextFieldControl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> TextFieldBuilder {
        TextFieldBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: TextFieldBuilder, cx: &mut Context<Self>) -> Self {
        let state = TextFieldState { cursor: builder.model.value.chars().count(), ..Default::default() };

        let mut this = Self {
            attachments: AttachmentHost::with_accessible_role(gpui::Role::TextInput),
            model: builder.model,
            state,
            focus_handle: cx.focus_handle().tab_stop(true),
            theme_epoch: 0,
            change_count: 0,
            submit_count: 0,
            focus_count: 0,
            blur_count: 0,
            caret_visible: false,
            caret_paused: false,
            caret_epoch: 0,
            caret_task: Task::ready(()),
            mouse_selecting: false,
            suppress_select_all_on_next_focus: false,
            marked_range: None,
            horizontal_scroll: px(0.0),
            layout_cache: None,
        };
        this.focus_handle = this.focus_handle.clone().tab_stop(this.model.enabled && this.model.tab_stop);
        this.recompute_invalid();
        observe_theme_revision(cx, |this, cx| {
            this.layout_cache = None;
            this.theme_epoch = this.theme_epoch.wrapping_add(1);
            cx.notify();
        })
        .detach();
        this
    }

    pub fn value(&self) -> String {
        self.model.value.to_string()
    }

    pub fn state(&self) -> TextFieldState {
        self.state
    }

    pub fn change_count(&self) -> usize {
        self.change_count
    }

    pub fn submit_count(&self) -> usize {
        self.submit_count
    }

    pub fn focus_count(&self) -> usize {
        self.focus_count
    }

    pub fn blur_count(&self) -> usize {
        self.blur_count
    }

    pub fn is_focused(&self) -> bool {
        self.state.focused
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }

        self.model.enabled = enabled;
        self.focus_handle = self.focus_handle.clone().tab_stop(enabled && self.model.tab_stop);
        if !enabled {
            self.state.set_hovered(false);
            self.mouse_selecting = false;
        }
        cx.emit(TextFieldEvent::EnabledChanged { enabled });
        cx.notify();
    }

    pub fn set_invalid(&mut self, invalid: bool, cx: &mut Context<Self>) {
        if self.state.invalid != invalid {
            self.state.set_invalid(invalid);
            cx.notify();
        }
    }

    pub fn set_tab_stop(&mut self, tab_stop: bool, cx: &mut Context<Self>) {
        self.model.tab_stop = tab_stop;
        self.focus_handle = self.focus_handle.clone().tab_stop(self.model.enabled && tab_stop);
        cx.notify();
    }

    pub fn set_size(&mut self, size: crate::theme::ControlSize, cx: &mut Context<Self>) {
        self.model.size = size;
        self.layout_cache = None;
        cx.notify();
    }

    pub fn set_value(&mut self, value: impl Into<String>, cx: &mut Context<Self>) {
        self.model.value = value.into().into();
        self.state.cursor = self.model.value.chars().count();
        self.state.clear_selection();
        self.state.preferred_column = None;
        self.marked_range = None;
        self.recompute_invalid();
        cx.notify();
    }

    pub fn select_all(&mut self, cx: &mut Context<Self>) {
        select_all(&mut self.state, self.model.value.chars().count());
        self.state.preferred_column = None;
        cx.notify();
    }

    pub fn copy_selection(&self, cx: &mut Context<Self>) {
        let range = self.selected_range();
        if range.start < range.end {
            let text = self.model.value.chars().skip(range.start).take(range.len()).collect::<String>();
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
    }

    pub fn cut_selection(&mut self, cx: &mut Context<Self>) {
        let range = self.selected_range();
        if range.start >= range.end {
            return;
        }

        let mut chars = self.model.value.chars().collect::<Vec<_>>();
        let text = chars[range.clone()].iter().collect::<String>();
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        chars.drain(range.clone());
        self.model.value = chars.into_iter().collect::<String>().into();
        self.state.cursor = range.start;
        self.state.clear_selection();
        self.state.preferred_column = None;
        self.marked_range = None;
        self.emit_change(cx);
        cx.notify();
    }

    pub fn paste(&mut self, cx: &mut Context<Self>) {
        let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) else {
            return;
        };
        let text = cap_clipboard_paste(&text, self.model.max_clipboard_paste_bytes).replace(['\n', '\r'], "");
        if text.is_empty() {
            return;
        }

        let range = self.selected_range();
        let mut chars = self.model.value.chars().collect::<Vec<_>>();
        chars.splice(range.clone(), text.chars());
        self.model.value = chars.into_iter().collect::<String>().into();
        self.state.cursor = range.start + text.chars().count();
        self.state.clear_selection();
        self.state.preferred_column = None;
        self.marked_range = None;
        self.emit_change(cx);
        cx.notify();
    }

    pub fn set_placeholder(&mut self, placeholder: impl Into<String>, cx: &mut Context<Self>) {
        self.model.placeholder = placeholder.into().into();
        cx.notify();
    }

    pub fn set_clean_on_escape(&mut self, clean_on_escape: bool, cx: &mut Context<Self>) {
        self.model.clean_on_escape = clean_on_escape;
        cx.notify();
    }

    pub fn set_max_clipboard_paste_bytes(&mut self, max_bytes: Option<usize>, cx: &mut Context<Self>) {
        self.model.max_clipboard_paste_bytes = max_bytes;
        cx.notify();
    }

    pub fn set_validator(&mut self, validator: Option<super::model::Validator>, cx: &mut Context<Self>) {
        self.model.validator = validator;
        self.recompute_invalid();
        cx.notify();
    }

    pub fn set_template(&mut self, template: Arc<dyn super::TextFieldTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        self.layout_cache = None;
        cx.notify();
    }

    pub fn set_font_family(&mut self, family: SharedString, cx: &mut Context<Self>) {
        let family = family.clone();
        self.model.look_override = Some(Arc::new(move |mut look| {
            look.font_family = family.clone();
            look
        }));
        self.layout_cache = None;
        cx.notify();
    }

    pub fn set_look_override(
        &mut self,
        override_fn: Option<super::model::TextFieldLookOverride>,
        cx: &mut Context<Self>,
    ) {
        self.model.look_override = override_fn;
        self.layout_cache = None;
        cx.notify();
    }

    fn resolved_look(&self, window: &Window, cx: &mut Context<Self>) -> TextFieldLook {
        let base_state = TextFieldState { focused: false, focus_visible: false, ..self.state };
        let mut look = self.model.template.resolve_look_with_scale(
            base_state,
            self.model.enabled,
            self.model.size,
            window.scale_factor(),
            cx,
        );
        let focus_state = TextFieldState { focused: true, focus_visible: true, ..self.state };
        let mut focus_look = self.model.template.resolve_look_with_scale(
            focus_state,
            self.model.enabled,
            self.model.size,
            window.scale_factor(),
            cx,
        );
        if self.model.compact {
            let geometry = &look.helper_geometry;
            look.padding_y = geometry.compact_textfield_padding_y;
            look.min_height = geometry.compact_textfield_height;
        }
        if let Some(override_fn) = &self.model.look_override {
            look = override_fn(look);
            focus_look = override_fn(focus_look);
        }
        if self.model.enabled && !self.state.invalid {
            look.focus_border = Some(focus_look.border);
        }
        look
    }

    fn render_model_with_offsets<'a>(
        &'a self,
        character_offsets: Vec<f32>,
        look: TextFieldLook,
    ) -> TextFieldRenderModel<'a> {
        TextFieldRenderModel {
            id: &self.model.id,
            placeholder: &self.model.placeholder,
            value: &self.model.value,
            prefix_icon: self.model.prefix_icon.as_ref(),
            size: self.model.size,
            enabled: self.model.enabled,
            full_width: self.model.full_width,
            state: self.state,
            caret_visible: self.state.focused && self.model.enabled && self.caret_visible,
            horizontal_scroll: self.horizontal_scroll.as_f32(),
            character_offsets,
            marked_range: self.marked_range.clone(),
            look,
        }
    }

    fn layout_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> TextFieldLayoutPreview {
        let look = self.resolved_look(window, cx);
        let run = TextRun {
            len: self.model.value.len(),
            font: {
                let mut font = font(look.font_family.clone());
                font.weight = look.typography.weight;
                font
            },
            color: look.foreground,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let line = window.text_system().shape_line(self.model.value.clone(), px(look.typography.size), &[run], None);
        let chars = self.model.value.chars().count();
        let mut character_offsets = Vec::with_capacity(chars + 1);
        for char_offset in 0..=chars {
            let byte_offset = Self::char_to_byte_offset(self.model.value.as_ref(), char_offset);
            character_offsets.push(line.x_for_index(byte_offset).as_f32());
        }

        TextFieldLayoutPreview { character_offsets, look }
    }

    fn template_handlers(&self, cx: &mut Context<Self>) -> TextFieldTemplateHandlers {
        TextFieldTemplateHandlers {
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

    fn sync_focus(&mut self, window: &Window, cx: &mut Context<Self>) {
        let focused = self.focus_handle.is_focused(window);
        if self.state.focused == focused {
            return;
        }

        self.state.set_focused(focused);

        if focused {
            let keyboard_focus = window.last_input_was_keyboard();
            if self.model.select_all_on_tab_focus && keyboard_focus && !self.suppress_select_all_on_next_focus {
                select_all(&mut self.state, self.model.value.chars().count());
            }
            self.state.preferred_column = None;
            self.state.set_focus_visible(true);
            self.focus_count += 1;
            cx.emit(TextFieldEvent::FocusChanged { focused: true });
            self.start_caret_blink(cx);
        } else {
            self.mouse_selecting = false;
            if self.model.select_all_on_tab_focus {
                self.state.clear_selection();
                self.state.cursor = self.model.value.chars().count();
            }
            self.blur_count += 1;
            cx.emit(TextFieldEvent::FocusChanged { focused: false });
            self.stop_caret_blink(cx);
        }

        self.suppress_select_all_on_next_focus = false;
        cx.notify();
    }

    fn next_caret_epoch(&mut self) -> usize {
        self.caret_epoch += 1;
        self.caret_epoch
    }

    fn start_caret_blink(&mut self, cx: &mut Context<Self>) {
        self.caret_paused = false;
        self.caret_visible = true;
        cx.notify();

        let epoch = self.next_caret_epoch();
        self.caret_task = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_millis(500)).await;
            if let Some(this) = this.upgrade() {
                this.update(cx, |this, cx| this.tick_caret_blink(epoch, cx));
            }
        });
    }

    fn stop_caret_blink(&mut self, cx: &mut Context<Self>) {
        self.caret_epoch = 0;
        self.caret_paused = false;
        self.caret_visible = false;
        cx.notify();
    }

    fn pause_caret_blink(&mut self, cx: &mut Context<Self>) {
        if !self.state.focused {
            return;
        }

        self.caret_paused = true;
        self.caret_visible = true;
        cx.notify();

        let epoch = self.next_caret_epoch();
        self.caret_task = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_millis(300)).await;
            if let Some(this) = this.upgrade() {
                this.update(cx, |this, cx| {
                    this.caret_paused = false;
                    this.tick_caret_blink(epoch, cx);
                });
            }
        });
    }

    fn tick_caret_blink(&mut self, epoch: usize, cx: &mut Context<Self>) {
        if !self.state.focused {
            self.caret_visible = false;
            return;
        }

        if self.caret_paused || epoch != self.caret_epoch {
            self.caret_visible = true;
            return;
        }

        self.caret_visible = !self.caret_visible;
        cx.notify();

        let epoch = self.next_caret_epoch();
        self.caret_task = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_millis(500)).await;
            if let Some(this) = this.upgrade() {
                this.update(cx, |this, cx| this.tick_caret_blink(epoch, cx));
            }
        });
    }

    fn recompute_invalid(&mut self) {
        let invalid = self.model.validator.as_ref().is_some_and(|validate| !validate(self.model.value.as_ref()));
        self.state.set_invalid(invalid);
    }

    fn emit_change(&mut self, cx: &mut Context<Self>) {
        self.change_count += 1;
        cx.emit(TextFieldEvent::Change { value: self.model.value.to_string() });
    }

    fn handle_hover_changed(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        self.state.set_hovered(*hovered);
        cx.notify();
    }

    fn char_offset_for_point(&self, point: Point<Pixels>) -> usize {
        let Some(cache) = self.layout_cache.as_ref() else {
            return self.model.value.chars().count();
        };

        let local_x = point.x - cache.text_origin.x;
        let byte_index = cache.line.closest_index_for_x(local_x.max(px(0.0)));
        Self::byte_to_char_offset(self.model.value.as_ref(), byte_index)
    }

    fn handle_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            cx.stop_propagation();
            return;
        }

        self.suppress_select_all_on_next_focus = true;
        self.focus_handle.focus(window, cx);
        let chars = self.model.value.chars().collect::<Vec<_>>();
        let len = chars.len();
        let index = self.char_offset_for_point(event.position).min(len);

        if event.click_count >= 3 {
            select_all(&mut self.state, len);
            self.mouse_selecting = false;
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
                self.pause_caret_blink(cx);
                cx.notify();
            }
            return;
        }

        if event.modifiers.shift {
            self.state.set_cursor(index, true);
        } else {
            self.state.set_cursor(index, false);
        }
        self.state.preferred_column = None;

        self.mouse_selecting = true;
        self.pause_caret_blink(cx);
        cx.notify();
    }

    fn handle_mouse_move(&mut self, event: &MouseMoveEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.mouse_selecting {
            return;
        }

        let index = self.char_offset_for_point(event.position).min(self.model.value.chars().count());
        self.state.set_cursor(index, true);
        self.state.preferred_column = None;
        self.pause_caret_blink(cx);
        cx.notify();
    }

    fn handle_drag_move(&mut self, event: &DragMoveEvent<TextFieldDrag>, _window: &mut Window, cx: &mut Context<Self>) {
        if event.drag(cx).id != self.model.id || !self.model.enabled {
            return;
        }

        self.mouse_selecting = true;
        let index = self.char_offset_for_point(event.event.position).min(self.model.value.chars().count());
        self.state.set_cursor(index, true);
        self.state.preferred_column = None;
        self.pause_caret_blink(cx);
        cx.notify();
    }

    fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.mouse_selecting = false;
        if self.state.selection_range().is_none() {
            self.state.clear_selection();
        }
        cx.notify();
    }

    fn handle_click(&mut self, _event: &gpui::ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            cx.stop_propagation();
        }
    }

    fn handle_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
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
                multiline: false,
                submit_on_enter: true,
                strip_newlines_on_paste: true,
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

        if result.submitted {
            self.submit_count += 1;
            cx.emit(TextFieldEvent::Submit { value: self.model.value.to_string() });
        }

        if result.handled {
            window.prevent_default();
            if !(self.model.propagate_home_end_to_parent
                && matches!(event.keystroke.key.as_str(), "home" | "end" | "homekey" | "endkey"))
            {
                cx.stop_propagation();
            }
        }

        if result.changed {
            self.model.value = result.value.into();
            self.marked_range = None;
            self.recompute_invalid();
            self.emit_change(cx);
        }

        if result.changed || result.handled {
            self.pause_caret_blink(cx);
            cx.notify();
        }
    }
}

impl Focusable for TextFieldControl {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TextFieldControl {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.model.enabled && self.focus_handle.is_focused(window) {
            window.blur(cx);
        }

        self.sync_focus(window, cx);
        let layout_preview = self.layout_preview(window, cx);
        let entity = cx.entity();
        let input_focus_handle = self.focus_handle.clone();
        let value = self.model.value.clone();
        let look = layout_preview.look.clone();
        let canvas_look = look.clone();
        let has_prefix_icon = self.model.prefix_icon.is_some();
        let horizontal_scroll = self.horizontal_scroll;

        let input_root = self.model.template.render(
            &self.render_model_with_offsets(layout_preview.character_offsets.clone(), look),
            self.template_handlers(cx),
            window,
            cx,
        );
        let input_root = self
            .attachments
            .render(
                input_root,
                InteractionState {
                    hovered: self.state.hovered,
                    focused: self.state.focused,
                    disabled: !self.model.enabled,
                    ..Default::default()
                },
                cx,
            )
            .track_focus(&self.focus_handle)
            .tab_stop(self.model.enabled && self.model.tab_stop);
        let root = div()
            .relative()
            .id((gpui::ElementId::from(("theme", self.theme_epoch)), self.model.id.clone()))
            .child(input_root)
            .child(
                canvas(
                    move |bounds, window, _| {
                        let run = TextRun {
                            len: value.len(),
                            font: {
                                let mut font = font(canvas_look.font_family.clone());
                                font.weight = canvas_look.typography.weight;
                                font
                            },
                            color: canvas_look.foreground,
                            background_color: None,
                            underline: None,
                            strikethrough: None,
                        };
                        let line = window.text_system().shape_line(
                            value.clone(),
                            px(canvas_look.typography.size),
                            &[run],
                            None,
                        );
                        let x_offset = canvas_look.padding_x
                            + if has_prefix_icon {
                                canvas_look.icon_size + canvas_look.gap
                            } else {
                                0.0
                            };
                        let text_viewport = Bounds::from_corners(
                            point(bounds.left() + px(x_offset), bounds.top()),
                            point(bounds.right() - px(canvas_look.padding_x), bounds.bottom()),
                        );
                        TextFieldLayoutCache {
                            bounds,
                            text_viewport,
                            text_origin: point(text_viewport.left() - horizontal_scroll, bounds.top()),
                            content_width: line.x_for_index(value.len()) + px(TEXTFIELD_TRAILING_HITBOX_WIDTH),
                            line,
                        }
                    },
                    move |bounds, cache, window, cx| {
                        window.handle_input(&input_focus_handle, ElementInputHandler::new(bounds, entity.clone()), cx);
                        entity.update(cx, |this, cx| {
                            if this.sync_horizontal_scroll(&cache) {
                                cx.notify();
                            }
                            this.layout_cache = Some(cache);
                        });
                    },
                )
                .absolute()
                .size_full(),
            );
        root.into_any_element()
    }
}

impl TextFieldControl {
    fn char_to_byte_offset(text: &str, char_offset: usize) -> usize {
        text.chars().take(char_offset).map(char::len_utf8).sum()
    }

    fn byte_to_char_offset(text: &str, byte_offset: usize) -> usize {
        let mut char_offset = 0usize;
        let mut consumed = 0usize;
        for ch in text.chars() {
            if consumed >= byte_offset {
                break;
            }
            consumed += ch.len_utf8();
            char_offset += 1;
        }
        char_offset
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
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

    fn offset_to_utf16(&self, offset: usize) -> usize {
        self.model.value.chars().take(offset).map(char::len_utf16).sum()
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn selected_range(&self) -> Range<usize> {
        self.state
            .selection_range()
            .map(|(start, end)| start..end)
            .unwrap_or(self.state.cursor..self.state.cursor)
    }

    fn replace_char_range(
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
        cx.notify();
    }

    fn range_bounds_from_cache(cache: &TextFieldLayoutCache, text: &str, range: Range<usize>) -> Bounds<Pixels> {
        let start = Self::char_to_byte_offset(text, range.start);
        let end = Self::char_to_byte_offset(text, range.end);
        let start_x = cache.text_origin.x + cache.line.x_for_index(start);
        let end_x = cache.text_origin.x + cache.line.x_for_index(end);
        Bounds::from_corners(point(start_x, cache.bounds.top()), point(end_x.max(start_x), cache.bounds.bottom()))
    }

    fn sync_horizontal_scroll(&mut self, cache: &TextFieldLayoutCache) -> bool {
        let viewport_width = cache.text_viewport.size.width;
        let max_scroll = (cache.content_width - viewport_width).max(px(0.0));
        let clamped_current = self.horizontal_scroll.min(max_scroll).max(px(0.0));
        let caret_bounds =
            Self::range_bounds_from_cache(cache, self.model.value.as_ref(), self.state.cursor..self.state.cursor);
        let reveal_padding = px(TEXTFIELD_SCROLL_REVEAL_PADDING);
        let left_reveal_edge = cache.text_viewport.left() + reveal_padding.min(viewport_width);
        let right_reveal_edge = cache.text_viewport.right() - reveal_padding.min(viewport_width);
        let mut target = clamped_current;

        if caret_bounds.left() < left_reveal_edge {
            target -= left_reveal_edge - caret_bounds.left();
        } else if caret_bounds.right() > right_reveal_edge {
            target += caret_bounds.right() - right_reveal_edge;
        }

        target = target.max(px(0.0)).min(max_scroll);
        if (target - self.horizontal_scroll).abs() <= px(0.5) {
            self.horizontal_scroll = clamped_current;
            return false;
        }

        self.horizontal_scroll = target;
        true
    }
}

impl EntityInputHandler for TextFieldControl {
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
        let replacement = text.replace(['\n', '\r'], "");
        self.replace_char_range(range, &replacement, None, false, cx);
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
        let replacement = new_text.replace(['\n', '\r'], "");
        self.replace_char_range(range, &replacement, new_selected_range, true, cx);
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
            .map(|cache| Self::range_bounds_from_cache(cache, self.model.value.as_ref(), range))
            .or(Some(element_bounds))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let cache = self.layout_cache.as_ref()?;
        let local_x = point.x - cache.text_origin.x;
        let byte_index = cache.line.closest_index_for_x(local_x.max(px(0.0)));
        let char_index = Self::byte_to_char_offset(self.model.value.as_ref(), byte_index);
        Some(self.offset_to_utf16(char_index))
    }
}

impl AttachmentTarget for TextFieldControl {
    fn attachments(&self) -> &AttachmentHost {
        &self.attachments
    }
    fn attachments_mut(&mut self) -> &mut AttachmentHost {
        &mut self.attachments
    }
}
