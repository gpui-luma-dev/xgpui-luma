//! Binding to tokens, the cascade (language §8) and resolution.

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::builtins;
use crate::color::Color;
use crate::compile::Stylesheet;
use crate::config::SnapPolicy;
use crate::diag::{Diagnostic, Diagnostics, Severity, Span};
use crate::model::{Model, Rule};
use crate::names::{self, Ctx, Target};
use crate::syntax::{BinOp, Expr, ExprKind};
use crate::token::{TokenCx, TokenSource};
use crate::value::{Kind, Motion, ShadowLayer, Value};

/// Control, attributes, states and scale bits.
type CacheKey = (String, Vec<(String, String)>, Vec<String>, u32);

/// What one render asks for: a control, its attributes, its states and the display scale.
#[derive(Clone, Debug, PartialEq)]
pub struct StyleKey {
    control: String,
    attrs: Vec<(String, String)>,
    states: Vec<String>,
    scale: f32,
}

impl StyleKey {
    pub fn new(control: &str) -> Self {
        Self { control: control.to_string(), attrs: Vec::new(), states: Vec::new(), scale: 1.0 }
    }

    /// Set an attribute, e.g. `.attr("variant", "soft")`.
    pub fn attr(mut self, name: &str, value: &str) -> Self {
        self.attrs.retain(|(n, _)| n != name);
        self.attrs.push((name.to_string(), value.to_string()));
        self.attrs.sort();
        self
    }

    /// Add an active state, e.g. `.state("checked")`.
    pub fn state(mut self, name: &str) -> Self {
        if !self.states.iter().any(|s| s == name) {
            self.states.push(name.to_string());
            self.states.sort();
        }
        self
    }

    pub fn states(self, names: &[&str]) -> Self {
        names.iter().fold(self, |key, s| key.state(s))
    }

    /// Device pixels per logical pixel, used for snapping.
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    pub fn control(&self) -> &str {
        &self.control
    }

    fn cache_key(&self) -> CacheKey {
        (self.control.clone(), self.attrs.clone(), self.states.clone(), self.scale.to_bits())
    }
}

/// Resolved values for every part of one control instance.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Resolved {
    parts: BTreeMap<String, BTreeMap<String, Value>>,
}

impl Resolved {
    pub fn part(&self, name: &str) -> PartView<'_> {
        PartView { values: self.parts.get(name) }
    }

    pub fn root(&self) -> PartView<'_> {
        self.part("root")
    }

    /// Part names that have at least one value.
    pub fn parts(&self) -> impl Iterator<Item = &str> {
        self.parts.keys().map(String::as_str)
    }

    /// Interpolate two resolved styles (e.g. unchecked → checked) for animation.
    pub fn lerp(a: &Resolved, b: &Resolved, t: f32) -> Resolved {
        let mut parts = BTreeMap::new();
        let names: HashSet<&String> = a.parts.keys().chain(b.parts.keys()).collect();
        for name in names {
            let empty = BTreeMap::new();
            let pa = a.parts.get(name).unwrap_or(&empty);
            let pb = b.parts.get(name).unwrap_or(&empty);
            let mut out = BTreeMap::new();
            for prop in pa.keys().chain(pb.keys()) {
                let v = match (pa.get(prop), pb.get(prop)) {
                    (Some(x), Some(y)) => x.lerp(y, t),
                    (Some(x), None) if t < 0.5 => x.clone(),
                    (None, Some(y)) if t >= 0.5 => y.clone(),
                    _ => continue,
                };
                out.insert(prop.clone(), v);
            }
            parts.insert(name.clone(), out);
        }
        Resolved { parts }
    }
}

/// Typed access to one part's values.
#[derive(Clone, Copy, Debug)]
pub struct PartView<'a> {
    values: Option<&'a BTreeMap<String, Value>>,
}

