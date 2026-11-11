mod chrome;
mod element;
mod input;
mod layout;

use std::{ops::Range, time::Duration};

use gpui::{
    App, Context, Empty, EventEmitter, Entity, FocusHandle, Focusable, IntoElement, Pixels, Point, Render,
    SharedString, Subscription, Task, Window, px,
};

use super::{TextAreaBuilder, TextAreaState, model::TextAreaModel};
use crate::controls::scrollbar::{Scrollbar, ScrollbarEvent, ScrollbarOrientation};
use crate::controls::text::select_all;
use crate::theme::{StandardBoxScale, observe_theme_revision};

use self::layout::TextAreaLayoutCache;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum TextAreaEvent {
    Change { value: String },
    FocusChanged { focused: bool },
    EnabledChanged { enabled: bool },
}

#[derive(Clone, Debug)]
pub struct TextAreaDrag {
    id: SharedString,
}

impl TextAreaDrag {
    pub(crate) fn new(id: SharedString) -> Self {
        Self { id }
    }
}

impl Render for TextAreaDrag {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

#[derive(Clone, Debug)]
pub struct TextAreaResizeDrag {
    id: SharedString,
}

impl TextAreaResizeDrag {
    pub(crate) fn new(id: SharedString) -> Self {
        Self { id }
    }
}

impl Render for TextAreaResizeDrag {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

pub struct TextArea {
    model: TextAreaModel,
    state: TextAreaState,
    focus_handle: FocusHandle,
    theme_epoch: u64,
    change_count: usize,
    focus_count: usize,
    blur_count: usize,
    caret_visible: bool,
    caret_paused: bool,
    caret_epoch: usize,
    caret_task: Task<()>,
    mouse_selecting: bool,
    selection_drag_position: Option<Point<Pixels>>,
    selection_scroll_epoch: usize,
    selection_scroll_task: Task<()>,
    suppress_select_all_on_next_focus: bool,
    marked_range: Option<Range<usize>>,
    vertical_scroll: Pixels,
    layout_cache: Option<TextAreaLayoutCache>,
    scrollbar: Entity<Scrollbar>,
    scrollbar_enabled: bool,
    resize_drag_id: SharedString,
    resize_dragging: bool,
    resize_start_y: f32,
    resize_start_rows: usize,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<TextAreaEvent> for TextArea {}

impl TextArea {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> TextAreaBuilder {
        TextAreaBuilder::new(id)
    }

    /// Current wheel settings. Keyboard ownership is independent.
    pub fn scroll_interaction(&self) -> crate::interaction::ScrollInteraction {
        self.model.scroll_interaction
    }

    /// Change wheel without resetting focus, selection or position.
    pub fn set_wheel_scroll_policy(&mut self, policy: crate::interaction::WheelScrollPolicy, cx: &mut Context<Self>) {
        self.model.scroll_interaction.wheel = policy;
        cx.notify();
    }

    /// Change boundary without resetting focus, selection or position.
    pub fn set_scroll_boundary_policy(
        &mut self,
        policy: crate::interaction::ScrollBoundaryPolicy,
        cx: &mut Context<Self>,
    ) {
        self.model.scroll_interaction.boundary = policy;
        cx.notify();
    }

    /// Change focus_scope without resetting focus, selection or position.
    pub fn set_wheel_focus_scope(&mut self, policy: crate::interaction::WheelFocusScope, cx: &mut Context<Self>) {
        self.model.scroll_interaction.focus_scope = policy;
        cx.notify();
    }

