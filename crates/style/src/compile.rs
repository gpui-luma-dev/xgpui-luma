//! Compiling style files: declarations, rules, tokens, constants and load-time checks.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::builtins;
use crate::config::Config;
use crate::diag::{Code, Diagnostic, Diagnostics, Severity, Span, suggest};
use crate::model::{ConstDef, ConstScope, ControlDef, FileInfo, Model, PartDef, PropDecl, Rule, TokenDef, Tokens};
use crate::names::{self, Ctx, Target};
use crate::syntax::{self, Binding, ControlAst, Decl, Expr, ExprKind, Item, Top, TokenItem};
use crate::handles::{ControlId, FromValue, PartId, Prop};
use crate::value::Kind;

/// One style file: a name for diagnostics and the inspector, and its text.
#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    pub name: Arc<str>,
    pub text: Arc<str>,
}

impl Source {
    pub fn new(name: impl Into<Arc<str>>, text: impl Into<Arc<str>>) -> Self {
        Self { name: name.into(), text: text.into() }
    }

    /// Read a style file from disk (for import, hot reload and editing in the lab).
    pub fn from_path(path: impl AsRef<std::path::Path>) -> Result<Self, Diagnostic> {
        let path = path.as_ref();
        std::fs::read_to_string(path)
            .map(|text| Source::new(path.display().to_string(), text))
            .map_err(|e| Diagnostic::error(Code::Io, format!("cannot read {}: {e}", path.display())))
    }
}

/// Collects style files by layer, then compiles them.
pub struct StylesheetBuilder {
    config: Arc<Config>,
    sources: Vec<(String, Source)>,
}

impl StylesheetBuilder {
    /// Add files to a layer, e.g. `.layer("sdk", xgpui_luma::style::sources())`.
    pub fn layer(mut self, layer: &str, sources: impl IntoIterator<Item = Source>) -> Self {
        self.sources.extend(sources.into_iter().map(|s| (layer.to_string(), s)));
        self
    }

    pub fn source(mut self, layer: &str, source: Source) -> Self {
        self.sources.push((layer.to_string(), source));
        self
    }

    pub fn compile(self) -> Result<Stylesheet, Diagnostics> {
        Stylesheet::compile_sources(self.config, self.sources)
    }
}

/// Validated rules, declarations, tokens and constants for one set of files. Cheap to clone.
#[derive(Clone)]
pub struct Stylesheet {
    pub(crate) model: Arc<Model>,
}

impl Stylesheet {
    pub fn builder(config: Arc<Config>) -> StylesheetBuilder {
        StylesheetBuilder { config, sources: Vec::new() }
    }

    pub(crate) fn compile_sources(
        config: Arc<Config>,
        sources: Vec<(String, Source)>,
    ) -> Result<Stylesheet, Diagnostics> {
        let mut c = Compiler::new(config);
        c.run(sources);
        let diags = std::mem::take(&mut c.diags);
        if diags.iter().any(|d| d.severity == Severity::Error) {
            return Err(Diagnostics(diags));
        }
        Ok(Stylesheet { model: Arc::new(c.model) })
    }

    pub fn config(&self) -> &Arc<Config> {
        &self.model.config
    }

    /// The files this stylesheet was compiled from, with their layers.
    pub fn sources(&self) -> impl Iterator<Item = (&str, &Source)> {
        self.model.sources.iter().map(|(l, s)| (l.as_str(), s))
    }

    /// Compile again with one file replaced (same name) or added. The current stylesheet is unchanged.
    pub fn with_source(&self, layer: &str, source: Source) -> Result<Stylesheet, Diagnostics> {
        let mut sources = self.model.sources.clone();
        match sources.iter_mut().find(|(_, s)| s.name == source.name) {
            Some(slot) => *slot = (layer.to_string(), source),
            None => sources.push((layer.to_string(), source)),
        }
        Stylesheet::compile_sources(self.model.config.clone(), sources)
    }

    /// Every token path the stylesheet defines (in any mode or layer), sorted. Read values with
    /// [`crate::Theme::token`].
    pub fn token_paths(&self) -> Vec<&str> {
        let mut paths: Vec<&str> = self.model.tokens.defs.keys().map(String::as_str).collect();
        paths.sort_unstable();
        paths
    }

    /// A control handle, if the stylesheet declares the control.
    pub fn control(&self, name: &str) -> Option<ControlId> {
        self.model.control(name).map(|i| ControlId::named(&self.model.controls[i].name))
    }

    /// A part handle, if the control has the part.
    pub fn part(&self, control: ControlId, name: &str) -> Option<PartId> {
        let c = self.model.control(control.name())?;
        self.model.controls[c].part(name).map(|_| PartId::named(name))
    }

