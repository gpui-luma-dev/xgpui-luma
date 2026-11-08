//! Shared live layer-tree sample for Custom Palette and the Style Guide.

use gpui::{Context, IntoElement, prelude::*, px};
use xgpui_luma::controls::tree_view::{TreeNode, TreeView};
use xgpui_luma_look_radix::Look;
use lucide_svg_static::Icon;

pub fn spawn<T: 'static>(id: &'static str, look: &Look, enabled: bool, cx: &mut Context<T>) -> TreeView<()> {
    let tree = xgpui_luma_look_radix::TreeView::new(id)
        .look(look)
        .enabled(enabled)
        .items([
            TreeNode::new("box", "Box", ()).icon(Icon::Square),
            TreeNode::new("grid", "Grid", ()).icon(Icon::Grid2x2).expanded(true).children([
                TreeNode::new("image-1", "Image", ()).icon(Icon::Image),
                TreeNode::new("image-2", "Image", ()).icon(Icon::Image),
                TreeNode::new("text", "Text", ()).icon(Icon::TypeIcon),
            ]),
        ])
        .spawn(cx);
    tree.update(cx, |tree, cx| tree.select_node_by_id("image-1", cx));
    tree
}

pub fn panel(tree: TreeView<()>, look: &Look) -> gpui::AnyElement {
    xgpui_luma_look_radix::tree_view_frame(look).w_full().h(px(142.0)).child(tree).into_any_element()
}
