//! Syntax tree for `.lss` files and the pest-based parser that builds it.

use std::sync::{Arc, OnceLock};

use pest::Parser as _;
use pest::error::LineColLocation;
use pest::iterators::Pair;
use pest::pratt_parser::{Assoc, Op, PrattParser};

use crate::diag::{Code, Diagnostic, Span};

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
struct LssParser;

#[derive(Clone, Debug)]
pub(crate) struct Ident {
    pub text: String,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub(crate) struct FileAst {
    pub items: Vec<Top>,
}

#[derive(Clone, Debug)]
pub(crate) enum Top {
    Const(Vec<Binding>),
    Tokens(Vec<TokenItem>),
    Control(ControlAst),
}

#[derive(Clone, Debug)]
pub(crate) struct Binding {
    /// Not used yet; kept for inspection docs on tokens and constants.
    #[allow(dead_code)]
    pub doc: Option<String>,
    pub name: Ident,
    pub kind: Option<Ident>,
    pub value: Arc<Expr>,
}

#[derive(Clone, Debug)]
pub(crate) enum TokenItem {
    Mode { name: Ident, items: Vec<TokenItem> },
    Const(Vec<Binding>),
    Scope { attr: Ident, value: Ident, items: Vec<TokenItem> },
    Group { name: Ident, items: Vec<TokenItem> },
    Binding(Binding),
}

#[derive(Clone, Debug)]
pub(crate) struct ControlAst {
    pub doc: Option<String>,
    pub name: Ident,
    pub extends: Option<PathRef>,
    pub items: Vec<Item>,
}

#[derive(Clone, Debug)]
pub(crate) enum Item {
    Const(Vec<Binding>),
    Part(PartAst),
    States(StatesAst),
    Scope(ScopeAst),
    Decl(Decl),
}

#[derive(Clone, Debug)]
pub(crate) struct PartAst {
    pub doc: Option<String>,
    pub name: Ident,
    pub extends: Option<PathRef>,
    /// Only `Item::States` and `Item::Decl`.
    pub items: Vec<Item>,
}

#[derive(Clone, Debug)]
pub(crate) struct ScopeAst {
    pub attr: Ident,
    pub value: Ident,
    pub extends: Option<Ident>,
    pub items: Vec<Item>,
}

#[derive(Clone, Debug)]
pub(crate) struct StatesAst {
    pub states: Vec<Ident>,
    pub decls: Vec<Decl>,
}

#[derive(Clone, Debug)]
pub(crate) struct Decl {
    pub doc: Option<String>,
    pub name: Ident,
    pub kind: Option<Ident>,
    pub owning: bool,
    pub value: Arc<Expr>,
}

#[derive(Clone, Debug)]
pub(crate) struct PathRef {
    pub segs: Vec<String>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Debug)]
