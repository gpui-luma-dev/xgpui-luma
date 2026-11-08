//! Headless tests on an application whose text system is Parley, as in the lab's
//! `--native-text-backend=parley` process.
use super::*;
use gpui::{Entity, InputHandler, Modifiers, TestAppContext, TestDispatcher, VisualTestContext};
use std::{cell::RefCell, rc::Rc};

/// Window and field width. The field is also the window's root view, so both
/// frames must lay it out at the same width.
const WIDTH: f32 = 240.0;

pub(super) struct Fixture {
    pub(super) field: Entity<ParleyTextField>,
    pub(super) visual: VisualTestContext,
    pub(super) events: Rc<RefCell<Vec<ParleyTextFieldEvent>>>,
    /// `cmd` on macOS hosts, `ctrl` elsewhere: the field's secondary modifier.
    pub(super) secondary: &'static str,
}

impl Fixture {
    pub(super) fn new(value: &str) -> Self {
        Self::build(value, None)
    }

    /// A text area showing `rows` lines.
    pub(super) fn area(value: &str, rows: usize) -> Self {
        Self::build(value, Some(rows))
    }

    pub(super) fn build(value: &str, rows: Option<usize>) -> Self {
        Self::with(value, move |builder| match rows {
            Some(rows) => builder.rows(rows),
            None => builder,
        })
    }

    /// A field with builder options applied on top of the standard ones.
    pub(super) fn with(
        value: &str,
        configure: impl FnOnce(ParleyTextFieldBuilder) -> ParleyTextFieldBuilder + 'static,
    ) -> Self {
        let backend = crate::parley_lab::Engine::new().unwrap().backend.clone();
        let mut cx = TestAppContext::build_with_text_system(TestDispatcher::new(0), None, backend);
        cx.update(|cx| xgpui_luma::init(cx).unwrap());
        let secondary = if cx.update(|cx| cx.platform_info().is_mac()) {
            "cmd"
        } else {
            "ctrl"
        };
        let look = crate::theme::default_look();
        let value = value.to_owned();
        let window = cx.open_window(size(px(WIDTH), px(200.0)), move |_, cx| {
            let builder = ParleyTextField::new("test-field").look(&look).surface().font(page::font()).value(value);
            let builder = configure(builder);
            ParleyTextField::from_builder(builder, cx)
        });
        let field = window.root(&mut cx).unwrap();
        let events = Rc::new(RefCell::new(Vec::new()));
        let sink = events.clone();
        cx.update(|cx| {
            cx.subscribe(&field, move |_, event: &ParleyTextFieldEvent, _| sink.borrow_mut().push(event.clone()))
                .detach()
        });
        let visual = VisualTestContext::from_window(window.into(), &cx);
        let mut fixture = Self { field, visual, events, secondary };
        fixture.focus();
        fixture
    }

    pub(super) fn draw(&mut self) {
        let field = self.field.clone();
        self.visual
            .draw(point(px(0.0), px(0.0)), size(px(WIDTH), px(200.0)), move |_, _| div().w(px(WIDTH)).child(field));
    }

    pub(super) fn focus(&mut self) {
        let field = self.field.clone();
        self.visual.update(|window, cx| window.focus(&field.focus_handle(cx), cx));
        self.draw();
        self.draw(); // Focus syncs during render; the second frame paints with it.
    }

    pub(super) fn keys(&mut self, keystrokes: &str) {
        for keystroke in keystrokes.split_whitespace() {
            self.visual.simulate_keystrokes(keystroke);
            self.draw(); // Visual movement uses the painted layout.
        }
    }

    pub(super) fn read<R>(&mut self, f: impl FnOnce(&ParleyTextField) -> R) -> R {
        let field = self.field.clone();
        self.visual.update(|_, cx| f(field.read(cx)))
    }

    pub(super) fn value(&mut self) -> String {
        self.read(|field| field.value().to_owned())
    }

    pub(super) fn selection(&mut self) -> (usize, usize) {
        self.read(|field| (field.anchor.byte_index, field.caret.byte_index))
    }

    pub(super) fn geometry(&mut self) -> Geometry {
        self.read(|field| field.geometry.clone().expect("painted geometry"))
    }

    /// Window position of the caret before `byte_index`.
    pub(super) fn point_at(&mut self, byte_index: usize) -> Point<Pixels> {
        let geometry = self.geometry();
        let caret = geometry.layout.caret_bounds(at(byte_index)).unwrap();
        geometry.origin + caret.center()
    }

    pub(super) fn handler(&mut self) -> ElementInputHandler<ParleyTextField> {
        ElementInputHandler::new(self.geometry().viewport, self.field.clone())
    }

    /// The text viewport and the control height the resolved look gives.
    pub(super) fn viewport_and_height(mut self) -> (Bounds<Pixels>, f32) {
        let viewport = self.geometry().viewport;
        let field = self.field.clone();
        let height = self.visual.update(|window, cx| field.read(cx).look(window).min_height);
        (viewport, height)
    }

    pub(super) fn mouse_down(&mut self, position: Point<Pixels>, click_count: usize) {
        self.visual.simulate_event(MouseDownEvent {
            button: MouseButton::Left,
            position,
            modifiers: Modifiers::default(),
            click_count,
            first_mouse: false,
        });
        self.draw();
    }
}

