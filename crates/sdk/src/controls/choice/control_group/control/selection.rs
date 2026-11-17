use gpui::SharedString;

use super::super::model::{ControlGroupItemLike, ControlGroupModel, ControlSelectionMode};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ControlGroupDirection {
    Previous,
    Next,
}
pub(crate) fn normalize_model<T>(model: &mut ControlGroupModel<T>)
where
    T: ControlGroupItemLike + 'static,
{
    normalize_selected_ids(model);
    normalize_active_id(model);
}

pub(crate) fn normalize_selected_ids<T>(model: &mut ControlGroupModel<T>)
where
    T: ControlGroupItemLike + 'static,
{
    model.default_selected_ids =
        normalize_selected_id_list(&model.items, &model.default_selected_ids, model.selection_mode);

    if let Some(managed_selected_ids) = model.managed_selected_ids.clone() {
        model.managed_selected_ids =
            Some(normalize_selected_id_list(&model.items, &managed_selected_ids, model.selection_mode));
    }
}

pub(crate) fn normalize_active_id<T>(model: &mut ControlGroupModel<T>)
where
    T: ControlGroupItemLike + 'static,
{
    if enabled_item_index_by_id(&model.items, model.active_id.as_ref()).is_some() {
        return;
    }

    model.active_id = model
        .effective_selected_ids()
        .iter()
        .find(|selected_id| enabled_item_index_by_id(&model.items, Some(selected_id)).is_some())
        .cloned()
        .or_else(|| model.items.iter().find(|item| item.is_enabled()).map(|item| item.id().clone()));
}

pub(super) fn normalize_selected_id_list<T>(
    items: &[T],
    selected_ids: &[SharedString],
    selection_mode: ControlSelectionMode,
) -> Vec<SharedString>
where
    T: ControlGroupItemLike + 'static,
{
    let mut normalized = Vec::new();

    for selected_id in selected_ids {
        if normalized.iter().any(|id| id == selected_id) {
            continue;
        }

        if items.iter().any(|item| item.is_enabled() && item.id() == selected_id) {
            normalized.push(selected_id.clone());
        }

        if matches!(selection_mode, ControlSelectionMode::SingleRequired | ControlSelectionMode::SingleAllowNone)
            && !normalized.is_empty()
        {
            break;
        }
    }

    if selection_mode == ControlSelectionMode::SingleRequired
        && normalized.is_empty()
        && let Some(first_enabled_id) = items.iter().find(|item| item.is_enabled()).map(|item| item.id().clone())
    {
        normalized.push(first_enabled_id);
    }

    normalized
}

pub(crate) fn enabled_item_index_by_id<T>(items: &[T], item_id: Option<&SharedString>) -> Option<usize>
where
    T: ControlGroupItemLike + 'static,
{
    let item_id = item_id?;
    items.iter().position(|item| item.is_enabled() && item.id() == item_id)
}

pub(crate) fn first_enabled_index<T>(items: &[T]) -> Option<usize>
where
    T: ControlGroupItemLike + 'static,
{
    items.iter().position(ControlGroupItemLike::is_enabled)
}

pub(crate) fn last_enabled_index<T>(items: &[T]) -> Option<usize>
where
    T: ControlGroupItemLike + 'static,
{
    items.iter().rposition(ControlGroupItemLike::is_enabled)
}

pub(crate) fn next_enabled_index_no_wrap<T>(
    items: &[T],
    current_index: Option<usize>,
    direction: ControlGroupDirection,
) -> Option<usize>
where
    T: ControlGroupItemLike + 'static,
{
    let len = items.len();
    if len == 0 {
        return None;
    }

    let current = match current_index {
        Some(index) => index,
        None => {
            return if direction == ControlGroupDirection::Next {
                items.iter().position(ControlGroupItemLike::is_enabled)
            } else {
                items.iter().rposition(ControlGroupItemLike::is_enabled)
            };
        }
    };

    match direction {
        ControlGroupDirection::Next => {
            for (i, item) in items.iter().enumerate().take(len).skip(current + 1) {
                if item.is_enabled() {
                    return Some(i);
                }
            }
        }
        ControlGroupDirection::Previous => {
            for i in (0..current).rev() {
                if items[i].is_enabled() {
                    return Some(i);
                }
            }
        }
    }
    None
}

