//! Host-owned transfers and reordering. A drag captures keys; data moves only on drop.

use std::collections::HashSet;

use gpui::SharedString;
use xgpui_luma::controls::listbox::{ListBoxError, ListBoxSnapshot, ListBoxState, ListBoxUpdate, SelectionMode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Side {
    Left,
    Right,
}

impl Side {
    pub const ALL: [Self; 2] = [Self::Left, Self::Right];
    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone)]
pub(super) struct TransferItem {
    pub id: u32,
    pub label: SharedString,
}

pub(super) struct TransferModel {
    pub lists: [ListBoxState<TransferItem, u32>; 2],
}

impl TransferModel {
    pub fn new() -> Self {
        Self {
            lists: Side::ALL.map(|side| {
                let first = side.index() as u32 * 10 + 1;
                let items = (first..first + 10).map(|id| TransferItem { id, label: format!("Item {id}").into() });
                ListBoxState::try_new(items, |item| item.id, SelectionMode::Multiple)
                    .expect("transfer example items have unique keys")
            }),
        }
    }

    /// Insert source items in their list order before a destination key, or append for `None`.
    /// Validate both snapshots before committing either. Keyed anchors remain
    /// correct after scrolling or changes earlier in the destination list.
    /// Same-list moves preserve selection and adjust the gap for removed rows.
    pub fn move_items(
        &mut self,
        source: Side,
        target: Side,
        keys: &[u32],
        before: Option<u32>,
    ) -> Result<[ListBoxUpdate<u32>; 2], ListBoxError> {
        let from = source.index();
        let to = target.index();
        let mut updates = std::array::from_fn(|_| ListBoxUpdate { changed: false, events: Vec::new(), reveal: None });
        if keys.is_empty() {
            return Ok(updates);
        }
        let moving: HashSet<u32> = keys.iter().copied().collect();
        if moving.len() != keys.len() {
            return Err(ListBoxError::DuplicateKey);
        }
        let source_items = self.lists[from].snapshot().items();
        let moved_items: Vec<_> = source_items.iter().filter(|item| moving.contains(&item.id)).cloned().collect();
        if moved_items.len() != keys.len() {
            return Err(ListBoxError::UnknownKey);
        }
        let first = moved_items[0].id;
        let mut target_items = self.lists[to].snapshot().items().to_vec();
        let mut insertion_index = match before {
            Some(anchor) => target_items.iter().position(|item| item.id == anchor).ok_or(ListBoxError::UnknownKey)?,
            None => target_items.len(),
        };
        if from == to {
            // Resolve the gap in the original list, then remove the moving rows.
            // This also handles anchors inside the dragged group without losing them.
            insertion_index = target_items[..insertion_index].iter().filter(|item| !moving.contains(&item.id)).count();
            target_items.retain(|item| !moving.contains(&item.id));
        }
        target_items.splice(insertion_index..insertion_index, moved_items);
        if from == to {
            if source_items.iter().map(|item| item.id).eq(target_items.iter().map(|item| item.id)) {
                return Ok(updates);
            }
            let snapshot = ListBoxSnapshot::try_new(target_items, |item| item.id)?;
            updates[to] = self.lists[to].replace_snapshot(snapshot);
            updates[to].reveal = Some(first);
            return Ok(updates);
        }
        let source_snapshot =
            ListBoxSnapshot::try_new(source_items.iter().filter(|item| !moving.contains(&item.id)).cloned(), |item| {
                item.id
            })?;
        let target_snapshot = ListBoxSnapshot::try_new(target_items, |item| item.id)?;

        // Validate and commit the fallible destination operation first. Source
        // replacement cannot fail; both states are final before events are returned.
        let mut target_update =
            self.lists[to].replace_snapshot_with_selection(target_snapshot, keys.iter().copied(), Some(first))?;
        let source_update = self.lists[from].replace_snapshot(source_snapshot);
        target_update.reveal = Some(first);
        updates[from] = source_update;
        updates[to] = target_update;
        Ok(updates)
    }
}

#[cfg(test)]
mod tests {
    use xgpui_luma::controls::listbox::{ListBoxEvent, ListBoxInput};
    use super::*;

    fn keys(model: &TransferModel, side: Side) -> Vec<u32> {
        model.lists[side.index()].snapshot().items().iter().map(|item| item.id).collect()
    }

