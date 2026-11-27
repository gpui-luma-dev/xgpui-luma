//! The slider read from `.lss` (issue #285): `slider.lss`.

use gpui::Hsla;
use xgpui_luma_style::{PartId, PartView, Value, box_shadows};

use super::theme::{SliderDecoration, SliderLook, SliderTheme};
use crate::style::StyleBinding;
use crate::theme::InteractionState;

fn color(part: PartView<'_>, name: &str) -> Hsla {
    part.value(name).and_then(Value::as_color).map(Hsla::from).unwrap_or_else(gpui::transparent_black)
}

fn number(part: PartView<'_>, name: &str) -> f32 {
    part.value(name).and_then(Value::as_number).unwrap_or(0.0)
}

/// The slider for `binding` (the `slider` control) in `state`.
pub fn styled_slider_look(binding: &StyleBinding, state: InteractionState) -> SliderLook {
    let states = [
        ("hover", state.hovered),
        ("pressed", state.pressed),
        ("focused", state.focused),
        ("disabled", state.disabled),
    ];
    let resolved = binding.resolve(&states, 1.0);
    let root = resolved.root();
    let (track, fill, thumb) = (
        resolved.part(PartId::new("track")),
        resolved.part(PartId::new("fill")),
        resolved.part(PartId::new("thumb")),
    );
    SliderLook {
        thumb_border_width: number(thumb, "border_width"),
        decoration: SliderDecoration {
            dial_size: number(root, "dial_size"),
            track_radius: number(root, "dial_track_radius"),
        },
        track_opacity: crate::style::part_opacity(track),
        fill_opacity: crate::style::part_opacity(fill),
        track_background: color(track, "background"),
        track_border: color(track, "border"),
        track_border_width: number(track, "border_width"),
        fill_background: color(fill, "background"),
        thumb_background: color(thumb, "background"),
        thumb_border: color(thumb, "border"),
        thumb_shadow: thumb.shadow().map(|layers| box_shadows(&layers)).unwrap_or_default(),
        width: number(root, "width"),
        height: number(root, "height"),
        track_height: number(root, "track_height"),
        thumb_size: number(root, "thumb_size"),
        radius: number(root, "radius"),
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
