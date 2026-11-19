//! Resolving a style against theme tokens.

use std::cell::RefCell;

use crate::leaf::Dyn;
use crate::lerp::Lerp;
use crate::shape::StyleShape;
use crate::state::StateSet;

/// Where token paths such as `palette.focus.ring` are looked up.
pub trait TokenSource {
    fn token(&self, path: &str) -> Option<Dyn>;
}

/// Everything resolving needs: tokens, the display scale, and a place to
/// record problems (missing tokens, wrong types, reference cycles).
pub struct ResolveCx<'a> {
    tokens: &'a dyn TokenSource,
    scale_factor: f32,
    errors: RefCell<Vec<String>>,
}

impl<'a> ResolveCx<'a> {
    pub fn new(tokens: &'a dyn TokenSource, scale_factor: f32) -> Self {
        Self { tokens, scale_factor, errors: RefCell::new(Vec::new()) }
    }

    pub fn token(&self, path: &str) -> Option<Dyn> {
        self.tokens.token(path)
    }

    pub fn tokens(&self) -> &'a dyn TokenSource {
        self.tokens
    }

    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    /// Round a logical length to the nearest device pixel.
    pub fn snap(&self, value: f32) -> f32 {
        if self.scale_factor <= 0.0 {
            return value;
        }
        (value * self.scale_factor).round() / self.scale_factor
    }

    pub fn error(&self, message: String) {
        self.errors.borrow_mut().push(message);
    }

    /// Problems recorded so far.
    pub fn errors(&self) -> Vec<String> {
        self.errors.borrow().clone()
    }
}

/// Resolves a field of the struct being resolved, for `Field` references.
/// Implemented by `#[derive(Style)]`.
pub trait FieldScope {
    fn field(&self, name: &str, cx: &ResolveCx<'_>, at: StateSet, depth: u8) -> Option<Dyn>;
}

/// Split a dotted path into its first segment and the rest.
#[doc(hidden)]
pub fn split_path(path: &str) -> (&str, Option<&str>) {
    match path.split_once('.') {
        Some((head, rest)) => (head, Some(rest)),
        None => (path, None),
    }
}

/// A style struct. Implemented by `#[derive(Style)]`.
pub trait Style: Clone + std::fmt::Debug + FieldScope + 'static {
    /// Plain values that templates read.
    type Resolved: Clone + std::fmt::Debug + PartialEq + Lerp;
    /// Field descriptors.
    const SHAPE: &'static StyleShape;

    fn resolve(&self, cx: &ResolveCx<'_>, at: StateSet) -> Self::Resolved;

    /// The resolved value at a dotted path (`"indicator.background"`), as other
    /// fields would see it. `None` for unknown paths, parts and computed fields.
    fn get(&self, path: &str, cx: &ResolveCx<'_>, at: StateSet) -> Option<Dyn>;
    /// Set a literal at a dotted path. Per-state fields set the rule for `at`
    /// (the base value for an empty set). Returns `false` for an unknown path,
    /// a computed field, or a value of the wrong type.
    fn set(&mut self, path: &str, at: StateSet, value: Dyn) -> bool;
    /// How the value at a path is authored: a literal, token path or expression.
    fn describe(&self, path: &str, at: StateSet) -> Option<String>;
    /// Every state that some per-state rule in this style (or its parts) uses.
    fn states_used(&self) -> StateSet;
}
