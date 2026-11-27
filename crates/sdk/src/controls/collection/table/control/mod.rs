use crate::infra::focus_report::ReportsFocus as _;
mod input;
mod render;
mod selection;
mod viewport;
#[cfg(all(test, feature = "test-support"))]
mod virtualization_tests;
mod keys;
mod reorder;
mod drag;
mod resize;
pub use reorder::{TableReorderError, TableRowDrop, TableRowDragEvent};

#[cfg(all(test, feature = "test-support"))]
mod tests;

pub use keys::TableSelectionError;

use gpui::{App, Context, EventEmitter, Focusable, IntoElement, ListOffset, ListState, SharedString, Window, px};

use super::layout::{clamp_page, page_after_size_change, page_count};
use super::model::{
    TableBuilder, TableHeaderTemplate, TableLabel, TableLookOverride, TableModel, TableRenderModel,
    TableRowRenderModel, TableScrollMode, TableSelectionMode, make_table_header_template, make_table_row_template,
};
use super::template::TableTemplate;
use super::theme::TableTheme;
use crate::theme::observe_theme_revision;

use self::selection::normalize_model;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum TableEvent {
    /// Shared drag lifecycle, emitted after any committed data/selection update.
    RowDrag(TableRowDragEvent),
    SelectionChanged {
        selected_indices: Vec<usize>,
    },
    /// Stable selected identities, emitted only when membership changes.
    /// Requires [`TableControl::set_row_key`]. Order follows the current rows.
    SelectedKeysChanged {
        selected_keys: Vec<SharedString>,
    },
    ActiveIndexChanged {
        active_index: Option<usize>,
    },
    PageChanged {
        page: usize,
    },
    PageSizeChanged {
        page_size: usize,
    },
    ScrollChanged {
        top_index: usize,
    },
    FocusChanged {
        focused: bool,
    },
    RowHoverChanged {
        index: usize,
        hovered: bool,
    },
    EnabledChanged {
        enabled: bool,
    },
}

pub struct TableControl<T>
where
    T: 'static,
{
    pub(crate) drop_layout: crate::motion::DropLayoutMotion<SharedString>,

    model: TableModel<T>,
    list_state: ListState,
    /// Height used to seed offscreen rows; None retains variable-height measurement.
    uniform_row_height: Option<f32>,
    uniform_hint_width: Option<gpui::Pixels>,
    focus_handle: gpui::FocusHandle,
    focus_subscriptions: crate::infra::focus_report::FocusSubscriptions,
    /// The hovered and pressed rows, by index.
    pub(super) rows_pointer: crate::infra::item_tracker::ItemTracker<usize>,
    current_page: usize,
    emitted_scroll_top_index: usize,
    focus_report: crate::infra::focus_report::FocusReport,
    wheel_gesture_active: bool,
    /// When fill-height + paged, row height stretched so `page_size` rows fill the viewport.
    fill_row_height: Option<f32>,
    /// Maps each row's drawn state by row index (`set_item_state_override`).
    item_state_override: Option<crate::infra::state::ItemStateOverride<usize>>,
    row_key: Option<keys::RowKeyFn<T>>,
    row_keys: Vec<SharedString>,
    selection_anchor: Option<usize>,
    row_reordering: bool,
    active_row_drag: Option<reorder::RowDrag>,
    row_drop_handler: Option<reorder::DropHandler<T>>,
    drag_scroll_scheduled: bool,
    measured_column_widths: Vec<f32>,
    measured_header_width: f32,
    column_resize: Option<resize::ColumnResize>,
}

impl<T> EventEmitter<TableEvent> for TableControl<T> where T: 'static {}

impl TableControl<TableLabel> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> TableBuilder<TableLabel> {
        TableBuilder::new(id)
    }
}

