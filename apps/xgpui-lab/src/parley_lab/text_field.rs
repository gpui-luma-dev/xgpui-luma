//! A text field and text area edited through GPUI's paragraph API.
//!
//! Chrome and colors come from Luma's text field look (`TextFieldTheme`), as the
//! SDK `TextField` resolves them; text layout, hit testing, carets, selection and
//! visual movement come from the application's paragraph backend (Parley). The
//! control therefore needs an application created with the Parley text system.
//!
//! Positions are byte offsets with an affinity, so carets sit on cluster
//! boundaries and keep their side at direction changes. Deletion removes whole
//! grapheme clusters. Left/Right move visually.
//!
//! One line (the default) scrolls sideways and Enter submits. With
//! [`ParleyTextField::with_rows`] it is a text area: Parley wraps lines at the
//! control width, Enter inserts a line break (stored as U+2028, Parley's line
//! separator, and reported as `\n`), Up/Down keep the caret's horizontal
//! position, Home/End go to the visual line's ends and the content scrolls
//! vertically.
use std::{ops::Range, sync::Arc, time::Duration};

use anyhow::{Result, anyhow};
use gpui::{
    AnyElement, App, Bounds, ClipboardItem, ContentMask, Context, ElementInputHandler, EntityInputHandler,
    EventEmitter, DispatchPhase, FocusHandle, Focusable, Font, FontRun, Hsla, KeyDownEvent, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParagraphPosition, Pixels, PlatformParagraphLayout, Point, Render,
    SharedString, ScrollEvent, Task, TextAffinity, UTF16Selection, VisualDirection, Window, canvas, div, fill, point,
    prelude::*, px, size,
};
use unicode_segmentation::UnicodeSegmentation;
use xgpui_luma::{
    controls::textfield::{
        TextFieldLook, TextFieldLookOverride, TextFieldRenderModel, TextFieldState, TextFieldTemplate,
        TextFieldTemplateHandlers, TextFieldTheme,
    },
    theme::{ControlSize, StandardBoxScale},
};

/// Wrap width used only to measure text wider than the field.
const MEASURE_WIDTH: f32 = 1_000_000.0;
/// Keep this much room between the caret and the viewport edge when scrolling.
const REVEAL_PADDING: f32 = 24.0;
const CARET_BLINK: Duration = Duration::from_millis(500);
const CARET_PAUSE: Duration = Duration::from_millis(300);

/// Semantic events, named as the SDK's `TextFieldEvent` and `TextAreaEvent`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParleyTextFieldEvent {
    /// The text changed; line breaks are `\n`.
    Change {
        value: String,
    },
    /// Enter in a one-line field.
    Submit {
        value: String,
    },
    FocusChanged {
        focused: bool,
    },
    EnabledChanged {
        enabled: bool,
    },
}

/// Geometry of the last painted frame, for hit testing and IME bounds.
#[derive(Clone)]
struct Geometry {
    /// Visible text area inside the padding.
    viewport: Bounds<Pixels>,
    /// Window position of the layout's local origin (scroll applied).
    origin: Point<Pixels>,
    layout: Arc<dyn PlatformParagraphLayout>,
}

pub struct ParleyTextField {
    id: SharedString,
    theme: Arc<dyn TextFieldTheme>,
    /// Draws a one-line field's chrome; a text area still draws its own.
    template: Arc<dyn TextFieldTemplate>,
    size: ControlSize,
    font: Font,
    /// Visible lines of a text area; `None` for a one-line field.
    rows: Option<usize>,
    enabled: bool,
    /// Source text; a text area stores line breaks as [`LINE_SEPARATOR`].
    value: String,
    placeholder: SharedString,
    /// Horizontal caret position kept across Up/Down, in layout coordinates.
    preferred_x: Option<Pixels>,
    anchor: ParagraphPosition,
    caret: ParagraphPosition,
    marked: Option<Range<usize>>,
    focus: FocusHandle,
    focused: bool,
    hovered: bool,
    dragging: bool,
    look_override: Option<TextFieldLookOverride>,
    scroll: Pixels,
    reveal_caret: bool,
    caret_visible: bool,
    caret_epoch: usize,
    caret_task: Task<()>,
    geometry: Option<Geometry>,
    layout_error: Option<String>,
}

impl EventEmitter<ParleyTextFieldEvent> for ParleyTextField {}

fn at(byte_index: usize) -> ParagraphPosition {
    ParagraphPosition { byte_index, affinity: TextAffinity::Downstream }
}

/// Byte offset of a UTF-16 offset, clamped to the text.
fn byte_from_utf16(text: &str, utf16: usize) -> usize {
    let mut count = 0;
    for (index, ch) in text.char_indices() {
        if count + ch.len_utf16() > utf16 {
            return index;
        }
        count += ch.len_utf16();
    }
    text.len()
}

/// Parley's line break inside one paragraph.
const LINE_SEPARATOR: char = '\u{2028}';

