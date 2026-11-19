//! The compiled form of a set of style files.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::config::Config;
use crate::diag::Span;
use crate::syntax::Expr;
use crate::value::Kind;

pub(crate) struct FileInfo {
    pub name: Arc<str>,
    pub text: Arc<str>,
}

/// A lexical scope of `const` bindings.
pub(crate) struct ConstScope {
    pub parent: Option<usize>,
    pub entries: HashMap<String, ConstDef>,
}

pub(crate) struct ConstDef {
    pub expr: Arc<Expr>,
    /// The scope the expression is evaluated in (its own block).
    pub scope: usize,
    pub kind: Option<Kind>,
    pub span: Span,
}

/// One definition of a token path (several layers, modes and attribute scopes may define it).
pub(crate) struct TokenDef {
    pub layer: usize,
    pub mode: Option<String>,
    pub attrs: Vec<(String, String)>,
    pub expr: Arc<Expr>,
    /// The enclosing group path, for sibling lookups (`control_height = height`).
    pub group: Vec<String>,
    pub scope: usize,
    pub kind: Option<Kind>,
    pub order: usize,
    pub span: Span,
}

#[derive(Default)]
pub(crate) struct Tokens {
    pub defs: HashMap<String, Vec<TokenDef>>,
    /// Every group path (`"palette"`, `"palette.app"`).
    pub groups: HashSet<String>,
}

impl Tokens {
    pub fn is_root(&self, name: &str) -> bool {
        self.groups.contains(name) || self.defs.contains_key(name)
    }

    /// The longest prefix of `segs` that is a token, and the remaining field names.
    pub fn find<'s>(&self, segs: &'s [String]) -> Option<(String, &'s [String])> {
        (1..=segs.len()).rev().find_map(|i| {
            let path = segs[..i].join(".");
            self.defs.contains_key(&path).then_some((path, &segs[i..]))
        })
    }
}

pub(crate) struct PartDef {
    pub name: String,
    pub doc: Option<String>,
}

pub(crate) struct PropDecl {
    pub kind: Kind,
    pub doc: Option<String>,
}

/// One assignment: `prop = expr` for a part, in a scope (attributes) and states, from one layer.
pub(crate) struct Rule {
    pub part: usize,
    pub prop: String,
    pub kind: Kind,
    pub attrs: Vec<(String, String)>,
    pub states: u32,
    pub layer: usize,
    pub order: usize,
    pub owning: bool,
    pub expr: Arc<Expr>,
    pub scope: usize,
    pub span: Span,
}

pub(crate) struct ControlDef {
    pub name: String,
    pub doc: Option<String>,
    pub declaring_layer: usize,
    pub base: Option<usize>,
    pub base_name: Option<(String, Span)>,
    /// Part 0 is `root`.
    pub parts: Vec<PartDef>,
    pub props: HashMap<String, PropDecl>,
    pub rules: Vec<Rule>,
    /// `part b extends a` / `part b extends other.part`: part → (control, part, layer of the extends).
    pub part_extends: HashMap<usize, (usize, usize, usize)>,
    /// `variant ghost extends outline`: (attr, value) → base value.
    pub aliases: HashMap<(String, String), String>,
    /// Every (attribute, value) used as a scope, for tools and alias checks.
    pub scope_values: Vec<(String, String)>,
}

impl ControlDef {
    pub fn part(&self, name: &str) -> Option<usize> {
        self.parts.iter().position(|p| p.name == name)
    }
}

pub(crate) struct Model {
    pub config: Arc<Config>,
    pub files: Vec<FileInfo>,
    pub controls: Vec<ControlDef>,
    pub control_index: HashMap<String, usize>,
    pub consts: Vec<ConstScope>,
    pub tokens: Tokens,
    /// Native roots and look functions referenced by the files, checked when a theme is bound.
    pub native_refs: Vec<(String, Span)>,
    pub look_calls: Vec<(String, usize, Span)>,
    /// The files as given, with their layer names, for recompiling after edits.
    pub sources: Vec<(String, crate::compile::Source)>,
}

impl Model {
    pub fn control(&self, name: &str) -> Option<usize> {
        self.control_index.get(name).copied()
    }

    /// A property's kind for a control (core, or declared on it or a base control).
    pub fn prop_kind(&self, control: usize, prop: &str) -> Option<Kind> {
        if let Some((kind, _)) = crate::builtins::core_property(prop) {
            return Some(kind);
        }
        let mut c = Some(control);
        let mut guard = 0;
        while let Some(i) = c {
            if let Some(d) = self.controls[i].props.get(prop) {
                return Some(d.kind);
            }
            c = self.controls[i].base;
            guard += 1;
            if guard > 32 {
                break;
            }
        }
        None
    }

    /// Core and declared property names usable on a control (for suggestions).
    pub fn prop_names(&self, control: usize) -> Vec<&str> {
        let mut names: Vec<&str> = crate::builtins::CORE_PROPERTIES.iter().map(|(n, _, _)| *n).collect();
        let mut c = Some(control);
        let mut guard = 0;
        while let Some(i) = c {
            names.extend(self.controls[i].props.keys().map(String::as_str));
            c = self.controls[i].base;
            guard += 1;
            if guard > 32 {
                break;
            }
        }
        names
    }

    pub fn const_lookup(&self, scope: usize, name: &str) -> Option<&ConstDef> {
        let mut s = Some(scope);
        while let Some(i) = s {
            if let Some(d) = self.consts[i].entries.get(name) {
                return Some(d);
            }
            s = self.consts[i].parent;
        }
        None
    }

    pub fn attr_match(&self, control: usize, instance: usize, attr: &str, want: &str, have: &str) -> Option<bool> {
        // Some(false): direct match; Some(true): matched through `extends` aliases; None: no match.
        if want == have {
            return Some(false);
        }
        let mut current = have.to_string();
        for _ in 0..16 {
            let next = [control, instance]
                .iter()
                .find_map(|&c| self.controls[c].aliases.get(&(attr.to_string(), current.clone())).cloned());
            match next {
                Some(base) if base == want => return Some(true),
                Some(base) => current = base,
                None => return None,
            }
        }
        None
    }
}
