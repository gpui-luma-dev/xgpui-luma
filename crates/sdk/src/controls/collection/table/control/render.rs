use gpui::{Context, IntoElement, MouseButton, Render, Window, div, list, prelude::*, px};

use super::super::layout::{
    body_rows_height as layout_body_rows_height, compute_shell_height, effective_visible_rows, visible_row_height,
};
use super::super::model::{render_grid_view_cells, TableRenderModel, TableRowRenderModel};
use super::super::row::render_table_row;
use super::super::theme::{TableLook, TableRowLook};
use super::TableControl;
use crate::infra::state::ControlFocusState;
use crate::theme::{InteractionState, ListRowScale};

impl<T> TableControl<T>
where
    T: 'static,
{
    pub(super) fn resolve_look(&self, window: &Window) -> TableLook {
        let focus = ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window);
        let mut look = self.model.theme.resolve_look(self.model.enabled, focus.focused, self.model.size);
        if let Some(override_fn) = &self.model.look_override {
            look = override_fn(look);
        }
        look
    }

    pub(super) fn resolve_row_look(&self, window: &Window, _cx: &mut Context<Self>) -> TableRowLook {
        let focus = ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window);
        let scale_factor = window.scale_factor();
        let scale = ListRowScale::compute(self.model.size, &self.model.theme.metrics(), scale_factor);
        self.model.theme.resolve_row_look(
            false,
            InteractionState { focused: focus.focused, ..Default::default() },
            self.model.size,
            &scale,
        )
    }

    pub(super) fn resolve_row_look_for_metrics(&self, scale_factor: f32) -> TableRowLook {
        let scale = ListRowScale::compute(self.model.size, &self.model.theme.metrics(), scale_factor);
        self.model.theme.resolve_row_look(false, InteractionState::default(), self.model.size, &scale)
    }

    pub(super) fn render_model(&self, window: &Window, cx: &mut Context<Self>) -> TableRenderModel<'_> {
        let look = self.resolve_look(window);
        let row_look = self.resolve_row_look(window, cx);
        let has_header = self.model.grid_header || self.model.header_template.is_some();
        let visible_rows =
            effective_visible_rows(self.model.visible_rows, self.model.scroll_mode, self.model.fill_height);
        let row_height = visible_row_height(&row_look, self.model.visible_row_height);
        let body_rows_height =
            visible_rows.map(|count| layout_body_rows_height(count, row_height, row_look.row_divider_width));
        let shell_height = visible_rows.map(|count| compute_shell_height(count, row_height, &look, has_header));

        TableRenderModel {
            id: &self.model.id,
            look,
            row_look,
            row_count: self.model.items.len(),
            selection_mode: self.model.selection_mode,
            scroll_mode: self.model.scroll_mode,
            visible_rows,
            body_rows_height,
            shell_height,
            has_header,
            current_page: self.current_page,
            page_size: self.page_size(),
            page_count: self.page_count(),
            selected_count: self.model.selected_indices.len(),
            enabled: self.model.enabled,
            size: self.model.size,
            focus: ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window),
        }
    }

    pub(super) fn render_row(
        &mut self,
        local_index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let index = self.local_to_global_index(local_index);
        let Some(item) = self.model.items.get(index) else {
            return div().into_any_element();
        };

        let focus = ControlFocusState::from_focus_handle(self.model.enabled, &self.focus_handle, window);
        let enabled = self.model.enabled && self.model.row_is_enabled(item);
        let selected = self.model.selected_indices.binary_search(&index).is_ok();
        let active = self.model.active_index == Some(index);
        let hovered = enabled && self.hovered_index == Some(index);
        let pressed = enabled && self.pressed_index == Some(index);
        let keyboard_active = active && focus.focused;
        let interaction = InteractionState {
            hovered,
            pressed,
            focused: keyboard_active,
            disabled: !enabled && !keyboard_active,
            invalid: false,
        };
        let scale_factor = window.scale_factor();
        let scale = ListRowScale::compute(self.model.size, &self.model.theme.metrics(), scale_factor);
        let mut look = self.model.theme.resolve_row_look(selected, interaction, self.model.size, &scale);
        if let Some(fill_height) = self.fill_row_height {
            look.min_height = fill_height;
        }
        if !self.model.columns.is_empty() {
            // Grid column slots own their horizontal alignment and padding. Keep the
            // body grid on the same column geometry as the header grid.
            look.padding_x = 0.0;
        }

        let row_model = TableRowRenderModel {
            table_id: &self.model.id,
            row: item,
            index,
            sibling_count: self.model.items.len(),
            selected,
            active,
            hovered,
            pressed,
            focused: focus.focused,
            focus_visible: focus.focus_visible,
            enabled,
            look,
        };

        let cells = if !self.model.columns.is_empty() {
            render_grid_view_cells(&row_model, &self.model.columns, window, cx)
        } else {
            div()
                .flex_1()
                .min_w(px(0.0))
                .when(
                    crate::infra::locale::LocaleContext::current(cx).direction
                        == crate::infra::locale::LayoutDirection::Rtl,
                    |label| label.text_right(),
                )
                .truncate()
                .child(self.model.label_for_row(item))
                .into_any_element()
        };

        let is_custom = self.model.row_template.is_some();
        let content = if let Some(row_template) = self.model.row_template.as_ref() {
            row_template(&row_model, cells, window, cx)
        } else {
            cells
        };

        let row_height = self.fill_row_height.or(self.model.visible_row_height);
        let row = render_table_row(
            self.row_keys
                .get(index)
                .map(|key| gpui::ElementId::from((gpui::ElementId::from(self.model.id.clone()), key.clone())))
                .unwrap_or_else(|| gpui::ElementId::from((self.model.id.clone(), index))),
            content,
            look,
            enabled,
            local_index > 0,
            is_custom,
            row_height,
        )
        .on_hover(cx.listener(move |this, hovered, _window, cx| {
            this.handle_item_hover(index, *hovered, cx);
        }))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, event, window, cx| {
                this.handle_item_mouse_down(index, event, window, cx);
            }),
        )
        .on_mouse_up(MouseButton::Left, cx.listener(Self::handle_item_mouse_up))
        .on_mouse_up_out(MouseButton::Left, cx.listener(Self::handle_item_mouse_up))
        .on_click(cx.listener(move |this, event, _window, cx| {
            this.handle_item_click(index, event, cx);
        }));

        let row = self.bind_row_drag(row, index, cx);
        if let Some(key) = self.row_keys.get(index) {
            #[cfg(any(test, feature = "test-support"))]
            let row = {
                let key = key.clone();
                row.debug_selector(move || format!("table-motion-{key}"))
            };
            self.drop_layout.item(key.clone(), row)
        } else {
            row.into_any_element()
        }
    }
}