/// A single-line field stores no line breaks.
fn single_line(text: &str) -> String {
    text.replace(['\r', '\n', LINE_SEPARATOR, '\u{2029}'], "")
}

/// A text area stores every line break (CRLF, CR, LF, U+2029) as one separator.
fn multi_line(text: &str) -> String {
    text.replace("\r\n", "\n").replace(['\r', '\n', '\u{2029}'], &LINE_SEPARATOR.to_string())
}

fn previous_grapheme(text: &str, index: usize) -> usize {
    text.grapheme_indices(true).rev().find(|(start, _)| *start < index).map_or(0, |(start, _)| start)
}

fn next_grapheme(text: &str, index: usize) -> usize {
    text.grapheme_indices(true).find(|(start, _)| *start > index).map_or(text.len(), |(start, _)| start)
}

fn is_word(segment: &str) -> bool {
    segment.chars().any(char::is_alphanumeric)
}

/// Start of the word before `index` (skipping separators), as on macOS Option-Left.
fn previous_word(text: &str, index: usize) -> usize {
    text.split_word_bound_indices()
        .rev()
        .find(|(start, segment)| *start < index && is_word(segment))
        .map_or(0, |(start, _)| start)
}

/// End of the word after `index` (skipping separators), as on macOS Option-Right.
fn next_word(text: &str, index: usize) -> usize {
    text.split_word_bound_indices()
        .find(|(start, segment)| start + segment.len() > index && is_word(segment))
        .map_or(text.len(), |(start, segment)| start + segment.len())
}

/// The word (or separator run) containing `index`, for double-click.
fn word_at(text: &str, index: usize) -> Range<usize> {
    text.split_word_bound_indices()
        .find(|(start, segment)| index < start + segment.len())
        .or_else(|| text.split_word_bound_indices().next_back())
        .map_or(0..0, |(start, segment)| start..start + segment.len())
}

