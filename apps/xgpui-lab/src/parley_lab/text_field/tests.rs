//! Headless tests on an application whose text system is Parley, as in the lab's
//! `--native-text-backend=parley` process.
use super::*;
use gpui::{Entity, InputHandler, Modifiers, TestAppContext, TestDispatcher, VisualTestContext};
use std::{cell::RefCell, rc::Rc};

/// Window and field width. The field is also the window's root view, so both
/// frames must lay it out at the same width.
const WIDTH: f32 = 240.0;

struct Fixture {
    field: Entity<ParleyTextField>,
    visual: VisualTestContext,
    events: Rc<RefCell<Vec<ParleyTextFieldEvent>>>,
    /// `cmd` on macOS hosts, `ctrl` elsewhere: the field's secondary modifier.
    secondary: &'static str,
}

impl Fixture {
    fn new(value: &str) -> Self {
        Self::build(value, None)
    }

    /// A text area showing `rows` lines.
    fn area(value: &str, rows: usize) -> Self {
        Self::build(value, Some(rows))
    }

    fn build(value: &str, rows: Option<usize>) -> Self {
        let backend = crate::parley_lab::Engine::new().unwrap().backend.clone();
        let mut cx = TestAppContext::build_with_text_system(TestDispatcher::new(0), None, backend);
        cx.update(|cx| xgpui_luma::init(cx).unwrap());
        let secondary = if cx.update(|cx| cx.platform_info().is_mac()) {
            "cmd"
        } else {
            "ctrl"
        };
        let look = crate::theme::default_look();
        let theme =
            xgpui_luma_look_radix::textfield_theme_with(&look, xgpui_luma_look_radix::TextFieldVariant::Surface);
        let value = value.to_owned();
        let window = cx.open_window(size(px(WIDTH), px(200.0)), move |_, cx| {
            let field = ParleyTextField::new("test-field", theme, page::font(), cx);
            match rows {
                Some(rows) => field.with_rows(rows),
                None => field,
            }
            .with_value(&value)
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

    fn draw(&mut self) {
        let field = self.field.clone();
        self.visual
            .draw(point(px(0.0), px(0.0)), size(px(WIDTH), px(200.0)), move |_, _| div().w(px(WIDTH)).child(field));
    }

    fn focus(&mut self) {
        let field = self.field.clone();
        self.visual.update(|window, cx| window.focus(&field.focus_handle(cx), cx));
        self.draw();
        self.draw(); // Focus syncs during render; the second frame paints with it.
    }

    fn keys(&mut self, keystrokes: &str) {
        for keystroke in keystrokes.split_whitespace() {
            self.visual.simulate_keystrokes(keystroke);
            self.draw(); // Visual movement uses the painted layout.
        }
    }

    fn read<R>(&mut self, f: impl FnOnce(&ParleyTextField) -> R) -> R {
        let field = self.field.clone();
        self.visual.update(|_, cx| f(field.read(cx)))
    }

    fn value(&mut self) -> String {
        self.read(|field| field.value().to_owned())
    }

    fn selection(&mut self) -> (usize, usize) {
        self.read(|field| (field.anchor.byte_index, field.caret.byte_index))
    }

    fn geometry(&mut self) -> Geometry {
        self.read(|field| field.geometry.clone().expect("painted geometry"))
    }

    /// Window position of the caret before `byte_index`.
    fn point_at(&mut self, byte_index: usize) -> Point<Pixels> {
        let geometry = self.geometry();
        let caret = geometry.layout.caret_bounds(at(byte_index)).unwrap();
        geometry.origin + caret.center()
    }

    fn handler(&mut self) -> ElementInputHandler<ParleyTextField> {
        ElementInputHandler::new(self.geometry().viewport, self.field.clone())
    }

    fn mouse_down(&mut self, position: Point<Pixels>, click_count: usize) {
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
    assert!(matches!(f.events.borrow().last(), Some(ParleyTextFieldEvent::Change(value)) if value == "ab€"));
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
    assert_eq!(f.events.borrow().last(), Some(&ParleyTextFieldEvent::Submit("onetwothree".into())));
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
    assert!(f.events.borrow().iter().all(|event| !matches!(event, ParleyTextFieldEvent::Submit(_))));
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
