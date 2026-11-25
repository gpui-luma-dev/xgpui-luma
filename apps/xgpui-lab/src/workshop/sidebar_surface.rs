//! The SDK sidebar on the design surface page (issue #383). Its `.lss` (`sidebar`) styles section labels
//! and navigation rows: hover, pressed, focused, disabled, and `selected` for the current page. Design mode
//! pins the picked states on every row; `:selected` keeps the current page, and without it no row is.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use gpui::{
    AnyElement, AnyView, App, Context, Div, Entity, Hsla, SharedString, Stateful, Subscription, Window, div, prelude::*,
};
use xgpui_luma::controls::scroll_container::ScrollContainer;
use xgpui_luma::controls::scrollbar::default_scrollbar_template;
use xgpui_luma::controls::sidebar::{
    self, SidebarControl, SidebarEvent, SidebarItemLook, SidebarNodeKind, SidebarNodeState, SidebarPanelRenderModel,
    SidebarPanelTemplate, SidebarPanelTemplateHandlers, SidebarRenderedNode, SidebarSectionLook, SidebarTheme,
    StyledSidebar, ThemedSidebarPanelTemplate,
};
use xgpui_luma::infra::icon::DisclosureIcons;
use xgpui_luma::style::lss::Theme;
use xgpui_luma::style::{StyleBinding, ThemeSource};
use xgpui_luma::theme::InteractionState;

use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl, SurfaceEvent};
use super::surface_page::ControlSide;
use crate::theme::Look;

/// The current page at the start.
const SELECTED: &str = "typography";
/// The group's section label.
const SECTION: &str = "Workspace";
/// The `.lss` states a row reads.
const STATES: [&str; 5] = ["hover", "pressed", "focused", "selected", "disabled"];
/// A matrix cell's rows: id and label.
const CELL_ROWS: [(&str, &str); 2] = [("typography", "Typography"), ("spacing", "Spacing")];

/// Gives the sidebar, which fills its parent, a size on the stage.
struct SidebarHost {
    sidebar: Entity<SidebarControl>,
}

impl Render for SidebarHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .debug_selector(|| "workshop-sidebar-viewport".into())
            .w(gpui::px(260.))
            .h(gpui::px(280.))
            .child(self.sidebar.clone())
    }
}

pub(super) struct SidebarSurface {
    sidebar: Entity<SidebarControl>,
    host: Entity<SidebarHost>,
    /// Each matrix cell's scroll region, kept across redraws.
    cell_scrolls: RefCell<HashMap<SharedString, ScrollContainer>>,
}

impl SidebarSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let menu = sidebar::sidebar_menu("destinations")
            .item(sidebar::sidebar_menu_item("typography", "Typography"))
            .item(
                sidebar::sidebar_menu_item("projects", "Projects").sub(
                    sidebar::sidebar_menu_sub()
                        .item(sidebar::sidebar_menu_item("playground", "Playground"))
                        .item(sidebar::sidebar_menu_item("library", "Library")),
                ),
            )
            .item(sidebar::sidebar_menu_item("legacy", "Legacy").disabled(true))
            .item(sidebar::sidebar_menu_item("settings", "Settings"));
        let panel = sidebar::sidebar("workshop-sidebar-panel")
            .content(sidebar::sidebar_content().group(sidebar::sidebar_group().label(SECTION).menu(menu)));
        let sidebar = sidebar::SidebarControlBuilder::new("workshop-sdk-sidebar")
            .selected_id(SELECTED)
            .sidebar(panel)
            .spawn(cx);
        let host = cx.new(|_| SidebarHost { sidebar: sidebar.clone() });
        Box::new(Self { sidebar, host, cell_scrolls: RefCell::default() })
    }

    /// The SDK sidebar, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn sidebar(&self) -> Entity<SidebarControl> {
        self.sidebar.clone()
    }
}

/// The sidebar from a theme's `sidebar` control, with the SDK's layout.
struct LssSidebarTheme(StyledSidebar);

impl SidebarTheme for LssSidebarTheme {
    fn foreground(&self) -> Hsla {
        self.0.foreground()
    }

    fn resolve_section(&self) -> SidebarSectionLook {
        self.0.section()
    }

    fn resolve_branch(&self, state: InteractionState) -> SidebarItemLook {
        self.0.branch(state)
    }

    fn resolve_item(&self, selected: bool, state: InteractionState) -> SidebarItemLook {
        self.0.item(selected, state)
    }
}

/// The sidebar template for the `sidebar` control in `theme`.
fn template(theme: Theme) -> ThemedSidebarPanelTemplate {
    let styled = StyledSidebar::new(StyleBinding::new("sidebar", ThemeSource::Fixed(theme)));
    ThemedSidebarPanelTemplate::new(Arc::new(LssSidebarTheme(styled)))
}

/// Puts `nodes` and their children in the picked states; `:selected` keeps the current page.
fn pin_nodes(nodes: &mut [SidebarRenderedNode], states: &StateSet) {
    for node in nodes {
        let state = &mut node.state;
        state.hovered = states.has("hover");
        state.pressed = states.has("pressed");
        state.focused = states.has("focused");
        state.enabled = !states.has("disabled");
        state.selected &= states.has("selected");
        pin_nodes(&mut node.children, states);
    }
}

/// `model` with every row in the picked states.
fn pinned_model(mut model: SidebarPanelRenderModel, states: &StateSet) -> SidebarPanelRenderModel {
    for nodes in [
        &mut model.header_nodes,
        &mut model.nodes,
        &mut model.footer_nodes,
        &mut model.rail_nodes,
        &mut model.rail_footer_nodes,
    ] {
        pin_nodes(nodes, states);
    }
    if !states.has("selected") {
        model.selected_id = None;
    }
    model
}

