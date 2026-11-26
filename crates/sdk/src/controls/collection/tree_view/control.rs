#[cfg(all(test, feature = "test-support"))]
use gpui::ScrollEvent;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use gpui::{
    App, ClickEvent, Context, EventEmitter, FocusOutEvent, Focusable, FocusHandle, IntoElement, ListAlignment,
    ListState, Render, SharedString, Subscription, Window, div, list, prelude::*, px,
};

use super::{
    FlatTreeNode, TreeNode, TreeViewBuilder, TreeViewModel, TreeViewRenderModel, TreeViewSelectionMode,
    TreeViewTemplateHandlers,
};
use crate::motion::DisclosureMotion;
use crate::infra::drag_drop::DragDropElementExt;
use crate::infra::state::{CompositeItemState, ControlFocusState};
use crate::key_handling::{
    ActivateControl, ControlKeyProfile, SelectFirstItem, SelectLastItem, SelectNextItem, SelectPreviousItem,
};
use crate::theme::observe_theme_revision;
use super::state::TreeIndex;
use super::TreeViewError;

const EXPAND_VISIBLE_EPSILON: f32 = 0.001;
// Tiny animated rows can defeat viewport virtualization. Large simultaneous
// transitions settle before layout instead of constructing thousands of rows.
const MAX_ANIMATED_DESCENDANTS: usize = 256;

mod viewport;
mod selection;
mod position;
mod projection;
mod drag;
mod transfer;
pub use transfer::TreeViewSubtreeState;

gpui::actions!(tree_view, [ExpandNode, CollapseNode, SelectAllNodes, ClearSelectedNodes]);

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum TreeViewEvent<T>
where
    T: Clone,
{
    DragDrop(super::TreeViewDragEvent),
    NodeActivated { node_id: SharedString, data: T },
    SelectionPolicyChanged { policy: super::TreeViewSelectionPolicy },
    NodeExpanded { node_id: SharedString, data: T },
    NodeCollapsed { node_id: SharedString, data: T },
    SelectionChanged { selected_ids: HashSet<SharedString> },
    ActiveNodeChanged { node_id: Option<SharedString> },
    ScrollChanged { top_index: usize },
    FocusChanged { focused: bool },
    RowHoverChanged { node_id: SharedString, index: usize, hovered: bool },
    EnabledChanged { enabled: bool },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TreeDirection {
    Previous,
    Next,
}

pub(crate) struct FlatNodeWrapper<T> {
    id: SharedString,
    label: SharedString,
    icon: Option<lucide_svg_static::Icon>,
    depth: usize,
    has_children: bool,
    enabled: bool,
    expand_progress: f32,
    row_height_factor: f32,
    data: T,
}

pub struct TreeViewControl<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub(crate) drop_layout: crate::motion::DropLayoutMotion<SharedString>,

    pub(super) model: TreeViewModel<T>,
    pub(super) revision: u64,
    pub(super) drag_revision: Option<u64>,
    drag_scroll_scheduled: bool,
    drag_scroll_last_frame: Option<std::time::Instant>,
    focus_handle: FocusHandle,
    focus_in_subscription: Option<Subscription>,
    focus_out_subscription: Option<Subscription>,
    list_state: ListState,
    position: Option<position::PositionRequest>,
    position_frame_scheduled: bool,
    scrollbar: Option<gpui::Entity<crate::controls::scrollbar::Scrollbar>>,
    _scrollbar_subscription: Option<Subscription>,
    scrollbar_active: bool,
    scrollbar_hide_task: Option<gpui::Task<()>>,
    /// Maps each row's drawn state (`set_item_state_override`).
    item_state_override: Option<crate::infra::state::ItemStateOverride<SharedString>>,
    scrollbar_scrollable: bool,
    viewport_offset: Option<gpui::ListOffset>,
    projection: super::projection::Projection,
    expanded_ids: HashSet<SharedString>,
    expand_transitions: HashMap<SharedString, DisclosureMotion>,
    selected_ids: HashSet<SharedString>,
    active_node_id: Option<SharedString>,
    selection_anchor: Option<SharedString>,
    hovered_node_id: Option<SharedString>,
    pub(super) pressed_node_id: Option<SharedString>,
    flat_cache: Vec<FlatNodeWrapper<T>>,
    flat_indices: HashMap<SharedString, usize>,
    row_handlers: RefCell<HashMap<SharedString, TreeViewTemplateHandlers>>,
    emitted_scroll_top_index: usize,
    emitted_focused: bool,
}

impl<T> EventEmitter<TreeViewEvent<T>> for TreeViewControl<T> where T: Clone + Send + Sync + 'static {}

