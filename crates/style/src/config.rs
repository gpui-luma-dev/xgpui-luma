//! Global configuration: the only styling facts declared in Rust.

use crate::diag::Diagnostic;

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

    pub(crate) fn layer(&self, name: &str) -> Option<usize> {
        self.layers.iter().position(|l| l == name)
    }

    pub(crate) fn state(&self, name: &str) -> Option<usize> {
        self.states.iter().position(|s| s == name)
    }

    pub(crate) fn attribute(&self, name: &str) -> Option<usize> {
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

    /// Attribute names usable as scopes (`variant ghost { … }`) and on instances.
    pub fn attributes(mut self, names: &[&str]) -> Self {
        self.attributes = names.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn snap(mut self, policy: SnapPolicy) -> Self {
        self.snap = policy;
        self
    }

    pub fn build(self) -> Result<Config, Diagnostic> {
        if self.layers.is_empty() {
            return Err(Diagnostic::error("config needs at least one layer"));
        }
        if self.states.len() > 32 {
            return Err(Diagnostic::error("config allows at most 32 states"));
        }
        let mut exclusive = Vec::new();
        for name in &self.exclusive {
            match self.states.iter().position(|s| s == name) {
                Some(i) => exclusive.push(i),
                None => return Err(Diagnostic::error(format!("exclusive state \"{name}\" is not a declared state"))),
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
