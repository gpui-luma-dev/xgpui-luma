//! Progress read from `.lss` (issue #285): `progress.lss`.

use crate::style::PartRead;
use xgpui_luma_style::PartId;

use super::theme::{ProgressLook, ProgressTheme};
use crate::style::StyleBinding;
use crate::theme::stylesheet::ProgressGeometry;

/// Progress for `binding` (the `progress` control), enabled or not.
pub fn styled_progress_look(binding: &StyleBinding, enabled: bool) -> ProgressLook {
    let resolved = binding.resolve(&[("disabled", !enabled)], 1.0);
    let root = resolved.root();
    let fill = resolved.part(PartId::new("fill")).color("background");
    let track = resolved.part(PartId::new("track"));
    let geometry = ProgressGeometry {
        track_height: root.number("track_height"),
        size: root.number("ring_size"),
        stroke_width: root.number("stroke_width"),
        thumb_size: root.number("thumb_size"),
        indeterminate_fraction: root.number("indeterminate_fraction"),
        shimmer_soft_opacity: root.number("shimmer_soft_opacity"),
        shimmer_mid_opacity: root.number("shimmer_mid_opacity"),
        shimmer_mid_offset: root.number("shimmer_mid_offset"),
        shimmer_mid_width: root.number("shimmer_mid_width"),
        shimmer_core_offset: root.number("shimmer_core_offset"),
        shimmer_core_width: root.number("shimmer_core_width"),
    };
    ProgressLook {
        track_color: track.color("background"),
        track_border: track.color("border"),
        track_border_width: track.number("border_width"),
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
