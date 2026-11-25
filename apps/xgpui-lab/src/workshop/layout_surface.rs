//! SDK layout controls on the design surface page (issue #383): DockPanel (with a DockSplitter, the part
//! that has an `.lss`), Resizable Panels and Split View. Each `.lss` has one state, `:disabled`; Design mode
//! pins it through the control's theme. Each State matrix cell is a small control of its own, drawn in that
//! cell's states.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, Entity, SharedString, Subscription, Window, div, prelude::*, px};
use xgpui_luma::DockPanel;
use xgpui_luma::controls::dock_splitter::{
    DockSplitter, DockSplitterEvent, DockSplitterLook, DockSplitterTheme, SplitterOrientation, StyledDockSplitterTheme,
};
use xgpui_luma::controls::resizable_panels::{
    ResizablePanelSpec, ResizablePanels, ResizablePanelsBuilder, ResizablePanelsEvent, ResizablePanelsLook,
    ResizablePanelsTheme, ResizeHandleMetrics, StyledResizablePanelsTheme,
};
use xgpui_luma::controls::split_view::{
    SplitView, SplitViewBuilder, SplitViewEvent, SplitViewLook, SplitViewTheme, StyledSplitViewTheme,
};
use xgpui_luma::style::lss::Theme;
use xgpui_luma::style::{StyleBinding, ThemeSource};
use xgpui_luma::theme::{InteractionState, ThemeTokens};

use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl, SurfaceEvent};
use super::surface_page::ControlSide;
use crate::theme::Look;

/// A passive pane of content in the SDK's panel colors (layout contents, not interactive chrome).
fn pane(label: &'static str) -> gpui::Div {
    let palette = ThemeTokens::default().palette.surface.panel;
    div()
        .size_full()
        .min_w_0()
        .min_h_0()
        .p(px(12.))
        .bg(palette.background)
        .text_color(palette.foreground)
        .child(label)
}

fn binding(control: &'static str, theme: Theme) -> StyleBinding {
    StyleBinding::new(control, ThemeSource::Fixed(theme))
}

/// Whether a pinned control draws enabled: `None` follows the control.
fn pinned_enabled(pinned: Option<&StateSet>) -> Option<bool> {
    pinned.map(|states| !states.has("disabled"))
}

/// Matrix cells, made with the theme last bound and cleared when the page binds again.
#[derive(Default)]
struct Cells(RefCell<HashMap<SharedString, AnyView>>);

impl Cells {
    fn get(&self, id: &SharedString, make: impl FnOnce() -> AnyView) -> AnyView {
        self.0.borrow_mut().entry(id.clone()).or_insert_with(make).clone()
    }

    fn clear(&self) {
        self.0.borrow_mut().clear();
    }
}

// --- DockPanel ---------------------------------------------------------------------------------------------

struct PinnedDockSplitterTheme {
    base: StyledDockSplitterTheme,
    enabled: Option<bool>,
}

impl DockSplitterTheme for PinnedDockSplitterTheme {
    fn resolve(&self, enabled: bool) -> DockSplitterLook {
        self.base.resolve(self.enabled.unwrap_or(enabled))
    }
}

fn dock_splitter_theme(theme: Theme, enabled: Option<bool>) -> Arc<PinnedDockSplitterTheme> {
    Arc::new(PinnedDockSplitterTheme { base: StyledDockSplitterTheme::new(binding("dock_splitter", theme)), enabled })
}

const LEFT_WIDTH: f32 = 120.;

/// A toolbar, a library and a playground docked in order; the splitter resizes the library.
struct DockHost {
    splitter: Entity<DockSplitter>,
    left_width: f32,
    drag_start: f32,
}

impl DockHost {
    fn new(cx: &mut Context<Self>) -> Self {
        let splitter = DockSplitter::new("workshop-sdk-dock-splitter", SplitterOrientation::Vertical).spawn(cx);
        cx.subscribe(&splitter, |this: &mut Self, _, event: &DockSplitterEvent, cx| match event {
            DockSplitterEvent::ResizeStart => this.drag_start = this.left_width,
            DockSplitterEvent::Resize { total_delta } => {
                this.left_width = (this.drag_start + total_delta).clamp(60., 240.);
                cx.notify();
            }
            _ => {}
        })
        .detach();
        Self { splitter, left_width: LEFT_WIDTH, drag_start: LEFT_WIDTH }
    }
}

