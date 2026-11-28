//! The segmented choice read from `.lss` (issue #285): `segmented.lss`. Looks render the track and
//! segments from [`SegmentedLook`] and [`SegmentLook`].

use crate::style::StyleStates;
use crate::theme::InteractionState;
use crate::style::PartRead;
use gpui::Hsla;
use xgpui_luma_style::PartId;

use crate::style::StyleBinding;

/// The track behind the segments.
#[deprecated(note = "read the track from `StyledSegmented::resolve_track` (#433)")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SegmentedLook {
    pub background: Hsla,
    pub radius: f32,
}

/// One segment in one state.
#[deprecated(note = "read a segment from `StyledSegmented::resolve_segment` (#433)")]
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

    /// The track's style (#433): read its root.
    pub fn resolve_track(&self) -> std::sync::Arc<xgpui_luma_style::Resolved> {
        self.binding.resolve(&[], 1.0)
    }

    /// A segment's style in `state` (#433): read its `item` part, and while focus is visible the root's
    /// `focus_border` for its edge (see [`segment_border`]).
    pub fn resolve_segment(&self, state: SegmentState) -> std::sync::Arc<xgpui_luma_style::Resolved> {
        // The focus ring is drawn from `focus_visible`, not a style state.
        let interaction = InteractionState {
            hovered: state.hovered,
            pressed: state.pressed,
            disabled: state.disabled,
            ..InteractionState::default()
        };
        self.binding.resolve_states(StyleStates::new(interaction).selected(state.selected), 1.0)
    }

    #[deprecated(note = "read the track from `resolve_track()` (#433)")]
    #[allow(deprecated)]
    pub fn track(&self) -> SegmentedLook {
        let resolved = self.resolve_track();
        let root = resolved.root();
        SegmentedLook { background: root.color("background"), radius: root.number("radius") }
    }

    #[deprecated(note = "read a segment from `resolve_segment()` (#433)")]
    #[allow(deprecated)]
    pub fn segment(&self, state: SegmentState) -> SegmentLook {
        let resolved = self.resolve_segment(state);
        let item = resolved.part(PartId::new("item"));
        SegmentLook {
            background: item.color("background"),
            foreground: item.color("foreground"),
            border: segment_border(&resolved, state),
            radius: item.number("radius"),
            height: item.number("height"),
            padding_x: item.number("padding_x"),
            font_size: item.font_size().unwrap_or(14.0),
            line_height: item.line_height().unwrap_or(20.0),
        }
    }
}

/// A segment's edge in `state`, from its resolved style (see [`StyledSegmented::resolve_segment`]): the focus
/// ring while focus is visible, else the `item` part's border.
pub fn segment_border(resolved: &xgpui_luma_style::Resolved, state: SegmentState) -> Hsla {
    if state.focus_visible {
        resolved.root().color("focus_border")
    } else {
        resolved.part(PartId::new("item")).color("border")
    }
}

#[cfg(test)]
#[allow(deprecated)]
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
