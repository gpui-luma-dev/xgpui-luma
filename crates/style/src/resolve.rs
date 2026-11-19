//! Binding to tokens, rebinding, the cascade (language §8) and resolution.

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::builtins;
use crate::color::Color;
use crate::compile::Stylesheet;
use crate::config::SnapPolicy;
use crate::diag::{Code, Diagnostic, Diagnostics, Severity, Span, suggest};
use crate::handles::{ControlId, FromValue, PartId, Prop, StyleKey, intern};
use crate::inspect::{Location, Reference, ReferenceKind};
use crate::model::{Model, Rule};
use crate::names::{self, Ctx, Target};
use crate::overrides::{OverrideValue, Overrides};
use crate::syntax::{BinOp, Expr, ExprKind};
use crate::token::{TokenCx, TokenSource};
use crate::value::{Kind, Motion, ShadowLayer, Value};

/// Resolved values for every part of one control instance.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Resolved {
    parts: BTreeMap<&'static str, BTreeMap<&'static str, Value>>,
}

impl Resolved {
    pub fn part(&self, part: PartId) -> PartView<'_> {
        PartView { values: self.parts.get(part.name()) }
    }

    pub fn root(&self) -> PartView<'_> {
        self.part(PartId::ROOT)
    }

    /// Parts that have at least one value.
    pub fn parts(&self) -> impl Iterator<Item = PartId> + '_ {
        self.parts.keys().map(|p| PartId::new(p))
    }

    /// Interpolate two resolved styles (e.g. unchecked → checked) for animation.
    pub fn lerp(a: &Resolved, b: &Resolved, t: f32) -> Resolved {
        let mut parts = BTreeMap::new();
        let names: HashSet<&&'static str> = a.parts.keys().chain(b.parts.keys()).collect();
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
                out.insert(*prop, v);
            }
            parts.insert(*name, out);
        }
        Resolved { parts }
    }
}

/// One part's values: typed reads through property handles (API topic 1).
#[derive(Clone, Copy, Debug)]
pub struct PartView<'a> {
    values: Option<&'a BTreeMap<&'static str, Value>>,
}

impl<'a> PartView<'a> {
    /// A property through its typed handle, e.g. `part.get(props::RADIUS)` or a declared property handle.
    pub fn get<T: FromValue>(&self, prop: Prop<T>) -> Option<T> {
        self.value(prop.name()).and_then(T::from_value)
    }

    /// The raw value by name, for tools.
    pub fn value(&self, name: &str) -> Option<&'a Value> {
        self.values.and_then(|v| v.get(name))
    }

    /// Property names with values, for tools.
    pub fn names(&self) -> impl Iterator<Item = &'static str> + 'a {
        self.values.into_iter().flat_map(|v| v.keys().copied())
    }

    pub fn background(&self) -> Option<Color> {
        self.value("background").and_then(Value::as_color)
    }
    pub fn foreground(&self) -> Option<Color> {
        self.value("foreground").and_then(Value::as_color)
    }
    pub fn border(&self) -> Option<Color> {
        self.value("border").and_then(Value::as_color)
    }
    pub fn focus_border(&self) -> Option<Color> {
        self.value("focus_border").and_then(Value::as_color)
    }
    pub fn radius(&self) -> Option<f32> {
        self.value("radius").and_then(Value::as_number)
    }
    pub fn height(&self) -> Option<f32> {
        self.value("height").and_then(Value::as_number)
    }
    pub fn gap(&self) -> Option<f32> {
        self.value("gap").and_then(Value::as_number)
    }
    pub fn font_size(&self) -> Option<f32> {
        self.value("font_size").and_then(Value::as_number)
    }
    pub fn line_height(&self) -> Option<f32> {
        self.value("line_height").and_then(Value::as_number)
    }
    pub fn font_family(&self) -> Option<&'a str> {
        self.value("font_family").and_then(Value::as_str)
    }
    pub fn shadow(&self) -> Option<&'a [ShadowLayer]> {
        self.value("shadow").and_then(Value::as_shadow)
    }
    pub fn transition(&self) -> Option<Motion> {
        self.value("transition").and_then(Value::as_motion)
    }
}

/// A stylesheet bound to native tokens and a mode. A theme never changes: every change returns a new
/// theme that shares the compiled stylesheet (API topic 4). Cheap to clone; caches resolved keys.
#[derive(Clone)]
pub struct Theme {
    pub(crate) inner: Arc<ThemeInner>,
}

