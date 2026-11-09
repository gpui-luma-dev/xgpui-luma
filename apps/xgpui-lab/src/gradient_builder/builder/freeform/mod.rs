//! Freeform tab: unstructured color points blended as a radial color field.
//!
//! | Concern | Where |
//! | --- | --- |
//! | State | `GradientBuilder::{points, point_ids, background}` (shared with Mesh, swapped on tab change), `point_spreads`, `point_transforms`, `freeform_*` fields in `builder.rs` |
//! | Default points, add/remove/shuffle, spread animation | [`actions`] |
//! | Point list + per-point settings in the inspector | `builder/mesh_stops.rs` (shared with Mesh) |
//! | Inspector sections (renderer, mixing, P3) | `builder/sections.rs` |
//! | On-canvas rotate/stretch handle | [`handles`] |
//! | Point dragging on the canvas | `builder/actions.rs` (shared with Mesh) |
//! | Surface selection | [`GradientBuilder::freeform_surface`] (called from `builder/view.rs`) |
//! | CPU raster + field model | `paint/freeform.rs` (`rasterize_freeform_preview`, `build_color_field`) |
//! | GPU field (Metal) | [`gpu_field`] → `color_field/` |
//! | P3 presentation of the CPU raster | [`p3`] |
//!
//! Colors stay source `ColorValue`s until `gradient_builder/encode.rs` encodes
//! them for the active output (sRGB, or Display P3 when the P3 path is on).

use super::*;
use gpui::AnyElement;

mod actions;
pub(super) mod gpu_field;
mod handles;
pub(super) mod p3;

pub(super) use actions::{animated_spread, default_points};

impl GradientBuilder {
    /// Freeform canvas surface: the GPU field when selected, else the P3
    /// presentation of the CPU raster. `None` shows the CPU raster as sRGB.
    pub(super) fn freeform_surface(&mut self) -> Option<AnyElement> {
        let freeform_p3_active = self.freeform_p3_active();
        if self.freeform_gpu {
            Some(self.freeform_gpu_surface(freeform_p3_active))
        } else if freeform_p3_active {
            self.cached_preview_image().map(|image| self.freeform_p3_cache.element(image))
        } else {
            None
        }
    }

    fn freeform_gpu_surface(&mut self, wide: bool) -> AnyElement {
        let output = OutputSpace::wide(wide);
        let field = super::super::paint::build_color_field(
            &encode_points(&self.render_points(), output),
            encode(self.render_background(), output),
            self.spread_percent,
            self.freeform_hsl,
            &super::super::paint::FieldShape {
                mixing: self.freeform_mixing,
                background_weight: self.field_background_weight,
                point_transforms: &self.point_transforms,
                point_spreads: &self.point_spreads,
                softness_percent: self.softness_percent,
            },
        );
        self.freeform_gpu_cache.element(field, wide)
    }
}
