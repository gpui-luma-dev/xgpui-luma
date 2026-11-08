use xgpui_luma::controls::tree_view::TreeViewEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::tree_view::{TreeNode, TreeViewControl, TreeViewEvent, TreeViewSelectionMode};
use xgpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextSize};
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma::{vstack};
use lucide_svg_static::Icon as LucideIcon;

use super::common::titled_card;

const TREE_VIEW_CARD_WIDTH: f32 = 360.0;
const TREE_VIEW_HEIGHT_PX: f32 = 320.0;

/// Number of nesting levels in the mock tree (depth 0 .. TREE_DEPTH - 1).
const TREE_DEPTH: usize = 5;

pub struct TreeViewPanel {
    look: Arc<ShadcnLook>,
    tree: Entity<TreeViewControl<SharedString>>,
    last_event: SharedString,
    _subscriptions: Vec<Subscription>,
}

impl TreeViewPanel {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let tree = shadcn::TreeView::new("studio-tree-view")
            .look(look.as_ref())
            .selection_mode(TreeViewSelectionMode::Single)
            .items(mock_file_tree())
            .spawn(cx);

        let subscriptions = Vec::new();
        tree.clone().on_event(cx, |panel, event, cx| {
            panel.handle_event(event, cx);
        });

        Self { look, tree, last_event: "None".into(), _subscriptions: subscriptions }
    }

    fn handle_event(&mut self, event: &TreeViewEvent<SharedString>, cx: &mut Context<Self>) {
        self.last_event = match event {
            TreeViewEvent::NodeExpanded { node_id, .. } => format!("expanded {node_id}").into(),
            TreeViewEvent::NodeCollapsed { node_id, .. } => format!("collapsed {node_id}").into(),
            TreeViewEvent::SelectionChanged { selected_ids } => {
                let ids: Vec<_> = selected_ids.iter().map(|id| id.as_ref()).collect();
                format!("selection {:?}", ids).into()
            }
            _ => self.last_event.clone(),
        };
        cx.notify();
    }
}

impl Render for TreeViewPanel {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let tree = self.tree.clone();
        let look = self.look.clone();
        let last_event = self.last_event.clone();
        let hint_style = self.look.typography_scale(ShadcnTextSize::Sm);

        titled_card(
            "luma-studio-tree-view-card",
            &self.look,
            TREE_VIEW_CARD_WIDTH,
            "File Explorer",
            "Virtualized tree view for hierarchical data.",
            move |_, cx| {
                vstack! {
                    gap=12;
                    div()
                        .typography_style(hint_style)
                        .text_color(chrome.muted_text)
                        .child(format!(
                            "Mock tree: {TREE_DEPTH} levels deep — click branch rows to expand or collapse"
                        )),
                    shadcn::Frame::new("studio-tree-view-frame")
                        .look(&look)
                        .w_full()
                        .h(px(TREE_VIEW_HEIGHT_PX))
                        .min_h(px(0.0))
                        .overflow_hidden()
                        .rounded(px(8.0))
                        .border_1()
                        .bg(chrome.content_background)
                        .child(tree.clone())
                        .render(cx),
                    div()
                        .typography_style(hint_style)
                        .text_color(chrome.muted_text)
                        .child(format!("Last event: {}", last_event)),
                }
                .w_full()
                .overflow_hidden()
                .into_any_element()
            },
            window,
            _cx,
        )
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
        .children(children)
}