impl<T> TreeViewControl<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Current wheel settings. Keyboard ownership is independent.
    pub fn scroll_interaction(&self) -> crate::interaction::ScrollInteraction {
        self.model.scroll_interaction
    }

    /// Change wheel without resetting focus, selection or position.
    pub fn set_wheel_scroll_policy(&mut self, policy: crate::interaction::WheelScrollPolicy, cx: &mut Context<Self>) {
        self.model.scroll_interaction.wheel = policy;
        cx.notify();
    }

    /// Change boundary without resetting focus, selection or position.
    pub fn set_scroll_boundary_policy(
        &mut self,
        policy: crate::interaction::ScrollBoundaryPolicy,
        cx: &mut Context<Self>,
    ) {
        self.model.scroll_interaction.boundary = policy;
        cx.notify();
    }

    /// Change focus_scope without resetting focus, selection or position.
    pub fn set_wheel_focus_scope(&mut self, policy: crate::interaction::WheelFocusScope, cx: &mut Context<Self>) {
        self.model.scroll_interaction.focus_scope = policy;
        cx.notify();
    }

    crate::motion_snapshot!(super::TreeViewMotion, model.motion);

    pub(crate) fn from_builder(builder: TreeViewBuilder<T>, cx: &mut Context<Self>) -> Self {
        let mut builder = builder;
        builder.model.motion.freeze();
        let enabled = builder.model.enabled;
        observe_theme_revision(cx, |_, cx| cx.notify()).detach();
        let scrollbar = builder.model.scrollbar_template.as_ref().map(|template| {
            crate::controls::scrollbar::Scrollbar::new(format!("{}-scrollbar", builder.model.id))
                .vertical()
                .template(template.clone())
                .spawn(cx)
        });
        let scrollbar_subscription = scrollbar
            .as_ref()
            .map(|scrollbar| cx.subscribe(scrollbar, |this, _, event, cx| this.handle_scrollbar(event, cx)));

        let projection = super::projection::Projection::new(
            &builder.model.items,
            builder.model.filter.as_ref(),
            builder.model.sort.as_ref(),
        );
        let mut control = Self {
            model: builder.model,
            drop_layout: Default::default(),
            revision: 0,
            drag_revision: None,
            drag_scroll_scheduled: false,
            drag_scroll_last_frame: None,
            focus_handle: cx.focus_handle().tab_stop(enabled),
            focus_in_subscription: None,
            focus_out_subscription: None,
            list_state: ListState::new(0, ListAlignment::Top, px(480.0)),
            position: None,
            position_frame_scheduled: false,
            scrollbar,
            _scrollbar_subscription: scrollbar_subscription,
            scrollbar_active: false,
            scrollbar_hide_task: None,
            item_state_override: None,
            scrollbar_scrollable: false,
            viewport_offset: None,
            projection,
            expanded_ids: HashSet::new(),
            expand_transitions: HashMap::new(),
            selected_ids: HashSet::new(),
            active_node_id: None,
            selection_anchor: None,
            hovered_node_id: None,
            pressed_node_id: None,
            flat_cache: Vec::new(),
            flat_indices: HashMap::new(),
            row_handlers: RefCell::new(HashMap::new()),
            emitted_scroll_top_index: 0,
            emitted_focused: false,
        };

        control.populate_initial_expands();
        control.sync_expand_transitions_for_tree();
        control.rebuild_flat_cache(cx);

        control
    }

    /// Retained keyed post-drop geometry and running transitions.
    pub fn drop_layout_motion(&self) -> crate::motion::DropLayoutMotion<SharedString> {
        self.drop_layout.clone()
    }

    pub fn items(&self) -> &[TreeNode<T>] {
        &self.model.items
    }

    pub fn selected_ids(&self) -> &HashSet<SharedString> {
        &self.selected_ids
    }

    pub fn active_node_id(&self) -> Option<&SharedString> {
        self.active_node_id.as_ref()
    }

    /// Effective expansion, including ancestor paths opened by a filter.
    pub fn is_expanded(&self, node_id: &SharedString) -> bool {
        self.expanded_ids.contains(node_id) || self.projection.forced_expanded.contains(node_id)
    }

    /// Reset items, selection, active state, and expansion. Duplicate IDs leave
    /// the control unchanged; use [`Self::try_set_items`] to handle errors.
    pub fn set_items(&mut self, items: impl IntoIterator<Item = TreeNode<T>>, cx: &mut Context<Self>) {
        let _ = self.try_set_items(items, cx);
    }

    /// Validated counterpart of [`Self::set_items`]. Invalid data changes nothing.
    pub fn try_set_items(
        &mut self,
        items: impl IntoIterator<Item = TreeNode<T>>,
        cx: &mut Context<Self>,
    ) -> Result<(), TreeViewError> {
        self.update_items(items.into_iter().collect(), false, cx)
    }

    /// Replace loaded data while preserving state by ID. Selected enabled nodes
    /// remain selected even under collapsed parents. Surviving branches retain
    /// user expansion; only new branches use `initially_expanded`.
    ///
    /// Active state falls back to a visible enabled ancestor, then the nearest
    /// displayed enabled row. An inactive tree stays inactive. Selection and
    /// active events are emitted only when changed, with final-state payloads.
    /// Expansion hints are applied without synthetic expand/collapse events.
    /// All IDs are validated before mutation. Content always repaints, and the
    /// top row's key/offset is retained when that row remains displayed.
    pub fn replace_items(
        &mut self,
        items: impl IntoIterator<Item = TreeNode<T>>,
        cx: &mut Context<Self>,
    ) -> Result<(), TreeViewError> {
        self.update_items(items.into_iter().collect(), true, cx)
    }

    fn update_items(
        &mut self,
        items: Vec<TreeNode<T>>,
        preserve: bool,
        cx: &mut Context<Self>,
    ) -> Result<(), TreeViewError> {
        let index = TreeIndex::new(&items)?;
        self.drop_layout.cancel();
        self.cancel_position();
        let selected_before = self.selected_ids.clone();
        let active_before = self.active_node_id.clone();
        let expanded = if preserve {
            index.reconcile_expanded(&self.model.index, &self.expanded_ids)
        } else {
            index.initial_expanded()
        };
        let projection =
            super::projection::Projection::new(&items, self.model.filter.as_ref(), self.model.sort.as_ref());
        let active = if preserve {
            index.reconcile_active(
                &self.model.index,
                active_before.as_ref(),
                &self.visible_ids(),
                &projection.visible(&items, &expanded),
            )
        } else {
            None
        };
        self.selection_anchor = if preserve {
            index.reconcile_active(
                &self.model.index,
                self.selection_anchor.as_ref(),
                &self.visible_ids(),
                &projection.visible(&items, &expanded),
            )
        } else {
            None
        };
        let old_top = self.list_state.logical_scroll_top();
        let top_id = self.flat_cache.get(old_top.item_ix).map(|node| node.id.clone());
        self.clear_hovered_node_id(cx);
        self.pressed_node_id = None;
        self.selected_ids.retain(|id| preserve && index.nodes.get(id).is_some_and(|info| info.enabled));
        self.active_node_id = active;
        self.expanded_ids = expanded;
        self.revision = self.revision.wrapping_add(1);
        self.model.items = items;
        self.model.index = index;
        self.projection = projection;
        // Replacement settles old geometry; motion tied to old ancestry must
        // not leave removed/reparented descendants in the visible cache.
        self.expand_transitions.clear();
        self.sync_expand_transitions_for_tree();
        self.rebuild_flat_cache(cx);
        // Same-count updates can change content, depth, and custom row heights.
        self.list_state.reset(self.flat_cache.len());
        if preserve && let Some(index) = top_id.as_ref().and_then(|id| self.flat_index_for_id(id)) {
            self.list_state.scroll_to(gpui::ListOffset { item_ix: index, ..old_top });
        }
        if selected_before != self.selected_ids {
            cx.emit(TreeViewEvent::SelectionChanged { selected_ids: self.selected_ids.clone() });
        }
        if active_before != self.active_node_id {
            cx.emit(TreeViewEvent::ActiveNodeChanged { node_id: self.active_node_id.clone() });
        }
        self.emit_scroll_changed_if_needed(cx);
        Ok(())
    }

    /// Control-wide policy. `false` settles motion immediately and keeps [`Self::motion_profile`].
    pub fn motion_enabled(&self) -> bool {
        self.model.motion_enabled
    }

    pub fn set_motion_enabled(&mut self, motion_enabled: bool, cx: &mut Context<Self>) {
        if self.model.motion_enabled == motion_enabled {
            return;
        }
        self.cancel_position();
        self.model.motion_enabled = motion_enabled;
        for (id, transition) in &mut self.expand_transitions {
            transition.set_motion_enabled(motion_enabled);
            transition.set_target(if self.expanded_ids.contains(id) { 1.0 } else { 0.0 });
        }
        self.rebuild_flat_cache(cx);
        self.list_state.remeasure();
        cx.notify();
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn super::TreeViewTemplate<T>>, cx: &mut Context<Self>) {
        self.cancel_position();
        self.model.template = template;
        self.list_state.remeasure();
        cx.notify();
    }

    /// Draw each row in the state `map` gives it, from its node id and the state input gives it, or as input gives it
    /// (`None`). Only the look changes; selection, input and events follow the real state (#397).
    pub fn set_item_state_override(
        &mut self,
        map: Option<crate::infra::state::ItemStateOverride<SharedString>>,
        cx: &mut Context<Self>,
    ) {
        self.item_state_override = map;
        self.list_state.remeasure();
        cx.notify();
    }

    /// Refresh the look of an existing scrollbar; does not add a viewport.
    pub fn set_scrollbar_template(
        &mut self,
        template: std::sync::Arc<dyn crate::controls::scrollbar::ScrollbarTemplate>,
        cx: &mut Context<Self>,
    ) {
        if let Some(scrollbar) = &self.scrollbar {
            scrollbar.update(cx, |scrollbar, cx| scrollbar.set_template(template, cx));
        }
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }

        self.cancel_position();
        self.model.enabled = enabled;
        self.focus_handle = self.focus_handle.clone().tab_stop(enabled);
        self.focus_in_subscription = None;
        self.focus_out_subscription = None;

        if !enabled {
            self.clear_hovered_node_id(cx);
            self.pressed_node_id = None;
            self.emit_focus_changed(false, cx);
        }

        cx.emit(TreeViewEvent::EnabledChanged { enabled });
        cx.notify();
    }

    pub fn expand(&mut self, node_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        let node_id = node_id.into();
        if !self.model.index.nodes.get(&node_id).is_some_and(|info| info.branch) {
            return;
        }
        if self.projection.forced_expanded.contains(&node_id) {
            return;
        }
        self.cancel_position();
        let already_expanded = self.expanded_ids.contains(&node_id);
        if !already_expanded {
            self.expanded_ids.insert(node_id.clone());
            if let Some(data) = self.data_for_id(&node_id) {
                cx.emit(TreeViewEvent::NodeExpanded { node_id: node_id.clone(), data });
            }
        }

        let was_visible = self.branch_children_visible(&node_id);
        self.set_expand_target(&node_id, 1.0);

        if !was_visible {
            self.rebuild_flat_cache(cx);
        } else {
            cx.notify();
        }
    }

    pub fn collapse(&mut self, node_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        let node_id = node_id.into();
        if self.projection.forced_expanded.contains(&node_id) {
            return;
        }
        self.cancel_position();
        let was_expanded = self.expanded_ids.remove(&node_id);
        if !was_expanded && !self.branch_children_visible(&node_id) {
            return;
        }

        if was_expanded && let Some(data) = self.data_for_id(&node_id) {
            cx.emit(TreeViewEvent::NodeCollapsed { node_id: node_id.clone(), data });
        }

        self.set_expand_target(&node_id, 0.0);
        let visible = self.visible_ids();
        self.selection_anchor = self.model.index.reconcile_active(
            &self.model.index,
            self.selection_anchor.as_ref(),
            &self.flat_cache.iter().map(|row| row.id.clone()).collect::<Vec<_>>(),
            &visible,
        );
        let next_active = self.model.index.reconcile_active(
            &self.model.index,
            self.active_node_id.as_ref(),
            &self.flat_cache.iter().map(|row| row.id.clone()).collect::<Vec<_>>(),
            &visible,
        );
        if self.active_node_id != next_active {
            self.active_node_id = next_active.clone();
            cx.emit(TreeViewEvent::ActiveNodeChanged { node_id: next_active });
        }
        self.refresh_flat_motion_factors();
        if self.hovered_node_id.as_ref().is_some_and(|id| !visible.contains(id)) {
            self.clear_hovered_node_id(cx);
        }
        if self.pressed_node_id.as_ref().is_some_and(|id| !visible.contains(id)) {
            self.pressed_node_id = None;
        }

        if !self.model.motion_enabled {
            self.rebuild_flat_cache(cx);
        } else {
            cx.notify();
        }
    }

    pub fn toggle_expand(&mut self, node_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        let node_id = node_id.into();
        if self.is_expanded(&node_id) {
            self.collapse(node_id, cx);
        } else {
            self.expand(node_id, cx);
        }
    }

    pub fn select_node_by_id(&mut self, node_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        if matches!(self.model.selection_policy.mode, TreeViewSelectionMode::None) {
            return;
        }

        let node_id = node_id.into();
        let Some(index) = self.flat_index_for_id(&node_id).filter(|&index| self.flat_cache[index].enabled) else {
            return;
        };

        if self.model.selection_policy.mode == TreeViewSelectionMode::Extended {
            self.selection_anchor = Some(node_id.clone());
        }
        let selection_changed = self.selected_ids.len() != 1 || !self.selected_ids.contains(&node_id);
        self.selected_ids.clear();
        self.selected_ids.insert(node_id.clone());
        self.set_active_index(Some(index), false, cx);
        if selection_changed {
            cx.emit(TreeViewEvent::SelectionChanged { selected_ids: self.selected_ids.clone() });
        }
        cx.notify();
    }

    fn populate_initial_expands(&mut self) {
        self.expanded_ids = self.model.index.initial_expanded();
    }

    fn sync_expand_transitions_for_tree(&mut self) {
        let mut branch_ids = HashSet::new();

        fn collect_branches<T>(node: &TreeNode<T>, branch_ids: &mut HashSet<SharedString>) {
            if !node.children.is_empty() || node.is_branch {
                branch_ids.insert(node.id.clone());
            }
            for child in &node.children {
                collect_branches(child, branch_ids);
            }
        }

        for item in &self.model.items {
            collect_branches(item, &mut branch_ids);
        }

        self.expand_transitions.retain(|id, _| branch_ids.contains(id));
        for id in branch_ids {
            let target = if self.expanded_ids.contains(&id) { 1.0 } else { 0.0 };
            self.expand_transitions
                .entry(id)
                .and_modify(|transition| {
                    if !transition.is_animating() {
                        *transition = DisclosureMotion::with_transition(
                            target,
                            self.model.motion_enabled,
                            self.model.motion.profile().expansion.clone(),
                        );
                    }
                })
                .or_insert_with(|| {
                    DisclosureMotion::with_transition(
                        target,
                        self.model.motion_enabled,
                        self.model.motion.profile().expansion.clone(),
                    )
                });
        }
    }

    fn expand_progress_for(&self, node_id: &SharedString) -> f32 {
        if self.projection.forced_expanded.contains(node_id) {
            return 1.0;
        }
        self.expand_transitions
            .get(node_id)
            .map(DisclosureMotion::progress)
            .unwrap_or_else(|| if self.expanded_ids.contains(node_id) { 1.0 } else { 0.0 })
    }

    fn branch_children_visible(&self, node_id: &SharedString) -> bool {
        self.expand_progress_for(node_id) > EXPAND_VISIBLE_EPSILON
            || self.expand_transitions.get(node_id).is_some_and(DisclosureMotion::is_animating)
    }

    fn set_expand_target(&mut self, node_id: &SharedString, target: f32) {
        let transition = self.expand_transitions.entry(node_id.clone()).or_insert_with(|| {
            DisclosureMotion::with_transition(
                if target > 0.5 { 0.0 } else { 1.0 },
                self.model.motion_enabled,
                self.model.motion.profile().expansion.clone(),
            )
        });
        transition.set_target(target);
    }

    fn sync_expand_transitions(&mut self) -> (bool, bool) {
        let mut was_animating = false;
        let mut is_animating = false;

        for transition in self.expand_transitions.values_mut() {
            was_animating |= transition.is_animating();
            is_animating |= transition.sync();
        }

        (was_animating, is_animating)
    }

    fn desired_flat_count(&self) -> usize {
        let mut count = 0;
        self.visit_projected_rows(|_, _, _, _, _| count += 1);
        count
    }

    fn settle_large_expansions(&mut self) -> bool {
        let mut affected = 0;
        let mut ancestors = vec![false];
        self.projection.visit(&self.model.items, |node, depth| {
            let inherited = ancestors[depth];
            if inherited {
                affected += 1;
            }
            ancestors.truncate(depth + 1);
            ancestors
                .push(inherited || self.expand_transitions.get(&node.id).is_some_and(DisclosureMotion::is_animating));
            affected <= MAX_ANIMATED_DESCENDANTS && self.branch_children_visible(&node.id)
        });
        if affected <= MAX_ANIMATED_DESCENDANTS {
            return false;
        }
        for (id, motion) in &mut self.expand_transitions {
            if motion.is_animating() {
                *motion = DisclosureMotion::with_transition(
                    if self.expanded_ids.contains(id) { 1.0 } else { 0.0 },
                    self.model.motion_enabled,
                    self.model.motion.profile().expansion.clone(),
                );
            }
        }
        true
    }

    /// Splice the changed keyed range, retaining unaffected measurements and
    /// scroll anchors even when multiple branches finish animating together.
    fn rebuild_flat_cache(&mut self, cx: &mut Context<Self>) {
        let old_ids: Vec<_> = self.flat_cache.iter().map(|node| node.id.clone()).collect();
        let old_top = self.list_state.logical_scroll_top();
        let top_id = old_ids.get(old_top.item_ix).cloned();
        let active_before = self.active_node_id.clone();
        let mut flat = Vec::new();

        self.visit_projected_rows(|node, depth, expand_progress, row_height_factor, enabled| {
            flat.push(FlatNodeWrapper {
                id: node.id.clone(),
                label: node.label.clone(),
                icon: node.icon,
                depth,
                has_children: !node.children.is_empty() || node.is_branch,
                enabled,
                expand_progress,
                row_height_factor,
                data: node.data.clone(),
            });
        });
        if self
            .hovered_node_id
            .as_ref()
            .is_some_and(|id| !flat.iter().any(|node| &node.id == id && node.enabled))
        {
            self.clear_hovered_node_id(cx);
        }
        if self
            .pressed_node_id
            .as_ref()
            .is_some_and(|id| !flat.iter().any(|node| &node.id == id && node.enabled))
        {
            self.pressed_node_id = None;
        }
        self.flat_cache = flat;
        self.flat_indices.clear();
        self.flat_indices
            .extend(self.flat_cache.iter().enumerate().map(|(index, node)| (node.id.clone(), index)));

        self.row_handlers.get_mut().retain(|id, _| self.flat_indices.contains_key(id));

        if let Some(active_id) = active_before
            && self.flat_index_for_id(&active_id).is_none()
        {
            self.active_node_id = None;
            cx.emit(TreeViewEvent::ActiveNodeChanged { node_id: None });
        }

        self.sync_list_state_after_flat_change(&old_ids);
        if let Some(index) = top_id.as_ref().and_then(|id| self.flat_index_for_id(id)) {
            // A single splice can encompass multiple completing branches;
            // preserve a surviving key inside that changed range too.
            self.list_state.scroll_to(gpui::ListOffset { item_ix: index, ..old_top });
        }
        cx.notify();
    }

    fn refresh_flat_motion_factors(&mut self) {
        // Traverse exactly the same projected order as row construction. No
        // domain data is cloned while updating animation geometry.
        let mut factors = Vec::with_capacity(self.flat_cache.len());
        self.visit_projected_rows(|_, _, progress, height, enabled| factors.push((progress, height, enabled)));
        for (row, (progress, height, enabled)) in self.flat_cache.iter_mut().zip(factors) {
            row.expand_progress = progress;
            row.row_height_factor = height;
            row.enabled = enabled;
        }
        self.list_state.remeasure_items(0..self.flat_cache.len());
    }

    fn sync_list_state_after_flat_change(&mut self, old_ids: &[SharedString]) {
        let prefix = old_ids.iter().zip(&self.flat_cache).take_while(|(id, row)| **id == row.id).count();
        let suffix = old_ids[prefix..]
            .iter()
            .rev()
            .zip(self.flat_cache[prefix..].iter().rev())
            .take_while(|(id, row)| **id == row.id)
            .count();
        let removed_end = old_ids.len() - suffix;
        let inserted = self.flat_cache.len() - prefix - suffix;
        if prefix != removed_end || inserted != 0 {
            self.list_state.splice(prefix..removed_end, inserted);
        }
    }

    fn flat_index_for_id(&self, node_id: &SharedString) -> Option<usize> {
        self.flat_indices.get(node_id).copied()
    }

    fn data_for_id(&self, node_id: &SharedString) -> Option<T> {
        fn find_in_tree<T: Clone>(nodes: &[TreeNode<T>], node_id: &SharedString) -> Option<T> {
            for node in nodes {
                if &node.id == node_id {
                    return Some(node.data.clone());
                }
                if let Some(data) = find_in_tree(&node.children, node_id) {
                    return Some(data);
                }
            }
            None
        }

        find_in_tree(&self.model.items, node_id)
    }

    fn set_active_index(&mut self, index: Option<usize>, scroll: bool, cx: &mut Context<Self>) -> bool {
        if scroll {
            self.cancel_position();
        }
        let next = index.and_then(|idx| self.flat_cache.get(idx).map(|node| node.id.clone()));
        if self.active_node_id == next {
            if scroll && let Some(idx) = index {
                self.reveal_item(idx);
                self.emit_scroll_changed_if_needed(cx);
            }
            return false;
        }

        self.active_node_id = next.clone();
        if scroll && let Some(idx) = index {
            self.reveal_item(idx);
            self.emit_scroll_changed_if_needed(cx);
        }
        cx.emit(TreeViewEvent::ActiveNodeChanged { node_id: next });
        true
    }

    pub fn toggle_node_at_index(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.flat_cache.len() {
            return;
        }

        let wrapper = &self.flat_cache[index];
        if !wrapper.has_children {
            return;
        }

        let id = wrapper.id.clone();
        if self.is_expanded(&id) {
            self.collapse(id, cx);
        } else {
            self.expand(id, cx);
        }
    }

    fn render_row(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) -> gpui::AnyElement {
        let Some(node) = self.flat_cache.get(index) else {
            return div().into_any_element();
        };

        let focus = ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window);
        let item_enabled = self.model.enabled && node.enabled;
        let selected = self.selected_ids.contains(&node.id);
        let active = self.active_node_id.as_ref() == Some(&node.id);
        let hovered = item_enabled && self.hovered_node_id.as_ref() == Some(&node.id);
        let pressed = item_enabled && self.pressed_node_id.as_ref() == Some(&node.id);

        let state = CompositeItemState {
            hovered,
            pressed,
            disabled: !item_enabled,
            selected,
            active: focus.focused && active,
            focus_visible: focus.focus_visible && active,
        };
        let state = self.item_state_override.as_ref().map_or(state, |map| map(&node.id, state));

        let render_node = FlatTreeNode {
            id: &node.id,
            label: &node.label,
            icon: node.icon,
            depth: node.depth,
            has_children: node.has_children,
            expanded: self.is_expanded(&node.id),
            expand_progress: node.expand_progress,
            disclosure_icons: &self.model.disclosure_icons,
            row_height_factor: node.row_height_factor,
            enabled: item_enabled,
            state,
            data: &node.data,
        };

        let handlers = self.template_row_handlers(node.id.clone(), cx);
        let callback = if node.has_children {
            &self.model.branch_content
        } else {
            &self.model.leaf_content
        };
        let content = if let Some(callback) = callback {
            let content = callback(&render_node, window, cx);
            self.model.template.render_node_with_content(&render_node, handlers, content, window, cx)
        } else {
            self.model.template.render_node(&render_node, handlers, window, cx)
        };
        let id = node.id.clone();
        let label = node.label.clone();
        let row = div()
            .id((id.clone(), 0usize))
            .relative()
            .role(gpui::Role::TreeItem)
            .aria_level(node.depth + 1)
            .aria_label(label.clone())
            .aria_selected(selected)
            .when(node.has_children, |row| row.aria_expanded(render_node.expanded))
            .when(active, |row| row.aria_active_descendant())
            .child(content);
        #[cfg(any(test, feature = "test-support"))]
        let row = {
            let tree_id = self.model.id.clone();
            let debug_node_id = id.clone();
            row.debug_selector(move || format!("{}/node/{}", tree_id, debug_node_id))
        };
        let key = id.clone();
        let row = self.bind_row_drag(row, id, label, index, cx);
        self.drop_layout.item(key, row)
    }

    fn handle_row_hover(&mut self, index: usize, hovered: bool, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.flat_cache.get(index).is_some_and(|node| node.enabled) {
            return;
        }

        let node_id = self.flat_cache[index].id.clone();
        if hovered {
            if self.hovered_node_id.as_ref() != Some(&node_id) {
                self.clear_hovered_node_id(cx);
                self.hovered_node_id = Some(node_id.clone());
                let node_id = self.flat_cache[index].id.clone();
                cx.emit(TreeViewEvent::RowHoverChanged { node_id, index, hovered: true });
                cx.notify();
            }
        } else if self.hovered_node_id.as_ref() == Some(&node_id) {
            self.clear_hovered_node_id(cx);
            if self.pressed_node_id.as_ref() == Some(&node_id) {
                self.pressed_node_id = None;
            }
            cx.notify();
        }
    }

    fn template_row_handlers(&self, node_id: SharedString, cx: &mut Context<Self>) -> TreeViewTemplateHandlers {
        if let Some(handlers) = self.row_handlers.borrow().get(&node_id) {
            return handlers.clone();
        }
        let cache_id = node_id.clone();
        let hover_id = node_id.clone();
        let press_id = node_id.clone();
        let disclosure_id = node_id.clone();
        let handlers = TreeViewTemplateHandlers {
            disclosure: Rc::new(cx.listener(move |this, event: &ClickEvent, window, cx| {
                cx.stop_propagation();
                let Some(idx) = this.flat_index_for_id(&disclosure_id) else {
                    return;
                };
                if event.click_count() != 1 || !this.model.enabled || !this.flat_cache[idx].enabled {
                    return;
                }
                this.cancel_position();
                this.model.pointer_focus.apply(&this.focus_handle, window, cx);
                this.toggle_node_at_index(idx, cx);
                if this.model.expand_on_row_click {
                    this.select_with_modifiers(idx, event.modifiers(), cx);
                }
            })),
            hover: Rc::new(cx.listener(move |this, hovered: &bool, _, cx| {
                if let Some(idx) = this.flat_index_for_id(&hover_id) {
                    this.handle_row_hover(idx, *hovered, cx);
                }
            })),
            mouse_down: Rc::new(cx.listener(move |this, _, window, cx| {
                let Some(idx) = this.flat_index_for_id(&press_id) else {
                    return;
                };
                if this.model.enabled && this.flat_cache.get(idx).is_some_and(|node| node.enabled) {
                    this.cancel_position();
                    this.prepare_selection_anchor();
                    this.pressed_node_id = Some(press_id.clone());
                    this.set_active_index(Some(idx), false, cx);
                    this.model.pointer_focus.apply(&this.focus_handle, window, cx);
                    this.emit_focus_changed(this.focus_handle.is_focused(window), cx);
                    cx.notify();
                }
            })),
            mouse_up: Rc::new(cx.listener(move |this, _, _, cx| {
                this.pressed_node_id = None;
                cx.notify();
            })),
            click: Rc::new(cx.listener(move |this, event: &ClickEvent, _, cx| {
                if event.click_count() != 1 {
                    return;
                }
                let Some(idx) = this.flat_index_for_id(&node_id) else {
                    return;
                };
                if !this.model.enabled || !this.flat_cache[idx].enabled {
                    return;
                }

                let is_branch = this.flat_cache.get(idx).is_some_and(|node| node.has_children);
                if is_branch && this.model.expand_on_row_click {
                    this.toggle_node_at_index(idx, cx);
                }
                this.select_with_modifiers(idx, event.modifiers(), cx);
            })),
        };
        let mut cache = self.row_handlers.borrow_mut();
        // Bound retained callbacks even when a large tree is scrolled without rebuilding its projection.
        if cache.len() >= 256 {
            cache.clear();
        }
        cache.insert(cache_id, handlers.clone());
        handlers
    }

    fn handle_node_select(&mut self, idx: usize, cx: &mut Context<Self>) {
        self.select_with_modifiers(idx, gpui::Modifiers::default(), cx);
    }

    fn move_active(&mut self, direction: TreeDirection, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }

        let current = self.active_node_id.as_ref().and_then(|id| self.flat_index_for_id(id));
        let next = if self.model.wrap_navigation {
            next_enabled_flat_index(&self.flat_cache, current, direction)
        } else {
            match (direction, current) {
                (TreeDirection::Next, Some(index)) => {
                    ((index + 1)..self.flat_cache.len()).find(|&i| self.flat_cache[i].enabled)
                }
                (TreeDirection::Previous, Some(index)) => (0..index).rev().find(|&i| self.flat_cache[i].enabled),
                (TreeDirection::Next, None) => first_enabled_flat_index(&self.flat_cache),
                (TreeDirection::Previous, None) => last_enabled_flat_index(&self.flat_cache),
            }
        };
        if let Some(idx) = next {
            self.navigate_selection(Some(idx), cx);
            cx.notify();
        }
    }

    fn handle_previous(&mut self, _: &SelectPreviousItem, _: &mut Window, cx: &mut Context<Self>) {
        self.move_active(TreeDirection::Previous, cx);
    }

    fn handle_next(&mut self, _: &SelectNextItem, _: &mut Window, cx: &mut Context<Self>) {
        self.move_active(TreeDirection::Next, cx);
    }

    fn handle_first(&mut self, _: &SelectFirstItem, _: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }
        if let Some(idx) = first_enabled_flat_index(&self.flat_cache) {
            self.navigate_selection(Some(idx), cx);
            cx.notify();
        }
    }

    fn handle_last(&mut self, _: &SelectLastItem, _: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }
        if let Some(idx) = last_enabled_flat_index(&self.flat_cache) {
            self.navigate_selection(Some(idx), cx);
            cx.notify();
        }
    }

    fn handle_expand_focused_node(&mut self, _: &ExpandNode, _: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }
        let Some(active_id) = self.active_node_id.clone() else {
            return;
        };
        let Some(idx) = self.flat_index_for_id(&active_id) else {
            return;
        };

        let node = &self.flat_cache[idx];
        if !node.has_children {
            return;
        }

        if !self.is_expanded(&active_id) {
            self.toggle_node_at_index(idx, cx);
            return;
        }

        let depth = node.depth;
        let child = ((idx + 1)..self.flat_cache.len())
            .take_while(|&i| self.flat_cache[i].depth > depth)
            .find(|&i| self.flat_cache[i].depth == depth + 1 && self.flat_cache[i].enabled);
        if let Some(child) = child {
            self.navigate_selection(Some(child), cx);
            cx.notify();
        }
    }

    fn handle_collapse_focused_node(&mut self, _: &CollapseNode, _: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }
        let Some(active_id) = self.active_node_id.clone() else {
            return;
        };
        let Some(idx) = self.flat_index_for_id(&active_id) else {
            return;
        };

        let node = &self.flat_cache[idx];
        if node.has_children && self.is_expanded(&active_id) && !self.projection.forced_expanded.contains(&active_id) {
            self.toggle_node_at_index(idx, cx);
            return;
        }

        if node.depth == 0 {
            return;
        }

        let mut current_depth = node.depth;
        let mut search_idx = idx;
        while search_idx > 0 {
            search_idx -= 1;
            if self.flat_cache[search_idx].depth < current_depth {
                current_depth = self.flat_cache[search_idx].depth;
                if !self.flat_cache[search_idx].enabled {
                    continue;
                }
                self.navigate_selection(Some(search_idx), cx);
                cx.notify();
                break;
            }
        }
    }

    fn handle_activate(&mut self, _: &ActivateControl, _: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled {
            return;
        }
        let Some(active_id) = self.active_node_id.clone() else {
            return;
        };
        let Some(idx) = self.flat_index_for_id(&active_id) else {
            return;
        };

        if self.model.selection_policy.mode == TreeViewSelectionMode::Extended {
            self.activate_node(idx, cx);
            return;
        }
        if self.flat_cache.get(idx).is_some_and(|node| node.has_children) {
            self.toggle_node_at_index(idx, cx);
        } else {
            self.handle_node_select(idx, cx);
        }
    }

    fn clear_hovered_node_id(&mut self, cx: &mut Context<Self>) {
        if let Some(node_id) = self.hovered_node_id.take()
            && let Some(index) = self.flat_index_for_id(&node_id)
        {
            cx.emit(TreeViewEvent::RowHoverChanged { node_id, index, hovered: false });
        }
    }

    fn handle_focus_in(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.model.enabled && self.emit_focus_changed(true, cx) {
            cx.notify();
        }
    }

    fn handle_focus_out(&mut self, _: FocusOutEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.emit_focus_changed(false, cx) {
            cx.notify();
        }
    }

    fn emit_scroll_changed_if_needed(&mut self, cx: &mut Context<Self>) -> bool {
        let top_index = self.list_state.logical_scroll_top().item_ix;
        if self.emitted_scroll_top_index == top_index {
            return false;
        }

        self.emitted_scroll_top_index = top_index;
        cx.emit(TreeViewEvent::ScrollChanged { top_index });
        true
    }

    fn emit_focus_changed(&mut self, focused: bool, cx: &mut Context<Self>) -> bool {
        if self.emitted_focused == focused {
            return false;
        }

        self.emitted_focused = focused;
        cx.emit(TreeViewEvent::FocusChanged { focused });
        true
    }
}