    #[test]
    fn each_list_starts_with_ten_unique_items() {
        let model = TransferModel::new();
        assert_eq!(keys(&model, Side::Left), (1..=10).collect::<Vec<_>>());
        assert_eq!(keys(&model, Side::Right), (11..=20).collect::<Vec<_>>());
    }

    #[test]
    fn moves_one_item_in_both_directions_and_reconciles_selection() {
        let mut model = TransferModel::new();
        model.lists[0].apply(ListBoxInput::Select(3));
        model.lists[1].apply(ListBoxInput::Select(12));
        let updates = model.move_items(Side::Left, Side::Right, &[3], None).unwrap();
        assert_eq!(keys(&model, Side::Left), vec![1, 2, 4, 5, 6, 7, 8, 9, 10]);
        assert_eq!(keys(&model, Side::Right), (11..=20).chain([3]).collect::<Vec<_>>());
        assert_eq!(model.lists[0].selected_key(), None);
        assert_eq!(model.lists[1].selected_key(), Some(&3));
        assert_eq!(model.lists[1].active_key(), Some(&3));
        assert_eq!(updates[1].reveal, Some(3));
        model.move_items(Side::Right, Side::Left, &[3], None).unwrap();
        assert_eq!(keys(&model, Side::Left), vec![1, 2, 4, 5, 6, 7, 8, 9, 10, 3]);
        assert_eq!(keys(&model, Side::Right), (11..=20).collect::<Vec<_>>());
    }

    #[test]
    fn dragging_an_unselected_item_does_not_move_the_selected_item() {
        let mut model = TransferModel::new();
        model.lists[0].apply(ListBoxInput::Select(1));
        let payload = model.lists[Side::Left.index()].drag_keys(&5).unwrap();
        assert_eq!(payload, vec![5]);
        model.move_items(Side::Left, Side::Right, &payload, None).unwrap();
        assert_eq!(model.lists[0].selected_key(), Some(&1));
        assert!(model.lists[0].snapshot().item(&1).is_some());
        assert!(model.lists[0].snapshot().item(&5).is_none());
        assert_eq!(model.lists[1].selected_key(), Some(&5));
    }

    #[test]
    fn rejects_stale_or_duplicate_drops_without_mutating_either_list() {
        let mut model = TransferModel::new();
        let before = [keys(&model, Side::Left), keys(&model, Side::Right)];
        assert_eq!(model.move_items(Side::Left, Side::Right, &[99], None).unwrap_err(), ListBoxError::UnknownKey);
        assert_eq!(model.move_items(Side::Left, Side::Right, &[1], Some(99)).unwrap_err(), ListBoxError::UnknownKey);
        assert_eq!([keys(&model, Side::Left), keys(&model, Side::Right)], before);
        let duplicate = model.lists[0].snapshot().item(&1).unwrap().clone();
        let snapshot = ListBoxSnapshot::try_new([duplicate], |item| item.id).unwrap();
        model.lists[1].replace_snapshot(snapshot);
        assert_eq!(model.move_items(Side::Left, Side::Right, &[1], None).unwrap_err(), ListBoxError::DuplicateKey);
        assert_eq!(keys(&model, Side::Left), before[0]);
        assert_eq!(keys(&model, Side::Right), vec![1]);
    }

    #[test]
    fn empty_list_accepts_a_returned_item_and_total_membership_is_preserved() {
        let mut model = TransferModel::new();
        model.lists[0].apply(ListBoxInput::SelectAll);
        let payload = model.lists[Side::Left.index()].drag_keys(&5).unwrap();
        assert_eq!(payload.len(), 10);
        model.move_items(Side::Left, Side::Right, &payload, None).unwrap();
        assert!(keys(&model, Side::Left).is_empty());
        assert_eq!(keys(&model, Side::Right).len(), 20);
        let payload = model.lists[Side::Right.index()].drag_keys(&10).unwrap();
        model.move_items(Side::Right, Side::Left, &payload, None).unwrap();
        assert_eq!(keys(&model, Side::Left), (1..=10).collect::<Vec<_>>());
        let mut all = keys(&model, Side::Left);
        all.extend(keys(&model, Side::Right));
        all.sort_unstable();
        assert_eq!(all, (1..=20).collect::<Vec<_>>());
    }

