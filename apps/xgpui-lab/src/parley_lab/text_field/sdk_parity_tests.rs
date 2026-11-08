//! The SDK text field and text area tests, ported to the Parley control.
//!
//! Each test names the SDK test (`crates/sdk/src/controls/text`) whose
//! behaviour it checks. SDK tests of SDK-only helpers (character/byte offset
//! maps, fixed-height row arithmetic) have no counterpart here; their
//! behaviour is covered through these tests. SDK features the Parley control
//! does not have yet are listed in #105, not tested here.
use super::tests::Fixture;
use super::*;
use gpui::{Entity, InputHandler, Modifiers, ScrollDelta, TestAppContext, TestDispatcher};
use xgpui_luma_look_radix::TextFieldSize;

/// SDK `editing::typed_text_ignores_secondary_shortcuts` and
/// `typed_text_ignores_tab_control_character`.
#[test]
fn shortcuts_and_tab_do_not_insert_text() {
    let mut f = Fixture::new("ab");
    let shortcut = format!("{}-b", f.secondary);
    f.keys(&shortcut);
    f.keys("tab");
    assert_eq!(f.value(), "ab");
}

/// SDK `editing::movement_targets_clamp_to_bounds`.
#[test]
fn movement_and_deletion_stop_at_the_ends() {
    let mut f = Fixture::new("ab");
    f.keys("right delete");
    assert_eq!((f.value(), f.selection()), ("ab".to_owned(), (2, 2)), "nothing after the end");
    f.keys("home left backspace");
    assert_eq!((f.value(), f.selection()), ("ab".to_owned(), (0, 0)), "nothing before the start");
}

/// SDK `editing::double_click_word_cluster_selects_whitespace_run`.
#[test]
fn double_click_on_spaces_selects_the_whole_run() {
    let mut f = Fixture::new("a   b");
    let target = f.point_at(2);
    f.mouse_down(target, 2);
    assert_eq!(f.selection(), (1, 4));
}

/// SDK `editing::paste_without_budget_keeps_the_full_clipboard`,
/// `cap_clipboard_paste_none_is_unlimited` and the default budget of
/// `paste_respects_configured_byte_budget`.
///
/// The default (1 MiB) is checked directly: pasting a mebibyte into a one-line
/// field currently takes about 12 s headless (tracked in #105).
#[test]
fn paste_budget_defaults_to_one_mebibyte_and_none_is_unlimited() {
    assert_eq!(Options::default().max_clipboard_paste_bytes, Some(DEFAULT_MAX_CLIPBOARD_PASTE_BYTES));
    let big = "x".repeat(DEFAULT_MAX_CLIPBOARD_PASTE_BYTES + 10);
    assert_eq!(cap_paste(&big, Some(DEFAULT_MAX_CLIPBOARD_PASTE_BYTES)).len(), DEFAULT_MAX_CLIPBOARD_PASTE_BYTES);
    assert_eq!(cap_paste(&big, None).len(), big.len());

    let text = "y".repeat(5_000);
    for (budget, expected) in [(Some(100), 100), (None, text.len())] {
        let mut f = Fixture::with("", move |builder| builder.max_clipboard_paste_bytes(budget));
        let paste = format!("{}-v", f.secondary);
        f.visual.update(|_, cx| cx.write_to_clipboard(ClipboardItem::new_string(text.clone())));
        f.keys(&paste);
        assert_eq!(f.value().len(), expected, "budget {budget:?}");
    }
}

/// SDK `textfield::state::selection_range_normalizes_order`: a backward
/// selection is reported in order and marked reversed.
#[test]
fn backward_selection_is_normalized_and_reversed() {
    let mut f = Fixture::new("hello");
    f.keys("shift-left shift-left");
    let mut handler = f.handler();
    f.visual.update(|window, cx| {
        let selection = handler.selected_text_range(false, window, cx).unwrap();
        assert_eq!(selection.range, 3..5);
        assert!(selection.reversed);
    });
    let copy = format!("{}-c", f.secondary);
    f.keys(&copy);
    let copied = f.visual.update(|_, cx| cx.read_from_clipboard().and_then(|item| item.text()));
    assert_eq!(copied.as_deref(), Some("lo"));
}

