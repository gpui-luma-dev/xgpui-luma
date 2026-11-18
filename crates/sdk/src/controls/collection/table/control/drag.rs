use web_time::{Duration, Instant};
use gpui::{
    Axis, Bounds, Context, CursorStyle, Div, IntoElement, ListOffset, Pixels, Point, Render, SharedString, Stateful,
    Window, div, prelude::*, px,
};
use crate::infra::drag_drop::{DropEdge, DropZone, KeyedDropTarget, bind_drag_source};
use super::{TableControl, reorder::RowDrag};
use super::super::theme::TableDragLook;

struct RowPreview {
    label: SharedString,
    offset: Point<Pixels>,
    look: TableDragLook,
}
impl Render for RowPreview {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        // GPUI positions the preview at pointer - native grab offset. Compensate
        // so a compact preview stays beside the pointer even on very wide rows.
        div().relative().child(
            div()
                .debug_selector(|| "table-row-drag-preview".to_string())
                .relative()
                .left(self.offset.x + px(self.look.geometry.drag_offset))
                .top(self.offset.y + px(self.look.geometry.drag_offset))
                .w(px(self.look.geometry.drag_width))
                .px(px(self.look.geometry.drag_padding_x))
                .py(px(self.look.geometry.drag_padding_y))
                .rounded(px(self.look.radius))
                .bg(self.look.background)
                .text_color(self.look.foreground)
                .border(px(self.look.geometry.drag_border_width))
                .border_color(self.look.valid_marker)
                .truncate()
                .child(self.label.clone()),
        )
    }
}