#[test]
fn typing_and_ime_composition_use_utf16_offsets() {
    let mut f = Fixture::new("");
    f.visual.simulate_input("ab");
    f.draw();
    assert_eq!(f.value(), "ab");
    assert_eq!(f.selection(), (2, 2));

    let mut handler = f.handler();
    f.visual.update(|window, cx| {
        // 𝄞 is two UTF-16 units and four bytes; select the combining pair after it.
        handler.replace_and_mark_text_in_range(None, "𝄞e\u{301}", Some(2..4), window, cx);
        assert_eq!(handler.marked_text_range(window, cx), Some(2..6));
        assert_eq!(handler.selected_text_range(false, window, cx).unwrap().range, 4..6);
    });
    assert_eq!(f.value(), "ab𝄞e\u{301}");
    assert_eq!(f.selection(), (6, 9));
    f.draw();
    let mut handler = f.handler();
    f.visual.update(|window, cx| {
        let rect = handler.bounds_for_range(2..2, window, cx).expect("composition geometry");
        assert!(rect.size.height > px(0.0));
        handler.replace_text_in_range(None, "€", window, cx);
        assert_eq!(handler.marked_text_range(window, cx), None);
    });
    assert_eq!(f.value(), "ab€");
    assert!(matches!(f.events.borrow().last(), Some(ParleyTextFieldEvent::Change { value }) if value == "ab€"));
}

#[test]
fn backspace_and_delete_remove_whole_graphemes() {
    let mut f = Fixture::new("a👍🏽e\u{301}z");
    f.keys("left backspace");
    assert_eq!(f.value(), "a👍🏽z", "combining mark goes with its base");
    f.keys("backspace");
    assert_eq!(f.value(), "az", "emoji and skin-tone modifier go together");
    f.keys("home delete");
    assert_eq!(f.value(), "z");
}

#[test]
fn word_deletion_and_movement_skip_separators() {
    let mut f = Fixture::new("alpha beta, gamma");
    f.keys("alt-backspace");
    assert_eq!(f.value(), "alpha beta, ");
    f.keys("alt-left");
    assert_eq!(f.selection(), (6, 6));
    f.keys("alt-right");
    assert_eq!(f.selection(), (10, 10));
}

#[test]
fn left_and_right_move_visually_in_right_to_left_text() {
    let mut f = Fixture::new("שלום עולם");
    f.keys("home");
    assert!(f.read(|field| field.is_rtl()));
    let start = f.point_at(0);
    let end = f.point_at("שלום עולם".len());
    assert!(start.x > end.x, "logical start is at the right");
    // Visual left from the logical start moves logically forward.
    f.keys("left");
    assert!(f.selection().1 > 0);
    f.keys("right");
    assert_eq!(f.selection().1, 0);
    // Short right-to-left text starts at the field's right edge.
    let viewport = f.geometry().viewport;
    assert!(start.x > viewport.center().x, "{start:?} in {viewport:?}");
}

#[test]
fn shift_selects_and_plain_arrows_collapse_to_the_selection_edge() {
    let mut f = Fixture::new("hello");
    f.keys("home shift-right shift-right");
    assert_eq!(f.selection(), (0, 2));
    f.keys("left");
    assert_eq!(f.selection(), (0, 0));
    let select_all = format!("{}-a", f.secondary);
    f.keys(&select_all);
    assert_eq!(f.selection(), (0, 5));
    f.keys("right");
    assert_eq!(f.selection(), (5, 5));
}

#[test]
fn long_text_scrolls_to_keep_the_caret_visible() {
    let mut f = Fixture::new(&"wide words ".repeat(12));
    f.draw();
    assert!(f.read(|field| field.scroll()) > px(0.0), "caret starts at the end");
    let geometry = f.geometry();
    assert!(geometry.layout.size().width > geometry.viewport.size.width);
    f.keys("home");
    assert_eq!(f.read(|field| field.scroll()), px(0.0));
}

#[test]
fn clicks_place_the_caret_and_select_words_and_all() {
    let mut f = Fixture::new("hello world");
    let target = f.point_at(7);
    f.visual.simulate_click(target, Modifiers::default());
    f.draw();
    assert_eq!(f.selection(), (7, 7));
    f.mouse_down(target, 2);
    assert_eq!(f.selection(), (6, 11));
    f.mouse_down(target, 3);
    assert_eq!(f.selection(), (0, 11));
}

#[test]
fn paste_strips_line_breaks_and_enter_submits() {
    let mut f = Fixture::new("");
    f.visual
        .update(|_, cx| cx.write_to_clipboard(ClipboardItem::new_string("one\ntwo\r\nthree".into())));
    let paste = format!("{}-v", f.secondary);
    f.keys(&paste);
    assert_eq!(f.value(), "onetwothree");
    f.keys("enter");
    assert_eq!(f.events.borrow().last(), Some(&ParleyTextFieldEvent::Submit { value: "onetwothree".into() }));
    assert_eq!(f.value(), "onetwothree", "enter inserts nothing");
}

