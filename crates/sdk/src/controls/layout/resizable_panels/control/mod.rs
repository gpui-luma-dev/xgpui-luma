mod collapse;
mod input;
mod render;
mod sizes;
#[cfg(all(test, feature = "test-support"))]
mod tests;

use gpui::{Context, EventEmitter, FocusHandle, Hsla, IntoElement, Pixels, Render, SharedString, Subscription, Window};

use super::{
    math::{apply_pair_collapse_px, solve_layout_px_with_min_overrides},
    model::{
        PanelHideMode, PanelId, PanelLayoutState, PanelSize, ResizeHandleVisibility, ResizablePanelsBuilder,
        ResizablePanelsModel, ResizablePanelsOrientation,
    },
};
use crate::motion::{VisualTransition};

use self::collapse::CollapseRestoreState;
use self::sizes::swap_region_ids;

#[derive(Clone, Debug)]
pub struct ResizablePanelsHandleDrag {
    pub(crate) id: SharedString,
    pub(crate) handle_index: usize,
}

impl Render for ResizablePanelsHandleDrag {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ResizablePanelsEvent {
    ResizeStart,
    /// Current visible panel sizes, including animation frames and parent layout changes.
    /// Use this to keep external layout elements aligned with the panels.
    SizesChanged {
        sizes_px: Vec<f32>,
    },
    ResizeEnd {
        sizes_px: Vec<f32>,
    },
    PanelHiddenChanged {
        panel_index: usize,
        hidden: bool,
    },
    /// Emitted alongside [`Self::PanelHiddenChanged`] when the panel has an identity.
    /// Both events are retained so physical-index subscribers remain compatible.
    PanelRegionHiddenChanged {
        region: PanelId,
        panel_index: usize,
        hidden: bool,
    },
    HandleFocusChanged {
        handle_index: usize,
        focused: bool,
    },
    HandleHoverChanged {
        handle_index: usize,
        hovered: bool,
    },
    EnabledChanged {
        enabled: bool,
    },
}
pub struct ResizablePanels {
    model: ResizablePanelsModel,
    layout_states: Vec<PanelLayoutState>,
    collapsed_min_overrides: Vec<bool>,
    collapse_restore: Vec<Option<CollapseRestoreState>>,
    panel_hide_restore: Vec<Option<CollapseRestoreState>>,
    panel_sizes_px: Vec<f32>,
    handle_focuses: Vec<FocusHandle>,
    handle_focus_subscriptions: Vec<Subscription>,
    emitted_focused_handles: Vec<bool>,
    hovered_handle: Option<usize>,
    dragging_handle: Option<usize>,
    drag_start_axis_px: f32,
    drag_start_states: Vec<PanelLayoutState>,
    /// Content-axis size at drag start; stabilizes weight panes during handle moves.
    drag_content_axis_px: Option<f32>,
    measured_size: Option<gpui::Size<Pixels>>,
    transitions: Vec<VisualTransition>,
}

impl EventEmitter<ResizablePanelsEvent> for ResizablePanels {}

impl ResizablePanels {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> ResizablePanelsBuilder {
        ResizablePanelsBuilder::new(id)
    }

    pub fn horizontal(id: impl Into<SharedString>) -> ResizablePanelsBuilder {
        ResizablePanelsBuilder::horizontal(id)
    }

    pub fn vertical(id: impl Into<SharedString>) -> ResizablePanelsBuilder {
        ResizablePanelsBuilder::vertical(id)
    }

    crate::motion_snapshot!(super::ResizablePanelsMotion, model.motion);

    pub(crate) fn from_builder(builder: ResizablePanelsBuilder, cx: &mut Context<Self>) -> Self {
        let mut builder = builder;
        builder.model.motion.freeze();
        let motion = builder.model.motion.profile().clone();
        let layout_states: Vec<PanelLayoutState> =
            builder.model.panels.iter().map(PanelLayoutState::from_spec).collect();

        let layout_states = if layout_states.len() == builder.model.panels.len() {
            layout_states
        } else {
            vec![PanelLayoutState::Weight(1.0)]
        };

        let handle_count = builder.model.panels.len().saturating_sub(1);
        let handle_focuses = (0..handle_count).map(|_| cx.focus_handle()).collect();

        let collapsed_min_overrides = vec![false; builder.model.panels.len()];
        let collapse_restore = vec![None; handle_count];
        let panel_hide_restore = vec![None; builder.model.panels.len()];
        let panel_sizes_px =
            solve_layout_px_with_min_overrides(&builder.model.panels, &layout_states, &collapsed_min_overrides, 1.0);

        let transitions =
            vec![
                VisualTransition::with_transition(1.0, motion.extent.enabled(builder.model.motion_enabled));
                builder.model.panels.len()
            ];

        Self {
            model: builder.model,
            layout_states,
            collapsed_min_overrides,
            collapse_restore,
            panel_hide_restore,
            panel_sizes_px,
            handle_focuses,
            handle_focus_subscriptions: Vec::new(),
            emitted_focused_handles: vec![false; handle_count],
            hovered_handle: None,
            dragging_handle: None,
            drag_start_axis_px: 0.0,
            drag_start_states: Vec::new(),
            drag_content_axis_px: None,
            measured_size: None,
            transitions,
        }
    }