    /// A typed handle to a property of a control; fails if the property is unknown or of another kind.
    pub fn prop<T: FromValue>(&self, control: ControlId, name: &str) -> Result<Prop<T>, Diagnostic> {
        let Some(c) = self.model.control(control.name()) else {
            return Err(Diagnostic::error(Code::UnknownControl, format!("unknown control \"{control}\"")));
        };
        match self.model.prop_kind(c, name) {
            Some(kind) if T::KINDS.contains(&kind) => Ok(Prop::new(crate::handles::intern(name))),
            Some(kind) => Err(Diagnostic::error(
                Code::KindMismatch,
                format!("{control}.{name} is {kind}, which this handle's type cannot read"),
            )),
            None => Err(Diagnostic::error(
                Code::UnknownProperty,
                format!("\"{name}\" is not a core or declared property of {control}"),
            )),
        }
    }

    /// Control names, in declaration order.
    pub fn controls(&self) -> impl Iterator<Item = ControlId> + '_ {
        self.model.controls.iter().map(|c| ControlId::named(&c.name))
    }

    /// A control's parts, `root` first.
    pub fn parts(&self, control: ControlId) -> Vec<PartId> {
        self.model
            .control(control.name())
            .map_or_else(Vec::new, |i| self.model.controls[i].parts.iter().map(|p| PartId::named(&p.name)).collect())
    }

    /// The doc comment of a control, part (`"checkbox.indicator"`) or declared property (`"checkbox.glyph_size"`).
    pub fn doc(&self, path: &str) -> Option<&str> {
        let (control, rest) = path.split_once('.').unwrap_or((path, ""));
        let c = &self.model.controls[self.model.control(control)?];
        if rest.is_empty() {
            return c.doc.as_deref();
        }
        if let Some(p) = c.part(rest) {
            return c.parts[p].doc.as_deref();
        }
        c.props.get(rest).and_then(|d| d.doc.as_deref())
    }

    /// A property's kind for a control.
    pub fn kind(&self, control: ControlId, prop: &str) -> Option<Kind> {
        self.model.prop_kind(self.model.control(control.name())?, prop)
    }

    /// The config's declared modes, in declaration order (e.g. `["light", "dark"]`). Token files give values
    /// per mode with `mode <name> { … }`; a declared mode with no such block uses the shared values.
    pub fn modes(&self) -> Vec<String> {
        self.model.config.modes().map(str::to_string).collect()
    }
}

struct ParsedFile {
    index: u32,
    layer: usize,
    ast: syntax::FileAst,
}

struct Compiler {
    model: Model,
    diags: Vec<Diagnostic>,
    order: usize,
    /// The consts scope of each layer's top level.
    layer_scopes: Vec<usize>,
}

/// Where a rule is being written.
#[derive(Clone)]
struct RuleCx {
    control: usize,
    part: usize,
    attrs: Vec<(String, String)>,
    states: u32,
    layer: usize,
    scope: usize,
    declaring: bool,
    in_part: bool,
}

impl Compiler {
    fn new(config: Arc<Config>) -> Self {
        let layers = config.layers.len();
        let mut consts = vec![ConstScope { parent: None, entries: HashMap::new() }];
        let mut layer_scopes = vec![0];
        for _ in 1..layers {
            consts.push(ConstScope { parent: Some(0), entries: HashMap::new() });
            layer_scopes.push(consts.len() - 1);
        }
        Self {
            model: Model {
                config,
                files: Vec::new(),
                controls: Vec::new(),
                control_index: HashMap::new(),
                consts,
                tokens: Tokens::default(),
                native_refs: Vec::new(),
                look_calls: Vec::new(),
                sources: Vec::new(),
            },
            diags: Vec::new(),
            order: 0,
            layer_scopes,
        }
    }

    fn err(&mut self, span: Span, code: Code, message: impl Into<String>) {
        let file = self.model.files.get(span.file as usize).map(|f| (&f.name, &*f.text));
        self.diags.push(Diagnostic::at(Severity::Error, code, span, file, message));
    }

    fn next_order(&mut self) -> usize {
        self.order += 1;
        self.order
    }

    fn run(&mut self, sources: Vec<(String, Source)>) {
        self.model.sources = sources.clone();
        let mut parsed = Vec::new();
        for (layer_name, source) in sources {
            let index = self.model.files.len() as u32;
            let Some(layer) = self.model.config.layer_index(&layer_name) else {
                let known = self.model.config.layers.join(", ");
                self.diags.push(Diagnostic {
                    source: Some(source.name.clone()),
                    ..Diagnostic::error(Code::UnknownLayer, format!("unknown layer \"{layer_name}\" (layers: {known})"))
                });
                continue;
            };
            self.model.files.push(FileInfo { name: source.name.clone(), text: source.text.clone() });
            match syntax::parse(&source.text, index) {
                Ok(ast) => parsed.push(ParsedFile { index, layer, ast }),
                Err(mut d) => {
                    d.source = Some(source.name.clone());
                    self.diags.push(d);
                }
            }
        }
        // Lower layers first; files keep their given order within a layer.
        parsed.sort_by_key(|f| (f.layer, f.index));

        for file in &parsed {
            for item in &file.ast.items {
                if let Top::Const(bindings) = item {
                    let scope = self.layer_scopes[file.layer];
                    self.add_consts(scope, bindings, file.layer);
                }
            }
        }
        for file in &parsed {
            for item in &file.ast.items {
                if let Top::Tokens(items) = item {
                    let scope = self.layer_scopes[file.layer];
                    self.add_tokens(items, &mut Vec::new(), None, &[], scope, file.layer);
                }
            }
        }
        for file in &parsed {
            for item in &file.ast.items {
                if let Top::Control(ast) = item {
                    self.declare_control(ast, file.layer);
                }
            }
        }
        self.link_bases();
        for file in &parsed {
            for item in &file.ast.items {
                if let Top::Control(ast) = item {
                    self.add_rules(ast, file.layer);
                }
            }
        }
        self.check_aliases();
        self.check_names_and_kinds();
    }

