//! Introspection for tools: where each value came from, what a control offers, and checking (API topics 6, 8).

use std::collections::HashSet;
use std::sync::Arc;

use crate::builtins;
use crate::compile::Stylesheet;
use crate::diag::Diagnostics;
use crate::handles::{AttrId, AttrValue, ControlId, PartId, StateId, StyleKey};
use crate::overrides::Overrides;
use crate::resolve::{Cand, Evaluator, Theme, expr_end, props_of_part, select};
use crate::value::{Kind, Value};

/// A place in a style file.
#[derive(Clone, Debug, PartialEq)]
pub struct Location {
    pub source: Arc<str>,
    pub line: u32,
    pub column: u32,
}

/// One rule as the inspector shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct RuleInfo {
    pub source: Arc<str>,
    pub layer: String,
    pub line: u32,
    pub column: u32,
    /// e.g. `"variant soft"`, empty for the base.
    pub scope: String,
    /// e.g. `":hover:checked"`, empty when the rule has no states.
    pub states: String,
    /// Source text of the value.
    pub expression: String,
    /// The `extends` path this rule was reached through, if any.
    pub via: Option<String>,
    /// Byte range of the value in the source, for edits.
    pub(crate) range: (u32, u32),
}

/// How a property got its value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueSource {
    Rule,
    InheritedFromRoot,
    InstanceOverride,
    Unset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceKind {
    Property,
    Token,
    Constant,
    Native,
}

/// Something an expression read, with its value, where it is defined, and what it read in turn.
#[derive(Clone, Debug, PartialEq)]
pub struct Reference {
    pub name: String,
    pub kind: ReferenceKind,
    pub value: Option<Value>,
    pub defined_at: Option<Location>,
    /// For native tokens, the token source's description (e.g. "--primary in theme.css").
    pub description: Option<String>,
    pub children: Vec<Reference>,
}

/// Where one property's value came from.
#[derive(Clone, Debug, PartialEq)]
pub struct Explanation {
    pub value: Option<Value>,
    pub source: ValueSource,
    pub winner: Option<RuleInfo>,
    /// Other matching rules, highest ranked first.
    pub overridden: Vec<RuleInfo>,
    /// The `:=` assignment that removed less specific rules, if one did.
    pub owned_by: Option<RuleInfo>,
    /// What the winning value read.
    pub references: Vec<Reference>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Colors,
    Layout,
    Typography,
    Elevation,
    Motion,
    Other,
}

impl Category {
    pub(crate) fn of(prop: &str, kind: Kind) -> Self {
        match kind {
            Kind::Color => Category::Colors,
            Kind::Shadow => Category::Elevation,
            Kind::Motion => Category::Motion,
            Kind::Font | Kind::Weight | Kind::Text => Category::Typography,
            Kind::Length if prop == "font_size" || prop == "line_height" => Category::Typography,
            Kind::Length | Kind::Number => Category::Layout,
            Kind::Icon => Category::Other,
        }
    }
}

/// One inspector row.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub property: &'static str,
    pub kind: Kind,
    pub category: Category,
    pub doc: Option<String>,
    pub explanation: Explanation,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

/// Box model data for drawing; built from the core layout properties.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoxModel {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub min_width: Option<f32>,
    pub min_height: Option<f32>,
    pub padding: Edges,
    pub border_width: f32,
    pub margin: Edges,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PartInspection {
    pub part: PartId,
    pub doc: Option<String>,
    pub rows: Vec<Row>,
    pub box_model: BoxModel,
}

/// Everything about one control instance, for the generic inspector.
#[derive(Clone, Debug, PartialEq)]
pub struct Inspection {
    pub control: ControlId,
    pub key: StyleKey,
    pub doc: Option<String>,
    pub parts: Vec<PartInspection>,
}

/// What a control's pickers should offer, read from the style files.
#[derive(Clone, Debug, PartialEq)]
pub struct Axes {
    pub attributes: Vec<(AttrId, Vec<AttrValue>)>,
    pub states: Vec<StateId>,
    pub modes: Vec<String>,
}