pub(crate) fn next_enabled_index<T>(
    items: &[T],
    current_index: Option<usize>,
    direction: ControlGroupDirection,
) -> Option<usize>
where
    T: ControlGroupItemLike + 'static,
{
    let len = items.len();
    if len == 0 {
        return None;
    }

    let step = match direction {
        ControlGroupDirection::Previous => len - 1,
        ControlGroupDirection::Next => 1,
    };

    let mut index = match current_index {
        Some(index) => (index + step) % len,
        None if direction == ControlGroupDirection::Previous => len - 1,
        None => 0,
    };

    for _ in 0..len {
        if items[index].is_enabled() {
            return Some(index);
        }
        index = (index + step) % len;
    }

    None
}

pub(super) fn selected_ids_contain(selected_ids: &[SharedString], item_id: &SharedString) -> bool {
    selected_ids.iter().any(|selected_id| selected_id == item_id)
}

pub(super) fn compute_next_selected_ids<T>(
    items: &[T],
    current: &[SharedString],
    selection_mode: ControlSelectionMode,
    toggled_index: usize,
) -> Vec<SharedString>
where
    T: ControlGroupItemLike + 'static,
{
    let Some(item) = items.get(toggled_index) else {
        return current.to_vec();
    };
    if !item.is_enabled() {
        return current.to_vec();
    }

    match selection_mode {
        ControlSelectionMode::SingleRequired => vec![item.id().clone()],
        ControlSelectionMode::SingleAllowNone => {
            if selected_ids_contain(current, item.id()) {
                Vec::new()
            } else {
                vec![item.id().clone()]
            }
        }
        ControlSelectionMode::Multiple => {
            if selected_ids_contain(current, item.id()) {
                current.iter().filter(|selected_id| *selected_id != item.id()).cloned().collect()
            } else {
                let mut next = current.to_vec();
                next.push(item.id().clone());
                normalize_selected_id_list(items, &next, ControlSelectionMode::Multiple)
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use gpui::SharedString;

    use super::{
        ControlGroupDirection, ControlGroupModel, compute_next_selected_ids, enabled_item_index_by_id,
        next_enabled_index, normalize_active_id, normalize_selected_ids,
    };
    use crate::controls::control_group::{
        ControlGroupFocusStrategy, ControlGroupItem, ControlGroupLayout, ControlGroupStateMode, ControlSelectionMode,
        default_control_group_template,
    };

    #[test]
    fn normalize_single_required_falls_back_to_first_enabled_item() {
        let mut model = model_with_selected(
            ControlSelectionMode::SingleRequired,
            ControlGroupStateMode::Unmanaged,
            vec!["missing".into()],
        );

        normalize_selected_ids(&mut model);

        assert_eq!(model.default_selected_ids, vec![SharedString::from("compact")]);
    }

    #[test]
    fn normalize_single_allow_none_keeps_empty_selection() {
        let mut model =
            model_with_selected(ControlSelectionMode::SingleAllowNone, ControlGroupStateMode::Unmanaged, Vec::new());

        normalize_selected_ids(&mut model);

        assert!(model.default_selected_ids.is_empty());
    }

    #[test]
    fn normalize_multiple_keeps_all_valid_unique_ids() {
        let mut model = model_with_selected(
            ControlSelectionMode::Multiple,
            ControlGroupStateMode::Unmanaged,
            vec!["compact".into(), "compact".into(), "expanded".into(), "missing".into()],
        );

        normalize_selected_ids(&mut model);

        assert_eq!(model.default_selected_ids, vec![SharedString::from("compact"), SharedString::from("expanded")]);
    }

    #[test]
    fn normalize_active_prefers_selected_then_first_enabled() {
        let mut model = model_with_selected(
            ControlSelectionMode::SingleAllowNone,
            ControlGroupStateMode::Unmanaged,
            vec!["expanded".into()],
        );
        model.active_id = Some("missing".into());

        normalize_active_id(&mut model);

        assert_eq!(model.active_id, Some(SharedString::from("expanded")));
    }

    #[test]
    fn enabled_item_index_ignores_disabled_items() {
        let items = vec![
            ControlGroupItem::new("compact").enabled(false),
            ControlGroupItem::new("comfortable"),
            ControlGroupItem::new("expanded"),
        ];

        assert_eq!(enabled_item_index_by_id(&items, Some(&SharedString::from("compact"))), None);
        assert_eq!(enabled_item_index_by_id(&items, Some(&SharedString::from("comfortable"))), Some(1));
    }

    #[test]
    fn next_enabled_index_skips_disabled_items() {
        let items = vec![
            ControlGroupItem::new("compact"),
            ControlGroupItem::new("comfortable").enabled(false),
            ControlGroupItem::new("expanded"),
        ];

        assert_eq!(next_enabled_index(&items, Some(0), ControlGroupDirection::Next), Some(2));
        assert_eq!(next_enabled_index(&items, Some(2), ControlGroupDirection::Next), Some(0));
        assert_eq!(next_enabled_index(&items, Some(2), ControlGroupDirection::Previous), Some(0));
    }

    #[test]
    fn single_allow_none_toggles_off_selected_item() {
        let items = items();
        let next = compute_next_selected_ids(
            &items,
            &[SharedString::from("comfortable")],
            ControlSelectionMode::SingleAllowNone,
            1,
        );

        assert!(next.is_empty());
    }

    #[test]
    fn single_required_keeps_one_selected_item() {
        let items = items();
        let next = compute_next_selected_ids(
            &items,
            &[SharedString::from("comfortable")],
            ControlSelectionMode::SingleRequired,
            1,
        );

        assert_eq!(next, vec![SharedString::from("comfortable")]);
    }

    #[test]
    fn multiple_toggles_membership() {
        let items = items();
        let next =
            compute_next_selected_ids(&items, &[SharedString::from("compact")], ControlSelectionMode::Multiple, 2);

        assert_eq!(next, vec![SharedString::from("compact"), SharedString::from("expanded")]);

        let toggled_off = compute_next_selected_ids(&items, &next, ControlSelectionMode::Multiple, 0);
        assert_eq!(toggled_off, vec![SharedString::from("expanded")]);
    }

    fn model_with_selected(
        selection_mode: ControlSelectionMode,
        state_mode: ControlGroupStateMode,
        selected_ids: Vec<SharedString>,
    ) -> ControlGroupModel<ControlGroupItem> {
        ControlGroupModel {
            motion: Default::default(),
            stylesheet: crate::theme::ThemeTokens::default().stylesheet,
            stylesheet_overrides_shell: false,
            id: "density".into(),
            items: items(),
            default_selected_ids: selected_ids,
            managed_selected_ids: None,
            active_id: None,
            selection_mode,
            state_mode,
            selection_follows_active: false,
            animated_selection: false,
            independent_toggles: false,
            focus_strategy: ControlGroupFocusStrategy::ActiveDescendant,
            focus_target_provider: None,
            enabled: true,
            tab_stop: true,
            layout: ControlGroupLayout::default(),
            locale_navigation: false,
            scrollable: false,
            template: default_control_group_template(),
            item_template: None,
            item_element_template: None,
        }
    }

    fn items() -> Vec<ControlGroupItem> {
        vec![
            ControlGroupItem::new("compact"),
            ControlGroupItem::new("comfortable"),
            ControlGroupItem::new("expanded"),
        ]
    }
}
