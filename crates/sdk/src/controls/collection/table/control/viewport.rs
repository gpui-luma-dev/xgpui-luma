use gpui::{Context, ListOffset, ListState, Window, px};

use super::super::layout::{
    distributed_row_height, effective_visible_rows, page_start, rows_for_viewport_height_border_box,
    visible_item_count, visible_row_height,
};
use super::super::model::{TableModel, TableScrollMode};
use super::super::styled::TableRowLayout;
use super::selection::TableDirection;
use super::{TableControl, TableEvent};

impl<T> TableControl<T>
where
    T: 'static,
{
    pub(super) fn is_paged(&self) -> bool {
        matches!(self.model.scroll_mode, TableScrollMode::Paged { .. })
    }

    pub(super) fn is_scroll_snap(&self) -> bool {
        matches!(self.model.scroll_mode, TableScrollMode::ScrollSnap)
    }

    pub(super) fn visible_list_item_count(model: &TableModel<T>, current_page: usize) -> usize {
        if let TableScrollMode::Paged { page_size } = model.scroll_mode {
            visible_item_count(model.items.len(), current_page, page_size)
        } else {
            model.items.len()
        }
    }

    pub(super) fn fixed_row_height(model: &TableModel<T>) -> Option<f32> {
        // Auto-sized pages stretch rows after layout, overriding the explicit
        // height. Their small current page retains the measured fallback.
        if model.fill_height && model.scroll_mode.is_paged() && model.visible_rows.is_none() {
            return None;
        }
        model.visible_row_height.filter(|height| height.is_finite() && *height > 0.0)
    }

    pub(super) fn new_list_state(model: &TableModel<T>, page: usize) -> ListState {
        let state = ListState::new(Self::visible_list_item_count(model, page), model.alignment, px(model.overdraw));
        if let Some(height) = Self::fixed_row_height(model) {
            state.with_uniform_item_height(px(height))
        } else if model.visible_rows.is_some() || model.fill_height {
            // Arbitrary row/cell content can grow. Preserve exact extents for
            // variable-height fixed viewports instead of assuming a row size.
            state.measure_all()
        } else {
            state
        }
    }

    pub(super) fn sync_row_measurement_policy(&mut self) {
        let height = Self::fixed_row_height(&self.model);
        if self.uniform_row_height != height {
            let top = self.list_state.logical_scroll_top();
            self.list_state = Self::new_list_state(&self.model, self.current_page);
            self.list_state.scroll_to(top);
            self.uniform_row_height = height;
            self.uniform_hint_width = None;
        }
    }

    pub(super) fn sync_uniform_row_hints(&mut self, cx: &mut Context<Self>) {
        let Some(height) = self.uniform_row_height else { return };
        let width = self.list_state.viewport_bounds().size.width;
        if width > px(0.0) && self.uniform_hint_width != Some(width) {
            // GPUI discards all height hints during width-change prepaint,
            // including first layout. Restore them after child prepaint and
            // request one bounded redraw so scroll extents become exact.
            self.list_state = self.list_state.clone().with_uniform_item_height(px(height));
            self.uniform_hint_width = Some(width);
            cx.notify();
        }
    }

    pub(super) fn reveal_row(&self, index: usize) {
        if self.uniform_row_height.is_some() || self.list_state.bounds_for_item(index).is_some() {
            self.list_state.scroll_to_reveal_item(index);
        } else {
            // Measured height sums cannot locate an unmeasured distant row.
            self.list_state.scroll_to(ListOffset { item_ix: index, offset_in_item: px(0.0) });
        }
    }

    pub(super) fn sync_list_state(&mut self) {
        self.sync_row_measurement_policy();
        let count = Self::visible_list_item_count(&self.model, self.current_page);
        if self.list_state.item_count() != count {
            if let Some(height) = self.uniform_row_height {
                self.list_state.reset_with_uniform_height(count, px(height));
            } else {
                self.list_state.reset(count);
            }
        }
    }

    pub(super) fn local_to_global_index(&self, local_index: usize) -> usize {
        if self.is_paged() {
            page_start(self.current_page, self.page_size().unwrap_or(1)) + local_index
        } else {
            local_index
        }
    }

    pub(super) fn ensure_page_for_index(&mut self, index: usize, cx: &mut Context<Self>) {
        if !self.is_paged() {
            return;
        }

        let page_size = self.page_size().unwrap_or(1);
        let page = index / page_size;
        if page != self.current_page {
            self.set_page(page, cx);
        }
    }

    pub(super) fn row_scroll_increment(&self, layout: &TableRowLayout) -> f32 {
        self.fill_row_height.unwrap_or_else(|| visible_row_height(layout, self.model.visible_row_height))
    }

    pub(super) fn token_row_height(&self, scale_factor: f32) -> f32 {
        visible_row_height(&self.row_layout(scale_factor), self.model.visible_row_height)
    }

    pub(super) fn visible_row_capacity(&self) -> usize {
        if let Some(count) =
            effective_visible_rows(self.model.visible_rows, self.model.scroll_mode, self.model.fill_height)
        {
            return count;
        }

        if self.should_sync_page_size_to_viewport() {
            return self.page_size().unwrap_or(1);
        }

        self.viewport_page_size(1.0).unwrap_or(1)
    }

    pub(super) fn should_sync_page_size_to_viewport(&self) -> bool {
        self.model.fill_height && self.is_paged() && self.model.visible_rows.is_none()
    }

    pub(super) fn viewport_page_size(&self, scale_factor: f32) -> Option<usize> {
        let viewport_height = self.list_state.viewport_bounds().size.height.as_f32();
        // Fill-height rows paint dividers as border_t inside `.h(...)`.
        rows_for_viewport_height_border_box(viewport_height, self.token_row_height(scale_factor))
    }

    /// When fill-height + paged, match `page_size` to the viewport and stretch rows to fill it.
    pub(super) fn sync_fill_height_page_size(&mut self, window: &Window, cx: &mut Context<Self>) {
        if !self.should_sync_page_size_to_viewport() {
            return;
        }
        let viewport_height = self.list_state.viewport_bounds().size.height.as_f32();
        let Some(page_size) = self.viewport_page_size(window.scale_factor()) else {
            return;
        };
        let Some(row_height) = distributed_row_height(viewport_height, page_size) else {
            return;
        };

        let height_changed = self.fill_row_height.is_none_or(|current| (current - row_height).abs() > 0.05);
        self.fill_row_height = Some(row_height);

        if self.page_size() != Some(page_size) {
            self.set_page_size(page_size, cx);
            self.list_state.remeasure();
        } else if height_changed {
            self.list_state.remeasure();
            cx.notify();
        }
    }

    pub(super) fn is_active_index_in_viewport(&self, index: usize) -> bool {
        let scroll_top = self.list_state.logical_scroll_top();
        let visible_rows = self.visible_row_capacity().max(1);
        let first = scroll_top.item_ix;
        let last = first.saturating_add(visible_rows.saturating_sub(1));
        index >= first && index <= last
    }

    /// Keep the active row rendered inside the viewport. When moving up within the
    /// first page, pin scroll at row 0 so the list top stays anchored.
    pub(super) fn scroll_active_into_view(
        &mut self,
        index: usize,
        direction: Option<TableDirection>,
        cx: &mut Context<Self>,
    ) {
        let visible_rows = self.visible_row_capacity().max(1);

        if direction == Some(TableDirection::Previous) && index < visible_rows {
            self.list_state.scroll_to(ListOffset { item_ix: 0, offset_in_item: px(0.0) });
            self.emit_scroll_changed_if_needed(cx);
            return;
        }

        if self.is_active_index_in_viewport(index) {
            return;
        }

        if index < visible_rows {
            self.list_state.scroll_to(ListOffset { item_ix: 0, offset_in_item: px(0.0) });
        } else {
            self.reveal_row(index);
        }
        self.emit_scroll_changed_if_needed(cx);
    }

    pub(super) fn snap_scroll_position(&mut self) {
        let layout = self.row_layout(1.0);
        let row_height = self.row_scroll_increment(&layout);
        let offset = self.list_state.logical_scroll_top();
        let mut item_ix = offset.item_ix;
        if row_height > 0.0 && offset.offset_in_item >= px(row_height * 0.5) {
            item_ix += 1;
        }
        self.list_state.scroll_to(ListOffset { item_ix, offset_in_item: px(0.0) });
    }

    pub(super) fn page_scroll_distance(&self) -> gpui::Pixels {
        let viewport_height = self.list_state.viewport_bounds().size.height;
        if viewport_height > px(0.0) {
            viewport_height * 0.9
        } else {
            px(self.row_scroll_increment(&self.row_layout(1.0)))
        }
    }

    pub(super) fn emit_scroll_changed_if_needed(&mut self, cx: &mut Context<Self>) -> bool {
        let top_index = self.list_state.logical_scroll_top().item_ix;
        if self.emitted_scroll_top_index == top_index {
            return false;
        }

        self.emitted_scroll_top_index = top_index;
        cx.emit(TableEvent::ScrollChanged { top_index });
        true
    }
}
