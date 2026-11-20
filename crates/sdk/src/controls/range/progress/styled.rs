//! Progress read from `.lss` (issue #285): `progress.lss`.

use gpui::Hsla;
use xgpui_luma_style::{PartId, PartView, Value};

use super::theme::{ProgressLook, ProgressTheme};
use crate::style::StyleBinding;
use crate::theme::stylesheet::ProgressGeometry;

fn color(part: PartView<'_>, name: &str) -> Hsla {
    part.value(name).and_then(Value::as_color).map(Hsla::from).unwrap_or_else(gpui::transparent_black)
}

fn number(part: PartView<'_>, name: &str) -> f32 {
    part.value(name).and_then(Value::as_number).unwrap_or(0.0)
}

/// Progress for `binding` (the `progress` control), enabled or not.
pub fn styled_progress_look(binding: &StyleBinding, enabled: bool) -> ProgressLook {
    let resolved = binding.resolve(&[("disabled", !enabled)], 1.0);
    let root = resolved.root();
    let fill = color(resolved.part(PartId::new("fill")), "background");
    let geometry = ProgressGeometry {
        track_height: number(root, "track_height"),
        size: number(root, "ring_size"),
        stroke_width: number(root, "stroke_width"),
        thumb_size: number(root, "thumb_size"),
        indeterminate_fraction: number(root, "indeterminate_fraction"),
        shimmer_soft_opacity: number(root, "shimmer_soft_opacity"),
        shimmer_mid_opacity: number(root, "shimmer_mid_opacity"),
        shimmer_mid_offset: number(root, "shimmer_mid_offset"),
        shimmer_mid_width: number(root, "shimmer_mid_width"),
        shimmer_core_offset: number(root, "shimmer_core_offset"),
        shimmer_core_width: number(root, "shimmer_core_width"),
    };
    ProgressLook {
        track_color: color(resolved.part(PartId::new("track")), "background"),
        progress_color: fill,
        thumb_color: fill,
        track_height: geometry.track_height,
        thumb_size: geometry.thumb_size,
        size: geometry.size,
        stroke_width: geometry.stroke_width,
        geometry,
    }
}

/// A progress theme read from `.lss`, resolved on every render.
#[derive(Clone)]
pub struct StyledProgressTheme {
    binding: StyleBinding,
}

impl StyledProgressTheme {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self { binding: StyleBinding::sdk("progress") }
    }
}

impl ProgressTheme for StyledProgressTheme {
    fn resolve(&self, enabled: bool) -> ProgressLook {
        styled_progress_look(&self.binding, enabled)
    }
}
