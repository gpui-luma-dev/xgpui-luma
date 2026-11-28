use super::*;
use std::{cmp::Ordering, rc::Rc};

impl<T: Clone + Send + Sync + 'static> TreeViewControl<T> {
    /// Show loaded matches plus ancestor paths, without changing selection or
    /// saved expansion. Filter-required ancestors stay open until the filter
    /// changes; expansion commands on those ancestors leave saved choices alone.
    /// No data is fetched. Reapply to reevaluate a predicate's captured state.
    pub fn set_filter(&mut self, predicate: impl Fn(&TreeNode<T>) -> bool + 'static, cx: &mut Context<Self>) {
        self.model.filter = Some(Rc::new(predicate));
        self.refresh_projection(cx);
    }

    /// Restore unfiltered visibility and saved user expansion.
    pub fn clear_filter(&mut self, cx: &mut Context<Self>) {
        if self.model.filter.take().is_some() {
            self.refresh_projection(cx);
        }
    }

    /// Stable sibling sorting for display only. Ties retain source order.
    /// Reapply to reevaluate a comparator's captured state.
    pub fn set_sort(
        &mut self,
        compare: impl Fn(&TreeNode<T>, &TreeNode<T>) -> Ordering + 'static,
        cx: &mut Context<Self>,
    ) {
        self.model.sort = Some(Rc::new(compare));
        self.refresh_projection(cx);
    }

    /// Restore source sibling order without changing filtering or selection.
    pub fn clear_sort(&mut self, cx: &mut Context<Self>) {
        if self.model.sort.take().is_some() {
            self.refresh_projection(cx);
        }
    }

    /// Before/after drop ordering is ambiguous under a filter or custom sort.
    pub fn has_projection(&self) -> bool {
        self.model.filter.is_some() || self.model.sort.is_some()
    }

    /// Effective displayed preorder, excluding descendants still animating closed.
    pub fn visible_ids(&self) -> Vec<SharedString> {
        self.projection.visible(&self.model.items, &self.expanded_ids)
    }

    fn refresh_projection(&mut self, cx: &mut Context<Self>) {
        self.cancel_position();
        let old_visible = self.visible_ids();
        let active_before = self.active_node_id.clone();
        self.clear_row_pointer(cx);
        self.projection = super::super::projection::Projection::new(
            &self.model.items,
            self.model.filter.as_ref(),
            self.model.sort.as_ref(),
        );
        self.revision = self.revision.wrapping_add(1);
        self.expand_transitions.clear();
        self.sync_expand_transitions_for_tree();
        self.selection_anchor = self.model.index.reconcile_active(
            &self.model.index,
            self.selection_anchor.as_ref(),
            &old_visible,
            &self.visible_ids(),
        );
        self.active_node_id = self.model.index.reconcile_active(
            &self.model.index,
            active_before.as_ref(),
            &old_visible,
            &self.visible_ids(),
        );
        self.rebuild_flat_cache(cx);
        self.list_state.remeasure();
        if active_before != self.active_node_id {
            cx.emit(TreeViewEvent::ActiveNodeChanged { node_id: self.active_node_id.clone() });
        }
        self.emit_scroll_changed_if_needed(cx);
    }

    pub(super) fn visit_projected_rows(&self, mut visitor: impl FnMut(&TreeNode<T>, usize, f32, f32, bool)) {
        let mut ancestors = vec![(1.0_f32, true)];
        self.projection.visit(&self.model.items, |node, depth| {
            let (height, visible) = ancestors[depth];
            let progress = self.expand_progress_for(&node.id);
            visitor(node, depth, progress, height, visible && node.enabled);
            ancestors.truncate(depth + 1);
            ancestors.push(((height * progress).clamp(0.0, 1.0), visible && self.is_expanded(&node.id)));
            self.branch_children_visible(&node.id)
        });
    }
}
