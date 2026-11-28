//! Layout chrome read from `.lss` (issue #285): `scrollbar.lss`, `dock_splitter.lss`,
//! `resizable_panels.lss` and `split_view.lss`. Looks bind their own stylesheet and use these themes.
//!
//! Templates draw from each control's resolved root (#433); the Looks here are built from it for code that still
//! reads them.

use std::sync::Arc;

use crate::style::StyleStates;
use crate::style::PartRead;
use xgpui_luma_style::Resolved;

use super::dock_splitter::DockSplitterTheme;
use super::resizable_panels::{ResizablePanelsTheme, ResizeHandleMetrics};
use super::scrollbar::{ScrollbarOrientation, ScrollbarStyle, ScrollbarTheme};
use super::split_view::SplitViewTheme;
use crate::style::StyleBinding;
use crate::theme::stylesheet::{DockSplitterGeometry, SplitViewGeometry};
use crate::theme::{InteractionState, MetricTokens};

/// A scrollbar's style for `binding` (the `scrollbar` control) in `state` and `style` (#433): templates draw from
/// its root.
pub fn resolve_scrollbar(binding: &StyleBinding, state: InteractionState, style: ScrollbarStyle) -> Arc<Resolved> {
    let variant = match style {
        ScrollbarStyle::Ghost => "ghost",
        ScrollbarStyle::Soft => "soft",
    };
    binding.resolve_states_for(&[("variant", variant)], StyleStates::new(state), 1.0)
}

/// A standalone bar's length for `orientation`, from a scrollbar's resolved root.
pub fn scrollbar_length(resolved: &Resolved, orientation: ScrollbarOrientation) -> f32 {
    resolved.root().number(match orientation {
        ScrollbarOrientation::Horizontal => "length_h",
        ScrollbarOrientation::Vertical => "length_v",
    })
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
    fn style(&self) -> StyleBinding {
        self.binding.clone()
    }

    fn metrics(&self) -> MetricTokens {
        self.metrics
    }
}

/// A dock splitter's style for `binding` (the `dock_splitter` control), enabled or not (#433): templates draw
/// from its root.
pub fn resolve_dock_splitter(binding: &StyleBinding, enabled: bool) -> Arc<Resolved> {
    binding.resolve(&[("disabled", !enabled)], 1.0)
}

impl DockSplitterGeometry {
    /// The splitter geometry `resolved` (see [`resolve_dock_splitter`]) describes (#433).
    pub fn from_resolved(resolved: &Resolved) -> Self {
        let root = resolved.root();
        Self {
            hit_target: root.number("hit_target"),
            visible_line: root.number("visible_line"),
            thumb_width: root.number("thumb_width"),
            thumb_height: root.number("thumb_height"),
            thumb_radius: root.number("thumb_radius"),
            thumb_rest_opacity: root.number("thumb_rest_opacity"),
        }
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
    fn style(&self) -> StyleBinding {
        self.0.clone()
    }
}

/// Resizable panels' style for `binding` (the `resizable_panels` control) in `state` (#433): templates draw from
/// its root.
pub fn resolve_resizable_panels(binding: &StyleBinding, state: InteractionState) -> Arc<Resolved> {
    binding.resolve_states(StyleStates::new(state), 1.0)
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

    fn style(&self) -> StyleBinding {
        self.binding.clone()
    }
}

/// A split view's style for `binding` (the `split_view` control), enabled or not (#433): templates draw from its
/// root.
pub fn resolve_split_view(binding: &StyleBinding, enabled: bool) -> Arc<Resolved> {
    binding.resolve(&[("disabled", !enabled)], 1.0)
}

impl SplitViewGeometry {
    /// The separator geometry `resolved` (see [`resolve_split_view`]) describes (#433).
    pub fn from_resolved(resolved: &Resolved) -> Self {
        let root = resolved.root();
        Self {
            hitbox_width: root.number("hitbox_width"),
            expand_inset_y: root.number("expand_inset_y"),
            expand_cue_left: root.number("expand_cue_left"),
            cue_inset_y: root.number("cue_inset_y"),
            cue_radius: root.number("cue_radius"),
            cue_width: root.number("cue_width"),
            cue_hovered_width: root.number("cue_hovered_width"),
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
    fn style(&self) -> StyleBinding {
        self.0.clone()
    }
}
