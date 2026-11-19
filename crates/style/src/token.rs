//! Native tokens and look functions supplied by the consumer.

use crate::value::{Kind, Value};

/// What a native token lookup or look function can see about the instance being styled.
pub struct TokenCx<'a> {
    pub(crate) mode: &'a str,
    pub(crate) attrs: &'a [(String, String)],
}

impl TokenCx<'_> {
    /// The bound mode, e.g. `"light"`.
    pub fn mode(&self) -> &str {
        self.mode
    }

    /// An instance attribute, e.g. `attr("tone")` for Radix `tone[n]`.
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str())
    }
}

/// A look function's signature, used to check calls when a theme is bound.
#[derive(Clone, Copy, Debug)]
pub struct FunctionSig {
    pub min_args: usize,
    pub max_args: usize,
    pub returns: Kind,
}

/// Native token roots (Shadcn `css.*`, Radix `gray[n]`) and look functions (`layer`, `shift`, …).
pub trait TokenSource: Send + Sync {
    /// Root names this source provides, e.g. `["css"]` or `["gray", "accent", "tone"]`.
    fn roots(&self) -> &[&str];

    /// A dotted native token, e.g. `"css.primary_foreground"`. `None` means absent (see `first()`).
    fn token(&self, path: &str, cx: &TokenCx<'_>) -> Option<Value>;

    /// An indexed native token, e.g. `gray[7]`.
    fn index(&self, _root: &str, _index: i64, _cx: &TokenCx<'_>) -> Option<Value> {
        None
    }

    /// The signature of a look function, or `None` if this source has no function of that name.
    fn function(&self, _name: &str) -> Option<FunctionSig> {
        None
    }

    /// Call a look function with evaluated arguments.
    fn call(&self, name: &str, _args: &[Value], _cx: &TokenCx<'_>) -> Result<Value, String> {
        Err(format!("unknown function \"{name}\""))
    }
}

/// A token source with no native roots or functions (a look that keeps all tokens in files).
pub struct NoNativeTokens;

impl TokenSource for NoNativeTokens {
    fn roots(&self) -> &[&str] {
        &[]
    }

    fn token(&self, _path: &str, _cx: &TokenCx<'_>) -> Option<Value> {
        None
    }
}
