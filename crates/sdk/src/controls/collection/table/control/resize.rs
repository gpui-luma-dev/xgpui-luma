use crate::infra::locale::{LayoutDirection, LocaleContext, LogicalStyled};
use gpui::{
    AnyElement, Context, DragMoveEvent, EntityId, MouseButton, MouseDownEvent, MouseUpEvent, Render, Window, div,
    prelude::*, px,
};

use super::super::model::{TableColumnWidth, render_grid_view_header_column_slot};
use super::super::theme::TableLook;
use super::TableControl;

const MIN_COLUMN_WIDTH: f32 = 48.0;

pub(super) struct ColumnResize {
    index: usize,
    direction: LayoutDirection,
    start_x: f32,
    viewport_width: f32,
    widths: Vec<f32>,
    original: Vec<(TableColumnWidth, f32)>,
}

#[derive(Clone)]
pub(super) struct ColumnResizeDrag(EntityId);

impl Render for ColumnResizeDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }
}

impl<T: 'static> TableControl<T> {
    pub(super) fn sync_column_resize_direction(&mut self, cx: &mut Context<Self>) {
        if self
            .column_resize
            .as_ref()
            .is_some_and(|resize| resize.direction != LocaleContext::current(cx).direction)
        {
            self.cancel_column_resize(cx);
        }
    }

    pub(super) fn render_grid_header(&self, look: &TableLook, cx: &mut Context<Self>) -> AnyElement {
        let direction = LocaleContext::current(cx).direction;
        let mut row = div()
            .w_full()
            .flex()
            .flex_row_logical(direction)
            .items_center()
            .when(direction == LayoutDirection::Rtl, |row| row.text_right());
        for (index, column) in self.model.columns.iter().enumerate() {
            let slot = render_grid_view_header_column_slot(
                column.width(),
                column.fill_weight,
                column.cell_layout(),
                look.column_padding_x,
                column.header().clone().into_any_element(),
            )
            .id(("table-column", index));
            #[cfg(any(test, feature = "test-support"))]
            let slot = slot.debug_selector(move || format!("table-column-{index}"));
            row = row.child(slot);
        }
        let owner = cx.entity().downgrade();
        let row = row.on_children_prepainted(move |bounds, _, cx| {
            let changed = owner
                .update(cx, |table, _| {
                    let widths = bounds.iter().map(|bounds| bounds.size.width.as_f32()).collect::<Vec<_>>();
                    let changed = table.measured_column_widths != widths;
                    table.measured_column_widths = widths;
                    changed && table.model.column_resizing
                })
                .unwrap_or(false);
            if changed {
                let owner = owner.clone();
                cx.defer(move |cx| {
                    let _ = owner.update(cx, |_, cx| cx.notify());
                });
            }
        });
        // Paint handles above all labels, outside the column slots. A zero-width
        // column must not clip its own resize handle or let labels cover it.
        let mut header = div().w_full().relative().child(row);
        let mut divider_x = 0.0;
        for (index, width) in self.measured_column_widths.iter().enumerate() {
            divider_x += width;
            if self.can_resize_column(index) {
                let drag = ColumnResizeDrag(cx.entity_id());
                let active = self.column_resize.as_ref().is_some_and(|resize| resize.index == index);
                let handle = div().id(("table-column-resize", index));
                #[cfg(any(test, feature = "test-support"))]
                let handle = handle.debug_selector(move || format!("table-column-resize-{index}"));
                header = header.child(
                    handle
                        .absolute()
                        // Center the hit target on the divider so boundary
                        // clicks work from either visual direction.
                        .start(px(divider_x - look.column_resize_hit_width * 0.5), direction)
                        .top(px(-look.column_resize_vertical_extension))
                        .bottom(px(-look.column_resize_vertical_extension))
                        .w(px(look.column_resize_hit_width))
                        .cursor_col_resize()
                        .group("table-column-resize")
                        .child(
                            div()
                                .absolute()
                                .left(px((look.column_resize_hit_width - look.column_resize_line_width) * 0.5))
                                .top(px(look.column_resize_vertical_extension))
                                .bottom(px(look.column_resize_vertical_extension))
                                .w(px(look.column_resize_line_width))
                                .bg(if active { look.header_label_color } else { look.border })
                                .group_hover("table-column-resize", |style| style.bg(look.header_label_color)),
                        )
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |table, event, window, cx| {
                                table.start_column_resize(index, event, window, cx);
                            }),
                        )
                        .on_drag(drag, |drag, _, _, cx| {
                            cx.stop_propagation();
                            cx.new(|_| drag.clone())
                        }),
                );
            }
        }
        let owner = cx.entity().downgrade();
        header
            .on_children_prepainted(move |bounds, _, cx| {
                let Some(header) = bounds.first() else { return };
                let width = header.size.width.as_f32();
                let cancel = owner
                    .update(cx, |table, _| {
                        // Measure the viewport itself: total column width can change
                        // during a valid drag when previously overflowing columns fit.
                        let changed = table
                            .column_resize
                            .as_ref()
                            .is_some_and(|resize| (resize.viewport_width - width).abs() > 1.0);
                        table.measured_header_width = width;
                        changed
                    })
                    .unwrap_or(false);
                if cancel {
                    // Apply after this frame so header and rows use the same geometry.
                    let owner = owner.clone();
                    cx.defer(move |cx| {
                        let _ = owner.update(cx, |table, cx| table.cancel_column_resize(cx));
                    });
                }
            })
            .into_any_element()
    }

    fn can_resize_column(&self, index: usize) -> bool {
        self.model.enabled
            && self.model.column_resizing
            && self.model.grid_header
            && self.model.columns.get(index).is_some_and(|column| column.resizable)
            && self.model.columns.get(index + 1).is_some_and(|column| column.resizable)
    }

    fn start_column_resize(
        &mut self,
        index: usize,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.can_resize_column(index)
            || self.measured_column_widths.len() != self.model.columns.len()
            || self.measured_column_widths.iter().any(|width| !width.is_finite() || *width < 0.0)
        {
            return;
        }
        self.cancel_column_resize(cx);
        self.focus_handle.focus(window, cx);
        self.column_resize = Some(ColumnResize {
            index,
            direction: LocaleContext::current(cx).direction,
            start_x: event.position.x.as_f32(),
            viewport_width: self.measured_header_width,
            widths: self.measured_column_widths.clone(),
            original: self.model.columns.iter().map(|column| (column.width, column.fill_weight)).collect(),
        });
        cx.stop_propagation();
        cx.notify();
    }

    pub(super) fn handle_column_resize_move(
        &mut self,
        event: &DragMoveEvent<ColumnResizeDrag>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.drag(cx).0 != cx.entity_id() {
            return;
        }
        let Some(resize) = self.column_resize.as_ref() else {
            return;
        };
        if resize.direction != LocaleContext::current(cx).direction {
            self.cancel_column_resize(cx);
            return;
        }
        let index = resize.index;
        let left = resize.widths[index];
        let right = resize.widths[index + 1];
        // Already narrow columns may grow, but cannot be made narrower.
        let sign = if resize.direction == LayoutDirection::Rtl {
            -1.0
        } else {
            1.0
        };
        let delta = ((event.event.position.x.as_f32() - resize.start_x) * sign)
            .clamp(MIN_COLUMN_WIDTH.min(left) - left, right - MIN_COLUMN_WIDTH.min(right));
        for (i, column) in self.model.columns.iter_mut().enumerate() {
            let width = resize.widths[i]
                + if i == index {
                    delta
                } else if i == index + 1 {
                    -delta
                } else {
                    0.0
                };
            match column.width {
                TableColumnWidth::Fixed(_) if i == index || i == index + 1 => {
                    column.width = TableColumnWidth::Fixed(width);
                }
                // Capture all fill ratios so other fill columns do not move while
                // this pair is resized. They still share space on viewport resize.
                // A fill column can collapse when fixed columns exhaust the
                // viewport. Keep its weight positive so it recovers with space.
                TableColumnWidth::Fill => column.fill_weight = width.max(1.0),
                _ => {}
            }
        }
        self.list_state.remeasure();
        cx.stop_propagation();
        cx.notify();
    }

    pub(super) fn finish_column_resize(&mut self, _: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.column_resize.take().is_some() {
            cx.stop_propagation();
            cx.notify();
        }
    }

    pub(super) fn cancel_column_resize(&mut self, cx: &mut Context<Self>) {
        if let Some(resize) = self.column_resize.take() {
            for (column, (width, weight)) in self.model.columns.iter_mut().zip(resize.original) {
                column.width = width;
                column.fill_weight = weight;
            }
            self.list_state.remeasure();
            cx.notify();
        }
    }
}