// Public API mirrors the SDK text field; the lab page uses only part of it.
#[allow(dead_code)]
impl ParleyTextField {
    /// Start building a field, as `xgpui_luma::controls::textfield::new(id)`.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> ParleyTextFieldBuilder {
        ParleyTextFieldBuilder::new(id)
    }

    /// The control a builder describes; `spawn` also binds its callbacks.
    pub(crate) fn from_builder(mut builder: ParleyTextFieldBuilder, cx: &mut Context<Self>) -> Self {
        let theme = builder.resolve_theme(cx);
        let template = builder.take_template(cx);
        let mut this = Self {
            id: builder.id,
            theme,
            template,
            size: builder.size.control_size(),
            font: builder.font,
            rows: builder.rows,
            enabled: builder.enabled,
            value: String::new(),
            placeholder: builder.placeholder,
            preferred_x: None,
            anchor: at(0),
            caret: at(0),
            marked: None,
            focus: cx.focus_handle().tab_stop(builder.enabled),
            focused: false,
            hovered: false,
            dragging: false,
            look_override: builder.look_override,
            scroll: px(0.0),
            reveal_caret: false,
            caret_visible: false,
            caret_epoch: 0,
            caret_task: Task::ready(()),
            geometry: None,
            layout_error: None,
        };
        this.value = this.normalize(&builder.value);
        this.anchor = at(this.value.len());
        this.caret = this.anchor;
        this
    }

    /// Replace the text, putting the caret at its end; emits no `Change`.
    pub fn set_value(&mut self, value: impl Into<String>, cx: &mut Context<Self>) {
        self.value = self.normalize(&value.into());
        self.marked = None;
        self.geometry = None;
        self.anchor = at(self.value.len());
        self.set_caret(self.anchor, false, cx);
    }

    pub fn set_placeholder(&mut self, placeholder: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.placeholder = placeholder.into();
        cx.notify();
    }

    pub fn set_size(&mut self, size: ControlSize, cx: &mut Context<Self>) {
        self.size = size;
        self.geometry = None;
        cx.notify();
    }

    /// A disabled control takes no focus, pointer or keyboard input and draws dimmed.
    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.enabled == enabled {
            return;
        }
        self.enabled = enabled;
        self.focus = self.focus.clone().tab_stop(enabled);
        if !enabled {
            self.hovered = false;
            self.dragging = false;
        }
        cx.emit(ParleyTextFieldEvent::EnabledChanged { enabled });
        cx.notify();
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn select_all(&mut self, cx: &mut Context<Self>) {
        self.anchor = at(0);
        self.set_caret(at(self.value.len()), true, cx);
    }

    /// Copy the selection to the clipboard, with line breaks as `\n`.
    pub fn copy_selection(&self, cx: &mut Context<Self>) {
        let range = self.selected_range();
        if !range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.value[range].replace(LINE_SEPARATOR, "\n")));
        }
    }

    pub fn cut_selection(&mut self, cx: &mut Context<Self>) {
        let range = self.selected_range();
        if !range.is_empty() {
            self.copy_selection(cx);
            self.replace(range, "", None, false, cx);
        }
    }

    /// Replace the selection with the clipboard text; a field drops its line breaks.
    pub fn paste(&mut self, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.replace(self.selected_range(), &text, None, false, cx);
        }
    }

    /// The text, with a text area's line breaks as `\n`.
    pub fn value(&self) -> String {
        self.value.replace(LINE_SEPARATOR, "\n")
    }

    pub fn is_multiline(&self) -> bool {
        self.rows.is_some()
    }

    fn normalize(&self, text: &str) -> String {
        if self.is_multiline() {
            multi_line(text)
        } else {
            single_line(text)
        }
    }

    /// Logical selection as (anchor, caret) positions.
    pub fn selection(&self) -> (ParagraphPosition, ParagraphPosition) {
        (self.anchor, self.caret)
    }

    pub fn scroll(&self) -> Pixels {
        self.scroll
    }

    pub fn layout_error(&self) -> Option<&str> {
        self.layout_error.as_deref()
    }

    fn selected_range(&self) -> Range<usize> {
        let (a, b) = (self.anchor.byte_index, self.caret.byte_index);
        a.min(b)..a.max(b)
    }

    fn utf16_range(&self, range: Range<usize>) -> Range<usize> {
        self.value[..range.start].encode_utf16().count()..self.value[..range.end].encode_utf16().count()
    }

    fn byte_range(&self, range: Range<usize>) -> Range<usize> {
        byte_from_utf16(&self.value, range.start)..byte_from_utf16(&self.value, range.end.max(range.start))
    }

    fn look(&self, window: &Window) -> TextFieldLook {
        let scale = StandardBoxScale::compute(self.size, &self.theme.metrics(), window.scale_factor());
        let state = TextFieldState { hovered: self.hovered, ..TextFieldState::default() };
        let mut look = self.theme.resolve_look(state, self.enabled, self.size, &scale);
        let focus_state = TextFieldState { focused: true, focus_visible: true, ..state };
        look.focus_border = Some(self.theme.resolve_look(focus_state, true, self.size, &scale).border);
        match &self.look_override {
            Some(look_override) => look_override(look),
            None => look,
        }
    }

    fn font(&self, look: &TextFieldLook) -> Font {
        Font { weight: look.typography.weight, ..self.font.clone() }
    }

    /// One-line layout no narrower than `viewport`, so start alignment (left for
    /// left-to-right, right for right-to-left) applies inside the field. Text
    /// wider than the field is laid out at its own width and scrolls.
    fn layout(
        &self,
        viewport: Pixels,
        look: &TextFieldLook,
        window: &Window,
    ) -> Result<Arc<dyn PlatformParagraphLayout>> {
        let system = window.text_system();
        let font_size = px(look.typography.size);
        let runs = if self.value.is_empty() {
            Vec::new()
        } else {
            vec![FontRun { len: self.value.len(), font_id: system.resolve_font(&self.font(look)) }]
        };
        let text: SharedString = self.value.clone().into();
        let width = viewport.max(px(1.0));
        let layout = system.layout_paragraph(text.clone(), font_size, &runs, width)?;
        // A text area wraps at the control width; a field keeps one line.
        if self.is_multiline() || !wrapped(layout.as_ref(), &self.value)? {
            return Ok(layout);
        }
        let wide = system.layout_paragraph(text.clone(), font_size, &runs, px(MEASURE_WIDTH))?;
        let content = content_width(wide.as_ref(), &self.value)?;
        system.layout_paragraph(text, font_size, &runs, (content + px(1.0)).ceil().max(width))
    }

    /// Lay out for this frame, keep the caret visible and record geometry.
    fn prepare(&mut self, viewport: Bounds<Pixels>, look: &TextFieldLook, window: &Window) -> Option<Geometry> {
        let layout = match self.layout(viewport.size.width, look, window) {
            Ok(layout) => {
                self.layout_error = None;
                layout
            }
            Err(error) => {
                self.layout_error = Some(format!("{error:#}"));
                self.geometry = None;
                return None;
            }
        };
        // A field scrolls sideways, a text area vertically.
        let multiline = self.is_multiline();
        let (content, visible) = if multiline {
            (layout.size().height, viewport.size.height)
        } else {
            (layout.size().width, viewport.size.width)
        };
        let max_scroll = (content - visible).max(px(0.0));
        self.scroll = self.scroll.clamp(px(0.0), max_scroll);
        if self.reveal_caret
            && let Ok(caret) = layout.caret_bounds(self.caret)
        {
            let padding = if multiline {
                px(0.0)
            } else {
                px(REVEAL_PADDING).min(visible / 2.0)
            };
            let (near, far) = if multiline {
                (caret.top(), caret.bottom())
            } else {
                (caret.left(), caret.right())
            };
            let (near, far) = (near - self.scroll, far - self.scroll);
            if near < padding {
                self.scroll -= padding - near;
            } else if far > visible - padding {
                self.scroll += far - (visible - padding);
            }
            self.scroll = self.scroll.clamp(px(0.0), max_scroll);
        }
        self.reveal_caret = false;
        let origin = if multiline {
            point(viewport.left(), viewport.top() - self.scroll)
        } else {
            point(viewport.left() - self.scroll, viewport.top() + (viewport.size.height - layout.size().height) / 2.0)
        };
        let geometry = Geometry { viewport, origin, layout };
        self.geometry = Some(geometry.clone());
        Some(geometry)
    }

    fn hit(&self, position: Point<Pixels>) -> Option<ParagraphPosition> {
        let geometry = self.geometry.as_ref()?;
        if geometry.layout.source() != self.value {
            return None;
        }
        let height = geometry.layout.size().height;
        let y = if self.is_multiline() {
            // Above the text is its first line, below it its last.
            (position.y - geometry.origin.y).clamp(px(0.0), (height - px(1.0)).max(px(0.0)))
        } else {
            // One line: hit test on its vertical center wherever the pointer is.
            height / 2.0
        };
        geometry.layout.hit_test(point(position.x - geometry.origin.x, y)).ok()
    }

    /// Logical ends of the visual line holding the caret.
    fn line_bounds(&self) -> Option<Range<usize>> {
        let geometry = self.geometry.as_ref()?;
        let layout = &geometry.layout;
        let caret = layout.caret_bounds(self.caret).ok()?;
        let y = caret.center().y;
        let left = layout.hit_test(point(px(-1.0), y)).ok()?.byte_index;
        let right = layout.hit_test(point(layout.size().width + px(1.0), y)).ok()?.byte_index;
        Some(left.min(right)..left.max(right))
    }

    /// Move one visual line up or down, keeping the horizontal position.
    fn move_vertical(&mut self, down: bool, extend: bool, cx: &mut Context<Self>) {
        let Some(layout) = self.geometry.as_ref().map(|geometry| geometry.layout.clone()) else {
            return;
        };
        let Ok(caret) = layout.caret_bounds(self.caret) else {
            return;
        };
        let x = self.preferred_x.unwrap_or(caret.left());
        let y = if down {
            caret.bottom() + px(1.0)
        } else {
            caret.top() - px(1.0)
        };
        let next = if y < px(0.0) {
            at(0)
        } else if y >= layout.size().height {
            at(self.value.len())
        } else {
            match layout.hit_test(point(x, y)) {
                Ok(next) => next,
                Err(_) => return,
            }
        };
        self.set_caret(next, extend, cx);
        self.preferred_x = Some(x);
    }

    fn scroll_wheel(&mut self, event: &ScrollEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.is_multiline() {
            return;
        }
        let line_height = px(self.look(window).typography.line_height);
        self.scroll -= event.delta.pixel_delta(line_height).y;
        cx.stop_propagation();
        cx.notify(); // The next prepare clamps the offset.
    }

    fn set_caret(&mut self, caret: ParagraphPosition, extend: bool, cx: &mut Context<Self>) {
        self.caret = caret;
        if !extend {
            self.anchor = caret;
        }
        self.preferred_x = None; // Vertical moves set it again afterwards.
        self.reveal_caret = true;
        self.pause_caret_blink(cx);
        cx.notify();
    }

    fn replace(
        &mut self,
        range: Range<usize>,
        text: &str,
        selected: Option<Range<usize>>,
        mark: bool,
        cx: &mut Context<Self>,
    ) {
        let inserted = self.normalize(text);
        // `selected` is UTF-16 within `text`; map it through line-break normalization.
        let selected = selected
            .map(|r| {
                self.normalize(&text[..byte_from_utf16(text, r.start)]).len()
                    ..self.normalize(&text[..byte_from_utf16(text, r.end.max(r.start))]).len()
            })
            .unwrap_or(inserted.len()..inserted.len());
        let changed = self.value[range.clone()] != inserted;
        self.value.replace_range(range.clone(), &inserted);
        self.marked = (mark && !inserted.is_empty()).then(|| range.start..range.start + inserted.len());
        self.anchor = at(range.start + selected.start);
        self.caret = at(range.start + selected.end);
        // Never answer hit tests or IME geometry from a previous revision.
        self.geometry = None;
        self.reveal_caret = true;
        if changed {
            cx.emit(ParleyTextFieldEvent::Change { value: self.value() });
        }
        self.pause_caret_blink(cx);
        cx.notify();
    }

    fn delete(&mut self, backward: bool, word: bool, line: bool, cx: &mut Context<Self>) {
        let mut range = self.selected_range();
        if range.is_empty() {
            let index = self.caret.byte_index;
            // To the visual line's end in a text area, the text's end in a field.
            let line_ends = self.line_bounds().filter(|_| self.is_multiline()).unwrap_or(0..self.value.len());
            if backward {
                range.start = match (line, word) {
                    (true, _) => line_ends.start.min(index),
                    (_, true) => previous_word(&self.value, index),
                    _ => previous_grapheme(&self.value, index),
                };
            } else {
                range.end = match (line, word) {
                    (true, _) => line_ends.end.max(index),
                    (_, true) => next_word(&self.value, index),
                    _ => next_grapheme(&self.value, index),
                };
            }
        }
        if !range.is_empty() {
            self.replace(range, "", None, false, cx);
        }
    }

    fn move_horizontal(&mut self, direction: VisualDirection, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let modifiers = event.keystroke.modifiers;
        let extend = modifiers.shift;
        if modifiers.secondary(cx) {
            // Line start/end: logical, as in a single-line native field.
            let end = matches!(direction, VisualDirection::Right) != self.is_rtl();
            self.move_to_line_end(end, extend, cx);
            return;
        }
        if modifiers.alt {
            let forward = matches!(direction, VisualDirection::Right) != self.is_rtl();
            let index = self.caret.byte_index;
            let next = if forward {
                next_word(&self.value, index)
            } else {
                previous_word(&self.value, index)
            };
            self.set_caret(at(next), extend, cx);
            return;
        }
        let Some(layout) = self.geometry.as_ref().map(|geometry| geometry.layout.clone()) else {
            return;
        };
        if self.anchor.byte_index != self.caret.byte_index && !extend {
            // Collapse to the selection edge on the side of the key.
            let (Ok(a), Ok(b)) = (layout.caret_bounds(self.anchor), layout.caret_bounds(self.caret)) else {
                return;
            };
            let anchor_left = a.left() <= b.left();
            let edge = if anchor_left == matches!(direction, VisualDirection::Left) {
                self.anchor
            } else {
                self.caret
            };
            self.set_caret(edge, false, cx);
            return;
        }
        if let Ok(next) = layout.move_visual(self.caret, direction) {
            self.set_caret(next, extend, cx);
        }
    }

    /// To the logical start or end of the visual line (a text area) or the text (a field).
    fn move_to_line_end(&mut self, end: bool, extend: bool, cx: &mut Context<Self>) {
        let ends = self.line_bounds().filter(|_| self.is_multiline()).unwrap_or(0..self.value.len());
        let target = if end {
            // A wrapped line's end sits before the next line: keep the caret on this row.
            ParagraphPosition { byte_index: ends.end, affinity: TextAffinity::Upstream }
        } else {
            at(ends.start)
        };
        self.set_caret(target, extend, cx);
    }

    /// Base direction of the current layout: right-to-left if the logical start
    /// is right of the logical end.
    fn is_rtl(&self) -> bool {
        let Some(geometry) = &self.geometry else {
            return false;
        };
        let (Ok(start), Ok(end)) =
            (geometry.layout.caret_bounds(at(0)), geometry.layout.caret_bounds(at(self.value.len())))
        else {
            return false;
        };
        start.left() > end.left()
    }

    fn key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.enabled {
            cx.stop_propagation();
            return;
        }
        if self.marked.is_some() {
            return; // The platform IME owns keys during composition.
        }
        let modifiers = event.keystroke.modifiers;
        let secondary = modifiers.secondary(cx);
        let key = event.keystroke.key.as_str();
        let handled = match key {
            "a" if secondary => {
                self.select_all(cx);
                true
            }
            "c" if secondary => {
                self.copy_selection(cx);
                true
            }
            "x" if secondary => {
                self.cut_selection(cx);
                true
            }
            "v" if secondary => {
                self.paste(cx);
                true
            }
            "backspace" => {
                self.delete(true, modifiers.alt, secondary, cx);
                true
            }
            "delete" => {
                self.delete(false, modifiers.alt, secondary, cx);
                true
            }
            "left" => {
                self.move_horizontal(VisualDirection::Left, event, cx);
                true
            }
            "right" => {
                self.move_horizontal(VisualDirection::Right, event, cx);
                true
            }
            "up" | "down" if self.is_multiline() && !secondary => {
                self.move_vertical(key == "down", modifiers.shift, cx);
                true
            }
            // Cmd/Ctrl-Up/Down, and Up/Down in a field: the text's ends.
            "up" | "down" => {
                self.set_caret(at(if key == "down" { self.value.len() } else { 0 }), modifiers.shift, cx);
                true
            }
            "home" | "end" => {
                self.move_to_line_end(key == "end", modifiers.shift, cx);
                true
            }
            "enter" if self.is_multiline() => {
                self.replace(self.selected_range(), "\n", None, false, cx);
                true
            }
            "enter" => {
                cx.emit(ParleyTextFieldEvent::Submit { value: self.value() });
                true
            }
            _ => false,
        };
        if handled {
            window.prevent_default();
            cx.stop_propagation();
        }
    }

    fn mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.enabled {
            cx.stop_propagation();
            return;
        }
        window.focus(&self.focus, cx);
        let Some(hit) = self.hit(event.position) else {
            return;
        };
        match event.click_count {
            2 => {
                let word = word_at(&self.value, hit.byte_index);
                self.anchor = at(word.start);
                self.set_caret(at(word.end), true, cx);
            }
            count if count >= 3 => {
                self.anchor = at(0);
                self.set_caret(at(self.value.len()), true, cx);
            }
            _ => {
                self.set_caret(hit, event.modifiers.shift, cx);
                self.dragging = true;
            }
        }
    }

    /// Extend a drag selection to `position`, which may be anywhere in the window:
    /// the field follows the pointer until the button is released.
    fn drag_to(&mut self, position: Point<Pixels>, cx: &mut Context<Self>) {
        if self.dragging
            && let Some(hit) = self.hit(position)
        {
            self.set_caret(hit, true, cx);
        }
    }

    fn end_drag(&mut self, cx: &mut Context<Self>) {
        if self.dragging {
            self.dragging = false;
            cx.notify();
        }
    }

    fn sync_focus(&mut self, window: &Window, cx: &mut Context<Self>) {
        let focused = self.focus.is_focused(window);
        if focused == self.focused {
            return;
        }
        self.focused = focused;
        self.dragging = false;
        if focused {
            self.reveal_caret = true;
            self.start_caret_blink(cx);
        } else {
            self.caret_epoch += 1;
            self.caret_visible = false;
        }
        cx.emit(ParleyTextFieldEvent::FocusChanged { focused });
    }

    fn start_caret_blink(&mut self, cx: &mut Context<Self>) {
        self.caret_visible = true;
        self.schedule_blink(CARET_BLINK, cx);
    }

    fn pause_caret_blink(&mut self, cx: &mut Context<Self>) {
        if self.focused {
            self.caret_visible = true;
            self.schedule_blink(CARET_PAUSE, cx);
        }
    }

    fn schedule_blink(&mut self, delay: Duration, cx: &mut Context<Self>) {
        self.caret_epoch += 1;
        let epoch = self.caret_epoch;
        self.caret_task = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(delay).await;
            if let Some(this) = this.upgrade() {
                this.update(cx, |this, cx| {
                    if this.focused && this.caret_epoch == epoch {
                        this.caret_visible = !this.caret_visible;
                        this.schedule_blink(CARET_BLINK, cx);
                        cx.notify();
                    }
                });
            }
        });
    }
}