    #[test]
    fn inserts_at_start_and_between_items_in_both_directions() {
        let mut model = TransferModel::new();
        model.move_items(Side::Left, Side::Right, &[3], Some(11)).unwrap();
        let updates = model.move_items(Side::Left, Side::Right, &[5], Some(14)).unwrap();
        assert_eq!(keys(&model, Side::Right), vec![3, 11, 12, 13, 5, 14, 15, 16, 17, 18, 19, 20]);
        assert_eq!(model.lists[1].selected_key(), Some(&5));
        assert_eq!(updates[1].reveal, Some(5));
        model.move_items(Side::Right, Side::Left, &[20], Some(1)).unwrap();
        model.move_items(Side::Right, Side::Left, &[3], Some(4)).unwrap();
        assert_eq!(keys(&model, Side::Left), vec![20, 1, 2, 3, 4, 6, 7, 8, 9, 10]);
        assert_eq!(keys(&model, Side::Right), vec![11, 12, 13, 5, 14, 15, 16, 17, 18, 19]);
    }

    #[test]
    fn selected_drag_captures_list_order_without_mutating_selection_or_data() {
        let mut model = TransferModel::new();
        for key in [7, 2, 4] {
            model.lists[0].apply(ListBoxInput::Select(key));
        }
        let payload = model.lists[Side::Left.index()].drag_keys(&4).unwrap();
        assert_eq!(payload, vec![2, 4, 7]);
        assert_eq!(model.lists[0].selected_keys().copied().collect::<Vec<_>>(), payload);
        assert_eq!(keys(&model, Side::Left), (1..=10).collect::<Vec<_>>());
        model.lists[0].apply(ListBoxInput::Select(1));
        // Selection changes after drag start do not change the captured group.
        model.move_items(Side::Left, Side::Right, &payload, Some(11)).unwrap();
        assert_eq!(model.lists[0].selected_keys().copied().collect::<Vec<_>>(), vec![1]);
        assert_eq!(model.lists[1].selected_keys().copied().collect::<Vec<_>>(), payload);
    }

    #[test]
    fn group_moves_preserve_source_order_in_both_directions() {
        let mut model = TransferModel::new();
        model.lists[0].set_selected_keys([1, 2, 5, 7]).unwrap();
        model.lists[1].apply(ListBoxInput::Select(12));
        let updates = model.move_items(Side::Left, Side::Right, &[7, 2, 5], Some(14)).unwrap();
        assert_eq!(keys(&model, Side::Left), vec![1, 3, 4, 6, 8, 9, 10]);
        assert_eq!(keys(&model, Side::Right), vec![11, 12, 13, 2, 5, 7, 14, 15, 16, 17, 18, 19, 20]);
        assert_eq!(model.lists[0].selected_keys().copied().collect::<Vec<_>>(), vec![1]);
        assert_eq!(model.lists[1].selected_keys().copied().collect::<Vec<_>>(), vec![2, 5, 7]);
        assert_eq!(model.lists[1].active_key(), Some(&2));
        assert_eq!(updates[1].reveal, Some(2));
        let updates = model.move_items(Side::Right, Side::Left, &[5, 7, 2], Some(3)).unwrap();
        assert_eq!(keys(&model, Side::Left), vec![1, 2, 5, 7, 3, 4, 6, 8, 9, 10]);
        assert_eq!(keys(&model, Side::Right), (11..=20).collect::<Vec<_>>());
        assert_eq!(model.lists[0].selected_keys().copied().collect::<Vec<_>>(), vec![2, 5, 7]);
        assert_eq!(model.lists[0].active_key(), Some(&2));
        assert_eq!(updates[0].reveal, Some(2));
    }