impl<T: 'static> TableControl<T> {
    pub(super) fn bind_row_drag(&self, mut row: Stateful<Div>, index: usize, cx: &mut Context<Self>) -> Stateful<Div> {
        let scope = cx.entity_id();
        if let Some(drag) = self.row_drag(index, scope) {
            let owner = cx.entity().downgrade();
            let label = if drag.keys().len() == 1 {
                self.model.label_for_row(&self.model.items[index])
            } else {
                format!("{} rows", drag.keys().len()).into()
            };
            let look = self.model.theme.resolve_drag();
            row = bind_drag_source(
                row,
                drag,
                move |drag, offset, window, app| {
                    let _ = owner.update(app, |table, cx| {
                        table.drop_layout.cancel();
                        table.active_row_drag = Some(drag.clone());
                        table.schedule_drag_scroll(window, cx, None);
                        cx.notify();
                    });
                    app.new(|_| RowPreview { label: label.clone(), offset, look: look.clone() })
                },
                cx,
                Self::handle_row_drag_event,
            );
        }
        if self.active_row_drag.as_ref().is_some_and(|drag| drag.accepts(scope)) {
            for (edge, gap) in [(DropEdge::Before, index), (DropEdge::After, index + 1)] {
                let before = self.row_keys.get(gap).cloned();
                let look = self.model.theme.resolve_drag();
                let zone = DropZone {
                    axis: Axis::Vertical,
                    edge,
                    item_extent: px(0.0),
                    following_gap: px(0.0),
                    marker_width: px(look.geometry.drag_marker_width),
                };
                let owner = cx.entity().downgrade();
                let highlight_before = before.clone();
                let cursor_before = before.clone();
                let surface = zone
                    .apply_measured(div().id(if edge == DropEdge::Before {
                        "row-insert-before"
                    } else {
                        "row-insert-after"
                    }))
                    .border_color(gpui::transparent_black())
                    .on_drag_move(cx.listener(move |table, event: &gpui::DragMoveEvent<RowDrag>, window, cx| {
                        if event.bounds.contains(&event.event.position)
                            && table.list_state.viewport_bounds().contains(&event.event.position)
                        {
                            let valid = event
                                .drag(cx)
                                .propose(scope, scope, cursor_before.clone())
                                .is_some_and(|proposal| table.drop_changes_order(&proposal, scope));
                            cx.set_active_drag_cursor_style(
                                if valid {
                                    CursorStyle::ClosedHand
                                } else {
                                    CursorStyle::OperationNotAllowed
                                },
                                window,
                            );
                        }
                    }))
                    .drag_over::<RowDrag>(move |style, drag, window, app| {
                        let valid = owner.upgrade().is_some_and(|owner| {
                            let table = owner.read(app);
                            drag.propose(scope, scope, highlight_before.clone())
                                .is_some_and(|proposal| table.drop_changes_order(&proposal, scope))
                        });
                        let cursor = if valid {
                            CursorStyle::ClosedHand
                        } else {
                            CursorStyle::OperationNotAllowed
                        };
                        // GPUI temporarily removes its active drag while resolving
                        // drag-over styles. Update the cursor after restoring it.
                        let pointer = window.mouse_position();
                        let session = drag.clone();
                        window.on_next_frame(move |window, app| {
                            if session.accepts(scope)
                                && window.mouse_position() == pointer
                                && app.has_active_drag()
                                && app.active_drag_cursor_style() != Some(cursor)
                            {
                                app.set_active_drag_cursor_style(cursor, window);
                            }
                        });
                        style.border_color(if valid { look.valid_marker } else { look.invalid_marker })
                    });
                row = row.child(KeyedDropTarget::new(scope, scope, before).bind(surface, cx, Self::handle_row_drop));
            }
        }
        row
    }

    pub(super) fn handle_drag_move(
        &mut self,
        event: &gpui::DragMoveEvent<RowDrag>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !event.drag(cx).accepts(cx.entity_id()) {
            return;
        }
        // Capture runs on the shell before gap listeners. Start with rejection
        // so whitespace and headers never retain a previous valid cursor.
        if cx.active_drag_cursor_style() != Some(CursorStyle::OperationNotAllowed) {
            cx.set_active_drag_cursor_style(CursorStyle::OperationNotAllowed, window);
        }
        self.schedule_drag_scroll(window, cx, None);
    }

    fn schedule_drag_scroll(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
        previous: Option<(Instant, ListOffset, Point<Pixels>)>,
    ) {
        if self.drag_scroll_scheduled || self.is_paged() {
            return;
        }
        self.drag_scroll_scheduled = true;
        let owner = cx.entity().downgrade();
        window.on_next_frame(move |window, app| {
            let _ = owner.update(app, |table, cx| {
                table.drag_scroll_scheduled = false;
                if !cx.has_active_drag()
                    || !window.is_window_hovered()
                    || !table.active_row_drag.as_ref().is_some_and(|drag| drag.accepts(cx.entity_id()))
                {
                    return;
                }
                let now = cx.background_executor().now();
                let offset = table.list_state.logical_scroll_top();
                if previous.as_ref().is_some_and(|(_, last, pointer)| {
                    *pointer == window.mouse_position()
                        && last.item_ix == offset.item_ix
                        && last.offset_in_item == offset.offset_in_item
                }) {
                    return;
                }
                let elapsed = previous.map_or(Duration::from_secs_f32(1.0 / 60.0), |(last, _, _)| now - last);
                let delta = edge_scroll_delta(table.list_state.viewport_bounds(), window.mouse_position(), elapsed);
                if delta == px(0.0) {
                    return;
                }
                table.list_state.scroll_by(delta);
                table.emit_scroll_changed_if_needed(cx);
                cx.notify();
                table.schedule_drag_scroll(window, cx, Some((now, offset, window.mouse_position())));
            });
        });
    }
}

fn edge_scroll_delta(bounds: Bounds<Pixels>, pointer: Point<Pixels>, elapsed: Duration) -> Pixels {
    if !bounds.contains(&pointer) {
        return px(0.0);
    }
    let extent = bounds.size.height.as_f32();
    let edge = 32.0_f32.min(extent / 4.0);
    if edge <= 0.0 {
        return px(0.0);
    }
    let position = (pointer.y - bounds.top()).as_f32();
    let leading = ((edge - position) / edge).clamp(0.0, 1.0);
    let trailing = ((edge - (extent - position)) / edge).clamp(0.0, 1.0);
    px((trailing - leading) * 540.0 * elapsed.as_secs_f32().min(0.05))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{point, size};
    #[test]
    fn edge_scrolling_is_bounded_and_directional() {
        let bounds = Bounds::new(point(px(0.0), px(0.0)), size(px(200.0), px(200.0)));
        let step = |x, y, millis| edge_scroll_delta(bounds, point(px(x), px(y)), Duration::from_millis(millis));
        assert_eq!(step(-1.0, 199.0, 20), px(0.0));
        assert_eq!(step(100.0, 100.0, 20), px(0.0));
        assert!(step(100.0, 1.0, 20) < px(0.0));
        assert!(step(100.0, 199.0, 20) > step(100.0, 180.0, 20));
        assert_eq!(step(100.0, 199.0, 10) * 2.0, step(100.0, 199.0, 20));
        assert_eq!(step(100.0, 199.0, 1000), step(100.0, 199.0, 50));
    }
}
