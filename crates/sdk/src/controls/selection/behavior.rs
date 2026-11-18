//! Shared filter/highlight/submit state for ComboBox, SearchSelector, and Autocomplete.

use gpui::SharedString;
use lucide_svg_static::Icon as LucideIcon;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectionItem {
    pub id: SharedString,
    pub label: SharedString,
    pub enabled: bool,
    pub trailing_icon: Option<LucideIcon>,
}

impl SelectionItem {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self { id: id.into(), label: label.into(), enabled: true, trailing_icon: None }
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn trailing_icon(mut self, icon: LucideIcon) -> Self {
        self.trailing_icon = Some(icon);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionStatus {
    Idle,
    Searching,
    ErrorNoMatch,
    Complete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SelectionEvent {
    Focus,
    Blur,
    Escape,
    Submit,
    MoveNext,
    MovePrevious,
    Clear,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectionState {
    pub query: SharedString,
    pub filtered: Vec<usize>,
    pub highlighted_filtered: Option<usize>,
    pub selected_item: Option<usize>,
    pub open: bool,
    pub focused: bool,
    pub status: SelectionStatus,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            query: SharedString::default(),
            filtered: Vec::new(),
            highlighted_filtered: None,
            selected_item: None,
            open: false,
            focused: false,
            status: SelectionStatus::Idle,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubmitResult {
    None,
    Select { index: usize, exact_complete: bool },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectionBehavior {
    pub state: SelectionState,
}

impl Default for SelectionBehavior {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectionBehavior {
    pub fn new() -> Self {
        Self { state: SelectionState::default() }
    }

    pub fn set_query(&mut self, query: impl Into<SharedString>, items: &[SelectionItem]) {
        self.state.query = query.into();
        self.recompute(items);
    }

    pub fn apply(&mut self, event: SelectionEvent, items: &[SelectionItem]) -> SubmitResult {
        match event {
            SelectionEvent::Focus => {
                self.state.focused = true;
                self.recompute(items);
                SubmitResult::None
            }
            SelectionEvent::Blur => {
                self.state.focused = false;
                self.state.open = false;
                self.state.highlighted_filtered = None;
                SubmitResult::None
            }
            SelectionEvent::Escape => {
                self.reset_query_state_preserving_focus();
                SubmitResult::None
            }
            SelectionEvent::Submit => self.submit(items),
            SelectionEvent::MoveNext => {
                self.move_highlight(items, true);
                SubmitResult::None
            }
            SelectionEvent::MovePrevious => {
                self.move_highlight(items, false);
                SubmitResult::None
            }
            SelectionEvent::Clear => {
                self.reset_query_state_preserving_focus();
                SubmitResult::None
            }
        }
    }

    pub fn select_index(&mut self, index: usize) {
        self.state.selected_item = Some(index);
        self.state.status = SelectionStatus::Complete;
        self.state.open = false;
        self.state.highlighted_filtered = None;
    }

    fn submit(&mut self, items: &[SelectionItem]) -> SubmitResult {
        let query = self.state.query.as_ref().trim().to_lowercase();

        if let Some(index) = self
            .state
            .highlighted_filtered
            .and_then(|highlighted| self.state.filtered.get(highlighted).copied())
            .filter(|index| items[*index].enabled)
        {
            return SubmitResult::Select { index, exact_complete: false };
        }

        if query.is_empty() {
            return SubmitResult::None;
        }

        if let Some(index) =
            self.state.filtered.iter().copied().find(|index| {
                items[*index].enabled && items[*index].label.to_string().eq_ignore_ascii_case(query.as_str())
            })
        {
            return SubmitResult::Select { index, exact_complete: true };
        }

        if let Some(first) = self.state.filtered.iter().copied().find(|index| items[*index].enabled) {
            return SubmitResult::Select { index: first, exact_complete: false };
        }

        self.state.status = SelectionStatus::ErrorNoMatch;
        self.state.selected_item = None;
        self.state.open = false;
        self.state.highlighted_filtered = None;
        SubmitResult::None
    }

    fn move_highlight(&mut self, items: &[SelectionItem], next: bool) {
        if self.state.filtered.is_empty() {
            return;
        }

        let enabled_positions = self
            .state
            .filtered
            .iter()
            .enumerate()
            .filter_map(|(position, index)| items[*index].enabled.then_some(position))
            .collect::<Vec<_>>();
        if enabled_positions.is_empty() {
            self.state.highlighted_filtered = None;
            self.state.open = true;
            return;
        }

        let current_position = self
            .state
            .highlighted_filtered
            .and_then(|highlighted| enabled_positions.iter().position(|position| *position == highlighted));

        let next_position = if next {
            current_position.map_or(0, |position| (position + 1) % enabled_positions.len())
        } else {
            current_position.map_or(enabled_positions.len() - 1, |position| {
                if position == 0 {
                    enabled_positions.len() - 1
                } else {
                    position - 1
                }
            })
        };

        self.state.highlighted_filtered = Some(enabled_positions[next_position]);
        self.state.open = true;
    }

    fn reset_query_state_preserving_focus(&mut self) {
        self.state.query = SharedString::default();
        self.state.filtered.clear();
        self.state.highlighted_filtered = None;
        self.state.selected_item = None;
        self.state.open = false;
        self.state.status = SelectionStatus::Idle;
    }

    fn recompute(&mut self, items: &[SelectionItem]) {
        let query = self.state.query.as_ref().trim().to_lowercase();
        self.state.filtered.clear();

        if query.is_empty() {
            self.state.status = SelectionStatus::Idle;
            self.state.selected_item = None;
            self.state.highlighted_filtered = None;
            self.state.open = false;
            return;
        }

        let mut starts_with = Vec::new();
        let mut contains = Vec::new();

        for (index, item) in items.iter().enumerate() {
            let candidate = item.label.to_string().to_lowercase();
            if candidate.starts_with(&query) {
                starts_with.push(index);
            } else if candidate.contains(&query) {
                contains.push(index);
            }
        }

        self.state.filtered.extend(starts_with);
        self.state.filtered.extend(contains);

        if self.state.filtered.is_empty() {
            self.state.status = SelectionStatus::ErrorNoMatch;
            self.state.selected_item = None;
            self.state.highlighted_filtered = None;
            self.state.open = false;
            return;
        }

        let exact_index = self.state.filtered.iter().copied().find(|index| {
            items[*index].enabled && items[*index].label.to_string().eq_ignore_ascii_case(query.as_str())
        });

        self.state.selected_item = exact_index;
        self.state.status = if exact_index.is_some() {
            SelectionStatus::Complete
        } else {
            SelectionStatus::Searching
        };
        self.state.highlighted_filtered = self
            .state
            .filtered
            .iter()
            .enumerate()
            .find_map(|(position, index)| items[*index].enabled.then_some(position));
        self.state.open = self.state.focused;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_items() -> Vec<SelectionItem> {
        vec![
            SelectionItem::new("ga", "Georgia"),
            SelectionItem::new("ha", "Hawaii"),
            SelectionItem::new("id", "Idaho"),
            SelectionItem::new("in", "Indiana"),
        ]
    }

    #[test]
    fn escape_after_partial_match_clears_session() {
        let items = sample_items();
        let mut behavior = SelectionBehavior::new();

        behavior.apply(SelectionEvent::Focus, &items);
        behavior.set_query("da", &items);
        assert_eq!(behavior.state.status, SelectionStatus::Searching);
        assert_eq!(behavior.state.filtered.len(), 1);

        behavior.apply(SelectionEvent::Escape, &items);
        assert!(!behavior.state.open);
        assert_eq!(behavior.state.query.as_ref(), "");
        assert_eq!(behavior.state.selected_item, None);
        assert_eq!(behavior.state.status, SelectionStatus::Idle);
    }

    #[test]
    fn query_change_clears_stale_complete_selection() {
        let items = sample_items();
        let mut behavior = SelectionBehavior::new();

        behavior.apply(SelectionEvent::Focus, &items);
        behavior.set_query("h", &items);
        assert_eq!(behavior.state.status, SelectionStatus::Searching);

        behavior.set_query("hawaii", &items);
        assert_eq!(behavior.state.status, SelectionStatus::Complete);
        assert!(behavior.state.selected_item.is_some());

        behavior.set_query("ha", &items);
        assert_eq!(behavior.state.status, SelectionStatus::Searching);
        assert_eq!(behavior.state.selected_item, None);
    }

    #[test]
    fn arrow_then_submit_selects_highlighted_item() {
        let items = sample_items();
        let mut behavior = SelectionBehavior::new();

        behavior.apply(SelectionEvent::Focus, &items);
        behavior.set_query("i", &items);
        behavior.apply(SelectionEvent::MoveNext, &items);

        let result = behavior.apply(SelectionEvent::Submit, &items);
        match result {
            SubmitResult::Select { index, exact_complete } => {
                assert_eq!(items[index].label.as_ref(), "Indiana");
                assert!(!exact_complete);
            }
            SubmitResult::None => panic!("expected selection result"),
        }
    }

    #[test]
    fn submit_selects_highlighted_item_even_when_query_is_empty() {
        let items = sample_items();
        let mut behavior = SelectionBehavior::new();

        behavior.apply(SelectionEvent::Focus, &items);
        behavior.state.filtered = (0..items.len()).collect();
        behavior.state.highlighted_filtered = Some(1);
        behavior.state.open = true;

        let result = behavior.apply(SelectionEvent::Submit, &items);
        match result {
            SubmitResult::Select { index, exact_complete } => {
                assert_eq!(items[index].label.as_ref(), "Hawaii");
                assert!(!exact_complete);
            }
            SubmitResult::None => panic!("expected selection result"),
        }
    }

    #[test]
    fn disabled_items_remain_visible_but_are_skipped_for_highlight_and_submit() {
        let items = vec![
            SelectionItem::new("ga", "Georgia").enabled(false),
            SelectionItem::new("ha", "Hawaii"),
            SelectionItem::new("id", "Idaho"),
        ];
        let mut behavior = SelectionBehavior::new();

        behavior.apply(SelectionEvent::Focus, &items);
        behavior.set_query("i", &items);

        assert_eq!(behavior.state.filtered, vec![2, 0, 1]);
        assert_eq!(behavior.state.highlighted_filtered, Some(0));

        let result = behavior.apply(SelectionEvent::Submit, &items);
        match result {
            SubmitResult::Select { index, .. } => assert_eq!(items[index].label.as_ref(), "Idaho"),
            SubmitResult::None => panic!("expected selection result"),
        }
    }
}