    /// True when the main layout axis uses an explicit frame size instead of prepaint measurement.
    fn main_axis_frame_pinned(&self) -> bool {
        match self.model.orientation {
            ResizablePanelsOrientation::Horizontal => self.model.frame_width.is_some(),
            ResizablePanelsOrientation::Vertical => self.model.frame_height.is_some(),
        }
    }

    pub fn set_measured_size(&mut self, size: gpui::Size<Pixels>, cx: &mut Context<Self>) {
        if self.main_axis_frame_pinned() {
            return;
        }
        if self.measured_size == Some(size) {
            return;
        }
        self.measured_size = Some(size);
        let previous_sizes = self.panel_sizes_px.clone();
        self.refresh_panel_sizes_px();
        if previous_sizes != self.panel_sizes_px {
            // Measurement runs during prepaint. Notify dependent chrome after layout,
            // just as animation frames defer their size events after render.
            let entity = cx.entity();
            cx.defer(move |cx| {
                entity.update(cx, |this, cx| {
                    cx.emit(ResizablePanelsEvent::SizesChanged { sizes_px: this.panel_sizes_px.clone() });
                });
            });
        }
        cx.notify();
    }

    /// Clears cached prepaint measurement so the next layout pass can re-measure the parent.
    pub fn invalidate_measured_size(&mut self, cx: &mut Context<Self>) {
        if self.main_axis_frame_pinned() {
            return;
        }
        if self.measured_size.take().is_some() {
            self.refresh_panel_sizes_px();
            cx.notify();
        }
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }
        self.model.enabled = enabled;
        if !enabled {
            if let Some(handle_index) = self.hovered_handle.take() {
                cx.emit(ResizablePanelsEvent::HandleHoverChanged { handle_index, hovered: false });
            }
            if self.dragging_handle.take().is_some() {
                cx.emit(ResizablePanelsEvent::ResizeEnd { sizes_px: self.panel_sizes_px.clone() });
            }
            self.drag_content_axis_px = None;
            for index in 0..self.emitted_focused_handles.len() {
                self.emit_handle_focus_changed(index, false, cx);
            }
            self.clear_all_restore_state(cx);
        }
        cx.emit(ResizablePanelsEvent::EnabledChanged { enabled });
        cx.notify();
    }

    pub fn set_show_handle(&mut self, show_handle: bool, cx: &mut Context<Self>) {
        let visibility = if show_handle {
            ResizeHandleVisibility::Always
        } else {
            ResizeHandleVisibility::Hidden
        };
        self.set_handle_visibility(visibility, cx);
    }

    pub fn set_handle_visibility(&mut self, visibility: ResizeHandleVisibility, cx: &mut Context<Self>) {
        if self.model.handle_visibility == visibility {
            return;
        }
        self.model.handle_visibility = visibility;
        cx.notify();
    }

    pub fn set_double_click_collapse(
        &mut self,
        behavior: Option<super::model::ResizeCollapseBehavior>,
        cx: &mut Context<Self>,
    ) {
        if self.model.double_click_collapse == behavior {
            return;
        }
        self.model.double_click_collapse = behavior;
        self.collapse_restore.fill(None);
        cx.notify();
    }

    pub fn set_theme(&mut self, theme: std::sync::Arc<dyn super::theme::ResizablePanelsTheme>, cx: &mut Context<Self>) {
        if std::sync::Arc::ptr_eq(&self.model.theme, &theme) {
            cx.notify();
            return;
        }
        self.model.theme = theme;
        cx.notify();
    }

    pub fn set_panel_background(&mut self, panel_index: usize, background: Option<Hsla>, cx: &mut Context<Self>) {
        let Some(spec) = self.model.panels.get_mut(panel_index) else {
            return;
        };
        if spec.background == background {
            return;
        }
        spec.background = background;
        cx.notify();
    }

