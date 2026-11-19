//! Editing and exporting `.lss` (API topic 7). Edits change source text in place, keeping comments and
//! layout; new text is printed in one standard layout. The crate never writes files.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::color::Color;
use crate::compile::{Source, Stylesheet};
use crate::diag::{Code, Diagnostic, Diagnostics};
use crate::inspect::RuleInfo;
use crate::value::{Motion, ShadowLayer, Value};

/// A byte range in a file and its new text.
type Replacement<'a> = ((u32, u32), &'a str);

/// A rule to insert or export, by names.
#[derive(Clone, Debug, PartialEq)]
pub struct RuleSpec {
    pub control: String,
    /// Attribute scopes, e.g. `[("variant", "soft")]`.
    pub scope: Vec<(String, String)>,
    pub part: String,
    pub states: Vec<String>,
    pub property: String,
    /// Source text of the value, e.g. `"tone[6]"`.
    pub expression: String,
}

impl RuleSpec {
    pub fn new(control: &str, part: &str, property: &str, expression: &str) -> Self {
        Self {
            control: control.into(),
            scope: Vec::new(),
            part: part.into(),
            states: Vec::new(),
            property: property.into(),
            expression: expression.into(),
        }
    }

    pub fn scope(mut self, attr: &str, value: &str) -> Self {
        self.scope.push((attr.into(), value.into()));
        self
    }

    pub fn state(mut self, state: &str) -> Self {
        self.states.push(state.into());
        self
    }
}

/// One change to style files.
#[derive(Clone, Debug, PartialEq)]
pub enum Edit {
    /// Replace a rule's value text.
    Replace { source: Arc<str>, range: (u32, u32), text: String },
    /// Append a new rule to a file (created in `layer` if the stylesheet has no file of that name).
    Insert { layer: String, source: Arc<str>, text: String },
}

impl Edit {
    /// Replace the value of a rule from an [`Explanation`](crate::Explanation) or inspector row.
    pub fn replace(rule: &RuleInfo, expression: &str) -> Edit {
        Edit::Replace { source: rule.source.clone(), range: rule.range, text: expression.to_string() }
    }

    /// Append a rule to `source` (in `layer` when the file is new), creating the blocks it needs.
    pub fn insert(layer: &str, source: &str, rule: &RuleSpec) -> Edit {
        Edit::Insert {
            layer: layer.into(),
            source: source.into(),
            text: print_rules(&rule.control, &rule.scope, &[rule]),
        }
    }
}

impl Stylesheet {
    /// Apply edits and recompile. Returns the new stylesheet and the changed files (for saving).
    /// On errors the current stylesheet is unchanged.
    pub fn apply(&self, edits: &[Edit]) -> Result<(Stylesheet, Vec<Source>), Diagnostics> {
        let mut sources = self.model.sources.clone();
        let mut changed: Vec<Arc<str>> = Vec::new();
        let mut replaces: BTreeMap<Arc<str>, Vec<Replacement<'_>>> = BTreeMap::new();
        for edit in edits {
            match edit {
                Edit::Replace { source, range, text } => {
                    replaces.entry(source.clone()).or_default().push((*range, text))
                }
                Edit::Insert { .. } => {}
            }
        }
        for (name, mut list) in replaces {
            let Some((_, src)) = sources.iter_mut().find(|(_, s)| s.name == name) else {
                return Err(Diagnostics(vec![Diagnostic::error(Code::Io, format!("no style file named \"{name}\""))]));
            };
            // Apply from the end so earlier ranges stay valid.
            list.sort_by_key(|((start, _), _)| std::cmp::Reverse(*start));
            let mut text = src.text.to_string();
            for ((start, end), new) in list {
                if end as usize > text.len() || start > end {
                    return Err(Diagnostics(vec![Diagnostic::error(
                        Code::Io,
                        format!("edit range is outside \"{name}\""),
                    )]));
                }
                text.replace_range(start as usize..end as usize, new);
            }
            src.text = text.into();
            changed.push(name);
        }
        for edit in edits {
            if let Edit::Insert { layer, source, text } = edit {
                match sources.iter_mut().find(|(_, s)| s.name == *source) {
                    Some((_, src)) => {
                        let mut t = src.text.to_string();
                        if !t.ends_with('\n') {
                            t.push('\n');
                        }
                        t.push('\n');
                        t.push_str(text);
                        src.text = t.into();
                    }
                    None => sources.push((layer.clone(), Source::new(source.clone(), text.clone()))),
                }
                if !changed.contains(source) {
                    changed.push(source.clone());
                }
            }
        }
        let sheet = Stylesheet::compile_sources(self.model.config.clone(), sources)?;
        let files = changed
            .iter()
            .filter_map(|n| sheet.model.sources.iter().find(|(_, s)| s.name == *n).map(|(_, s)| s.clone()))
            .collect();
        Ok((sheet, files))
    }