impl<'a> PartView<'a> {
    pub fn get(&self, prop: &str) -> Option<&'a Value> {
        self.values.and_then(|v| v.get(prop))
    }

    pub fn color(&self, prop: &str) -> Option<Color> {
        self.get(prop).and_then(Value::as_color)
    }

    pub fn length(&self, prop: &str) -> Option<f32> {
        self.get(prop).and_then(Value::as_number)
    }

    pub fn number(&self, prop: &str) -> Option<f32> {
        self.length(prop)
    }

    pub fn shadow(&self, prop: &str) -> Option<&'a [ShadowLayer]> {
        self.get(prop).and_then(Value::as_shadow)
    }

    pub fn string(&self, prop: &str) -> Option<&'a str> {
        self.get(prop).and_then(Value::as_str)
    }

    pub fn motion(&self, prop: &str) -> Option<Motion> {
        self.get(prop).and_then(Value::as_motion)
    }

    pub fn background(&self) -> Option<Color> {
        self.color("background")
    }
    pub fn foreground(&self) -> Option<Color> {
        self.color("foreground")
    }
    pub fn border(&self) -> Option<Color> {
        self.color("border")
    }
    pub fn focus_border(&self) -> Option<Color> {
        self.color("focus_border")
    }
    pub fn radius(&self) -> Option<f32> {
        self.length("radius")
    }
    pub fn height(&self) -> Option<f32> {
        self.length("height")
    }
    pub fn gap(&self) -> Option<f32> {
        self.length("gap")
    }
    pub fn font_size(&self) -> Option<f32> {
        self.length("font_size")
    }
    pub fn line_height(&self) -> Option<f32> {
        self.length("line_height")
    }
    pub fn transition(&self) -> Option<Motion> {
        self.motion("transition")
    }
}

/// Where one property's value came from.
#[derive(Clone, Debug, PartialEq)]
pub struct Explanation {
    pub value: Option<Value>,
    pub winner: Option<RuleInfo>,
    /// Other matching rules, highest ranked first.
    pub overridden: Vec<RuleInfo>,
    /// The `:=` assignment that removed less specific rules, if one did.
    pub owned_by: Option<RuleInfo>,
    pub inherited_from_root: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuleInfo {
    pub source: Arc<str>,
    pub layer: String,
    pub line: u32,
    pub column: u32,
    /// e.g. `"variant soft"`, empty for the base.
    pub scope: String,
    /// e.g. `":checked:hover"`.
    pub states: String,
    /// Source text of the value.
    pub expression: String,
    /// The `extends` path this rule was reached through, if any.
    pub via: Option<String>,
}

/// A stylesheet bound to native tokens and a mode. Cheap to clone; caches resolved keys.
#[derive(Clone)]
pub struct Theme {
    inner: Arc<ThemeInner>,
}

struct ThemeInner {
    model: Arc<Model>,
    tokens: Arc<dyn TokenSource>,
    mode: String,
    roots: HashSet<String>,
    cache: Mutex<HashMap<CacheKey, Arc<Resolved>>>,
    diags: Mutex<Vec<Diagnostic>>,
}

impl Theme {
    pub fn bind(stylesheet: &Stylesheet, tokens: Arc<dyn TokenSource>, mode: &str) -> Result<Theme, Diagnostics> {
        let model = stylesheet.model.clone();
        let roots: HashSet<String> = tokens.roots().iter().map(|r| r.to_string()).collect();
        let mut diags = Vec::new();
        let err = |span: Span, message: String| {
            let file = model.files.get(span.file as usize);
            Diagnostic {
                severity: Severity::Error,
                source: file.map(|f| f.name.clone()),
                line: span.line,
                column: span.column,
                message,
                snippet: file.and_then(|f| f.text.lines().nth(span.line.saturating_sub(1) as usize)).map(str::to_owned),
            }
        };
        for (root, span) in &model.native_refs {
            if !roots.contains(root) {
                diags.push(err(*span, format!("unknown name \"{root}\": not a token, constant or native token root")));
            }
        }
        for (name, args, span) in &model.look_calls {
            match tokens.function(name) {
                None => diags.push(err(*span, format!("unknown function \"{name}\""))),
                Some(sig) if *args < sig.min_args || *args > sig.max_args => diags.push(err(
                    *span,
                    format!("{name} expects {} to {} arguments, found {args}", sig.min_args, sig.max_args),
                )),
                Some(_) => {}
            }
        }
        for root in &roots {
            if model.tokens.is_root(root) || model.control(root).is_some() || model.consts[0].entries.contains_key(root)
            {
                diags.push(Diagnostic::error(format!(
                    "native token root \"{root}\" clashes with a token group, control or constant"
                )));
            }
        }
        if !diags.is_empty() {
            let mut seen = HashSet::new();
            diags.retain(|d| seen.insert((d.line, d.column, d.message.clone())));
            return Err(Diagnostics(diags));
        }
        Ok(Theme {
            inner: Arc::new(ThemeInner {
                model,
                tokens,
                mode: mode.to_string(),
                roots,
                cache: Mutex::new(HashMap::new()),
                diags: Mutex::new(Vec::new()),
            }),
        })
    }