    pub(crate) fn from_builder(builder: TextAreaBuilder, cx: &mut Context<Self>) -> Self {
        let mut state = TextAreaState { cursor: builder.model.value.chars().count(), ..Default::default() };
        let initial_rows = builder.model.rows.max(1);
        let resize_drag_id = format!("{}-resize", builder.model.id).into();

        let scrollbar = Scrollbar::new(format!("{}-scrollbar", builder.model.id))
            .orientation(ScrollbarOrientation::Vertical)
            .spawn(cx);
        let subscriptions = vec![cx.subscribe(&scrollbar, |this, _, event: &ScrollbarEvent, cx| {
            if let ScrollbarEvent::Change { value } = event {
                this.vertical_scroll = px(*value);
                this.clamp_vertical_scroll_to_cache();
                cx.notify();
            }
        })];

        let mut this = Self {
            model: builder.model,
            state,
            focus_handle: cx.focus_handle().tab_stop(true),
            theme_epoch: 0,
            change_count: 0,
            focus_count: 0,
            blur_count: 0,
            caret_visible: false,
            caret_paused: false,
            caret_epoch: 0,
            caret_task: Task::ready(()),
            mouse_selecting: false,
            selection_drag_position: None,
            selection_scroll_epoch: 0,
            selection_scroll_task: Task::ready(()),
            suppress_select_all_on_next_focus: false,
            marked_range: None,
            vertical_scroll: px(0.0),
            layout_cache: None,
            scrollbar,
            scrollbar_enabled: false,
            resize_drag_id,
            resize_dragging: false,
            resize_start_y: 0.0,
            resize_start_rows: initial_rows,
            _subscriptions: subscriptions,
        };
        state.clamp_cursor(this.model.value.chars().count());
        this.state = state;
        this.focus_handle = this.focus_handle.clone().tab_stop(this.model.enabled);
        this.recompute_invalid();
        this._subscriptions.push(observe_theme_revision(cx, |this, cx| {
            this.layout_cache = None;
            this.theme_epoch = this.theme_epoch.wrapping_add(1);
            cx.notify();
        }));
        this
    }

    pub fn value(&self) -> String {
        self.model.value.to_string()
    }

    pub fn state(&self) -> TextAreaState {
        self.state
    }

    pub fn change_count(&self) -> usize {
        self.change_count
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
        self.focus_handle = self.focus_handle.clone().tab_stop(enabled);
        if !enabled {
            self.state.set_hovered(false);
            self.mouse_selecting = false;
            self.stop_selection_autoscroll();
        }
        cx.emit(TextAreaEvent::EnabledChanged { enabled });
        cx.notify();
    }