/// Whether a no-break line wrapped: its start and end carets sit on different rows.
fn wrapped(layout: &dyn PlatformParagraphLayout, text: &str) -> Result<bool> {
    if text.is_empty() {
        return Ok(false);
    }
    let start = layout.caret_bounds(at(0))?;
    let end = layout.caret_bounds(ParagraphPosition { byte_index: text.len(), affinity: TextAffinity::Upstream })?;
    Ok((start.top() - end.top()).abs() > px(0.5))
}

/// Horizontal extent of a one-line layout across every character boundary.
fn content_width(layout: &dyn PlatformParagraphLayout, text: &str) -> Result<Pixels> {
    let (mut left, mut right) = (px(f32::MAX), px(f32::MIN));
    for index in text.char_indices().map(|(index, _)| index).chain([text.len()]) {
        let caret = layout.caret_bounds(at(index))?;
        left = left.min(caret.left());
        right = right.max(caret.right());
    }
    if left > right {
        Err(anyhow!("empty paragraph extent"))
    } else {
        Ok(right - left)
    }
}

impl Focusable for ParleyTextField {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for ParleyTextField {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.enabled && self.focus.is_focused(window) {
            window.blur(cx);
        }
        self.sync_focus(window, cx);
        let look = self.look(window);
        let enabled = self.enabled;
        let focused = self.focused;
        let focus_gap = self.theme.metrics().focus.gap + look.border_width.max(0.0);
        let show_placeholder = self.value.is_empty() && !focused;
        let entity = cx.entity();
        let focus = self.focus.clone();
        let (anchor, caret, marked) = (self.anchor, self.caret, self.marked.clone());
        let selected = self.selected_range();
        let caret_visible = focused && self.caret_visible;
        let dragging = self.dragging;
        let canvas_look = look.clone();
        let line_height = px(look.typography.line_height);