    // ---- constants -------------------------------------------------------------------------

    fn add_consts(&mut self, scope: usize, bindings: &[Binding], layer: usize) {
        for b in bindings {
            let kind = b.kind.as_ref().and_then(|k| self.kind_of(k));
            if self.model.consts[scope].entries.contains_key(&b.name.text) {
                self.err(b.name.span, Code::Duplicate, format!("constant \"{}\" is defined twice", b.name.text));
                continue;
            }
            if layer > 0 && scope == self.layer_scopes[layer] && self.model.consts[0].entries.contains_key(&b.name.text)
            {
                self.err(b.name.span, Code::Duplicate, format!("\"{}\" redefines a global constant", b.name.text));
                continue;
            }
            self.model.consts[scope]
                .entries
                .insert(b.name.text.clone(), ConstDef { expr: b.value.clone(), scope, kind, span: b.name.span });
        }
    }

    fn child_scope(&mut self, parent: usize) -> usize {
        self.model.consts.push(ConstScope { parent: Some(parent), entries: HashMap::new() });
        self.model.consts.len() - 1
    }

    fn kind_of(&mut self, ident: &syntax::Ident) -> Option<Kind> {
        let kind = Kind::parse(&ident.text);
        if kind.is_none() {
            self.err(
                ident.span,
                Code::UnknownKind,
                format!(
                    "unknown kind \"{}\" (kinds: length, number, color, shadow, text, font, weight, icon, motion)",
                    ident.text
                ),
            );
        }
        kind
    }

    // ---- tokens ----------------------------------------------------------------------------

    fn add_tokens(
        &mut self,
        items: &[TokenItem],
        group: &mut Vec<String>,
        mode: Option<&str>,
        attrs: &[(String, String)],
        scope: usize,
        layer: usize,
    ) {
        // A block's constants are visible to the whole block.
        let mut scope = scope;
        if items.iter().any(|i| matches!(i, TokenItem::Const(_))) {
            scope = self.child_scope(scope);
            for item in items {
                if let TokenItem::Const(bindings) = item {
                    self.add_consts(scope, bindings, layer);
                }
            }
        }
        for item in items {
            match item {
                TokenItem::Const(_) => {}
                TokenItem::Mode { name, items } => {
                    if mode.is_some() {
                        self.err(name.span, Code::Syntax, "mode blocks cannot be nested");
                        continue;
                    }
                    if !self.model.config.modes.contains(&name.text) {
                        let modes: Vec<&str> = self.model.config.modes().collect();
                        let message = if modes.is_empty() {
                            format!("unknown mode \"{}\" (the config declares no modes)", name.text)
                        } else {
                            format!(
                                "unknown mode \"{}\" (modes: {}){}",
                                name.text,
                                modes.join(", "),
                                crate::diag::suggest(&name.text, modes.iter().copied())
                            )
                        };
                        self.err(name.span, Code::UnknownMode, message);
                        continue;
                    }
                    self.add_tokens(items, group, Some(&name.text), attrs, scope, layer);
                }
                TokenItem::Scope { attr, value, items } => {
                    if self.model.config.attribute_index(&attr.text).is_none() {
                        self.err(attr.span, Code::UnknownAttribute, format!("unknown attribute \"{}\"", attr.text));
                        continue;
                    }
                    let mut attrs = attrs.to_vec();
                    attrs.push((attr.text.clone(), value.text.clone()));
                    attrs.sort();
                    self.add_tokens(items, group, mode, &attrs, scope, layer);
                }
                TokenItem::Group { name, items, .. } => {
                    group.push(name.text.clone());
                    self.model.tokens.groups.insert(group.join("."));
                    self.add_tokens(items, group, mode, attrs, scope, layer);
                    group.pop();
                }
                TokenItem::Binding(b) => {
                    let kind = b.kind.as_ref().and_then(|k| self.kind_of(k));
                    let mut path = group.clone();
                    path.push(b.name.text.clone());
                    let order = self.next_order();
                    self.model.tokens.defs.entry(path.join(".")).or_default().push(TokenDef {
                        layer,
                        mode: mode.map(str::to_owned),
                        attrs: attrs.to_vec(),
                        expr: b.value.clone(),
                        group: group.clone(),
                        scope,
                        kind,
                        order,
                        span: b.name.span,
                    });
                }
            }
        }
    }

