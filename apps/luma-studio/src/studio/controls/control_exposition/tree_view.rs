//! Tree view control exposition — virtualized file tree with an integrated scrollbar.

use xgpui_luma::controls::tree_view::TreeViewEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, IntoElement, Render, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::button::Button;
use xgpui_luma::controls::ScrollbarVisibility;
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma::controls::tree_view::{TreeNode, TreeView, TreeViewEvent, TreeViewSelectionMode};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::collection_theme_inspectors::TreeViewThemeInspector;
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::template::render_control_exposition_card;
use super::inspector::{
    TreeViewInspectorAdapter, TREE_VIEW_INSPECTOR_SPEC, tree_view_motion_report, tree_view_motion_source,
};

const TREE_DEPTH: usize = 5;

mod state_updates;
mod positioning;
mod selection;
mod scale;
mod workspaces;

pub struct TreeViewControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<TreeViewExpositionLeftPane>,
    theme_inspector: Entity<TreeViewThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

struct TreeViewExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    tree: TreeView<SharedString>,
    event_stream: Entity<ControlEventStream>,
    state_updates: Entity<state_updates::StateUpdates>,
    workspaces: Entity<workspaces::Workspaces>,
    positioning: Entity<positioning::Positioning>,
    selection: Entity<selection::Selection>,
    scale: Entity<scale::Scale>,
    scrollbar_options: Vec<Entity<Button>>,
    scrollbar_visibility: ScrollbarVisibility,
}

impl TreeViewExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.tree.update(cx, |tree, cx| {
            tree.set_template(look.tree_view_template(), cx);
            tree.set_scrollbar_template(look.scrollbar_template(), cx);
        });
        for button in &self.scrollbar_options {
            button.update(cx, |_, cx| cx.notify());
        }
        self.scale.update(cx, |fixture, cx| fixture.sync_look(look.clone(), cx));
        self.selection.update(cx, |fixture, cx| fixture.sync_look(look.clone(), cx));
        self.positioning.update(cx, |fixture, cx| fixture.sync_look(look.clone(), cx));
        self.workspaces.update(cx, |fixture, cx| fixture.sync_look(look.clone(), cx));
        self.state_updates.update(cx, |fixture, cx| fixture.sync_look(look.clone(), cx));
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for TreeViewExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        for (button, visibility) in self.scrollbar_options.iter().zip([
            ScrollbarVisibility::AutoHide,
            ScrollbarVisibility::Hidden,
            ScrollbarVisibility::AlwaysVisible,
        ]) {
            button.update(cx, |button, cx| button.set_enabled(visibility != self.scrollbar_visibility, cx));
        }
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let preview =
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(12.0))
                    .child(div().text_size(px(12.0)).text_color(chrome.muted_text).child(format!(
                        "Mock tree: {TREE_DEPTH} levels deep — click to focus, then scroll. Arrow keys navigate; Home/End jump."
                    )))
                    .child(div().text_size(px(12.0)).text_color(chrome.muted_text).child(format!(
                        "Scrollbar: {}", match self.scrollbar_visibility {
                            ScrollbarVisibility::AutoHide => "While scrolling",
                            ScrollbarVisibility::Hidden => "Never",
                            ScrollbarVisibility::AlwaysVisible => "Always",
                        }
                    )))
                    .child(div().flex().gap(px(8.0)).children(self.scrollbar_options.iter().cloned()))
                    .child(
                        shadcn::Frame::new("tree-preview-frame")
                            .look(&self.look)
                            .w(px(360.0))
                            .h(px(480.0))
                            .flex()
                            .flex_col()
                            .min_h(px(0.0))
                            .overflow_hidden()
                            .rounded(px(8.0))
                            .border_1()
                            .child(self.tree.clone())
                            .render(cx),
                    )
                    .child(self.event_stream.clone())
                    .child(self.state_updates.clone())
                    .child(self.workspaces.clone())
                    .child(self.positioning.clone())
                    .child(self.selection.clone())
                    .child(self.scale.clone());

            div()
                .id("controls-doc-tree-view-left-pane")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .overflow_y_scroll()
                .child(render_control_exposition_card(
                    &self.look,
                    self.entry,
                    preview.into_any_element(),
                    None,
                    ControlExpositionLayout { skip_snippet: true, ..ControlExpositionLayout::BORDERLESS },
                ))
        })
    }
}