/// Draws the base template with the picked states (Design mode), or as the sidebar is.
struct PinnedSidebarTemplate {
    base: ThemedSidebarPanelTemplate,
    pinned: Option<StateSet>,
}

impl SidebarPanelTemplate for PinnedSidebarTemplate {
    fn render(
        &self,
        model: SidebarPanelRenderModel,
        main_scroll: &ScrollContainer,
        handlers: SidebarPanelTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let model = match &self.pinned {
            Some(states) => pinned_model(model, states),
            None => model,
        };
        self.base.render(model, main_scroll, handlers, window, cx)
    }
}

/// A matrix cell's node: a section label or a row.
fn cell_node(id: &str, kind: SidebarNodeKind, label: &str, index: usize, selected: bool) -> SidebarRenderedNode {
    SidebarRenderedNode {
        drop_layout: None,
        drag_drop: None,
        tooltip: None,
        id: id.to_string().into(),
        kind,
        label: Some(label.to_string().into()),
        icon: None,
        state: SidebarNodeState {
            id: id.to_string().into(),
            depth: 0,
            sidebar_collapsed: false,
            index,
            sibling_count: CELL_ROWS.len() + 1,
            selected,
            hovered: false,
            pressed: false,
            focused: false,
            expanded: false,
            enabled: true,
        },
        custom_element: None,
        focus_handle: None,
        has_children: false,
        expansion_progress: 0.,
        children_height_px: 0.,
        children: Vec::new(),
    }
}

impl SurfaceControl for SidebarSurface {
    fn title(&self) -> &'static str {
        "Sidebar"
    }

    fn view(&self) -> AnyView {
        self.host.clone().into()
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        let template = PinnedSidebarTemplate { base: template(theme), pinned: pinned.cloned() };
        self.sidebar.update(cx, |sidebar, cx| sidebar.set_panel_template(Arc::new(template), cx));
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.sidebar.update(cx, |sidebar, cx| sidebar.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        STATES.to_vec()
    }

    /// Whether a row is the current page.
    fn lasting_states(&self, cx: &App) -> Vec<(&'static str, bool)> {
        vec![("selected", self.sidebar.read(cx).selected_id().is_some())]
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.sidebar, move |side: &mut ControlSide, _, event: &SidebarEvent, cx| {
            // Enabling is the page's own doing (Design disables input), not something the person did.
            if matches!(event, SidebarEvent::EnabledChanged { .. }) {
                return;
            }
            let state = match event {
                SidebarEvent::HoverChanged { id } => Some(("hover", id.is_some())),
                SidebarEvent::Select { .. } => Some(("selected", true)),
                _ => None,
            };
            on_event(side, SurfaceEvent { line: format!("{event:?}"), state }, cx);
        })
    }

    /// The section label and two rows in `states`, the first one current when `:selected` is.
    fn cell(
        &self,
        theme: &Theme,
        id: SharedString,
        states: &StateSet,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let scroll = self
            .cell_scrolls
            .borrow_mut()
            .entry(id.clone())
            .or_insert_with(|| ScrollContainer::new(format!("{id}-scroll"), default_scrollbar_template(), cx))
            .clone();
        let mut nodes = vec![cell_node("section", SidebarNodeKind::Section, SECTION, 0, false)];
        nodes.extend(
            CELL_ROWS
                .iter()
                .enumerate()
                .map(|(index, (row, label))| cell_node(row, SidebarNodeKind::Item, label, index + 1, *row == SELECTED)),
        );
        let model = SidebarPanelRenderModel {
            id: id.clone(),
            title: None,
            subtitle: None,
            header_nodes: Vec::new(),
            nodes,
            footer_nodes: Vec::new(),
            rail_nodes: Vec::new(),
            rail_footer_nodes: Vec::new(),
            rail_submenu: None,
            selected_id: Some(SELECTED.into()),
            collapsed: false,
            disclosure_icons: DisclosureIcons::default(),
        };
        let model = pinned_model(model, states);
        let panel = template(theme.clone()).render(model, &scroll, SidebarPanelTemplateHandlers::default(), window, cx);
        div().w(gpui::px(200.)).h(gpui::px(104.)).child(panel).into_any_element()
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::{TestAppContext, point, px};

    use super::*;
    use crate::workshop::surface_page::WorkMode;
    use crate::workshop::tests_support::open_surface;

    #[test]
    fn the_sidebar_page_expands_skips_disabled_rows_and_pins_its_states() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "sidebar");
        let side = cx.update(|_, cx| page.read(cx).side());
        let sidebar = cx.update(|_, cx| side.read(cx).control_as::<SidebarSurface>().expect("a sidebar").sidebar());
        assert!(cx.update(|_, cx| side.read(cx).state_on("selected")), "a row starts current");

        let viewport = cx.debug_bounds("workshop-sidebar-viewport").expect("sidebar viewport");
        cx.simulate_click(point(viewport.left() + px(40.), viewport.top() + px(40.)), Default::default());
        cx.simulate_keystrokes("home down right down enter");
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(sidebar.read(cx).selected_id().map(|id| id.as_ref()), Some("playground")));
        let text = cx.update(|_, cx| side.read(cx).events_text(cx));
        assert!(text.contains("Select"), "selection is logged: {text}");

        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        side.update(cx, |side, cx| side.choose_interaction("pressed", cx));
        cx.run_until_parked();
        let pinned = cx.update(|_, cx| side.read(cx).pinned()).expect("Design draws picked states");
        assert!(pinned.has("pressed") && pinned.has("selected"), "{pinned:?}");

        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "matrix cells draw sidebars");
    }
}
