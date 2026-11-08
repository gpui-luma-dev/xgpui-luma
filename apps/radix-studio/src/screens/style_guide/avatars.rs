//! Avatar content, palette, and size/radius matrices.
use gpui::{AnyElement, App, Entity, Hsla, div, prelude::*, px};
use xgpui_luma::controls::{button::ControlIcon, tabs::Tabs};
use xgpui_luma_look_radix::{Accent, Avatar, AvatarSize, AvatarVariant, Gray, Look, Radius, Tone};
use super::matrix_grid::{column_header, empty_corner, equal_data_columns, fixed_grid, preview_tabbed, row_label};

pub fn tabbed(look: &Look, navigation: Entity<Tabs>, muted: Hsla, border: Hsla, cx: &App) -> AnyElement {
    let active = navigation.read(cx).active_id().map(|id| id.as_ref()).unwrap_or("template-preview");
    preview_tabbed(
        navigation,
        border,
        match active {
            "colors" => colors(look, muted),
            "all-sizes" => sizes(look, muted),
            _ => variants(look, muted),
        },
    )
}
fn variants(look: &Look, muted: Hsla) -> AnyElement {
    let mut grid = fixed_grid(equal_data_columns(90.0, 40.0, 10), 16.0, 16.0)
        .child(empty_corner(), 0, 0)
        .child_with_span(column_header("Accent", muted), 0, 1, 5)
        .child_with_span(column_header("Gray", muted), 0, 6, 5);
    for (row, variant) in AvatarVariant::ALL.into_iter().enumerate() {
        grid = grid.child(row_label(variant.label(), muted), row + 1, 0);
        for (group, tone) in [Tone::Accent, Tone::Gray].into_iter().enumerate() {
            for column in 0..5 {
                let avatar = Avatar::new(look, if column == 1 { "V" } else { "BG" }).variant(variant).tone(tone);
                let avatar = match column {
                    0 => avatar.image("assets/avatars/portrait.jpg"),
                    3 | 4 => avatar
                        .icon(ControlIcon::SvgPath("assets/react-icons/person.svg".into()))
                        .high_contrast(column == 4),
                    _ => avatar,
                };
                grid = grid.child(avatar, row + 1, 1 + group * 5 + column);
            }
        }
    }
    grid.into_any_element()
}
fn colors(look: &Look, muted: Hsla) -> AnyElement {
    let mut grid = fixed_grid(equal_data_columns(100.0, 140.0, 2), 16.0, 16.0).child(empty_corner(), 0, 0);
    for (column, variant) in AvatarVariant::ALL.into_iter().enumerate() {
        grid = grid.child(column_header(variant.label(), muted), 0, column + 1);
    }
    for (row, accent) in Accent::ALL.into_iter().enumerate() {
        let palette = look.fork();
        palette.set_palettes(accent, Gray::Auto);
        grid = grid.child(row_label(super::palettes::title_case(accent.as_str()), muted), row + 1, 0);
        for (column, variant) in AvatarVariant::ALL.into_iter().enumerate() {
            grid = grid.child(
                div().flex().justify_center().gap(px(16.0)).children(
                    [false, true].map(|high| Avatar::new(&palette, "BG").variant(variant).high_contrast(high)),
                ),
                row + 1,
                column + 1,
            );
        }
    }
    grid.into_any_element()
}
fn sizes(look: &Look, muted: Hsla) -> AnyElement {
    let mut grid = fixed_grid(equal_data_columns(80.0, 160.0, 5), 16.0, 16.0).child(empty_corner(), 0, 0);
    for (column, radius) in Radius::ALL.into_iter().enumerate() {
        grid = grid.child(column_header(radius.label(), muted), 0, column + 1);
    }
    for (row, size) in AvatarSize::ALL.into_iter().enumerate() {
        grid = grid.child(row_label(size.label(), muted), row + 1, 0);
        for (column, radius) in Radius::ALL.into_iter().enumerate() {
            grid = grid.child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(Avatar::new(look, "BG").variant(AvatarVariant::Solid).size(size).radius(radius)),
                row + 1,
                column + 1,
            );
        }
    }
    grid.into_any_element()
}
