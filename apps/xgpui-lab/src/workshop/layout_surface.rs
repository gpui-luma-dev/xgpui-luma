//! SDK layout controls on the design surface page (issue #383): DockPanel (with DockSplitters, the part that
//! has an `.lss`), Resizable Panels and Split View. Splitters and dividers draw their grips (thumbs), as Luma
//! Studio shows them, so the thumb styling is on the stage. Each `.lss` has one state, `:disabled`; Design mode
//! pins it through the control's theme. Each State matrix cell is a small control of its own, drawn in that
//! cell's states.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, Entity, SharedString, Subscription, Window, div, prelude::*, px};
use xgpui_luma::DockPanel;
use xgpui_luma::controls::dock_splitter::{
    DockSplitter, DockSplitterBuilder, DockSplitterEvent, DockSplitterTheme, SplitterOrientation,
    StyledDockSplitterTheme, ThemedDockSplitterTemplate,
};
use xgpui_luma::controls::resizable_panels::{
    ResizablePanelSpec, ResizablePanels, ResizablePanelsBuilder, ResizablePanelsEvent, ResizablePanelsTheme,
    ResizeHandleMetrics, StyledResizablePanelsTheme,
};
use xgpui_luma::controls::split_view::{SplitView, SplitViewBuilder, SplitViewEvent, SplitViewTheme, StyledSplitViewTheme};
use xgpui_luma::style::lss::Theme;
use xgpui_luma::style::{StyleBinding, ThemeSource};
use xgpui_luma::theme::ThemeTokens;

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

/// `binding` drawn disabled when `enabled` pins it off. Binding states only add, so a pinned enabled state draws as
/// the control is.
fn pin_disabled(binding: StyleBinding, enabled: Option<bool>) -> StyleBinding {
    if enabled == Some(false) {
        binding.state("disabled", true)
    } else {
        binding
    }
}

// --- DockPanel ---------------------------------------------------------------------------------------------

struct PinnedDockSplitterTheme {
    base: StyledDockSplitterTheme,
    enabled: Option<bool>,
}

impl DockSplitterTheme for PinnedDockSplitterTheme {
    /// The base style, drawn disabled when the picked state is (#433).
    fn style(&self) -> StyleBinding {
        pin_disabled(self.base.style(), self.enabled)
    }
}

fn dock_splitter_theme(theme: Theme, enabled: Option<bool>) -> Arc<PinnedDockSplitterTheme> {
    Arc::new(PinnedDockSplitterTheme { base: StyledDockSplitterTheme::new(binding("dock_splitter", theme)), enabled })
}

/// The library's and the console's sizes at rest, and how far a drag can take them.
const LEFT_WIDTH: f32 = 140.;
const BOTTOM_HEIGHT: f32 = 80.;
const LEFT_RANGE: (f32, f32) = (80., 260.);
const BOTTOM_RANGE: (f32, f32) = (40., 160.);
/// The docked layout's size on the stage.
const DOCK_SIZE: (f32, f32) = (520., 300.);
/// The split view's size on the stage, at actual size.
const SPLIT_SIZE: (f32, f32) = (460., 240.);

/// A splitter drawn with its grip (the thumb), as Luma Studio shows them.
fn grip_splitter(id: impl Into<SharedString>, orientation: SplitterOrientation) -> DockSplitterBuilder {
    DockSplitter::new(id, orientation).template(Arc::new(ThemedDockSplitterTemplate::new(true)))
}

/// A toolbar, a library, a console and a playground, docked in order; a splitter (with its grip) resizes
/// the library and another the console, so both orientations show.
struct DockHost {
    left_splitter: Entity<DockSplitter>,
    bottom_splitter: Entity<DockSplitter>,
    left_width: f32,
    bottom_height: f32,
    drag_start: f32,
}

impl DockHost {
    fn new(cx: &mut Context<Self>) -> Self {
        let left_splitter = grip_splitter("workshop-sdk-dock-splitter", SplitterOrientation::Vertical).spawn(cx);
        let bottom_splitter =
            grip_splitter("workshop-sdk-dock-bottom-splitter", SplitterOrientation::Horizontal).spawn(cx);
        cx.subscribe(&left_splitter, |this: &mut Self, _, event: &DockSplitterEvent, cx| match event {
            DockSplitterEvent::ResizeStart => this.drag_start = this.left_width,
            DockSplitterEvent::Resize { total_delta } => {
                this.left_width = (this.drag_start + total_delta).clamp(LEFT_RANGE.0, LEFT_RANGE.1);
                cx.notify();
            }
            _ => {}
        })
        .detach();
        // The console sits below its splitter, so dragging down shrinks it.
        cx.subscribe(&bottom_splitter, |this: &mut Self, _, event: &DockSplitterEvent, cx| match event {
            DockSplitterEvent::ResizeStart => this.drag_start = this.bottom_height,
            DockSplitterEvent::Resize { total_delta } => {
                this.bottom_height = (this.drag_start - total_delta).clamp(BOTTOM_RANGE.0, BOTTOM_RANGE.1);
                cx.notify();
            }
            _ => {}
        })
        .detach();
        Self { left_splitter, bottom_splitter, left_width: LEFT_WIDTH, bottom_height: BOTTOM_HEIGHT, drag_start: 0. }
    }
}