#[test]
fn drag_selection_follows_the_pointer_outside_the_field() {
    let mut f = Fixture::new("hello world");
    let start = f.point_at(2);
    f.mouse_down(start, 1);
    // Far right of the field and above it: still part of the drag.
    let outside = point(px(WIDTH + 120.0), px(-20.0));
    f.visual.simulate_event(MouseMoveEvent {
        position: outside,
        pressed_button: Some(MouseButton::Left),
        modifiers: Modifiers::default(),
    });
    f.draw();
    assert_eq!(f.selection(), (2, 11));
    f.visual.simulate_event(MouseMoveEvent {
        position: point(px(-50.0), start.y),
        pressed_button: Some(MouseButton::Left),
        modifiers: Modifiers::default(),
    });
    f.draw();
    assert_eq!(f.selection(), (2, 0), "dragging back past the start");
    // Releasing outside ends the drag; later moves change nothing.
    f.visual.simulate_event(MouseUpEvent {
        button: MouseButton::Left,
        position: outside,
        modifiers: Modifiers::default(),
        click_count: 1,
    });
    f.draw();
    f.visual
        .simulate_event(MouseMoveEvent { position: start, pressed_button: None, modifiers: Modifiers::default() });
    f.draw();
    assert_eq!(f.selection(), (2, 0));
    assert!(!f.read(|field| field.dragging));
}

/// Top of the caret's row, in layout coordinates.
fn caret_row(f: &mut Fixture) -> Pixels {
    let geometry = f.geometry();
    let caret = f.read(|field| field.caret);
    geometry.layout.caret_bounds(caret).unwrap().top()
}

#[test]
fn text_area_enter_inserts_line_breaks_and_clipboard_uses_newlines() {
    let mut f = Fixture::area("ab", 3);
    f.keys("enter");
    f.visual.simulate_input("cd");
    f.draw();
    assert_eq!(f.value(), "ab\ncd");
    assert!(f.events.borrow().iter().all(|event| !matches!(event, ParleyTextFieldEvent::Submit { .. })));
    let select_all = format!("{}-a", f.secondary);
    let copy = format!("{}-c", f.secondary);
    f.keys(&select_all);
    f.keys(&copy);
    let copied = f.visual.update(|_, cx| cx.read_from_clipboard().and_then(|item| item.text()));
    assert_eq!(copied.as_deref(), Some("ab\ncd"));
    // CRLF, CR and LF each paste as one line break.
    f.visual.update(|_, cx| cx.write_to_clipboard(ClipboardItem::new_string("1\r\n2\r3\n4".into())));
    let paste = format!("{}-v", f.secondary);
    f.keys(&paste);
    assert_eq!(f.value(), "1\n2\n3\n4");
}

#[test]
fn text_area_wraps_at_its_width() {
    let mut f = Fixture::area(&"wrapping words ".repeat(8), 4);
    let geometry = f.geometry();
    assert!(geometry.layout.size().width <= geometry.viewport.size.width + px(1.0));
    let start = geometry.layout.caret_bounds(at(0)).unwrap();
    let end = geometry.layout.caret_bounds(at(f.read(|field| field.value.len()))).unwrap();
    assert!(end.top() > start.top(), "soft-wrapped onto several rows");
}

#[test]
fn up_and_down_keep_the_caret_column_across_short_lines() {
    let mut f = Fixture::area("abcdefgh\nab\nabcdefgh", 4);
    let top = format!("{}-up", f.secondary);
    f.keys(&top);
    f.keys("right right right right right right");
    assert_eq!(f.selection(), (6, 6));
    let line = |text: &str| text.len() + LINE_SEPARATOR.len_utf8();
    let second = line("abcdefgh");
    let third = second + line("ab");
    f.keys("down");
    assert_eq!(f.selection().1, second + 2, "clamped to the short line's end");
    f.keys("down");
    assert_eq!(f.selection().1, third + 6, "column restored");
    f.keys("up up");
    assert_eq!(f.selection().1, 6);
    f.keys("up");
    assert_eq!(f.selection().1, 0, "up from the first line goes to the start");
    f.keys("shift-down");
    assert_eq!(f.selection().0, 0, "shift extends");
}

#[test]
fn home_and_end_stay_on_the_visual_line() {
    let text = "first row words then more words that wrap onto another row of the area, and a third row after it";
    let mut f = Fixture::area(text, 4);
    // Click mid-way along the second visual row.
    let geometry = f.geometry();
    let first = geometry.layout.caret_bounds(at(0)).unwrap();
    let row = point(geometry.origin.x + px(60.0), geometry.origin.y + first.bottom() + first.size.height / 2.0);
    f.visual.simulate_click(row, Modifiers::default());
    f.draw();
    let clicked_row = caret_row(&mut f);
    assert!(clicked_row > first.top());
    f.keys("home");
    let start = f.selection().1;
    assert!(start > 0 && caret_row(&mut f) == clicked_row, "start of this row, not of the text");
    f.keys("end");
    let end = f.selection().1;
    assert!(end > start && end < text.len(), "end of this row: {end}");
    assert_eq!(caret_row(&mut f), clicked_row, "the end caret stays on its row");
    // Cmd/Ctrl-Backspace deletes back to this row's start only.
    let delete_line = format!("{}-backspace", f.secondary);
    f.keys(&delete_line);
    assert_eq!(f.value(), format!("{}{}", &text[..start], &text[end..]));
}

