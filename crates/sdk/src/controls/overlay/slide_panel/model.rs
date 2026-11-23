use gpui::{Context, FocusHandle, KeyDownEvent, Point, Pixels, Window, px};

crate::motion_profile!(SlidePanelMotion => slide_panel {
    presence: crate::motion::PresenceMotion,
});

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlidePanelEdge {
    Left,
    Right,
    Top,
    Bottom,
}

impl SlidePanelEdge {
    pub fn label(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Top => "Top",
            Self::Bottom => "Bottom",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Left => "Navigation drawer pinned to the leading edge.",
            Self::Right => "Detail panel suited for inspectors and settings.",
            Self::Top => "Command sheet for short-lived announcements or filters.",
            Self::Bottom => "Mobile-style tray for actions and summaries.",
        }
    }

    pub fn is_horizontal_main_axis(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlidePanelTopAnchor {
    WindowEdge,
    BelowTopBar,
}

pub(super) const SUPPORTED_TOP_ANCHORS: [SlidePanelTopAnchor; 2] =
    [SlidePanelTopAnchor::WindowEdge, SlidePanelTopAnchor::BelowTopBar];

impl SlidePanelTopAnchor {
    pub fn inset(self) -> gpui::Pixels {
        match self {
            Self::WindowEdge => px(0.0),
            Self::BelowTopBar => crate::shell::TITLE_BAR_HEIGHT,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct SlidePanelResizeSession {
    start_pointer: f32,
    start_size: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct SlidePanelSizeConfig {
    size: f32,
    min: f32,
    max: f32,
}

impl SlidePanelSizeConfig {
    pub fn new(size: f32, min: f32, max: f32) -> Self {
        Self { size, min, max }
    }
}

pub struct SlidePanelState {
    active_edge: Option<SlidePanelEdge>,
    open_progress: f32,
    transition: crate::motion::VisualTransition,
    motion: crate::motion::MotionConfig<SlidePanelMotion>,
    closing: bool,
    restore_focus: Option<FocusHandle>,
    pending_panel_focus: bool,
    pending_focus_restore: bool,
    backdrop_click_closes: bool,
    top_anchor: SlidePanelTopAnchor,
    size_config: SlidePanelSizeConfig,
    resize_drag: Option<SlidePanelResizeSession>,
    resize_handle_hovered: bool,
    geometry: crate::theme::stylesheet::SlidePanelGeometryStylesheet,
    handle_metrics: crate::controls::resizable_panels::ResizeHandleMetrics,
}

impl SlidePanelState {
    crate::motion_methods!(SlidePanelMotion, motion);
    pub fn motion_enabled(mut self, enabled: bool) -> Self {
        self.set_motion_enabled(enabled);
        self
    }
    /// Control-wide policy. `false` settles motion immediately and keeps [`Self::motion_profile`].
    pub fn is_motion_enabled(&self) -> bool {
        self.motion.enabled()
    }

    pub fn set_motion_enabled(&mut self, enabled: bool) {
        self.motion.set_enabled(enabled);
        let profile = self.motion.resolve();
        self.transition.set_transition(
            if self.closing {
                profile.presence.exit
            } else {
                profile.presence.enter
            }
            .enabled(enabled),
        );
        self.sync_animation();
    }

    pub fn new(top_anchor: SlidePanelTopAnchor, size_config: SlidePanelSizeConfig) -> Self {
        Self {
            active_edge: None,
            open_progress: 0.0,
            transition: crate::motion::VisualTransition::with_transition(0.0, crate::motion::Transition::instant()),
            motion: Default::default(),
            closing: false,
            restore_focus: None,
            pending_panel_focus: false,
            pending_focus_restore: false,
            backdrop_click_closes: true,
            top_anchor,
            size_config,
            resize_drag: None,
            resize_handle_hovered: false,
            geometry: crate::theme::stylesheet::CommonStylesheet::sdk_default()
                .slide_panel
                .authored_geometry_for("overlay"),
            handle_metrics: crate::controls::resizable_panels::ResizeHandleMetrics::default(),
        }
    }

    /// Capture visual defaults independently of motion. Explicit size config still wins.
    pub fn stylesheet(mut self, stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        let baseline = crate::theme::stylesheet::CommonStylesheet::sdk_default()
            .slide_panel
            .authored_geometry_for("overlay");
        self.geometry = stylesheet.slide_panel.authored_geometry_for("overlay").with_fallback(&baseline);
        self
    }

    /// The resize edge's handle size (a look's resizable panels handle); the SDK's by default.
    pub fn handle_metrics(mut self, metrics: crate::controls::resizable_panels::ResizeHandleMetrics) -> Self {
        self.handle_metrics = metrics;
        self
    }

    /// Captured visual sizing; changes to the original stylesheet do not affect this state.
    pub fn geometry(&self) -> &crate::theme::stylesheet::SlidePanelGeometryStylesheet {
        &self.geometry
    }

    pub fn resize_handle_metrics(&self) -> crate::controls::resizable_panels::ResizeHandleMetrics {
        self.handle_metrics
    }

    pub fn set_size_config(&mut self, config: SlidePanelSizeConfig) {
        self.size_config = config;
    }

    pub fn main_axis_size(&self, fallback: f32) -> f32 {
        if self.size_config.size > 0.0 {
            self.size_config.size
        } else {
            fallback
        }
    }

    pub fn resize_handle_hovered(&self) -> bool {
        self.resize_handle_hovered
    }

    pub fn set_resize_handle_hovered(&mut self, hovered: bool) -> bool {
        if self.resize_handle_hovered == hovered {
            return false;
        }
        self.resize_handle_hovered = hovered;
        true
    }

    pub fn is_resizing(&self) -> bool {
        self.resize_drag.is_some()
    }

    pub fn begin_resize(&mut self, pointer: Point<Pixels>, edge: SlidePanelEdge) -> bool {
        self.resize_drag = Some(SlidePanelResizeSession {
            start_pointer: main_axis_pointer(pointer, edge),
            start_size: self.size_config.size,
        });
        true
    }

    pub fn update_resize(&mut self, pointer: Point<Pixels>, edge: SlidePanelEdge) -> bool {
        let Some(drag) = self.resize_drag else {
            return false;
        };

        let pointer_main = main_axis_pointer(pointer, edge);
        let delta = match edge {
            SlidePanelEdge::Right | SlidePanelEdge::Bottom => drag.start_pointer - pointer_main,
            SlidePanelEdge::Left | SlidePanelEdge::Top => pointer_main - drag.start_pointer,
        };
        let next = (drag.start_size + delta).clamp(self.size_config.min, self.size_config.max);
        if (next - self.size_config.size).abs() <= f32::EPSILON {
            return false;
        }
        self.size_config.size = next;
        true
    }

    pub fn finish_resize(&mut self) -> bool {
        self.resize_drag.take().is_some()
    }

    pub fn active_edge(&self) -> Option<SlidePanelEdge> {
        self.active_edge
    }

    pub fn open_progress(&self) -> f32 {
        self.open_progress
    }

    pub fn backdrop_click_closes(&self) -> bool {
        self.backdrop_click_closes
    }

    pub fn top_anchor(&self) -> SlidePanelTopAnchor {
        self.top_anchor
    }

    pub fn set_backdrop_click_closes(&mut self, backdrop_click_closes: bool) {
        self.backdrop_click_closes = backdrop_click_closes;
    }

    pub fn open(&mut self, edge: SlidePanelEdge, opener: FocusHandle) {
        self.restore_focus = Some(opener);
        self.active_edge = Some(edge);
        self.pending_panel_focus = true;
        self.pending_focus_restore = false;
        self.start_animation(1.0);
    }

    pub fn request_close(&mut self) -> bool {
        if self.active_edge.is_none() && self.open_progress <= 0.0 {
            return false;
        }

        self.pending_panel_focus = false;
        self.pending_focus_restore = self.restore_focus.is_some();
        self.start_animation(0.0);
        true
    }

    pub fn handle_escape<T>(&mut self, window: &mut Window, cx: &mut Context<T>) -> bool
    where
        T: 'static,
    {
        if self.active_edge.is_none() {
            return false;
        }

        if !self.request_close() {
            return false;
        }

        window.prevent_default();
        cx.stop_propagation();
        true
    }

    pub fn handle_tab_navigation<T>(
        &self,
        primary_focus: FocusHandle,
        extra_focuses: &[FocusHandle],
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<T>,
    ) -> bool
    where
        T: 'static,
    {
        if self.active_edge.is_none() {
            return false;
        }

        let mut focus_handles = Vec::with_capacity(extra_focuses.len() + 1);
        focus_handles.push(primary_focus);
        focus_handles.extend(extra_focuses.iter().cloned());

        let backwards = event.keystroke.modifiers.shift;
        let focused_index = focus_handles.iter().position(|handle| handle.is_focused(window));
        let next_index = match (focused_index, backwards) {
            (Some(0), true) => focus_handles.len() - 1,
            (Some(index), true) => index.saturating_sub(1),
            (Some(index), false) => (index + 1) % focus_handles.len(),
            (None, true) => focus_handles.len() - 1,
            (None, false) => 0,
        };

        focus_handles[next_index].focus(window, cx);
        window.prevent_default();
        cx.stop_propagation();
        true
    }

    pub fn sync_animation(&mut self) -> bool {
        let was_animating = self.transition.is_animating();
        self.transition.sync();
        self.open_progress = self.transition.progress();
        if !self.transition.is_animating() && self.closing {
            self.active_edge = None;
        }
        was_animating
    }

    pub fn schedule_animation_frame<T: 'static>(&self, window: &mut Window, cx: &mut Context<T>) {
        self.transition.schedule_frame(window, cx);
    }

    pub fn schedule_pending_focus<T>(&mut self, window: &mut Window, cx: &mut Context<T>, primary_focus: FocusHandle)
    where
        T: 'static,
    {
        if self.pending_panel_focus && !self.transition.is_animating() && !self.closing {
            self.pending_panel_focus = false;
            cx.on_next_frame(window, move |_this, window, cx| {
                primary_focus.focus(window, cx);
            });
        }

        if self.pending_focus_restore && self.active_edge.is_none() && self.open_progress <= 0.001 {
            self.pending_focus_restore = false;
            if let Some(focus) = self.restore_focus.clone() {
                cx.on_next_frame(window, move |_this, window, cx| {
                    focus.focus(window, cx);
                });
            }
        }
    }

    fn start_animation(&mut self, target: f32) {
        self.motion.freeze();
        self.closing = target == 0.0;
        let profile = self.motion.profile();
        let definition = if self.closing {
            &profile.presence.exit
        } else {
            &profile.presence.enter
        };
        self.transition.set_transition(definition.enabled(self.motion.enabled()));
        self.transition.set_target(target);
        self.open_progress = self.transition.progress();
        if self.closing && !self.transition.is_animating() {
            self.active_edge = None;
        }
    }
}

fn main_axis_pointer(pointer: Point<Pixels>, edge: SlidePanelEdge) -> f32 {
    match edge {
        SlidePanelEdge::Left | SlidePanelEdge::Right => pointer.x.as_f32(),
        SlidePanelEdge::Top | SlidePanelEdge::Bottom => pointer.y.as_f32(),
    }
}

#[cfg(all(test, feature = "test-support"))]
mod motion_tests {
    use super::*;
    use std::time::Duration;
    #[test]
    fn default_duration_and_custom_exit_preserve_presence_lifecycle() {
        let mut state =
            SlidePanelState::new(SlidePanelTopAnchor::WindowEdge, SlidePanelSizeConfig::new(360., 200., 500.));
        state.active_edge = Some(SlidePanelEdge::Left);
        state.transition.elapse_for_test(Duration::ZERO);
        state.start_animation(1.);
        state.transition.elapse_for_test(Duration::from_millis(110));
        state.sync_animation();
        assert_eq!(state.open_progress(), 0.875);
        state.request_close();
        assert!(state.active_edge().is_some());
        state.transition.elapse_for_test(Duration::from_millis(220));
        state.sync_animation();
        assert!(state.active_edge().is_none());
        let mut state = state.motion(|m| m.presence.exit = crate::motion::Transition::instant());
        state.active_edge = Some(SlidePanelEdge::Right);
        state.start_animation(1.);
        state.transition.elapse_for_test(Duration::from_millis(220));
        state.sync_animation();
        state.request_close();
        assert!(state.active_edge().is_none());
        assert!(!state.transition.is_animating());
    }
}
