crate::motion_profile!(TreeViewMotion => tree_view {
    expansion: crate::motion::Transition,
    position: crate::motion::Transition,
    drop_layout: crate::motion::Transition,
});

use crate::interaction::{ScrollInteraction, WheelScrollPolicy, ScrollBoundaryPolicy, WheelFocusScope};
use std::{sync::Arc, rc::Rc};

/// Owned UI content callback; captures need not be Send or Sync.
pub type TreeViewContent<T> = Rc<dyn Fn(&FlatTreeNode<'_, T>, &mut gpui::Window, &mut gpui::App) -> gpui::AnyElement>;

use gpui::{AppContext, Entity, SharedString};
use lucide_svg_static::Icon as LucideIcon;

use super::{TreeViewControl, TreeViewTemplate, default_tree_view_template};
use super::template::modified_tree_view_template;
use super::state::TreeIndex;
use super::TreeViewError;
use crate::infra::icon::DisclosureIcons;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TreeViewSelectionMode {
    None,
    #[default]
    Single,
    Multiple,
    /// Plain clicks replace selection; Ctrl/Cmd toggles and Shift extends a
    /// stable-ID range in displayed preorder. Enter activates; Space selects.
    Extended,
}

/// Selection settings; defaults preserve the existing Single/Multiple behavior.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TreeViewSelectionPolicy {
    pub mode: TreeViewSelectionMode,
    /// Repeated plain clicks deselect in Single or Extended. Extended retains
    /// other selected nodes; modifier and range gestures are unchanged.
    pub toggle_off: bool,
    /// Keyboard navigation selects its target in Single mode only.
    /// Enabling this does not immediately change selection.
    pub selection_follows_active: bool,
}

impl TreeViewSelectionPolicy {
    pub fn new(mode: TreeViewSelectionMode) -> Self {
        Self { mode, toggle_off: false, selection_follows_active: false }
    }
}

/// A node in the hierarchical tree model.
#[derive(Clone)]
pub struct TreeNode<T> {
    /// Stable identity, unique across every loaded node in this tree. Keep it
    /// unchanged when moving or renaming a node; do not derive it from its path.
    pub id: SharedString,
    pub label: SharedString,
    pub icon: Option<LucideIcon>,
    pub data: T,
    pub children: Vec<TreeNode<T>>,
    /// Show a branch chevron even when `children` is empty (lazy loading).
    pub is_branch: bool,
    pub initially_expanded: bool,
    pub enabled: bool,
}

impl<T> TreeNode<T> {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>, data: T) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            data,
            children: Vec::new(),
            is_branch: false,
            initially_expanded: false,
            enabled: true,
        }
    }

    pub fn icon(mut self, icon: LucideIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn branch(mut self, is_branch: bool) -> Self {
        self.is_branch = is_branch;
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = TreeNode<T>>) -> Self {
        self.children = children.into_iter().collect();
        self.is_branch = !self.children.is_empty() || self.is_branch;
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.initially_expanded = expanded;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Read-only view of one flattened row passed to templates.
pub struct FlatTreeNode<'a, T> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub icon: Option<LucideIcon>,
    pub depth: usize,
    pub has_children: bool,
    pub expanded: bool,
    /// Branch expand progress (`0..1`) for chevron rotation.
    pub expand_progress: f32,
    pub disclosure_icons: &'a DisclosureIcons,
    /// Layout height factor (`0..1`) for clipped expand/collapse of descendant rows.
    pub row_height_factor: f32,
    pub enabled: bool,
    pub state: crate::infra::state::CompositeItemState,
    pub data: &'a T,
}

#[derive(Clone)]
pub struct TreeViewModel<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub(crate) motion: crate::motion::MotionConfig<TreeViewMotion>,
    pub(crate) id: SharedString,
    pub(crate) items: Vec<TreeNode<T>>,
    pub(super) index: TreeIndex,
    pub(super) filter: Option<super::projection::Filter<T>>,
    pub(super) sort: Option<super::projection::Sort<T>>,
    pub(crate) selection_policy: TreeViewSelectionPolicy,
    pub(crate) enabled: bool,
    pub(crate) motion_enabled: bool,
    pub(super) drag_drop: Option<super::TreeViewDragDrop<T>>,
    pub(super) branch_content: Option<TreeViewContent<T>>,
    pub(super) leaf_content: Option<TreeViewContent<T>>,
    pub(super) expand_on_row_click: bool,
    pub(crate) scroll_interaction: ScrollInteraction,
    pub(crate) pointer_focus: crate::interaction::PointerFocusPolicy,
    pub(super) wrap_navigation: bool,
    pub(super) scrollbar_visibility: crate::controls::ScrollbarVisibility,
    pub(super) scrollbar_template: Option<Arc<dyn crate::controls::scrollbar::ScrollbarTemplate>>,
    pub(crate) disclosure_icons_explicit: bool,
    pub(crate) disclosure_icons: DisclosureIcons,
    pub(crate) template: Arc<dyn TreeViewTemplate<T>>,
}

