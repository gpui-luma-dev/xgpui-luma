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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::dock_splitter::DefaultDockSplitterTheme;
    use crate::controls::resizable_panels::DefaultResizablePanelsTheme;
    use crate::controls::scrollbar::DefaultScrollbarTheme;
    use crate::controls::split_view::DefaultSplitViewTheme;

    fn close(a: Hsla, b: Hsla) -> bool {
        let (a, b) = (a.to_rgb(), b.to_rgb());
        [(a.r, b.r), (a.g, b.g), (a.b, b.b), (a.a, b.a)].iter().all(|(x, y)| (x - y).abs() < 2e-3)
    }

    fn states() -> [InteractionState; 4] {
        [
            InteractionState::default(),
            InteractionState { hovered: true, ..Default::default() },
            InteractionState { pressed: true, ..Default::default() },
            InteractionState { disabled: true, ..Default::default() },
        ]
    }

    #[test]
    fn sdk_layout_matches_the_token_themes() {
        let (lss, native) = (StyledScrollbarTheme::sdk(), DefaultScrollbarTheme::default());
        for style in [ScrollbarStyle::Ghost, ScrollbarStyle::Soft] {
            for orientation in [ScrollbarOrientation::Horizontal, ScrollbarOrientation::Vertical] {
                for state in states() {
                    let (got, want) =
                        (lss.resolve(state, orientation, style), native.resolve(state, orientation, style));
                    let case = format!("{style:?} {orientation:?} {state:?}");
                    assert!(close(got.track_background, want.track_background), "{case} track");
                    assert!(close(got.thumb_background, want.thumb_background), "{case} thumb");
                    assert_eq!(
                        (got.length, got.thickness, got.track_thickness, got.thumb_thickness, got.min_thumb_length),
                        (
                            want.length,
                            want.thickness,
                            want.track_thickness,
                            want.thumb_thickness,
                            want.min_thumb_length
                        ),
                        "{case}"
                    );
                    assert_eq!((got.radius, got.disabled_opacity), (want.radius, want.disabled_opacity));
                }
            }
        }

        let (lss, native) = (StyledDockSplitterTheme::sdk(), DefaultDockSplitterTheme::default());
        for enabled in [true, false] {
            let (got, want) = (lss.resolve(enabled), native.resolve(enabled));
            assert!(close(got.line_color, want.line_color) && close(got.hover_color, want.hover_color));
            assert!(close(got.thumb_color, want.thumb_color));
            assert_eq!(got.geometry, want.geometry);
        }

        let (lss, native) = (StyledResizablePanelsTheme::sdk(), DefaultResizablePanelsTheme::default());
        for state in states() {
            let (got, want) = (lss.resolve(state), native.resolve(state));
            assert!(close(got.border, want.border) && close(got.divider, want.divider), "{state:?}");
            assert!(close(got.grip, want.grip) && close(got.grip_emphasis, want.grip_emphasis), "{state:?}");
            assert_eq!(
                (got.border_width, got.divider_width, got.disabled_opacity),
                (want.border_width, want.divider_width, want.disabled_opacity)
            );
        }
        assert_eq!(lss.resize_handle_metrics(), native.resize_handle_metrics());

        let (lss, native) = (StyledSplitViewTheme::sdk(), DefaultSplitViewTheme::default());
        for enabled in [true, false] {
            let (got, want) = (lss.resolve(false, enabled), native.resolve(false, enabled));
            assert!(close(got.separator, want.separator) && close(got.separator_hover, want.separator_hover));
            assert_eq!(got.geometry, want.geometry);
        }
    }
}