        let text = canvas(
            {
                let entity = entity.clone();
                let look = canvas_look.clone();
                move |bounds, window, cx| entity.update(cx, |this, _| this.prepare(bounds, &look, window))
            },
            move |bounds, geometry, window, cx| {
                if dragging {
                    // Capture the pointer for the drag, wherever it goes in the window.
                    let mover = entity.clone();
                    window.on_mouse_event(move |event: &MouseMoveEvent, phase, _, cx| {
                        if phase == DispatchPhase::Capture {
                            mover.update(cx, |this, cx| this.drag_to(event.position, cx));
                        }
                    });
                    let releaser = entity.clone();
                    window.on_mouse_event(move |event: &MouseUpEvent, phase, _, cx| {
                        if phase == DispatchPhase::Capture && event.button == MouseButton::Left {
                            releaser.update(cx, |this, cx| this.end_drag(cx));
                        }
                    });
                }
                window.handle_input(&focus, ElementInputHandler::new(bounds, entity), cx);
                let Some(geometry) = geometry else {
                    return;
                };
                let look = canvas_look;
                let layout = geometry.layout.as_ref();
                let origin = geometry.origin;
                window.with_content_mask(Some(ContentMask { bounds: geometry.viewport }), |window| {
                    if let Ok(rects) = layout.selection_bounds(anchor, caret) {
                        for rect in rects {
                            window.paint_quad(fill(
                                Bounds::new(origin + rect.origin, rect.size),
                                look.selection_background,
                            ));
                        }
                    }
                    let color = |index: usize| -> Hsla {
                        if selected.contains(&index) {
                            look.selection_foreground
                        } else {
                            look.foreground
                        }
                    };
                    if let Err(error) = gpui::paint_paragraph(layout, origin, color, window) {
                        eprintln!("Parley text field paint: {error:#}");
                    }
                    if let Some(marked) = &marked
                        && let Ok(rects) = layout.selection_bounds(at(marked.start), at(marked.end))
                    {
                        let thickness = px(look.helper_geometry.composition_underline_width.max(1.0));
                        for rect in rects {
                            window.paint_quad(fill(
                                Bounds::new(
                                    origin + point(rect.left(), rect.bottom() - thickness),
                                    size(rect.size.width, thickness),
                                ),
                                look.foreground,
                            ));
                        }
                    }
                    if caret_visible
                        && selected.is_empty()
                        && let Ok(rect) = layout.caret_bounds(caret)
                    {
                        window.paint_quad(fill(
                            Bounds::new(origin + rect.origin, size(px(look.caret_width), rect.size.height)),
                            look.caret,
                        ));
                    }
                });
            },
        )
        .size_full();