    #[test]
    fn cross_list_drop_emits_each_final_change_once_in_both_directions() {
        for (source, target) in [(Side::Left, Side::Right), (Side::Right, Side::Left)] {
            let mut model = TransferModel::new();
            let from = source.index();
            let to = target.index();
            let offset = from as u32 * 10;
            let target_offset = to as u32 * 10;
            for key in [offset + 2, offset + 5] {
                model.lists[from].apply(ListBoxInput::Select(key));
            }
            model.lists[to].apply(ListBoxInput::Select(target_offset + 3));
            let updates = model.move_items(source, target, &[offset + 5, offset + 2], Some(target_offset + 4)).unwrap();
            assert_eq!(
                updates[from].events,
                vec![
                    ListBoxEvent::ProjectionChanged { visible_count: 8 },
                    ListBoxEvent::SelectionChanged { selected: vec![], active: Some(offset + 1) },
                    ListBoxEvent::ActiveItemChanged { active: Some(offset + 1) },
                ]
            );
            assert_eq!(
                updates[to].events,
                vec![
                    ListBoxEvent::ProjectionChanged { visible_count: 12 },
                    ListBoxEvent::SelectionChanged { selected: vec![offset + 2, offset + 5], active: Some(offset + 2) },
                    ListBoxEvent::ActiveItemChanged { active: Some(offset + 2) },
                ]
            );
        }
    }

    #[test]
    fn empty_destination_reports_no_intermediate_active_or_selection_events() {
        let mut model = TransferModel::new();
        model.lists[1].replace_snapshot(ListBoxSnapshot::try_new([], |item: &TransferItem| item.id).unwrap());
        let updates = model.move_items(Side::Left, Side::Right, &[5, 2], None).unwrap();
        assert_eq!(updates[0].events, vec![ListBoxEvent::ProjectionChanged { visible_count: 8 }]);
        assert_eq!(
            updates[1].events,
            vec![
                ListBoxEvent::ProjectionChanged { visible_count: 2 },
                ListBoxEvent::SelectionChanged { selected: vec![2, 5], active: Some(2) },
                ListBoxEvent::ActiveItemChanged { active: Some(2) },
            ]
        );
    }

    #[test]
    fn destination_selection_failure_leaves_both_collections_unchanged() {
        let mut model = TransferModel::new();
        // Exercise the fallible SDK commit, beyond snapshot and key validation.
        model.lists[1] = ListBoxState::try_new(
            model.lists[1].snapshot().items().to_vec(),
            |item| item.id,
            SelectionMode::SingleRequired,
        )
        .unwrap();
        let before = [keys(&model, Side::Left), keys(&model, Side::Right)];
        assert_eq!(
            model.move_items(Side::Left, Side::Right, &[2, 5], None).unwrap_err(),
            ListBoxError::TooManySelected
        );
        assert_eq!([keys(&model, Side::Left), keys(&model, Side::Right)], before);
        assert_eq!(model.lists[1].selected_key(), Some(&11));
        assert_eq!(model.lists[1].active_key(), Some(&11));
    }

    #[test]
    fn invalid_group_rejects_entire_move_and_preserves_selection() {
        let mut model = TransferModel::new();
        model.lists[0].set_selected_keys([1, 2]).unwrap();
        model.lists[1].apply(ListBoxInput::Select(12));
        let before = [keys(&model, Side::Left), keys(&model, Side::Right)];
        for (payload, anchor, error) in [
            (vec![2, 99], None, ListBoxError::UnknownKey),
            (vec![2, 2], None, ListBoxError::DuplicateKey),
            (vec![1, 2], Some(99), ListBoxError::UnknownKey),
        ] {
            assert_eq!(model.move_items(Side::Left, Side::Right, &payload, anchor).unwrap_err(), error);
            assert_eq!([keys(&model, Side::Left), keys(&model, Side::Right)], before);
            assert_eq!(model.lists[0].selected_keys().copied().collect::<Vec<_>>(), vec![1, 2]);
            assert_eq!(model.lists[1].selected_keys().copied().collect::<Vec<_>>(), vec![12]);
        }
        let updates = model.move_items(Side::Left, Side::Right, &[], None).unwrap();
        assert!(updates.iter().all(|update| !update.changed && update.events.is_empty() && update.reveal.is_none()));
        let duplicate = model.lists[0].snapshot().item(&2).unwrap().clone();
        model.lists[1].replace_snapshot(ListBoxSnapshot::try_new([duplicate], |item| item.id).unwrap());
        assert_eq!(model.move_items(Side::Left, Side::Right, &[1, 2], None).unwrap_err(), ListBoxError::DuplicateKey);
        assert_eq!(keys(&model, Side::Left), before[0]);
        assert_eq!(keys(&model, Side::Right), vec![2]);
        assert_eq!(model.lists[0].selected_keys().copied().collect::<Vec<_>>(), vec![1, 2]);
    }

