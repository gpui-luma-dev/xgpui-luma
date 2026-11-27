use gpui::{
    App, ClickEvent, Context, Div, FocusHandle, KeyDownEvent, MouseButton, Role, Stateful, Subscription, Window,
    prelude::*,
};

use super::{ListBoxInput, ListBoxItemState, ListBoxNavigation, ListBoxSelectionModifiers, SelectionMode};

/// Physical key mapping belongs to the binding, not the collection state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ListBoxAxis {
    Vertical,
    Horizontal,
}

type InputHandler<M, K> = fn(&mut M, ListBoxInput<K>, &mut Window, &mut Context<M>);

/// Focus and input wiring for a host-owned list surface. Keep one binding per
/// list for its lifetime. Nested interactive children must stop consumed pointer
/// events; keyboard events are handled only while the list itself holds focus.
pub struct ListBoxBinding {
    focus: FocusHandle,
    pointer_focus: crate::interaction::PointerFocusPolicy,
    subscriptions: Vec<Subscription>,
}

impl ListBoxBinding {
    /// Create a stable focus/input binding. Selection policy remains in the model.
    pub fn new(cx: &mut App) -> Self {
        Self {
            focus: cx.focus_handle().tab_stop(true),
            pointer_focus: Default::default(),
            subscriptions: Vec::new(),
        }
    }

    /// Configure focus acquisition on row press, independently of wheel input.
    pub fn set_pointer_focus_policy(&mut self, policy: crate::interaction::PointerFocusPolicy) {
        self.pointer_focus = policy;
    }

    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus
    }

    /// Whether the host should paint the active item's keyboard focus indicator.
    pub fn focus_visible(&self, window: &Window) -> bool {
        crate::infra::state::focus_visible(self.focus.is_focused(window), window)
    }

    /// Attach to a stable, host-owned viewport. Register focus subscriptions on
    /// first render, then reuse them for the lifetime of the owning view. Pass
    /// the current state selection mode each render so runtime changes also
    /// update keyboard shortcuts without replacing the focus handle.
    pub fn bind_root<M: 'static, K: 'static>(
        &mut self,
        root: Stateful<Div>,
        axis: ListBoxAxis,
        mode: SelectionMode,
        window: &mut Window,
        cx: &mut Context<M>,
        on_input: InputHandler<M, K>,
    ) -> Stateful<Div> {
        if self.subscriptions.is_empty() {
            // GPUI delivers focus observers after painting. Defer model/event
            // updates so their notifications request a new frame instead of
            // leaving focus styling and wheel routing stale until another input.
            self.subscriptions.push(cx.on_focus_in(&self.focus, window, move |_, window, cx| {
                cx.defer_in(window, move |owner, window, cx| {
                    on_input(owner, ListBoxInput::Focus(true), window, cx);
                });
            }));
            self.subscriptions.push(cx.on_focus_out(&self.focus, window, move |_, _, window, cx| {
                cx.defer_in(window, move |owner, window, cx| {
                    on_input(owner, ListBoxInput::Focus(false), window, cx);
                });
            }));
        }
        let focus = self.focus.clone();
        root.track_focus(&self.focus)
            .role(Role::ListBox)
            .aria_orientation(match axis {
                ListBoxAxis::Vertical => gpui::Orientation::Vertical,
                ListBoxAxis::Horizontal => gpui::Orientation::Horizontal,
            })
            .on_key_down(cx.listener(move |owner, event: &KeyDownEvent, window, cx| {
                if !focus.is_focused(window) {
                    return;
                }
                let Some(input) = key_input(mode, axis, &event.keystroke.key, event.keystroke.modifiers) else {
                    return;
                };
                on_input(owner, input, window, cx);
                cx.stop_propagation();
            }))
    }

    /// Attach to a look-styled row with a stable element ID derived from its
    /// list identity and key. Content and dimensions remain host choices.
    pub fn bind_row<M: 'static, K: Clone + 'static>(
        &self,
        row: Stateful<Div>,
        key: K,
        state: ListBoxItemState,
        cx: &mut Context<M>,
        on_input: InputHandler<M, K>,
    ) -> Stateful<Div> {
        let row = row
            .role(Role::ListBoxOption)
            .aria_selected(state.selected)
            .when(state.active, |row| row.aria_active_descendant());
        if !state.enabled {
            return row;
        }
        let focus = self.focus.clone();
        let pointer_focus = self.pointer_focus;
        row.on_mouse_down(MouseButton::Left, move |_, window, cx| {
            pointer_focus.apply(&focus, window, cx);
            cx.stop_propagation();
        })
        .on_click(cx.listener(move |owner, event: &ClickEvent, window, cx| {
            if event.is_keyboard() {
                return;
            }
            let input = if event.click_count() == 2 {
                ListBoxInput::Activate(key.clone())
            } else {
                let modifiers = selection_modifiers(event.modifiers());
                ListBoxInput::SelectWithModifiers { key: key.clone(), modifiers }
            };
            on_input(owner, input, window, cx);
            cx.stop_propagation();
        }))
    }
}