#[test]
fn text_area_scrolls_vertically_to_the_caret_and_with_the_wheel() {
    let mut f = Fixture::area("one\ntwo\nthree\nfour\nfive\nsix", 2);
    f.draw();
    assert!(f.read(|field| field.scroll()) > px(0.0), "caret starts on the last line");
    let top = format!("{}-up", f.secondary);
    f.keys(&top);
    assert_eq!(f.read(|field| field.scroll()), px(0.0));
    let center = f.geometry().viewport.center();
    f.visual.simulate_event(ScrollEvent {
        position: center,
        delta: gpui::ScrollDelta::Pixels(point(px(0.0), px(-30.0))),
        ..Default::default()
    });
    f.draw();
    assert_eq!(f.read(|field| field.scroll()), px(30.0), "wheel scrolls the content");
    // Clicking below the text lands on the last line.
    let viewport = f.geometry().viewport;
    f.visual
        .simulate_click(point(viewport.left() + px(4.0), viewport.bottom() - px(2.0)), Modifiers::default());
    f.draw();
    assert!(f.selection().1 > 0);
}

#[test]
fn builder_callbacks_receive_sdk_named_events() {
    struct Owner {
        log: Vec<String>,
    }
    let backend = crate::parley_lab::Engine::new().unwrap().backend.clone();
    let mut cx = TestAppContext::build_with_text_system(TestDispatcher::new(0), None, backend);
    cx.update(|cx| xgpui_luma::init(cx).unwrap());
    let owner = cx.new(|_| Owner { log: Vec::new() });
    let field = owner.update(&mut cx, |_, cx| {
        ParleyTextField::new("callbacks")
            .value("hi")
            .on_value_changed(cx, |owner: &mut Owner, value, _| owner.log.push(format!("change {value}")))
            .on_submit(cx, |owner: &mut Owner, value, _| owner.log.push(format!("submit {value}")))
            .on_focus_changed(cx, |owner: &mut Owner, focused, _| owner.log.push(format!("focus {focused}")))
            .on_enabled_changed(cx, |owner: &mut Owner, enabled, _| owner.log.push(format!("enabled {enabled}")))
            .spawn(cx)
    });
    field.update(&mut cx, |field, cx| {
        assert_eq!(field.value(), "hi");
        field.select_all(cx);
        field.replace(field.selected_range(), "yo", None, false, cx);
        cx.emit(ParleyTextFieldEvent::Submit { value: field.value() });
        field.set_enabled(false, cx);
        field.set_enabled(false, cx); // No change, no event.
    });
    cx.run_until_parked();
    let log = owner.read_with(&cx, |owner, _| owner.log.clone());
    assert_eq!(log, ["change yo", "submit yo", "enabled false"]);
}

#[test]
fn set_value_replaces_the_text_without_a_change_event() {
    let mut f = Fixture::area("old", 3);
    let field = f.field.clone();
    f.visual.update(|_, cx| field.update(cx, |field, cx| field.set_value("one\r\ntwo", cx)));
    f.draw();
    assert_eq!(f.value(), "one\ntwo");
    assert_eq!(f.selection().1, f.read(|field| field.value.len()), "caret at the end");
    assert!(f.events.borrow().iter().all(|event| !matches!(event, ParleyTextFieldEvent::Change { .. })));
}

#[test]
fn disabled_control_ignores_input_and_gives_up_focus() {
    let mut f = Fixture::new("fixed");
    assert!(f.read(|field| field.is_focused()));
    let field = f.field.clone();
    f.visual.update(|_, cx| field.update(cx, |field, cx| field.set_enabled(false, cx)));
    f.draw();
    f.draw();
    assert!(!f.read(|field| field.is_focused()), "disabling blurs");
    assert!(f.events.borrow().contains(&ParleyTextFieldEvent::EnabledChanged { enabled: false }));
    assert!(f.events.borrow().contains(&ParleyTextFieldEvent::FocusChanged { focused: false }));
    let target = f.point_at(2);
    f.visual.simulate_click(target, Modifiers::default());
    f.draw();
    f.visual.simulate_keystrokes("backspace");
    f.visual.simulate_input("x");
    f.draw();
    assert_eq!(f.value(), "fixed");
    assert!(!f.read(|field| field.is_focused()), "clicks do not focus a disabled control");
    // Re-enabled, it takes focus and input again.
    f.visual.update(|_, cx| field.update(cx, |field, cx| field.set_enabled(true, cx)));
    f.draw();
    f.visual.simulate_click(target, Modifiers::default());
    f.draw();
    f.visual.simulate_input("x");
    f.draw();
    assert_eq!(f.value(), "fixxed");
}

/// Background a builder's resolved theme gives a resting, enabled field.
fn resolved_background(builder: &ParleyTextFieldBuilder, cx: &App) -> Hsla {
    let theme = builder.resolve_theme(cx);
    let scale = StandardBoxScale::compute(ControlSize::Md, &theme.metrics(), 1.0);
    theme.resolve_look(TextFieldState::default(), true, ControlSize::Md, &scale).background
}