    // ---- controls: declarations ------------------------------------------------------------

    fn declare_control(&mut self, ast: &ControlAst, layer: usize) {
        let name = &ast.name.text;
        let index = match self.model.control(name) {
            Some(i) => i,
            None => {
                self.model.controls.push(ControlDef {
                    name: name.clone(),
                    doc: None,
                    declaring_layer: layer,
                    base: None,
                    base_name: None,
                    parts: vec![PartDef { name: "root".into(), doc: None }],
                    props: HashMap::new(),
                    rules: Vec::new(),
                    part_extends: HashMap::new(),
                    aliases: HashMap::new(),
                    scope_values: Vec::new(),
                });
                let i = self.model.controls.len() - 1;
                self.model.control_index.insert(name.clone(), i);
                i
            }
        };
        let declaring = self.model.controls[index].declaring_layer == layer;
        if declaring {
            if ast.doc.is_some() {
                self.model.controls[index].doc = ast.doc.clone();
            }
            if let Some(base) = &ast.extends {
                self.model.controls[index].base_name = Some((base.segs.join("."), base.span));
            }
        } else if let Some(base) = &ast.extends {
            self.err(
                base.span,
                Code::Declaration,
                format!("\"{name} extends …\" belongs in the layer that declares {name}"),
            );
        }
        self.declare_items(index, &ast.items, declaring);
    }

    fn declare_items(&mut self, control: usize, items: &[Item], declaring: bool) {
        for item in items {
            match item {
                Item::Part(p) => {
                    let existing = self.model.controls[control].part(&p.name.text);
                    let index = match existing {
                        Some(i) => i,
                        None if declaring => {
                            self.model.controls[control].parts.push(PartDef { name: p.name.text.clone(), doc: None });
                            self.model.controls[control].parts.len() - 1
                        }
                        // Checked later: the part may come from a base control.
                        None => continue,
                    };
                    if declaring && p.doc.is_some() {
                        self.model.controls[control].parts[index].doc = p.doc.clone();
                    }
                    self.declare_items(control, &p.items, declaring);
                }
                Item::Scope(s) => self.declare_items(control, &s.items, declaring),
                Item::States(s) => {
                    for d in &s.decls {
                        self.declare_decl(control, d, declaring);
                    }
                }
                Item::Decl(d) => self.declare_decl(control, d, declaring),
                Item::Const(_) => {}
            }
        }
    }

    fn declare_decl(&mut self, control: usize, d: &Decl, declaring: bool) {
        let Some(kind_ident) = &d.kind else { return };
        let name = &d.name.text;
        let cname = self.model.controls[control].name.clone();
        if !declaring {
            self.err(
                d.name.span,
                Code::Declaration,
                format!("\"{name}\" can only be declared in the layer that declares {cname}"),
            );
            return;
        }
        if builtins::core_property(name).is_some() {
            self.err(
                d.name.span,
                Code::Declaration,
                format!("\"{name}\" is a core property; its kind is fixed and needs no declaration"),
            );
            return;
        }
        if builtins::keyword(name).is_some() {
            self.err(d.name.span, Code::Declaration, format!("\"{name}\" is a keyword and cannot be a property name"));
            return;
        }
        let Some(kind) = self.kind_of(kind_ident) else { return };
        if let Some(prev) = self.model.controls[control].props.get(name) {
            if prev.kind != kind {
                self.err(d.name.span, Code::Duplicate, format!("\"{name}\" is already declared as {}", prev.kind));
            } else {
                self.err(d.name.span, Code::Duplicate, format!("\"{name}\" is declared twice"));
            }
            return;
        }
        self.model.controls[control].props.insert(name.clone(), PropDecl { kind, doc: d.doc.clone() });
    }

    fn link_bases(&mut self) {
        for i in 0..self.model.controls.len() {
            let Some((name, span)) = self.model.controls[i].base_name.clone() else {
                continue;
            };
            match self.model.control(&name) {
                Some(b) if b != i => self.model.controls[i].base = Some(b),
                _ => self.err(span, Code::UnknownControl, format!("unknown control \"{name}\" in extends")),
            }
        }
        // Cycles, then inherit base parts in dependency order.
        for i in 0..self.model.controls.len() {
            let mut seen = HashSet::new();
            let mut c = Some(i);
            while let Some(x) = c {
                if !seen.insert(x) {
                    let (name, span) = self.model.controls[i].base_name.clone().unwrap_or_default();
                    self.err(span, Code::Cycle, format!("extends cycle through \"{name}\""));
                    self.model.controls[i].base = None;
                    break;
                }
                c = self.model.controls[x].base;
            }
        }
        let mut done = vec![false; self.model.controls.len()];
        for i in 0..self.model.controls.len() {
            self.inherit_parts(i, &mut done, 0);
        }
    }

