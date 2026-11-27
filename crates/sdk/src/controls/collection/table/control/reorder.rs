use std::{collections::HashSet, rc::Rc};

use gpui::{Context, EntityId, SharedString, Window};
use crate::infra::drag_drop::{DragDropEvent, DropProposal, KeyedDrag};
use super::{TableControl, TableEvent};

/// A synchronous owner decision for a same-table keyed insertion gap.
pub type TableRowDrop = DropProposal<EntityId, SharedString>;
/// Shared keyed drag lifecycle emitted through `TableEvent::RowDrag`.
pub type TableRowDragEvent = DragDropEvent<EntityId, SharedString>;
pub(super) type RowDrag = KeyedDrag<EntityId, SharedString>;
pub(super) type DropHandler<T> =
    Rc<dyn Fn(&mut TableControl<T>, &TableRowDrop, &mut Window, &mut Context<TableControl<T>>)>;

/// Invalid reordering configuration or rejected commit. Data is left unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TableReorderError {
    KeysNotConfigured,
    Disabled,
    InvalidSession,
    UnknownKey,
    DisabledRow,
}
impl std::fmt::Display for TableReorderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::KeysNotConfigured => "table row keys are not configured",
            Self::Disabled => "table row reordering is disabled",
            Self::InvalidSession => "row drag is stale or belongs to another table",
            Self::UnknownKey => "a dragged row or insertion gap no longer exists",
            Self::DisabledRow => "a dragged row is disabled",
        })
    }
}
impl std::error::Error for TableReorderError {}

/// Resolve a gap in original order, removing captured rows before inserting them.
/// A gap anchored on a dragged row can still change a discontiguous selection.
fn reordered_indices(
    keys: &[SharedString],
    moving: &[SharedString],
    before: Option<&SharedString>,
) -> Result<Vec<usize>, TableReorderError> {
    let moving: HashSet<_> = moving.iter().collect();
    let existing: HashSet<_> = keys.iter().collect();
    if moving.is_empty() || !moving.is_subset(&existing) {
        return Err(TableReorderError::UnknownKey);
    }
    let gap = match before {
        Some(key) => keys.iter().position(|candidate| candidate == key).ok_or(TableReorderError::UnknownKey)?,
        None => keys.len(),
    };
    let mut remaining = Vec::new();
    let mut captured = Vec::new();
    let mut insertion = 0;
    for (index, key) in keys.iter().enumerate() {
        if moving.contains(key) {
            captured.push(index);
        } else {
            if index < gap {
                insertion += 1;
            }
            remaining.push(index);
        }
    }
    remaining.splice(insertion..insertion, captured);
    Ok(remaining)
}