#[test]
fn look_resolves_explicit_then_ambient_then_built_in() {
    use xgpui_luma_look_radix::Look;
    let cx = TestAppContext::single();
    cx.update(|cx| {
        xgpui_luma::init(cx).unwrap();
        let lab = crate::theme::default_look().as_ref().clone();
        let built_in = Look::built_in();
        let builder = || ParleyTextField::new("look");
        let built_in_background = resolved_background(&builder(), cx);
        let lab_background = resolved_background(&builder().look(&lab), cx);
        assert_ne!(built_in_background, lab_background, "the two looks must differ for this test");
        assert_eq!(built_in_background, resolved_background(&builder().look(&built_in), cx));
        // A shell installs its look as the ambient one.
        cx.set_global(lab.clone());
        assert_eq!(resolved_background(&builder(), cx), lab_background, "ambient look");
        assert_eq!(resolved_background(&builder().look(&built_in), cx), built_in_background, "explicit wins");
    });
}

#[test]
fn look_override_adjusts_the_resolved_look() {
    let backend = crate::parley_lab::Engine::new().unwrap().backend.clone();
    let mut cx = TestAppContext::build_with_text_system(TestDispatcher::new(0), None, backend);
    cx.update(|cx| xgpui_luma::init(cx).unwrap());
    let red = gpui::red();
    let window = cx.open_window(size(px(WIDTH), px(80.0)), move |_, cx| {
        let builder = ParleyTextField::new("override").look_override(move |mut look| {
            look.background = red;
            look
        });
        ParleyTextField::from_builder(builder, cx)
    });
    let background = window.update(&mut cx, |field, window, _| field.look(window).background).unwrap();
    assert_eq!(background, red);
}

#[test]
fn one_line_chrome_comes_from_the_sdk_template_with_modifiers() {
    let backend = crate::parley_lab::Engine::new().unwrap().backend.clone();
    let mut cx = TestAppContext::build_with_text_system(TestDispatcher::new(0), None, backend);
    cx.update(|cx| xgpui_luma::init(cx).unwrap());
    let look = crate::theme::default_look();
    let window = cx.open_window(size(px(WIDTH), px(80.0)), move |_, cx| {
        let builder = ParleyTextField::new("templated")
            .look(&look)
            .surface()
            .font(page::font())
            .value("hello")
            .with_template_modifier(|root, _| root.debug_selector(|| "template-root".into()));
        ParleyTextField::from_builder(builder, cx)
    });
    let field = window.root(&mut cx).unwrap();
    let mut visual = VisualTestContext::from_window(window.into(), &cx);
    let draw = |visual: &mut VisualTestContext| {
        let field = field.clone();
        visual.draw(point(px(0.0), px(0.0)), size(px(WIDTH), px(80.0)), move |_, _| div().w(px(WIDTH)).child(field));
    };
    draw(&mut visual);
    let chrome = visual.debug_bounds("template-root").expect("the modifier ran on the template's root");
    let (viewport, inset) = visual.update(|window, cx| {
        let field = field.read(cx);
        let look = field.look(window);
        let inset = field.theme.metrics().focus.gap + 2.0 * look.border_width.max(0.0) + look.padding_x;
        (field.geometry.as_ref().expect("painted").viewport, px(inset))
    });
    // The Parley text sits inside the template's focus ring, border and padding.
    assert_eq!(viewport.left(), chrome.left() + inset);
    assert_eq!(viewport.right(), chrome.right() - inset);
    assert!(viewport.top() > chrome.top() && viewport.bottom() < chrome.bottom());
}

/// Layout-local caret rectangle at a byte offset, with the given side.
fn caret_at(f: &mut Fixture, byte_index: usize, affinity: TextAffinity) -> Bounds<Pixels> {
    f.geometry().layout.caret_bounds(ParagraphPosition { byte_index, affinity }).unwrap()
}

#[test]
fn each_text_area_line_has_its_own_direction() {
    let hebrew = "שלום עולם";
    let english = "office café 👍🏽";
    let mut f = Fixture::area(&format!("{hebrew}\n{english}"), 4);
    let width = f.geometry().layout.size().width;
    let second = hebrew.len() + LINE_SEPARATOR.len_utf8();
    // Hebrew first line: right to left, starting at the right edge.
    let (start, end) = (
        caret_at(&mut f, 0, TextAffinity::Downstream),
        caret_at(&mut f, hebrew.len(), TextAffinity::Upstream),
    );
    assert!(start.left() > end.left() && start.left() > width / 2.0, "{start:?} {end:?}");
    // English second line: left to right, starting at the left edge, emoji last.
    let total = second + english.len();
    let (start, end) =
        (caret_at(&mut f, second, TextAffinity::Downstream), caret_at(&mut f, total, TextAffinity::Upstream));
    assert!(start.left() < end.left() && start.left() < px(2.0), "{start:?} {end:?}");
    assert!(start.top() > caret_at(&mut f, 0, TextAffinity::Downstream).top(), "stacked below the first line");
    let emoji = second + english.find('👍').unwrap();
    assert!(
        caret_at(&mut f, emoji, TextAffinity::Downstream).left()
            > caret_at(&mut f, second + 6, TextAffinity::Downstream).left()
    );
}

