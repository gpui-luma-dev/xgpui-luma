//! Badge matrices: variants/contrast, named palettes, and all sizes.

use gpui::{AnyElement, App, Entity, Hsla, IntoElement, div, prelude::*, px};
use xgpui_luma::controls::tabs::Tabs;
use xgpui_luma_look_radix::{Accent, Badge, BadgeSize, BadgeVariant, Gray, Look, Tone};
use super::matrix_grid::{column_header, empty_corner, equal_data_columns, fixed_grid, preview_tabbed, row_label};

pub fn tabbed(look: &Look, navigation: Entity<Tabs>, muted: Hsla, border: Hsla, cx: &App) -> AnyElement {
    let active = navigation.read(cx).active_id().map(|id| id.as_ref()).unwrap_or("template-preview");
    let body = match active {
        "colors" => colors(look, muted),
        "all-sizes" => sizes(look, muted),
        _ => variants(look, muted),
    };
    preview_tabbed(navigation, border, body)
}

fn sample(look: &Look, variant: BadgeVariant, tone: Tone, high: bool, size: BadgeSize) -> AnyElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .child(Badge::new(look, "New").variant(variant).tone(tone).high_contrast(high).size(size))
        .into_any_element()
}

fn variants(look: &Look, muted: Hsla) -> AnyElement {
    let mut grid = fixed_grid(equal_data_columns(100.0, 64.0, 4), 16.0, 16.0)
        .child(empty_corner(), 0, 0)
        .child_with_span(column_header("Accent", muted), 0, 1, 2)
        .child_with_span(column_header("Gray", muted), 0, 3, 2);
    for (row, variant) in BadgeVariant::ALL.into_iter().enumerate() {
        grid = grid.child(row_label(variant.label(), muted), row + 1, 0);
        for (column, (tone, high)) in
            [(Tone::Accent, false), (Tone::Accent, true), (Tone::Gray, false), (Tone::Gray, true)]
                .into_iter()
                .enumerate()
        {
            grid = grid.child(sample(look, variant, tone, high, BadgeSize::One), row + 1, column + 1);
        }
    }
    grid.into_any_element()
}

fn colors(look: &Look, muted: Hsla) -> AnyElement {
    let mut grid = fixed_grid(equal_data_columns(88.0, 128.0, 4), 16.0, 16.0).child(empty_corner(), 0, 0);
    for (column, variant) in BadgeVariant::ALL.into_iter().enumerate() {
        grid = grid.child(column_header(variant.label(), muted), 0, column + 1);
    }
    for (row, accent) in Accent::ALL.iter().enumerate() {
        let palette = look.fork();
        palette.set_palettes(*accent, Gray::Auto);
        grid = grid.child(row_label(super::palettes::title_case(accent.as_str()), muted), row + 1, 0);
        for (column, variant) in BadgeVariant::ALL.into_iter().enumerate() {
            grid = grid.child(
                div().flex().items_center().justify_center().gap(px(12.0)).children([
                    sample(&palette, variant, Tone::Accent, false, BadgeSize::One),
                    sample(&palette, variant, Tone::Accent, true, BadgeSize::One),
                ]),
                row + 1,
                column + 1,
            );
        }
    }
    grid.into_any_element()
}

fn sizes(look: &Look, muted: Hsla) -> AnyElement {
    let mut grid = fixed_grid(equal_data_columns(100.0, 110.0, 3), 16.0, 16.0).child(empty_corner(), 0, 0);
    for (column, size) in BadgeSize::ALL.into_iter().enumerate() {
        grid = grid.child(column_header(size.label(), muted), 0, column + 1);
    }
    for (row, variant) in BadgeVariant::ALL.into_iter().enumerate() {
        grid = grid.child(row_label(variant.label(), muted), row + 1, 0);
        for (column, size) in BadgeSize::ALL.into_iter().enumerate() {
            grid = grid.child(sample(look, variant, Tone::Accent, false, size), row + 1, column + 1);
        }
    }
    grid.into_any_element()
}