        if self.rows.is_none() {
            return self.render_with_template(text, look, focus_gap, window, cx);
        }

        // A text area shows `rows` lines inside the field's padding and border; it
        // draws its own chrome until it moves to the SDK's text area template.
        let rows = self.rows;
        let viewport_height = line_height * rows.unwrap_or(1) as f32;
        let control_height = viewport_height + px(2.0 * (look.padding_y + look.border_width));
        let viewport = div().relative().min_w(px(0.0)).flex_1().h(viewport_height).child(text).when(
            show_placeholder,
            |viewport| {
                viewport.child(
                    div()
                        .absolute()
                        .inset_0()
                        .flex()
                        .overflow_hidden()
                        .text_color(look.placeholder)
                        .child(self.placeholder.clone()),
                )
            },
        );

        let control = div()
            .id("control")
            .relative()
            .w_full()
            .h(control_height)
            .flex()
            .items_center()
            .px(px(look.padding_x))
            .py(px(look.padding_y))
            .bg(look.background)
            .border(px(look.border_width))
            .border_color(look.border)
            .rounded(px(look.radius))
            .font_family(look.font_family.clone())
            .text_size(px(look.typography.size))
            .line_height(line_height)
            .when(enabled, |control| control.cursor_text())
            .when(!enabled, |control| control.cursor_not_allowed().opacity(look.disabled_opacity))
            .when_some(look.shadow.clone().filter(|shadows| !shadows.is_empty()), |control, shadows| {
                control.shadow(shadows)
            })
            .child(viewport);

