use gpui::{AnyElement, FontWeight, Hsla, IntoElement, SharedString, div, prelude::*, px};
use xgpui_luma::theme::LumaChrome;

const DEFAULT_VARIANT_COLUMN_WIDTH: f32 = 188.0;
const DEFAULT_STATE_COLUMN_WIDTH: f32 = 92.0;
const DEFAULT_HEADER_HEIGHT: f32 = 48.0;
const DEFAULT_ROW_HEIGHT: f32 = 52.0;
const DEFAULT_CORNER_RADIUS: f32 = 10.0;

#[derive(Clone, Debug)]
pub(crate) struct VariantStateTableStyle {
    pub variant_column_width: f32,
    pub state_column_width: f32,
    pub header_height: f32,
    pub row_height: f32,
    pub corner_radius: f32,
    pub variant_column_align_center: bool,
    pub header_corner_padding_bottom: f32,
    pub border: Hsla,
    pub title_color: Hsla,
    pub muted_text: Hsla,
}

impl VariantStateTableStyle {
    pub(crate) fn from_chrome(chrome: &LumaChrome) -> Self {
        Self {
            variant_column_width: DEFAULT_VARIANT_COLUMN_WIDTH,
            state_column_width: DEFAULT_STATE_COLUMN_WIDTH,
            header_height: DEFAULT_HEADER_HEIGHT,
            row_height: DEFAULT_ROW_HEIGHT,
            corner_radius: DEFAULT_CORNER_RADIUS,
            variant_column_align_center: false,
            header_corner_padding_bottom: 8.0,
            border: chrome.border,
            title_color: chrome.title_text,
            muted_text: chrome.muted_text,
        }
    }

    pub(crate) fn row_height(mut self, row_height: f32) -> Self {
        self.row_height = row_height;
        self
    }

    pub(crate) fn header_height(mut self, header_height: f32) -> Self {
        self.header_height = header_height;
        self
    }

    pub(crate) fn state_column_width(mut self, width: f32) -> Self {
        self.state_column_width = width;
        self
    }

    pub(crate) fn header_corner_padding_bottom(mut self, padding: f32) -> Self {
        self.header_corner_padding_bottom = padding;
        self
    }

    pub(crate) fn variant_column_width(mut self, width: f32) -> Self {
        self.variant_column_width = width;
        self
    }
}

pub(crate) struct VariantStateTableRow {
    pub label: SharedString,
    pub description: SharedString,
    pub cells: Vec<AnyElement>,
}

pub(crate) struct VariantStateTable {
    style: VariantStateTableStyle,
    row_group_label: SharedString,
    column_headers: Vec<AnyElement>,
    rows: Vec<VariantStateTableRow>,
}

impl VariantStateTable {
    pub(crate) fn new(style: VariantStateTableStyle) -> Self {
        Self { style, row_group_label: SharedString::from("VARIANTS"), column_headers: Vec::new(), rows: Vec::new() }
    }

    pub(crate) fn row_group_label(mut self, label: impl Into<SharedString>) -> Self {
        self.row_group_label = label.into();
        self
    }

    pub(crate) fn column_headers(mut self, headers: impl IntoIterator<Item = AnyElement>) -> Self {
        self.column_headers = headers.into_iter().collect();
        self
    }

    pub(crate) fn rows(mut self, rows: impl IntoIterator<Item = VariantStateTableRow>) -> Self {
        self.rows = rows.into_iter().collect();
        self
    }

    pub(crate) fn build(self) -> AnyElement {
        let Self { style, row_group_label, column_headers, rows } = self;
        let border = table_border_color(&style);
        let width = table_width(&style, column_headers.len());

        let header_row = render_header_row(&style, &row_group_label, column_headers, border);
        let row_count = rows.len();
        let data_rows = rows
            .into_iter()
            .enumerate()
            .map(|(index, row)| render_variant_row(&style, row, index == row_count.saturating_sub(1), border));

        div()
            .w(px(width))
            .max_w_full()
            .flex_shrink_0()
            .border_1()
            .border_color(border)
            .rounded(px(style.corner_radius))
            .overflow_hidden()
            .child(header_row)
            .children(data_rows)
            .into_any_element()
    }
}

