//! Visual states and per-state value tables.
//!
//! A [`States`] table holds a base value plus rules keyed by a [`StateSet`].
//! A rule matches when all of its states are active. Among matching rules the
//! one with the highest score wins, where a set's score is its bit value and
//! each state's bit follows the precedence
//! `Disabled > Pressed > Hovered > Checked > Selected > Focused > Invalid`.
//!
//! Because each state outweighs every lower state combined, a rule that
//! contains a higher state always beats a rule without it, and a rule that adds
//! states to another rule always beats it. The result never depends on the
//! order rules were written in, so editors can add rules freely.

use std::ops::Add;

/// One visual state. The discriminant is the state's precedence weight.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u16)]
pub enum State {
    Invalid = 1 << 0,
    Focused = 1 << 1,
    Selected = 1 << 2,
    Checked = 1 << 3,
    Hovered = 1 << 4,
    Pressed = 1 << 5,
    Disabled = 1 << 6,
}

impl State {
    /// Every state, lowest precedence first.
    pub const ALL: [State; 7] = [
        State::Invalid,
        State::Focused,
        State::Selected,
        State::Checked,
        State::Hovered,
        State::Pressed,
        State::Disabled,
    ];

    /// Lowercase name used in descriptors and tools.
    pub const fn name(self) -> &'static str {
        match self {
            State::Invalid => "invalid",
            State::Focused => "focused",
            State::Selected => "selected",
            State::Checked => "checked",
            State::Hovered => "hovered",
            State::Pressed => "pressed",
            State::Disabled => "disabled",
        }
    }
}

/// A set of active states, or a rule's state selector.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct StateSet(u16);

impl StateSet {
    /// No states: the base value.
    pub const NONE: StateSet = StateSet(0);

    pub const fn with(self, state: State) -> Self {
        Self(self.0 | state as u16)
    }

    pub const fn without(self, state: State) -> Self {
        Self(self.0 & !(state as u16))
    }

    /// Every state in either set.
    pub const fn union(self, other: StateSet) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn contains(self, state: State) -> bool {
        self.0 & state as u16 != 0
    }

    /// Whether every state in `self` is active in `active`.
    pub const fn matches(self, active: StateSet) -> bool {
        self.0 & !active.0 == 0
    }

    /// The rule score: higher wins among matching rules.
    pub const fn score(self) -> u16 {
        self.0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The states in this set, lowest precedence first.
    pub fn iter(self) -> impl Iterator<Item = State> {
        State::ALL.into_iter().filter(move |state| self.contains(*state))
    }
}

impl From<State> for StateSet {
    fn from(state: State) -> Self {
        StateSet::NONE.with(state)
    }
}

impl Add for State {
    type Output = StateSet;
    fn add(self, rhs: State) -> StateSet {
        StateSet::from(self).with(rhs)
    }
}

impl Add<State> for StateSet {
    type Output = StateSet;
    fn add(self, rhs: State) -> StateSet {
        self.with(rhs)
    }
}

impl std::fmt::Display for StateSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return f.write_str("default");
        }
        let mut first = true;
        for state in self.iter().collect::<Vec<_>>().into_iter().rev() {
            if !first {
                f.write_str(" + ")?;
            }
            f.write_str(state.name())?;
            first = false;
        }
        Ok(())
    }
}

/// A base value plus per-state overrides. See the module docs for how a rule
/// is chosen.
#[derive(Clone, Debug, PartialEq)]
pub struct States<T> {
    base: T,
    rules: Vec<(StateSet, T)>,
}

impl<T> States<T> {
    /// A table with only a base value.
    pub fn base(base: T) -> Self {
        Self { base, rules: Vec::new() }
    }

    /// Set the value for `at`, replacing an existing rule for the same set.
    /// An empty set replaces the base value.
    pub fn set(&mut self, at: StateSet, value: T) {
        if at.is_empty() {
            self.base = value;
        } else if let Some(rule) = self.rules.iter_mut().find(|(key, _)| *key == at) {
            rule.1 = value;
        } else {
            self.rules.push((at, value));
        }
    }

    /// Remove the rule for `at`. Returns whether a rule was removed.
    pub fn clear(&mut self, at: StateSet) -> bool {
        let before = self.rules.len();
        self.rules.retain(|(key, _)| *key != at);
        before != self.rules.len()
    }

    /// The value for the active states.
    pub fn select(&self, active: StateSet) -> &T {
        self.rules
            .iter()
            .filter(|(key, _)| key.matches(active))
            .max_by_key(|(key, _)| key.score())
            .map_or(&self.base, |(_, value)| value)
    }

    /// The exact rule for `at` (the base for an empty set), without matching.
    pub fn rule(&self, at: StateSet) -> Option<&T> {
        if at.is_empty() {
            Some(&self.base)
        } else {
            self.rules.iter().find(|(key, _)| *key == at).map(|(_, value)| value)
        }
    }