        div()
            .id(self.id.clone())
            .relative()
            .w_full()
            .p(px(focus_gap))
            .track_focus(&self.focus)
            .on_hover(cx.listener(|this, hovered: &bool, _, cx| {
                this.hovered = *hovered && this.enabled;
                cx.notify();
            }))
            .on_key_down(cx.listener(Self::key_down))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::mouse_down))
            .on_scroll(cx.listener(Self::scroll_wheel))
            .child(control)
            .when(focused, |root| {
                root.child(
                    div()
                        .absolute()
                        .inset_0()
                        .border(px(look.border_width))
                        .border_color(look.focus_border.unwrap_or(look.border))
                        .rounded(px(look.radius + focus_gap)),
                )
            })
            .into_any_element()
    }
}

impl ParleyTextField {
    /// A one-line field: the SDK template draws the chrome (background, border,
    /// padding, focus ring, placeholder, modifiers) around an empty value, and the
    /// Parley text is laid over its text area.
    ///
    /// The template paints text itself with `shape_line` and has no slot for text
    /// it does not draw, so the overlay repeats its geometry: focus ring, border
    /// and horizontal padding. That missing slot is what the SDK template needs
    /// before this control can move into the SDK.
    fn render_with_template(
        &self,
        text: impl IntoElement,
        look: TextFieldLook,
        focus_gap: f32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let empty = SharedString::default();
        // The template shows its placeholder for an empty value; hide it under real text.
        let placeholder = if self.value.is_empty() {
            self.placeholder.clone()
        } else {
            empty.clone()
        };
        let state = TextFieldState {
            hovered: self.hovered,
            focused: self.focused,
            focus_visible: self.focused,
            ..TextFieldState::default()
        };
        let model = TextFieldRenderModel {
            id: &self.id,
            placeholder: &placeholder,
            value: &empty,
            prefix_icon: None,
            size: self.size,
            enabled: self.enabled,
            full_width: true,
            state,
            marked_range: None,
            caret_visible: false,
            horizontal_scroll: 0.0,
            character_offsets: vec![0.0],
            look: look.clone(),
        };
        // Pointer input is handled on the wrapper below: in headless tests the
        // template root's mouse-down listener never fired under the text overlay,
        // while the wrapper's did. Drags are captured window-wide; see `drag_to`.
        let handlers = TextFieldTemplateHandlers {
            hover: Box::new(|_, _, _| {}),
            mouse_down: Box::new(|_, _, _| {}),
            mouse_move: Box::new(|_, _, _| {}),
            mouse_up: Box::new(|_, _, _| {}),
            mouse_up_out: Box::new(|_, _, _| {}),
            click: Box::new(|_, _, _| {}),
            key_down: Box::new(cx.listener(Self::key_down)),
            drag_move: Box::new(|_, _, _| {}),
        };
        // As the SDK text field does, the template's root is the focused element.
        let chrome = self.template.render(&model, handlers, window, cx).track_focus(&self.focus);
        let inset = px(focus_gap + look.border_width.max(0.0) + look.padding_x);
        let line_height = px(look.typography.line_height);
        div()
            .id(SharedString::from(format!("{}-parley", self.id)))
            .relative()
            .w_full()
            .on_hover(cx.listener(|this, hovered: &bool, _, cx| {
                this.hovered = *hovered && this.enabled;
                cx.notify();
            }))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::mouse_down))
            .child(chrome)
            .child(
                // The chrome's own height, not the wrapper's: a stretched wrapper
                // must not move the text off the template's text row.
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .h(px(look.min_height + 2.0 * focus_gap))
                    .px(inset)
                    .flex()
                    .items_center()
                    .when(!self.enabled, |overlay| overlay.opacity(look.disabled_opacity))
                    .child(div().w_full().h(line_height).child(text)),
            )
            .into_any_element()
    }
}

