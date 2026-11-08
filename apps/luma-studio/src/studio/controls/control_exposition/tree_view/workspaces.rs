//! In-memory domain mutation stays here; TreeView supplies interaction and proposals.
use xgpui_luma::controls::tree_view::TreeViewEventExt;
use xgpui_luma::controls::textfield::TextFieldEventExt;
use xgpui_luma::controls::selector::SelectorEventExt;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::Arc,
};
use gpui::{App, Context, Entity, Render, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::button::Button;
use xgpui_luma::controls::textfield::{TextField, TextFieldEvent};
use xgpui_luma::controls::selector::{Selector, SelectorEvent, SelectorItem};
use xgpui_luma::controls::tree_view::{
    FlatTreeNode, TreeNode, TreeView, TreeViewBuilder, TreeViewDragDrop, TreeDropProposal, TreeDropLocation,
    TreeDropPosition, TreeViewSelectionMode, TreeViewEvent, TreeViewDragEvent, TreeDragEndReason,
};
use xgpui_luma::infra::{drag_drop::DragDropElementExt, presenter::HasPresenter};
use xgpui_luma_look_shadcn::{self as shadcn, ShadcnLook, prelude::*};
use lucide_svg_static::Icon;
use super::ControlEventStream;

#[derive(Clone, Debug)]
pub(super) struct Entry {
    detail: SharedString,
}

// Local construction syntax expands only while constructing the fixture.
macro_rules! workspace_tree {
    ($cx:expr, $look:expr, $id:expr, $items:expr, $drop:expr, $leaf:expr) => {
        shadcn::TreeView::new($id)
            .look($look)
            .items($items)
            .selection_mode(TreeViewSelectionMode::Extended)
            .expand_on_row_click(false)
            .wrap_navigation(false)
            .require_focus_for_scroll(true)
            .branch_content(folder_content)
            .leaf_content($leaf)
            .drag_drop($drop)
            .spawn($cx)
    };
}

fn folder_content(node: &FlatTreeNode<'_, Entry>, _: &mut Window, _: &mut App) -> gpui::AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .child(node.label.clone())
        .child(div().text_size(px(10.0)).opacity(0.65).child(node.data.detail.clone()))
        .into_any_element()
}

pub(super) struct Workspaces {
    look: Arc<ShadcnLook>,
    trees: [TreeView<Entry>; 2],
    actions: Vec<Entity<Button>>,
    search: TextField,
    sort: Entity<Selector>,
    event_stream: Entity<ControlEventStream>,
}

impl Workspaces {
    pub(super) fn trees(&self) -> [TreeView<Entry>; 2] {
        self.trees.clone()
    }

    pub(super) fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let actions: HashMap<SharedString, Entity<Button>> = ["readme", "guide", "draft"]
            .into_iter()
            .map(|id| {
                let button = shadcn::Button::new(format!("workspace-action-{id}"))
                    .look(&look)
                    .secondary()
                    .size(shadcn::ShadcnSize::Sm)
                    .label("Info")
                    .with_template_modifier(move |root, _| {
                        root.debug_selector(move || format!("workspace-action-{id}"))
                    })
                    .spawn(cx)
                    .on_click(cx, move |this: &mut Self, cx| {
                        this.log(format!("Info: {id}"), cx);
                    });
                (id.into(), button)
            })
            .collect();
        let action_entities = actions.values().cloned().collect();
        let actions = Rc::new(actions);
        let owner = cx.entity().downgrade();
        let drop = TreeViewDragDrop::new(cx.entity_id(), move |proposal, _, app| {
            let _ = owner.update(app, |this, cx| this.apply(proposal, cx));
        })
        .drag_selected(true);
        let data = sample_items();
        let trees = std::array::from_fn(|i| {
            let actions = actions.clone();
            workspace_tree!(
                cx,
                &look,
                if i == 0 { "workspace-left" } else { "workspace-right" },
                data[i].clone(),
                drop.clone(),
                move |node: &FlatTreeNode<'_, Entry>, _: &mut Window, _: &mut App| {
                    let content = div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(div().flex_1().truncate().child(node.label.clone()))
                        .child(div().text_size(px(10.0)).opacity(0.65).child(node.data.detail.clone()));
                    content.when_some(actions.get(node.id).cloned(), |content, action| {
                        content.child(div().id(format!("action-boundary-{}", node.id)).drag_boundary().child(action))
                    })
                }
            )
        });
        for tree in &trees {
            tree.clone().on_event(cx, |this, event, cx| {
                if let TreeViewEvent::DragDrop(event) = event {
                    this.log(this.format_drag_event(event), cx);
                }
                cx.notify();
            });
        }
        let search = shadcn::TextField::new("workspace-filter")
            .look(&look)
            .placeholder("Filter names or details…")
            .clean_on_escape(true)
            .full_width(true)
            .spawn(cx);
        search.clone().on_event(cx, |this, event, cx| {
            if let TextFieldEvent::Change { value } = event {
                let query = value.trim().to_lowercase();
                for tree in &this.trees {
                    let query = query.clone();
                    tree.update(cx, |tree, cx| {
                        if query.is_empty() {
                            tree.clear_filter(cx);
                        } else {
                            tree.set_filter(
                                move |node| {
                                    node.label.to_lowercase().contains(&query)
                                        || node.data.detail.to_lowercase().contains(&query)
                                },
                                cx,
                            );
                        }
                    });
                }
                cx.notify();
            }
        });
        let sort = shadcn::Selector::new("workspace-sort")
            .look(&look)
            .label("Sibling order")
            .items([
                SelectorItem::new("source").label("Source order"),
                SelectorItem::new("ascending").label("Name A–Z"),
                SelectorItem::new("descending").label("Name Z–A"),
            ])
            .selected_id("source")
            .spawn(cx);
        sort.clone().on_event(cx, |this, event, cx| {
            if let SelectorEvent::Change { item_id, .. } = event {
                for tree in &this.trees {
                    tree.update(cx, |tree, cx| match item_id.as_ref() {
                        "ascending" => tree.set_sort(|a, b| a.label.cmp(&b.label), cx),
                        "descending" => tree.set_sort(|a, b| b.label.cmp(&a.label), cx),
                        _ => tree.clear_sort(cx),
                    });
                }
                cx.notify();
            }
        });
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-tree-view-workspaces-event-log",
                "Drag row edges before/after; folder centers move into folders. Blank space appends to root. Escape cancels.",
            )
        });
        Self { look, trees, actions: action_entities, search, sort, event_stream }
    }

    fn log(&mut self, line: String, cx: &mut Context<Self>) {
        self.event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
    }

    fn format_drag_event(&self, event: &TreeViewDragEvent) -> String {
        let workspace = |id| {
            if id == self.trees[0].entity_id() {
                "Working"
            } else {
                "Archive"
            }
        };
        match event {
            TreeViewDragEvent::Started { source, node_ids, .. } => {
                format!("Started: {} in {}", ids(node_ids), workspace(*source))
            }
            TreeViewDragEvent::ItemsRemoved { source, target, node_ids, .. } => {
                format!("Removed: {} · {} → {}", ids(node_ids), workspace(*source), workspace(*target))
            }
            TreeViewDragEvent::ItemsAdded { target, node_ids, .. } => {
                format!("Added: {} in {}", ids(node_ids), workspace(*target))
            }
            TreeViewDragEvent::ItemsReordered { target, node_ids, .. } => {
                format!("Reordered: {} in {}", ids(node_ids), workspace(*target))
            }
            TreeViewDragEvent::Dropped { source, target, node_ids, location, changed, .. } => {
                let position = match location.position {
                    TreeDropPosition::Before => "before",
                    TreeDropPosition::After => "after",
                    TreeDropPosition::Into => "into",
                };
                let anchor = location.anchor_id.as_deref().unwrap_or("root");
                let outcome = if *changed { "moved" } else { "unchanged" };
                format!(
                    "Dropped: {} · {} → {} · {position} {anchor} · {outcome}",
                    ids(node_ids),
                    workspace(*source),
                    workspace(*target)
                )
            }
            TreeViewDragEvent::Rejected { source, target, node_ids, error, .. } => {
                format!("Rejected: {} · {} → {} · {error}", ids(node_ids), workspace(*source), workspace(*target))
            }
            TreeViewDragEvent::Ended { node_ids, reason, .. } => {
                let reason = match reason {
                    TreeDragEndReason::MovedWithinTree => "moved within workspace",
                    TreeDragEndReason::MovedAcrossTrees => "moved across workspaces",
                    TreeDragEndReason::Unchanged => "unchanged",
                    TreeDragEndReason::Rejected => "rejected",
                    TreeDragEndReason::Cancelled => "cancelled",
                };
                format!("Ended: {} · {reason}", ids(node_ids))
            }
        }
    }

    fn apply(&mut self, proposal: TreeDropProposal<Entry>, cx: &mut Context<Self>) {
        if let Err(error) = proposal.validate(cx) {
            proposal.rejected(error, cx);
            return;
        }
        let (Some(source), Some(target)) = (proposal.source(), proposal.target()) else {
            return;
        };
        let same = source == target;
        let transfer = source.read(cx).capture_subtrees_state(proposal.node_ids());
        let prepared = prepare_move(
            source.read(cx).items(),
            target.read(cx).items(),
            proposal.node_ids(),
            proposal.location(),
            same,
        );
        let (src, dst, changed) = match prepared {
            Ok(result) => result,
            Err(error) => {
                proposal.rejected(error, cx);
                return;
            }
        };
        if changed {
            // Both snapshots were validated before either update. Events are
            // delivered after this synchronous host transaction completes.
            if same {
                source.update(cx, |tree, cx| tree.replace_items(dst, cx).expect("validated move snapshot"));
            } else {
                source.update(cx, |tree, cx| tree.replace_items(src, cx).expect("validated source snapshot"));
                target.update(cx, |tree, cx| {
                    tree.replace_items(dst, cx).expect("validated destination snapshot");
                    tree.restore_subtree_state(&transfer, cx);
                });
            }
        }
        proposal.committed(changed, cx);
    }

    pub(super) fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for tree in &self.trees {
            tree.update(cx, |tree, cx| {
                tree.set_template(look.tree_view_template(), cx);
                tree.set_scrollbar_template(look.scrollbar_template(), cx);
            });
        }
        for action in &self.actions {
            action.update(cx, |_, cx| cx.notify());
        }
        self.search.update(cx, |_, cx| cx.notify());
        self.sort.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for Workspaces {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(10.0))
                .child(div().text_color(chrome.title_text).child("Move between workspaces"))
                .child(
                    div().text_size(px(12.0)).text_color(chrome.muted_text).child(
                        "Ctrl/Cmd-click toggles; Shift-click selects a range. Drag a selected row to move the visible selection together. Chevrons expand; Info buttons stay independent.",
                    ),
                )
                .child(div().flex().flex_wrap().gap(px(12.0))
                    .child(div().w(px(340.0)).child(self.search.clone()))
                    .child(self.sort.clone()))
                .child(div().text_size(px(12.0)).text_color(chrome.muted_text).child(
                    "Selected folders carry their entire subtree once. Hidden selections outside those folders stay put. Filtered or sorted destinations accept drops into folders or the root only.",
                ))
                .child(div().flex().flex_wrap().gap(px(12.0)).children(self.trees.iter().enumerate().map(
                    |(i, tree)| {
                        let state = tree.read(cx);
                        let visible = state.visible_ids();
                        let hidden = state.selected_ids().iter().filter(|id| !visible.contains(id)).count();
                        let summary = if visible.is_empty() { "No matching rows".to_owned() } else { format!("{} visible rows", visible.len()) };
                        div()
                            .w(px(340.0))
                            .flex()
                            .flex_col()
                            .gap(px(6.0))
                            .child(div().text_color(chrome.title_text).child(if i == 0 {
                                "Working"
                            } else {
                                "Archive"
                            }))
                            .child(
                                shadcn::Frame::new(format!("workspace-tree-frame-{i}"))
                                    .look(&self.look)
                                    .h(px(300.0))
                                    .border_1()
                                    .rounded_md()
                                    .overflow_hidden()
                                    .child(tree.clone())
                                    .render(cx),
                            )
                            .child(div().text_size(px(12.0)).text_color(chrome.muted_text)
                                .child(format!("{summary} · {} selected ({hidden} hidden)", state.selected_ids().len())))
                    },
                )))
                .child(self.event_stream.clone())
        })
    }
}