fn table_width(style: &VariantStateTableStyle, column_count: usize) -> f32 {
    style.variant_column_width + style.state_column_width * column_count as f32
}

fn table_border_color(style: &VariantStateTableStyle) -> Hsla {
    gpui::hsla(style.border.h, style.border.s, style.border.l, 0.0)
}

fn render_header_row(
    style: &VariantStateTableStyle,
    row_group_label: &SharedString,
    column_headers: Vec<AnyElement>,
    border: Hsla,
) -> AnyElement {
    let column_count = column_headers.len();
    let mut row = div().flex().items_stretch();
    row = row.child(table_cell(
        style.variant_column_width,
        style.header_height,
        border,
        true,
        true,
        div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .justify_end()
            .items_start()
            .px(px(16.0))
            .pb(px(style.header_corner_padding_bottom))
            .text_xs()
            .line_height(px(15.0))
            .font_weight(FontWeight::MEDIUM)
            .text_color(style.muted_text)
            .child(row_group_label.clone()),
    ));

    for (index, header) in column_headers.into_iter().enumerate() {
        row = row.child(table_cell(
            style.state_column_width,
            style.header_height,
            border,
            index < column_count.saturating_sub(1),
            true,
            header,
        ));
    }

    row.into_any_element()
}

fn render_variant_row(
    style: &VariantStateTableStyle,
    row: VariantStateTableRow,
    is_last_row: bool,
    border: Hsla,
) -> AnyElement {
    let cell_count = row.cells.len();
    let mut table_row = div().flex().items_stretch();
    table_row = table_row.child(table_cell(
        style.variant_column_width,
        style.row_height,
        border,
        true,
        !is_last_row,
        render_variant_column_cell(style, &row),
    ));

    for (index, cell) in row.cells.into_iter().enumerate() {
        table_row = table_row.child(table_cell(
            style.state_column_width,
            style.row_height,
            border,
            index < cell_count.saturating_sub(1),
            !is_last_row,
            div().w_full().h_full().flex().items_center().justify_center().child(cell),
        ));
    }

    table_row.into_any_element()
}

fn render_variant_column_cell(style: &VariantStateTableStyle, row: &VariantStateTableRow) -> AnyElement {
    let has_description = !row.description.is_empty();
    let label = div()
        .text_sm()
        .line_height(px(18.0))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(style.title_color)
        .child(row.label.clone());
    let description = has_description
        .then(|| div().text_xs().line_height(px(15.0)).text_color(style.muted_text).child(row.description.clone()));

    if style.variant_column_align_center {
        let mut stack = div().flex().flex_col().items_center().justify_center().gap(px(2.0)).child(label);
        if let Some(description) = description {
            stack = stack.child(description);
        }
        return div()
            .w_full()
            .h_full()
            .flex()
            .items_center()
            .justify_center()
            .px(px(16.0))
            .py(px(4.0))
            .child(stack)
            .into_any_element();
    }

    let mut stack = div().px(px(16.0)).py(px(8.0)).flex().flex_col().justify_center().gap(px(2.0)).child(label);
    if let Some(description) = description {
        stack = stack.child(description);
    }
    stack.into_any_element()
}

fn table_cell(
    width: f32,
    min_height: f32,
    border: Hsla,
    bordered_right: bool,
    bordered_bottom: bool,
    content: impl IntoElement,
) -> AnyElement {
    let mut cell = div().w(px(width)).min_h(px(min_height)).flex().items_stretch().child(content);

    if bordered_right {
        cell = cell.border_r_1().border_color(border);
    }
    if bordered_bottom {
        cell = cell.border_b_1().border_color(border);
    }

    cell.into_any_element()
}