pub(crate) struct ThemeInner {
    pub(crate) sheet: Stylesheet,
    pub(crate) tokens: Arc<dyn TokenSource>,
    pub(crate) mode: String,
    pub(crate) roots: HashSet<String>,
    pub(crate) token_values: HashMap<String, Value>,
    generation: u64,
    cache: Mutex<HashMap<CacheKey, Arc<Resolved>>>,
    pub(crate) diags: Mutex<Vec<Diagnostic>>,
}

impl ThemeInner {
    pub(crate) fn model(&self) -> &Model {
        &self.sheet.model
    }
}

fn next_generation() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

impl Theme {
    /// Bind a stylesheet to native tokens and a mode. The SDK alone binds with
    /// [`NoNativeTokens`](crate::NoNativeTokens) and mode `""`.
    pub fn bind(stylesheet: &Stylesheet, tokens: Arc<dyn TokenSource>, mode: &str) -> Result<Theme, Diagnostics> {
        Self::bind_with(stylesheet.clone(), tokens, mode, HashMap::new())
    }

    fn bind_with(
        sheet: Stylesheet,
        tokens: Arc<dyn TokenSource>,
        mode: &str,
        token_values: HashMap<String, Value>,
    ) -> Result<Theme, Diagnostics> {
        let model = sheet.model.clone();
        let roots: HashSet<String> = tokens.roots().iter().map(|r| r.to_string()).collect();
        let mut diags = Vec::new();
        let at = |span: Span, code: Code, message: String| {
            let file = model.files.get(span.file as usize).map(|f| (&f.name, &*f.text));
            Diagnostic::at(Severity::Error, code, span, file, message)
        };
        for (root, span) in &model.native_refs {
            if !roots.contains(root) {
                let hint = suggest(root, model.tokens.groups.iter().filter(|g| !g.contains('.')).map(String::as_str));
                diags.push(at(
                    *span,
                    Code::UnknownName,
                    format!("unknown name \"{root}\": not a token, constant or native token root{hint}"),
                ));
            }
        }
        for (name, args, span) in &model.look_calls {
            match tokens.function(name) {
                None => diags.push(at(*span, Code::UnknownFunction, format!("unknown function \"{name}\""))),
                Some(sig) if *args < sig.min_args || *args > sig.max_args => diags.push(at(
                    *span,
                    Code::WrongArgumentCount,
                    format!("{name} expects {} to {} arguments, found {args}", sig.min_args, sig.max_args),
                )),
                Some(_) => {}
            }
        }
        for root in &roots {
            if model.tokens.is_root(root) || model.control(root).is_some() || model.consts[0].entries.contains_key(root)
            {
                diags.push(Diagnostic::error(
                    Code::AmbiguousName,
                    format!("native token root \"{root}\" clashes with a token group, control or constant"),
                ));
            }
        }
        let modes = sheet.modes();
        if !modes.is_empty() && !modes.iter().any(|m| m == mode) {
            diags.push(Diagnostic::error(
                Code::UnknownMode,
                format!(
                    "unknown mode \"{mode}\" (modes: {}){}",
                    modes.join(", "),
                    suggest(mode, modes.iter().map(String::as_str))
                ),
            ));
        }
        if !diags.is_empty() {
            let mut seen = HashSet::new();
            diags.retain(|d| seen.insert((d.line, d.column, d.message.clone())));
            return Err(Diagnostics(diags));
        }
        Ok(Theme {
            inner: Arc::new(ThemeInner {
                sheet,
                tokens,
                mode: mode.to_string(),
                roots,
                token_values,
                generation: next_generation(),
                cache: Mutex::new(HashMap::new()),
                diags: Mutex::new(Vec::new()),
            }),
        })
    }

    /// The same stylesheet and tokens in another mode.
    pub fn with_mode(&self, mode: &str) -> Result<Theme, Diagnostics> {
        let i = &self.inner;
        Self::bind_with(i.sheet.clone(), i.tokens.clone(), mode, i.token_values.clone())
    }

    /// The same stylesheet with a different native token source (e.g. a newly loaded Shadcn CSS theme).
    pub fn with_tokens(&self, tokens: Arc<dyn TokenSource>) -> Result<Theme, Diagnostics> {
        let i = &self.inner;
        Self::bind_with(i.sheet.clone(), tokens, &i.mode, i.token_values.clone())
    }