impl Render for DockHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().debug_selector(|| "workshop-layout-viewport".into()).size_full().child(
            DockPanel::new()
                .top(div().debug_selector(|| "workshop-dock-top".into()).h(px(40.)).flex_none().child(pane("Toolbar")))
                .left(
                    div()
                        .debug_selector(|| "workshop-dock-left".into())
                        .w(px(self.left_width))
                        .flex_none()
                        .child(pane("Library")),
                )
                .left(self.left_splitter.clone())
                .bottom(
                    div()
                        .debug_selector(|| "workshop-dock-bottom".into())
                        .h(px(self.bottom_height))
                        .flex_none()
                        .child(pane("Console")),
                )
                .bottom(self.bottom_splitter.clone())
                .fill(div().debug_selector(|| "workshop-dock-fill".into()).size_full().child(pane("Playground"))),
        )
    }
}

pub(super) struct DockPanelSurface {
    host: Entity<DockHost>,
    splitters: [Entity<DockSplitter>; 2],
    cells: Cells,
}

impl DockPanelSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let host = cx.new(DockHost::new);
        let splitters = {
            let host = host.read(cx);
            [host.left_splitter.clone(), host.bottom_splitter.clone()]
        };
        Box::new(Self { host, splitters, cells: Cells::default() })
    }
}

impl SurfaceControl for DockPanelSurface {
    fn title(&self) -> &'static str {
        "DockPanel"
    }

    fn view(&self) -> AnyView {
        self.host.clone().into()
    }

    /// The host fills the size the stage gives it, which can be dragged.
    fn stage_width(&self) -> Option<f32> {
        Some(DOCK_SIZE.0)
    }

    fn stage_height(&self) -> Option<f32> {
        Some(DOCK_SIZE.1)
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        for splitter in &self.splitters {
            let styled = dock_splitter_theme(theme.clone(), pinned_enabled(pinned));
            splitter.update(cx, |splitter, cx| splitter.set_theme(styled, cx));
        }
        self.cells.clear();
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        for splitter in &self.splitters {
            splitter.update(cx, |splitter, cx| splitter.set_enabled(enabled, cx));
        }
    }

    fn template_states(&self) -> Vec<&'static str> {
        vec!["disabled"]
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        let follow = |splitter: &Entity<DockSplitter>, name: &'static str, cx: &mut Context<ControlSide>| {
            cx.subscribe(splitter, move |side: &mut ControlSide, _, event: &DockSplitterEvent, cx| {
                // Enabling is the page's own doing; every drag frame is noise.
                if matches!(event, DockSplitterEvent::EnabledChanged { .. } | DockSplitterEvent::Resize { .. }) {
                    return;
                }
                on_event(side, SurfaceEvent { line: format!("{name}: {event:?}"), state: None }, cx);
            })
        };
        let left = follow(&self.splitters[0], "library", cx);
        let bottom = follow(&self.splitters[1], "console", cx);
        Subscription::join(left, bottom)
    }

    /// A library, a playground and the splitter (with its grip) between them.
    fn cell(&self, theme: &Theme, id: SharedString, states: &StateSet, _: &mut Window, cx: &mut App) -> AnyElement {
        let splitter = self.cells.get(&id, || {
            grip_splitter(id.clone(), SplitterOrientation::Vertical)
                .enabled(false)
                .theme(dock_splitter_theme(theme.clone(), pinned_enabled(Some(states))))
                .spawn(cx)
                .into()
        });
        div()
            .flex()
            .w(px(240.))
            .h(px(96.))
            .child(div().w(px(100.)).flex_none().child(pane("Library")))
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

    /// The base style, drawn disabled when the picked state is (#433).
    fn style(&self) -> StyleBinding {
        pin_disabled(self.base.style(), self.enabled)
    }
}

fn resizable_theme(theme: Theme, enabled: Option<bool>) -> Arc<PinnedResizablePanelsTheme> {
    Arc::new(PinnedResizablePanelsTheme {
        base: StyledResizablePanelsTheme::new(binding("resizable_panels", theme)),
        enabled,
    })
}

/// Two panels split by a divider drawn with its grip, always shown, as Luma Studio shows them.
fn panels(id: impl Into<SharedString>, first: &'static str, second: &'static str) -> ResizablePanelsBuilder {
    ResizablePanelsBuilder::new(id)
        .show_handle(true)
        .handle_grip(true)
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
        div().debug_selector(|| "workshop-layout-viewport".into()).size_full().child(self.panels.clone())
    }
}

pub(super) struct ResizablePanelsSurface {
    /// The outer, side-by-side split and the stacked split nested in its second panel.
    panels: [Entity<ResizablePanels>; 2],
    host: Entity<PanelsHost>,
    cells: Cells,
}