impl Stylesheet {
    /// Attribute values (from scopes), states (from rules) and modes (from token files) for a control,
    /// including what it reaches through `extends`.
    pub fn axes(&self, control: ControlId) -> Axes {
        let model = &self.model;
        let mut attrs: Vec<(String, String)> = Vec::new();
        let mut states = 0u32;
        let mut seen = HashSet::new();
        let mut todo: Vec<usize> = model.control(control.name()).into_iter().collect();
        while let Some(c) = todo.pop() {
            if !seen.insert(c) {
                continue;
            }
            let def = &model.controls[c];
            for pair in &def.scope_values {
                if !attrs.contains(pair) {
                    attrs.push(pair.clone());
                }
            }
            for (attr, value) in def.aliases.keys() {
                let pair = (attr.clone(), value.clone());
                if !attrs.contains(&pair) {
                    attrs.push(pair);
                }
            }
            for r in &def.rules {
                states |= r.states;
            }
            todo.extend(def.base);
            todo.extend(def.part_extends.values().map(|(c, _, _)| *c));
        }
        let mut attributes: Vec<(AttrId, Vec<AttrValue>)> = Vec::new();
        for (attr, value) in attrs {
            let Some(id) = model.config.attribute(&attr) else {
                continue;
            };
            match attributes.iter_mut().find(|(a, _)| *a == id) {
                Some((_, values)) => values.push(AttrValue::named(&value)),
                None => attributes.push((id, vec![AttrValue::named(&value)])),
            }
        }
        attributes.sort_by_key(|(a, _)| *a);
        let states = (0..model.config.states.len())
            .filter(|i| states & (1 << i) != 0)
            .map(|i| StateId(i as u8))
            .collect();
        Axes { attributes, states, modes: self.modes() }
    }
}

impl Theme {
    /// Where `part.prop` gets its value for `key`.
    pub fn explain(&self, key: &StyleKey, part: PartId, prop: &str) -> Explanation {
        self.explain_inner(key, None, part, prop)
    }

    /// Like [`Theme::explain`], with values set on this instance.
    pub fn explain_with(&self, key: &StyleKey, overrides: &Overrides, part: PartId, prop: &str) -> Explanation {
        self.explain_inner(key, Some(overrides), part, prop)
    }

    fn explain_inner(&self, key: &StyleKey, overrides: Option<&Overrides>, part: PartId, prop: &str) -> Explanation {
        let unset = Explanation {
            value: None,
            source: ValueSource::Unset,
            winner: None,
            overridden: Vec::new(),
            owned_by: None,
            references: Vec::new(),
        };
        let Some(k) = self.key_data(key) else { return unset };
        let model = self.inner.model();
        let Some(p) = model.controls[k.control].part(part.name()) else {
            return unset;
        };

        let ev = Evaluator::new(&self.inner, k.control, overrides, true);
        let value = ev.output(k.control, p, prop, &k).ok().flatten();
        let references = ev.take_references();

        let overridden_by_instance =
            overrides.and_then(|o| o.get(part.name(), prop)).is_some() && model.prop_kind(k.control, prop).is_some();
        let sel = select(model, k.control, p, prop, &k);
        let mut ranked: Vec<&Cand<'_>> = sel.kept.iter().collect();
        ranked.sort_by_key(|c| std::cmp::Reverse(c.rank()));
        let inherits = builtins::core_property(prop).is_some_and(|(_, i)| i);
        let source = if overridden_by_instance {
            ValueSource::InstanceOverride
        } else if !ranked.is_empty() {
            ValueSource::Rule
        } else if p != 0 && inherits && value.is_some() {
            ValueSource::InheritedFromRoot
        } else {
            ValueSource::Unset
        };
        let (winner, overridden) = match source {
            ValueSource::Rule => (
                ranked.first().map(|c| self.rule_info(c)),
                ranked.iter().skip(1).map(|c| self.rule_info(c)).collect(),
            ),
            ValueSource::InstanceOverride => (None, ranked.iter().map(|c| self.rule_info(c)).collect()),
            _ => (None, Vec::new()),
        };
        Explanation {
            value,
            source,
            winner,
            overridden,
            owned_by: sel.owner.as_ref().map(|c| self.rule_info(c)),
            references,
        }
    }

    pub(crate) fn rule_info(&self, c: &Cand<'_>) -> RuleInfo {
        let model = self.inner.model();
        let file = &model.files[c.rule.span.file as usize];
        let range = (c.rule.expr.span.start, expr_end(&c.rule.expr));
        let expression = file.text.get(range.0 as usize..range.1 as usize).unwrap_or("").trim().to_string();
        RuleInfo {
            source: file.name.clone(),
            layer: model.config.layers[c.rule.layer].clone(),
            line: c.rule.span.line,
            column: c.rule.span.column,
            scope: c.rule.attrs.iter().map(|(a, v)| format!("{a} {v}")).collect::<Vec<_>>().join(", "),
            states: (0..model.config.states.len())
                .filter(|i| c.rule.states & (1 << i) != 0)
                .map(|i| format!(":{}", model.config.states[i]))
                .collect(),
            expression,
            via: c.via.clone(),
            range,
        }
    }