impl<T> TableControl<T>
where
    T: 'static,
{
    #[allow(clippy::new_ret_no_self)]
    pub fn new_typed(id: impl Into<SharedString>) -> TableBuilder<T> {
        TableBuilder::new_typed(id)
    }

    /// Current wheel settings. Keyboard ownership is independent.
    pub fn scroll_interaction(&self) -> crate::interaction::ScrollInteraction {
        self.model.scroll_interaction
    }

    /// Change wheel without resetting focus, selection or position.
    pub fn set_wheel_scroll_policy(&mut self, policy: crate::interaction::WheelScrollPolicy, cx: &mut Context<Self>) {
        self.model.scroll_interaction.wheel = policy;
        self.wheel_gesture_active = false;
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

    /// Control-wide policy. `false` settles motion immediately and keeps [`Self::motion_profile`].
    pub fn motion_enabled(&self) -> bool {
        self.model.motion.enabled()
    }

    pub fn set_motion_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.model.motion.set_enabled(enabled);
        self.drop_layout.cancel();
        cx.notify();
    }
    crate::motion_snapshot!(super::TableMotion, model.motion);

    pub(crate) fn from_builder(builder: TableBuilder<T>, cx: &mut Context<Self>) -> Self {
        let mut builder = builder;
        builder.model.motion.freeze();
        normalize_model(&mut builder.model);
        observe_theme_revision(cx, |_, cx| cx.notify()).detach();

        let uniform_row_height = Self::fixed_row_height(&builder.model);
        let list_state = Self::new_list_state(&builder.model, 0);
        Self {
            list_state,
            uniform_row_height,
            uniform_hint_width: None,
            focus_handle: cx.focus_handle().tab_stop(builder.model.enabled),
            focus_subscriptions: Default::default(),
            model: builder.model,
            drop_layout: Default::default(),
            rows_pointer: Default::default(),
            current_page: 0,
            emitted_scroll_top_index: 0,
            focus_report: Default::default(),
            wheel_gesture_active: false,
            fill_row_height: None,
            item_state_override: None,
            row_key: None,
            row_keys: Vec::new(),
            selection_anchor: None,
            row_reordering: false,
            active_row_drag: None,
            row_drop_handler: None,
            drag_scroll_scheduled: false,
            measured_column_widths: Vec::new(),
            measured_header_width: 0.0,
            column_resize: None,
        }
    }

    /// Retained keyed post-drop geometry and running transitions.
    pub fn drop_layout_motion(&self) -> crate::motion::DropLayoutMotion<SharedString> {
        self.drop_layout.clone()
    }

    pub fn items(&self) -> &[T] {
        &self.model.items
    }

    pub fn selected_indices(&self) -> &[usize] {
        &self.model.selected_indices
    }

    pub fn active_index(&self) -> Option<usize> {
        self.model.active_index
    }

    pub fn selection_mode(&self) -> TableSelectionMode {
        self.model.selection_mode
    }

    pub fn scroll_mode(&self) -> TableScrollMode {
        self.model.scroll_mode
    }

    pub fn fills_height(&self) -> bool {
        self.model.fill_height
    }

    pub fn is_enabled(&self) -> bool {
        self.model.enabled
    }

    pub fn current_page(&self) -> usize {
        self.current_page
    }

    pub fn page_size(&self) -> Option<usize> {
        match self.model.scroll_mode {
            TableScrollMode::Paged { page_size } => Some(page_size.max(1)),
            _ => None,
        }
    }

    pub fn page_count(&self) -> usize {
        page_count(self.model.items.len(), self.page_size().unwrap_or(1))
    }

    pub fn selected_items(&self) -> Vec<&T> {
        self.model.selected_indices.iter().filter_map(|index| self.model.items.get(*index)).collect()
    }

    /// Replace rows, retaining selected/active identities when keys are configured.
    /// Duplicate keys reject the whole update. Unkeyed tables retain indices.
    pub fn set_items(
        &mut self,
        items: impl IntoIterator<Item = T>,
        cx: &mut Context<Self>,
    ) -> Result<(), TableSelectionError> {
        let items: Vec<T> = items.into_iter().collect();
        let keys = self.row_key.as_ref().map(|key| keys::collect_keys(&items, key.as_ref())).transpose()?;
        self.drop_layout.cancel();
        self.cancel_row_drag(cx);
        let previous = self.selection_snapshot();
        let previous_page = self.current_page;
        if let Some(keys) = keys {
            self.remap_selection(&keys);
            self.row_keys = keys;
        } else {
            self.selection_anchor = None;
        }
        self.model.items = items;
        self.clear_pointer_interaction(cx);
        self.current_page = clamp_page(self.current_page, self.model.items.len(), self.page_size().unwrap_or(1));
        self.sync_list_state();
        normalize_model(&mut self.model);
        self.selection_anchor = self
            .selection_anchor
            .filter(|&index| self.model.items.get(index).is_some_and(|row| self.model.row_is_enabled(row)));
        self.list_state.remeasure();
        self.emit_selection_changes(previous, cx);
        if previous_page != self.current_page {
            cx.emit(TableEvent::PageChanged { page: self.current_page });
        }
        cx.notify();
        Ok(())
    }

    /// Owner-driven replacement. Preserves focus, active row and scroll position;
    /// invalid/disabled indices are discarded and Single keeps the first valid row.
    /// Changed membership resets the range anchor; an owner echo preserves it.
    pub fn set_selected_indices(&mut self, selected_indices: impl IntoIterator<Item = usize>, cx: &mut Context<Self>) {
        let previous = self.selection_snapshot();
        let next = selection::normalize_selected_indices(
            self.model.selection_mode,
            &self.model.items,
            self.model.row_enabled.as_ref(),
            &selected_indices.into_iter().collect::<Vec<_>>(),
        );
        if next == self.model.selected_indices {
            return;
        }
        self.model.selected_indices = next;
        self.selection_anchor = None;
        self.emit_selection_changes(previous, cx);
        cx.notify();
    }

    /// Toggles a row in Multiple/Extended; selects it in Single.
    pub fn toggle_selected_index(&mut self, index: usize, cx: &mut Context<Self>) -> bool {
        self.select_index(index, true, false, cx)
    }

    pub fn set_active_index(&mut self, active_index: Option<usize>, cx: &mut Context<Self>) {
        self.set_active_index_internal(active_index, None, cx);
    }

    pub fn set_selection_mode(&mut self, selection_mode: TableSelectionMode, cx: &mut Context<Self>) {
        if self.model.selection_mode == selection_mode {
            return;
        }

        let previous = self.selection_snapshot();
        self.model.selection_mode = selection_mode;
        self.selection_anchor = None;
        normalize_model(&mut self.model);
        self.emit_selection_changes(previous, cx);
        cx.notify();
    }

    pub fn set_scroll_mode(&mut self, scroll_mode: TableScrollMode, cx: &mut Context<Self>) {
        if self.model.scroll_mode == scroll_mode {
            return;
        }

        self.model.scroll_mode = scroll_mode;
        self.fill_row_height = None;
        self.current_page = clamp_page(self.current_page, self.model.items.len(), self.page_size().unwrap_or(1));
        self.sync_list_state();
        cx.notify();
    }

    pub fn set_page(&mut self, page: usize, cx: &mut Context<Self>) {
        if !self.is_paged() {
            return;
        }

        let next = clamp_page(page, self.model.items.len(), self.page_size().unwrap_or(1));
        if self.current_page == next {
            return;
        }

        self.current_page = next;
        self.sync_list_state();
        cx.emit(TableEvent::PageChanged { page: next });
        cx.notify();
    }

    pub fn next_page(&mut self, cx: &mut Context<Self>) {
        self.set_page(self.current_page.saturating_add(1), cx);
    }

    pub fn prev_page(&mut self, cx: &mut Context<Self>) {
        self.set_page(self.current_page.saturating_sub(1), cx);
    }

    pub fn first_page(&mut self, cx: &mut Context<Self>) {
        self.set_page(0, cx);
    }

    pub fn last_page(&mut self, cx: &mut Context<Self>) {
        let last = self.page_count().saturating_sub(1);
        self.set_page(last, cx);
    }

    pub fn set_page_size(&mut self, page_size: usize, cx: &mut Context<Self>) {
        let page_size = page_size.max(1);
        if self.page_size() == Some(page_size) {
            return;
        }

        let old_page_size = self.page_size().unwrap_or(1);
        let keep_in_view = self.focus_index_for_page_size_change();
        let next_page =
            page_after_size_change(self.current_page, old_page_size, page_size, self.model.items.len(), keep_in_view);
        self.model.scroll_mode = TableScrollMode::Paged { page_size };
        self.current_page = next_page;
        self.sync_list_state();
        cx.emit(TableEvent::PageSizeChanged { page_size });
        cx.emit(TableEvent::PageChanged { page: self.current_page });
        cx.notify();
    }

    /// Row to keep visible across a page-size change: active when selected (or when nothing is
    /// selected), otherwise the first selected index.
    fn focus_index_for_page_size_change(&self) -> Option<usize> {
        if let Some(active) = self.model.active_index
            && (self.model.selected_indices.is_empty() || self.model.selected_indices.contains(&active))
        {
            return Some(active);
        }
        self.model.selected_indices.first().copied()
    }

    pub fn scroll_to_row(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.is_paged() {
            let page_size = self.page_size().unwrap_or(1);
            self.set_page(index / page_size, cx);
            self.set_active_index_internal(Some(index), None, cx);
            return;
        }

        if self.is_scroll_snap() {
            self.list_state.scroll_to(ListOffset { item_ix: index, offset_in_item: px(0.0) });
        } else {
            self.reveal_row(index);
        }
        self.emit_scroll_changed_if_needed(cx);
        cx.notify();
    }

    pub fn scroll_by_pixels(&mut self, pixels: f32, cx: &mut Context<Self>) {
        if self.is_paged() || !pixels.is_finite() {
            return;
        }

        self.list_state.scroll_by(px(pixels));
        if self.is_scroll_snap() {
            self.snap_scroll_position();
        }
        self.emit_scroll_changed_if_needed(cx);
        cx.notify();
    }

    /// Draw each row in the state `map` gives it, from its row index and the state input gives it, or as input gives
    /// it (`None`). Only the look changes; selection, input and events follow the real state (#397).
    pub fn set_item_state_override(
        &mut self,
        map: Option<crate::infra::state::ItemStateOverride<usize>>,
        cx: &mut Context<Self>,
    ) {
        self.item_state_override = map;
        cx.notify();
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if !enabled {
            self.drop_layout.cancel();
            self.cancel_row_drag(cx);
            self.cancel_column_resize(cx);
        }
        if self.model.enabled == enabled {
            return;
        }

        self.model.enabled = enabled;
        self.focus_handle = self.focus_handle.clone().tab_stop(enabled);
        self.focus_subscriptions.reset();

        if !enabled {
            self.clear_pointer_interaction(cx);
            self.emit_focus_changed(false, cx);
        }

        cx.emit(TableEvent::EnabledChanged { enabled });
        cx.notify();
    }

    pub fn set_template(&mut self, template: std::sync::Arc<dyn TableTemplate>, cx: &mut Context<Self>) {
        self.model.template = template;
        cx.notify();
    }

    pub fn set_theme(&mut self, theme: std::sync::Arc<dyn TableTheme>, cx: &mut Context<Self>) {
        self.model.theme = theme;
        self.list_state.remeasure();
        cx.notify();
    }

    pub fn set_look_override(&mut self, look_override: Option<TableLookOverride>, cx: &mut Context<Self>) {
        self.model.look_override = look_override;
        self.list_state.remeasure();
        cx.notify();
    }

    pub fn set_header_template(&mut self, header_template: Option<TableHeaderTemplate>, cx: &mut Context<Self>) {
        self.cancel_column_resize(cx);
        self.model.grid_header = false;
        self.model.header_template = header_template;
        cx.notify();
    }

    pub fn set_header_template_fn<F, E>(&mut self, template: F, cx: &mut Context<Self>)
    where
        F: for<'a> Fn(&TableRenderModel<'a>, &mut Window, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.cancel_column_resize(cx);
        self.model.grid_header = false;
        self.model.header_template = Some(make_table_header_template(template));
        cx.notify();
    }

    pub fn clear_header_template(&mut self, cx: &mut Context<Self>) {
        if self.model.grid_header || self.model.header_template.is_some() {
            self.cancel_column_resize(cx);
            self.model.grid_header = false;
            self.model.header_template = None;
            cx.notify();
        }
    }

    pub fn set_row_template_fn<F, E>(&mut self, template: F, cx: &mut Context<Self>)
    where
        F: for<'a> Fn(&TableRowRenderModel<'a, T>, gpui::AnyElement, &mut Window, &mut App) -> E
            + Send
            + Sync
            + 'static,
        E: IntoElement + 'static,
    {
        self.model.row_template = Some(make_table_row_template(template));
        self.list_state.remeasure();
        cx.notify();
    }

    pub fn clear_row_template(&mut self, cx: &mut Context<Self>) {
        if self.model.row_template.is_some() {
            self.model.row_template = None;
            self.list_state.remeasure();
            cx.notify();
        }
    }
}

impl<T> Focusable for TableControl<T>
where
    T: 'static,
{
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

#[cfg(all(test, feature = "test-support"))]
#[test]
fn wheel_policy_dispatch_matrix() {
    crate::interaction_tests::matrix(
        |policy, cx| {
            TableBuilder::new_typed("matrix-table")
                .items(0..100)
                .row_label(|i| i.to_string())
                .visible_rows(4)
                .visible_row_height(30.0_f32)
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

impl<T: 'static> crate::infra::focus_report::ReportsFocus for TableControl<T> {
    type FocusEvent = TableEvent;

    fn focus_report(&mut self) -> &mut crate::infra::focus_report::FocusReport {
        &mut self.focus_report
    }

    fn focus_event(focused: bool) -> TableEvent {
        TableEvent::FocusChanged { focused }
    }

    fn focus_enabled(&self) -> bool {
        self.model.enabled
    }
}