/// Translate only list-owned keys; leave other shortcuts available to ancestors.
fn key_input<K>(
    mode: SelectionMode,
    axis: ListBoxAxis,
    key: &str,
    modifiers: gpui::Modifiers,
) -> Option<ListBoxInput<K>> {
    if mode.allows_multiple()
        && key.eq_ignore_ascii_case("a")
        && (modifiers.platform || modifiers.control)
        && !modifiers.alt
        && !modifiers.function
    {
        return Some(if modifiers.shift {
            ListBoxInput::ClearSelection
        } else {
            ListBoxInput::SelectAll
        });
    }
    let selection = selection_modifiers(modifiers);
    let extended_modifiers = mode == SelectionMode::Extended && !modifiers.alt && !modifiers.function;
    if modifiers.modified() && !extended_modifiers {
        return None;
    }
    let direction = match key {
        "up" if axis == ListBoxAxis::Vertical => Some(ListBoxNavigation::Previous),
        "down" if axis == ListBoxAxis::Vertical => Some(ListBoxNavigation::Next),
        "left" if axis == ListBoxAxis::Horizontal => Some(ListBoxNavigation::Previous),
        "right" if axis == ListBoxAxis::Horizontal => Some(ListBoxNavigation::Next),
        "home" => Some(ListBoxNavigation::First),
        "end" => Some(ListBoxNavigation::Last),
        _ => None,
    };
    if let Some(direction) = direction {
        return Some(if selection.extend {
            ListBoxInput::NavigateRange { direction, additive: selection.toggle }
        } else {
            ListBoxInput::Navigate(direction)
        });
    }
    match key {
        "space" if mode != SelectionMode::None => Some(if modifiers.modified() {
            ListBoxInput::SelectActiveWithModifiers(selection)
        } else {
            ListBoxInput::SelectActive
        }),
        "enter" if !modifiers.modified() => Some(ListBoxInput::ConfirmActive),
        _ => None,
    }
}

