use gpui::{AnyElement, Stateful, div, prelude::*, px};

use super::theme::TableRowLook;

/// Render SDK row chrome from a resolved look, without selection or input bindings.
/// `is_custom` delegates row background/padding to the supplied content; otherwise
/// SDK row styling is applied. `fill_row_height` fixes the border-box row height.
/// This presentation helper also supports forced-state template previews.
pub fn render_table_row(
    id: impl Into<gpui::ElementId>,
    content: AnyElement,
    look: TableRowLook,
    enabled: bool,
    show_top_divider: bool,
    is_custom: bool,
    fill_row_height: Option<f32>,
) -> Stateful<gpui::Div> {
    let mut row = div().id(id).relative().w_full();

    if let Some(height) = fill_row_height {
        row = row.h(px(height)).overflow_hidden();
    }

    if is_custom {
        row = row.flex().items_center().child(div().flex_1().min_w(px(0.0)).h_full().child(content));
    } else {
        let mut body = row
            .flex()
            .items_center()
            .px(px(look.padding_x))
            .py(px(look.padding_y))
            .bg(look.background)
            .text_color(look.label_color)
            .text_size(px(look.label_typography.size))
            .line_height(px(look.label_typography.line_height))
            .font_weight(look.label_typography.weight);
        if fill_row_height.is_none() {
            body = body.min_h(px(look.min_height));
        }
        row = body.child(div().flex_1().min_w(px(0.0)).mt(px(look.label_baseline_shift)).child(content));
    }

    if show_top_divider {
        row = row.border_t(px(look.row_divider_width)).border_color(look.divider);
    }

    if enabled {
        row = row.cursor_pointer();
    }
    row = crate::style::fade(row, look.opacity);

    row
}
