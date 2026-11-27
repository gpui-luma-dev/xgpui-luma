//! Layout chrome read from `.lss` (issue #285): `scrollbar.lss`, `dock_splitter.lss`,
//! `resizable_panels.lss` and `split_view.lss`. Looks bind their own stylesheet and use these themes.

use crate::style::StyleStates;
use crate::style::PartRead;

use super::dock_splitter::{DockSplitterLook, DockSplitterTheme};
use super::resizable_panels::{ResizablePanelsLook, ResizablePanelsTheme, ResizeHandleMetrics};
use super::scrollbar::{ScrollbarLook, ScrollbarOrientation, ScrollbarStyle, ScrollbarTheme};
use super::split_view::{SplitViewLook, SplitViewTheme};
use crate::style::StyleBinding;
use crate::theme::stylesheet::{DockSplitterGeometry, SplitViewGeometry};
use crate::theme::{InteractionState, MetricTokens};

/// The scrollbar from `scrollbar.lss`; `metrics` are the control metrics the look sizes with.
pub struct StyledScrollbarTheme {
    binding: StyleBinding,
    metrics: MetricTokens,
}

impl StyledScrollbarTheme {
    pub fn new(binding: StyleBinding, metrics: MetricTokens) -> Self {
        Self { binding, metrics }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self::new(StyleBinding::sdk("scrollbar"), MetricTokens::default())
    }
}

impl ScrollbarTheme for StyledScrollbarTheme {
    fn resolve(
        &self,
        state: InteractionState,
        orientation: ScrollbarOrientation,
        style: ScrollbarStyle,
    ) -> ScrollbarLook {
        let variant = match style {
            ScrollbarStyle::Ghost => "ghost",
            ScrollbarStyle::Soft => "soft",
        };
        let states = StyleStates::new(state);
        let resolved = self.binding.resolve_states_for(&[("variant", variant)], states, 1.0);
        let root = resolved.root();
        ScrollbarLook {
            opacity: root.opacity(),
            track_background: root.color("background"),
            track_border: root.color("border"),
            track_border_width: root.number("border_width"),
            thumb_background: root.color("thumb"),
            length: root.number(match orientation {
                ScrollbarOrientation::Horizontal => "length_h",
                ScrollbarOrientation::Vertical => "length_v",
            }),
            thickness: root.number("thickness"),
            track_thickness: root.number("track_thickness"),
            thumb_thickness: root.number("thumb_thickness"),
            min_thumb_length: root.number("min_thumb_length"),
            radius: root.number("radius"),
        }
    }

    fn metrics(&self) -> MetricTokens {
        self.metrics
    }
}

/// The dock splitter from `dock_splitter.lss`.
pub struct StyledDockSplitterTheme(StyleBinding);

impl StyledDockSplitterTheme {
    pub fn new(binding: StyleBinding) -> Self {
        Self(binding)
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self(StyleBinding::sdk("dock_splitter"))
    }
}

impl DockSplitterTheme for StyledDockSplitterTheme {
    fn resolve(&self, enabled: bool) -> DockSplitterLook {
        let resolved = self.0.resolve(&[("disabled", !enabled)], 1.0);
        let root = resolved.root();
        let geometry = DockSplitterGeometry {
            hit_target: root.number("hit_target"),
            visible_line: root.number("visible_line"),
            thumb_width: root.number("thumb_width"),
            thumb_height: root.number("thumb_height"),
            thumb_radius: root.number("thumb_radius"),
            thumb_rest_opacity: root.number("thumb_rest_opacity"),
        };
        DockSplitterLook {
            line_color: root.color("line"),
            hover_color: root.color("hover_line"),
            thumb_color: root.color("thumb"),
            hit_target_px: geometry.hit_target,
            visible_line_px: geometry.visible_line,
            geometry,
        }
    }
}

/// Resizable panels from `resizable_panels.lss`, handle sizes included.
pub struct StyledResizablePanelsTheme {
    binding: StyleBinding,
}

impl StyledResizablePanelsTheme {
    pub fn new(binding: StyleBinding) -> Self {
        Self { binding }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self::new(StyleBinding::sdk("resizable_panels"))
    }
}

impl ResizablePanelsTheme for StyledResizablePanelsTheme {
    fn resize_handle_metrics(&self) -> ResizeHandleMetrics {
        ResizeHandleMetrics::from_binding(&self.binding)
    }

    fn resolve(&self, state: InteractionState) -> ResizablePanelsLook {
        let states = StyleStates::new(state);
        let resolved = self.binding.resolve_states(states, 1.0);
        let root = resolved.root();
        ResizablePanelsLook {
            border: root.color("border"),
            border_width: root.number("border_width"),
            divider_width: root.number("divider_width"),
            divider: root.color("divider"),
            grip: root.color("grip"),
            grip_emphasis: root.color("grip_emphasis"),
            opacity: root.opacity(),
        }
    }
}

/// The split view from `split_view.lss`.
pub struct StyledSplitViewTheme(StyleBinding);

impl StyledSplitViewTheme {
    pub fn new(binding: StyleBinding) -> Self {
        Self(binding)
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self(StyleBinding::sdk("split_view"))
    }
}

impl SplitViewTheme for StyledSplitViewTheme {
    fn resolve(&self, _hovered: bool, enabled: bool) -> SplitViewLook {
        let resolved = self.0.resolve(&[("disabled", !enabled)], 1.0);
        let root = resolved.root();
        SplitViewLook {
            geometry: SplitViewGeometry {
                hitbox_width: root.number("hitbox_width"),
                expand_inset_y: root.number("expand_inset_y"),
                expand_cue_left: root.number("expand_cue_left"),
                cue_inset_y: root.number("cue_inset_y"),
                cue_radius: root.number("cue_radius"),
                cue_width: root.number("cue_width"),
                cue_hovered_width: root.number("cue_hovered_width"),
            },
            separator: root.color("separator"),
            separator_hover: root.color("separator_hover"),
        }
    }
}
