//! Global configuration: the only styling facts declared in Rust.

use crate::diag::{Code, Diagnostic};
use crate::handles::{AttrId, MAX_ATTRIBUTES, StateId, StateSet};

/// How lengths snap to device pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SnapPolicy {
    /// Lengths are used as computed.
    #[default]
    None,
    /// Every length property snaps to device pixels unless its value is wrapped in `exact(…)`.
    Lengths,
}

/// Layers, states, the exclusive state group, attributes and snapping.
#[derive(Clone, Debug)]
pub struct Config {
    pub(crate) layers: Vec<String>,
    pub(crate) states: Vec<String>,
    /// Indices into `states`, strongest first.
    pub(crate) exclusive: Vec<usize>,
    pub(crate) attributes: Vec<String>,
    pub(crate) snap: SnapPolicy,
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// Start a config that adds a look's own attributes and states to this one (e.g. Radix `tone`,
    /// `radius`, `high_contrast`). Existing attributes and states keep their positions, so handles
    /// from this config stay valid in the extended one.
    pub fn extend(&self) -> ConfigExtension {
        ConfigExtension { base: self.clone(), attributes: Vec::new(), states: Vec::new() }
    }

    /// Attribute names, in handle order.
    pub fn attribute_names(&self) -> impl Iterator<Item = &str> {
        self.attributes.iter().map(String::as_str)
    }

    /// State names, in handle order.
    pub fn state_names(&self) -> impl Iterator<Item = &str> {
        self.states.iter().map(String::as_str)
    }

    /// A state handle, e.g. `config.state("checked")`.
    pub fn state(&self, name: &str) -> Option<StateId> {
        self.state_index(name).map(|i| StateId(i as u8))
    }

    /// An attribute handle, e.g. `config.attribute("variant")`.
    pub fn attribute(&self, name: &str) -> Option<AttrId> {
        self.attribute_index(name).map(|i| AttrId(i as u8))
    }

    pub fn state_name(&self, state: StateId) -> &str {
        &self.states[state.0 as usize]
    }

    pub fn attribute_name(&self, attr: AttrId) -> &str {
        &self.attributes[attr.0 as usize]
    }

    pub fn layers(&self) -> impl Iterator<Item = &str> {
        self.layers.iter().map(String::as_str)
    }

    /// States from names; unknown names are skipped (use [`Config::state`] to check).
    pub fn state_set(&self, names: &[&str]) -> StateSet {
        names.iter().filter_map(|n| self.state(n)).collect()
    }

    /// The exclusive state group, strongest first. When several of its states are active, only the
    /// strongest counts (e.g. `disabled` over `pressed` over `hover`), so tools can treat the group as
    /// one choice rather than independent states.
    pub fn exclusive_states(&self) -> impl Iterator<Item = &str> {
        self.exclusive.iter().map(|&i| self.states[i].as_str())
    }

    pub(crate) fn layer_index(&self, name: &str) -> Option<usize> {
        self.layers.iter().position(|l| l == name)
    }

    pub(crate) fn state_index(&self, name: &str) -> Option<usize> {
        self.states.iter().position(|s| s == name)
    }

    pub(crate) fn attribute_index(&self, name: &str) -> Option<usize> {
        self.attributes.iter().position(|a| a == name)
    }

    pub(crate) fn exclusive_mask(&self) -> u32 {
        self.exclusive.iter().fold(0, |mask, &i| mask | (1 << i))
    }

    /// Keep only the strongest active member of the exclusive group.
    pub(crate) fn normalize(&self, states: u32) -> u32 {
        match self.exclusive.iter().find(|&&i| states & (1 << i) != 0) {
            Some(&strongest) => (states & !self.exclusive_mask()) | (1 << strongest),
            None => states,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ConfigBuilder {
    layers: Vec<String>,
    states: Vec<String>,
    exclusive: Vec<String>,
    attributes: Vec<String>,
    snap: SnapPolicy,
}

impl ConfigBuilder {
    /// Layers, lowest first. The lowest layer's top-level constants are global.
    pub fn layers(mut self, ordered: &[&str]) -> Self {
        self.layers = ordered.iter().map(|s| s.to_string()).collect();
        self
    }

    /// State names (at most 32).
    pub fn states(mut self, states: &[&str]) -> Self {
        self.states = states.iter().map(|s| s.to_string()).collect();
        self
    }

    /// States of which only the strongest active one counts, strongest first.
    pub fn exclusive(mut self, strongest_first: &[&str]) -> Self {
        self.exclusive = strongest_first.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Attribute names usable as scopes (`variant ghost { … }`) and on keys (at most 8).
    pub fn attributes(mut self, names: &[&str]) -> Self {
        self.attributes = names.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn snap(mut self, policy: SnapPolicy) -> Self {
        self.snap = policy;
        self
    }

    pub fn build(self) -> Result<Config, Diagnostic> {
        let fail = |message: String| Err(Diagnostic::error(Code::Config, message));
        if self.layers.is_empty() {
            return fail("config needs at least one layer".into());
        }
        if self.states.len() > 32 {
            return fail("config allows at most 32 states".into());
        }
        if self.attributes.len() > MAX_ATTRIBUTES {
            return fail(format!("config allows at most {MAX_ATTRIBUTES} attributes"));
        }
        let mut exclusive = Vec::new();
        for name in &self.exclusive {
            match self.states.iter().position(|s| s == name) {
                Some(i) => exclusive.push(i),
                None => return fail(format!("exclusive state \"{name}\" is not a declared state")),
            }
        }
        Ok(
            Config {
                layers: self.layers,
                states: self.states,
                exclusive,
                attributes: self.attributes,
                snap: self.snap,
            },
        )
    }
}

/// Adds a look's own attributes and states to a base config; see [`Config::extend`].
#[derive(Clone, Debug)]
pub struct ConfigExtension {
    base: Config,
    attributes: Vec<String>,
    states: Vec<String>,
}

impl ConfigExtension {
    /// Attributes to add after the base config's.
    pub fn attributes(mut self, names: &[&str]) -> Self {
        self.attributes.extend(names.iter().map(|s| s.to_string()));
        self
    }

    /// States to add after the base config's.
    pub fn states(mut self, names: &[&str]) -> Self {
        self.states.extend(names.iter().map(|s| s.to_string()));
        self
    }

    pub fn build(self) -> Result<Config, Diagnostic> {
        let mut config = self.base;
        for name in self.attributes {
            if config.attributes.contains(&name) {
                return Err(Diagnostic::error(Code::Config, format!("attribute \"{name}\" is already declared")));
            }
            config.attributes.push(name);
        }
        for name in self.states {
            if config.states.contains(&name) {
                return Err(Diagnostic::error(Code::Config, format!("state \"{name}\" is already declared")));
            }
            config.states.push(name);
        }
        if config.attributes.len() > MAX_ATTRIBUTES {
            return Err(Diagnostic::error(Code::Config, format!("config allows at most {MAX_ATTRIBUTES} attributes")));
        }
        if config.states.len() > 32 {
            return Err(Diagnostic::error(Code::Config, "config allows at most 32 states"));
        }
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exclusive_states_are_listed_strongest_first() {
        let config = Config::builder()
            .layers(&["base"])
            .states(&["hover", "pressed", "focused", "disabled"])
            .exclusive(&["disabled", "pressed", "hover"])
            .build()
            .unwrap();
        assert_eq!(config.exclusive_states().collect::<Vec<_>>(), ["disabled", "pressed", "hover"]);
        let none = Config::builder().layers(&["base"]).states(&["hover"]).build().unwrap();
        assert_eq!(none.exclusive_states().count(), 0);
    }
}