    /// Start a key for a control.
    pub fn key(&self, control: &str) -> StyleKey {
        StyleKey::new(control)
    }

    pub fn mode(&self) -> &str {
        &self.inner.mode
    }

    /// Every part's values for one key. Problems leave a property unset and are recorded in [`Theme::diagnostics`].
    pub fn resolve(&self, key: &StyleKey) -> Arc<Resolved> {
        let ck = key.cache_key();
        if let Some(hit) = self.inner.cache.lock().expect("cache lock").get(&ck) {
            return hit.clone();
        }
        let resolved = Arc::new(self.compute(key));
        self.inner.cache.lock().expect("cache lock").insert(ck, resolved.clone());
        resolved
    }

    /// Problems found while resolving so far.
    pub fn diagnostics(&self) -> Diagnostics {
        Diagnostics(self.inner.diags.lock().expect("diagnostics lock").clone())
    }

    /// Where `part.prop` gets its value for `key`.
    pub fn explain(&self, key: &StyleKey, part: &str, prop: &str) -> Explanation {
        let empty = Explanation {
            value: None,
            winner: None,
            overridden: Vec::new(),
            owned_by: None,
            inherited_from_root: false,
        };
        let Some(k) = self.key_data(key) else { return empty };
        let model = &self.inner.model;
        let Some(p) = model.controls[k.control].part(part) else {
            return empty;
        };
        let ev = Evaluator::new(self);
        let value = ev.output(k.control, p, prop, &k).ok().flatten();
        let sel = select(model, k.control, p, prop, &k);
        let info = |c: &Cand<'_>| self.rule_info(c);
        let mut ranked: Vec<&Cand<'_>> = sel.kept.iter().collect();
        ranked.sort_by_key(|c| std::cmp::Reverse(c.rank()));
        let inherited_from_root = ranked.is_empty() && p != 0 && builtins::core_property(prop).is_some_and(|(_, i)| i);
        Explanation {
            value,
            winner: ranked.first().map(|c| info(c)),
            overridden: ranked.iter().skip(1).map(|c| info(c)).collect(),
            owned_by: sel.owner.as_ref().map(info),
            inherited_from_root,
        }
    }

    fn rule_info(&self, c: &Cand<'_>) -> RuleInfo {
        let model = &self.inner.model;
        let file = &model.files[c.rule.span.file as usize];
        let e = &c.rule.expr.span;
        let expression =
            file.text.get(e.start as usize..expr_end(&c.rule.expr) as usize).unwrap_or("").trim().to_string();
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
        }
    }

    fn key_data(&self, key: &StyleKey) -> Option<KeyData> {
        let model = &self.inner.model;
        let mut problem = None;
        let control = model.control(&key.control);
        if control.is_none() {
            problem = Some(format!("unknown control \"{}\"", key.control));
        }
        let mut states = 0u32;
        for s in &key.states {
            match model.config.state(s) {
                Some(i) => states |= 1 << i,
                None => problem = Some(format!("unknown state \"{s}\"")),
            }
        }
        for (a, _) in &key.attrs {
            if model.config.attribute(a).is_none() {
                problem = Some(format!("unknown attribute \"{a}\""));
            }
        }
        if let Some(message) = problem {
            self.inner.diags.lock().expect("diagnostics lock").push(Diagnostic::error(message));
            return None;
        }
        Some(KeyData {
            control: control?,
            attrs: key.attrs.clone(),
            states: model.config.normalize(states),
            scale: key.scale,
        })
    }