    /// Live token edits: values for file tokens that rank above every layer, without recompiling.
    pub fn with_token_values(&self, values: &[(&str, Value)]) -> Result<Theme, Diagnostics> {
        let i = &self.inner;
        let model = i.model();
        let mut merged = i.token_values.clone();
        let mut problems = Vec::new();
        for (path, value) in values {
            if model.tokens.defs.contains_key(*path) {
                merged.insert(path.to_string(), value.clone());
            } else {
                let hint = suggest(path, model.tokens.defs.keys().map(String::as_str));
                problems.push(Diagnostic::error(Code::UnknownName, format!("unknown token \"{path}\"{hint}")));
            }
        }
        if !problems.is_empty() {
            return Err(Diagnostics(problems));
        }
        Self::bind_with(i.sheet.clone(), i.tokens.clone(), &i.mode, merged)
    }

    /// The same tokens and mode with another stylesheet (after edits, an import or a theme swap).
    pub fn with_stylesheet(&self, stylesheet: &Stylesheet) -> Result<Theme, Diagnostics> {
        let i = &self.inner;
        let values = i.token_values.iter().filter(|(p, _)| stylesheet.model.tokens.defs.contains_key(*p));
        Self::bind_with(
            stylesheet.clone(),
            i.tokens.clone(),
            &i.mode,
            values.map(|(p, v)| (p.clone(), v.clone())).collect(),
        )
    }

    pub fn stylesheet(&self) -> &Stylesheet {
        &self.inner.sheet
    }

    pub fn mode(&self) -> &str {
        &self.inner.mode
    }

    /// Changes with every new theme; tells consumers when to refresh.
    pub fn generation(&self) -> u64 {
        self.inner.generation
    }

    /// Live token edits in effect, sorted by path.
    pub fn token_values(&self) -> Vec<(String, Value)> {
        let mut v: Vec<_> = self.inner.token_values.iter().map(|(p, v)| (p.clone(), v.clone())).collect();
        v.sort_by(|a, b| a.0.cmp(&b.0));
        v
    }

    /// A key from names, for tools and tests. Templates build keys from handles instead.
    pub fn key_named(&self, control: &str, attrs: &[(&str, &str)], states: &[&str]) -> Result<StyleKey, Diagnostic> {
        let config = self.inner.model().config.clone();
        let Some(control) = self.inner.sheet.control(control) else {
            let hint = suggest(control, self.inner.model().controls.iter().map(|c| c.name.as_str()));
            return Err(Diagnostic::error(Code::UnknownControl, format!("unknown control \"{control}\"{hint}")));
        };
        let mut key = StyleKey::new(control);
        for (attr, value) in attrs {
            let Some(a) = config.attribute(attr) else {
                let hint = suggest(attr, config.attributes.iter().map(String::as_str));
                return Err(Diagnostic::error(Code::UnknownAttribute, format!("unknown attribute \"{attr}\"{hint}")));
            };
            key = key.attr(a, crate::handles::AttrValue::named(value));
        }
        for state in states {
            let Some(s) = config.state(state) else {
                let hint = suggest(state, config.states.iter().map(String::as_str));
                return Err(Diagnostic::error(Code::UnknownState, format!("unknown state \"{state}\"{hint}")));
            };
            key = key.state(s);
        }
        Ok(key)
    }

    /// Every part's values for one key. Problems leave a property unset and are recorded in
    /// [`Theme::diagnostics`]; resolving never fails.
    pub fn resolve(&self, key: &StyleKey) -> Arc<Resolved> {
        self.resolve_inner(key, None)
    }

    /// Like [`Theme::resolve`], with values set on this instance.
    pub fn resolve_with(&self, key: &StyleKey, overrides: &Overrides) -> Arc<Resolved> {
        self.resolve_inner(key, Some(overrides))
    }

    fn resolve_inner(&self, key: &StyleKey, overrides: Option<&Overrides>) -> Arc<Resolved> {
        let overrides = overrides.filter(|o| !o.is_empty());
        let ck = (*key, overrides.map_or_else(|| Arc::from(""), |o| o.content().clone()));
        if let Some(hit) = self.inner.cache.lock().expect("cache lock").get(&ck) {
            return hit.clone();
        }
        let resolved = Arc::new(self.compute(key, overrides));
        let mut cache = self.inner.cache.lock().expect("cache lock");
        if cache.len() >= MAX_CACHED {
            cache.clear();
        }
        cache.insert(ck, resolved.clone());
        resolved
    }

