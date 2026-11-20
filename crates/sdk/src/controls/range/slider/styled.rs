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
    let states = [("hover", state.hovered), ("pressed", state.pressed), ("disabled", state.disabled)];
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
            disabled_opacity: number(root, "disabled_opacity"),
        },
        track_background: color(track, "background"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sdk_slider_matches_the_token_slider() {
        let close = |a: Hsla, b: Hsla| {
            let (a, b) = (a.to_rgb(), b.to_rgb());
            [(a.r, b.r), (a.g, b.g), (a.b, b.b), (a.a, b.a)].iter().all(|(x, y)| (x - y).abs() < 2e-3)
        };
        let native = super::super::theme::DefaultSliderTheme::default();
        for bits in 0..8u8 {
            let state = InteractionState {
                hovered: bits & 1 != 0,
                pressed: bits & 2 != 0,
                disabled: bits & 4 != 0,
                ..Default::default()
            };
            let (lss, want) = (StyledSliderTheme::sdk().resolve(state), native.resolve(state));
            for (got, want, name) in [
                (lss.track_background, want.track_background, "track"),
                (lss.fill_background, want.fill_background, "fill"),
                (lss.thumb_background, want.thumb_background, "thumb"),
                (lss.thumb_border, want.thumb_border, "thumb border"),
            ] {
                assert!(close(got, want), "{state:?} {name}: {got:?} vs {want:?}");
            }
            assert_eq!(
                (lss.width, lss.height, lss.track_height, lss.thumb_size, lss.radius, lss.thumb_border_width),
                (want.width, want.height, want.track_height, want.thumb_size, want.radius, want.thumb_border_width)
            );
            assert_eq!(lss.thumb_shadow.len(), want.thumb_shadow.len());
        }
    }
}
