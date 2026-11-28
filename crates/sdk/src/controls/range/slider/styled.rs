//! The slider read from `.lss` (issue #285): `slider.lss`.

use std::sync::Arc;

use crate::style::StyleStates;
use crate::style::PartRead;
use xgpui_luma_style::{PartId, Resolved};

#[allow(deprecated)]
use super::theme::SliderLook;
use super::theme::{SliderDecoration, SliderTheme};
use crate::style::StyleBinding;
use crate::theme::InteractionState;

/// The slider's style for `binding` (the `slider` control) in `state` (#433): templates draw from its root and its
/// `track`, `fill` and `thumb` parts.
pub fn resolve_slider(binding: &StyleBinding, state: InteractionState) -> Arc<Resolved> {
    binding.resolve_states(StyleStates::new(state), 1.0)
}

/// The slider for `binding` (the `slider` control) in `state`.
#[deprecated(note = "templates draw from the resolved style; use `resolve_slider` (#433)")]
#[allow(deprecated)]
pub fn styled_slider_look(binding: &StyleBinding, state: InteractionState) -> SliderLook {
    SliderLook::from_resolved(&resolve_slider(binding, state))
}

#[allow(deprecated)]
impl SliderLook {
    /// The look `resolved` (a slider's resolved style, see [`resolve_slider`]) describes: the compatibility path for
    /// code that still reads a `SliderLook` (#433).
    pub fn from_resolved(resolved: &Resolved) -> Self {
        let root = resolved.root();
        let (track, fill, thumb) = (
            resolved.part(PartId::new("track")),
            resolved.part(PartId::new("fill")),
            resolved.part(PartId::new("thumb")),
        );
        SliderLook {
            thumb_border_width: thumb.number("border_width"),
            decoration: SliderDecoration {
                dial_size: root.number("dial_size"),
                track_radius: root.number("dial_track_radius"),
            },
            track_opacity: track.opacity(),
            fill_opacity: fill.opacity(),
            track_background: track.color("background"),
            track_border: track.color("border"),
            track_border_width: track.number("border_width"),
            fill_background: fill.color("background"),
            thumb_background: thumb.color("background"),
            thumb_border: thumb.color("border"),
            thumb_shadow: thumb.box_shadow(),
            width: root.number("width"),
            height: root.number("height"),
            track_height: root.number("track_height"),
            thumb_size: root.number("thumb_size"),
            radius: root.number("radius"),
        }
    }
}

/// A slider theme read from `.lss`, resolved on every render.
#[derive(Clone)]
pub struct StyledSliderTheme {
    binding: StyleBinding,
}

impl StyledSliderTheme {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self { binding: StyleBinding::sdk("slider") }
    }
}

impl SliderTheme for StyledSliderTheme {
    fn style(&self) -> StyleBinding {
        self.binding.clone()
    }
}