    /// Problems found while resolving so far.
    pub fn diagnostics(&self) -> Diagnostics {
        Diagnostics(self.inner.diags.lock().expect("diagnostics lock").clone())
    }

    /// Problems found since `mark` (a value returned by an earlier call; start with 0), and the new mark.
    pub fn diagnostics_since(&self, mark: usize) -> (Diagnostics, usize) {
        let all = self.inner.diags.lock().expect("diagnostics lock");
        let mark = mark.min(all.len());
        (Diagnostics(all[mark..].to_vec()), all.len())
    }

    pub(crate) fn record(&self, new: Vec<Diagnostic>) {
        if new.is_empty() {
            return;
        }
        let mut all = self.inner.diags.lock().expect("diagnostics lock");
        for d in new {
            if !all.contains(&d) {
                all.push(d);
            }
        }
    }

    pub(crate) fn key_data(&self, key: &StyleKey) -> Option<KeyData> {
        let model = self.inner.model();
        let Some(control) = model.control(key.control.name()) else {
            self.record(vec![Diagnostic::error(Code::UnknownControl, format!("unknown control \"{}\"", key.control))]);
            return None;
        };
        let attrs = key
            .attrs
            .iter()
            .enumerate()
            .filter_map(|(i, v)| v.map(|v| (intern(&model.config.attributes[i]), v.name())))
            .collect();
        Some(KeyData { control, attrs, states: model.config.normalize(key.states.0), scale: key.scale_factor() })
    }

    fn compute(&self, key: &StyleKey, overrides: Option<&Overrides>) -> Resolved {
        let Some(k) = self.key_data(key) else {
            return Resolved::default();
        };
        let model = self.inner.model();
        if let Some(o) = overrides {
            self.record(self.check_overrides(k.control, o));
        }
        let ev = Evaluator::new(&self.inner, k.control, overrides, false);
        let mut parts = BTreeMap::new();
        for (pi, part) in model.controls[k.control].parts.iter().enumerate() {
            let mut values = BTreeMap::new();
            for prop in props_of_part(model, k.control, pi, overrides) {
                match ev.output(k.control, pi, &prop, &k) {
                    Ok(Some(v)) => {
                        values.insert(intern(&prop), v);
                    }
                    Ok(None) => {}
                    Err(e) => ev.report_err(e, &model.controls[k.control].name, &part.name, &prop),
                }
            }
            if !values.is_empty() {
                parts.insert(intern(&part.name), values);
            }
        }
        self.record(ev.diags.into_inner());
        Resolved { parts }
    }

    /// Report overrides for parts or properties the control does not have.
    pub(crate) fn check_overrides(&self, control: usize, overrides: &Overrides) -> Vec<Diagnostic> {
        let model = self.inner.model();
        let def = &model.controls[control];
        let mut problems = Vec::new();
        for (part, prop, _) in overrides.iter() {
            if def.part(part).is_none() {
                let hint = suggest(part, def.parts.iter().map(|p| p.name.as_str()));
                problems.push(Diagnostic {
                    severity: Severity::Warning,
                    ..Diagnostic::error(
                        Code::UnknownPart,
                        format!("instance override: unknown part \"{part}\" for {}{hint}", def.name),
                    )
                });
            } else if model.prop_kind(control, prop).is_none() {
                let hint = suggest(prop, model.prop_names(control));
                problems.push(Diagnostic {
                    severity: Severity::Warning,
                    ..Diagnostic::error(
                        Code::UnknownProperty,
                        format!(
                            "instance override: \"{prop}\" is not a core or declared property of {}{hint}",
                            def.name
                        ),
                    )
                });
            }
        }
        problems
    }

    /// The control handle for a key's control index (for inspection).
    pub(crate) fn control_id(&self, control: usize) -> ControlId {
        ControlId::named(&self.inner.model().controls[control].name)
    }
}

