//! Card variants and sizes with local profile content.
use gpui::{AnyElement, Hsla, div, prelude::*, px};
use xgpui_luma_look_radix::{CardSize, CardVariant, Look};
use super::matrix_grid::{column_header, empty_corner, equal_data_columns, fixed_grid, row_label};

pub fn matrix(look: &Look, muted: Hsla) -> AnyElement {
    let mut grid = fixed_grid(equal_data_columns(70.0, 310.0, 3), 24.0, 20.0).child(empty_corner(), 0, 0);
    for (column, variant) in CardVariant::ALL.into_iter().enumerate() {
        grid = grid.child(column_header(variant.label(), muted), 0, column + 1);
    }
    for (row, size) in CardSize::ALL.into_iter().enumerate() {
        grid = grid.child(row_label(size.label(), muted), row + 1, 0);
        for (column, variant) in CardVariant::ALL.into_iter().enumerate() {
            // Reserve the ghost's cancelled padding so matrix columns stay aligned.
            let sample = div()
                .when(variant == CardVariant::Ghost, |el| el.p(px(size.padding())))
                .child(super::super::shared::card_samples::profile(look, variant, size));
            grid = grid.child(sample, row + 1, column + 1);
        }
    }
    grid.into_any_element()
}