pub struct TreeViewRenderModel<'a> {
    pub id: &'a SharedString,
    pub selection_mode: TreeViewSelectionMode,
    pub enabled: bool,
    pub focus: crate::infra::state::ControlFocusState,
}

pub struct TreeViewBuilder<T>
where
    T: Clone + Send + Sync + 'static,
{
    callbacks: crate::infra::callbacks::EventCallbacks<crate::controls::tree_view::TreeViewEvent<T>>,

    pub(crate) model: TreeViewModel<T>,
}

impl<T> TreeViewBuilder<T>
where
    T: Clone + Send + Sync + 'static,
{
    crate::motion_methods!(TreeViewMotion, model.motion);

    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::tree_view::TreeViewEvent<T>, &mut gpui::Context<M>) + 'static,
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
            if let crate::controls::tree_view::TreeViewDragEvent::ItemsRemoved { source, target, node_id, node_ids } =
                event
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
            &crate::controls::tree_view::TreeDropLocation,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::controls::tree_view::TreeViewDragEvent::ItemsAdded {
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
            &crate::controls::tree_view::TreeDropLocation,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::controls::tree_view::TreeViewDragEvent::ItemsReordered {
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
            if let crate::controls::tree_view::TreeViewDragEvent::Started { source, node_id, node_ids } = event {
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
            &crate::controls::tree_view::TreeDropLocation,
            bool,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::controls::tree_view::TreeViewDragEvent::Dropped {
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
            if let crate::controls::tree_view::TreeViewDragEvent::Rejected {
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
            crate::controls::tree_view::TreeDragEndReason,
            &mut gpui::Context<M>,
        ) + 'static,
    ) -> Self {
        self.on_drag_drop(cx, move |owner, event, cx| {
            if let crate::controls::tree_view::TreeViewDragEvent::Ended { source, node_id, node_ids, reason } = event {
                callback(owner, *source, node_id, node_ids, *reason, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::DragDrop` through the existing event stream.
    pub fn on_drag_drop<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::tree_view::TreeViewDragEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tree_view::TreeViewEvent::DragDrop(event) = event {
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
            if let crate::controls::tree_view::TreeViewEvent::NodeActivated { node_id, data } = event {
                callback(owner, node_id, data, cx);
            }
        })
    }

    /// Handle `TreeViewEvent::SelectionPolicyChanged` through the existing event stream.
    pub fn on_selection_policy_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::tree_view::TreeViewSelectionPolicy, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::tree_view::TreeViewEvent::SelectionPolicyChanged { policy } = event {
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
            if let crate::controls::tree_view::TreeViewEvent::NodeExpanded { node_id, data } = event {
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
            if let crate::controls::tree_view::TreeViewEvent::NodeCollapsed { node_id, data } = event {
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
            if let crate::controls::tree_view::TreeViewEvent::SelectionChanged { selected_ids } = event {
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
            if let crate::controls::tree_view::TreeViewEvent::ActiveNodeChanged { node_id } = event {
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
            if let crate::controls::tree_view::TreeViewEvent::ScrollChanged { top_index } = event {
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
            if let crate::controls::tree_view::TreeViewEvent::FocusChanged { focused } = event {
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
            if let crate::controls::tree_view::TreeViewEvent::RowHoverChanged { node_id, index, hovered } = event {
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
            if let crate::controls::tree_view::TreeViewEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            callbacks: Default::default(),

            model: TreeViewModel {
                motion: Default::default(),
                id: id.into(),
                items: Vec::new(),
                index: TreeIndex::default(),
                filter: None,
                sort: None,
                selection_policy: TreeViewSelectionPolicy::new(TreeViewSelectionMode::Single),
                enabled: true,
                motion_enabled: true,
                drag_drop: None,
                branch_content: None,
                leaf_content: None,
                expand_on_row_click: true,
                scroll_interaction: ScrollInteraction::VIEWPORT,
                pointer_focus: Default::default(),
                wrap_navigation: true,
                scrollbar_visibility: crate::controls::ScrollbarVisibility::AutoHide,
                scrollbar_template: None,
                disclosure_icons_explicit: false,
                disclosure_icons: DisclosureIcons::default(),
                template: default_tree_view_template::<T>(),
            },
        }
    }

    /// Set initial items. Invalid duplicate IDs leave the previous items intact.
    /// Use [`Self::try_items`] to report validation errors to the caller.
    pub fn items(mut self, items: impl IntoIterator<Item = TreeNode<T>>) -> Self {
        let items: Vec<_> = items.into_iter().collect();
        if let Ok(index) = TreeIndex::new(&items) {
            self.model.items = items;
            self.model.index = index;
        }
        self
    }

    /// Validate IDs across the entire loaded tree before setting initial items.
    pub fn try_items(mut self, items: impl IntoIterator<Item = TreeNode<T>>) -> Result<Self, TreeViewError> {
        let items: Vec<_> = items.into_iter().collect();
        let index = TreeIndex::new(&items)?;
        self.model.items = items;
        self.model.index = index;
        Ok(self)
    }

    /// Show loaded matches and their ancestor paths. Matching a branch does not
    /// include unmatched descendants. Ancestors open without changing user expansion.
    /// The predicate is reevaluated on data replacement or `set_filter`.
    pub fn filter(mut self, predicate: impl Fn(&TreeNode<T>) -> bool + 'static) -> Self {
        self.model.filter = Some(Rc::new(predicate));
        self
    }

    /// Stable display-only sibling ordering; source items are never reordered.
    pub fn sort(mut self, compare: impl Fn(&TreeNode<T>, &TreeNode<T>) -> std::cmp::Ordering + 'static) -> Self {
        self.model.sort = Some(Rc::new(compare));
        self
    }

    pub fn selection_mode(mut self, mode: TreeViewSelectionMode) -> Self {
        self.model.selection_policy.mode = mode;
        self
    }

    /// Configure optional plain-click deselection and Single-mode keyboard selection.
    pub fn selection_policy(mut self, policy: TreeViewSelectionPolicy) -> Self {
        self.model.selection_policy = policy;
        self
    }

    /// Configure pointer_focus independently of wheel and keyboard ownership.
    pub fn pointer_focus_policy(mut self, policy: crate::interaction::PointerFocusPolicy) -> Self {
        self.model.pointer_focus = policy;
        self
    }

    /// Override wheel without changing other interaction settings.
    pub fn wheel_scroll_policy(mut self, policy: WheelScrollPolicy) -> Self {
        self.model.scroll_interaction.wheel = policy;
        self
    }

    /// Override boundary without changing other interaction settings.
    pub fn scroll_boundary_policy(mut self, policy: ScrollBoundaryPolicy) -> Self {
        self.model.scroll_interaction.boundary = policy;
        self
    }

    /// Override focus_scope without changing other interaction settings.
    pub fn wheel_focus_scope(mut self, policy: WheelFocusScope) -> Self {
        self.model.scroll_interaction.focus_scope = policy;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    /// Animate small branch transitions. When simultaneous transitions affect
    /// more than 256 displayed descendants, settle before layout to keep row
    /// construction bounded as descendant heights shrink.
    pub fn motion_enabled(mut self, motion_enabled: bool) -> Self {
        self.model.motion_enabled = motion_enabled;
        self
    }

    /// Enable scoped dragging and synchronous host-owned drops.
    /// The configuration can opt into dragging selected groups.
    pub fn drag_drop(mut self, config: super::TreeViewDragDrop<T>) -> Self {
        self.model.drag_drop = Some(config);
        self
    }

    /// Render branch content inside the themed row shell. Wrap interactive
    /// children in `DragDropElementExt::drag_boundary()` to isolate their input.
    pub fn branch_content<F, E>(mut self, content: F) -> Self
    where
        F: Fn(&FlatTreeNode<'_, T>, &mut gpui::Window, &mut gpui::App) -> E + 'static,
        E: gpui::IntoElement + 'static,
    {
        self.model.branch_content = Some(Rc::new(move |node, window, cx| content(node, window, cx).into_any_element()));
        self
    }

    /// Render leaf content inside the themed row shell.
    pub fn leaf_content<F, E>(mut self, content: F) -> Self
    where
        F: Fn(&FlatTreeNode<'_, T>, &mut gpui::Window, &mut gpui::App) -> E + 'static,
        E: gpui::IntoElement + 'static,
    {
        self.model.leaf_content = Some(Rc::new(move |node, window, cx| content(node, window, cx).into_any_element()));
        self
    }

    /// Defaults to true for compatibility. False reserves expansion for the
    /// disclosure control and Left/Right keys; clicking a row only selects it.
    pub fn expand_on_row_click(mut self, expand: bool) -> Self {
        self.model.expand_on_row_click = expand;
        self
    }

    /// Compatibility alias: true selects RequireFocus, false selects Pointer.
    /// Leaves boundary policy unchanged; the last explicit wheel setter wins.
    pub fn require_focus_for_scroll(mut self, required: bool) -> Self {
        self.model.scroll_interaction.wheel = required.into();
        self
    }

    /// Whether Up/Down wrap at the first/last eligible row. Defaults to true.
    pub fn wrap_navigation(mut self, wrap: bool) -> Self {
        self.model.wrap_navigation = wrap;
        self
    }

    /// Control scrollbar chrome independently of scrolling. Defaults to AutoHide,
    /// which reveals on scrolling and hides after the toolkit's idle delay.
    /// Hidden reserves no gutter; AlwaysVisible shows only when scrollable.
    pub fn scrollbar_visibility(mut self, visibility: crate::controls::ScrollbarVisibility) -> Self {
        self.model.scrollbar_visibility = visibility;
        self
    }

    /// Add a vertical SDK scrollbar connected to this tree's list viewport.
    pub fn scrollbar_template(mut self, template: Arc<dyn crate::controls::scrollbar::ScrollbarTemplate>) -> Self {
        self.model.scrollbar_template = Some(template);
        self
    }

    /// Sets the expanded and collapsed disclosure icons used by branch rows.
    /// Capture shared disclosure defaults; explicit glyphs win in either method order.
    pub fn stylesheet(mut self, stylesheet: &crate::theme::stylesheet::CommonStylesheet) -> Self {
        if !self.model.disclosure_icons_explicit {
            self.model.disclosure_icons = DisclosureIcons::from_stylesheet(stylesheet);
        }
        self
    }

    pub fn disclosure_icons(mut self, icons: DisclosureIcons) -> Self {
        self.model.disclosure_icons_explicit = true;
        self.model.disclosure_icons = icons;
        self
    }

    pub fn template(mut self, template: Arc<dyn TreeViewTemplate<T>>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &TreeViewRenderModel<'_>) -> gpui::Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.model.template = modified_tree_view_template(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn(mut self, cx: &mut impl AppContext) -> Entity<TreeViewControl<T>> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = { cx.new(|cx| TreeViewControl::from_builder(self, cx)) };
        callbacks.bind(&control, cx);
        control
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_tree_view_template::<()>();
        let builder = TreeViewBuilder::new("tree-test")
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }

    #[test]
    fn duplicate_ids_are_rejected_across_collapsed_branches() {
        let nodes =
            vec![TreeNode::new("a", "A", ()).children([TreeNode::new("b", "B", ())]), TreeNode::new("b", "B", ())];
        assert!(
            matches!(TreeViewBuilder::new("test").try_items(nodes.clone()), Err(TreeViewError::DuplicateId(id)) if id.as_ref() == "b")
        );
        let builder = TreeViewBuilder::new("test").items([TreeNode::new("valid", "Valid", ())]).items(nodes);
        assert_eq!(builder.model.items.len(), 1);
        assert_eq!(builder.model.items[0].id.as_ref(), "valid");
    }
}