fn node(id: &str, label: &str, detail: &str) -> TreeNode<Entry> {
    TreeNode::new(id.to_owned(), label.to_owned(), Entry { detail: detail.to_owned().into() })
}
fn sample_items() -> [Vec<TreeNode<Entry>>; 2] {
    [
        vec![
            node("project", "Project", "workspace").icon(Icon::Folder).expanded(true).children([
                node("docs", "Documents", "folder")
                    .expanded(true)
                    .children([node("readme", "Readme", "2 KB"), node("guide", "Guide", "8 KB")]),
                node("empty", "Empty folder", "drop here").branch(true),
                node("locked", "Locked", "read only").branch(true).enabled(false),
            ]),
            node("draft", "Draft", "1 KB"),
        ],
        vec![node("archive", "Archive", "drop here").branch(true).expanded(true)],
    ]
}

type PreparedMove = (Vec<TreeNode<Entry>>, Vec<TreeNode<Entry>>, bool);
fn prepare_move(
    source: &[TreeNode<Entry>],
    target: &[TreeNode<Entry>],
    ids: &[SharedString],
    location: &TreeDropLocation,
    same: bool,
) -> Result<PreparedMove, SharedString> {
    if ids.is_empty() {
        return Err("No moving roots".into());
    }
    let roots: HashSet<_> = ids.iter().collect();
    if roots.len() != ids.len() {
        return Err("Duplicate moving roots".into());
    }
    let mut src = source.to_vec();
    // Resolve the gap before removing roots, so before/after a moving sibling
    // still has a deterministic location among the surviving siblings.
    let mut original = if same { source.to_vec() } else { target.to_vec() };
    let siblings = destination_siblings(&mut original, location)?;
    let gap = match location.position {
        TreeDropPosition::Into => {
            if location.anchor_id != location.parent_id {
                return Err("Invalid Into target".into());
            }
            siblings.len()
        }
        TreeDropPosition::Before | TreeDropPosition::After => {
            let i = siblings
                .iter()
                .position(|node| Some(&node.id) == location.anchor_id.as_ref())
                .ok_or_else(|| SharedString::from("Sibling anchor disappeared"))?;
            if !siblings[i].enabled {
                return Err("Anchor is disabled".into());
            }
            i + usize::from(location.position == TreeDropPosition::After)
        }
    };
    let index = siblings[..gap].iter().filter(|node| !same || !roots.contains(&node.id)).count();
    let mut extracted = HashMap::new();
    extract_roots(&mut src, &roots, &mut extracted);
    if extracted.len() != ids.len() {
        return Err("Moving root missing or overlapping".into());
    }
    let mut moving = Vec::with_capacity(ids.len());
    for id in ids {
        let node = extracted.remove(id).ok_or_else(|| SharedString::from("Moving root disappeared"))?;
        if !node.enabled {
            return Err("Moving root is disabled".into());
        }
        moving.push(node);
    }
    let mut dst = if same { src.clone() } else { target.to_vec() };
    let siblings = destination_siblings(&mut dst, location)?;
    siblings.splice(index..index, moving);
    TreeViewBuilder::new("validate-source")
        .try_items(src.clone())
        .map_err(|e| SharedString::from(e.to_string()))?;
    TreeViewBuilder::new("validate-target")
        .try_items(dst.clone())
        .map_err(|e| SharedString::from(e.to_string()))?;
    let changed = !same || shape(source) != shape(&dst);
    Ok((src, dst, changed))
}
fn destination_siblings<'a>(
    nodes: &'a mut Vec<TreeNode<Entry>>,
    location: &TreeDropLocation,
) -> Result<&'a mut Vec<TreeNode<Entry>>, SharedString> {
    if let Some(parent) = &location.parent_id {
        let parent = find_mut(nodes, parent)
            .ok_or_else(|| SharedString::from("Destination parent disappeared or creates a cycle"))?;
        if !parent.enabled || (!parent.is_branch && parent.children.is_empty()) {
            return Err("Destination is not an enabled folder".into());
        }
        Ok(&mut parent.children)
    } else {
        Ok(nodes)
    }
}
fn ids(ids: &[SharedString]) -> String {
    ids.iter().map(|id| id.as_ref()).collect::<Vec<_>>().join(", ")
}

