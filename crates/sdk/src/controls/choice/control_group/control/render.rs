use gpui::{App, Context, IntoElement, Render, SharedString, Window, div, prelude::*};

use super::super::model::{
    ControlGroupFocusStrategy, ControlGroupItemLike, ControlGroupItemRenderModel, ControlGroupRenderModel,
};
use super::selection::selected_ids_contain;
use super::ControlGroupControl;
use crate::infra::state::{CompositeItemState, ControlFocusState};
use crate::key_handling::ControlKeyProfile;
use crate::motion::{VisualTransition};

impl<T> ControlGroupControl<T>
where
    T: ControlGroupItemLike + 'static,
{
    pub(super) fn render_model<'a>(&'a self, window: &mut Window, cx: &mut App) -> ControlGroupRenderModel<'a, T> {
        let focus = ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window);
        let effective_selected_ids = self.model.effective_selected_ids();
        let item_count = self.model.items.len();

        let items = self
            .model
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let enabled = self.model.enabled && item.is_enabled();
                let selected = selected_ids_contain(effective_selected_ids, item.id());
                let active = self.model.active_id.as_ref().is_some_and(|active_id| active_id == item.id());
                let mut item_active = enabled && focus.focused && active;
                let mut item_focus_visible = enabled && focus.focus_visible && active;

                if enabled
                    && active
                    && self.model.focus_strategy == ControlGroupFocusStrategy::RovingItemFocus
                    && let Some(target) = self.focus_target_for_index(index, window, cx)
                    && target.focus_handle.is_focused(window)
                {
                    item_active = true;
                    item_focus_visible = crate::infra::state::focus_visible(true, window);
                }

                let state = CompositeItemState {
                    hovered: enabled && self.items_pointer.is_hovered(item.id()),
                    pressed: enabled && self.items_pointer.is_pressed(item.id()),
                    disabled: !enabled,
                    selected,
                    active: item_active,
                    focus_visible: item_focus_visible,
                };
                let state = self.item_state_override.as_ref().map_or(state, |map| map(item.id(), state));
                // The drawn selection follows the mapped state.
                let selected = state.selected;

                ControlGroupItemRenderModel {
                    group_id: &self.model.id,
                    stylesheet: &self.model.stylesheet,
                    layout: self.model.layout,
                    item,
                    index,
                    sibling_count: item_count,
                    selected,
                    selection_progress: if self.model.animated_selection {
                        self.selection_transitions
                            .get(item.id())
                            .map_or(if selected { 1.0 } else { 0.0 }, VisualTransition::progress)
                    } else if selected {
                        1.0
                    } else {
                        0.0
                    },
                    active,
                    enabled,
                    state,
                    selection_mode: self.model.selection_mode,
                    foreground: None,
                }
            })
            .collect();

        ControlGroupRenderModel {
            id: &self.model.id,
            stylesheet: &self.model.stylesheet,
            items,
            selected_ids: effective_selected_ids,
            active_id: self.model.active_id.as_ref(),
            selection_mode: self.model.selection_mode,
            state_mode: self.model.state_mode,
            enabled: self.model.enabled,
            layout: self.model.layout,
            scroll_handle: self.scroll_handle.as_ref(),
            focus,
            focus_strategy: self.model.focus_strategy,
            item_template: self.model.item_template.as_ref(),
            item_element_template: self.model.item_element_template.as_ref(),
        }
    }

    pub(super) fn sync_selection_transitions(&mut self) {
        let selected_ids = self.model.effective_selected_ids().to_vec();
        self.selection_transitions.retain(|id, _| self.model.items.iter().any(|item| item.id() == id));

        for item in &self.model.items {
            self.selection_transitions.entry(item.id().clone()).or_insert_with(|| {
                VisualTransition::with_transition(
                    if selected_ids_contain(&selected_ids, item.id()) {
                        1.0
                    } else {
                        0.0
                    },
                    self.model.motion.profile().selection.clone(),
                )
            });
        }

        self.sync_selection_transitions_to(&selected_ids);
    }

    pub(super) fn sync_selection_transitions_to(&mut self, selected_ids: &[SharedString]) {
        for item in &self.model.items {
            if let Some(transition) = self.selection_transitions.get_mut(item.id()) {
                let target = if selected_ids_contain(selected_ids, item.id()) {
                    1.0
                } else {
                    0.0
                };
                if self.model.animated_selection {
                    transition.set_target(target);
                } else {
                    transition.snap_to(target);
                }
            }
        }
    }
}

impl<T> Render for ControlGroupControl<T>
where
    T: ControlGroupItemLike + 'static,
{
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.model.animated_selection {
            let mut animating = false;
            for transition in self.selection_transitions.values_mut() {
                animating |= transition.sync();
            }
            if animating {
                cx.on_next_frame(window, |_, _, cx| cx.notify());
            }
        }

        let focus_handle = self.focus_handle.clone();

        self.focus_subscriptions.ensure(
            &focus_handle,
            window,
            cx,
            Self::handle_group_focus_entry,
            Self::handle_group_focus_out,
        );
        self.ensure_item_focus_subscriptions(window, cx);

        if self.model.focus_strategy == ControlGroupFocusStrategy::RovingItemFocus
            && self.focus_handle.is_focused(window)
            && let Some(target) = self.active_focus_target(window, cx)
        {
            target.focus_handle.focus(window, cx);
        }

        let model = self.render_model(window, cx);
        let handlers = self.template_handlers(cx);

        let root = div().child(
            (self.model.template)(&model, handlers, window, cx)
                .track_focus(&self.focus_handle)
                .key_context(ControlKeyProfile::TabList.context())
                .on_action(cx.listener(Self::handle_select_previous_item))
                .on_action(cx.listener(Self::handle_select_next_item))
                .on_action(cx.listener(Self::handle_select_previous_row))
                .on_action(cx.listener(Self::handle_select_next_row))
                .on_action(cx.listener(Self::handle_select_first_item))
                .on_action(cx.listener(Self::handle_select_last_item))
                .on_action(cx.listener(Self::handle_activate_control))
                .on_action(cx.listener(Self::handle_group_next_focus))
                .on_action(cx.listener(Self::handle_group_prev_focus)),
        );

        root.into_any_element()
    }
}

#[cfg(test)]
mod item_state_override_tests {
    use crate::controls::control_group::{ControlGroupControl, ControlGroupItem};
    use crate::infra::state::CompositeItemState;
    use gpui::SharedString;

    /// A state override changes the mapped item's drawn state and selection only (#397).
    #[test]
    fn a_state_override_maps_one_items_drawn_state() {
        let mut app = gpui::TestAppContext::single();
        let (group, cx) = app.add_window_view(|_, cx| {
            ControlGroupControl::from_builder(
                ControlGroupControl::new("override-group")
                    .items([ControlGroupItem::new("a"), ControlGroupItem::new("b")])
                    .selected_ids(["b"]),
                cx,
            )
        });
        cx.run_until_parked();
        cx.update(|window, app| {
            group.update(app, |group, cx| {
                group.set_item_state_override(
                    Some(std::sync::Arc::new(|id: &SharedString, state: CompositeItemState| CompositeItemState {
                        pressed: id.as_ref() == "a",
                        selected: id.as_ref() == "a",
                        ..state
                    })),
                    cx,
                );
                let model = group.render_model(window, cx);
                let (a, b) = (&model.items[0], &model.items[1]);
                assert!(a.state.pressed && a.selected && a.state.selected, "a drawn pressed and selected");
                assert!(!b.selected, "b drawn unselected");
                assert_eq!(model.selected_ids, [SharedString::from("b")], "the real selection is b");
            });
        });
    }
}