/// Properties to resolve for a part: any rule's (through `extends`), inherited text properties, overrides.
pub(crate) fn props_of_part(model: &Model, control: usize, part: usize, overrides: Option<&Overrides>) -> Vec<String> {
    let mut props: Vec<String> = Vec::new();
    collect_props(model, control, part, &mut props, 0);
    if part != 0 {
        let mut root_props = Vec::new();
        collect_props(model, control, 0, &mut root_props, 0);
        for p in root_props {
            if builtins::core_property(&p).is_some_and(|(_, inherits)| inherits) && !props.contains(&p) {
                props.push(p);
            }
        }
    }
    if let Some(o) = overrides {
        let part_name = &model.controls[control].parts[part].name;
        for (p, prop, _) in o.iter() {
            if p == part_name && model.prop_kind(control, prop).is_some() && !props.iter().any(|x| x == prop) {
                props.push(prop.to_string());
            }
        }
    }
    props
}

pub(crate) fn expr_end(expr: &Expr) -> u32 {
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
pub(crate) fn collect_props(model: &Model, control: usize, part: usize, out: &mut Vec<String>, depth: usize) {
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
pub(crate) struct KeyData {
    pub control: usize,
    pub attrs: Vec<(&'static str, &'static str)>,
    pub states: u32,
    pub scale: f32,
}

/// A matching rule with how it was reached.
pub(crate) struct Cand<'m> {
    pub rule: &'m Rule,
    /// The rule's layer, raised to the layer of the `extends` it was reached through.
    pub layer: usize,
    /// 0 for the control's own rules; more for rules reached through `extends`.
    pub depth: u32,
    pub interaction: bool,
    pub via: Option<String>,
}

impl Cand<'_> {
    /// Language §8.3: state count, interaction state, attribute count, layer, own over inherited, order.
    /// A rule reached through `extends` counts at least at the layer where the `extends` was written.
    pub fn rank(&self) -> (u32, bool, usize, usize, i64, usize) {
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

pub(crate) struct Selection<'m> {
    pub kept: Vec<Cand<'m>>,
    pub owner: Option<Cand<'m>>,
}

pub(crate) fn select<'m>(model: &'m Model, control: usize, part: usize, prop: &str, key: &KeyData) -> Selection<'m> {
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

/// A key and the exact contents of its override set (empty for none).
type CacheKey = (StyleKey, Arc<str>);

/// A span with no file: for values that come from Rust (instance overrides), not from a style file.
const NO_SPAN: Span = Span { file: u32::MAX, start: 0, end: 0, line: 0, column: 0 };

/// Most cached resolved keys per theme; the cache is cleared when it is full.
const MAX_CACHED: usize = 4096;

pub(crate) enum EvalErr {
    /// A native token is missing; `first()` skips these.
    Absent(String),
    Error(Span, Code, String),
}

/// Evaluates expressions for one control instance. With tracing on, records what each value read.
pub(crate) struct Evaluator<'t> {
    theme: &'t ThemeInner,
    instance: usize,
    overrides: Option<&'t Overrides>,
    stack: RefCell<Vec<String>>,
    pub(crate) diags: RefCell<Vec<Diagnostic>>,
    trace: Option<RefCell<Vec<Vec<Reference>>>>,
}

impl<'t> Evaluator<'t> {
    pub(crate) fn new(theme: &'t ThemeInner, instance: usize, overrides: Option<&'t Overrides>, trace: bool) -> Self {
        Self {
            theme,
            instance,
            overrides,
            stack: RefCell::new(Vec::new()),
            diags: RefCell::new(Vec::new()),
            trace: trace.then(|| RefCell::new(vec![Vec::new()])),
        }
    }