impl<T> Focusable for TreeViewControl<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl<T> Render for TreeViewControl<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.model.enabled && self.focus_handle.contains_focused(window, cx) {
            window.blur(cx);
        }
        if self.focus_in_subscription.is_none() {
            let focus_handle = self.focus_handle.clone();
            self.focus_in_subscription = Some(cx.on_focus_in(&focus_handle, window, |_, window, cx| {
                cx.defer_in(window, |tree, window, cx| tree.handle_focus_in(window, cx));
            }));
        }
        if self.focus_out_subscription.is_none() {
            let focus_handle = self.focus_handle.clone();
            self.focus_out_subscription = Some(cx.on_focus_out(&focus_handle, window, |_, event, window, cx| {
                cx.defer_in(window, move |tree, window, cx| tree.handle_focus_out(event, window, cx));
            }));
        }

        let (was_animating, mut is_animating) = self.sync_expand_transitions();
        if is_animating && self.settle_large_expansions() {
            is_animating = false;
        }
        if self.desired_flat_count() != self.flat_cache.len() {
            self.rebuild_flat_cache(cx);
        } else if was_animating || is_animating {
            self.refresh_flat_motion_factors();
        }

        for transition in self.expand_transitions.values() {
            transition.schedule_frame(window, cx);
        }
        if was_animating || is_animating {
            cx.notify();
        }

        self.schedule_position_frame(window, cx);
        let render_model = TreeViewRenderModel {
            id: &self.model.id,
            selection_mode: self.model.selection_policy.mode,
            enabled: self.model.enabled,
            focus: ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window),
        };

        let body = self.render_viewport(window, cx);
        self.drop_layout.configure_transition(&self.model.motion.profile().drop_layout);
        self.drop_layout.set_motion_enabled(self.model.motion_enabled);
        let scroll = self.list_state.clone();
        let body = self.drop_layout.scope(body, move || scroll.scroll_px_offset_for_scrollbar());

        self.model
            .template
            .render(&render_model, body, window, cx)
            .role(gpui::Role::Tree)
            .cancel_drag_on_escape()
            .track_focus(&self.focus_handle)
            .key_context(if self.model.selection_policy.mode == TreeViewSelectionMode::Extended {
                ControlKeyProfile::TreeViewExtended.context()
            } else {
                ControlKeyProfile::TreeView.context()
            })
            .on_key_down(cx.listener(Self::handle_selection_key))
            .on_action(cx.listener(|this, event, window, cx| {
                if this.model.enabled && this.focus_handle.is_focused(window) {
                    this.handle_select_all(event, window, cx);
                } else {
                    cx.propagate();
                }
            }))
            .on_action(cx.listener(|this, event, window, cx| {
                if this.model.enabled && this.focus_handle.is_focused(window) {
                    this.handle_clear_selection(event, window, cx);
                } else {
                    cx.propagate();
                }
            }))
            .on_action(cx.listener(|this, event, window, cx| {
                if this.model.enabled && this.focus_handle.is_focused(window) {
                    this.handle_previous(event, window, cx);
                } else {
                    cx.propagate();
                }
            }))
            .on_action(cx.listener(|this, event, window, cx| {
                if this.model.enabled && this.focus_handle.is_focused(window) {
                    this.handle_next(event, window, cx);
                } else {
                    cx.propagate();
                }
            }))
            .on_action(cx.listener(|this, event, window, cx| {
                if this.model.enabled && this.focus_handle.is_focused(window) {
                    this.handle_first(event, window, cx);
                } else {
                    cx.propagate();
                }
            }))
            .on_action(cx.listener(|this, event, window, cx| {
                if this.model.enabled && this.focus_handle.is_focused(window) {
                    this.handle_last(event, window, cx);
                } else {
                    cx.propagate();
                }
            }))
            .on_action(cx.listener(|this, event, window, cx| {
                if this.model.enabled && this.focus_handle.is_focused(window) {
                    this.handle_expand_focused_node(event, window, cx);
                } else {
                    cx.propagate();
                }
            }))
            .on_action(cx.listener(|this, event, window, cx| {
                if this.model.enabled && this.focus_handle.is_focused(window) {
                    this.handle_collapse_focused_node(event, window, cx);
                } else {
                    cx.propagate();
                }
            }))
            .on_action(cx.listener(|this, event, window, cx| {
                if this.model.enabled && this.focus_handle.is_focused(window) {
                    this.handle_activate(event, window, cx);
                } else {
                    cx.propagate();
                }
            }))
            .on_mouse_down_out(cx.listener(|this, _, window, cx| {
                if this.focus_handle.contains_focused(window, cx) {
                    window.blur(cx);
                    this.emit_focus_changed(false, cx);
                    cx.notify();
                }
            }))
    }
}