    pub fn set_template(
        &mut self,
        template: std::sync::Arc<dyn super::template::ResizablePanelsTemplate>,
        cx: &mut Context<Self>,
    ) {
        self.model.template = template;
        cx.notify();
    }

    pub fn set_frame_size(&mut self, width: Pixels, height: Pixels, cx: &mut Context<Self>) {
        if self.model.frame_width == Some(width) && self.model.frame_height == Some(height) {
            return;
        }
        self.model.frame_width = Some(width);
        self.model.frame_height = Some(height);
        self.refresh_panel_sizes_px();
        cx.notify();
    }

    /// Sets weight coefficients for weight-only panel strips.
    pub fn set_weights(&mut self, weights: Vec<f32>, cx: &mut Context<Self>) {
        if weights.len() != self.model.panels.len() || weights.is_empty() {
            return;
        }
        if !self.model.panels.iter().all(|spec| matches!(spec.size, PanelSize::Weight(_))) {
            return;
        }

        let changed = self.layout_states.iter().zip(weights.iter()).any(|(current, next)| match current {
            PanelLayoutState::Weight(weight) => (weight - next).abs() >= f32::EPSILON,
            PanelLayoutState::Absolute(px) => (px - next).abs() >= f32::EPSILON,
        });
        if !changed {
            return;
        }

        for (state, weight) in self.layout_states.iter_mut().zip(weights.iter()) {
            *state = PanelLayoutState::Weight(weight.max(0.0));
        }
        self.clear_all_restore_state(cx);

        self.refresh_panel_sizes_px();
        cx.emit(ResizablePanelsEvent::SizesChanged { sizes_px: self.panel_sizes_px.clone() });
        cx.notify();
    }

    /// Main-axis pixel size of each panel after the current layout solve.
    pub fn panel_sizes_px(&self) -> Vec<f32> {
        self.panel_sizes_px.clone()
    }

    pub fn layout_states(&self) -> Vec<PanelLayoutState> {
        self.layout_states.clone()
    }

    pub fn orientation(&self) -> ResizablePanelsOrientation {
        self.model.orientation
    }

    pub fn enabled(&self) -> bool {
        self.model.enabled
    }

    pub fn motion_enabled(&self) -> bool {
        self.model.motion_enabled
    }

    pub fn set_motion_enabled(&mut self, motion_enabled: bool, cx: &mut Context<Self>) {
        if self.model.motion_enabled == motion_enabled {
            return;
        }
        self.model.motion_enabled = motion_enabled;
        for transition in &mut self.transitions {
            transition.set_transition(self.model.motion.profile().extent.enabled(motion_enabled));
        }
        cx.notify();
    }

    pub fn show_handle(&self) -> bool {
        self.model.handle_visibility == ResizeHandleVisibility::Always
    }

    pub fn handle_visibility(&self) -> ResizeHandleVisibility {
        self.model.handle_visibility
    }

    pub fn double_click_collapse(&self) -> Option<super::model::ResizeCollapseBehavior> {
        self.model.double_click_collapse
    }

    pub fn is_panel_hidden(&self, panel_index: usize) -> bool {
        self.panel_hide_restore.get(panel_index).is_some_and(Option::is_some)
    }

    pub fn panel_index_for_region(&self, region: &PanelId) -> Option<usize> {
        self.model.panels.iter().position(|panel| panel.panel_id.as_ref() == Some(region))
    }

    pub fn region_for_panel_index(&self, panel_index: usize) -> Option<PanelId> {
        self.model.panels.get(panel_index).and_then(|panel| panel.panel_id.clone())
    }

    pub fn is_region_hidden(&self, region: &PanelId) -> Option<bool> {
        self.panel_index_for_region(region).map(|panel_index| self.is_panel_hidden(panel_index))
    }

    /// Swaps two logical identities without changing physical panel layout state.
    ///
    /// This is useful when an application changes which edge owns a logical panel.
    /// Sizes, collapse state, and restore state remain attached to their physical slots.
    pub fn swap_regions(&mut self, first: &PanelId, second: &PanelId, cx: &mut Context<Self>) -> bool {
        if !swap_region_ids(&mut self.model.panels, first, second) {
            return false;
        }
        cx.notify();
        true
    }

    pub fn hide_region(&mut self, region: &PanelId, mode: PanelHideMode, cx: &mut Context<Self>) {
        if let Some(panel_index) = self.panel_index_for_region(region) {
            self.hide_panel(panel_index, mode, cx);
        }
    }

