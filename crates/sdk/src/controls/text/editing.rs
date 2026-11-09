use gpui::{KeyDownEvent, Modifiers};

use super::state::{EditableTextPolicy, TextSelectionState, select_all};

/// Truncate clipboard text to `max_bytes` UTF-8 bytes (`None` keeps the full clipboard).
pub(crate) fn cap_clipboard_paste(text: &str, max_bytes: Option<usize>) -> &str {
    let Some(max_bytes) = max_bytes else {
        return text;
    };
    if text.len() <= max_bytes {
        return text;
    }

    match text.get(..max_bytes) {
        Some(exact) => exact,
        None => {
            let mut end = max_bytes;
            while end > 0 && !text.is_char_boundary(end) {
                end -= 1;
            }
            &text[..end]
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FocusNavigation {
    Next,
    Prev,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct KeyHandlingResult {
    pub value: String,
    pub changed: bool,
    pub handled: bool,
    pub submitted: bool,
    pub clipboard_write: Option<String>,
    pub focus_navigation: Option<FocusNavigation>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CharClass {
    Word,
    Whitespace,
    Other,
}

impl CharClass {
    fn from_char(ch: char) -> Self {
        if is_word_char(ch) {
            Self::Word
        } else if ch.is_whitespace() {
            Self::Whitespace
        } else {
            Self::Other
        }
    }

    fn is_connectable(self, ch: char) -> bool {
        Self::from_char(ch) == self
    }
}

pub(crate) fn typed_text_from_event(event: &KeyDownEvent) -> Option<&str> {
    let modifiers = &event.keystroke.modifiers;
    let shortcut_modifier = modifiers.control || modifiers.platform || modifiers.function;
    if shortcut_modifier {
        return None;
    }

    event
        .keystroke
        .key_char
        .as_deref()
        .filter(|text| !text.is_empty() && !text.chars().any(|ch| ch.is_control()))
}

pub(crate) fn move_left_target(chars: &[char], cursor: usize, modifiers: Modifiers) -> usize {
    #[cfg(target_os = "macos")]
    {
        if modifiers.platform {
            return 0;
        }
        if modifiers.alt {
            return prev_word_boundary(chars, cursor);
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        if modifiers.control {
            return prev_word_boundary(chars, cursor);
        }
    }

    cursor.saturating_sub(1)
}

pub(crate) fn move_right_target(chars: &[char], cursor: usize, modifiers: Modifiers) -> usize {
    #[cfg(target_os = "macos")]
    {
        if modifiers.platform {
            return chars.len();
        }
        if modifiers.alt {
            return next_word_boundary(chars, cursor);
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        if modifiers.control {
            return next_word_boundary(chars, cursor);
        }
    }

    (cursor + 1).min(chars.len())
}

pub(crate) fn delete_left_target(chars: &[char], cursor: usize, modifiers: Modifiers) -> usize {
    #[cfg(target_os = "macos")]
    {
        if modifiers.platform {
            return 0;
        }
        if modifiers.alt {
            return prev_word_boundary(chars, cursor);
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        if modifiers.control {
            return prev_word_boundary(chars, cursor);
        }
    }

    cursor.saturating_sub(1)
}

pub(crate) fn delete_right_target(chars: &[char], cursor: usize, modifiers: Modifiers) -> usize {
    #[cfg(target_os = "macos")]
    {
        if modifiers.platform {
            return chars.len();
        }
        if modifiers.alt {
            return next_word_boundary(chars, cursor);
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        if modifiers.control {
            return next_word_boundary(chars, cursor);
        }
    }

    (cursor + 1).min(chars.len())
}

pub(crate) fn word_cluster_range(chars: &[char], offset: usize) -> Option<(usize, usize)> {
    if chars.is_empty() {
        return None;
    }

    let target = offset.min(chars.len().saturating_sub(1));
    let class = CharClass::from_char(chars[target]);
    let mut start = target;
    let mut end = target + 1;

    while start > 0 && class.is_connectable(chars[start - 1]) {
        start -= 1;
    }
    while end < chars.len() && class.is_connectable(chars[end]) {
        end += 1;
    }

    Some((start, end))
}

#[cfg(test)]
pub(crate) fn line_range(chars: &[char], offset: usize) -> Option<(usize, usize)> {
    if chars.is_empty() {
        return Some((0, 0));
    }

    let mut start = offset.min(chars.len());
    while start > 0 && chars[start - 1] != '\n' {
        start -= 1;
    }

    let mut end = offset.min(chars.len());
    while end < chars.len() && chars[end] != '\n' {
        end += 1;
    }

    Some((start, end))
}

fn line_start_index(chars: &[char], offset: usize) -> usize {
    let mut start = offset.min(chars.len());
    while start > 0 && chars[start - 1] != '\n' {
        start -= 1;
    }
    start
}

fn line_end_index(chars: &[char], offset: usize) -> usize {
    let mut end = offset.min(chars.len());
    while end < chars.len() && chars[end] != '\n' {
        end += 1;
    }
    end
}

fn move_vertical_target(
    chars: &[char],
    cursor: usize,
    preferred_column: Option<usize>,
    delta: isize,
) -> (usize, usize) {
    let current_start = line_start_index(chars, cursor);
    let current_end = line_end_index(chars, cursor);
    let column = preferred_column.unwrap_or(cursor.saturating_sub(current_start));

    if delta < 0 {
        if current_start == 0 {
            return (cursor, column);
        }
        let prev_newline = current_start - 1;
        let prev_start = line_start_index(chars, prev_newline);
        let prev_end = prev_newline;
        return ((prev_start + column).min(prev_end), column);
    }

    if current_end == chars.len() {
        return (cursor, column);
    }

    let next_start = current_end + 1;
    let next_end = line_end_index(chars, next_start);
    ((next_start + column).min(next_end), column)
}

pub(crate) fn handle_key_down<S: TextSelectionState>(
    state: &mut S,
    value: &str,
    event: &KeyDownEvent,
    clipboard_text: Option<&str>,
    secondary: bool,
    policy: EditableTextPolicy,
) -> KeyHandlingResult {
    let mut chars = value.chars().collect::<Vec<_>>();
    state.clamp_cursor(chars.len());

    let mut result = KeyHandlingResult { value: value.to_string(), ..Default::default() };
    let selecting = event.keystroke.modifiers.shift;
    let modifiers = event.keystroke.modifiers;

    match event.keystroke.key.as_str() {
        "backspace" => {
            if delete_selection(state, &mut chars) {
                result.changed = true;
            } else {
                let next = delete_left_target(&chars, state.cursor(), modifiers);
                if next < state.cursor() {
                    chars.drain(next..state.cursor());
                    state.set_cursor_raw(next);
                    result.changed = true;
                }
            }
            state.set_preferred_column(None);
            result.handled = true;
        }
        "delete" => {
            if delete_selection(state, &mut chars) {
                result.changed = true;
            } else {
                let next = delete_right_target(&chars, state.cursor(), modifiers);
                if next > state.cursor() {
                    chars.drain(state.cursor()..next);
                    result.changed = true;
                }
            }
            state.set_preferred_column(None);
            result.handled = true;
        }
        "left" => {
            let next = if !selecting {
                if let Some((start, _)) = state.selection_range() {
                    start
                } else {
                    move_left_target(&chars, state.cursor(), modifiers)
                }
            } else {
                move_left_target(&chars, state.cursor(), modifiers)
            };
            state.set_cursor(next, selecting);
            state.set_preferred_column(None);
            result.handled = true;
        }
        "right" => {
            let next = if !selecting {
                if let Some((_, end)) = state.selection_range() {
                    end
                } else {
                    move_right_target(&chars, state.cursor(), modifiers)
                }
            } else {
                move_right_target(&chars, state.cursor(), modifiers)
            };
            state.set_cursor(next, selecting);
            state.set_preferred_column(None);
            result.handled = true;
        }
        "home" => {
            let next = if policy.multiline {
                line_start_index(&chars, state.cursor())
            } else {
                0
            };
            state.set_cursor(next, selecting);
            state.set_preferred_column(None);
            result.handled = true;
        }
        "end" => {
            let next = if policy.multiline {
                line_end_index(&chars, state.cursor())
            } else {
                chars.len()
            };
            state.set_cursor(next, selecting);
            state.set_preferred_column(None);
            result.handled = true;
        }
        "up" if policy.multiline => {
            let base = if !selecting {
                if let Some((start, _)) = state.selection_range() {
                    state.set_cursor_raw(start);
                    state.clear_selection();
                    start
                } else {
                    state.cursor()
                }
            } else {
                state.cursor()
            };
            let (next, column) = move_vertical_target(&chars, base, state.preferred_column(), -1);
            state.set_cursor(next, selecting);
            state.set_preferred_column(Some(column));
            result.handled = true;
        }
        "down" if policy.multiline => {
            let base = if !selecting {
                if let Some((_, end)) = state.selection_range() {
                    state.set_cursor_raw(end);
                    state.clear_selection();
                    end
                } else {
                    state.cursor()
                }
            } else {
                state.cursor()
            };
            let (next, column) = move_vertical_target(&chars, base, state.preferred_column(), 1);
            state.set_cursor(next, selecting);
            state.set_preferred_column(Some(column));
            result.handled = true;
        }
        "enter" => {
            if policy.submit_on_enter {
                result.submitted = true;
                result.handled = true;
            } else if policy.multiline {
                result.changed = insert_text(state, &mut chars, "\n");
                state.set_preferred_column(None);
                result.handled = true;
            }
        }
        "tab" => {
            if policy.allow_tab_character {
                result.changed = insert_text(state, &mut chars, "\t");
                state.set_preferred_column(None);
                result.handled = true;
            } else {
                result.focus_navigation = Some(if modifiers.shift {
                    FocusNavigation::Prev
                } else {
                    FocusNavigation::Next
                });
                result.handled = true;
            }
        }
        "escape" => {
            if policy.clear_on_escape && !chars.is_empty() {
                chars.clear();
                state.set_cursor_raw(0);
                state.clear_selection();
                result.changed = true;
            }
            state.set_preferred_column(None);
            result.handled = true;
        }
        "a" if secondary => {
            select_all(state, chars.len());
            state.set_preferred_column(None);
            result.handled = true;
        }
        "c" if secondary => {
            result.clipboard_write = selected_text(state, &chars);
            result.handled = true;
        }
        "x" if secondary => {
            result.clipboard_write = selected_text(state, &chars);
            if delete_selection(state, &mut chars) {
                result.changed = true;
            }
            result.handled = true;
        }
        "v" if secondary => {
            if let Some(text) = clipboard_text {
                let text = cap_clipboard_paste(text, policy.max_clipboard_paste_bytes);
                let text = if policy.strip_newlines_on_paste {
                    text.replace(['\n', '\r'], "")
                } else {
                    text.to_string()
                };
                if !text.is_empty() {
                    result.changed = insert_text(state, &mut chars, &text);
                }
            }
            state.set_preferred_column(None);
            result.handled = true;
        }
        _ => {
            if let Some(typed) = typed_text_from_event(event) {
                result.changed = insert_text(state, &mut chars, typed);
                state.set_preferred_column(None);
                result.handled = true;
            }
        }
    }

    if result.changed {
        result.value = chars.into_iter().collect();
    }

    result
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

fn prev_word_boundary(chars: &[char], cursor: usize) -> usize {
    if cursor == 0 {
        return 0;
    }

    let mut ix = cursor;
    while ix > 0 && chars.get(ix - 1).is_some_and(|ch| ch.is_whitespace()) {
        ix -= 1;
    }
    while ix > 0 && chars.get(ix - 1).is_some_and(|ch| is_word_char(*ch)) {
        ix -= 1;
    }
    ix
}

fn next_word_boundary(chars: &[char], cursor: usize) -> usize {
    let mut ix = cursor.min(chars.len());
    while ix < chars.len() && chars.get(ix).is_some_and(|ch| ch.is_whitespace()) {
        ix += 1;
    }
    while ix < chars.len() && chars.get(ix).is_some_and(|ch| is_word_char(*ch)) {
        ix += 1;
    }
    ix
}

fn delete_selection<S: TextSelectionState>(state: &mut S, chars: &mut Vec<char>) -> bool {
    let Some((start, end)) = state.selection_range() else {
        return false;
    };
    chars.drain(start..end);
    state.set_cursor_raw(start);
    state.clear_selection();
    true
}

fn insert_text<S: TextSelectionState>(state: &mut S, chars: &mut Vec<char>, value: &str) -> bool {
    if value.is_empty() {
        return false;
    }

    let _ = delete_selection(state, chars);
    let inserted: Vec<char> = value.chars().collect();
    let count = inserted.len();
    if count == 0 {
        return false;
    }
    let at = state.cursor();
    chars.splice(at..at, inserted);
    state.set_cursor_raw(at + count);
    state.clear_selection();
    true
}

fn selected_text<S: TextSelectionState>(state: &S, chars: &[char]) -> Option<String> {
    let (start, end) = state.selection_range()?;
    Some(chars.iter().skip(start).take(end - start).collect())
}

#[cfg(test)]
mod tests {
    use gpui::{KeyDownEvent, Keystroke, Modifiers};

    use super::*;

    #[derive(Clone, Copy, Debug, Default)]
    struct TestState {
        cursor: usize,
        selection_anchor: Option<usize>,
        preferred_column: Option<usize>,
    }

    impl TextSelectionState for TestState {
        fn cursor(&self) -> usize {
            self.cursor
        }

        fn set_cursor_raw(&mut self, cursor: usize) {
            self.cursor = cursor;
        }

        fn selection_anchor(&self) -> Option<usize> {
            self.selection_anchor
        }

        fn set_selection_anchor_raw(&mut self, anchor: Option<usize>) {
            self.selection_anchor = anchor;
        }

        fn preferred_column(&self) -> Option<usize> {
            self.preferred_column
        }

        fn set_preferred_column(&mut self, column: Option<usize>) {
            self.preferred_column = column;
        }
    }

    #[test]
    fn typed_text_uses_key_char() {
        let event = KeyDownEvent {
            keystroke: Keystroke { modifiers: Modifiers::none(), key: "a".into(), key_char: Some("A".into()) },
            is_held: false,
            prefer_character_input: false,
        };
        assert_eq!(typed_text_from_event(&event), Some("A"));
    }

    #[test]
    fn typed_text_ignores_secondary_shortcuts() {
        let event = KeyDownEvent {
            keystroke: Keystroke { modifiers: secondary_modifiers(), key: "a".into(), key_char: Some("a".into()) },
            is_held: false,
            prefer_character_input: false,
        };
        assert_eq!(typed_text_from_event(&event), None);
    }

    #[test]
    fn typed_text_ignores_tab_control_character() {
        let event = KeyDownEvent {
            keystroke: Keystroke { modifiers: Modifiers::none(), key: "tab".into(), key_char: Some("\t".into()) },
            is_held: false,
            prefer_character_input: false,
        };
        assert_eq!(typed_text_from_event(&event), None);
    }

    #[test]
    fn movement_targets_clamp_to_bounds() {
        let chars = "hello world".chars().collect::<Vec<_>>();
        assert_eq!(move_left_target(&chars, 0, Modifiers::none()), 0);
        assert_eq!(move_right_target(&chars, chars.len(), Modifiers::none()), chars.len());
        assert_eq!(delete_left_target(&chars, 0, Modifiers::none()), 0);
        assert_eq!(delete_right_target(&chars, chars.len(), Modifiers::none()), chars.len());
    }

    #[test]
    fn double_click_word_cluster_selects_word() {
        let chars = "abc 123".chars().collect::<Vec<_>>();
        assert_eq!(word_cluster_range(&chars, 1), Some((0, 3)));
        assert_eq!(word_cluster_range(&chars, 4), Some((4, 7)));
    }

    #[test]
    fn double_click_word_cluster_selects_whitespace_run() {
        let chars = "a   b".chars().collect::<Vec<_>>();
        assert_eq!(word_cluster_range(&chars, 2), Some((1, 4)));
    }

    #[test]
    fn multiline_policy_inserts_newline() {
        let mut state = TestState { cursor: 1, ..Default::default() };
        let event = KeyDownEvent {
            keystroke: Keystroke { modifiers: Modifiers::none(), key: "enter".into(), key_char: None },
            is_held: false,
            prefer_character_input: false,
        };

        let result = handle_key_down(
            &mut state,
            "ab",
            &event,
            None,
            false,
            EditableTextPolicy {
                multiline: true,
                submit_on_enter: false,
                strip_newlines_on_paste: false,
                allow_tab_character: true,
                clear_on_escape: false,
                ..Default::default()
            },
        );

        assert!(result.changed);
        assert_eq!(result.value, "a\nb");
        assert_eq!(state.cursor, 2);
    }

    #[test]
    fn line_range_stops_at_newline_boundaries() {
        let chars = "ab\ncd\nef".chars().collect::<Vec<_>>();
        assert_eq!(line_range(&chars, 0), Some((0, 2)));
        assert_eq!(line_range(&chars, 2), Some((0, 2)));
        assert_eq!(line_range(&chars, 3), Some((3, 5)));
        assert_eq!(line_range(&chars, 6), Some((6, 8)));
    }

    #[test]
    fn multiline_up_down_preserves_column() {
        let event_down = KeyDownEvent {
            keystroke: Keystroke { modifiers: Modifiers::none(), key: "down".into(), key_char: None },
            is_held: false,
            prefer_character_input: false,
        };
        let event_up = KeyDownEvent {
            keystroke: Keystroke { modifiers: Modifiers::none(), key: "up".into(), key_char: None },
            is_held: false,
            prefer_character_input: false,
        };
        let mut state = TestState { cursor: 1, ..Default::default() };
        let policy = EditableTextPolicy {
            multiline: true,
            submit_on_enter: false,
            strip_newlines_on_paste: false,
            allow_tab_character: true,
            clear_on_escape: false,
            ..Default::default()
        };

        let result = handle_key_down(&mut state, "ab\ncdef\nxy", &event_down, None, false, policy);
        assert!(result.handled);
        assert_eq!(state.cursor, 4);
        assert_eq!(state.preferred_column, Some(1));

        let result = handle_key_down(&mut state, "ab\ncdef\nxy", &event_down, None, false, policy);
        assert!(result.handled);
        assert_eq!(state.cursor, 9);

        let result = handle_key_down(&mut state, "ab\ncdef\nxy", &event_up, None, false, policy);
        assert!(result.handled);
        assert_eq!(state.cursor, 4);
    }

    #[test]
    fn multiline_home_end_are_line_local() {
        let event_home = KeyDownEvent {
            keystroke: Keystroke { modifiers: Modifiers::none(), key: "home".into(), key_char: None },
            is_held: false,
            prefer_character_input: false,
        };
        let event_end = KeyDownEvent {
            keystroke: Keystroke { modifiers: Modifiers::none(), key: "end".into(), key_char: None },
            is_held: false,
            prefer_character_input: false,
        };
        let policy = EditableTextPolicy {
            multiline: true,
            submit_on_enter: false,
            strip_newlines_on_paste: false,
            allow_tab_character: true,
            clear_on_escape: false,
            ..Default::default()
        };
        let mut state = TestState { cursor: 4, ..Default::default() };

        let result = handle_key_down(&mut state, "ab\ncdef\nxy", &event_home, None, false, policy);
        assert!(result.handled);
        assert_eq!(state.cursor, 3);

        let result = handle_key_down(&mut state, "ab\ncdef\nxy", &event_end, None, false, policy);
        assert!(result.handled);
        assert_eq!(state.cursor, 7);
    }

    #[test]
    fn copy_cut_and_paste_follow_selection() {
        let mut state = TestState { cursor: 1, selection_anchor: Some(4), ..Default::default() };
        let policy = EditableTextPolicy {
            multiline: false,
            submit_on_enter: false,
            strip_newlines_on_paste: true,
            allow_tab_character: false,
            clear_on_escape: false,
            ..Default::default()
        };

        let copy = handle_key_down(&mut state, "abcdef", &event("c", secondary_modifiers()), None, true, policy);
        assert_eq!(copy.clipboard_write.as_deref(), Some("bcd"));
        assert!(!copy.changed);

        let cut = handle_key_down(&mut state, "abcdef", &event("x", secondary_modifiers()), None, true, policy);
        assert_eq!(cut.clipboard_write.as_deref(), Some("bcd"));
        assert!(cut.changed);
        assert_eq!(cut.value, "aef");
        assert_eq!(state.cursor, 1);
        assert_eq!(state.selection_range(), None);

        let paste = handle_key_down(&mut state, "aef", &event("v", secondary_modifiers()), Some("x\ny"), true, policy);
        assert!(paste.changed);
        assert_eq!(paste.value, "axyef");
    }

    #[test]
    fn paste_respects_configured_byte_budget() {
        let mut state = TestState { cursor: 0, ..Default::default() };
        let policy = EditableTextPolicy { max_clipboard_paste_bytes: Some(8), ..Default::default() };
        let paste =
            handle_key_down(&mut state, "", &event("v", secondary_modifiers()), Some("hello world"), true, policy);
        assert!(paste.changed);
        assert_eq!(paste.value, "hello wo");
    }

    #[test]
    fn paste_without_budget_keeps_the_full_clipboard() {
        let mut state = TestState { cursor: 0, ..Default::default() };
        let policy = EditableTextPolicy { max_clipboard_paste_bytes: None, ..Default::default() };
        let paste =
            handle_key_down(&mut state, "", &event("v", secondary_modifiers()), Some("hello world"), true, policy);
        assert!(paste.changed);
        assert_eq!(paste.value, "hello world");
    }

    #[test]
    fn cap_clipboard_paste_truncates_on_char_boundary() {
        let capped = cap_clipboard_paste("éééé", Some(3));
        assert_eq!(capped, "é");
        assert!(capped.is_char_boundary(capped.len()));
    }

    #[test]
    fn cap_clipboard_paste_none_is_unlimited() {
        assert_eq!(cap_clipboard_paste("hello world", None), "hello world");
    }

    fn event(key: &str, modifiers: Modifiers) -> KeyDownEvent {
        KeyDownEvent {
            keystroke: Keystroke { modifiers, key: key.into(), key_char: None },
            is_held: false,
            prefer_character_input: false,
        }
    }

    #[cfg(target_os = "macos")]
    fn secondary_modifiers() -> Modifiers {
        Modifiers { platform: true, ..Modifiers::default() }
    }

    #[cfg(not(target_os = "macos"))]
    fn secondary_modifiers() -> Modifiers {
        Modifiers { control: true, ..Modifiers::default() }
    }
}
