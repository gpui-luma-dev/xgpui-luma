//! Shared fixed-column grids and label chrome for style-guide matrices.

use gpui::{AnyElement, Entity, FontWeight, Hsla, IntoElement, SharedString, div, prelude::*, px};
use xgpui_luma::controls::tabs::Tabs;
use xgpui_luma::{GridLayout, GridTrack};

/// Header band height for corner + column labels.
pub const HEADER_HEIGHT: f32 = 36.0;
/// Default gutter between data columns (and often between rows).
pub const COL_GAP: f32 = 15.0;

/// Tabs list, under-tabs hairline, then centered body — shared section preview chrome.
pub fn preview_tabbed(preview_tabs: Entity<Tabs>, border: Hsla, body: AnyElement) -> AnyElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .child(div().w_full().flex().justify_start().child(preview_tabs))
        .child(div().w_full().h(px(1.0)).bg(border))
        .child(div().w_full().flex().justify_center().mt(px(16.0)).child(body))
        .into_any_element()
}

/// Label column + `n` equal data columns.
pub fn equal_data_columns(label_width: f32, data_width: f32, n: usize) -> Vec<GridTrack> {
    let mut columns = vec![GridTrack::Px(label_width)];
    columns.extend(std::iter::repeat(GridTrack::Px(data_width)).take(n));
    columns
}

pub fn fixed_grid(columns: Vec<GridTrack>, gap_x: f32, gap_y: f32) -> GridLayout {
    GridLayout::new().columns(columns).gap_x(gap_x).gap_y(gap_y)
}

pub fn corner_label(label: impl Into<SharedString>, muted: Hsla) -> AnyElement {
    header_label(label, muted, Justify::Start)
}

pub fn column_header(label: impl Into<SharedString>, muted: Hsla) -> AnyElement {
    header_label(label, muted, Justify::Center)
}

pub fn empty_corner() -> AnyElement {
    div().w_full().h(px(HEADER_HEIGHT)).into_any_element()
}

pub fn row_label(label: impl Into<SharedString>, fg: Hsla) -> AnyElement {
    div()
        .w_full()
        .min_w(px(0.0))
        .pr(px(8.0))
        .flex()
        .items_center()
        .text_xs()
        .line_height(px(15.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(fg)
        .child(div().w_full().min_w(px(0.0)).truncate().child(label.into()))
        .into_any_element()
}

enum Justify {
    Start,
    Center,
}

fn header_label(label: impl Into<SharedString>, muted: Hsla, justify: Justify) -> AnyElement {
    let mut cell = div()
        .w_full()
        .h(px(HEADER_HEIGHT))
        .flex()
        .items_end()
        .pb(px(8.0))
        .text_xs()
        .line_height(px(15.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(muted)
        .child(label.into());
    cell = match justify {
        Justify::Start => cell.justify_start(),
        Justify::Center => cell.justify_center(),
    };
    cell.into_any_element()
}

/// Centers a cell's content in a full-width flex wrapper.
pub fn centered(content: impl IntoElement) -> AnyElement {
    div().w_full().flex().items_center().justify_center().child(content).into_any_element()
}