impl Render for DockHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().debug_selector(|| "workshop-layout-viewport".into()).w(px(460.)).h(px(240.)).child(
            DockPanel::new()
                .top(div().debug_selector(|| "workshop-dock-top".into()).h(px(40.)).flex_none().child(pane("Toolbar")))
                .left(
                    div()
                        .debug_selector(|| "workshop-dock-left".into())
                        .w(px(self.left_width))
                        .flex_none()
                        .child(pane("Library")),
                )
                .left(self.splitter.clone())
                .fill(div().debug_selector(|| "workshop-dock-fill".into()).size_full().child(pane("Playground"))),
        )
    }
}

pub(super) struct DockPanelSurface {
    host: Entity<DockHost>,
    splitter: Entity<DockSplitter>,
    cells: Cells,
}

impl DockPanelSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let host = cx.new(DockHost::new);
        let splitter = host.read(cx).splitter.clone();
        Box::new(Self { host, splitter, cells: Cells::default() })
    }
}

impl SurfaceControl for DockPanelSurface {
    fn title(&self) -> &'static str {
        "DockPanel"
    }

    fn view(&self) -> AnyView {
        self.host.clone().into()
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        let styled = dock_splitter_theme(theme, pinned_enabled(pinned));
        self.splitter.update(cx, |splitter, cx| splitter.set_theme(styled, cx));
        self.cells.clear();
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.splitter.update(cx, |splitter, cx| splitter.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        vec!["disabled"]
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.splitter, move |side: &mut ControlSide, _, event: &DockSplitterEvent, cx| {
            // Enabling is the page's own doing; every drag frame is noise.
            if matches!(event, DockSplitterEvent::EnabledChanged { .. } | DockSplitterEvent::Resize { .. }) {
                return;
            }
            on_event(side, SurfaceEvent { line: format!("{event:?}"), state: None }, cx);
        })
    }

    /// Two panes and the splitter between them.
    fn cell(&self, theme: &Theme, id: SharedString, states: &StateSet, _: &mut Window, cx: &mut App) -> AnyElement {
        let splitter = self.cells.get(&id, || {
            DockSplitter::new(id.clone(), SplitterOrientation::Vertical)
                .enabled(false)
                .theme(dock_splitter_theme(theme.clone(), pinned_enabled(Some(states))))
                .spawn(cx)
                .into()
        });
        div()
            .flex()
            .w(px(220.))
            .h(px(72.))
            .child(div().w(px(90.)).flex_none().child(pane("Library")))
            .child(splitter)
            .child(div().flex_1().child(pane("Playground")))
            .into_any_element()
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// --- Resizable Panels --------------------------------------------------------------------------------------

struct PinnedResizablePanelsTheme {
    base: StyledResizablePanelsTheme,
    enabled: Option<bool>,
}

impl ResizablePanelsTheme for PinnedResizablePanelsTheme {
    fn resize_handle_metrics(&self) -> ResizeHandleMetrics {
        self.base.resize_handle_metrics()
    }

    fn resolve(&self, state: InteractionState) -> ResizablePanelsLook {
        let disabled = self.enabled.map_or(state.disabled, |enabled| !enabled);
        self.base.resolve(InteractionState { disabled, ..state })
    }
}

fn resizable_theme(theme: Theme, enabled: Option<bool>) -> Arc<PinnedResizablePanelsTheme> {
    Arc::new(PinnedResizablePanelsTheme {
        base: StyledResizablePanelsTheme::new(binding("resizable_panels", theme)),
        enabled,
    })
}

fn panels(id: impl Into<SharedString>, first: &'static str, second: &'static str) -> ResizablePanelsBuilder {
    ResizablePanelsBuilder::new(id)
        .panel(
            ResizablePanelSpec::new_render(move || pane(first).debug_selector(|| "workshop-layout-first".into()))
                .min(px(60.)),
        )
        .panel(
            ResizablePanelSpec::new_render(move || pane(second).debug_selector(|| "workshop-layout-second".into()))
                .min(px(60.)),
        )
}

/// Gives the panels, which fill their parent, a size on the stage.
struct PanelsHost {
    panels: Entity<ResizablePanels>,
}

impl Render for PanelsHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .debug_selector(|| "workshop-layout-viewport".into())
            .w(px(460.))
            .h(px(240.))
            .child(self.panels.clone())
    }
}