#[test]
fn left_and_right_continue_onto_neighbouring_lines() {
    let mut f = Fixture::area("ab\ncd", 3);
    let top = format!("{}-up", f.secondary);
    f.keys(&top);
    f.keys("right right");
    assert_eq!(f.selection().1, 2, "end of the first line");
    f.keys("right");
    assert_eq!(f.selection().1, 2 + LINE_SEPARATOR.len_utf8(), "on to the second line's start");
    f.keys("left");
    assert_eq!(f.selection().1, 2, "back to the first line's end");

    // In right-to-left lines Left reads forward, so it continues downwards.
    let mut f = Fixture::area("אב\nגד", 3);
    f.keys(&top);
    f.keys("left left");
    assert_eq!(f.selection().1, "אב".len());
    f.keys("left");
    assert_eq!(f.selection().1, "אב".len() + LINE_SEPARATOR.len_utf8());
    f.keys("right");
    assert_eq!(f.selection().1, "אב".len());
}

#[test]
fn selecting_across_line_breaks_marks_each_break() {
    let mut f = Fixture::area("ab\n\ncd", 3);
    let layout = f.geometry().layout;
    let rows = |rects: &[Bounds<Pixels>]| {
        let mut tops: Vec<_> = rects.iter().map(|rect| f32::from(rect.top()).round() as i32).collect();
        tops.dedup();
        tops.len()
    };
    let all = layout.selection_bounds(at(0), at(layout.source().len())).unwrap();
    assert_eq!(rows(&all), 3, "text, the empty line's break and the last line: {all:?}");
    // Within one line there is no break mark.
    let within = layout.selection_bounds(at(0), at(1)).unwrap();
    assert_eq!(within.len(), 1);
    // A selection ending exactly at a line's end does not include its break.
    let to_end = layout.selection_bounds(at(0), at(2)).unwrap();
    assert_eq!(rows(&to_end), 1);
}

#[test]
fn validator_marks_the_field_invalid_and_drops_its_focus_ring() {
    let digits: Validator = Arc::new(|text: &str| text.chars().all(|ch| ch.is_ascii_digit()));
    let mut f = Fixture::with("123", move |builder| builder.validator(digits));
    assert!(!f.read(|field| field.is_invalid()));
    // A visible ring is any opaque ring color; an invalid field's ring is transparent.
    let ring = |f: &mut Fixture| {
        f.visual
            .update(|window, cx| f.field.read(cx).look(window).focus_border.is_some_and(|ring| ring.a > 0.0))
    };
    assert!(ring(&mut f), "a valid, enabled field has a focus ring");
    f.visual.simulate_input("x");
    f.draw();
    assert!(f.read(|field| field.is_invalid()), "typing a letter fails the validator");
    assert!(!ring(&mut f), "an invalid field keeps its resting border and shows no ring");
    f.keys("backspace");
    assert!(!f.read(|field| field.is_invalid()));
    let field = f.field.clone();
    f.visual.update(|_, cx| field.update(cx, |field, cx| field.set_invalid(true, cx)));
    assert!(f.read(|field| field.is_invalid()), "set_invalid holds until the text changes");
}

#[test]
fn escape_clears_only_with_clean_on_escape() {
    let mut f = Fixture::new("keep");
    f.keys("escape");
    assert_eq!(f.value(), "keep");
    let mut f = Fixture::with("clear me", |builder| builder.clean_on_escape(true));
    f.keys("escape");
    assert_eq!(f.value(), "");
    assert!(matches!(f.events.borrow().last(), Some(ParleyTextFieldEvent::Change { value }) if value.is_empty()));
}

#[test]
fn paste_is_capped_on_a_character_boundary() {
    let mut f = Fixture::with("", |builder| builder.max_clipboard_paste_bytes(Some(4)));
    f.visual.update(|_, cx| cx.write_to_clipboard(ClipboardItem::new_string("héllo".into())));
    let paste = format!("{}-v", f.secondary);
    f.keys(&paste);
    assert_eq!(f.value(), "hél", "4 bytes would split nothing; 'é' is 2 bytes");
    let mut f = Fixture::with("", |builder| builder.max_clipboard_paste_bytes(Some(2)));
    f.visual.update(|_, cx| cx.write_to_clipboard(ClipboardItem::new_string("héllo".into())));
    f.keys(&paste);
    assert_eq!(f.value(), "h", "2 bytes would split 'é', so it stops before it");
}

#[test]
fn keyboard_focus_selects_all_and_blur_collapses_to_the_end() {
    let mut f = Fixture::with("select me", |builder| builder.select_all_on_tab_focus(true));
    f.visual.update(|window, cx| window.blur(cx));
    f.draw();
    f.visual.simulate_keystrokes("shift"); // The last input is now the keyboard.
    f.focus();
    assert_eq!(f.selection(), (0, 9), "keyboard focus selects all");
    f.visual.update(|window, cx| window.blur(cx));
    f.draw();
    assert_eq!(f.selection(), (9, 9), "blur collapses to the end");
    // A click focuses without selecting all.
    let target = f.point_at(3);
    f.visual.simulate_click(target, Modifiers::default());
    f.draw();
    assert_eq!(f.selection(), (3, 3));
}

