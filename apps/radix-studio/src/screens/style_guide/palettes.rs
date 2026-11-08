//! Named palette readout: the theme's color and gray 12-step rows.

use std::sync::Arc;

use gpui::{AnyElement, FontWeight, Hsla, IntoElement, SharedString, div, prelude::*, px};
use xgpui_luma::{GridLayout, GridTrack, hstack, vstack};
use xgpui_luma_look_radix::{Look, SCALE_LEN, ScaleFamily};

/// Gutter between shade cells (matches Custom Palette).
const SCALE_SWATCH_GAP: f32 = 3.0;
const SWATCH_HEIGHT: f32 = 44.0;
const ROW_LABEL_WIDTH: f32 = 72.0;
const STEP_ROW_HEIGHT: f32 = 15.0;
/// text_xs line + gap + underline bar, matching [`legend_group`].
const LEGEND_ROW_HEIGHT: f32 = 21.0;

const SCALE_LEGEND_GROUPS: &[(&str, usize)] = &[
    ("Backgrounds", 2),
    ("Interactive components", 3),
    ("Borders and separators", 3),
    ("Solid colors", 2),
    ("Accessible text", 2),
];

pub fn indicator(look: &Arc<Look>, muted: Hsla) -> AnyElement {
    let accent_name = title_case(look.palette_label(ScaleFamily::Color));
    let gray_name = title_case(look.palette_label(ScaleFamily::Gray));

    let columns = vec![GridTrack::Star(1.0); SCALE_LEN];
    let mut grid = GridLayout::new().columns(columns).gap_x(SCALE_SWATCH_GAP).gap_y(SCALE_SWATCH_GAP);

    let mut col = 0usize;
    for &(label, span) in SCALE_LEGEND_GROUPS {
        grid = grid.child_with_span(legend_group(label, muted), 0, col, span);
        col += span;
    }

    for step in 1..=SCALE_LEN as u8 {
        grid = grid.child(scale_step_number(step, muted), 1, (step - 1) as usize);
    }
    for step in 1..=SCALE_LEN as u8 {
        grid = grid.child(scale_swatch(look, ScaleFamily::Color, step), 2, (step - 1) as usize);
    }
    for step in 1..=SCALE_LEN as u8 {
        grid = grid.child(scale_swatch(look, ScaleFamily::Gray, step), 3, (step - 1) as usize);
    }

    hstack! {
        gap=10 align=start;
        row_labels(&accent_name, &gray_name, muted),
        div().flex_1().min_w(px(0.0)).child(grid.into_element()),
    }
    .w_full()
    .into_any_element()
}

fn row_labels(accent: &str, gray: &str, muted: Hsla) -> AnyElement {
    // Align with legend, step numbers, then the two swatch rows.
    div()
        .w(px(ROW_LABEL_WIDTH))
        .flex()
        .flex_col()
        .gap(px(SCALE_SWATCH_GAP))
        .child(div().h(px(LEGEND_ROW_HEIGHT)))
        .child(div().h(px(STEP_ROW_HEIGHT)))
        .child(row_label(accent, muted))
        .child(row_label(gray, muted))
        .into_any_element()
}

fn row_label(name: &str, muted: Hsla) -> AnyElement {
    div()
        .h(px(SWATCH_HEIGHT))
        .flex()
        .items_center()
        .text_xs()
        .line_height(px(15.0))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(muted)
        .child(SharedString::from(name.to_owned()))
        .into_any_element()
}

fn legend_group(label: &'static str, muted: Hsla) -> impl IntoElement {
    vstack! {
        gap=4 align=center;
        div()
            .w_full()
            .text_xs()
            .font_weight(FontWeight::MEDIUM)
            .text_color(muted)
            .text_center()
            .truncate()
            .child(label),
        div().w_full().h(px(2.0)).rounded_full().bg(muted.opacity(0.45)),
    }
    .w_full()
}

fn scale_swatch(look: &Look, family: ScaleFamily, step: u8) -> impl IntoElement {
    let color = look.resolve_step(family, step).hsla();
    div()
        .id(SharedString::from(format!("style-guide-swatch-{}-{step}", family.as_str())))
        .w_full()
        .h(px(SWATCH_HEIGHT))
        .bg(color)
}

fn scale_step_number(step: u8, muted: Hsla) -> impl IntoElement {
    hstack! {
        justify=center;
        div()
            .text_xs()
            .font_weight(FontWeight::MEDIUM)
            .text_color(muted)
            .child(format!("{step}")),
    }
    .w_full()
}

pub fn title_case(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