    pub fn set_value(&mut self, value: impl Into<String>, cx: &mut Context<Self>) {
        self.model.value = value.into().into();
        self.state.cursor = self.model.value.chars().count();
        self.state.clear_selection();
        self.state.preferred_column = None;
        self.marked_range = None;
        self.recompute_invalid();
        self.ensure_cursor_visible();
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

    pub fn set_template(&mut self, template: std::sync::Arc<dyn super::TextAreaTemplate>, cx: &mut Context<Self>) {
        if let Some(theme) = template.theme() {
            self.model.theme = theme;
        }
        self.model.template = template;
        self.layout_cache = None;
        cx.notify();
    }

    pub fn set_theme(&mut self, theme: std::sync::Arc<dyn super::TextAreaTheme>, cx: &mut Context<Self>) {
        self.model.theme = theme;
        self.layout_cache = None;
        cx.notify();
    }

    pub fn set_look_override(
        &mut self,
        look_override: Option<super::model::TextAreaLookOverride>,
        cx: &mut Context<Self>,
    ) {
        self.model.look_override = look_override;
        self.layout_cache = None;
        cx.notify();
    }

    fn resolved_look(&self, scale: &StandardBoxScale) -> crate::controls::textarea::TextAreaLook {
        let mut look = self.model.theme.resolve_look(self.state, self.model.enabled, self.model.size, scale);
        if let Some(override_fn) = &self.model.look_override {
            look = override_fn(look);
        }
        look
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
            cx.emit(TextAreaEvent::FocusChanged { focused: true });
            self.start_caret_blink(cx);
        } else {
            if !self.mouse_selecting {
                self.stop_selection_autoscroll();
            }
            if self.model.select_all_on_tab_focus {
                self.state.clear_selection();
                self.state.cursor = self.model.value.chars().count();
            }
            self.blur_count += 1;
            cx.emit(TextAreaEvent::FocusChanged { focused: false });
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
        cx.emit(TextAreaEvent::Change { value: self.model.value.to_string() });
    }
}

impl Focusable for TextArea {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

#[cfg(all(test, feature = "test-support"))]
#[test]
fn wheel_policy_dispatch_matrix() {
    crate::interaction_tests::matrix(
        |policy, cx| {
            TextAreaBuilder::new("matrix-textarea")
                .value((0..100).map(|i| format!("line {i}\n")).collect::<String>())
                .rows(4)
                .wheel_scroll_policy(policy.wheel)
                .scroll_boundary_policy(policy.boundary)
                .spawn(cx)
        },
        |view, _cx| view.vertical_scroll.as_f32(),
        |view, cx| {
            view.vertical_scroll = px(100_000.0);
            view.clamp_vertical_scroll_to_cache();
            cx.notify();
        },
    );
}

#[cfg(all(test, feature = "test-support"))]
#[test]
fn scrollbar_focus_does_not_allow_ancestor_text_editing() {
    let mut app = gpui::TestAppContext::single();
    let (editor, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        TextArea::from_builder(TextAreaBuilder::new("scrollbar-key-owner").value("line\n".repeat(40)).rows(3), cx)
    });
    cx.run_until_parked();
    let original = cx.update(|window, app| {
        let view = editor.read(app);
        let focus = view.scrollbar.read(app).focus_handle(app);
        let original = view.model.value.clone();
        focus.focus(window, app);
        original
    });
    cx.run_until_parked();
    cx.simulate_keystrokes("backspace");
    cx.update(|window, app| {
        let view = editor.read(app);
        assert!(view.scrollbar.read(app).focus_handle(app).is_focused(window));
        assert_eq!(view.model.value, original);
        let focus = view.focus_handle.clone();
        focus.focus(window, app);
    });
    cx.run_until_parked();
    cx.simulate_keystrokes("backspace");
    cx.update(|_, app| assert_ne!(editor.read(app).model.value, original));
}

#[cfg(all(test, feature = "test-support"))]
#[test]
fn shaped_lines_reuse_and_invalidate_on_editing_and_style_changes() {
    use std::sync::Arc;

    let mut app = gpui::TestAppContext::single();
    let (editor, cx) = app.add_window_view(|window, cx| {
        window.activate_window();
        TextArea::from_builder(TextAreaBuilder::new("layout-reuse").value("Unicode é中🙂\n".repeat(40)).rows(3), cx)
    });
    cx.run_until_parked();
    let mut previous = cx.update(|_, app| editor.read(app).layout_cache.as_ref().unwrap().lines.clone());
    editor.update(cx, |view, cx| {
        view.state.cursor = 2;
        view.vertical_scroll = px(20.0);
        cx.notify();
    });
    cx.run_until_parked();
    cx.update(|_, app| {
        let cache = editor.read(app).layout_cache.as_ref().unwrap();
        assert!(Arc::ptr_eq(&previous, &cache.lines));
    });

    // Each change reaches prepaint through the real control, rather than testing the key in isolation.
    for change in 0..10 {
        editor.update(cx, |view, cx| {
            match change {
                0 => view.model.value = "changed é中🙂\n".repeat(40).into(),
                1 => view.marked_range = Some(0..3),
                2 => view.state.selection_anchor = Some(0),
                3..=9 => {
                    view.model.look_override = Some(Arc::new(move |mut look| {
                        look.typography.size += 2.0;
                        if change >= 4 {
                            look.typography.weight = gpui::FontWeight::BOLD;
                        }
                        if change >= 5 {
                            look.foreground = gpui::red();
                        }
                        if change >= 6 {
                            look.selection_foreground = gpui::blue();
                        }
                        if change >= 7 {
                            look.padding_x += 20.0;
                        }
                        if change >= 8 {
                            look.font_family = "monospace".to_owned();
                        }
                        if change >= 9 {
                            look.typography.line_height += 5.0;
                        }
                        look
                    }));
                }
                _ => unreachable!(),
            }
            cx.notify();
        });
        cx.run_until_parked();
        let current = cx.update(|_, app| editor.read(app).layout_cache.as_ref().unwrap().lines.clone());
        if matches!(change, 1 | 2 | 6) {
            assert!(Arc::ptr_eq(&previous, &current), "decoration change {change} must preserve geometry");
        } else {
            assert!(!Arc::ptr_eq(&previous, &current), "geometry change {change} must reshape");
        }
        previous = current;
        editor.update(cx, |_, cx| cx.notify());
        cx.run_until_parked();
        cx.update(|_, app| assert!(Arc::ptr_eq(&previous, &editor.read(app).layout_cache.as_ref().unwrap().lines)));
    }
}

#[cfg(all(test, feature = "test-support"))]
#[test]
fn wrapping_shapes_logical_lines_once_and_selection_decorates_only_visible_lines() {
    use gpui::prelude::*;
    use std::sync::Arc;

    let mut app = gpui::TestAppContext::single();
    let value = "office e\u{301} é中🙂 العربية ".repeat(100);
    let (editor, cx) = app.add_window_view(|_, cx| {
        TextArea::from_builder(
            TextAreaBuilder::new("shape-counts")
                .value(value.clone())
                .rows(3)
                .with_template_modifier(|control, _| control.w(px(180.0)))
                .look_override(|mut look| {
                    look.foreground = gpui::red();
                    look.selection_foreground = gpui::blue();
                    look
                }),
            cx,
        )
    });
    cx.run_until_parked();
    let initial = cx.update(|_, app| {
        let cache = editor.read(app).layout_cache.as_ref().unwrap();
        assert!(cache.lines.len() > 10, "fixture must wrap");
        assert_eq!(cache.lines.iter().map(|line| line.text.as_str()).collect::<String>(), value);
        assert!(cache.lines.iter().all(|line| line.end > line.start));
        cache.lines.clone()
    });
    let before = element::SHAPING_COUNTS.with(|counts| counts.get());
    editor.update(cx, |view, cx| {
        view.model.value = format!("{}z", view.model.value).into();
        cx.notify();
    });
    cx.run_until_parked();
    let after = element::SHAPING_COUNTS.with(|counts| counts.get());
    assert_eq!(after.0 - before.0, 1, "one logical line requires one geometry shaping call");
    assert_eq!(after.1, before.1, "plain wrapped segments must not be reshaped");
    let geometry = cx.update(|_, app| editor.read(app).layout_cache.as_ref().unwrap().lines.clone());
    assert!(!Arc::ptr_eq(&initial, &geometry));

    for end in [2, 5, 9] {
        let before = element::SHAPING_COUNTS.with(|counts| counts.get());
        editor.update(cx, |view, cx| {
            view.state.selection_anchor = Some(0);
            view.state.cursor = end;
            view.marked_range = Some(0..end);
            cx.notify();
        });
        cx.run_until_parked();
        let after = element::SHAPING_COUNTS.with(|counts| counts.get());
        assert_eq!(after.0, before.0, "selection and IME must preserve wrapping");
        cx.update(|window, app| {
            let cache = editor.read(app).layout_cache.as_ref().unwrap();
            assert!(Arc::ptr_eq(&geometry, &cache.lines));
            let affected = cache.paint_lines.iter().filter(|line| line.key.selection.is_some()).count();
            assert!(affected > 0);
            for painted in &cache.paint_lines {
                assert!(
                    Arc::ptr_eq(&painted.line, &cache.lines[painted.key.line_index].line),
                    "decorations must preserve contextual glyph geometry"
                );
            }
            assert_eq!(after.1 - before.1, affected, "only affected visible lines need new decorations");
            let underlines = window.painted_underlines();
            assert!(!underlines.is_empty(), "IME underline must reach paint");
            assert!(
                underlines.iter().all(|underline| underline.color == gpui::blue()),
                "selected composition must use the selection foreground"
            );
        });
        editor.update(cx, |_, cx| cx.notify());
        cx.run_until_parked();
        assert_eq!(element::SHAPING_COUNTS.with(|counts| counts.get()), after, "unchanged decoration must reuse");
    }
    editor.update(cx, |view, cx| {
        view.model.look_override = Some(Arc::new(|mut look| {
            look.foreground = gpui::red();
            look.selection_foreground = gpui::green();
            look
        }));
        cx.notify();
    });
    cx.run_until_parked();
    cx.update(|window, app| {
        assert!(Arc::ptr_eq(&geometry, &editor.read(app).layout_cache.as_ref().unwrap().lines));
        let underlines = window.painted_underlines();
        assert!(!underlines.is_empty());
        assert!(
            underlines.iter().all(|underline| underline.color == gpui::green()),
            "selection foreground changes must refresh painted decorations"
        );
    });
    editor.update(cx, |view, cx| {
        view.state.selection_anchor = None;
        view.marked_range = None;
        cx.notify();
    });
    cx.run_until_parked();
    cx.update(|window, app| {
        let cache = editor.read(app).layout_cache.as_ref().unwrap();
        assert!(Arc::ptr_eq(&geometry, &cache.lines));
        assert!(cache.paint_lines.iter().all(|line| line.key.selection.is_none() && line.key.marked_range.is_none()));
        assert!(window.painted_underlines().is_empty(), "unmarking must remove the underline");
    });
}

#[cfg(all(test, feature = "test-support"))]
mod perf_tests;