#[test]
fn prefix_icon_and_compact_follow_the_template() {
    let (plain, plain_height) = Fixture::new("text").viewport_and_height();
    let (with_icon, _) =
        Fixture::with("text", |builder| builder.prefix_icon(lucide_svg_static::Icon::Search)).viewport_and_height();
    let (_, compact_height) = Fixture::with("text", |builder| builder.compact()).viewport_and_height();
    let shift = with_icon.left() - plain.left();
    assert!(shift > px(8.0), "text starts after the icon: {shift:?}");
    assert_eq!(with_icon.right(), plain.right(), "the icon only takes room on the left");
    assert!(compact_height < plain_height, "compact uses the stylesheet's shorter height");
}

/// A parent view whose key listener counts Home presses that reach it.
struct Host {
    field: Entity<ParleyTextField>,
    homes: Rc<RefCell<usize>>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let homes = self.homes.clone();
        div()
            .w(px(WIDTH))
            .on_key_down(move |event: &KeyDownEvent, _, _| {
                if event.keystroke.key == "home" {
                    *homes.borrow_mut() += 1;
                }
            })
            .child(self.field.clone())
    }
}

#[test]
fn home_and_end_reach_the_parent_only_when_asked() {
    for propagate in [false, true] {
        let backend = crate::parley_lab::Engine::new().unwrap().backend.clone();
        let mut cx = TestAppContext::build_with_text_system(TestDispatcher::new(0), None, backend);
        cx.update(|cx| xgpui_luma::init(cx).unwrap());
        let homes = Rc::new(RefCell::new(0));
        let seen = homes.clone();
        let window = cx.open_window(size(px(WIDTH), px(80.0)), move |_, cx| {
            let field = ParleyTextField::new("host-field")
                .font(page::font())
                .value("text")
                .propagate_home_end_to_parent(propagate)
                .spawn(cx);
            Host { field, homes: seen }
        });
        let field = window.update(&mut cx, |host, _, _| host.field.clone()).unwrap();
        let mut visual = VisualTestContext::from_window(window.into(), &cx);
        visual.update(|window, cx| window.focus(&field.focus_handle(cx), cx));
        visual.run_until_parked();
        visual.simulate_keystrokes("home");
        let caret = visual.update(|_, cx| field.read(cx).caret.byte_index);
        assert_eq!(caret, 0, "the field still moves");
        assert_eq!(*homes.borrow() > 0, propagate, "propagate {propagate}");
    }
}

#[test]
fn tab_stop_follows_the_option_and_the_enabled_state() {
    let mut f = Fixture::with("text", |builder| builder.tab_stop(false));
    assert!(!f.read(|field| field.focus.tab_stop));
    let field = f.field.clone();
    f.visual.update(|_, cx| field.update(cx, |field, cx| field.set_tab_stop(true, cx)));
    assert!(f.read(|field| field.focus.tab_stop));
    f.visual.update(|_, cx| field.update(cx, |field, cx| field.set_enabled(false, cx)));
    assert!(!f.read(|field| field.focus.tab_stop), "disabled fields are never tab stops");
}

#[test]
fn disabling_or_invalidating_keeps_the_field_size() {
    let mut f = Fixture::new("steady");
    let field = f.field.clone();
    let layout = |f: &mut Fixture| {
        f.draw();
        let viewport = f.geometry().viewport;
        let height = f.visual.update(|window, cx| {
            let field = f.field.read(cx);
            let look = field.look(window);
            look.min_height + 2.0 * (field.theme.metrics().focus.gap + look.border_width.max(0.0))
        });
        (viewport, height)
    };
    let enabled = layout(&mut f);
    f.visual.update(|_, cx| field.update(cx, |field, cx| field.set_enabled(false, cx)));
    assert_eq!(layout(&mut f), enabled, "disabled: same text position and outer height");
    f.visual.update(|_, cx| field.update(cx, |field, cx| field.set_enabled(true, cx)));
    f.visual.update(|_, cx| field.update(cx, |field, cx| field.set_invalid(true, cx)));
    assert_eq!(layout(&mut f), enabled, "invalid: same text position and outer height");
}

/// The old measurement, kept as an oracle: the extent over every character boundary.
fn width_by_every_caret(layout: &dyn PlatformParagraphLayout, text: &str) -> Pixels {
    let (mut left, mut right) = (px(f32::MAX), px(f32::MIN));
    for index in text.char_indices().map(|(index, _)| index).chain([text.len()]) {
        let caret = layout.caret_bounds(at(index)).unwrap();
        left = left.min(caret.left());
        right = right.max(caret.right());
    }
    right - left
}