impl ResizablePanelsSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let stacked = ResizablePanelsBuilder::vertical("workshop-sdk-resizable-stacked")
            .show_handle(true)
            .handle_grip(true)
            .panel(ResizablePanelSpec::new_render(|| pane("Display")).min(px(40.)))
            .panel(ResizablePanelSpec::new_render(|| pane("Spacing")).min(px(40.)))
            .spawn(cx);
        let nested = stacked.clone();
        let outer = ResizablePanelsBuilder::new("workshop-sdk-resizable-panels")
            .show_handle(true)
            .handle_grip(true)
            .panel(
                ResizablePanelSpec::new_render(|| pane("Typography").debug_selector(|| "workshop-layout-first".into()))
                    .weight(2.)
                    .min(px(60.)),
            )
            .panel(
                ResizablePanelSpec::new_render(move || {
                    div().debug_selector(|| "workshop-layout-second".into()).size_full().child(nested.clone())
                })
                .weight(3.)
                .min(px(60.)),
            )
            .spawn(cx);
        let host = cx.new(|_| PanelsHost { panels: outer.clone() });
        Box::new(Self { panels: [outer, stacked], host, cells: Cells::default() })
    }
}

impl SurfaceControl for ResizablePanelsSurface {
    fn title(&self) -> &'static str {
        "Resizable Panels"
    }

    fn view(&self) -> AnyView {
        self.host.clone().into()
    }

    /// The host fills the size the stage gives it, which can be dragged.
    fn stage_width(&self) -> Option<f32> {
        Some(DOCK_SIZE.0)
    }

    fn stage_height(&self) -> Option<f32> {
        Some(DOCK_SIZE.1)
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        for panels in &self.panels {
            let styled = resizable_theme(theme.clone(), pinned_enabled(pinned));
            panels.update(cx, |panels, cx| panels.set_theme(styled, cx));
        }
        self.cells.clear();
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        for panels in &self.panels {
            panels.update(cx, |panels, cx| panels.set_enabled(enabled, cx));
        }
    }

    fn template_states(&self) -> Vec<&'static str> {
        vec!["disabled"]
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        let follow = |panels: &Entity<ResizablePanels>, name: &'static str, cx: &mut Context<ControlSide>| {
            cx.subscribe(panels, move |side: &mut ControlSide, _, event: &ResizablePanelsEvent, cx| {
                // Enabling is the page's own doing; size reports arrive every frame.
                if matches!(
                    event,
                    ResizablePanelsEvent::EnabledChanged { .. } | ResizablePanelsEvent::SizesChanged { .. }
                ) {
                    return;
                }
                on_event(side, SurfaceEvent { line: format!("{name}: {event:?}"), state: None }, cx);
            })
        };
        let outer = follow(&self.panels[0], "side by side", cx);
        let stacked = follow(&self.panels[1], "stacked", cx);
        Subscription::join(outer, stacked)
    }

    fn cell(&self, theme: &Theme, id: SharedString, states: &StateSet, _: &mut Window, cx: &mut App) -> AnyElement {
        let panels = self.cells.get(&id, || {
            panels(id.clone(), "Typography", "Display")
                .enabled(false)
                .theme(resizable_theme(theme.clone(), pinned_enabled(Some(states))))
                .spawn(cx)
                .into()
        });
        div().w(px(240.)).h(px(96.)).child(panels).into_any_element()
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
    /// The base style, drawn disabled when the picked state is (#433).
    fn style(&self) -> StyleBinding {
        pin_disabled(self.base.style(), self.enabled)
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
        div().debug_selector(|| "workshop-layout-viewport".into()).size_full().child(self.split.clone())
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

    /// The host fills the size the stage gives it, which can be dragged.
    fn stage_width(&self) -> Option<f32> {
        Some(SPLIT_SIZE.0)
    }

    fn stage_height(&self) -> Option<f32> {
        Some(SPLIT_SIZE.1)
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

        // The console docks at the bottom under its own splitter; dragging that splitter up grows it.
        let bottom = cx.debug_bounds("workshop-dock-bottom").expect("console");
        let fill = cx.debug_bounds("workshop-dock-fill").expect("fill");
        assert_eq!(bottom.size.height, px(BOTTOM_HEIGHT));
        let start = point(fill.center().x, (fill.bottom() + bottom.top()) / 2.);
        let end = start - point(px(0.), px(30.));
        cx.simulate_mouse_down(start, MouseButton::Left, Default::default());
        cx.simulate_mouse_move(end, MouseButton::Left, Default::default());
        cx.simulate_mouse_move(end, MouseButton::Left, Default::default());
        cx.simulate_mouse_up(end, MouseButton::Left, Default::default());
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-dock-bottom").expect("console").size.height > bottom.size.height, "grows");
        let text = cx.update(|_, cx| side.read(cx).events_text(cx));
        assert!(text.contains("console: ResizeEnd"), "each splitter's events are labelled: {text}");

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