    fn model(&self) -> &'t Model {
        self.theme.model()
    }

    /// The references recorded at the top level (tracing only).
    pub(crate) fn take_references(&self) -> Vec<Reference> {
        self.trace.as_ref().map(|t| std::mem::take(&mut t.borrow_mut()[0])).unwrap_or_default()
    }

    pub(crate) fn location(&self, span: Span) -> Option<Location> {
        self.model().files.get(span.file as usize).map(|f| Location {
            source: f.name.clone(),
            line: span.line,
            column: span.column,
        })
    }

    fn trace_push(&self) {
        if let Some(t) = &self.trace {
            t.borrow_mut().push(Vec::new());
        }
    }

    fn trace_pop(
        &self,
        name: String,
        kind: ReferenceKind,
        value: Option<&Value>,
        at: Option<Span>,
        description: Option<String>,
    ) {
        if let Some(t) = &self.trace {
            let mut t = t.borrow_mut();
            let children = t.pop().unwrap_or_default();
            let reference = Reference {
                name,
                kind,
                value: value.cloned(),
                defined_at: at.and_then(|s| self.location(s)),
                description,
                children,
            };
            if let Some(parent) = t.last_mut() {
                parent.push(reference);
            }
        }
    }

    pub(crate) fn report_err(&self, err: EvalErr, control: &str, part: &str, prop: &str) {
        match err {
            EvalErr::Absent(path) => {
                self.report(None, Code::AbsentToken, format!("{control}.{part}.{prop}: token \"{path}\" is absent"))
            }
            EvalErr::Error(span, code, message) => self.report(Some(span), code, message),
        }
    }

    fn report(&self, span: Option<Span>, code: Code, message: String) {
        let model = self.model();
        let d = match span {
            Some(s) => {
                let file = model.files.get(s.file as usize).map(|f| (&f.name, &*f.text));
                Diagnostic::at(Severity::Warning, code, s, file, message)
            }
            None => Diagnostic { severity: Severity::Warning, ..Diagnostic::error(code, message) },
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
            return Err(EvalErr::Error(span, Code::Cycle, format!("cycle: {chain} → {label}")));
        }
        stack.push(label);
        Ok(())
    }

    fn leave(&self) {
        self.stack.borrow_mut().pop();
    }

    /// The value of `part.prop` for `key` (language §8), or `None` when unset; plus where it was set.
    pub(crate) fn prop_at(
        &self,
        control: usize,
        part: usize,
        prop: &str,
        key: &KeyData,
    ) -> Result<(Option<Value>, Option<Span>), EvalErr> {
        let model = self.model();
        let kind = model.prop_kind(control, prop);
        let part_name = &model.controls[control].parts[part].name;
        let label = format!("{}.{part_name}.{prop}", model.controls[control].name);

        // Instance overrides rank above every layer and own their property in all states.
        // Overrides for unknown properties are reported once per resolve and ignored (see `compute`).
        if control == self.instance
            && let Some(kind) = kind
            && let Some(value) = self.overrides.and_then(|o| o.get(part_name, prop))
        {
            // Overrides come from Rust, not a style file, so their problems have no file location.
            let context = format!("instance override for {part_name}.{prop}");
            self.enter(label, NO_SPAN)?;
            let result = match value {
                OverrideValue::Value(v) => Ok(v.clone()),
                OverrideValue::Token(path) => {
                    let expr =
                        Expr { kind: ExprKind::Path(path.split('.').map(str::to_owned).collect()), span: NO_SPAN };
                    self.eval(&expr, Ctx::Const { scope: 0 }, key, false)
                }
            };
            self.leave();
            let value = result.map_err(|e| match e {
                EvalErr::Error(span, code, message) => EvalErr::Error(span, code, format!("{context}: {message}")),
                absent => absent,
            })?;
            return match value.coerce(kind) {
                Ok(Value::None) => Ok((None, None)),
                Ok(v) => Ok((Some(v), None)),
                Err(v) => Err(EvalErr::Error(
                    NO_SPAN,
                    Code::KindMismatch,
                    format!("{context}: expects {kind}, found {}", v.describe()),
                )),
            };
        }

        let sel = select(model, control, part, prop, key);
        let Some(winner) = sel.kept.iter().max_by_key(|c| c.rank()) else {
            let inherits = builtins::core_property(prop).is_some_and(|(_, i)| i);
            return if part != 0 && inherits {
                self.prop_at(control, 0, prop, key)
            } else {
                Ok((None, None))
            };
        };
        self.enter(label, winner.rule.span)?;
        let ctx = Ctx::Prop { control, part, scope: winner.rule.scope };
        let result = self.eval(&winner.rule.expr, ctx, key, false);
        self.leave();
        let value = result?;
        let kind = kind.unwrap_or(winner.rule.kind);
        match value.coerce(kind) {
            Ok(Value::None) => Ok((None, Some(winner.rule.span))),
            Ok(v) => Ok((Some(v), Some(winner.rule.span))),
            Err(v) => Err(EvalErr::Error(
                winner.rule.expr.span,
                Code::KindMismatch,
                format!("{prop} expects {kind}, found {}", v.describe()),
            )),
        }
    }

    pub(crate) fn prop(
        &self,
        control: usize,
        part: usize,
        prop: &str,
        key: &KeyData,
    ) -> Result<Option<Value>, EvalErr> {
        self.prop_at(control, part, prop, key).map(|(v, _)| v)
    }

    /// The value as a template receives it: lengths snapped to device pixels unless the winning value
    /// is `exact(…)`. References between properties use unsnapped values.
    pub(crate) fn output(
        &self,
        control: usize,
        part: usize,
        prop: &str,
        key: &KeyData,
    ) -> Result<Option<Value>, EvalErr> {
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
        let err = |code: Code, message: String| EvalErr::Error(expr.span, code, message);
        match &expr.kind {
            ExprKind::Number(n) => Ok(Value::Number(*n)),
            ExprKind::Hex(h) => {
                Color::parse_hex(h).map(Value::Color).ok_or_else(|| err(Code::Syntax, format!("invalid color {h}")))
            }
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
                other => Err(err(Code::KindMismatch, format!("cannot negate {}", other.describe()))),
            },
            ExprKind::Bin(op, a, b) => {
                let (a, b) = (self.eval(a, ctx, key, false)?, self.eval(b, ctx, key, false)?);
                match (a, b) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Number(match op {
                        BinOp::Add => a + b,
                        BinOp::Sub => a - b,
                        BinOp::Mul => a * b,
                        BinOp::Div if b == 0.0 => return Err(err(Code::Evaluation, "division by zero".into())),
                        BinOp::Div => a / b,
                    })),
                    (a, b) => Err(err(
                        Code::KindMismatch,
                        format!("arithmetic needs numbers, found {} and {}", a.describe(), b.describe()),
                    )),
                }
            }
            ExprKind::Index(base, index) => {
                let n = match self.eval(index, ctx, key, false)? {
                    Value::Number(n) => n as i64,
                    other => {
                        return Err(err(
                            Code::KindMismatch,
                            format!("an index must be a number, found {}", other.describe()),
                        ));
                    }
                };
                if let ExprKind::Path(segs) = &base.kind
                    && segs.len() == 1
                    && self.theme.roots.contains(&segs[0])
                {
                    self.trace_push();
                    let name = format!("{}[{n}]", segs[0]);
                    let value = self.theme.tokens.index(&segs[0], n, &self.token_cx(key));
                    self.trace_pop(name.clone(), ReferenceKind::Native, value.as_ref(), None, None);
                    return value.ok_or(EvalErr::Absent(name));
                }
                match self.eval(base, ctx, key, false)? {
                    Value::List(items) => items
                        .get(n as usize)
                        .cloned()
                        .ok_or_else(|| err(Code::Evaluation, format!("index {n} is out of range"))),
                    Value::Shadow(layers) => layers
                        .get(n as usize)
                        .map(|l| Value::Shadow(vec![*l].into()))
                        .ok_or_else(|| err(Code::Evaluation, format!("index {n} is out of range"))),
                    other => Err(err(Code::KindMismatch, format!("cannot index {}", other.describe()))),
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
                result.map_err(|m| EvalErr::Error(expr.span, Code::Evaluation, format!("{name}: {m}")))
            }
        }
    }

    fn path(&self, expr: &Expr, segs: &[String], ctx: Ctx<'_>, key: &KeyData, as_arg: bool) -> Result<Value, EvalErr> {
        let model = self.model();
        let target = names::lookup(model, Some(&self.theme.roots), ctx, segs)
            .map_err(|m| EvalErr::Error(expr.span, Code::AmbiguousName, m))?;
        let name = segs.join(".");
        let value = match target {
            Target::Keyword(v) => v,
            Target::Prop { part, prop } => {
                let Ctx::Prop { control, part: current, .. } = ctx else {
                    return Err(EvalErr::Error(
                        expr.span,
                        Code::UnknownName,
                        format!("\"{prop}\" is a property, not available here"),
                    ));
                };
                self.trace_push();
                let result = match part {
                    Some(p) => self.prop_at(control, p, &prop, key),
                    None => match self.prop_at(control, current, &prop, key) {
                        Ok((Some(v), at)) => Ok((Some(v), at)),
                        Ok((None, _)) if current != 0 => self.prop_at(control, 0, &prop, key),
                        other => other,
                    },
                };
                let (value, at) = match result {
                    Ok(r) => r,
                    Err(e) => {
                        self.trace_pop(name, ReferenceKind::Property, None, None, None);
                        return Err(e);
                    }
                };
                self.trace_pop(name, ReferenceKind::Property, value.as_ref(), at, None);
                value.unwrap_or(Value::None)
            }
            Target::OtherControl { control, scope_value, prop } => {
                let mut other = KeyData { control, attrs: Vec::new(), states: 0, scale: key.scale };
                if let Some(v) = scope_value {
                    let attr = model.scope_attribute(control, &v).map_err(|_| {
                        EvalErr::Error(expr.span, Code::UnknownName, format!("no single scope \"{v}\" in this control"))
                    })?;
                    other.attrs.push((intern(&attr), intern(&v)));
                }
                self.trace_push();
                let result = self.prop_at(control, 0, &prop, &other);
                let (value, at) = match result {
                    Ok(r) => r,
                    Err(e) => {
                        self.trace_pop(name, ReferenceKind::Property, None, None, None);
                        return Err(e);
                    }
                };
                self.trace_pop(name, ReferenceKind::Property, value.as_ref(), at, None);
                value.unwrap_or(Value::None)
            }
            Target::Const { scope, name: cname, fields } => {
                let def = model.const_lookup(scope, &cname).expect("lookup found it");
                self.enter(format!("const {cname}"), expr.span)?;
                self.trace_push();
                let v = self.eval(&def.expr, Ctx::Const { scope: def.scope }, key, false);
                self.leave();
                let v = v.and_then(|v| self.fields(expr, v, &fields));
                self.trace_pop(name, ReferenceKind::Constant, v.as_ref().ok(), Some(def.span), None);
                v?
            }
            Target::Token { path, fields } => {
                self.trace_push();
                let r = self.token(expr, &path, key);
                let (v, at) = match r {
                    Ok((v, at)) => (self.fields(expr, v, &fields), at),
                    Err(e) => (Err(e), None),
                };
                self.trace_pop(name, ReferenceKind::Token, v.as_ref().ok(), at, None);
                v?
            }
            Target::Native { path } => {
                let joined = path.join(".");
                self.trace_push();
                let cx = self.token_cx(key);
                let value = self.theme.tokens.token(&joined, &cx);
                let description = self.theme.tokens.describe(&joined, &cx);
                self.trace_pop(joined.clone(), ReferenceKind::Native, value.as_ref(), None, description);
                value.ok_or(EvalErr::Absent(joined))?
            }
            Target::Unknown if as_arg && segs.len() == 1 => Value::Symbol(segs[0].as_str().into()),
            Target::Unknown => {
                return Err(EvalErr::Error(expr.span, Code::UnknownName, format!("unknown name \"{name}\"")));
            }
        };
        Ok(value)
    }

    fn token(&self, expr: &Expr, path: &str, key: &KeyData) -> Result<(Value, Option<Span>), EvalErr> {
        if let Some(v) = self.theme.token_values.get(path) {
            return Ok((v.clone(), None));
        }
        let model = self.model();
        let defs = &model.tokens.defs[path];
        let mode = &self.theme.mode;
        let best = defs
            .iter()
            .filter(|d| d.mode.as_deref().is_none_or(|m| m == mode))
            .filter(|d| d.attrs.iter().all(|(a, v)| key.attrs.iter().any(|(ka, kv)| ka == a && kv == v)))
            .max_by_key(|d| (d.attrs.len(), d.layer, d.mode.is_some(), d.order));
        let Some(def) = best else {
            return Err(EvalErr::Absent(path.to_string()));
        };
        self.enter(format!("token {path}"), expr.span)?;
        let ctx = Ctx::Token { group: &def.group, self_path: path, scope: def.scope };
        let v = self.eval(&def.expr, ctx, key, false);
        self.leave();
        let v = v?;
        let v = match def.kind {
            Some(kind) => v.coerce(kind).map_err(|v| {
                EvalErr::Error(
                    def.span,
                    Code::KindMismatch,
                    format!("token {path} is declared {kind} but evaluated to {}", v.describe()),
                )
            })?,
            None => v,
        };
        Ok((v, Some(def.span)))
    }

    fn fields(&self, expr: &Expr, value: Value, fields: &[String]) -> Result<Value, EvalErr> {
        let mut v = value;
        for f in fields {
            v = match (&v, f.as_str()) {
                (Value::Text(t), "size") => Value::Number(t.size),
                (Value::Text(t), "line_height") => Value::Number(t.line_height),
                (Value::Text(t), "weight") => Value::Number(t.weight),
                _ => {
                    return Err(EvalErr::Error(
                        expr.span,
                        Code::UnknownName,
                        format!("{} has no field \"{f}\"", v.describe()),
                    ));
                }
            };
        }
        Ok(v)
    }
}