impl<T: 'static> TableControl<T> {
    /// Opt in after configuring unique row keys. Disable for externally sorted,
    /// filtered or grouped views unless their displayed order is authoritative.
    pub fn set_row_reordering(&mut self, enabled: bool, cx: &mut Context<Self>) -> Result<(), TableReorderError> {
        if enabled && self.row_key.is_none() {
            return Err(TableReorderError::KeysNotConfigured);
        }
        if !enabled {
            self.cancel_row_drag(cx);
        }
        self.row_reordering = enabled;
        cx.notify();
        Ok(())
    }

    /// Whether row reordering is opted in (the table must also be enabled).
    pub fn row_reordering(&self) -> bool {
        self.row_reordering
    }

    /// Current row identities in dataset order, including rows on other pages.
    pub fn row_keys(&self) -> &[SharedString] {
        &self.row_keys
    }

    /// Override automatic commits. The handler must synchronously call
    /// `commit_row_drop` or `reject_row_drop`; an unanswered proposal is rejected.
    /// Use events to persist the resulting order after a successful commit.
    pub fn set_row_drop_handler(
        &mut self,
        handler: impl Fn(&mut Self, &TableRowDrop, &mut Window, &mut Context<Self>) + 'static,
    ) {
        self.row_drop_handler = Some(Rc::new(handler));
    }

    pub(super) fn row_drag(&self, index: usize, scope: EntityId) -> Option<RowDrag> {
        if !self.row_reordering || !self.can_use_item(index) {
            return None;
        }
        let keys = if self.model.selected_indices.contains(&index) {
            self.selected_keys()
        } else {
            vec![self.row_keys.get(index)?.clone()]
        };
        RowDrag::new(scope, scope, keys)
    }

    fn drop_order(&self, proposal: &TableRowDrop, scope: EntityId) -> Result<Vec<usize>, TableReorderError> {
        if !self.row_reordering || !self.model.enabled {
            return Err(TableReorderError::Disabled);
        }
        if !proposal.is_active()
            || *proposal.source() != scope
            || *proposal.target() != scope
            || !self.active_row_drag.as_ref().is_some_and(|drag| proposal.is_from(drag) && drag.accepts(scope))
        {
            return Err(TableReorderError::InvalidSession);
        }
        let order = reordered_indices(&self.row_keys, proposal.keys(), proposal.before())?;
        let moving: HashSet<_> = proposal.keys().iter().collect();
        if self
            .row_keys
            .iter()
            .enumerate()
            .any(|(index, key)| moving.contains(key) && !self.can_use_item(index))
        {
            return Err(TableReorderError::DisabledRow);
        }
        Ok(order)
    }

    pub(super) fn drop_changes_order(&self, proposal: &TableRowDrop, scope: EntityId) -> bool {
        self.drop_order(proposal, scope).is_ok_and(|order| order.iter().enumerate().any(|(a, b)| a != *b))
    }

    /// Validate again and move rows atomically. Preserves selection, active row,
    /// range anchor, focus and viewport. Returns false for a valid unchanged drop.
    /// Failed validation rejects the proposal without changing the collection.
    pub fn commit_row_drop(
        &mut self,
        proposal: &TableRowDrop,
        cx: &mut Context<Self>,
    ) -> Result<bool, TableReorderError> {
        let order = match self.drop_order(proposal, cx.entity_id()) {
            Ok(order) => order,
            Err(error) => {
                self.reject_row_drop(proposal, error.to_string(), cx);
                return Err(error);
            }
        };
        let changed = order.iter().enumerate().any(|(a, b)| a != *b);
        let moving: HashSet<_> = proposal.keys().iter().collect();
        let moved = self.row_keys.iter().filter(|key| moving.contains(key)).cloned().collect();
        if changed {
            self.drop_layout.start();
            let previous = self.selection_snapshot();
            let keys = order.iter().map(|&index| self.row_keys[index].clone()).collect::<Vec<_>>();
            self.remap_selection(&keys);
            self.model.selected_indices.sort_unstable();
            // Move owned values without requiring row data to implement Clone.
            let mut items: Vec<_> = std::mem::take(&mut self.model.items).into_iter().map(Some).collect();
            self.model.items = order.into_iter().filter_map(|index| items[index].take()).collect();
            self.row_keys = keys;
            self.clear_pointer_interaction(cx);
            self.list_state.remeasure();
            self.emit_selection_changes(previous, cx);
        }
        for event in proposal.committed(changed, moved) {
            self.handle_row_drag_event(event, cx);
        }
        cx.notify();
        Ok(changed)
    }

    /// Reject a proposal from the current session without changing row order.
    pub fn reject_row_drop(
        &mut self,
        proposal: &TableRowDrop,
        reason: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) {
        if !self.active_row_drag.as_ref().is_some_and(|drag| proposal.is_from(drag)) {
            return;
        }
        for event in proposal.rejected(reason) {
            self.handle_row_drag_event(event, cx);
        }
    }

    pub(super) fn handle_row_drop(&mut self, proposal: TableRowDrop, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(handler) = self.row_drop_handler.clone() {
            handler(self, &proposal, window, cx);
            if proposal.is_active() {
                self.reject_row_drop(&proposal, "owner did not accept the drop", cx);
            }
        } else {
            let _ = self.commit_row_drop(&proposal, cx);
        }
    }

    pub(super) fn handle_row_drag_event(&mut self, event: TableRowDragEvent, cx: &mut Context<Self>) {
        if matches!(event, DragDropEvent::DragEnded { .. }) {
            self.active_row_drag = None;
            let changes = self.rows_pointer.release();
            self.emit_row_changes(changes, cx);
        }
        cx.emit(TableEvent::RowDrag(event));
        cx.notify();
    }

    pub(super) fn cancel_row_drag(&mut self, cx: &mut Context<Self>) {
        if let Some(event) = self.active_row_drag.take().and_then(|drag| drag.cancel()) {
            self.handle_row_drag_event(event, cx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gaps_preserve_source_order_and_detect_real_noops() {
        let keys: Vec<SharedString> = ["a", "b", "c", "d", "e"].into_iter().map(Into::into).collect();
        let order = |moving: &[usize], before: Option<usize>| {
            reordered_indices(
                &keys,
                &moving.iter().map(|&i| keys[i].clone()).collect::<Vec<_>>(),
                before.map(|i| &keys[i]),
            )
            .unwrap()
        };
        assert_eq!(order(&[1], Some(0)), [1, 0, 2, 3, 4]);
        assert_eq!(order(&[1], None), [0, 2, 3, 4, 1]);
        assert_eq!(order(&[1], Some(1)), [0, 1, 2, 3, 4]);
        assert_eq!(order(&[1], Some(2)), [0, 1, 2, 3, 4]);
        assert_eq!(order(&[3, 1], Some(3)), [0, 2, 1, 3, 4]);
        assert_eq!(order(&[1, 2], Some(2)), [0, 1, 2, 3, 4]);
        assert_eq!(order(&[0, 1, 2, 3, 4], None), [0, 1, 2, 3, 4]);
        assert_eq!(reordered_indices(&[], &["a".into()], None), Err(TableReorderError::UnknownKey));
        assert_eq!(
            reordered_indices(&keys, &["a".into()], Some(&"missing".into())),
            Err(TableReorderError::UnknownKey)
        );
    }
}