fn selection_modifiers(modifiers: gpui::Modifiers) -> ListBoxSelectionModifiers {
    ListBoxSelectionModifiers { toggle: modifiers.platform || modifiers.control, extend: modifiers.shift }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::Modifiers;

    #[test]
    fn extended_keyboard_gestures_follow_axis_and_current_runtime_mode() {
        use super::super::ListBoxState;
        for (axis, next) in [(ListBoxAxis::Vertical, "down"), (ListBoxAxis::Horizontal, "right")] {
            let mut state = ListBoxState::try_new(1..=5, |n| *n, SelectionMode::Extended).unwrap();
            state.apply(ListBoxInput::Select(2));
            let shift = Modifiers { shift: true, ..Default::default() };
            let input = key_input(state.selection_mode(), axis, next, shift).unwrap();
            assert_eq!(state.apply(input).reveal, Some(3));
            assert_eq!(state.selected_keys().copied().collect::<Vec<_>>(), vec![2, 3]);
            for toggle in [Modifiers { platform: true, ..shift }, Modifiers { control: true, ..shift }] {
                assert!(matches!(
                    key_input::<u32>(state.selection_mode(), axis, "end", toggle),
                    Some(ListBoxInput::NavigateRange { direction: ListBoxNavigation::Last, additive: true })
                ));
            }
            let toggle = Modifiers { control: true, ..Default::default() };
            state.apply(key_input(state.selection_mode(), axis, "space", toggle).unwrap());
            assert_eq!(state.selected_keys().copied().collect::<Vec<_>>(), vec![2]);
            assert!(matches!(
                key_input::<u32>(state.selection_mode(), axis, "a", toggle),
                Some(ListBoxInput::SelectAll)
            ));
            state.set_selection_mode(SelectionMode::Multiple);
            assert!(key_input::<u32>(state.selection_mode(), axis, next, shift).is_none());
            state.set_selection_mode(SelectionMode::None);
            assert!(key_input::<u32>(state.selection_mode(), axis, "space", Modifiers::default()).is_none());
            assert!(key_input::<u32>(state.selection_mode(), axis, "a", toggle).is_none());
            assert!(matches!(
                key_input::<u32>(state.selection_mode(), axis, "enter", Modifiers::default()),
                Some(ListBoxInput::ConfirmActive)
            ));
        }
    }

    #[test]
    fn return_selects_navigated_item_before_activation_without_toggling_off() {
        use super::super::{ListBoxEvent, ListBoxSnapshot, ListBoxState, SelectionPolicy};
        for (axis, next) in [(ListBoxAxis::Vertical, "down"), (ListBoxAxis::Horizontal, "right")] {
            for mode in [
                SelectionMode::SingleRequired,
                SelectionMode::SingleAllowNone,
                SelectionMode::Multiple,
                SelectionMode::Extended,
                SelectionMode::None,
            ] {
                let snapshot = ListBoxSnapshot::try_with_enabled(1..=3, |n| *n, |n| *n != 2).unwrap();
                let mut state = ListBoxState::from_snapshot(snapshot, mode);
                state.apply(ListBoxInput::Select(1));
                state.set_selection_policy(SelectionPolicy { toggle_off: true, ..SelectionPolicy::new(mode) });
                state.apply(key_input(mode, axis, next, Modifiers::default()).unwrap());
                assert_eq!(state.active_key(), Some(&3));
                assert!(!state.visible_items().find(|item| item.key == 3).unwrap().state.selected);
                let update = state.apply(key_input(mode, axis, "enter", Modifiers::default()).unwrap());
                let selected = match mode {
                    SelectionMode::None => vec![],
                    SelectionMode::Multiple => vec![1, 3],
                    _ => vec![3],
                };
                assert_eq!(state.selected_keys().copied().collect::<Vec<_>>(), selected);
                let mut expected = Vec::new();
                if mode != SelectionMode::None {
                    expected.push(ListBoxEvent::SelectionChanged { selected: selected.clone(), active: Some(3) });
                }
                expected.push(ListBoxEvent::ItemActivated { key: 3 });
                assert_eq!(update.events, expected);
                let repeated = state.apply(key_input(mode, axis, "enter", Modifiers::default()).unwrap());
                assert!(!repeated.changed);
                assert_eq!(repeated.events, vec![ListBoxEvent::ItemActivated { key: 3 }]);
                assert_eq!(state.selected_keys().copied().collect::<Vec<_>>(), selected);
                if mode.allows_multiple() {
                    state.apply(ListBoxInput::SelectAll);
                    state.apply(ListBoxInput::ConfirmActive);
                    assert_eq!(state.selected_keys().copied().collect::<Vec<_>>(), vec![1, 3]);
                }
                state.replace_snapshot(ListBoxSnapshot::try_with_enabled([1], |n| *n, |_| false).unwrap());
                assert!(state.apply(ListBoxInput::ConfirmActive).events.is_empty());
            }
        }
    }

    #[test]
    fn modified_shortcuts_outside_selection_remain_available_to_ancestors() {
        for key in ["escape", "tab", "c", "v", "x", "enter"] {
            assert!(
                key_input::<u32>(
                    SelectionMode::Extended,
                    ListBoxAxis::Vertical,
                    key,
                    Modifiers { platform: true, ..Default::default() }
                )
                .is_none()
            );
        }
        for modifiers in
            [Modifiers { alt: true, ..Default::default() }, Modifiers { function: true, ..Default::default() }]
        {
            assert!(key_input::<u32>(SelectionMode::Extended, ListBoxAxis::Vertical, "down", modifiers).is_none());
        }
    }

    #[test]
    fn selection_shortcuts_are_scoped_to_multiple_mode() {
        for modifiers in [
            Modifiers { platform: true, ..Default::default() },
            Modifiers { control: true, ..Default::default() },
        ] {
            assert!(matches!(
                key_input::<u32>(SelectionMode::Multiple, ListBoxAxis::Vertical, "a", modifiers),
                Some(ListBoxInput::SelectAll)
            ));
            assert!(key_input::<u32>(SelectionMode::SingleAllowNone, ListBoxAxis::Vertical, "a", modifiers).is_none());
            let clear_modifiers = Modifiers { shift: true, ..modifiers };
            assert!(matches!(
                key_input::<u32>(SelectionMode::Multiple, ListBoxAxis::Vertical, "a", clear_modifiers),
                Some(ListBoxInput::ClearSelection)
            ));
            assert!(
                key_input::<u32>(SelectionMode::SingleAllowNone, ListBoxAxis::Vertical, "a", clear_modifiers).is_none()
            );
        }
        // Escape belongs to the enclosing focus scope, even in multiple mode.
        assert!(
            key_input::<u32>(SelectionMode::Multiple, ListBoxAxis::Horizontal, "escape", Modifiers::default())
                .is_none()
        );
        assert!(
            key_input::<u32>(SelectionMode::SingleRequired, ListBoxAxis::Vertical, "escape", Modifiers::default())
                .is_none()
        );
        assert!(
            key_input::<u32>(
                SelectionMode::Multiple,
                ListBoxAxis::Vertical,
                "a",
                Modifiers { control: true, alt: true, ..Default::default() }
            )
            .is_none()
        );
    }

    #[test]
    fn clear_shortcut_removes_selection_without_losing_focus_or_active_item() {
        use super::super::{ListBoxEvent, ListBoxState};

        for axis in [ListBoxAxis::Vertical, ListBoxAxis::Horizontal] {
            let mut list = ListBoxState::try_new([1, 2, 3], |item| *item, SelectionMode::Multiple).unwrap();
            list.apply(ListBoxInput::Focus(true));
            list.apply(ListBoxInput::Select(2));
            list.apply(ListBoxInput::SelectAll);
            let input = key_input(
                SelectionMode::Multiple,
                axis,
                "A",
                Modifiers { platform: true, shift: true, ..Default::default() },
            )
            .unwrap();
            let update = list.apply(input);
            assert_eq!(update.events, vec![ListBoxEvent::SelectionChanged { selected: vec![], active: Some(2) }]);
            assert_eq!(list.selected_keys().count(), 0);
            assert!(list.is_focused());
            assert_eq!(list.active_key(), Some(&2));
            assert!(!list.apply(ListBoxInput::ClearSelection).changed);
        }
    }

    #[test]
    fn navigation_uses_each_lists_axis_without_consuming_unrelated_keys() {
        assert!(matches!(
            key_input::<u32>(SelectionMode::Multiple, ListBoxAxis::Horizontal, "right", Modifiers::default()),
            Some(ListBoxInput::Navigate(ListBoxNavigation::Next))
        ));
        assert!(matches!(
            key_input::<u32>(SelectionMode::Multiple, ListBoxAxis::Vertical, "down", Modifiers::default()),
            Some(ListBoxInput::Navigate(ListBoxNavigation::Next))
        ));
        assert!(
            key_input::<u32>(SelectionMode::Multiple, ListBoxAxis::Horizontal, "down", Modifiers::default()).is_none()
        );
        assert!(
            key_input::<u32>(
                SelectionMode::Multiple,
                ListBoxAxis::Vertical,
                "down",
                Modifiers { shift: true, ..Default::default() }
            )
            .is_none()
        );
        assert!(matches!(
            key_input::<u32>(SelectionMode::Multiple, ListBoxAxis::Vertical, "space", Modifiers::default()),
            Some(ListBoxInput::SelectActive)
        ));
    }
}
