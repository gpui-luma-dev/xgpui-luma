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

/// A style struct. Implemented by `#[derive(Style)]`.
pub trait Style: Clone + std::fmt::Debug + FieldScope + 'static {
    /// Plain values that templates read.
    type Resolved: Clone + std::fmt::Debug + PartialEq + Lerp;
    /// A sparse overlay used by looks, apps and editors.
    type Patch: Clone + std::fmt::Debug + Default;
    /// Field descriptors.
    const SHAPE: &'static StyleShape;

    fn resolve(&self, cx: &ResolveCx<'_>, at: StateSet) -> Self::Resolved;
    /// Apply a patch: every field the patch sets replaces this style's value.
    fn apply(&mut self, patch: &Self::Patch);

    /// A copy with `patches` applied in order.
    fn with(&self, patches: &[&Self::Patch]) -> Self {
        let mut style = self.clone();
        for patch in patches {
            style.apply(patch);
        }
        style
    }
}
