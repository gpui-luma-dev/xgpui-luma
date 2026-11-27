//! The slider read from `.lss` (issue #285): `slider.lss`.

use crate::style::StyleStates;
use crate::style::PartRead;
use xgpui_luma_style::PartId;

use super::theme::{SliderDecoration, SliderLook, SliderTheme};
use crate::style::StyleBinding;
use crate::theme::InteractionState;

/// The slider for `binding` (the `slider` control) in `state`.
pub fn styled_slider_look(binding: &StyleBinding, state: InteractionState) -> SliderLook {
    let states = StyleStates::new(state);
    let resolved = binding.resolve_states(states, 1.0);
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
    fn resolve(&self, state: InteractionState) -> SliderLook {
        styled_slider_look(&self.binding, state)
    }
}