    /// Every part and property of one control instance, for the generic inspector.
    pub fn inspect(&self, key: &StyleKey, overrides: Option<&Overrides>) -> Inspection {
        let mut inspection = Inspection { control: key.control, key: *key, doc: None, parts: Vec::new() };
        let Some(k) = self.key_data(key) else { return inspection };
        let model = self.inner.model();
        let control = &model.controls[k.control];
        inspection.control = self.control_id(k.control);
        inspection.doc = control.doc.clone();
        let resolved = match overrides {
            Some(o) => self.resolve_with(key, o),
            None => self.resolve(key),
        };
        for (pi, part) in control.parts.iter().enumerate() {
            let part_id = PartId::named(&part.name);
            let mut props = props_of_part(model, k.control, pi, overrides);
            if pi == 0 {
                // Declared properties are listed even when unset, so authors see what is available.
                let mut c = Some(k.control);
                while let Some(i) = c {
                    let mut names: Vec<&String> = model.controls[i].props.keys().collect();
                    names.sort();
                    for n in names {
                        if !props.contains(n) {
                            props.push(n.clone());
                        }
                    }
                    c = model.controls[i].base;
                }
            }
            let rows: Vec<Row> = props
                .iter()
                .map(|prop| {
                    let kind = model.prop_kind(k.control, prop).unwrap_or(Kind::Number);
                    let explanation = self.explain_inner(key, overrides, part_id, prop);
                    Row {
                        property: crate::handles::intern(prop),
                        kind,
                        category: Category::of(prop, kind),
                        doc: declared_doc(model, k.control, prop),
                        explanation,
                    }
                })
                .collect();
            let view = resolved.part(part_id);
            let len = |name: &str| view.value(name).and_then(Value::as_number);
            let padding_x = len("padding_x").or(len("padding")).unwrap_or(0.0);
            let padding_y = len("padding_y").or(len("padding")).unwrap_or(0.0);
            let margin = len("margin").unwrap_or(0.0);
            let box_model = BoxModel {
                width: len("width"),
                height: len("height"),
                min_width: len("min_width"),
                min_height: len("min_height"),
                padding: Edges { top: padding_y, right: padding_x, bottom: padding_y, left: padding_x },
                border_width: len("border_width").unwrap_or(0.0),
                margin: Edges { top: margin, right: margin, bottom: margin, left: margin },
            };
            inspection.parts.push(PartInspection { part: part_id, doc: part.doc.clone(), rows, box_model });
        }
        inspection
    }

    /// Resolve every combination of a control's axes (attribute values × states × modes) and return
    /// the problems found. For tests and for the lab after a live edit.
    pub fn check(&self, control: ControlId) -> Diagnostics {
        let axes = self.stylesheet().axes(control);
        let themes: Vec<Theme> = if axes.modes.is_empty() {
            self.with_mode(self.mode()).into_iter().collect()
        } else {
            axes.modes.iter().filter_map(|m| self.with_mode(m).ok()).collect()
        };
        let mut keys = vec![StyleKey::new(control)];
        for (attr, values) in &axes.attributes {
            let mut next = Vec::new();
            for key in &keys {
                next.push(*key);
                for v in values {
                    next.push(key.attr(*attr, *v));
                }
            }
            keys = next;
        }
        let states: Vec<StateId> = axes.states.iter().copied().take(10).collect();
        let mut all = Vec::new();
        for theme in &themes {
            for key in &keys {
                for mask in 0u32..(1 << states.len()) {
                    let mut k = *key;
                    for (i, s) in states.iter().enumerate() {
                        k = k.state_if(*s, mask & (1 << i) != 0);
                    }
                    theme.resolve(&k);
                }
            }
            for d in theme.diagnostics().0 {
                if !all.contains(&d) {
                    all.push(d);
                }
            }
        }
        Diagnostics(all)
    }
}

fn declared_doc(model: &crate::model::Model, control: usize, prop: &str) -> Option<String> {
    let mut c = Some(control);
    while let Some(i) = c {
        if let Some(d) = model.controls[i].props.get(prop) {
            return d.doc.clone();
        }
        c = model.controls[i].base;
    }
    None
}
