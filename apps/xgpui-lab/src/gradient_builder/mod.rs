//! Gradient Builder demo (Gradients / Mesh / Freeform tabs).
//!
//! # Layout
//!
//! - `builder.rs` + `builder/`: the `GradientBuilder` view: state, editing,
//!   inspector UI and preview selection. Its module doc has the per-tab map.
//! - `paint/`: CPU rasterizers and geometry for each tab, taking encoded
//!   colors (`PaintPoint`, `(f32, Hsla)` stops).
//! - `color_field/`: the Freeform GPU field model and its Metal shader.
//! - `encode.rs`: the only place source colors become paint inputs.
//! - `color.rs`: color formatting and interpolation helpers.
//!
//! # Color flow
//!
//! ```text
//! source ColorValue (stop_colors, points, background; unclipped)
//!   → image adjustments (GradientBuilder::render_points / render_background)
//!   → encode.rs (encode / encode_points / encode_stops, per OutputSpace)
//!   → paint/* CPU rasters (sRGB)   or   P3 / GPU shaders (Display P3)
//! ```
//!
//! Freeform code: `builder/freeform/` (start at its `mod.rs`),
//! `paint/freeform.rs`, `color_field/`.

mod builder;
mod color_field;
mod color;
mod encode;
mod paint;

pub use builder::GradientBuilder;