/// SDK `textfield::control::composition_and_selection_use_character_ranges_and_byte_lengths`
/// and `textarea::control::composition_spans_wrapped_lines_and_disappears_when_unmarked`.
#[test]
fn composition_spans_wrapped_rows_and_disappears_when_unmarked() {
    let mut f = Fixture::area("", 4);
    let composing = "a long composition that wraps onto another row of the area";
    let mut handler = f.handler();
    f.visual.update(|window, cx| {
        handler.replace_and_mark_text_in_range(None, composing, None, window, cx);
    });
    f.draw();
    let units = composing.encode_utf16().count();
    let mut handler = f.handler();
    f.visual.update(|window, cx| assert_eq!(handler.marked_text_range(window, cx), Some(0..units)));
    let geometry = f.geometry();
    let rects = geometry.layout.selection_bounds(at(0), at(composing.len())).unwrap();
    let rows = rects
        .iter()
        .map(|rect| f32::from(rect.top()).round() as i32)
        .collect::<std::collections::BTreeSet<_>>();
    assert!(rows.len() > 1, "the composition wraps: {rects:?}");
    f.visual.update(|window, cx| handler.unmark_text(window, cx));
    f.draw();
    assert!(f.read(|field| field.marked.is_none()));
    assert_eq!(f.value(), composing, "unmarking keeps the text");
}

/// SDK `textfield::model::with_template_modifier_forwards_resolve_look`.
#[test]
fn template_modifier_forwards_look_resolution() {
    let cx = TestAppContext::single();
    cx.update(|cx| {
        xgpui_luma::init(cx).unwrap();
        let look = crate::theme::default_look();
        let resolve = |mut builder: ParleyTextFieldBuilder, cx: &mut App| {
            let template = builder.take_template(cx);
            template
                .resolve_look_with_scale(TextFieldState::default(), true, ControlSize::Md, 1.0, cx)
                .background
        };
        let plain = resolve(ParleyTextField::new("plain").look(&look), cx);
        let modified = resolve(ParleyTextField::new("modified").look(&look).with_template_modifier(|root, _| root), cx);
        assert_eq!(plain, modified);
    });
}

/// SDK `textfield::theme::default_theme_scales_typography_with_control_size`.
#[test]
fn size_scales_typography_and_height() {
    let measure = |size: TextFieldSize| {
        let mut f = Fixture::with("text", move |builder| builder.size(size));
        let field = f.field.clone();
        f.visual.update(|window, cx| {
            let look = field.read(cx).look(window);
            (look.typography.size, look.min_height)
        })
    };
    let (one, two, three) = (measure(TextFieldSize::One), measure(TextFieldSize::Two), measure(TextFieldSize::Three));
    assert!(one.0 <= two.0 && two.0 <= three.0 && one.0 < three.0, "{one:?} {two:?} {three:?}");
    assert!(one.1 < two.1 && two.1 < three.1, "{one:?} {two:?} {three:?}");
}

/// SDK `textarea::control::layout::max_vertical_scroll_uses_content_minus_viewport_height`,
/// `clamped_vertical_scroll_stays_inside_scrollable_range` and
/// `vertical_scroll_to_reveal_cursor_scrolls_{down,up}_to_caret_line`.
#[test]
fn vertical_scroll_is_clamped_and_reveals_the_caret_both_ways() {
    let text = (0..20).map(|i| format!("line {i}")).collect::<Vec<_>>().join("\n");
    let mut f = Fixture::area(&text, 3);
    let top = format!("{}-up", f.secondary);
    let bottom = format!("{}-down", f.secondary);
    f.keys(&top);
    assert_eq!(f.read(|field| field.scroll()), px(0.0), "revealed upwards");
    let geometry = f.geometry();
    let max = geometry.layout.size().height - geometry.viewport.size.height;
    assert!(max > px(0.0));
    f.keys(&bottom);
    assert_eq!(f.read(|field| field.scroll()), max, "revealed downwards, at the end of the range");
    let wheel = |f: &mut Fixture, dy: f32| {
        let position = f.geometry().viewport.center();
        f.visual.simulate_event(ScrollEvent {
            position,
            delta: ScrollDelta::Pixels(point(px(0.0), px(dy))),
            modifiers: Modifiers::default(),
            ..Default::default()
        });
        f.draw();
        f.read(|field| field.scroll())
    };
    assert_eq!(wheel(&mut f, -100_000.0), max, "never past the content");
    assert_eq!(wheel(&mut f, 100_000.0), px(0.0), "never before the start");
}

