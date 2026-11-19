//! Name resolution (language §7), shared by the compiler's checks and the evaluator.

use std::collections::HashSet;

use crate::builtins;
use crate::model::Model;
use crate::value::Value;

/// Where an expression is being evaluated.
#[derive(Clone, Copy)]
pub(crate) enum Ctx<'a> {
    /// A property of `part` in `control`.
    Prop { control: usize, part: usize, scope: usize },
    /// A token defined at `self_path` inside group `group`.
    Token { group: &'a [String], self_path: &'a str, scope: usize },
    /// A constant.
    Const { scope: usize },
}

#[derive(Clone, Debug)]
pub(crate) enum Target {
    Keyword(Value),
    /// A property of this control. `part: None` means "the current part, falling back to root".
    Prop {
        part: Option<usize>,
        prop: String,
    },
    /// Another control's root property (geometry), optionally under a variant.
    OtherControl {
        control: usize,
        variant: Option<String>,
        prop: String,
    },
    Const {
        scope: usize,
        name: String,
        fields: Vec<String>,
    },
    Token {
        path: String,
        fields: Vec<String>,
    },
    /// A native token (`css.primary`); checked against the token source when bound.
    Native {
        path: Vec<String>,
    },
    Unknown,
}

/// Resolve a name. `roots` are the bound token source's native roots (`None` while compiling).
pub(crate) fn lookup(
    model: &Model,
    roots: Option<&HashSet<String>>,
    ctx: Ctx<'_>,
    segs: &[String],
) -> Result<Target, String> {
    if segs.len() == 1
        && let Some(v) = builtins::keyword(&segs[0])
    {
        return Ok(Target::Keyword(v));
    }
    let first = &segs[0];
    let rest = &segs[1..];
    let is_native = |name: &str| roots.is_some_and(|r| r.contains(name));
    let const_target = |scope: usize| {
        model
            .const_lookup(scope, first)
            .map(|_| Target::Const { scope, name: first.clone(), fields: rest.to_vec() })
    };

    match ctx {
        Ctx::Prop { control, scope, .. } => {
            // The first segment picks a namespace; a name in more than one is an error (language §7).
            let single = segs.len() == 1;
            let mut namespaces: Vec<&str> = Vec::new();
            if single && model.prop_kind(control, first).is_some() {
                namespaces.push("a property of this control");
            }
            if !single && model.controls[control].part(first).is_some() {
                namespaces.push("a part of this control");
            }
            if model.const_lookup(scope, first).is_some() {
                namespaces.push("a constant");
            }
            if model.tokens.is_root(first) {
                namespaces.push("a token group");
            }
            if !single && model.control(first).is_some() {
                namespaces.push("a control");
            }
            if !single && is_native(first) {
                namespaces.push("a native token root");
            }
            match namespaces.as_slice() {
                [] if roots.is_none() && !single => Ok(Target::Native { path: segs.to_vec() }),
                [] => Ok(Target::Unknown),
                [only] => Ok(match *only {
                    "a property of this control" => Target::Prop { part: None, prop: first.clone() },
                    "a part of this control" => match rest {
                        [prop] if model.prop_kind(control, prop).is_some() => {
                            Target::Prop { part: model.controls[control].part(first), prop: prop.clone() }
                        }
                        _ => Target::Unknown,
                    },
                    "a constant" => const_target(scope).unwrap_or(Target::Unknown),
                    "a token group" => match model.tokens.find(segs) {
                        Some((path, fields)) => Target::Token { path, fields: fields.to_vec() },
                        None => Target::Unknown,
                    },
                    "a control" => {
                        let other = model.control(first).expect("checked");
                        match rest {
                            [prop] => Target::OtherControl { control: other, variant: None, prop: prop.clone() },
                            [variant, prop] => Target::OtherControl {
                                control: other,
                                variant: Some(variant.clone()),
                                prop: prop.clone(),
                            },
                            _ => Target::Unknown,
                        }
                    }
                    _ => Target::Native { path: segs.to_vec() },
                }),
                many => Err(format!("\"{}\" is ambiguous: \"{first}\" is {}", segs.join("."), many.join(" and "))),
            }
        }
        Ctx::Token { group, self_path, scope } => {
            for k in (0..=group.len()).rev() {
                let mut full: Vec<String> = group[..k].to_vec();
                full.extend_from_slice(segs);
                if let Some((path, fields)) = model.tokens.find(&full)
                    && path.split('.').count() > k
                    && path != self_path
                {
                    return Ok(Target::Token { path, fields: fields.to_vec() });
                }
            }
            if let Some(t) = const_target(scope) {
                return Ok(t);
            }
            if is_native(first) || (roots.is_none() && segs.len() > 1) {
                return Ok(Target::Native { path: segs.to_vec() });
            }
            Ok(Target::Unknown)
        }
        Ctx::Const { scope } => {
            if let Some(t) = const_target(scope) {
                return Ok(t);
            }
            if let Some((path, fields)) = model.tokens.find(segs) {
                return Ok(Target::Token { path, fields: fields.to_vec() });
            }
            if is_native(first) || (roots.is_none() && segs.len() > 1) {
                return Ok(Target::Native { path: segs.to_vec() });
            }
            Ok(Target::Unknown)
        }
    }
}