    /// A complete `.lss` file with `rules` as a named variant of `control` (their own scopes are ignored),
    /// e.g. the lab's "export style".
    pub fn export_variant(&self, control: &str, variant: &str, rules: &[RuleSpec]) -> String {
        let refs: Vec<&RuleSpec> = rules.iter().collect();
        print_rules(control, &[("variant".to_string(), variant.to_string())], &refs)
    }
}

/// A `tokens { … }` file for token values, e.g. live edits from [`Theme::token_values`](crate::Theme::token_values).
pub fn export_tokens(values: &[(String, Value)]) -> String {
    #[derive(Default)]
    struct Node {
        value: Option<Value>,
        children: BTreeMap<String, Node>,
    }
    let mut root = Node::default();
    for (path, value) in values {
        let mut node = &mut root;
        for seg in path.split('.') {
            node = node.children.entry(seg.to_string()).or_default();
        }
        node.value = Some(value.clone());
    }
    fn write(node: &Node, indent: usize, out: &mut String) {
        for (name, child) in &node.children {
            let pad = "  ".repeat(indent);
            if let Some(v) = &child.value {
                out.push_str(&format!("{pad}{name} = {}\n", v.to_lss()));
            }
            if !child.children.is_empty() {
                out.push_str(&format!("{pad}{name} {{\n"));
                write(child, indent + 1, out);
                out.push_str(&format!("{pad}}}\n"));
            }
        }
    }
    let mut out = String::from("tokens {\n");
    write(&root, 1, &mut out);
    out.push_str("}\n");
    out
}

fn print_rules(control: &str, scope: &[(String, String)], rules: &[&RuleSpec]) -> String {
    let mut out = format!("{control} {{\n");
    let mut depth = 1;
    for (attr, value) in scope {
        out.push_str(&format!("{}{attr} {value} {{\n", "  ".repeat(depth)));
        depth += 1;
    }
    // Group by part, then by state list, keeping first-seen order.
    let mut parts: Vec<(&str, Vec<&RuleSpec>)> = Vec::new();
    for r in rules {
        match parts.iter_mut().find(|(p, _)| *p == r.part) {
            Some((_, list)) => list.push(r),
            None => parts.push((&r.part, vec![r])),
        }
    }
    for (part, list) in parts {
        let mut inner = depth;
        if part != "root" {
            out.push_str(&format!("{}part {part} {{\n", "  ".repeat(depth)));
            inner += 1;
        }
        let pad = "  ".repeat(inner);
        let mut groups: Vec<(String, Vec<&RuleSpec>)> = Vec::new();
        for r in list {
            let states: String = r.states.iter().map(|s| format!(":{s}")).collect();
            match groups.iter_mut().find(|(s, _)| *s == states) {
                Some((_, g)) => g.push(r),
                None => groups.push((states, vec![r])),
            }
        }
        for (states, group) in groups {
            if states.is_empty() {
                for r in group {
                    out.push_str(&format!("{pad}{} = {}\n", r.property, r.expression));
                }
            } else {
                let body: Vec<String> = group.iter().map(|r| format!("{} = {}", r.property, r.expression)).collect();
                out.push_str(&format!("{pad}{states} {{ {} }}\n", body.join("  ")));
            }
        }
        if part != "root" {
            out.push_str(&format!("{}}}\n", "  ".repeat(depth)));
        }
    }
    while depth > 1 {
        depth -= 1;
        out.push_str(&format!("{}}}\n", "  ".repeat(depth)));
    }
    out.push_str("}\n");
    out
}

fn num(n: f32) -> String {
    let s = format!("{:.4}", n);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".into() } else { s.into() }
}

fn color(c: &Color) -> String {
    format!("oklch({}, {}, {}, {})", num(c.l), num(c.c), num(c.h), num(c.alpha))
}

fn shadow(l: &ShadowLayer) -> String {
    let f = if l.inset { "inset_shadow" } else { "box_shadow" };
    format!("{f}({}, {}, {}, {}, {})", color(&l.color), num(l.x), num(l.y), num(l.blur), num(l.spread))
}

impl Value {
    /// The value as `.lss` source text.
    pub fn to_lss(&self) -> String {
        match self {
            Value::Number(n) => num(*n),
            Value::Color(c) => color(c),
            Value::Shadow(layers) => format!("[{}]", layers.iter().map(shadow).collect::<Vec<_>>().join(", ")),
            Value::Text(t) => format!("text({}, {}, {})", num(t.size), num(t.line_height), num(t.weight)),
            Value::Str(s) | Value::Icon(s) => format!("\"{s}\""),
            Value::Motion(Motion::Instant) => "instant".into(),
            Value::Motion(Motion::Timed { duration_ms, easing }) => {
                format!("timed({}, {})", num(*duration_ms), easing.name())
            }
            Value::Motion(Motion::Spring { stiffness, damping }) => {
                format!("spring({}, {})", num(*stiffness), num(*damping))
            }
            Value::Symbol(s) => s.to_string(),
            Value::List(items) => format!("[{}]", items.iter().map(Value::to_lss).collect::<Vec<_>>().join(", ")),
            Value::None => "none".into(),
        }
    }
}