#[test]
fn line_width_from_edges_matches_every_caret() {
    let mut f = Fixture::new("");
    let window_system = f.visual.update(|window, _| window.text_system().clone());
    let font_id = window_system.resolve_font(&page::font());
    for text in ["office café", "שלום עולם", "abc אבג 123 مرحبا", "👍🏽 emoji 🇮🇹 flags", "שלום office עולם"]
    {
        let runs = [FontRun { len: text.len(), font_id }];
        let width = px(2_000.0);
        let layout = window_system.layout_paragraph(text.into(), px(14.0), &runs, width).unwrap();
        let edges = line_width(layout.as_ref()).unwrap();
        let every = width_by_every_caret(layout.as_ref(), text);
        assert!((edges - every).abs() <= px(1.0), "{text:?}: edges {edges:?}, every caret {every:?}");
    }
}

#[test]
fn a_long_unbreakable_line_scrolls_to_its_end() {
    // No break opportunities: the line never wraps, it overflows the field. The
    // old code took "does not wrap" for "fits" and could not scroll to the end.
    let text = "x".repeat(150_000);
    let mut f = Fixture::new(&text);
    let layout = f.geometry().layout;
    assert!(!wrapped(layout.as_ref(), &text).unwrap(), "one row");
    assert!(f.read(|field| field.scroll()) > px(500_000.0), "scrolled to the caret at the end");
}

#[test]
fn width_is_measured_once_per_text_change() {
    let mut f = Fixture::new("measure me");
    let cached =
        |f: &mut Fixture| f.read(|field| field.measured.get().map(|(revision, ..)| (revision, field.revision)));
    let first = cached(&mut f).expect("measured while drawing");
    assert_eq!(first.0, first.1, "for the current text");
    f.draw();
    assert_eq!(cached(&mut f), Some(first), "a redraw reuses it");
    f.visual.simulate_input("!");
    f.draw();
    let after = cached(&mut f).unwrap();
    assert!(after.1 > first.1 && after.0 == after.1, "an edit measures again: {first:?} -> {after:?}");
}

/// Manual: `cargo test --release -p xgpui-lab --features test-support --bin
/// xgpui-lab measure_long_one_line_field -- --ignored --nocapture`.
#[test]
#[ignore = "manual headless performance measurement"]
fn measure_long_one_line_field() {
    use std::time::Instant;
    let ms = |started: Instant| started.elapsed().as_secs_f64() * 1e3;
    for len in [10_000usize, 100_000, 1_000_000] {
        let text = "x".repeat(len);
        let started = Instant::now();
        let mut f = Fixture::new(&text);
        let open = ms(started);
        let started = Instant::now();
        f.draw();
        let warm = ms(started);
        let started = Instant::now();
        f.visual.simulate_input("y");
        f.draw();
        let typed = ms(started);
        println!(
            "ParleyTextField/headless {len} chars: open {open:.0} ms | warm frame {warm:.1} ms | type 1 char {typed:.0} ms"
        );
    }
}

#[test]
fn an_unbreakable_word_wider_than_the_field_scrolls_both_ways() {
    // No spaces or hyphens: nothing to break at, in either direction.
    for text in ["abcdefghij".repeat(8), "אבגדהוזחטי".repeat(8)] {
        let text = text.as_str();
        let mut f = Fixture::new(text);
        let geometry = f.geometry();
        let extent = width_by_every_caret(geometry.layout.as_ref(), text);
        assert!(extent > geometry.viewport.size.width, "{text}: wider than the field");
        assert!(geometry.layout.size().width >= extent, "{text}: laid out wide enough for all of it");
        // Both ends are reachable by scrolling.
        f.keys("home");
        let start = f.read(|field| field.scroll());
        f.keys("end");
        let end = f.read(|field| field.scroll());
        assert_ne!(start, end, "{text}: scrolling moves between the ends");
    }
}

/// Draws the lab page; reports whether its first field was laid out, and
/// whether it tried and failed.
fn page_lays_out_fields(parley: bool) -> (bool, bool) {
    let mut cx = if parley {
        let backend = crate::parley_lab::Engine::new().unwrap().backend.clone();
        TestAppContext::build_with_text_system(TestDispatcher::new(0), None, backend)
    } else {
        TestAppContext::single()
    };
    cx.update(|cx| {
        xgpui_luma::init(cx).unwrap();
        if parley {
            cx.set_global(crate::parley_lab::ParleyTextSystemActive);
        }
    });
    let look = crate::theme::default_look();
    let window = cx.open_window(size(px(600.0), px(900.0)), move |_, cx| page::TextFieldPage::new(look, false, cx));
    let page = window.root(&mut cx).unwrap();
    let mut visual = VisualTestContext::from_window(window.into(), &cx);
    visual.run_until_parked();
    let page = page.clone();
    visual.draw(point(px(0.0), px(0.0)), size(px(600.0), px(900.0)), move |_, _| {
        div().size_full().child(page.clone())
    });
    window
        .update(&mut cx, |page, _, cx| {
            let field = page.fields[0].read(cx);
            (field.geometry.is_some(), field.layout_error().is_some())
        })
        .unwrap()
}

#[test]
fn lab_page_edits_on_parley_and_explains_on_the_native_text_system() {
    assert_eq!(page_lays_out_fields(true), (true, false), "on Parley, the fields are live");
    assert_eq!(
        page_lays_out_fields(false),
        (false, false),
        "on the native text system, the page shows a note and never lays the fields out"
    );
}
