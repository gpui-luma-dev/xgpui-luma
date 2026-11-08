//! Progress template previews: values, contrast, radii, colors, and sizes.
use std::sync::Arc;
use gpui::{AnyElement, App, Entity, Hsla, IntoElement, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::progress::{ProgressDirection, ProgressRenderModel};
use xgpui_luma::controls::tabs::Tabs;
use xgpui_luma::infra::value::ControlRange;
use xgpui_luma_look_radix::{Accent, Gray, Look, LookControlExt, Paint, ProgressSize, ProgressVariant, Radius};
use super::matrix_grid::{centered, column_header, empty_corner, equal_data_columns, fixed_grid, preview_tabbed, row_label};

pub fn tabbed(
    look: &Arc<Look>,
    tabs: Entity<Tabs>,
    fg: Hsla,
    muted: Hsla,
    border: Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let active = tabs.read(cx).active_id().cloned().unwrap_or_else(|| "template-preview".into());
    let body = match active.as_ref() {
        "colors" => colors(look, fg, muted, window, cx),
        "all-sizes" => sizes(look, fg, muted, window, cx),
        _ => preview(look, fg, muted, window, cx),
    };
    preview_tabbed(tabs, border, body)
}
struct Sample {
    variant: ProgressVariant,
    size: ProgressSize,
    radius: Radius,
    high_contrast: bool,
    value: f32,
    enabled: bool,
    indeterminate: bool,
}

fn sample(look: &Look, id: String, sample: Sample, window: &mut Window, cx: &mut App) -> AnyElement {
    let Sample { variant, size, radius, high_contrast, value, enabled, indeterminate } = sample;
    let id = SharedString::from(format!("guide-progress-{id}"));
    let template =
        look.progress_template(variant, Paint { tone: xgpui_luma_look_radix::Tone::Accent, high_contrast }, radius);
    let model = ProgressRenderModel {
        id: &id,
        range: ControlRange::from(0..100),
        value,
        percentage: value / 100.0,
        size: size.control_size(),
        enabled,
        direction: ProgressDirection::LeftToRight,
        show_thumb: false,
        indeterminate,
        phase: 0.5,
    };
    centered(
        div()
            .w_full()
            .max_w(px(160.0))
            .h(px(24.0))
            .flex()
            .items_center()
            .child(template.render(&model, window, cx)),
    )
}
fn preview(look: &Look, fg: Hsla, muted: Hsla, window: &mut Window, cx: &mut App) -> AnyElement {
    let columns = [
        ("Empty", 0.0, true, false),
        ("40%", 40.0, true, false),
        ("Complete", 100.0, true, false),
        ("Disabled", 40.0, false, false),
        ("Indeterminate", 0.0, true, true),
    ];
    let mut grid = fixed_grid(equal_data_columns(168.0, 140.0, columns.len()), 16.0, 20.0).child(empty_corner(), 0, 0);
    for (col, (label, ..)) in columns.iter().enumerate() {
        grid = grid.child(column_header(*label, muted), 0, col + 1);
    }
    let mut row = 1;
    for variant in ProgressVariant::ALL {
        for hc in [false, true] {
            grid = grid.child(
                row_label(format!("{}{}", variant.label(), if hc { " · High contrast" } else { "" }), fg),
                row,
                0,
            );
            for (col, (_, value, enabled, indeterminate)) in columns.iter().enumerate() {
                grid = grid.child(
                    sample(
                        look,
                        format!("preview-{row}-{col}"),
                        Sample {
                            variant,
                            size: ProgressSize::Two,
                            radius: Radius::Medium,
                            high_contrast: hc,
                            value: *value,
                            enabled: *enabled,
                            indeterminate: *indeterminate,
                        },
                        window,
                        cx,
                    ),
                    row,
                    col + 1,
                );
            }
            row += 1;
        }
    }
    grid.into_any_element()
}

fn sizes(look: &Look, fg: Hsla, muted: Hsla, window: &mut Window, cx: &mut App) -> AnyElement {
    let mut tables = div().flex().flex_col().gap(px(28.0));
    for variant in ProgressVariant::ALL {
        for high_contrast in [false, true] {
            let mut grid =
                fixed_grid(equal_data_columns(96.0, 160.0, Radius::ALL.len()), 16.0, 12.0).child(empty_corner(), 0, 0);
            for (col, radius) in Radius::ALL.iter().enumerate() {
                grid = grid.child(column_header(radius.label(), muted), 0, col + 1);
            }
            for (row, size) in ProgressSize::ALL.iter().enumerate() {
                grid = grid.child(row_label(size.label(), fg), row + 1, 0);
                let value = match size {
                    ProgressSize::One => 33.0,
                    ProgressSize::Two => 50.0,
                    ProgressSize::Three => 67.0,
                };
                for (col, radius) in Radius::ALL.iter().enumerate() {
                    grid = grid.child(
                        sample(
                            look,
                            format!("size-{}-{high_contrast}-{row}-{col}", variant.as_str()),
                            Sample {
                                variant,
                                size: *size,
                                radius: *radius,
                                high_contrast,
                                value,
                                enabled: true,
                                indeterminate: false,
                            },
                            window,
                            cx,
                        ),
                        row + 1,
                        col + 1,
                    );
                }
            }
            tables = tables.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .child(row_label(
                        format!("{}{}", variant.label(), if high_contrast { " · High contrast" } else { "" }),
                        fg,
                    ))
                    .child(grid),
            );
        }
    }
    tables.into_any_element()
}

fn colors(look: &Look, fg: Hsla, muted: Hsla, window: &mut Window, cx: &mut App) -> AnyElement {
    let mut grid = fixed_grid(equal_data_columns(168.0, 160.0, 4), 16.0, 20.0).child(empty_corner(), 0, 0);
    let variants = [
        (ProgressVariant::Surface, false),
        (ProgressVariant::Surface, true),
        (ProgressVariant::Soft, false),
        (ProgressVariant::Soft, true),
    ];
    for (col, (variant, hc)) in variants.iter().enumerate() {
        grid = grid.child(
            column_header(format!("{}{}", variant.label(), if *hc { " · HC" } else { "" }), muted),
            0,
            col + 1,
        );
    }
    for (row, accent) in Accent::ALL.iter().enumerate() {
        let palette = look.fork();
        palette.set_palettes(*accent, Gray::Auto);
        grid = grid.child(row_label(super::palettes::title_case(accent.as_str()), fg), row + 1, 0);
        for (col, (variant, hc)) in variants.iter().enumerate() {
            grid = grid.child(
                sample(
                    &palette,
                    format!("color-{row}-{col}"),
                    Sample {
                        variant: *variant,
                        size: ProgressSize::Two,
                        radius: Radius::Medium,
                        high_contrast: *hc,
                        value: 40.0,
                        enabled: true,
                        indeterminate: false,
                    },
                    window,
                    cx,
                ),
                row + 1,
                col + 1,
            );
        }
    }
    grid.into_any_element()
}
