//! The Luma style language (`.lss`).
//!
//! Style files declare controls, parts and properties, assign values per part, variant/size scope
//! and state, and define tokens and constants. This crate parses and validates them
//! ([`Stylesheet::compile`]), binds them to native tokens and a mode ([`Theme::bind`]), and resolves
//! values for one control instance ([`Theme::resolve`]) or explains where a value came from
//! ([`Theme::explain`]).
//!
//! Design: `docs/style-language-design.md` and `docs/style-crate-api.md`. The crate has no UI and no
//! knowledge of Luma's controls; everything per control comes from the style files.

mod builtins;
mod color;
mod compile;
mod config;
mod diag;
mod model;
mod names;
mod resolve;
mod syntax;
mod token;
mod value;

pub use color::{Color, RgbSpace};
pub use compile::{Source, Stylesheet};
pub use config::{Config, ConfigBuilder, SnapPolicy};
pub use diag::{Diagnostic, Diagnostics, Severity, Span};
pub use resolve::{Explanation, PartView, Resolved, RuleInfo, StyleKey, Theme};
pub use token::{FunctionSig, NoNativeTokens, TokenCx, TokenSource};
pub use value::{Easing, Kind, Motion, ShadowLayer, TextStyle, Value};
