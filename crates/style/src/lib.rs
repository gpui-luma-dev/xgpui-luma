//! Declarative control styles with generated descriptors (issue #282 spike).
//!
//! A control's style is one struct annotated with `#[derive(Style)]`. Fields
//! hold [`V`] values (literal, token, field reference or expression) or
//! [`States`] tables of them; nested structs are parts. The derive generates:
//!
//! - `ResolvedX`: plain values templates read, from [`Style::resolve`];
//! - path access (`get`, `set`, `describe`) for tools such as the inspector;
//! - [`Style::SHAPE`]: field descriptors (path, label, doc, kind, range) that
//!   stand in for runtime reflection;
//! - `X::FIELD_NAME` handles for references between fields;
//! - [`Lerp`] for `ResolvedX`, for state animations.
//!
//! Top-level styles (`#[style(path = "...")]`) register their shape with
//! `inventory` so tools can list them; see [`registered`].

extern crate self as xgpui_luma_style;

mod leaf;
mod lerp;
mod resolve;
mod shape;
mod state;
mod value;

pub use leaf::{Color, Dyn, FontFamily, Leaf, Numeric, Opacity, Px, Scalar, Shadow, Weight, opacity, px, scalar};
pub use lerp::Lerp;
pub use resolve::{FieldScope, ResolveCx, Style, TokenSource, split_path};
pub use shape::{FieldContent, FieldDesc, FieldKind, Registered, StyleShape, ValueKind, registered};
pub use state::{State, StateSet, States};
pub use value::{Computed, Factor, Field, NumExpr, Op, Token, V, none};

pub use xgpui_luma_style_derive::Style;

#[doc(hidden)]
pub use inventory;

#[cfg(test)]
mod tests;
