//! The Luma style language (`.lss`).
//!
//! Style files declare controls, parts and properties, assign values per part, variant/size scope
//! and state, and define tokens and constants. This crate parses and validates them
//! ([`Stylesheet::builder`]), binds them to native tokens and a mode ([`Theme::bind`]), resolves values
//! for one control instance ([`Theme::resolve`]), explains and inspects where values came from
//! ([`Theme::explain`], [`Theme::inspect`]), and edits or exports `.lss` text ([`Stylesheet::apply`]).
//!
//! Design: `docs/style-language-design.md` and `docs/style-crate-api.md`. The crate has no UI and no
//! knowledge of Luma's controls; everything per control comes from the style files.

mod builtins;
mod color;
mod compile;
mod config;
mod diag;
mod edit;
mod handles;
mod inspect;
mod model;
mod names;
mod overrides;
mod resolve;
mod syntax;
mod token;
mod value;

pub use color::{Color, RgbSpace};
pub use compile::{Source, Stylesheet, StylesheetBuilder};
pub use config::{Config, ConfigBuilder, SnapPolicy};
pub use diag::{Code, Diagnostic, Diagnostics, Severity, Span};
pub use edit::{Edit, RuleSpec, export_tokens};
pub use handles::{
    AttrId, AttrValue, ControlId, FromValue, MAX_ATTRIBUTES, PartId, Prop, StateId, StateSet, StyleKey, props,
};
pub use inspect::{
    Axes, BoxModel, Category, Edges, Explanation, Inspection, Location, PartInspection, Reference, ReferenceKind, Row,
    RuleInfo, ValueSource,
};
pub use overrides::{OverrideValue, Overrides};
pub use resolve::{PartView, Resolved, Theme};
pub use token::{FunctionSig, NoNativeTokens, TokenCx, TokenSource};
pub use value::{Easing, Kind, Motion, ShadowLayer, TextStyle, Value};
