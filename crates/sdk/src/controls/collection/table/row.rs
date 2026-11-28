use gpui::{AnyElement, Stateful, div, prelude::*, px};

use super::styled::TableRowLayout;
use crate::style::PartRead;

/// Render SDK row chrome from a row's resolved style (its `row` part and the table's properties, #433) and
/// layout, without selection or input bindings.
/// `is_custom` delegates row background/padding to the supplied content; otherwise
/// SDK row styling is applied. `fill_row_height` fixes the border-box row height.
/// This presentation helper also supports forced-state template previews.
#[allow(clippy::too_many_arguments)] // A public presentation helper: each argument is one part of the row.
pub fn render_table_row(
    id: impl Into<gpui::ElementId>,
    content: AnyElement,
    style: &xgpui_luma_style::Resolved,
    layout: TableRowLayout,
    enabled: bool,
    show_top_divider: bool,
    is_custom: bool,
    fill_row_height: Option<f32>,
) -> Stateful<gpui::Div> {
    let (root, part) = (style.root(), style.part(xgpui_luma_style::PartId::new("row")));
    let typography = part.typography(14.0, 20.0);
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
            .px(px(layout.padding_x))
            .py(px(layout.padding_y))
            .bg(part.color("background"))
            .text_color(part.color("foreground"))
            .text_size(px(typography.size))
            .line_height(px(typography.line_height))
            .font_weight(typography.weight);
        if fill_row_height.is_none() {
            body = body.min_h(px(layout.min_height));
        }
        row = body.child(div().flex_1().min_w(px(0.0)).mt(px(root.number("label_baseline_shift"))).child(content));
    }

    if show_top_divider {
        row = row.border_t(px(root.number("row_divider_width"))).border_color(part.color("border"));
    }

    if enabled {
        row = row.cursor_pointer();
    }
    row = crate::style::fade(row, part.opacity());

    row
}