fn first_enabled_flat_index<T>(flat: &[FlatNodeWrapper<T>]) -> Option<usize> {
    flat.iter().position(|node| node.enabled)
}

fn last_enabled_flat_index<T>(flat: &[FlatNodeWrapper<T>]) -> Option<usize> {
    flat.iter().rposition(|node| node.enabled)
}

fn next_enabled_flat_index<T>(
    flat: &[FlatNodeWrapper<T>],
    current_index: Option<usize>,
    direction: TreeDirection,
) -> Option<usize> {
    let len = flat.len();
    if len == 0 {
        return None;
    }

    let step = match direction {
        TreeDirection::Previous => len - 1,
        TreeDirection::Next => 1,
    };

    let mut index = match current_index {
        Some(idx) => (idx + step) % len,
        None if direction == TreeDirection::Previous => len - 1,
        None => 0,
    };

    for _ in 0..len {
        if flat[index].enabled {
            return Some(index);
        }
        index = (index + step) % len;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{TreeDirection, first_enabled_flat_index, next_enabled_flat_index};

    fn flat_nodes() -> Vec<super::FlatNodeWrapper<&'static str>> {
        vec![
            super::FlatNodeWrapper {
                id: "a".into(),
                label: "A".into(),
                icon: None,
                depth: 0,
                has_children: true,
                enabled: true,
                expand_progress: 1.0,
                row_height_factor: 1.0,
                data: "a",
            },
            super::FlatNodeWrapper {
                id: "b".into(),
                label: "B".into(),
                icon: None,
                depth: 1,
                has_children: false,
                enabled: false,
                expand_progress: 0.0,
                row_height_factor: 1.0,
                data: "b",
            },
            super::FlatNodeWrapper {
                id: "c".into(),
                label: "C".into(),
                icon: None,
                depth: 0,
                has_children: false,
                enabled: true,
                expand_progress: 0.0,
                row_height_factor: 1.0,
                data: "c",
            },
        ]
    }

    #[test]
    fn flat_navigation_skips_disabled_rows() {
        let flat = flat_nodes();

        assert_eq!(first_enabled_flat_index(&flat), Some(0));
        assert_eq!(next_enabled_flat_index(&flat, Some(0), TreeDirection::Next), Some(2));
        assert_eq!(next_enabled_flat_index(&flat, Some(2), TreeDirection::Next), Some(0));
    }
}

