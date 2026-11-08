//! Look-owned tree-view builder. Spawn synthesizes the SDK [`xgpui_luma::controls::tree_view::TreeView`].

use std::sync::Arc;

use gpui::{Context, Entity, SharedString};
use xgpui_luma::controls::tree_view::{TreeNode, TreeViewBuilder, TreeViewControl, TreeViewSelectionMode, TreeViewTemplate};
use xgpui_luma::infra::icon::DisclosureIcons;
use xgpui_luma::controls::tree_view::TreeViewError;
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

/// Builder in the guise of a tree view: Shadcn template plus SDK options, until `.spawn(cx)`.
pub struct TreeView<T>
where
    T: Clone + Send + Sync + 'static,
{
    callbacks: xgpui_luma::infra::callbacks::EventCallbacks<xgpui_luma::controls::tree_view::TreeViewEvent<T>>,

    look: Option<ShadcnLook>,
    builder: TreeViewBuilder<T>,
    custom_template: bool,
    size: ShadcnSize,
}

impl<T> TreeView<T>
where
    T: Clone + Send + Sync + 'static,
{
    xgpui_luma::forward_motion!(xgpui_luma::controls::tree_view::TreeViewMotion, builder builder);

    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::tree_view::TreeViewEvent<T>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Observe the committed `ItemsRemoved` phase before successful completion.
    pub fn on_items_removed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &gpui::SharedString,
            &Vec<gpui::SharedString>,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewDragEvent::ItemsRemoved {
                source,
                target,
                node_id,
                node_ids,
            } = event
            {
                callback(owner, *source, *target, node_id, node_ids, cx);
            }
        })
    }

    /// Observe the committed `ItemsAdded` phase before successful completion.
    pub fn on_items_added<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &gpui::SharedString,
            &Vec<gpui::SharedString>,
            &xgpui_luma::controls::tree_view::TreeDropLocation,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewDragEvent::ItemsAdded {
                source,
                target,
                node_id,
                node_ids,
                location,
            } = event
            {
                callback(owner, *source, *target, node_id, node_ids, location, cx);
            }
        })
    }

    /// Observe the committed `ItemsReordered` phase before successful completion.
    pub fn on_items_reordered<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            &gpui::SharedString,
            &Vec<gpui::SharedString>,
            &xgpui_luma::controls::tree_view::TreeDropLocation,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewDragEvent::ItemsReordered {
                target,
                node_id,
                node_ids,
                location,
            } = event
            {
                callback(owner, *target, node_id, node_ids, location, cx);
            }
        })
    }

    /// Observe `Started` through the existing drag/drop lifecycle.
    pub fn on_drag_started<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, gpui::EntityId, &gpui::SharedString, &Vec<gpui::SharedString>, &mut gpui::Context<M>)
        + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewDragEvent::Started { source, node_id, node_ids } = event {
                callback(owner, *source, node_id, node_ids, cx);
            }
        })
    }

    /// Observe `Dropped` through the existing drag/drop lifecycle.
    pub fn on_dropped<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &gpui::SharedString,
            &Vec<gpui::SharedString>,
            &xgpui_luma::controls::tree_view::TreeDropLocation,
            bool,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewDragEvent::Dropped {
                source,
                target,
                node_id,
                node_ids,
                location,
                changed,
            } = event
            {
                callback(owner, *source, *target, node_id, node_ids, location, *changed, cx);
            }
        })
    }

    /// Observe `Rejected` through the existing drag/drop lifecycle.
    pub fn on_drop_rejected<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            gpui::EntityId,
            &gpui::SharedString,
            &Vec<gpui::SharedString>,
            &gpui::SharedString,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewDragEvent::Rejected {
                source,
                target,
                node_id,
                node_ids,
                error,
            } = event
            {
                callback(owner, *source, *target, node_id, node_ids, error, cx);
            }
        })
    }

    /// Observe `Ended` through the existing drag/drop lifecycle.
    pub fn on_drag_ended<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(
            &mut M,
            gpui::EntityId,
            &gpui::SharedString,
            &Vec<gpui::SharedString>,
            xgpui_luma::controls::tree_view::TreeDragEndReason,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewDragEvent::Ended { source, node_id, node_ids, reason } =
                event
            {
                callback(owner, *source, node_id, node_ids, *reason, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::DragDrop` through the existing event stream.
    pub fn on_drag_drop<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::tree_view::TreeViewDragEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewEvent::DragDrop(event) = event {
                callback(owner, event, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::NodeActivated` through the existing event stream.
    pub fn on_node_activated<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &T, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewEvent::NodeActivated { node_id, data } = event {
                callback(owner, node_id, data, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::SelectionPolicyChanged` through the existing event stream.
    pub fn on_selection_policy_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &xgpui_luma::controls::tree_view::TreeViewSelectionPolicy, &mut gpui::Context<M>)
        + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewEvent::SelectionPolicyChanged { policy } = event {
                callback(owner, policy, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::NodeExpanded` through the existing event stream.
    pub fn on_node_expanded<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &T, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewEvent::NodeExpanded { node_id, data } = event {
                callback(owner, node_id, data, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::NodeCollapsed` through the existing event stream.
    pub fn on_node_collapsed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, &T, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewEvent::NodeCollapsed { node_id, data } = event {
                callback(owner, node_id, data, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::SelectionChanged` through the existing event stream.
    pub fn on_selection_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &std::collections::HashSet<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewEvent::SelectionChanged { selected_ids } = event {
                callback(owner, selected_ids, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::ActiveNodeChanged` through the existing event stream.
    pub fn on_active_node_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &Option<gpui::SharedString>, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewEvent::ActiveNodeChanged { node_id } = event {
                callback(owner, node_id, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::ScrollChanged` through the existing event stream.
    pub fn on_scroll_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, usize, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewEvent::ScrollChanged { top_index } = event {
                callback(owner, *top_index, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::RowHoverChanged` through the existing event stream.
    pub fn on_row_hover_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &gpui::SharedString, usize, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewEvent::RowHoverChanged { node_id, index, hovered } = event {
                callback(owner, node_id, *index, *hovered, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::EnabledChanged` through the existing event stream.
    pub fn on_enabled_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let xgpui_luma::controls::tree_view::TreeViewEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            look: None,
            builder: xgpui_luma::controls::tree_view::new(id),
            custom_template: false,
            size: ShadcnSize::Md,
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    /// Set initial items, retaining previous items if IDs are duplicated.
    /// Use [`Self::try_items`] when validation errors need to be reported.
    pub fn items(mut self, items: impl IntoIterator<Item = TreeNode<T>>) -> Self {
        self.builder = self.builder.items(items);
        self
    }

    /// Validate IDs across all loaded descendants before constructing the tree.
    pub fn try_items(mut self, items: impl IntoIterator<Item = TreeNode<T>>) -> Result<Self, TreeViewError> {
        self.builder = self.builder.try_items(items)?;
        Ok(self)
    }

    /// Show loaded matches and ancestor paths, retaining user expansion separately.
    pub fn filter(mut self, predicate: impl Fn(&TreeNode<T>) -> bool + 'static) -> Self {
        self.builder = self.builder.filter(predicate);
        self
    }

    /// Stable display-only ordering within each sibling group.
    pub fn sort(mut self, compare: impl Fn(&TreeNode<T>, &TreeNode<T>) -> std::cmp::Ordering + 'static) -> Self {
        self.builder = self.builder.sort(compare);
        self
    }

    pub fn selection_mode(mut self, mode: TreeViewSelectionMode) -> Self {
        self.builder = self.builder.selection_mode(mode);
        self
    }

    /// Configure selection mode, repeated-click deselection and keyboard selection.
    pub fn selection_policy(mut self, policy: xgpui_luma::controls::tree_view::TreeViewSelectionPolicy) -> Self {
        self.builder = self.builder.selection_policy(policy);
        self
    }

    /// Override the SDK interaction policy.
    /// Override focus behavior independently of wheel routing.
    pub fn pointer_focus_policy(mut self, policy: xgpui_luma::interaction::PointerFocusPolicy) -> Self {
        self.builder = self.builder.pointer_focus_policy(policy);
        self
    }

    pub fn wheel_scroll_policy(mut self, policy: xgpui_luma::interaction::WheelScrollPolicy) -> Self {
        self.builder = self.builder.wheel_scroll_policy(policy);
        self
    }

    /// Override the SDK interaction policy.
    pub fn scroll_boundary_policy(mut self, policy: xgpui_luma::interaction::ScrollBoundaryPolicy) -> Self {
        self.builder = self.builder.scroll_boundary_policy(policy);
        self
    }

    /// Override the SDK interaction policy.
    pub fn wheel_focus_scope(mut self, policy: xgpui_luma::interaction::WheelFocusScope) -> Self {
        self.builder = self.builder.wheel_focus_scope(policy);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }

    pub fn size(mut self, size: ShadcnSize) -> Self {
        self.size = size;
        self
    }

    /// Animate small branches; transitions affecting over 256 displayed
    /// descendants settle before layout to preserve virtualization.
    pub fn motion_enabled(mut self, motion_enabled: bool) -> Self {
        self.builder = self.builder.motion_enabled(motion_enabled);
        self
    }

    /// Compatibility alias for wheel_scroll_policy; leaves boundary policy unchanged.
    pub fn require_focus_for_scroll(mut self, required: bool) -> Self {
        self.builder = self.builder.require_focus_for_scroll(required);
        self
    }

    /// Opt out of the legacy Up/Down wrapping behavior.
    pub fn wrap_navigation(mut self, wrap: bool) -> Self {
        self.builder = self.builder.wrap_navigation(wrap);
        self
    }

    /// Set scrollbar chrome visibility. AutoHide (the default) shows while scrolling.
    pub fn scrollbar_visibility(mut self, visibility: xgpui_luma::controls::ScrollbarVisibility) -> Self {
        self.builder = self.builder.scrollbar_visibility(visibility);
        self
    }

    /// Enable scoped dragging and host-owned drop transactions.
    /// The configuration can opt into dragging selected groups.
    pub fn drag_drop(mut self, config: xgpui_luma::controls::tree_view::TreeViewDragDrop<T>) -> Self {
        self.builder = self.builder.drag_drop(config);
        self
    }

    /// Named or inline content inside the Shadcn branch row shell.
    pub fn branch_content<F, E>(mut self, content: F) -> Self
    where
        F: Fn(&xgpui_luma::controls::tree_view::FlatTreeNode<'_, T>, &mut gpui::Window, &mut gpui::App) -> E + 'static,
        E: gpui::IntoElement + 'static,
    {
        self.builder = self.builder.branch_content(content);
        self
    }

    /// Named or inline content inside the Shadcn leaf row shell.
    pub fn leaf_content<F, E>(mut self, content: F) -> Self
    where
        F: Fn(&xgpui_luma::controls::tree_view::FlatTreeNode<'_, T>, &mut gpui::Window, &mut gpui::App) -> E + 'static,
        E: gpui::IntoElement + 'static,
    {
        self.builder = self.builder.leaf_content(content);
        self
    }

    /// False reserves pointer expansion for the disclosure control.
    pub fn expand_on_row_click(mut self, expand: bool) -> Self {
        self.builder = self.builder.expand_on_row_click(expand);
        self
    }

    pub fn disclosure_icons(mut self, icons: DisclosureIcons) -> Self {
        self.builder = self.builder.disclosure_icons(icons);
        self
    }

    pub fn template(mut self, template: Arc<dyn TreeViewTemplate<T>>) -> Self {
        self.custom_template = true;
        self.builder = self.builder.template(template);
        self
    }

    pub fn spawn<M: 'static>(mut self, cx: &mut Context<M>) -> Entity<TreeViewControl<T>> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = {
            let look = resolve_look_from(self.look.as_ref(), cx);
            self.into_sdk_builder(look).spawn(cx)
        };
        callbacks.bind(&control, cx);
        control
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> TreeViewBuilder<T> {
        let self_builder =
            self.builder.stylesheet(&look.stylesheet().common).motion_defaults(look.motion_defaults().tree_view);
        let builder = self_builder.size(self.size.control_size()).scrollbar_template(look.scrollbar_template());
        if self.custom_template {
            builder
        } else {
            builder.template(look.tree_view_template::<T>())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = TreeView::new("ok")
            .look(&look)
            .items([TreeNode::new("root", "Root", SharedString::from("root"))])
            .into_sdk_builder(look);
    }
}

#[cfg(all(test, feature = "test-support"))]
#[test]
fn forwards_independent_wheel_policies_after_look_synthesis() {
    use xgpui_luma::interaction::{WheelScrollPolicy, ScrollBoundaryPolicy, WheelFocusScope};
    let mut app = gpui::TestAppContext::single();
    let control = TreeView::<()>::new("policy")
        .wheel_scroll_policy(WheelScrollPolicy::PassThrough)
        .scroll_boundary_policy(ScrollBoundaryPolicy::Chain)
        .wheel_focus_scope(WheelFocusScope::Owner)
        .into_sdk_builder(ShadcnLook::built_in())
        .spawn(&mut app);
    control.read_with(&app, |view, _| {
        let policy = view.scroll_interaction();
        assert_eq!(policy.wheel, WheelScrollPolicy::PassThrough);
        assert_eq!(policy.boundary, ScrollBoundaryPolicy::Chain);
        assert_eq!(policy.focus_scope, WheelFocusScope::Owner);
    });
}