    fn inherit_parts(&mut self, i: usize, done: &mut [bool], depth: usize) {
        if done[i] || depth > 32 {
            return;
        }
        done[i] = true;
        if let Some(b) = self.model.controls[i].base {
            self.inherit_parts(b, done, depth + 1);
            let base_parts: Vec<String> = self.model.controls[b].parts.iter().map(|p| p.name.clone()).collect();
            for name in base_parts {
                if self.model.controls[i].part(&name).is_none() {
                    self.model.controls[i].parts.push(PartDef { name, doc: None });
                }
            }
        }
    }

    // ---- controls: rules -------------------------------------------------------------------

    fn add_rules(&mut self, ast: &ControlAst, layer: usize) {
        let Some(control) = self.model.control(&ast.name.text) else {
            return;
        };
        let declaring = self.model.controls[control].declaring_layer == layer;
        let cx = RuleCx {
            control,
            part: 0,
            attrs: Vec::new(),
            states: 0,
            layer,
            scope: self.layer_scopes[layer],
            declaring,
            in_part: false,
        };
        self.rule_items(&ast.items, cx);
    }

    fn block_scope(&mut self, items: &[Item], parent: usize, layer: usize) -> usize {
        if !items.iter().any(|i| matches!(i, Item::Const(_))) {
            return parent;
        }
        let scope = self.child_scope(parent);
        for item in items {
            if let Item::Const(bindings) = item {
                self.add_consts(scope, bindings, layer);
            }
        }
        scope
    }

    fn rule_items(&mut self, items: &[Item], mut cx: RuleCx) {
        cx.scope = self.block_scope(items, cx.scope, cx.layer);
        for item in items {
            match item {
                Item::Const(_) => {}
                Item::Part(p) => {
                    if cx.in_part {
                        self.err(p.name.span, Code::Syntax, "parts cannot be nested");
                        continue;
                    }
                    let control = &self.model.controls[cx.control];
                    let Some(part) = control.part(&p.name.text) else {
                        let parts = control.parts.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(", ");
                        let hint = suggest(&p.name.text, control.parts.iter().map(|p| p.name.as_str()));
                        let message =
                            format!("unknown part \"{}\" for {} (parts: {parts}){hint}", p.name.text, control.name);
                        self.err(p.name.span, Code::UnknownPart, message);
                        continue;
                    };
                    if let Some(target) = &p.extends {
                        self.part_extends(&cx, part, target);
                    }
                    let mut inner = cx.clone();
                    inner.part = part;
                    inner.in_part = true;
                    self.rule_items(&p.items, inner);
                }
                Item::Scope(s) => {
                    if self.model.config.attribute_index(&s.attr.text).is_none() {
                        let known = self.model.config.attributes.join(", ");
                        self.err(
                            s.attr.span,
                            Code::UnknownAttribute,
                            format!(
                                "unknown attribute \"{}\" (attributes: {known}){}",
                                s.attr.text,
                                suggest(&s.attr.text, self.model.config.attributes.iter().map(String::as_str))
                            ),
                        );
                        continue;
                    }
                    let pair = (s.attr.text.clone(), s.value.text.clone());
                    if let Some(base) = &s.extends {
                        let aliases = &mut self.model.controls[cx.control].aliases;
                        match aliases.get(&pair) {
                            Some(existing) if existing != &base.text => {
                                let message = format!("{} {} already extends {existing}", s.attr.text, s.value.text);
                                self.err(base.span, Code::Duplicate, message);
                            }
                            _ => {
                                aliases.insert(pair.clone(), base.text.clone());
                            }
                        }
                    }
                    let control = &mut self.model.controls[cx.control];
                    if !control.scope_values.contains(&pair) {
                        control.scope_values.push(pair.clone());
                    }
                    let mut inner = cx.clone();
                    inner.attrs.retain(|(a, _)| a != &pair.0);
                    inner.attrs.push(pair);
                    inner.attrs.sort();
                    self.rule_items(&s.items, inner);
                }
                Item::States(s) => {
                    let mut states = cx.states;
                    let mut ok = true;
                    for st in &s.states {
                        match self.model.config.state_index(&st.text) {
                            Some(i) => states |= 1 << i,
                            None => {
                                let known = self.model.config.states.join(", ");
                                self.err(
                                    st.span,
                                    Code::UnknownState,
                                    format!(
                                        "unknown state \":{}\" (states: {known}){}",
                                        st.text,
                                        suggest(&st.text, self.model.config.states.iter().map(String::as_str))
                                    ),
                                );
                                ok = false;
                            }
                        }
                    }
                    if !ok {
                        continue;
                    }
                    let mut inner = cx.clone();
                    inner.states = states;
                    for d in &s.decls {
                        self.rule_decl(d, &inner);
                    }
                }
                Item::Decl(d) => self.rule_decl(d, &cx),
            }
        }
    }

