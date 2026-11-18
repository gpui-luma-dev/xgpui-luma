//! Build-time Fluent resource checks and typed Rust constructors.
//!
//! Add this crate as a build dependency with `default-features = false` and
//! `features = ["build-support"]`. In `build.rs`, read the English sources in
//! SDK-first/app-last order, call `Catalog::from_sources`, and write the result
//! of `generate_rust()` to `OUT_DIR/messages.rs`. Emit `cargo:rerun-if-changed`
//! for each input. Include that file inside a module in your app:
//! `mod messages { include!(concat!(env!("OUT_DIR"), "/messages.rs")); }`.
//!
//! ```no_run
//! use xgpui_luma_l10n::build_support::Catalog;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let source = std::fs::read_to_string("locales/en/lab.ftl")?;
//! let catalog = Catalog::from_sources([("locales/en/lab.ftl", source.as_str())])?;
//! let output = std::path::PathBuf::from(std::env::var("OUT_DIR")?).join("messages.rs");
//! std::fs::write(output, catalog.generate_rust()?)?;
//! println!("cargo:rerun-if-changed=locales/en/lab.ftl");
//! # Ok(())
//! # }
//! ```
//!
//! `lab-items-count` becomes `messages::msg_lab_items_count(count)`; attributes
//! become constructors such as `msg_search_placeholder()`. Argument order is
//! alphabetical and documented on each constructor. Types accept Fluent values
//! (including strings and numbers); argument *names and presence* are checked,
//! not semantic types. The runtime dependency must be named `xgpui_luma_l10n`.
//!
//! Compare every locale against English with `Catalog::compare`. Missing entries
//! are reported even though runtime fallback permits partial translations. An
//! explicit usage manifest (one Fluent ID or `id.attribute` per line) drives
//! `Catalog::unused`; generated IDs are available as `MESSAGE_IDS`. No heuristic
//! Rust source scanning is used. The `luma-l10n-check` binary exposes these checks.
//!
//! Run it with `cargo run -p xgpui-luma-l10n --no-default-features --features
//! build-support --bin luma-l10n-check -- --reference locales/en/lab.ftl
//! --locale de=locales/de/lab.ftl --used used-ids.txt`.

mod analyze;
mod generate;

use std::collections::{BTreeMap, BTreeSet};

use fluent_syntax::{ast, parser};

/// A resource or generation error; messages include the source/entry involved.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct CatalogError(pub(crate) String);

#[derive(Clone, Debug, Default)]
struct Node {
    variables: BTreeSet<String>,
    // Term arguments are scoped by Fluent and do not become caller arguments.
    references: Vec<(String, bool)>,
}

#[derive(Clone)]
struct Resolved {
    variables: BTreeSet<String>,
    depth: usize,
}

/// Validated Fluent definitions. Multiple sources use runtime-compatible
/// SDK-first/app-last overrides; duplicate entries *within* a source are errors.
#[derive(Debug)]
pub struct Catalog {
    nodes: BTreeMap<String, Node>,
    messages: BTreeMap<String, BTreeSet<String>>,
}

/// A translation contract difference. Ordering is stable for CI output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Diagnostic {
    MissingMessage(String),
    UnknownMessage(String),
    Arguments { id: String, expected: Vec<String>, actual: Vec<String> },
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingMessage(id) => write!(f, "missing message: {id}"),
            Self::UnknownMessage(id) => write!(f, "unknown message: {id}"),
            Self::Arguments { id, expected, actual } => {
                write!(f, "argument mismatch for {id}: expected {expected:?}, found {actual:?}")
            }
        }
    }
}

