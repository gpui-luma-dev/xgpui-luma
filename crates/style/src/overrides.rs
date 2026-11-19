//! Instance overrides: values set on one control instance (API topic 5).

use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

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
/// Override sets with equal contents share cached results, so building one per frame is fine.
#[derive(Clone, Debug)]
pub struct Overrides {
    entries: Arc<[(&'static str, &'static str, OverrideValue)]>,
    /// Exact description of the contents; the cache key. Empty when there are no entries.
    content: Arc<str>,
}

impl Default for Overrides {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for Overrides {
    fn eq(&self, other: &Self) -> bool {
        self.content == other.content
    }
}

impl Eq for Overrides {}

impl Overrides {
    pub fn new() -> Self {
        Self { entries: Arc::from(Vec::new()), content: Arc::from("") }
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

    /// A hash of the contents: equal override sets have equal ids.
    pub fn id(&self) -> u64 {
        let mut h = DefaultHasher::new();
        self.content.hash(&mut h);
        h.finish()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &'static str, &OverrideValue)> {
        self.entries.iter().map(|(p, n, v)| (*p, *n, v))
    }

    pub(crate) fn content(&self) -> &Arc<str> {
        &self.content
    }

    pub(crate) fn get(&self, part: &str, prop: &str) -> Option<&OverrideValue> {
        self.entries.iter().find(|(p, n, _)| *p == part && *n == prop).map(|(_, _, v)| v)
    }

    fn with(self, part: &str, prop: &str, value: OverrideValue) -> Self {
        let mut entries: Vec<_> =
            self.entries.iter().filter(|(p, n, _)| !(*p == part && *n == prop)).cloned().collect();
        entries.push((intern(part), intern(prop), value));
        // Sorted, so the order of `set` calls does not change the identity.
        entries.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));
        // `Debug` prints floats exactly, so equal descriptions mean equal contents.
        let content: String = entries.iter().map(|(p, n, v)| format!("{p}.{n}={v:?};")).collect();
        Self { entries: entries.into(), content: content.into() }
    }
}