    fn part_extends(&mut self, cx: &RuleCx, part: usize, target: &syntax::PathRef) {
        if !cx.attrs.is_empty() || cx.states != 0 {
            self.err(
                target.span,
                Code::Extends,
                "a part can only extend another part outside variant, size and state blocks",
            );
            return;
        }
        let resolved = match target.segs.as_slice() {
            [p] => self.model.controls[cx.control].part(p).map(|i| (cx.control, i)),
            [c, p] => self.model.control(c).and_then(|ci| self.model.controls[ci].part(p).map(|pi| (ci, pi))),
            _ => None,
        };
        match resolved {
            Some(t) if t != (cx.control, part) => {
                self.model.controls[cx.control].part_extends.insert(part, (t.0, t.1, cx.layer));
            }
            _ => self.err(target.span, Code::Extends, format!("unknown part \"{}\" in extends", target.segs.join("."))),
        }
    }

    fn rule_decl(&mut self, d: &Decl, cx: &RuleCx) {
        let name = &d.name.text;
        let Some(kind) = self.model.prop_kind(cx.control, name) else {
            if d.kind.is_none() || !cx.declaring {
                let cname = &self.model.controls[cx.control].name;
                let hint = suggest(name, self.model.prop_names(cx.control));
                let message = format!("\"{name}\" is not a core or declared property of {cname}{hint}");
                self.err(d.name.span, Code::UnknownProperty, message);
            }
            return;
        };
        let order = self.next_order();
        self.model.controls[cx.control].rules.push(Rule {
            part: cx.part,
            prop: name.clone(),
            kind,
            attrs: cx.attrs.clone(),
            states: cx.states,
            layer: cx.layer,
            order,
            owning: d.owning,
            expr: d.value.clone(),
            scope: cx.scope,
            span: d.name.span,
        });
    }

    fn check_aliases(&mut self) {
        let mut problems = Vec::new();
        for c in &self.model.controls {
            for ((attr, value), base) in &c.aliases {
                let known = {
                    let mut ci = Some(c);
                    let mut found = false;
                    let mut guard = 0;
                    while let Some(x) = ci {
                        if x.scope_values.iter().any(|(a, v)| a == attr && v == base) {
                            found = true;
                            break;
                        }
                        ci = x.base.map(|b| &self.model.controls[b]);
                        guard += 1;
                        if guard > 32 {
                            break;
                        }
                    }
                    found
                };
                if !known {
                    let span = c.rules.first().map(|r| r.span).unwrap_or_default();
                    problems.push((
                        span,
                        Code::Extends,
                        format!("{attr} {value} extends unknown {attr} \"{base}\" in {}", c.name),
                    ));
                }
            }
        }
        for (span, code, message) in problems {
            self.err(span, code, message);
        }
    }

    // ---- names and kinds -------------------------------------------------------------------

    fn check_names_and_kinds(&mut self) {
        let mut checker = Checker { model: &self.model, problems: Vec::new(), native: Vec::new(), calls: Vec::new() };
        for (ci, control) in self.model.controls.iter().enumerate() {
            for rule in &control.rules {
                let ctx = Ctx::Prop { control: ci, part: rule.part, scope: rule.scope };
                let mut stack = Vec::new();
                if let Some(found) = checker.infer(&rule.expr, ctx, false, &mut stack)
                    && !rule.kind.accepts(found)
                {
                    checker.problems.push((
                        rule.expr.span,
                        Code::KindMismatch,
                        format!("{} expects {}, found {found} ({})", rule.prop, rule.kind, describe_expr(&rule.expr)),
                    ));
                }
            }
        }
        let mut token_paths: Vec<&String> = self.model.tokens.defs.keys().collect();
        token_paths.sort();
        for path in token_paths {
            let defs = &self.model.tokens.defs[path];
            let mut kinds: Vec<(Kind, Span)> = Vec::new();
            for def in defs {
                let ctx = Ctx::Token { group: &def.group, self_path: path, scope: def.scope };
                let mut stack = vec![format!("token {path}")];
                let found = checker.infer(&def.expr, ctx, false, &mut stack);
                if let (Some(declared), Some(found)) = (def.kind, found)
                    && !declared.accepts(found)
                {
                    checker.problems.push((
                        def.span,
                        Code::KindMismatch,
                        format!("token {path} is declared {declared} but its value is {found}"),
                    ));
                }
                if let Some(k) = def.kind.or(found) {
                    kinds.push((k, def.span));
                }
            }
            if let Some((first, _)) = kinds.first().copied() {
                for (k, span) in &kinds[1..] {
                    if !first.accepts(*k) || !k.accepts(first) {
                        checker.problems.push((
                            *span,
                            Code::KindMismatch,
                            format!("token {path} is {k} here but {first} elsewhere"),
                        ));
                    }
                }
            }
        }
        for scope in 0..self.model.consts.len() {
            let mut names: Vec<&String> = self.model.consts[scope].entries.keys().collect();
            names.sort();
            for name in names {
                let def = &self.model.consts[scope].entries[name];
                let mut stack = vec![format!("const {name}")];
                let found = checker.infer(&def.expr, Ctx::Const { scope: def.scope }, false, &mut stack);
                if let (Some(declared), Some(found)) = (def.kind, found)
                    && !declared.accepts(found)
                {
                    checker.problems.push((
                        def.span,
                        Code::KindMismatch,
                        format!("constant {name} is declared {declared} but its value is {found}"),
                    ));
                }
                if self.model.tokens.is_root(name) || self.model.control(name).is_some() {
                    checker.problems.push((
                        def.span,
                        Code::AmbiguousName,
                        format!("constant \"{name}\" shadows a token group or control"),
                    ));
                }
            }
        }
        let Checker { problems, native, calls, .. } = checker;
        self.model.native_refs = native;
        self.model.look_calls = calls;
        let mut seen = HashSet::new();
        for (span, code, message) in problems {
            if seen.insert((span, message.clone())) {
                self.err(span, code, message);
            }
        }
    }
}