impl TreeViewControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("tree-view").expect("tree-view catalog entry");

        let tree = shadcn::TreeView::new("controls-doc-tree-view")
            .look(look.as_ref())
            .selection_mode(TreeViewSelectionMode::Single)
            .require_focus_for_scroll(true)
            .wrap_navigation(false)
            .try_items(mock_file_tree())
            .expect("mock file tree IDs are unique")
            .spawn(cx);

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-tree-view-event-log",
                "Expand branches and select leaves; TreeViewEvent variants appear below.",
            )
        });

        let scale = cx.new(|cx| scale::Scale::new(look.clone(), cx));
        let selection = cx.new(|cx| selection::Selection::new(look.clone(), cx));
        let positioning = cx.new(|cx| positioning::Positioning::new(look.clone(), cx));
        let workspaces = cx.new(|cx| workspaces::Workspaces::new(look.clone(), cx));
        let state_updates = cx.new(|cx| state_updates::StateUpdates::new(look.clone(), cx));
        let left_pane = cx.new(|cx| {
            let scrollbar_options = [
                ("auto", "While scrolling", ScrollbarVisibility::AutoHide),
                ("hidden", "Never", ScrollbarVisibility::Hidden),
                ("always", "Always", ScrollbarVisibility::AlwaysVisible),
            ]
            .into_iter()
            .map(|(id, label, visibility)| {
                shadcn::Button::new(format!("tree-scrollbar-{id}"))
                    .look(&look)
                    .secondary()
                    .label(label)
                    .spawn(cx)
                    .on_click(cx, move |pane: &mut TreeViewExpositionLeftPane, cx| {
                        pane.tree.update(cx, |tree, cx| tree.set_scrollbar_visibility(visibility, cx));
                        pane.scrollbar_visibility = visibility;
                        cx.notify();
                    })
            })
            .collect();
            TreeViewExpositionLeftPane {
                look: look.clone(),
                entry,
                tree: tree.clone(),
                event_stream: event_stream.clone(),
                state_updates,
                workspaces,
                positioning,
                selection,
                scale,
                scrollbar_options,
                scrollbar_visibility: ScrollbarVisibility::AutoHide,
            }
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-tree-view-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &TREE_VIEW_INSPECTOR_SPEC,
            TreeViewInspectorAdapter::shared(),
        );
        theme_inspector.update(cx, |inspector, cx| {
            let (state_updates, workspaces, positioning, selection, scale, file_tree) = {
                let pane = left_pane.read(cx);
                (
                    pane.state_updates.clone(),
                    pane.workspaces.clone(),
                    pane.positioning.clone(),
                    pane.selection.clone(),
                    pane.scale.clone(),
                    pane.tree.clone(),
                )
            };
            let workspace_trees = workspaces.read(cx).trees();
            inspector.set_motion_report(
                tree_view_motion_report([
                    ("File tree", tree_view_motion_source(file_tree)),
                    ("State-preserving updates", tree_view_motion_source(state_updates.read(cx).tree())),
                    ("Workspace left", tree_view_motion_source(workspace_trees[0].clone())),
                    ("Workspace right", tree_view_motion_source(workspace_trees[1].clone())),
                    ("Positioning", tree_view_motion_source(positioning.read(cx).tree())),
                    ("Selection and ranges", tree_view_motion_source(selection.read(cx).tree())),
                    ("Large tree", tree_view_motion_source(scale.read(cx).tree())),
                ]),
                cx,
            );
        });

        tree.clone().on_event(cx, {
            let event_stream = event_stream.downgrade();
            move |_, event: &TreeViewEvent<SharedString>, cx| {
                let Some(event_stream) = event_stream.upgrade() else {
                    return;
                };
                if let Some(line) = format_tree_view_event(event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }
        });

        Self { look, entry, left_pane, theme_inspector, inspector_split }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn fills_viewport(&self) -> bool {
        true
    }

    pub fn request_layout_refresh(&mut self, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.request_layout_refresh(cx));
    }

    pub fn set_viewport_size(&mut self, size: gpui::Size<gpui::Pixels>, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.set_viewport_size(size, cx));
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        sync_viewport_inspector(look, &self.theme_inspector, &self.inspector_split, cx);
        cx.notify();
    }
}

impl Render for TreeViewControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-tree-view-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn format_tree_view_event(event: &TreeViewEvent<SharedString>) -> Option<String> {
    match event {
        TreeViewEvent::NodeExpanded { node_id, .. } => {
            Some(format!("TreeViewEvent::NodeExpanded {{ node_id: \"{node_id}\" }}"))
        }
        TreeViewEvent::NodeCollapsed { node_id, .. } => {
            Some(format!("TreeViewEvent::NodeCollapsed {{ node_id: \"{node_id}\" }}"))
        }
        TreeViewEvent::SelectionChanged { selected_ids } => {
            let mut ids: Vec<_> = selected_ids.iter().map(|id| id.as_ref()).collect();
            ids.sort_unstable();
            Some(format!("TreeViewEvent::SelectionChanged {{ selected_ids: {ids:?} }}"))
        }
        TreeViewEvent::ActiveNodeChanged { node_id } => Some(match node_id {
            Some(id) => format!("TreeViewEvent::ActiveNodeChanged {{ node_id: \"{id}\" }}"),
            None => "TreeViewEvent::ActiveNodeChanged { node_id: None }".to_string(),
        }),
        TreeViewEvent::FocusChanged { focused } => {
            Some(format!("TreeViewEvent::FocusChanged {{ focused: {focused} }}"))
        }
        TreeViewEvent::ScrollChanged { top_index } => {
            Some(format!("TreeViewEvent::ScrollChanged {{ top_index: {top_index} }}"))
        }
        TreeViewEvent::RowHoverChanged { node_id, index, hovered } => Some(format!(
            "TreeViewEvent::RowHoverChanged {{ node_id: {node_id:?}, index: {index}, hovered: {hovered} }}"
        )),
        _ => None,
    }
}

fn mock_file_tree() -> Vec<TreeNode<SharedString>> {
    (0..4)
        .map(|root| {
            let root_id: SharedString = format!("root-{root}").into();
            TreeNode::new(root_id.clone(), format!("Workspace {root}"), root_id)
                .icon(LucideIcon::Folder)
                .expanded(root == 0)
                .children(vec![mock_branch(&format!("root-{root}"), 1)])
        })
        .collect()
}

fn mock_branch(prefix: &str, depth: usize) -> TreeNode<SharedString> {
    if depth >= TREE_DEPTH - 1 {
        let id: SharedString = format!("{prefix}-file").into();
        return TreeNode::new(id.clone(), format!("{prefix}.txt"), id).icon(LucideIcon::File);
    }

    let id: SharedString = format!("{prefix}-folder").into();
    let child_count = if depth == 1 { 4 } else { 3 };
    let children: Vec<_> = (0..child_count).map(|index| mock_branch(&format!("{prefix}-{index}"), depth + 1)).collect();

    TreeNode::new(id.clone(), format!("Folder {prefix}"), id)
        .icon(LucideIcon::Folder)
        .expanded(true)
        .children(children)
}