    fn compute(&self, key: &StyleKey) -> Resolved {
        let Some(k) = self.key_data(key) else {
            return Resolved::default();
        };
        let model = &self.inner.model;
        let ev = Evaluator::new(self);
        let mut parts = BTreeMap::new();
        for (pi, part) in model.controls[k.control].parts.iter().enumerate() {
            let mut props: Vec<String> = Vec::new();
            collect_props(model, k.control, pi, &mut props, 0);
            if pi != 0 {
                let mut root_props = Vec::new();
                collect_props(model, k.control, 0, &mut root_props, 0);
                for p in root_props {
                    if builtins::core_property(&p).is_some_and(|(_, inherits)| inherits) && !props.contains(&p) {
                        props.push(p);
                    }
                }
            }
            let mut values = BTreeMap::new();
            for prop in props {
                match ev.output(k.control, pi, &prop, &k) {
                    Ok(Some(v)) => {
                        values.insert(prop, v);
                    }
                    Ok(None) => {}
                    Err(EvalErr::Absent(path)) => ev.report(
                        None,
                        format!("{}.{}.{prop}: token \"{path}\" is absent", model.controls[k.control].name, part.name),
                    ),
                    Err(EvalErr::Error(span, message)) => ev.report(Some(span), message),
                }
            }
            if !values.is_empty() {
                parts.insert(part.name.clone(), values);
            }
        }
        let new = ev.diags.into_inner();
        if !new.is_empty() {
            let mut all = self.inner.diags.lock().expect("diagnostics lock");
            for d in new {
                if !all.contains(&d) {
                    all.push(d);
                }
            }
        }
        Resolved { parts }
    }
}

fn expr_end(expr: &Expr) -> u32 {
    let mut end = expr.span.end;
    let mut visit = |e: &Expr| end = end.max(expr_end(e));
    match &expr.kind {
        ExprKind::Call(_, args) | ExprKind::List(args) => args.iter().for_each(&mut visit),
        ExprKind::Index(a, b) | ExprKind::Bin(_, a, b) => {
            visit(a);
            visit(b);
        }
        ExprKind::Neg(a) => visit(a),
        _ => {}
    }
    end
}

/// Every property any rule sets for `part` of `control`, through `extends`.
fn collect_props(model: &Model, control: usize, part: usize, out: &mut Vec<String>, depth: usize) {
    if depth > 16 {
        return;
    }
    let c = &model.controls[control];
    for r in &c.rules {
        if r.part == part && !out.contains(&r.prop) {
            out.push(r.prop.clone());
        }
    }
    if let Some(&(sc, sp, _)) = c.part_extends.get(&part) {
        collect_props(model, sc, sp, out, depth + 1);
    }
    if let Some(b) = c.base
        && let Some(bp) = model.controls[b].part(&c.parts[part].name)
    {
        collect_props(model, b, bp, out, depth + 1);
    }
}

#[derive(Clone, Debug)]
struct KeyData {
    control: usize,
    attrs: Vec<(String, String)>,
    states: u32,
    scale: f32,
}

/// A matching rule with how it was reached.
struct Cand<'m> {
    rule: &'m Rule,
    /// The rule's layer, raised to the layer of the `extends` it was reached through.
    layer: usize,
    /// 0 for the control's own rules; more for rules reached through `extends`.
    depth: u32,
    interaction: bool,
    via: Option<String>,
}

impl Cand<'_> {
    /// Language §8.3: state count, interaction state, attribute count, layer, own over inherited, order.
    /// A rule reached through `extends` counts at least at the layer where the `extends` was written.
    fn rank(&self) -> (u32, bool, usize, usize, i64, usize) {
        (
            self.rule.states.count_ones(),
            self.interaction,
            self.rule.attrs.len(),
            self.layer,
            -(self.depth as i64),
            self.rule.order,
        )
    }
}