pub(crate) struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub(crate) enum ExprKind {
    Number(f32),
    Hex(String),
    Str(String),
    Path(Vec<String>),
    Call(Vec<String>, Vec<Expr>),
    List(Vec<Expr>),
    Index(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
    Bin(BinOp, Box<Expr>, Box<Expr>),
}

impl Expr {
    /// True for `exact(…)` at the top of an expression (opts out of pixel snapping).
    pub fn is_exact(&self) -> bool {
        matches!(&self.kind, ExprKind::Call(f, _) if f.len() == 1 && f[0] == "exact")
    }
}

/// Parse one file. `file` is the index used in spans.
pub(crate) fn parse(text: &str, file: u32) -> Result<FileAst, Diagnostic> {
    let mut pairs = LssParser::parse(Rule::file, text).map_err(|err| syntax_error(err, text))?;
    let file_pair = pairs.next().expect("the file rule always produces one pair");
    let cx = Cx { file };
    let mut items = Vec::new();
    for pair in file_pair.into_inner() {
        match pair.as_rule() {
            Rule::const_block => items.push(Top::Const(cx.bindings(pair))),
            Rule::tokens_block => items.push(Top::Tokens(cx.token_items(pair.into_inner().collect()))),
            Rule::control => items.push(Top::Control(cx.control(pair))),
            Rule::EOI => {}
            other => unreachable!("unexpected top-level rule {other:?}"),
        }
    }
    Ok(FileAst { items })
}

fn syntax_error(err: pest::error::Error<Rule>, text: &str) -> Diagnostic {
    let (line, column) = match err.line_col {
        LineColLocation::Pos(pos) | LineColLocation::Span(pos, _) => pos,
    };
    let err = err.renamed_rules(rule_name);
    let message = match &err.variant {
        pest::error::ErrorVariant::ParsingError { positives, .. } if !positives.is_empty() => {
            format!("syntax error: expected {}", positives.iter().map(|r| rule_name(r)).collect::<Vec<_>>().join(", "))
        }
        variant => format!("syntax error: {}", variant.message()),
    };
    let mut d = Diagnostic::error(Code::Syntax, message);
    d.line = line as u32;
    d.column = column as u32;
    d.snippet = text.lines().nth(line.saturating_sub(1)).map(str::to_owned);
    d
}

fn rule_name(rule: &Rule) -> String {
    match rule {
        Rule::expr | Rule::postfix => "a value".into(),
        Rule::ident | Rule::path => "a name".into(),
        Rule::key => "a key".into(),
        Rule::assign => "\"=\" or \":=\"".into(),
        Rule::decl => "a property".into(),
        Rule::state => "a state like :hover".into(),
        Rule::EOI => "end of file".into(),
        Rule::kind_ann => "\":\" and a kind".into(),
        other => format!("{other:?}"),
    }
}

struct Cx {
    file: u32,
}

impl Cx {
    fn span(&self, pair: &Pair<'_, Rule>) -> Span {
        let (line, column) = pair.line_col();
        let s = pair.as_span();
        Span {
            file: self.file,
            start: s.start() as u32,
            end: s.end() as u32,
            line: line as u32,
            column: column as u32,
        }
    }

    fn ident(&self, pair: Pair<'_, Rule>) -> Ident {
        Ident { span: self.span(&pair), text: pair.as_str().to_owned() }
    }

    fn path(&self, pair: Pair<'_, Rule>) -> PathRef {
        PathRef { span: self.span(&pair), segs: pair.as_str().split('.').map(str::to_owned).collect() }
    }

    fn doc(&self, docs: &mut Vec<String>, pair: &Pair<'_, Rule>) -> bool {
        if pair.as_rule() == Rule::doc {
            docs.push(pair.as_str().trim_start_matches('/').trim().to_owned());
            true
        } else {
            false
        }
    }

    fn join_doc(docs: Vec<String>) -> Option<String> {
        if docs.is_empty() { None } else { Some(docs.join(" ")) }
    }

    fn bindings(&self, block: Pair<'_, Rule>) -> Vec<Binding> {
        block.into_inner().filter(|p| p.as_rule() == Rule::binding).map(|p| self.binding(p)).collect()
    }

    fn binding(&self, pair: Pair<'_, Rule>) -> Binding {
        let mut docs = Vec::new();
        let mut name = None;
        let mut kind = None;
        let mut value = None;
        for p in pair.into_inner() {
            if self.doc(&mut docs, &p) {
                continue;
            }
            match p.as_rule() {
                Rule::ident => name = Some(self.ident(p)),
                Rule::kind_ann => kind = Some(self.ident(p.into_inner().next().expect("kind name"))),
                Rule::expr => value = Some(Arc::new(self.expr(p))),
                _ => {}
            }
        }
        Binding {
            doc: Self::join_doc(docs),
            name: name.expect("binding name"),
            kind,
            value: value.expect("binding value"),
        }
    }

    fn token_items(&self, pairs: Vec<Pair<'_, Rule>>) -> Vec<TokenItem> {
        let mut out = Vec::new();
        for pair in pairs {
            match pair.as_rule() {
                Rule::kw_tokens | Rule::kw_mode => {}
                Rule::mode_block => {
                    let mut inner = pair.into_inner();
                    inner.next(); // kw_mode
                    let name = self.ident(inner.next().expect("mode name"));
                    out.push(TokenItem::Mode { name, items: self.token_items(inner.collect()) });
                }
                Rule::const_block => out.push(TokenItem::Const(self.bindings(pair))),
                Rule::token_scope => {
                    let mut inner = pair.into_inner();
                    let attr = self.ident(inner.next().expect("scope attribute"));
                    let value = self.ident(inner.next().expect("scope key"));
                    out.push(TokenItem::Scope { attr, value, items: self.token_items(inner.collect()) });
                }
                Rule::group => {
                    let mut inner = pair.into_inner().skip_while(|p| p.as_rule() == Rule::doc);
                    let name = self.ident(inner.next().expect("group name"));
                    out.push(TokenItem::Group { name, items: self.token_items(inner.collect()) });
                }
                Rule::binding => out.push(TokenItem::Binding(self.binding(pair))),
                other => unreachable!("unexpected token item {other:?}"),
            }
        }
        out
    }

    fn control(&self, pair: Pair<'_, Rule>) -> ControlAst {
        let mut docs = Vec::new();
        let mut name = None;
        let mut extends = None;
        let mut items = Vec::new();
        for p in pair.into_inner() {
            if self.doc(&mut docs, &p) {
                continue;
            }
            match p.as_rule() {
                Rule::ident => name = Some(self.ident(p)),
                Rule::extends => extends = Some(self.extends(p)),
                _ => items.push(self.item(p)),
            }
        }
        ControlAst { doc: Self::join_doc(docs), name: name.expect("control name"), extends, items }
    }

    fn extends(&self, pair: Pair<'_, Rule>) -> PathRef {
        let path = pair.into_inner().find(|p| p.as_rule() == Rule::path).expect("extends target");
        self.path(path)
    }

    fn item(&self, pair: Pair<'_, Rule>) -> Item {
        match pair.as_rule() {
            Rule::const_block => Item::Const(self.bindings(pair)),
            Rule::part => {
                let mut docs = Vec::new();
                let mut name = None;
                let mut extends = None;
                let mut items = Vec::new();
                for p in pair.into_inner() {
                    if self.doc(&mut docs, &p) {
                        continue;
                    }
                    match p.as_rule() {
                        Rule::kw_part => {}
                        Rule::ident => name = Some(self.ident(p)),
                        Rule::extends => extends = Some(self.extends(p)),
                        _ => items.push(self.item(p)),
                    }
                }
                Item::Part(PartAst { doc: Self::join_doc(docs), name: name.expect("part name"), extends, items })
            }
            Rule::scope => {
                let mut inner = pair.into_inner();
                let attr = self.ident(inner.next().expect("scope attribute"));
                let value = self.ident(inner.next().expect("scope key"));
                let mut extends = None;
                let mut items = Vec::new();
                for p in inner {
                    match p.as_rule() {
                        Rule::extends => {
                            let path = self.extends(p);
                            extends = Some(Ident { text: path.segs.join("."), span: path.span });
                        }
                        _ => items.push(self.item(p)),
                    }
                }
                Item::Scope(ScopeAst { attr, value, extends, items })
            }
            Rule::states => {
                let mut states = Vec::new();
                let mut decls = Vec::new();
                for p in pair.into_inner() {
                    match p.as_rule() {
                        Rule::state => {
                            let ident = p.into_inner().next().expect("state name");
                            states.push(self.ident(ident));
                        }
                        Rule::decl => decls.push(self.decl(p)),
                        _ => {}
                    }
                }
                Item::States(StatesAst { states, decls })
            }
            Rule::decl => Item::Decl(self.decl(pair)),
            other => unreachable!("unexpected item {other:?}"),
        }
    }

    fn decl(&self, pair: Pair<'_, Rule>) -> Decl {
        let mut docs = Vec::new();
        let mut name = None;
        let mut kind = None;
        let mut owning = false;
        let mut value = None;
        for p in pair.into_inner() {
            if self.doc(&mut docs, &p) {
                continue;
            }
            match p.as_rule() {
                Rule::ident => name = Some(self.ident(p)),
                Rule::kind_ann => kind = Some(self.ident(p.into_inner().next().expect("kind name"))),
                Rule::assign => owning = p.as_str() == ":=",
                Rule::expr => value = Some(Arc::new(self.expr(p))),
                _ => {}
            }
        }
        Decl {
            doc: Self::join_doc(docs),
            name: name.expect("property name"),
            kind,
            owning,
            value: value.expect("property value"),
        }
    }

    fn expr(&self, pair: Pair<'_, Rule>) -> Expr {
        pratt()
            .map_primary(|p| self.postfix(p))
            .map_prefix(|op, rhs| {
                let span = self.span(&op);
                Expr { span, kind: ExprKind::Neg(Box::new(rhs)) }
            })
            .map_infix(|lhs, op, rhs| {
                let op_kind = match op.as_rule() {
                    Rule::add => BinOp::Add,
                    Rule::sub => BinOp::Sub,
                    Rule::mul => BinOp::Mul,
                    Rule::div => BinOp::Div,
                    other => unreachable!("unexpected operator {other:?}"),
                };
                let span = lhs.span;
                Expr { span, kind: ExprKind::Bin(op_kind, Box::new(lhs), Box::new(rhs)) }
            })
            .parse(pair.into_inner())
    }

    fn postfix(&self, pair: Pair<'_, Rule>) -> Expr {
        let mut inner = pair.into_inner();
        let mut expr = self.primary(inner.next().expect("primary"));
        for index in inner {
            // The indexed expression starts where its base starts (`gray[7]`, not `[7]`).
            let span = Span { end: self.span(&index).end, ..expr.span };
            let idx = self.expr(index.into_inner().next().expect("index expression"));
            expr = Expr { span, kind: ExprKind::Index(Box::new(expr), Box::new(idx)) };
        }
        expr
    }

    fn primary(&self, pair: Pair<'_, Rule>) -> Expr {
        let span = self.span(&pair);
        let kind = match pair.as_rule() {
            Rule::expr => return self.expr(pair),
            Rule::number => ExprKind::Number(pair.as_str().parse().unwrap_or(0.0)),
            Rule::hex_color => ExprKind::Hex(pair.as_str().to_owned()),
            Rule::string => {
                let s = pair.as_str();
                ExprKind::Str(s[1..s.len() - 1].to_owned())
            }
            Rule::path => ExprKind::Path(pair.as_str().split('.').map(str::to_owned).collect()),
            Rule::call => {
                let mut inner = pair.into_inner();
                let func = inner.next().expect("function name").as_str().split('.').map(str::to_owned).collect();
                ExprKind::Call(func, inner.map(|p| self.expr(p)).collect())
            }
            Rule::list => ExprKind::List(pair.into_inner().map(|p| self.expr(p)).collect()),
            other => unreachable!("unexpected primary {other:?}"),
        };
        Expr { kind, span }
    }
}

fn pratt() -> &'static PrattParser<Rule> {
    static PRATT: OnceLock<PrattParser<Rule>> = OnceLock::new();
    PRATT.get_or_init(|| {
        PrattParser::new()
            .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::sub, Assoc::Left))
            .op(Op::infix(Rule::mul, Assoc::Left) | Op::infix(Rule::div, Assoc::Left))
            .op(Op::prefix(Rule::neg))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_control_with_parts_states_scopes_and_owning_assignments() {
        let src = r#"
            /// A checkbox.
            checkbox {
              /// Indicator box size.
              indicator_size: length = metrics.control.control_height * 0.5
              part indicator {
                background = palette.form.input.background
                :checked:hover { background = tone[10]  border = background }
              }
              variant soft extends outline {
                part indicator { background := tone[5] }
              }
            }
        "#;
        let file = parse(src, 0).expect("parses");
        let Top::Control(control) = &file.items[0] else {
            panic!("control")
        };
        assert_eq!(control.name.text, "checkbox");
        assert_eq!(control.doc.as_deref(), Some("A checkbox."));
        let Item::Decl(decl) = &control.items[0] else {
            panic!("decl")
        };
        assert_eq!(decl.kind.as_ref().map(|k| k.text.as_str()), Some("length"));
        assert!(matches!(decl.value.kind, ExprKind::Bin(BinOp::Mul, _, _)));
        let Item::Scope(scope) = &control.items[2] else {
            panic!("scope")
        };
        assert_eq!(scope.extends.as_ref().map(|e| e.text.as_str()), Some("outline"));
        let Item::Part(part) = &scope.items[0] else {
            panic!("part")
        };
        let Item::Decl(owning) = &part.items[0] else {
            panic!("decl")
        };
        assert!(owning.owning);
    }

    #[test]
    fn unary_minus_and_precedence() {
        let file = parse("t { x: number = -a + b * -2 }", 0).expect("parses");
        let Top::Control(c) = &file.items[0] else { panic!() };
        let Item::Decl(d) = &c.items[0] else { panic!() };
        let ExprKind::Bin(BinOp::Add, lhs, rhs) = &d.value.kind else {
            panic!("add at top: {:?}", d.value.kind)
        };
        assert!(matches!(lhs.kind, ExprKind::Neg(_)));
        assert!(matches!(rhs.kind, ExprKind::Bin(BinOp::Mul, _, _)));
    }

    #[test]
    fn statements_need_no_separators() {
        let file = parse("t { part p { background = transparent  border = none  shadow = none } }", 0).expect("parses");
        let Top::Control(c) = &file.items[0] else { panic!() };
        let Item::Part(p) = &c.items[0] else { panic!() };
        assert_eq!(p.items.len(), 3);
    }

    #[test]
    fn syntax_errors_have_a_location() {
        let err = parse("t {\n  part p { background = }\n}", 0).expect_err("fails");
        assert_eq!(err.line, 2);
        assert!(err.message.contains("expected"), "{}", err.message);
    }

    #[test]
    fn doc_comments_after_a_value_are_not_division() {
        let file = parse("t {\n  a = x.y\n  /// Doc.\n  b: length = 2\n}", 0).expect("parses");
        let Top::Control(c) = &file.items[0] else { panic!() };
        let Item::Decl(b) = &c.items[1] else {
            panic!("two decls: {:?}", c.items)
        };
        assert_eq!(b.doc.as_deref(), Some("Doc."));
    }

    #[test]
    fn tokens_with_modes_and_size_scopes() {
        let src = r#"
            tokens {
              metrics { control { height = 36  size sm { height = 28 } } }
              shadcn { mode dark { delta = 0.04 } }
            }
        "#;
        let file = parse(src, 0).expect("parses");
        let Top::Tokens(items) = &file.items[0] else { panic!() };
        assert_eq!(items.len(), 2);
    }
}