    /// The selectors that have rules, not including the base.
    pub fn selectors(&self) -> impl Iterator<Item = StateSet> + '_ {
        self.rules.iter().map(|(key, _)| *key)
    }
}

impl<T: Default> Default for States<T> {
    fn default() -> Self {
        Self::base(T::default())
    }
}

#[cfg(test)]
mod tests {
    use super::State::*;
    use super::*;

    /// The SDK checkbox indicator background, as `DefaultCheckboxTheme::resolve`
    /// computes it today, written as a state table.
    fn sdk_indicator_background() -> States<&'static str> {
        let mut table = States::base("input");
        table.set(Hovered.into(), "hover");
        table.set(Pressed.into(), "pressed");
        table.set(Checked.into(), "selected");
        table.set(Checked + Hovered, "selected");
        table.set(Disabled.into(), "disabled");
        table
    }

    /// Today's checkbox `match (checked, layer)`, for comparison.
    fn current_checkbox_background(active: StateSet) -> &'static str {
        let checked = active.contains(Checked);
        if active.contains(Disabled) {
            "disabled"
        } else if active.contains(Pressed) {
            "pressed"
        } else if checked {
            "selected"
        } else if active.contains(Hovered) {
            "hover"
        } else {
            "input"
        }
    }

    fn every_combination() -> impl Iterator<Item = StateSet> {
        (0u16..1 << State::ALL.len()).map(|bits| {
            State::ALL
                .iter()
                .enumerate()
                .filter(|(index, _)| bits & (1 << index) != 0)
                .fold(StateSet::NONE, |set, (_, state)| set.with(*state))
        })
    }

    #[test]
    fn checkbox_table_matches_the_current_match_for_every_state_combination() {
        let table = sdk_indicator_background();
        for active in every_combination() {
            assert_eq!(*table.select(active), current_checkbox_background(active), "states: {active}");
        }
    }

    #[test]
    fn precedence_table() {
        let mut table = States::base("base");
        table.set(Hovered.into(), "hovered");
        table.set(Pressed.into(), "pressed");
        table.set(Checked.into(), "checked");
        table.set(Checked + Hovered, "checked+hovered");
        table.set(Checked + Pressed, "checked+pressed");
        table.set(Disabled.into(), "disabled");
        table.set(Disabled + Checked, "disabled+checked");

        let cases: &[(StateSet, &str)] = &[
            (StateSet::NONE, "base"),
            (Focused.into(), "base"),
            (Hovered.into(), "hovered"),
            (Hovered + Pressed, "pressed"),
            (Checked.into(), "checked"),
            (Checked + Hovered, "checked+hovered"),
            (Checked + Pressed, "checked+pressed"),
            (Checked + Hovered + Pressed, "checked+pressed"),
            (Disabled.into(), "disabled"),
            (Disabled + Hovered + Pressed, "disabled"),
            (Disabled + Checked, "disabled+checked"),
            (Disabled + Checked + Pressed, "disabled+checked"),
        ];
        for (active, expected) in cases {
            assert_eq!(table.select(*active), expected, "states: {active}");
        }
    }

    #[test]
    fn a_higher_state_beats_any_combination_of_lower_states() {
        let mut table = States::base("base");
        table.set(Checked + Hovered + Focused + Invalid, "many lower");
        table.set(Pressed.into(), "pressed");
        assert_eq!(*table.select(Checked + Hovered + Focused + Invalid + Pressed), "pressed");
    }

    #[test]
    fn without_a_combined_rule_the_higher_single_state_wins() {
        let mut table = States::base("base");
        table.set(Checked.into(), "checked");
        table.set(Hovered.into(), "hovered");
        assert_eq!(*table.select(Checked + Hovered), "hovered");
    }

    #[test]
    fn rule_order_does_not_matter() {
        let mut forward = States::base(0);
        forward.set(Checked.into(), 1);
        forward.set(Pressed.into(), 2);
        let mut reverse = States::base(0);
        reverse.set(Pressed.into(), 2);
        reverse.set(Checked.into(), 1);
        for active in every_combination() {
            assert_eq!(forward.select(active), reverse.select(active), "states: {active}");
        }
    }

    #[test]
    fn set_replaces_and_clear_removes() {
        let mut table = States::base(0);
        table.set(Hovered.into(), 1);
        table.set(Hovered.into(), 2);
        assert_eq!(table.selectors().count(), 1);
        assert_eq!(*table.select(Hovered.into()), 2);
        table.set(StateSet::NONE, 9);
        assert_eq!(*table.select(StateSet::NONE), 9);
        assert!(table.clear(Hovered.into()));
        assert_eq!(*table.select(Hovered.into()), 9);
    }

    #[test]
    fn display_lists_highest_precedence_first() {
        assert_eq!((Checked + Disabled).to_string(), "disabled + checked");
        assert_eq!(StateSet::NONE.to_string(), "default");
    }
}