impl Catalog {
    /// Parse labeled sources, validate references/cycles, and resolve message
    /// argument contracts through message and attribute references.
    /// Reference chains deeper than 100 nodes are rejected to bound traversal.
    pub fn from_sources<'a>(sources: impl IntoIterator<Item = (&'a str, &'a str)>) -> Result<Self, CatalogError> {
        let mut catalog = Self { nodes: BTreeMap::new(), messages: BTreeMap::new() };
        for (name, source) in sources {
            let resource = parser::parse(source)
                .map_err(|(_, errors)| CatalogError(format!("{name}: invalid Fluent: {errors:?}")))?;
            let mut seen = BTreeSet::new();
            for entry in resource.body {
                let (id, value, attributes) = match entry {
                    ast::Entry::Message(message) => (message.id.name.to_owned(), message.value, message.attributes),
                    ast::Entry::Term(term) => (format!("-{}", term.id.name), Some(term.value), term.attributes),
                    _ => continue,
                };
                if !seen.insert(id.clone()) {
                    return Err(CatalogError(format!("{name}: duplicate entry {id}")));
                }
                catalog.nodes.retain(|key, _| key != &id && !key.starts_with(&format!("{id}.")));
                if let Some(value) = value {
                    catalog.insert(&id, &value);
                }
                let mut attrs = BTreeSet::new();
                for attr in attributes {
                    if !attrs.insert(attr.id.name) {
                        return Err(CatalogError(format!("{name}: duplicate attribute {id}.{}", attr.id.name)));
                    }
                    catalog.insert(&format!("{id}.{}", attr.id.name), &attr.value);
                }
            }
        }
        let mut resolved = BTreeMap::new();
        for id in catalog.nodes.keys() {
            let variables = catalog.resolve(id, &mut BTreeSet::new(), &mut resolved)?;
            if !id.starts_with('-') {
                catalog.messages.insert(id.clone(), variables);
            }
        }
        Ok(catalog)
    }

    fn insert(&mut self, id: &str, pattern: &ast::Pattern<&str>) {
        let mut node = Node::default();
        analyze::pattern(pattern, &mut node);
        self.nodes.insert(id.to_owned(), node);
    }

    fn resolve(
        &self,
        id: &str,
        visiting: &mut BTreeSet<String>,
        resolved: &mut BTreeMap<String, Resolved>,
    ) -> Result<BTreeSet<String>, CatalogError> {
        if let Some(result) = resolved.get(id) {
            if visiting.len() + result.depth > 100 {
                return Err(CatalogError(format!("Fluent reference depth exceeds 100 at {id}")));
            }
            return Ok(result.variables.clone());
        }
        if visiting.len() >= 100 {
            return Err(CatalogError(format!("Fluent reference depth exceeds 100 at {id}")));
        }
        let node = self.nodes.get(id).ok_or_else(|| CatalogError(format!("unresolved Fluent reference: {id}")))?;
        if !visiting.insert(id.to_owned()) {
            return Err(CatalogError(format!("cyclic Fluent reference: {id}")));
        }
        let mut variables = node.variables.clone();
        let mut depth = 1;
        for (reference, inherit) in &node.references {
            let referenced = self.resolve(reference, visiting, resolved)?;
            if let Some(result) = resolved.get(reference) {
                depth = depth.max(result.depth + 1);
            }
            if *inherit {
                variables.extend(referenced);
            }
        }
        visiting.remove(id);
        resolved.insert(id.to_owned(), Resolved { variables: variables.clone(), depth });
        Ok(variables)
    }

    /// Message values and attributes with required caller argument names.
    /// Terms are private to Fluent and are omitted from this public contract.
    pub fn messages(&self) -> &BTreeMap<String, BTreeSet<String>> {
        &self.messages
    }

    /// Compare a translated catalog with this (normally English) reference.
    pub fn compare(&self, translation: &Self) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for (id, expected) in &self.messages {
            match translation.messages.get(id) {
                None => diagnostics.push(Diagnostic::MissingMessage(id.clone())),
                Some(actual) if actual != expected => diagnostics.push(Diagnostic::Arguments {
                    id: id.clone(),
                    expected: expected.iter().cloned().collect(),
                    actual: actual.iter().cloned().collect(),
                }),
                _ => {}
            }
        }
        diagnostics.extend(
            translation
                .messages
                .keys()
                .filter(|id| !self.messages.contains_key(*id))
                .cloned()
                .map(Diagnostic::UnknownMessage),
        );
        diagnostics
    }

    /// Report unused values/attributes from an explicit list of code-used IDs.
    /// Referenced messages (including those used through terms) count as used.
    /// Unknown used IDs are errors, ensuring every supplied ID exists in English.
    pub fn unused<S: AsRef<str>>(&self, used: impl IntoIterator<Item = S>) -> Result<Vec<String>, CatalogError> {
        let mut reachable = BTreeSet::new();
        for id in used {
            let id = id.as_ref();
            if !self.messages.contains_key(id) {
                return Err(CatalogError(format!("used message missing from reference catalog: {id}")));
            }
            self.mark(id, &mut reachable);
        }
        Ok(self.messages.keys().filter(|id| !reachable.contains(*id)).cloned().collect())
    }

    fn mark(&self, id: &str, reachable: &mut BTreeSet<String>) {
        if reachable.insert(id.to_owned()) {
            if let Some(node) = self.nodes.get(id) {
                for (reference, _) in &node.references {
                    self.mark(reference, reachable);
                }
            }
        }
    }
}
