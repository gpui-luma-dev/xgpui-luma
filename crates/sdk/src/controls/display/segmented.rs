//! The segmented choice read from `.lss` (issue #285): `segmented.lss`. Looks render the track and
//! segments from [`SegmentedLook`] and [`SegmentLook`].

use gpui::Hsla;
use xgpui_luma_style::{PartId, PartView, Value};

use crate::style::StyleBinding;

/// The track behind the segments.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SegmentedLook {
    pub background: Hsla,
    pub radius: f32,
}

/// One segment in one state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SegmentLook {
    pub background: Hsla,
    pub foreground: Hsla,
    /// The edge, or the focus ring while focus is visible.
    pub border: Hsla,
    pub radius: f32,
    pub height: f32,
    pub padding_x: f32,
    pub font_size: f32,
    pub line_height: f32,
}

fn color(part: PartView<'_>, name: &str) -> Hsla {
    part.value(name).and_then(Value::as_color).map(Hsla::from).unwrap_or_else(gpui::transparent_black)
}

fn number(part: PartView<'_>, name: &str) -> f32 {
    part.value(name).and_then(Value::as_number).unwrap_or(0.0)
}

/// A segment's states.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SegmentState {
    pub selected: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub focus_visible: bool,
    pub disabled: bool,
}

/// The segmented choice's `.lss` values for one binding (the `segmented` control).
#[derive(Clone)]
pub struct StyledSegmented {
    binding: StyleBinding,
}

impl StyledSegmented {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self::new(StyleBinding::sdk("segmented"))
    }

    pub fn track(&self) -> SegmentedLook {
        let resolved = self.binding.resolve(&[], 1.0);
        let root = resolved.root();
        SegmentedLook { background: color(root, "background"), radius: number(root, "radius") }
    }

    pub fn segment(&self, state: SegmentState) -> SegmentLook {
        let states = [
            ("selected", state.selected),
            ("hover", state.hovered),
            ("pressed", state.pressed),
            ("disabled", state.disabled),
        ];
        let resolved = self.binding.resolve(&states, 1.0);
        let (root, item) = (resolved.root(), resolved.part(PartId::new("item")));
        SegmentLook {
            background: color(item, "background"),
            foreground: color(item, "foreground"),
            border: if state.focus_visible {
                color(root, "focus_border")
            } else {
                color(item, "border")
            },
            radius: number(item, "radius"),
            height: number(item, "height"),
            padding_x: number(item, "padding_x"),
            font_size: item.font_size().unwrap_or(14.0),
            line_height: item.line_height().unwrap_or(20.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sdk_segments_follow_selection_and_focus() {
        let lss = StyledSegmented::sdk();
        let idle = lss.segment(SegmentState::default());
        let selected = lss.segment(SegmentState { selected: true, ..Default::default() });
        let focused = lss.segment(SegmentState { focus_visible: true, ..Default::default() });
        assert_ne!(idle.background, selected.background);
        assert_ne!(idle.border, focused.border);
        assert!(lss.track().radius > 0.0 && idle.height > 0.0);
        assert!(StyledCallout::sdk().look().padding > 0.0);
    }

    use crate::controls::display::callout::StyledCallout;
}