    #[test]
    fn single_rows_reorder_up_down_and_to_end_in_either_list() {
        for side in Side::ALL {
            let mut model = TransferModel::new();
            let index = side.index();
            let offset = index as u32 * 10;
            let other = 1 - index;
            let untouched: Vec<_> = model.lists[other].snapshot().items().iter().map(|item| item.id).collect();
            model.lists[index].apply(ListBoxInput::Select(offset + 1));
            for (anchor, expected) in [
                (Some(2), vec![1, 4, 2, 3, 5, 6, 7, 8, 9, 10]),
                (Some(8), vec![1, 2, 3, 5, 6, 7, 4, 8, 9, 10]),
                (None, vec![1, 2, 3, 5, 6, 7, 8, 9, 10, 4]),
            ] {
                let updates = model.move_items(side, side, &[offset + 4], anchor.map(|key| key + offset)).unwrap();
                assert_eq!(keys(&model, side), expected.into_iter().map(|key| key + offset).collect::<Vec<_>>());
                assert_eq!(model.lists[index].selected_keys().copied().collect::<Vec<_>>(), vec![offset + 1]);
                assert_eq!(model.lists[index].active_key(), Some(&(offset + 1)));
                assert_eq!(updates[index].reveal, Some(offset + 4));
                assert!(!updates[other].changed);
                assert_eq!(
                    model.lists[other].snapshot().items().iter().map(|item| item.id).collect::<Vec<_>>(),
                    untouched
                );
            }
        }
    }

    #[test]
    fn discontiguous_group_reorders_at_gaps_including_its_own_anchors() {
        for (anchor, expected) in [
            (Some(1), vec![2, 5, 7, 1, 3, 4, 6, 8, 9, 10]),
            (Some(5), vec![1, 3, 4, 2, 5, 7, 6, 8, 9, 10]),
            (Some(6), vec![1, 3, 4, 2, 5, 7, 6, 8, 9, 10]),
            (Some(9), vec![1, 3, 4, 6, 8, 2, 5, 7, 9, 10]),
            (None, vec![1, 3, 4, 6, 8, 9, 10, 2, 5, 7]),
        ] {
            let mut model = TransferModel::new();
            for key in [2, 5, 7] {
                model.lists[0].apply(ListBoxInput::Select(key));
            }
            let updates = model.move_items(Side::Left, Side::Left, &[7, 2, 5], anchor).unwrap();
            assert_eq!(keys(&model, Side::Left), expected);
            assert_eq!(model.lists[0].selected_keys().copied().collect::<Vec<_>>(), vec![2, 5, 7]);
            assert_eq!(model.lists[0].active_key(), Some(&7));
            assert_eq!(updates[0].reveal, Some(2));
            assert!(!updates[1].changed);
        }
    }

    #[test]
    fn same_list_self_drops_and_invalid_drops_leave_state_unchanged() {
        let mut model = TransferModel::new();
        model.lists[0].set_selected_keys([2, 3, 4]).unwrap();
        let original = keys(&model, Side::Left);
        for (payload, anchor) in [
            (vec![3], Some(3)),
            (vec![3], Some(4)),
            (vec![2, 3, 4], Some(2)),
            (vec![2, 3, 4], Some(3)),
            (vec![2, 3, 4], Some(5)),
            ((1..=10).collect(), Some(6)),
            ((1..=10).collect(), None),
        ] {
            let updates = model.move_items(Side::Left, Side::Left, &payload, anchor).unwrap();
            assert!(
                updates.iter().all(|update| !update.changed && update.events.is_empty() && update.reveal.is_none())
            );
        }
        for (payload, anchor, error) in [
            (vec![2, 99], None, ListBoxError::UnknownKey),
            (vec![2, 2], None, ListBoxError::DuplicateKey),
            (vec![2, 3], Some(99), ListBoxError::UnknownKey),
        ] {
            assert_eq!(model.move_items(Side::Left, Side::Left, &payload, anchor).unwrap_err(), error);
        }
        assert_eq!(keys(&model, Side::Left), original);
        assert_eq!(model.lists[0].selected_keys().copied().collect::<Vec<_>>(), vec![2, 3, 4]);
        assert_eq!(model.lists[0].active_key(), Some(&1));
    }
}