fn shape(nodes: &[TreeNode<Entry>]) -> Vec<(SharedString, Option<SharedString>)> {
    fn walk(
        nodes: &[TreeNode<Entry>],
        parent: Option<&SharedString>,
        out: &mut Vec<(SharedString, Option<SharedString>)>,
    ) {
        for node in nodes {
            out.push((node.id.clone(), parent.cloned()));
            walk(&node.children, Some(&node.id), out);
        }
    }
    let mut result = Vec::new();
    walk(nodes, None, &mut result);
    result
}
fn find_mut<'a>(nodes: &'a mut [TreeNode<Entry>], id: &SharedString) -> Option<&'a mut TreeNode<Entry>> {
    for node in nodes {
        if &node.id == id {
            return Some(node);
        }
        if let Some(found) = find_mut(&mut node.children, id) {
            return Some(found);
        }
    }
    None
}
fn extract_roots(
    nodes: &mut Vec<TreeNode<Entry>>,
    roots: &HashSet<&SharedString>,
    out: &mut HashMap<SharedString, TreeNode<Entry>>,
) {
    for mut node in std::mem::take(nodes) {
        if roots.contains(&node.id) {
            out.insert(node.id.clone(), node);
        } else {
            extract_roots(&mut node.children, roots, out);
            nodes.push(node);
        }
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests;