impl EntityInputHandler for ParleyTextField {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        actual: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.byte_range(range);
        *actual = Some(self.utf16_range(range.clone()));
        Some(self.value[range].to_owned())
    }

    fn text_length_utf16(&mut self, _: &mut Window, _: &mut Context<Self>) -> Option<usize> {
        Some(self.value.encode_utf16().count())
    }

    fn selected_text_range(&mut self, _: bool, _: &mut Window, _: &mut Context<Self>) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.utf16_range(self.selected_range()),
            reversed: self.caret.byte_index < self.anchor.byte_index,
        })
    }

    fn set_selected_text_range(&mut self, range: Range<usize>, _: &mut Window, cx: &mut Context<Self>) {
        let range = self.byte_range(range);
        self.anchor = at(range.start);
        self.set_caret(at(range.end), true, cx);
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked.clone().map(|range| self.utf16_range(range))
    }

    fn unmark_text(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.marked = None;
        cx.notify();
    }

    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range.map(|r| self.byte_range(r)).or(self.marked.clone()).unwrap_or_else(|| self.selected_range());
        self.replace(range, text, None, false, cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range.map(|r| self.byte_range(r)).or(self.marked.clone()).unwrap_or_else(|| self.selected_range());
        self.replace(range, text, selected, true, cx);
    }

    fn bounds_for_range(
        &mut self,
        range: Range<usize>,
        _: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let geometry = self.geometry.as_ref()?;
        if geometry.layout.source() != self.value {
            return None;
        }
        let range = self.byte_range(range);
        let start = if range.start == self.caret.byte_index {
            self.caret
        } else {
            at(range.start)
        };
        let caret = geometry.layout.caret_bounds(start).ok()?;
        let rect = if range.is_empty() {
            caret
        } else {
            geometry.layout.selection_bounds(start, at(range.end)).ok()?.first().copied().unwrap_or(caret)
        };
        Some(Bounds::new(geometry.origin + rect.origin, rect.size))
    }

    fn character_index_for_point(
        &mut self,
        position: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        let hit = self.hit(position)?;
        Some(self.value[..hit.byte_index].encode_utf16().count())
    }
}

#[path = "text_field/builder.rs"]
mod builder;
pub use builder::ParleyTextFieldBuilder;

#[path = "text_field/page.rs"]
pub(crate) mod page;

#[cfg(all(test, feature = "test-support"))]
#[path = "text_field/tests.rs"]
mod tests;
