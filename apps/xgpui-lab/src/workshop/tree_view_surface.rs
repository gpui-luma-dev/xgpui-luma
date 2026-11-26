//! The SDK tree view on the design surface page (issue #383). Its `.lss` (`tree_view`) styles a row: hover,
//! pressed, disabled, and `selected` for selected nodes. Design mode pins the picked states on every row
//! through the tree's theme; `:selected` keeps the selection, and without it no row is selected. A tree lays
//! out a virtualized list, so each State matrix cell is a small tree of its own, drawn in that cell's states.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use gpui::{AnyElement, AnyView, App, Context, Entity, Hsla, SharedString, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::tree_view::{
    StyledTreeView, ThemedTreeViewTemplate, TreeNode, TreeView, TreeViewBuilder, TreeViewEvent, TreeViewPalette,
    TreeViewScale, TreeViewTheme,
};
use xgpui_luma::style::lss::Theme;
use xgpui_luma::style::{StyleBinding, ThemeSource};
use xgpui_luma::theme::{InteractionState, MetricTokens};

use super::surface_control::{OnSurfaceEvent, StateSet, SurfaceControl, SurfaceEvent};
use super::surface_page::ControlSide;
use crate::theme::Look;

/// The `.lss` states a row reads, and `focused` for the keyboard outline.
const STATES: [&str; 5] = ["hover", "pressed", "focused", "selected", "disabled"];

/// The live tree: folders and files, with ascenders and descenders.
fn nodes() -> Vec<TreeNode<()>> {
    vec![
        TreeNode::new("projects", "Projects", ())
            .children([
                TreeNode::new("budget", "Quarterly budget.xlsx", ()),
                TreeNode::new("playground", "Playground", ()).children([
                    TreeNode::new("display", "Display.svg", ()),
                    TreeNode::new("spacing", "Spacing.svg", ()),
                ]),
            ])
            .expanded(true),
        TreeNode::new("legacy", "Legacy photos", ()).children([TreeNode::new("holiday", "Holiday.jpg", ())]),
    ]
}

/// A matrix cell's tree: one folder, open, with two files.
fn cell_nodes() -> Vec<TreeNode<()>> {
    vec![
        TreeNode::new("projects", "Projects", ())
            .children([
                TreeNode::new("budget", "Quarterly budget.xlsx", ()),
                TreeNode::new("typography", "Typography.pdf", ()),
            ])
            .expanded(true),
    ]
}

/// How a pinned tree draws `:selected`: keeping the tree's own selection, or on every row (a matrix cell,
/// whose tree has none).
#[derive(Clone, Copy)]
enum Selected {
    Keep,
    Every,
}

/// The tree from a theme's `tree_view` control, every row in the picked states when some are pinned.
struct PinnedTreeTheme {
    base: StyledTreeView,
    pinned: Option<(StateSet, Selected)>,
}

impl PinnedTreeTheme {
    fn new(theme: Theme, pinned: Option<(StateSet, Selected)>) -> Self {
        Self { base: StyledTreeView::new(StyleBinding::new("tree_view", ThemeSource::Fixed(theme))), pinned }
    }

    fn state(&self, state: InteractionState) -> InteractionState {
        self.pinned.as_ref().map_or(state, |(states, _)| states.interaction())
    }
}

impl TreeViewTheme for PinnedTreeTheme {
    fn resolve_row(&self, state: InteractionState, selected: bool) -> TreeViewPalette {
        let selected = match &self.pinned {
            Some((states, Selected::Keep)) => selected && states.has("selected"),
            Some((states, Selected::Every)) => states.has("selected"),
            None => selected,
        };
        self.base.row(self.state(state), selected)
    }

    fn row_outline(&self, state: InteractionState) -> Option<Hsla> {
        self.base.outline(self.state(state))
    }

    fn metrics(&self) -> MetricTokens {
        MetricTokens::default()
    }

    /// In Design mode the page turns the tree's input off, which fades every row; only a picked `:disabled`
    /// fades them (#397).
    fn resolve_scale(&self, scale_factor: f32) -> TreeViewScale {
        let scale = self.base.scale(scale_factor);
        match &self.pinned {
            Some((states, _)) if !states.has("disabled") => TreeViewScale { disabled_opacity: 1.0, ..scale },
            _ => scale,
        }
    }
}

fn template(theme: PinnedTreeTheme) -> Arc<ThemedTreeViewTemplate> {
    Arc::new(ThemedTreeViewTemplate::new(Arc::new(theme)))
}

/// The tree's size on the stage, at actual size.
const STAGE_SIZE: (f32, f32) = (300., 220.);

/// Gives the tree, which fills its parent, a size on the stage.
struct TreeHost {
    tree: TreeView<()>,
}

impl Render for TreeHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.tree.clone())
    }
}

pub(super) struct TreeViewSurface {
    tree: TreeView<()>,
    host: Entity<TreeHost>,
    /// Each matrix cell's tree, made with the theme last bound.
    cells: RefCell<HashMap<SharedString, TreeView<()>>>,
}

impl TreeViewSurface {
    pub(super) fn make(_look: &Arc<Look>, cx: &mut Context<ControlSide>) -> Box<dyn SurfaceControl> {
        let tree = TreeViewBuilder::new("workshop-sdk-tree").items(nodes()).spawn(cx);
        let host = cx.new(|_| TreeHost { tree: tree.clone() });
        Box::new(Self { tree, host, cells: RefCell::default() })
    }