    pub fn show_region(&mut self, region: &PanelId, cx: &mut Context<Self>) {
        if let Some(panel_index) = self.panel_index_for_region(region) {
            self.show_panel(panel_index, cx);
        }
    }

    pub fn toggle_region_hidden(&mut self, region: &PanelId, mode: PanelHideMode, cx: &mut Context<Self>) {
        if let Some(panel_index) = self.panel_index_for_region(region) {
            self.toggle_panel_hidden(panel_index, mode, cx);
        }
    }

    /// Hides a panel and records its layout for a later [`Self::show_panel`] call.
    ///
    /// `PanelHideMode::ToMinSize` leaves the panel in the interactive resize model;
    /// `PanelHideMode::Completely` removes its visible extent and requires an
    /// explicit show operation for recovery.
    pub fn hide_panel(&mut self, panel_index: usize, mode: PanelHideMode, cx: &mut Context<Self>) {
        if panel_index >= self.layout_states.len() || self.is_panel_hidden(panel_index) {
            return;
        }
        let Some(handle_index) = self.hide_handle_index(panel_index) else {
            return;
        };
        self.clear_pair_restore_state(handle_index, cx);
        let target_px = match mode {
            PanelHideMode::ToMinSize => self.model.panels[panel_index].min_px.unwrap_or(0.0),
            PanelHideMode::Completely => 0.0,
        };
        let restore = self.current_restore_state(handle_index, panel_index);
        let content_axis_px = self.content_axis_size_px();
        let changed = apply_pair_collapse_px(
            &self.model.panels,
            &mut self.layout_states,
            &mut self.collapsed_min_overrides,
            handle_index,
            panel_index,
            target_px,
            mode == PanelHideMode::Completely,
            content_axis_px,
        );
        if !changed {
            return;
        }

        self.panel_hide_restore[panel_index] = Some(restore);
        self.transitions[panel_index].set_target(0.0);
        self.refresh_panel_sizes_px();
        let sizes_px = self.panel_sizes_px.clone();
        cx.emit(ResizablePanelsEvent::ResizeStart);
        if !self.model.motion_enabled {
            cx.emit(ResizablePanelsEvent::SizesChanged { sizes_px: sizes_px.clone() });
            cx.emit(ResizablePanelsEvent::ResizeEnd { sizes_px });
        }
        self.emit_panel_hidden_changed(panel_index, true, cx);
        cx.notify();
    }

    /// Restores a panel hidden with [`Self::hide_panel`] or collapsed by double-click.
    pub fn show_panel(&mut self, panel_index: usize, cx: &mut Context<Self>) {
        let restore = self.panel_hide_restore.get(panel_index).and_then(Option::as_ref).cloned().or_else(|| {
            self.collapse_restore.iter().flatten().find(|restore| restore.target_index == panel_index).cloned()
        });
        let Some(restore) = restore else {
            return;
        };
        if !self.is_panel_hidden(panel_index) {
            let motion_enabled = self.model.motion_enabled
                && self.model.panels.len() == 2
                && (self.transitions[panel_index].is_animating() || self.transitions[panel_index].progress() < 1.0);
            if self.apply_restore_state(restore, !motion_enabled, cx) && motion_enabled {
                self.transitions[panel_index].set_target(1.0);
                self.refresh_panel_sizes_px();
                cx.emit(ResizablePanelsEvent::ResizeStart);
            }
            return;
        }
        // When animated, suppress settle events here — render defers them when the transition ends.
        if self.apply_restore_state(restore, !self.model.motion_enabled, cx) {
            if let Some(panel_restore) = self.panel_hide_restore.get_mut(panel_index) {
                *panel_restore = None;
            }
            self.transitions[panel_index].set_target(1.0);
            self.refresh_panel_sizes_px();
            if self.model.motion_enabled {
                cx.emit(ResizablePanelsEvent::ResizeStart);
            }
            self.emit_panel_hidden_changed(panel_index, false, cx);
            cx.notify();
        }
    }

    pub fn toggle_panel_hidden(&mut self, panel_index: usize, mode: PanelHideMode, cx: &mut Context<Self>) {
        if self.is_panel_hidden(panel_index)
            || self.collapse_restore.iter().flatten().any(|restore| restore.target_index == panel_index)
        {
            self.show_panel(panel_index, cx);
        } else {
            self.hide_panel(panel_index, mode, cx);
        }
    }
}
