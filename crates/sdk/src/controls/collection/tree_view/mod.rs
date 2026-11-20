//! An undecorated hierarchical viewport. Compose it inside a
//! [`FrameBuilder`](crate::controls::frame::FrameBuilder) for optional outer fill,
//! border, radius, and inset. Rows, focus, selection, and scrolling stay in TreeView.

mod styled;
pub use styled::{StyledTreeView, StyledTreeViewTheme};
mod control;
mod drag;
pub use drag::{
    TreeDropLocation, TreeDropPosition, TreeDropProposal, TreeViewDragDrop, TreeViewDragEvent, TreeDragEndReason,
};
mod model;
mod projection;
mod state;
mod template;
pub(crate) mod theme;

pub use control::{
    TreeViewSubtreeState, CollapseNode, ExpandNode, SelectAllNodes, ClearSelectedNodes, TreeViewControl, TreeViewEvent,
};
pub use model::{
    FlatTreeNode, TreeNode, TreeViewBuilder, TreeViewModel, TreeViewRenderModel, TreeViewSelectionMode,
    TreeViewSelectionPolicy,
};
pub use state::TreeViewError;
pub use template::{
    TreeViewTemplate, TreeViewTemplateHandlers, TreeViewTemplateModifier, ThemedTreeViewTemplate,
    default_tree_view_template,
};
pub use theme::{DefaultTreeViewTheme, TreeViewPalette, TreeViewScale, TreeViewTheme, default_tree_view_theme};

pub use crate::infra::state::{CompositeItemState as TreeViewItemState, ControlFocusState};
use gpui::{Entity, SharedString};

/// Entity handle for a tree view control.
pub type TreeView<T> = Entity<TreeViewControl<T>>;

/// Creates a new builder for a [`TreeView`].
pub fn new<T>(id: impl Into<SharedString>) -> TreeViewBuilder<T>
where
    T: Clone + Send + Sync + 'static,
{
    TreeViewBuilder::new(id)
}

mod callback;
pub use callback::TreeViewEventExt;

pub use model::TreeViewMotion;