pub(super) struct ResizablePanelsSurface {
    panels: Entity<ResizablePanels>,
    host: Entity<PanelsHost>,
    cells: Cells,
}

impl ResizablePanelsSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let panels = panels("workshop-sdk-resizable-panels", "Typography", "Display").spawn(cx);
        let host = cx.new(|_| PanelsHost { panels: panels.clone() });
        Box::new(Self { panels, host, cells: Cells::default() })
    }
}

impl SurfaceControl for ResizablePanelsSurface {
    fn title(&self) -> &'static str {
        "Resizable Panels"
    }

    fn view(&self) -> AnyView {
        self.host.clone().into()
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        let styled = resizable_theme(theme, pinned_enabled(pinned));
        self.panels.update(cx, |panels, cx| panels.set_theme(styled, cx));
        self.cells.clear();
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.panels.update(cx, |panels, cx| panels.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        vec!["disabled"]
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.panels, move |side: &mut ControlSide, _, event: &ResizablePanelsEvent, cx| {
            // Enabling is the page's own doing; size reports arrive every frame.
            if matches!(event, ResizablePanelsEvent::EnabledChanged { .. } | ResizablePanelsEvent::SizesChanged { .. })
            {
                return;
            }
            on_event(side, SurfaceEvent { line: format!("{event:?}"), state: None }, cx);
        })
    }

    fn cell(&self, theme: &Theme, id: SharedString, states: &StateSet, _: &mut Window, cx: &mut App) -> AnyElement {
        let panels = self.cells.get(&id, || {
            panels(id.clone(), "Typography", "Display")
                .enabled(false)
                .theme(resizable_theme(theme.clone(), pinned_enabled(Some(states))))
                .spawn(cx)
                .into()
        });
        div().w(px(220.)).h(px(72.)).child(panels).into_any_element()
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// --- Split View --------------------------------------------------------------------------------------------

struct PinnedSplitViewTheme {
    base: StyledSplitViewTheme,
    enabled: Option<bool>,
}

impl SplitViewTheme for PinnedSplitViewTheme {
    fn resolve(&self, hovered: bool, enabled: bool) -> SplitViewLook {
        self.base.resolve(hovered, self.enabled.unwrap_or(enabled))
    }
}

fn split_theme(theme: Theme, enabled: Option<bool>) -> Arc<PinnedSplitViewTheme> {
    Arc::new(PinnedSplitViewTheme { base: StyledSplitViewTheme::new(binding("split_view", theme)), enabled })
}

fn split(id: impl Into<SharedString>) -> SplitViewBuilder {
    SplitViewBuilder::new(id)
        .sidebar(|| pane("Projects").debug_selector(|| "workshop-layout-sidebar".into()))
        .content(|| pane("Typography preview").debug_selector(|| "workshop-layout-content".into()))
}

/// Gives the split view, which fills its parent, a size on the stage.
struct SplitHost {
    split: Entity<SplitView>,
}

impl Render for SplitHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .debug_selector(|| "workshop-layout-viewport".into())
            .w(px(460.))
            .h(px(240.))
            .child(self.split.clone())
    }
}

pub(super) struct SplitViewSurface {
    split: Entity<SplitView>,
    host: Entity<SplitHost>,
    cells: Cells,
}

impl SplitViewSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let split = split("workshop-sdk-split-view").spawn(cx);
        let host = cx.new(|_| SplitHost { split: split.clone() });
        Box::new(Self { split, host, cells: Cells::default() })
    }
}