struct Selection<'m> {
    kept: Vec<Cand<'m>>,
    owner: Option<Cand<'m>>,
}

fn select<'m>(model: &'m Model, control: usize, part: usize, prop: &str, key: &KeyData) -> Selection<'m> {
    let mut all = Vec::new();
    collect_cands(model, control, control, part, prop, key, 0, 0, None, &mut all);
    // `:=` (language §8.4): drop less specific candidates, and lower layers' candidates for the owner's scope.
    let owner_index =
        all.iter().enumerate().filter(|(_, c)| c.rule.owning).max_by_key(|(_, c)| c.rank()).map(|(i, _)| i);
    let Some(oi) = owner_index else {
        return Selection { kept: all, owner: None };
    };
    let (o_attrs, o_layer, o_depth) = (all[oi].rule.attrs.clone(), all[oi].layer, all[oi].depth);
    let mut owner = None;
    let mut kept = Vec::new();
    for (i, c) in all.into_iter().enumerate() {
        let subset = c.rule.attrs.iter().all(|a| o_attrs.contains(a));
        let same_scope = c.depth == o_depth && c.layer == o_layer && c.rule.attrs == o_attrs;
        let below = c.depth > o_depth || c.layer < o_layer || (c.layer == o_layer && c.rule.attrs != o_attrs);
        if i == oi {
            owner = Some(Cand {
                rule: c.rule,
                layer: c.layer,
                depth: c.depth,
                interaction: c.interaction,
                via: c.via.clone(),
            });
        }
        if subset && below && !same_scope {
            continue;
        }
        kept.push(c);
    }
    Selection { kept, owner }
}

#[allow(clippy::too_many_arguments)]
fn collect_cands<'m>(
    model: &'m Model,
    instance: usize,
    control: usize,
    part: usize,
    prop: &str,
    key: &KeyData,
    depth: u32,
    min_layer: usize,
    via: Option<String>,
    out: &mut Vec<Cand<'m>>,
) {
    if depth > 16 {
        return;
    }
    let c = &model.controls[control];
    let exclusive = model.config.exclusive_mask();
    for rule in &c.rules {
        if rule.part != part || rule.prop != prop || rule.states & !key.states != 0 {
            continue;
        }
        let mut aliased = false;
        let mut matched = true;
        for (attr, want) in &rule.attrs {
            match key.attrs.iter().find(|(a, _)| a == attr) {
                Some((_, have)) => match model.attr_match(control, instance, attr, want, have) {
                    Some(alias) => aliased |= alias,
                    None => matched = false,
                },
                None => matched = false,
            }
        }
        if !matched {
            continue;
        }
        out.push(Cand {
            rule,
            layer: rule.layer.max(min_layer),
            depth: depth + u32::from(aliased),
            interaction: rule.states & exclusive != 0,
            via: via.clone(),
        });
    }
    if let Some(&(sc, sp, layer)) = c.part_extends.get(&part) {
        let label = format!("{}.{}", model.controls[sc].name, model.controls[sc].parts[sp].name);
        collect_cands(model, instance, sc, sp, prop, key, depth + 1, min_layer.max(layer), Some(label), out);
    }
    if let Some(b) = c.base
        && let Some(bp) = model.controls[b].part(&c.parts[part].name)
    {
        let layer = min_layer.max(c.declaring_layer);
        collect_cands(model, instance, b, bp, prop, key, depth + 1, layer, Some(model.controls[b].name.clone()), out);
    }
}

enum EvalErr {
    /// A native token is missing; `first()` skips these.
    Absent(String),
    Error(Span, String),
}

struct Evaluator<'t> {
    theme: &'t ThemeInner,
    stack: RefCell<Vec<String>>,
    diags: RefCell<Vec<Diagnostic>>,
}

impl<'t> Evaluator<'t> {
    fn new(theme: &'t Theme) -> Self {
        Self { theme: &theme.inner, stack: RefCell::new(Vec::new()), diags: RefCell::new(Vec::new()) }
    }

