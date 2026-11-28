//! The segmented choice read from `.lss` (issue #285): `segmented.lss`. Looks render the track and
//! segments from [`SegmentedLook`] and [`SegmentLook`].

use crate::style::StyleStates;
use crate::theme::InteractionState;
use crate::style::PartRead;
use gpui::Hsla;
use xgpui_luma_style::PartId;

use crate::style::StyleBinding;

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
mod tests {}
