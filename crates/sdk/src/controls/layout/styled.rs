//! Layout chrome read from `.lss` (issue #285): `scrollbar.lss`, `dock_splitter.lss`,
//! `resizable_panels.lss` and `split_view.lss`. Looks bind their own stylesheet and use these themes.

use gpui::Hsla;
use xgpui_luma_style::{PartView, Value};

use super::dock_splitter::{DockSplitterLook, DockSplitterTheme};
use super::resizable_panels::{ResizablePanelsLook, ResizablePanelsTheme, ResizeHandleMetrics};
use super::scrollbar::{ScrollbarLook, ScrollbarOrientation, ScrollbarStyle, ScrollbarTheme};
use super::split_view::{SplitViewLook, SplitViewTheme};
use crate::style::StyleBinding;
use crate::theme::stylesheet::{DockSplitterGeometry, SplitViewGeometry};
use crate::theme::{InteractionState, MetricTokens};

fn color(part: PartView<'_>, name: &str) -> Hsla {
    part.value(name).and_then(Value::as_color).map(Hsla::from).unwrap_or_else(gpui::transparent_black)
}

fn number(part: PartView<'_>, name: &str) -> f32 {
    part.value(name).and_then(Value::as_number).unwrap_or(0.0)
}

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
        let states = [("hover", state.hovered), ("pressed", state.pressed), ("disabled", state.disabled)];
        let resolved = self.binding.resolve_for(&[("variant", variant)], &states, 1.0);
        let root = resolved.root();
        ScrollbarLook {
            disabled_opacity: number(root, "disabled_opacity"),
            track_background: color(root, "background"),
            thumb_background: color(root, "thumb"),
            length: number(
                root,
                match orientation {
                    ScrollbarOrientation::Horizontal => "length_h",
                    ScrollbarOrientation::Vertical => "length_v",
                },
            ),
            thickness: number(root, "thickness"),
            track_thickness: number(root, "track_thickness"),
            thumb_thickness: number(root, "thumb_thickness"),
            min_thumb_length: number(root, "min_thumb_length"),
            radius: number(root, "radius"),
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
            hit_target: number(root, "hit_target"),
            visible_line: number(root, "visible_line"),
            thumb_width: number(root, "thumb_width"),
            thumb_height: number(root, "thumb_height"),
            thumb_radius: number(root, "thumb_radius"),
            thumb_rest_opacity: number(root, "thumb_rest_opacity"),
        };
        DockSplitterLook {
            line_color: color(root, "line"),
            hover_color: color(root, "hover_line"),
            thumb_color: color(root, "thumb"),
            hit_target_px: geometry.hit_target,
            visible_line_px: geometry.visible_line,
            geometry,
        }
    }
}

/// Resizable panels from `resizable_panels.lss`; handle sizes stay the stylesheet's `handle_metrics`.
pub struct StyledResizablePanelsTheme {
    binding: StyleBinding,
    handle_metrics: ResizeHandleMetrics,
}

impl StyledResizablePanelsTheme {
    pub fn new(binding: StyleBinding, handle_metrics: ResizeHandleMetrics) -> Self {
        Self { binding, handle_metrics }
    }

    /// The SDK alone.
    pub fn sdk() -> Self {
        Self::new(StyleBinding::sdk("resizable_panels"), ResizeHandleMetrics::default())
    }
}

impl ResizablePanelsTheme for StyledResizablePanelsTheme {
    fn resize_handle_metrics(&self) -> ResizeHandleMetrics {
        self.handle_metrics
    }

    fn resolve(&self, state: InteractionState) -> ResizablePanelsLook {
        let states = [("hover", state.hovered), ("pressed", state.pressed), ("disabled", state.disabled)];
        let resolved = self.binding.resolve(&states, 1.0);
        let root = resolved.root();
        ResizablePanelsLook {
            border: color(root, "border"),
            border_width: number(root, "border_width"),
            divider_width: number(root, "divider_width"),
            divider: color(root, "divider"),
            grip: color(root, "grip"),
            grip_emphasis: color(root, "grip_emphasis"),
            disabled_opacity: if state.disabled {
                number(root, "disabled_opacity")
            } else {
                1.0
            },
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
                hitbox_width: number(root, "hitbox_width"),
                expand_inset_y: number(root, "expand_inset_y"),
                expand_cue_left: number(root, "expand_cue_left"),
                cue_inset_y: number(root, "cue_inset_y"),
                cue_radius: number(root, "cue_radius"),
                cue_width: number(root, "cue_width"),
                cue_hovered_width: number(root, "cue_hovered_width"),
            },
            separator: color(root, "separator"),
            separator_hover: color(root, "separator_hover"),
        }
    }
}