    fn model(&self) -> &'t Model {
        &self.theme.model
    }

    fn report(&self, span: Option<Span>, message: String) {
        let model = self.model();
        let file = span.and_then(|s| model.files.get(s.file as usize));
        let d = Diagnostic {
            severity: Severity::Warning,
            source: file.map(|f| f.name.clone()),
            line: span.map_or(0, |s| s.line),
            column: span.map_or(0, |s| s.column),
            message,
            snippet: file
                .zip(span)
                .and_then(|(f, s)| f.text.lines().nth(s.line.saturating_sub(1) as usize))
                .map(str::to_owned),
        };
        let mut diags = self.diags.borrow_mut();
        if !diags.contains(&d) {
            diags.push(d);
        }
    }

    fn enter(&self, label: String, span: Span) -> Result<(), EvalErr> {
        let mut stack = self.stack.borrow_mut();
        if stack.contains(&label) {
            let chain = stack.iter().skip_while(|l| **l != label).cloned().collect::<Vec<_>>().join(" → ");
            return Err(EvalErr::Error(span, format!("cycle: {chain} → {label}")));
        }
        stack.push(label);
        Ok(())
    }

    fn leave(&self) {
        self.stack.borrow_mut().pop();
    }

    /// The value of `part.prop` for `key` (language §8), or `None` when unset.
    fn prop(&self, control: usize, part: usize, prop: &str, key: &KeyData) -> Result<Option<Value>, EvalErr> {
        let model = self.model();
        let label = format!("{}.{}.{prop}", model.controls[control].name, model.controls[control].parts[part].name);
        let kind = model.prop_kind(control, prop);
        let sel = select(model, control, part, prop, key);
        let Some(winner) = sel.kept.iter().max_by_key(|c| c.rank()) else {
            let inherits = builtins::core_property(prop).is_some_and(|(_, i)| i);
            return if part != 0 && inherits {
                self.prop(control, 0, prop, key)
            } else {
                Ok(None)
            };
        };
        self.enter(label, winner.rule.span)?;
        let ctx = Ctx::Prop { control, part, scope: winner.rule.scope };
        let result = self.eval(&winner.rule.expr, ctx, key, false);
        self.leave();
        let value = result?;
        let kind = kind.unwrap_or(winner.rule.kind);
        let value = match value.coerce(kind) {
            Ok(Value::None) => return Ok(None),
            Ok(v) => v,
            Err(v) => {
                return Err(EvalErr::Error(
                    winner.rule.expr.span,
                    format!("{prop} expects {kind}, found {}", v.describe()),
                ));
            }
        };
        Ok(Some(value))
    }

    /// The value as a template receives it: like [`Self::prop`], with lengths snapped to device pixels
    /// unless the winning value is `exact(…)`. References between properties use unsnapped values.
    fn output(&self, control: usize, part: usize, prop: &str, key: &KeyData) -> Result<Option<Value>, EvalErr> {
        let model = self.model();
        let value = self.prop(control, part, prop, key)?;
        if model.config.snap != SnapPolicy::Lengths || model.prop_kind(control, prop) != Some(Kind::Length) {
            return Ok(value);
        }
        let exact = {
            let sel = select(model, control, part, prop, key);
            match sel.kept.iter().max_by_key(|c| c.rank()) {
                Some(w) => w.rule.expr.is_exact(),
                None => select(model, control, 0, prop, key)
                    .kept
                    .iter()
                    .max_by_key(|c| c.rank())
                    .is_some_and(|w| w.rule.expr.is_exact()),
            }
        };
        Ok(match value {
            Some(Value::Number(n)) if !exact && key.scale > 0.0 => {
                Some(Value::Number((n * key.scale).round() / key.scale))
            }
            other => other,
        })
    }

    fn token_cx<'k>(&'k self, key: &'k KeyData) -> TokenCx<'k> {
        TokenCx { mode: &self.theme.mode, attrs: &key.attrs }
    }

    fn eval(&self, expr: &Expr, ctx: Ctx<'_>, key: &KeyData, as_arg: bool) -> Result<Value, EvalErr> {
        let err = |message: String| EvalErr::Error(expr.span, message);
        match &expr.kind {
            ExprKind::Number(n) => Ok(Value::Number(*n)),
            ExprKind::Hex(h) => Color::parse_hex(h).map(Value::Color).ok_or_else(|| err(format!("invalid color {h}"))),
            ExprKind::Str(s) => Ok(Value::Str(s.as_str().into())),
            ExprKind::List(items) => {
                let values = items.iter().map(|e| self.eval(e, ctx, key, false)).collect::<Result<Vec<_>, _>>()?;
                if values.iter().all(|v| matches!(v, Value::Shadow(_))) {
                    let layers: Vec<ShadowLayer> =
                        values.iter().filter_map(Value::as_shadow).flat_map(|s| s.iter().copied()).collect();
                    Ok(Value::Shadow(layers.into()))
                } else {
                    Ok(Value::List(values.into()))
                }
            }
            ExprKind::Neg(inner) => match self.eval(inner, ctx, key, false)? {
                Value::Number(n) => Ok(Value::Number(-n)),
                other => Err(err(format!("cannot negate {}", other.describe()))),
            },
            ExprKind::Bin(op, a, b) => {
                let (a, b) = (self.eval(a, ctx, key, false)?, self.eval(b, ctx, key, false)?);
                match (a, b) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Number(match op {
                        BinOp::Add => a + b,
                        BinOp::Sub => a - b,
                        BinOp::Mul => a * b,
                        BinOp::Div if b == 0.0 => return Err(err("division by zero".into())),
                        BinOp::Div => a / b,
                    })),
                    (a, b) => {
                        Err(err(format!("arithmetic needs numbers, found {} and {}", a.describe(), b.describe())))
                    }
                }
            }
            ExprKind::Index(base, index) => {
                let n = match self.eval(index, ctx, key, false)? {
                    Value::Number(n) => n as i64,
                    other => return Err(err(format!("an index must be a number, found {}", other.describe()))),
                };
                if let ExprKind::Path(segs) = &base.kind
                    && segs.len() == 1
                    && self.theme.roots.contains(&segs[0])
                {
                    return self
                        .theme
                        .tokens
                        .index(&segs[0], n, &self.token_cx(key))
                        .ok_or(EvalErr::Absent(format!("{}[{n}]", segs[0])));
                }
                match self.eval(base, ctx, key, false)? {
                    Value::List(items) => {
                        items.get(n as usize).cloned().ok_or_else(|| err(format!("index {n} is out of range")))
                    }
                    Value::Shadow(layers) => layers
                        .get(n as usize)
                        .map(|l| Value::Shadow(vec![*l].into()))
                        .ok_or_else(|| err(format!("index {n} is out of range"))),
                    other => Err(err(format!("cannot index {}", other.describe()))),
                }
            }
            ExprKind::Call(func, args) => self.call(expr, &func.join("."), args, ctx, key),
            ExprKind::Path(segs) => self.path(expr, segs, ctx, key, as_arg),
        }
    }

    fn call(&self, expr: &Expr, name: &str, args: &[Expr], ctx: Ctx<'_>, key: &KeyData) -> Result<Value, EvalErr> {
        match name {
            "first" => {
                let mut last = None;
                for a in args {
                    match self.eval(a, ctx, key, true) {
                        Ok(Value::None) => continue,
                        Ok(v) => return Ok(v),
                        Err(EvalErr::Absent(p)) => last = Some(p),
                        Err(e) => return Err(e),
                    }
                }
                Err(EvalErr::Absent(last.unwrap_or_else(|| "first(…)".into())))
            }
            "exact" => self.eval(&args[0], ctx, key, false),
            _ => {
                let builtin = builtins::builtin(name).is_some();
                let mut values = Vec::with_capacity(args.len());
                for a in args {
                    match self.eval(a, ctx, key, true) {
                        Ok(v) => values.push(v),
                        // Look functions see an absent token as `none` and may fall back themselves.
                        Err(EvalErr::Absent(_)) if !builtin => values.push(Value::None),
                        Err(e) => return Err(e),
                    }
                }
                let result = if builtin {
                    builtins::call(name, &values)
                } else {
                    self.theme.tokens.call(name, &values, &self.token_cx(key))
                };
                result.map_err(|m| EvalErr::Error(expr.span, format!("{name}: {m}")))
            }
        }
    }

    fn path(&self, expr: &Expr, segs: &[String], ctx: Ctx<'_>, key: &KeyData, as_arg: bool) -> Result<Value, EvalErr> {
        let model = self.model();
        let target =
            names::lookup(model, Some(&self.theme.roots), ctx, segs).map_err(|m| EvalErr::Error(expr.span, m))?;
        let value = match target {
            Target::Keyword(v) => v,
            Target::Prop { part, prop } => {
                let Ctx::Prop { control, part: current, .. } = ctx else {
                    return Err(EvalErr::Error(expr.span, format!("\"{prop}\" is a property, not available here")));
                };
                let value = match part {
                    Some(p) => self.prop(control, p, &prop, key)?,
                    None => match self.prop(control, current, &prop, key)? {
                        Some(v) => Some(v),
                        None if current != 0 => self.prop(control, 0, &prop, key)?,
                        None => None,
                    },
                };
                value.unwrap_or(Value::None)
            }
            Target::OtherControl { control, variant, prop } => {
                let mut other = KeyData { control, attrs: Vec::new(), states: 0, scale: key.scale };
                if let Some(v) = variant {
                    other.attrs.push(("variant".into(), v));
                }
                self.prop(control, 0, &prop, &other)?.unwrap_or(Value::None)
            }
            Target::Const { scope, name, fields } => {
                let def = model.const_lookup(scope, &name).expect("lookup found it");
                self.enter(format!("const {name}"), expr.span)?;
                let v = self.eval(&def.expr, Ctx::Const { scope: def.scope }, key, false);
                self.leave();
                self.fields(expr, v?, &fields)?
            }
            Target::Token { path, fields } => {
                let v = self.token(expr, &path, key)?;
                self.fields(expr, v, &fields)?
            }
            Target::Native { path } => {
                let joined = path.join(".");
                self.theme.tokens.token(&joined, &self.token_cx(key)).ok_or(EvalErr::Absent(joined))?
            }
            Target::Unknown if as_arg && segs.len() == 1 => Value::Symbol(segs[0].as_str().into()),
            Target::Unknown => return Err(EvalErr::Error(expr.span, format!("unknown name \"{}\"", segs.join(".")))),
        };
        Ok(value)
    }

    fn token(&self, expr: &Expr, path: &str, key: &KeyData) -> Result<Value, EvalErr> {
        let model = self.model();
        let defs = &model.tokens.defs[path];
        let mode = &self.theme.mode;
        let best = defs
            .iter()
            .filter(|d| d.mode.as_deref().is_none_or(|m| m == mode))
            .filter(|d| d.attrs.iter().all(|a| key.attrs.contains(a)))
            .max_by_key(|d| (d.attrs.len(), d.layer, d.mode.is_some(), d.order));
        let Some(def) = best else {
            return Err(EvalErr::Absent(path.to_string()));
        };
        self.enter(format!("token {path}"), expr.span)?;
        let ctx = Ctx::Token { group: &def.group, self_path: path, scope: def.scope };
        let v = self.eval(&def.expr, ctx, key, false);
        self.leave();
        let v = v?;
        match def.kind {
            Some(kind) => v.coerce(kind).map_err(|v| {
                EvalErr::Error(def.span, format!("token {path} is declared {kind} but evaluated to {}", v.describe()))
            }),
            None => Ok(v),
        }
    }

    fn fields(&self, expr: &Expr, value: Value, fields: &[String]) -> Result<Value, EvalErr> {
        let mut v = value;
        for f in fields {
            v = match (&v, f.as_str()) {
                (Value::Text(t), "size") => Value::Number(t.size),
                (Value::Text(t), "line_height") => Value::Number(t.line_height),
                (Value::Text(t), "weight") => Value::Number(t.weight),
                _ => return Err(EvalErr::Error(expr.span, format!("{} has no field \"{f}\"", v.describe()))),
            };
        }
        Ok(v)
    }
}