impl<T> Render for TableControl<T>
where
    T: 'static,
{
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_column_resize_direction(cx);
        self.sync_row_measurement_policy();
        if !self.model.enabled && self.focus_handle.contains_focused(window, cx) {
            window.blur(cx);
        }
        if self.focus_in_subscription.is_none() {
            let focus_handle = self.focus_handle.clone();
            self.focus_in_subscription = Some(cx.on_focus(&focus_handle, window, Self::handle_focus_in));
        }
        if self.focus_out_subscription.is_none() {
            let focus_handle = self.focus_handle.clone();
            self.focus_out_subscription = Some(cx.on_focus_out(&focus_handle, window, Self::handle_focus_out));
        }

        let policy = self.model.scroll_interaction;
        if !self.model.enabled
            || !policy.wheel.accepts(policy.focus_scope.focused(&self.focus_handle, None, window, cx))
        {
            self.wheel_gesture_active = false;
        }
        let render_model = self.render_model(window, cx);
        let header = if self.model.grid_header {
            Some(self.render_grid_header(&render_model.look, cx))
        } else {
            self.model.header_template.as_ref().map(|template| template(&render_model, window, cx))
        };
        let mut list_element = list(self.list_state.clone(), cx.processor(Self::render_row));
        if let Some(body_rows_height) = render_model.body_rows_height {
            list_element = list_element.h(px(body_rows_height)).w_full();
        } else {
            list_element = list_element.size_full();
        }
        let focus = self.focus_handle.clone();
        let policy = self.model.scroll_interaction;
        let enabled = self.model.enabled && !self.is_paged();
        let owner = cx.entity().downgrade();
        let routing = crate::interaction::list_wheel_routing(
            self.list_state.clone(),
            move |window, cx| enabled && policy.wheel.accepts(policy.focus_scope.focused(&focus, None, window, cx)),
            policy.boundary,
            move |event, window, cx| {
                let _ = owner.update(cx, |table, cx| table.handle_scroll_wheel(event, window, cx));
            },
        );
        self.drop_layout.configure_transition(&self.model.motion.profile().drop_layout);
        self.drop_layout.set_motion_enabled(self.model.motion.enabled());
        let scroll = self.list_state.clone();
        let list_element = self.drop_layout.scope(list_element, move || scroll.scroll_px_offset_for_scrollbar());
        let body = div()
            .relative()
            .w_full()
            .when(render_model.body_rows_height.is_none(), |body| body.h_full())
            .child(routing)
            .child(list_element)
            .into_any_element();

        let shell = self
            .model
            .template
            .render(&render_model, header, body, window, cx)
            .track_focus(&self.focus_handle)
            .key_context("LumaTableControl")
            .on_drag_move(cx.listener(Self::handle_drag_move))
            .on_drag_move(cx.listener(Self::handle_column_resize_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::finish_column_resize))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::finish_column_resize))
            .on_key_down(cx.listener(Self::handle_key_down))
            .on_action(cx.listener(|this, event, window, cx| {
                if this.model.enabled && this.focus_handle.is_focused(window) {
                    this.handle_select_previous_item(event, window, cx);
                } else {
                    cx.propagate();
                }
            }))
            .on_action(cx.listener(|this, event, window, cx| {
                if this.model.enabled && this.focus_handle.is_focused(window) {
                    this.handle_select_next_item(event, window, cx);
                } else {
                    cx.propagate();
                }
            }))
            .on_action(cx.listener(|this, event, window, cx| {
                if this.model.enabled && this.focus_handle.is_focused(window) {
                    this.handle_decrease_value_large(event, window, cx);
                } else {
                    cx.propagate();
                }
            }))
            .on_action(cx.listener(|this, event, window, cx| {
                if this.model.enabled && this.focus_handle.is_focused(window) {
                    this.handle_increase_value_large(event, window, cx);
                } else {
                    cx.propagate();
                }
            }))
            .on_action(cx.listener(|this, event, window, cx| {
                if this.model.enabled && this.focus_handle.is_focused(window) {
                    this.handle_select_first_item(event, window, cx);
                } else {
                    cx.propagate();
                }
            }))
            .on_action(cx.listener(|this, event, window, cx| {
                if this.model.enabled && this.focus_handle.is_focused(window) {
                    this.handle_select_last_item(event, window, cx);
                } else {
                    cx.propagate();
                }
            }))
            .on_action(cx.listener(|this, event, window, cx| {
                if this.model.enabled && this.focus_handle.is_focused(window) {
                    this.handle_activate_control(event, window, cx);
                } else {
                    cx.propagate();
                }
            }));

        let sync_page_size = self.should_sync_page_size_to_viewport();
        let entity = cx.entity();
        let mut root = div().w_full();
        if render_model.shell_height.is_none() {
            root = root.h_full();
        }
        if sync_page_size || self.uniform_row_height.is_some() {
            root = root.on_children_prepainted(move |_child_bounds, window, cx| {
                entity.update(cx, |this, cx| {
                    if sync_page_size {
                        this.sync_fill_height_page_size(window, cx);
                    }
                    this.sync_uniform_row_hints(cx);
                });
            });
        }
        root.child(shell).into_any_element()
    }
}