fn describe_expr(expr: &Expr) -> String {
    match &expr.kind {
        ExprKind::Path(segs) => segs.join("."),
        ExprKind::Call(f, _) => format!("{}(…)", f.join(".")),
        ExprKind::Number(n) => n.to_string(),
        ExprKind::Hex(h) => h.clone(),
        ExprKind::Str(s) => format!("\"{s}\""),
        _ => "expression".into(),
    }
}

/// Static name and kind checks. Unknown kinds (native tokens, look functions) are checked later.
struct Checker<'m> {
    model: &'m Model,
    problems: Vec<(Span, Code, String)>,
    native: Vec<(String, Span)>,
    calls: Vec<(String, usize, Span)>,
}

impl Checker<'_> {
    fn infer(&mut self, expr: &Expr, ctx: Ctx<'_>, as_arg: bool, stack: &mut Vec<String>) -> Option<Kind> {
        match &expr.kind {
            ExprKind::Number(_) => Some(Kind::Number),
            ExprKind::Hex(h) => {
                if crate::color::Color::parse_hex(h).is_none() {
                    self.problems.push((expr.span, Code::Syntax, format!("invalid color {h}")));
                }
                Some(Kind::Color)
            }
            ExprKind::Str(_) => Some(Kind::Font),
            ExprKind::List(items) => {
                let mut all_shadow = true;
                for item in items {
                    match self.infer(item, ctx, false, stack) {
                        Some(Kind::Shadow) | None => {}
                        Some(_) => all_shadow = false,
                    }
                }
                all_shadow.then_some(Kind::Shadow)
            }
            ExprKind::Neg(inner) => {
                self.numeric(inner, ctx, stack);
                Some(Kind::Number)
            }
            ExprKind::Bin(_, a, b) => {
                self.numeric(a, ctx, stack);
                self.numeric(b, ctx, stack);
                Some(Kind::Number)
            }
            ExprKind::Index(base, index) => {
                self.numeric(index, ctx, stack);
                if let ExprKind::Path(segs) = &base.kind
                    && segs.len() == 1
                {
                    let local = names::lookup(self.model, None, ctx, segs).ok();
                    if matches!(local, None | Some(Target::Unknown)) {
                        self.native.push((segs[0].clone(), base.span));
                        return None;
                    }
                }
                self.infer(base, ctx, false, stack);
                None
            }
            ExprKind::Call(func, args) => self.call(expr, func, args, ctx, stack),
            ExprKind::Path(segs) => self.path(expr, segs, ctx, as_arg, stack),
        }
    }

    fn numeric(&mut self, expr: &Expr, ctx: Ctx<'_>, stack: &mut Vec<String>) {
        if let Some(k) = self.infer(expr, ctx, false, stack)
            && !Kind::Number.accepts(k)
        {
            self.problems.push((
                expr.span,
                Code::KindMismatch,
                format!("arithmetic needs numbers, found {k} ({})", describe_expr(expr)),
            ));
        }
    }

    fn call(
        &mut self,
        expr: &Expr,
        func: &[String],
        args: &[Expr],
        ctx: Ctx<'_>,
        stack: &mut Vec<String>,
    ) -> Option<Kind> {
        if func.len() != 1 {
            self.problems
                .push((expr.span, Code::UnknownFunction, format!("unknown function \"{}\"", func.join("."))));
            return None;
        }
        let name = &func[0];
        let kinds: Vec<Option<Kind>> = args.iter().map(|a| self.infer(a, ctx, true, stack)).collect();
        match builtins::builtin(name) {
            Some((min, max, ret)) => {
                if args.len() < min || args.len() > max {
                    let expected = if min == max {
                        min.to_string()
                    } else if max == usize::MAX {
                        format!("at least {min}")
                    } else {
                        format!("{min} to {max}")
                    };
                    self.problems.push((
                        expr.span,
                        Code::WrongArgumentCount,
                        format!("{name} expects {expected} arguments, found {}", args.len()),
                    ));
                }
                match name.as_str() {
                    "first" => kinds.iter().flatten().next().copied(),
                    "exact" => kinds.first().copied().flatten(),
                    _ => ret,
                }
            }
            None => {
                self.calls.push((name.clone(), args.len(), expr.span));
                None
            }
        }
    }

    fn path(
        &mut self,
        expr: &Expr,
        segs: &[String],
        ctx: Ctx<'_>,
        as_arg: bool,
        stack: &mut Vec<String>,
    ) -> Option<Kind> {
        let target = match names::lookup(self.model, None, ctx, segs) {
            Ok(t) => t,
            Err(message) => {
                self.problems.push((expr.span, Code::AmbiguousName, message));
                return None;
            }
        };
        match target {
            Target::Keyword(v) => v.kind(),
            Target::Prop { part, prop } => {
                let Ctx::Prop { control, .. } = ctx else { return None };
                let _ = part;
                self.model.prop_kind(control, &prop)
            }
            Target::OtherControl { control, scope_value, prop } => {
                if let Some(value) = &scope_value {
                    let name = &self.model.controls[control].name;
                    match self.model.scope_attribute(control, value) {
                        Ok(_) => {}
                        Err(attrs) if attrs.is_empty() => {
                            let values = self.model.controls[control].scope_values.iter().map(|(_, v)| v.as_str());
                            let hint = suggest(value, values);
                            self.problems.push((
                                expr.span,
                                Code::UnknownName,
                                format!("{name} has no scope \"{value}\"{hint}"),
                            ));
                        }
                        Err(attrs) => self.problems.push((
                            expr.span,
                            Code::AmbiguousName,
                            format!(
                                "\"{value}\" is a value of more than one attribute of {name} ({})",
                                attrs.join(", ")
                            ),
                        )),
                    }
                }
                let kind = self.model.prop_kind(control, &prop);
                if kind.is_none() {
                    let message = format!("{} has no property \"{prop}\"", self.model.controls[control].name);
                    self.problems.push((expr.span, Code::UnknownProperty, message));
                }
                kind
            }
            Target::Const { scope, name, fields } => {
                let label = format!("const {name}");
                if stack.contains(&label) {
                    self.problems.push((expr.span, Code::Cycle, format!("cycle: {} → {label}", stack.join(" → "))));
                    return None;
                }
                let def = self.model.const_lookup(scope, &name).expect("lookup found it");
                let (expr2, scope2, declared) = (def.expr.clone(), def.scope, def.kind);
                stack.push(label);
                let kind = declared.or_else(|| self.infer(&expr2, Ctx::Const { scope: scope2 }, false, stack));
                stack.pop();
                self.fields(expr, kind, &fields)
            }
            Target::Token { path, fields } => {
                let label = format!("token {path}");
                if stack.contains(&label) {
                    self.problems.push((expr.span, Code::Cycle, format!("cycle: {} → {label}", stack.join(" → "))));
                    return None;
                }
                let defs = &self.model.tokens.defs[&path];
                let def = defs.last().expect("token has a definition");
                let (expr2, group, scope2, declared) = (def.expr.clone(), def.group.clone(), def.scope, def.kind);
                stack.push(label);
                let kind = declared.or_else(|| {
                    self.infer(&expr2, Ctx::Token { group: &group, self_path: &path, scope: scope2 }, false, stack)
                });
                stack.pop();
                self.fields(expr, kind, &fields)
            }
            Target::Native { path } => {
                self.native.push((path[0].clone(), expr.span));
                None
            }
            Target::Unknown => {
                if !as_arg || segs.len() > 1 {
                    let what = if self.model.tokens.groups.contains(&segs.join(".")) {
                        format!("\"{}\" is a token group, not a value", segs.join("."))
                    } else {
                        let joined = segs.join(".");
                        let hint = if segs.len() == 1 {
                            match ctx {
                                Ctx::Prop { control, .. } => suggest(&joined, self.model.prop_names(control)),
                                _ => String::new(),
                            }
                        } else {
                            suggest(&joined, self.model.tokens.defs.keys().map(String::as_str))
                        };
                        format!("unknown name \"{joined}\"{hint}")
                    };
                    self.problems.push((expr.span, Code::UnknownName, what));
                }
                None
            }
        }
    }

    fn fields(&mut self, expr: &Expr, kind: Option<Kind>, fields: &[String]) -> Option<Kind> {
        match (kind, fields) {
            (k, []) => k,
            (Some(Kind::Text), [f]) if matches!(f.as_str(), "size" | "line_height" | "weight") => Some(Kind::Number),
            (Some(k), [f, ..]) => {
                self.problems.push((expr.span, Code::UnknownName, format!("{k} has no field \"{f}\"")));
                None
            }
            (None, _) => None,
        }
    }
}
