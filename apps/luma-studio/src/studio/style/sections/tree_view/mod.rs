use std::sync::Arc;

use gpui::{AnyElement, App, Context, Entity, FontWeight, IntoElement, Render, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::tabs::Tabs;
use xgpui_luma::controls::tree_view::{TreeNode, TreeView, TreeViewSelectionMode};
use xgpui_luma_look_shadcn::ShadcnLook;
use xgpui_luma_look_shadcn as shadcn;
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::style::shared::shell::section_shell_with_width;

pub(crate) struct TreeViewPreview {
    look: Arc<ShadcnLook>,
    sm: TreeView<SharedString>,
    md: TreeView<SharedString>,
    lg: TreeView<SharedString>,
    template: TreeView<SharedString>,
}

impl TreeViewPreview {
    pub(crate) fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        Self {
            sm: sample_tree(&look, "luma-studio-tree-sm", shadcn::ShadcnSize::Sm, cx),
            md: sample_tree(&look, "luma-studio-tree-md", shadcn::ShadcnSize::Md, cx),
            lg: sample_tree(&look, "luma-studio-tree-lg", shadcn::ShadcnSize::Lg, cx),
            template: sample_tree(&look, "luma-studio-tree-template", shadcn::ShadcnSize::Md, cx),
            look,
        }
    }

    pub(crate) fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        let template = look.tree_view_template::<SharedString>();
        for tree in [&self.sm, &self.md, &self.lg, &self.template] {
            let template = template.clone();
            let scrollbar_template = look.scrollbar_template();
            tree.update(cx, move |tree, cx| {
                tree.set_template(template, cx);
                tree.set_scrollbar_template(scrollbar_template, cx);
            });
        }
        cx.notify();
    }
}

impl Render for TreeViewPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

pub(crate) fn render_tree_view_template_section(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<Tabs>,
    preview: Entity<TreeViewPreview>,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let active_tab =
        preview_tabs.read(cx).active_id().cloned().unwrap_or_else(|| SharedString::from("template-preview"));
    let preview = preview.read(cx);

    section_shell_with_width(
        960.0,
        "Tree View",
        "Hierarchical rows. Sizes tab: Sm/Md/Lg row, icon, chevron, and label sizing.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        div()
            .w_full()
            .flex()
            .flex_col()
            .child(div().w_full().flex().justify_start().child(preview_tabs))
            .child(div().w_full().h(px(1.0)).bg(chrome.border))
            .child(div().w_full().flex().justify_center().mt(px(16.0)).child(match active_tab.as_ref() {
                "sizes" => render_sizes_body(preview, chrome.muted_text, cx),
                _ => render_template_body(preview, chrome.muted_text, cx),
            }))
            .into_any_element(),
    )
}

fn render_template_body(preview: &TreeViewPreview, muted: gpui::Hsla, cx: &App) -> AnyElement {
    size_column("Template", muted, &preview.look, cx, preview.template.clone(), 260.0)
}

fn render_sizes_body(preview: &TreeViewPreview, muted: gpui::Hsla, cx: &App) -> AnyElement {
    div()
        .flex()
        .flex_wrap()
        .items_start()
        .gap(px(16.0))
        .child(size_column("Sm", muted, &preview.look, cx, preview.sm.clone(), 220.0))
        .child(size_column("Md", muted, &preview.look, cx, preview.md.clone(), 220.0))
        .child(size_column("Lg", muted, &preview.look, cx, preview.lg.clone(), 220.0))
        .into_any_element()
}

fn size_column(
    label: &'static str,
    muted: gpui::Hsla,
    look: &ShadcnLook,
    cx: &App,
    tree: TreeView<SharedString>,
    width: f32,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .w(px(width))
        .child(
            div()
                .text_size(px(11.0))
                .line_height(px(14.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(muted)
                .child(label),
        )
        .child(
            shadcn::Frame::new(format!("tree-size-{label}-frame"))
                .look(look)
                .bg(gpui::transparent_black())
                .w_full()
                .h(px(200.0))
                .rounded(px(6.0))
                .border_1()
                .overflow_hidden()
                .child(tree)
                .render(cx),
        )
        .into_any_element()
}

fn sample_tree<M: 'static>(
    look: &Arc<ShadcnLook>,
    id: impl Into<SharedString>,
    size: shadcn::ShadcnSize,
    cx: &mut Context<M>,
) -> TreeView<SharedString> {
    let id = id.into();
    shadcn::TreeView::new(id.clone())
        .look(look.as_ref())
        .selection_mode(TreeViewSelectionMode::Single)
        .size(size)
        .items(sample_nodes(id.as_ref()))
        .spawn(cx)
}

fn sample_nodes(prefix: &str) -> Vec<TreeNode<SharedString>> {
    vec![
        TreeNode::new(format!("{prefix}-docs"), "Documents", SharedString::from("docs"))
            .icon(LucideIcon::Folder)
            .expanded(true)
            .children([
                TreeNode::new(format!("{prefix}-readme"), "README.md", SharedString::from("readme"))
                    .icon(LucideIcon::File),
                TreeNode::new(format!("{prefix}-guide"), "Guide.md", SharedString::from("guide"))
                    .icon(LucideIcon::File),
            ]),
        TreeNode::new(format!("{prefix}-src"), "src", SharedString::from("src"))
            .icon(LucideIcon::Folder)
            .children([
                TreeNode::new(format!("{prefix}-main"), "main.rs", SharedString::from("main")).icon(LucideIcon::File)
            ]),
    ]
}