impl SurfaceControl for SplitViewSurface {
    fn title(&self) -> &'static str {
        "Split View"
    }

    fn view(&self) -> AnyView {
        self.host.clone().into()
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        let styled = split_theme(theme, pinned_enabled(pinned));
        self.split.update(cx, |split, cx| split.set_theme(styled, cx));
        self.cells.clear();
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.split.update(cx, |split, cx| split.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        vec!["disabled"]
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.split, move |side: &mut ControlSide, _, event: &SplitViewEvent, cx| {
            // Enabling is the page's own doing; width reports arrive every frame.
            if matches!(event, SplitViewEvent::EnabledChanged { .. } | SplitViewEvent::SidebarWidthChanged { .. }) {
                return;
            }
            on_event(side, SurfaceEvent { line: format!("{event:?}"), state: None }, cx);
        })
    }

    fn cell(&self, theme: &Theme, id: SharedString, states: &StateSet, _: &mut Window, cx: &mut App) -> AnyElement {
        let split = self.cells.get(&id, || {
            split(id.clone())
                .sidebar_min_width(px(60.))
                .sidebar_width(px(90.))
                .enabled(false)
                .theme(split_theme(theme.clone(), pinned_enabled(Some(states))))
                .spawn(cx)
                .into()
        });
        div().w(px(240.)).h(px(72.)).child(split).into_any_element()
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::{MouseButton, TestAppContext, point};

    use super::*;
    use crate::workshop::surface_page::WorkMode;
    use crate::workshop::tests_support::open_surface;

    #[test]
    fn the_dock_panel_page_docks_in_order_and_its_splitter_resizes() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "dock-panel");
        let side = cx.update(|_, cx| page.read(cx).side());
        assert!(cx.update(|_, cx| side.read(cx).control_as::<DockPanelSurface>().is_some()));
        let top = cx.debug_bounds("workshop-dock-top").expect("top");
        let left = cx.debug_bounds("workshop-dock-left").expect("left");
        let fill = cx.debug_bounds("workshop-dock-fill").expect("fill");
        assert_eq!(top.size.height, px(40.));
        assert_eq!(left.size.width, px(LEFT_WIDTH));
        assert_eq!(fill.top(), top.bottom());
        assert!(fill.left() >= left.right(), "the splitter and fill follow the library");

        let start = point((left.right() + fill.left()) / 2., left.center().y);
        let end = start + point(px(30.), px(0.));
        cx.simulate_mouse_down(start, MouseButton::Left, Default::default());
        cx.simulate_mouse_move(end, MouseButton::Left, Default::default());
        cx.simulate_mouse_move(end, MouseButton::Left, Default::default());
        cx.simulate_mouse_up(end, MouseButton::Left, Default::default());
        cx.run_until_parked();
        assert!(
            cx.debug_bounds("workshop-dock-left").expect("left").size.width > left.size.width,
            "dragging resizes"
        );

        pin_disabled_and_show_matrix(&side, cx);
    }

    #[test]
    fn the_resizable_and_split_view_pages_resize_and_pin_disabled() {
        for (id, first, second) in [
            ("resizable-panels", "workshop-layout-first", "workshop-layout-second"),
            ("split-view-unified", "workshop-layout-sidebar", "workshop-layout-content"),
        ] {
            let mut app = TestAppContext::single();
            let (page, cx) = open_surface(&mut app, id);
            let side = cx.update(|_, cx| page.read(cx).side());
            let before = cx.debug_bounds(first).expect("first pane");
            let next = cx.debug_bounds(second).expect("second pane");
            let start = point((before.right() + next.left()) / 2., before.center().y);
            let end = start + point(px(32.), px(0.));
            cx.simulate_mouse_down(start, MouseButton::Left, Default::default());
            cx.simulate_mouse_move(end, MouseButton::Left, Default::default());
            cx.simulate_mouse_move(end, MouseButton::Left, Default::default());
            cx.simulate_mouse_up(end, MouseButton::Left, Default::default());
            cx.run_until_parked();
            assert!(cx.debug_bounds(first).expect("first pane").size.width > before.size.width, "{id} resizes");
            let text = cx.update(|_, cx| side.read(cx).events_text(cx));
            assert!(text.contains("ResizeEnd"), "{id} logs resizing: {text}");

            pin_disabled_and_show_matrix(&side, cx);
        }
    }

    fn pin_disabled_and_show_matrix(side: &Entity<ControlSide>, cx: &mut gpui::VisualTestContext) {
        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        side.update(cx, |side, cx| side.choose_interaction("disabled", cx));
        cx.run_until_parked();
        let pinned = cx.update(|_, cx| side.read(cx).pinned()).expect("Design draws picked states");
        assert_eq!(pinned, StateSet::new(["disabled"]));

        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "matrix cells draw the control");
    }
}