    /// The SDK tree view, for tests.
    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn tree(&self) -> TreeView<()> {
        self.tree.clone()
    }
}

impl SurfaceControl for TreeViewSurface {
    fn title(&self) -> &'static str {
        "Tree View"
    }

    fn view(&self) -> AnyView {
        self.host.clone().into()
    }

    /// The tree fills the size the stage gives it, which can be dragged.
    fn stage_width(&self) -> Option<f32> {
        Some(STAGE_SIZE.0)
    }

    fn stage_height(&self) -> Option<f32> {
        Some(STAGE_SIZE.1)
    }

    fn bind(&self, theme: Theme, pinned: Option<&StateSet>, cx: &mut App) {
        let template = template(PinnedTreeTheme::new(theme, pinned.map(|states| (states.clone(), Selected::Keep))));
        self.tree.update(cx, |tree, cx| tree.set_template(template, cx));
        // Cells are remade in the new theme.
        self.cells.borrow_mut().clear();
    }

    fn set_enabled(&self, enabled: bool, cx: &mut App) {
        self.tree.update(cx, |tree, cx| tree.set_enabled(enabled, cx));
    }

    fn template_states(&self) -> Vec<&'static str> {
        STATES.to_vec()
    }

    /// Whether a node is selected.
    fn lasting_states(&self, cx: &App) -> Vec<(&'static str, bool)> {
        vec![("selected", !self.tree.read(cx).selected_ids().is_empty())]
    }

    fn subscribe(&self, cx: &mut Context<ControlSide>, on_event: OnSurfaceEvent) -> Subscription {
        cx.subscribe(&self.tree, move |side: &mut ControlSide, _, event: &TreeViewEvent<()>, cx| {
            // Enabling is the page's own doing (Design disables input); scrolling reports are noise.
            if matches!(event, TreeViewEvent::EnabledChanged { .. } | TreeViewEvent::ScrollChanged { .. }) {
                return;
            }
            let state = match event {
                TreeViewEvent::RowHoverChanged { hovered, .. } => Some(("hover", *hovered)),
                TreeViewEvent::FocusChanged { focused } => Some(("focused", *focused)),
                TreeViewEvent::SelectionChanged { selected_ids } => Some(("selected", !selected_ids.is_empty())),
                _ => None,
            };
            on_event(side, SurfaceEvent { line: format!("{event:?}"), state }, cx);
        })
    }

    /// A folder and two files in `states`, every row selected when `:selected` is.
    fn cell(&self, theme: &Theme, id: SharedString, states: &StateSet, _: &mut Window, cx: &mut App) -> AnyElement {
        let tree = self
            .cells
            .borrow_mut()
            .entry(id.clone())
            .or_insert_with(|| {
                let theme = PinnedTreeTheme::new(theme.clone(), Some((states.clone(), Selected::Every)));
                TreeViewBuilder::new(id).items(cell_nodes()).enabled(false).template(template(theme)).spawn(cx)
            })
            .clone();
        div().w(px(240.)).h(px(96.)).child(tree).into_any_element()
    }

    #[cfg(all(test, feature = "test-support"))]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use gpui::{Focusable, TestAppContext};

    use super::*;
    use crate::workshop::surface_page::WorkMode;
    use crate::workshop::tests_support::open_surface;

    #[test]
    fn the_tree_view_page_expands_selects_and_pins_its_states() {
        let mut app = TestAppContext::single();
        let (page, cx) = open_surface(&mut app, "tree-view");
        let side = cx.update(|_, cx| page.read(cx).side());
        let tree = cx.update(|_, cx| side.read(cx).control_as::<TreeViewSurface>().expect("a tree").tree());

        cx.update(|window, cx| tree.read(cx).focus_handle(cx).focus(window, cx));
        cx.run_until_parked();
        cx.simulate_keystrokes("end right end space");
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert!(tree.read(cx).is_expanded(&"legacy".into()));
            assert!(tree.read(cx).selected_ids().contains("holiday"));
        });
        assert!(cx.update(|_, cx| side.read(cx).state_on("selected")), "the page follows the selection");

        side.update(cx, |side, cx| side.choose_work_mode(WorkMode::Design, cx));
        side.update(cx, |side, cx| side.choose_interaction("pressed", cx));
        cx.run_until_parked();
        let pinned = cx.update(|_, cx| side.read(cx).pinned()).expect("Design draws picked states");
        assert!(pinned.has("pressed") && pinned.has("selected"), "{pinned:?}");

        side.update(cx, |side, cx| side.show_matrix(cx));
        cx.run_until_parked();
        assert!(cx.debug_bounds("workshop-surface-matrix-cell-0-0").is_some(), "matrix cells draw trees");
    }

    /// Design mode turns the tree's input off; its rows fade only when `:disabled` is picked (#397).
    #[test]
    fn design_mode_rows_fade_only_for_a_picked_disabled() {
        let theme = xgpui_luma::style::default_theme;
        let fade = |pinned: Option<StateSet>| {
            PinnedTreeTheme::new(theme(), pinned.map(|states| (states, Selected::Keep)))
                .resolve_scale(1.0)
                .disabled_opacity
        };
        let sdk = fade(None);
        assert!(sdk < 1.0, "the SDK tree fades disabled rows: {sdk}");
        assert_eq!(fade(Some(StateSet::new(["pressed"]))), 1.0, "input off in Design, but not :disabled");
        assert_eq!(fade(Some(StateSet::new(["disabled"]))), sdk, "a picked :disabled fades");
    }
}