#[cfg(all(test, feature = "test-support"))]
#[path = "state_tests.rs"]
mod state_tests;

#[cfg(all(test, feature = "test-support"))]
#[path = "viewport_tests.rs"]
mod viewport_tests;

#[cfg(all(test, feature = "test-support"))]
#[path = "drag_tests.rs"]
mod drag_tests;

#[cfg(all(test, feature = "test-support"))]
#[path = "projection_tests.rs"]
mod projection_tests;

#[cfg(all(test, feature = "test-support"))]
#[path = "position_tests.rs"]
mod position_tests;

#[cfg(all(test, feature = "test-support"))]
#[path = "selection_tests.rs"]
mod selection_tests;

#[cfg(all(test, feature = "test-support"))]
#[path = "scale_tests.rs"]
mod scale_tests;

#[cfg(all(test, feature = "test-support"))]
#[test]
fn wheel_policy_dispatch_matrix() {
    crate::interaction_tests::matrix(
        |policy, cx| {
            TreeViewBuilder::new("matrix-tree")
                .items((0..100).map(|i| TreeNode::new(i.to_string(), i.to_string(), ())))
                .motion_enabled(false)
                .wheel_scroll_policy(policy.wheel)
                .scroll_boundary_policy(policy.boundary)
                .spawn(cx)
        },
        |view, _cx| -view.list_state.scroll_px_offset_for_scrollbar().y.as_f32(),
        |view, cx| {
            view.list_state.scroll_to(gpui::ListOffset { item_ix: 99, offset_in_item: px(0.0) });
            cx.notify();
        },
    );
}

#[cfg(all(test, feature = "test-support"))]
#[path = "perf_tests.rs"]
mod perf_tests;