/// SDK `textarea::control::perf_tests::measure_textarea_hot_paths`, with the
/// same text and workloads, for comparing the Parley text area with the SDK's.
/// Manual: `cargo test -p xgpui-lab --features test-support --bin xgpui-lab
/// measure_parley_text_area -- --ignored --nocapture`.
#[test]
#[ignore = "manual headless performance measurement"]
fn measure_parley_text_area_hot_paths() {
    use std::time::Instant;
    for count in [100, 10_000] {
        let value = "office é中🙂 e\u{301} العربية\n".repeat(count);
        let mut f = Fixture::area(&value, 5);
        let field = f.field.clone();
        for case in ["warm", "edit", "resize", "scroll", "selection", "IME"] {
            let mut times = Vec::new();
            for step in 0..5 {
                let started = Instant::now();
                f.visual.update(|_, cx| {
                    field.update(cx, |field, cx| {
                        match case {
                            "edit" => {
                                let end = field.value.len();
                                if step % 2 == 0 {
                                    field.replace(end..end, "z", None, false, cx);
                                } else {
                                    field.replace(end - 1..end, "", None, false, cx);
                                }
                            }
                            "resize" => {
                                let padding = if step % 2 == 0 { 5.0 } else { 25.0 };
                                field.look_override = Some(Arc::new(move |mut look| {
                                    look.padding_x = padding;
                                    look
                                }));
                            }
                            "scroll" => field.scroll = px(if step % 2 == 0 { 0.0 } else { 200.0 }),
                            "selection" => {
                                field.anchor = at(0);
                                field.caret = at(3 + step % 3);
                            }
                            "IME" => field.marked = Some(0..3 + step % 3),
                            _ => {}
                        }
                        cx.notify();
                    })
                });
                f.draw();
                times.push(started.elapsed().as_secs_f64() * 1e3);
            }
            times.sort_by(f64::total_cmp);
            println!("ParleyTextArea/headless {case}/{count} lines: median {:.2} ms", times[times.len() / 2]);
        }
    }
}

/// Two fields in one view, for focus movement.
struct Pair {
    first: Entity<ParleyTextField>,
    second: Entity<ParleyTextField>,
}

impl Render for Pair {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(240.0)).child(self.first.clone()).child(self.second.clone())
    }
}

/// SDK `editing` focus navigation (`FocusNavigation::{Next, Prev}` on Tab).
#[test]
fn tab_and_shift_tab_move_focus_between_fields() {
    let backend = crate::parley_lab::Engine::new().unwrap().backend.clone();
    let mut cx = TestAppContext::build_with_text_system(TestDispatcher::new(0), None, backend);
    cx.update(|cx| xgpui_luma::init(cx).unwrap());
    let window = cx.open_window(size(px(240.0), px(120.0)), |_, cx| {
        let field =
            |id: &str, cx: &mut Context<Pair>| ParleyTextField::new(id.to_owned()).value(id.to_owned()).spawn(cx);
        Pair { first: field("first", cx), second: field("second", cx) }
    });
    let (first, second) = window.update(&mut cx, |pair, _, _| (pair.first.clone(), pair.second.clone())).unwrap();
    let mut visual = gpui::VisualTestContext::from_window(window.into(), &cx);
    visual.update(|window, cx| window.focus(&first.focus_handle(cx), cx));
    visual.run_until_parked();
    visual.simulate_keystrokes("tab");
    visual.run_until_parked();
    let focused = |visual: &mut gpui::VisualTestContext, field: &Entity<ParleyTextField>| {
        visual.update(|window, cx| field.focus_handle(cx).is_focused(window))
    };
    assert!(focused(&mut visual, &second), "Tab moves to the next field");
    visual.simulate_keystrokes("shift-tab");
    visual.run_until_parked();
    assert!(focused(&mut visual, &first), "Shift-Tab moves back");
    let values = visual.update(|_, cx| (first.read(cx).value(), second.read(cx).value()));
    assert_eq!(values, ("first".to_owned(), "second".to_owned()), "no tab characters inserted");
}

/// SDK `editing::multiline_policy_inserts_newline`'s `allow_tab_character`
/// policy, exposed here as a builder option.
#[test]
fn allow_tab_character_types_tabs_instead_of_moving_focus() {
    let mut f = Fixture::with("ab", |builder| builder.rows(3).allow_tab_character(true));
    f.keys("tab");
    assert_eq!(f.value(), "ab\t");
    assert!(f.read(|field| field.is_focused()), "focus stays in the field");
    let paste = format!("{}-v", f.secondary);
    f.visual.update(|_, cx| cx.write_to_clipboard(ClipboardItem::new_string("1\t2".into())));
    f.keys(&paste);
    assert_eq!(f.value(), "ab\t1\t2", "pasted tabs are kept too");
}
