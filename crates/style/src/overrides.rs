//! Instance overrides: values set on one control instance (API topic 5).

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::handles::{PartId, Prop, intern};
use crate::value::Value;

/// A value set on one instance: a literal, or a token reference that follows mode and theme changes.
#[derive(Clone, Debug, PartialEq)]
pub enum OverrideValue {
    Value(Value),
    Token(Arc<str>),
}

/// Values set on one control instance. Each replaces its property in every state, like `:=` in a
/// layer above all others, and takes part in the cascade (so `border = background` follows it).
///
/// Build once (e.g. in a control builder) and reuse: resolved results are cached per override set.
#[derive(Clone, Debug)]
pub struct Overrides {
    id: u64,
    entries: Arc<[(&'static str, &'static str, OverrideValue)]>,
}

fn next_id() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

impl Default for Overrides {
    fn default() -> Self {
        Self::new()
    }
}

impl Overrides {
    pub fn new() -> Self {
        Self { id: next_id(), entries: Arc::from(Vec::new()) }
    }

    /// Set a property of a part to a value of the property's type.
    pub fn set<T: Into<Value>>(self, part: PartId, prop: Prop<T>, value: T) -> Self {
        self.with(part.name(), prop.name(), OverrideValue::Value(value.into()))
    }

    /// Set a property of a part to a token, e.g. `"palette.state.selected.background"`.
    pub fn set_token<T>(self, part: PartId, prop: Prop<T>, token: &str) -> Self {
        self.with(part.name(), prop.name(), OverrideValue::Token(token.into()))
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Identifies this override set in caches.
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &'static str, &OverrideValue)> {
        self.entries.iter().map(|(p, n, v)| (*p, *n, v))
    }

    pub(crate) fn get(&self, part: &str, prop: &str) -> Option<&OverrideValue> {
        self.entries.iter().rev().find(|(p, n, _)| *p == part && *n == prop).map(|(_, _, v)| v)
    }

    fn with(self, part: &str, prop: &str, value: OverrideValue) -> Self {
        let mut entries: Vec<_> =
            self.entries.iter().filter(|(p, n, _)| !(*p == part && *n == prop)).cloned().collect();
        entries.push((intern(part), intern(prop), value));
        Self { id: next_id(), entries: entries.into() }
    }
}
