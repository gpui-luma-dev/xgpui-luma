//! Colors tab screen — Radix Colors catalog matrix.

use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::color::{ColorValue, gpui_bridge::from_hsla};
use xgpui_luma::controls::button::Button;
use std::sync::Arc;
use crate::controls::swatch_info::{self, ColorDetails, SwatchSelection};
use gpui::{Context, Entity, Hsla, div, hsla, linear_color_stop, linear_gradient, prelude::*, px};
use xgpui_luma::hstack;
use xgpui_luma::vstack;
use xgpui_luma::theme::ThemeMode;
use xgpui_luma_look_radix::{BLACK_ALPHA_STEPS, Look, WHITE_ALPHA_STEPS, color_families, parse_color};

const COLOR_MATRIX_LABEL_W: f32 = 112.0;
const COLOR_MATRIX_CELL_W: f32 = 95.0;
const COLOR_MATRIX_CELL_H: f32 = 50.0;

pub struct State {
    swatches: Vec<Vec<Entity<Button>>>,
    alpha: Vec<Vec<Entity<Button>>>,
}

impl State {
    pub fn new<M: 'static>(look: &Arc<Look>, panel: Entity<ColorDetails>, cx: &mut Context<M>) -> Self {
        let mut swatches = Vec::new();
        for family in color_families(look.mode()) {
            let name = family.family;
            let mut row = Vec::new();
            for step in 1..=12_u8 {
                let swatch_look = Arc::clone(look);
                let color = move || catalog_color(&swatch_look, name, step);
                let button = swatch_info::swatch(
                    format!("catalog-{name}-{step}"),
                    look,
                    Some(COLOR_MATRIX_CELL_W),
                    COLOR_MATRIX_CELL_H,
                    color,
                    cx,
                );
                let source_look = Arc::clone(look);
                let panel = panel.downgrade();
                button.clone().on_click(cx, move |_, cx| {
                    let Some(panel) = panel.upgrade() else {
                        return;
                    };

                    let source = from_hsla(catalog_color(&source_look, name, step));
                    let background = catalog_background(source_look.mode());
                    let title = title_case(name);
                    match SwatchSelection::from_source(&title, step, source, background) {
                        Ok(selection) => panel.update(cx, |panel, cx| panel.open(selection, None, cx)),
                        Err(error) => eprintln!("failed to resolve catalog color: {error}"),
                    }
                });
                row.push(button);
            }
            swatches.push(row);
        }
        let mut alpha = Vec::new();
        for (name, values) in [("Black", &BLACK_ALPHA_STEPS), ("White", &WHITE_ALPHA_STEPS)] {
            let mut row = Vec::new();
            for (index, value) in values.iter().enumerate() {
                let step = (index + 1) as u8;
                let color = parse_color(value);
                let button = swatch_info::swatch(
                    format!("catalog-{name}-{step}"),
                    look,
                    Some(COLOR_MATRIX_CELL_W),
                    COLOR_MATRIX_CELL_H,
                    move || color,
                    cx,
                );
                let source_look = Arc::clone(look);
                let panel = panel.downgrade();
                button.clone().on_click(cx, move |_, cx| {
                    let Some(panel) = panel.upgrade() else {
                        return;
                    };

                    match SwatchSelection::from_alpha(
                        name,
                        step,
                        from_hsla(color),
                        catalog_background(source_look.mode()),
                    ) {
                        Ok(selection) => panel.update(cx, |panel, cx| panel.open(selection, None, cx)),
                        Err(error) => eprintln!("failed to resolve alpha color: {error}"),
                    }
                });
                row.push(button);
            }
            alpha.push(row);
        }
        Self { swatches, alpha }
    }

    pub fn render(&self, look: &Look, fg: Hsla, muted: Hsla) -> gpui::AnyElement {
        page(look, fg, muted, &self.swatches, &self.alpha)
    }
}

fn catalog_background(mode: ThemeMode) -> ColorValue {
    from_hsla(if mode == ThemeMode::Dark {
        gpui::rgb(0x111111).into()
    } else {
        gpui::white()
    })
}

fn catalog_color(look: &Look, name: &str, step: u8) -> Hsla {
    color_families(look.mode())
        .iter()
        .find(|family| family.family == name)
        .map(|family| parse_color(family.steps[usize::from(step - 1)]))
        .unwrap_or_else(gpui::transparent_black)
}

fn page(
    look: &Look,
    fg: Hsla,
    muted: Hsla,
    swatches: &[Vec<Entity<Button>>],
    alpha: &[Vec<Entity<Button>>],
) -> gpui::AnyElement {
    let mut matrix = div().flex().flex_col().gap(px(3.0)).flex_none();
    let mut groups = hstack! { gap=3; div().w(px(COLOR_MATRIX_LABEL_W)).flex_none() };
    for (label, span) in [("Backgrounds", 2), ("Interactive", 3), ("Borders", 3), ("Solid", 2), ("Text", 2)] {
        groups = groups.child(
            div()
                .w(px(COLOR_MATRIX_CELL_W * span as f32 + 3.0 * (span as f32 - 1.0)))
                .flex_none()
                .text_xs()
                .text_center()
                .text_color(muted)
                .border_b_1()
                .border_color(muted.opacity(0.45))
                .child(label),
        );
    }
    matrix = matrix.child(groups);

    let mut steps = hstack! { gap=3; div().w(px(COLOR_MATRIX_LABEL_W)).flex_none() };
    for step in 1..=12 {
        steps = steps.child(
            div()
                .w(px(COLOR_MATRIX_CELL_W))
                .flex_none()
                .text_xs()
                .text_center()
                .text_color(muted)
                .child(format!("{step}")),
        );
    }
    matrix = matrix.child(steps);

    for (family, swatches) in color_families(look.mode()).iter().zip(swatches) {
        matrix = matrix.child(color_catalog_row(family, swatches, muted));
    }

    let mut alpha_matrix = div().flex().flex_col().gap(px(3.0)).flex_none();
    let mut alpha_header = hstack! { gap=3; div().w(px(COLOR_MATRIX_LABEL_W)).flex_none() };
    for step in 1..=12 {
        alpha_header = alpha_header.child(
            div()
                .w(px(COLOR_MATRIX_CELL_W))
                .flex_none()
                .text_xs()
                .text_center()
                .text_color(muted)
                .child(format!("{step}")),
        );
    }
    alpha_matrix = alpha_matrix.child(alpha_header);
    alpha_matrix = alpha_matrix.child(alpha_catalog_row("Black", &alpha[0], muted));
    alpha_matrix = alpha_matrix.child(alpha_catalog_row("White", &alpha[1], muted));

    vstack! {
        gap=16;
        div().text_2xl().font_weight(gpui::FontWeight::BOLD).text_color(fg).child("Colors"),
        div().text_sm().text_color(muted).child("Radix Colors v3.0.0 · sRGB catalog"),
        div().child(matrix),
        div().text_lg().font_weight(gpui::FontWeight::SEMIBOLD).text_color(fg).child("Shadows, highlights, and overlays"),
        div().child(alpha_matrix),
    }
    .w_full()
    .into_any_element()
}

pub fn page_background(mode: ThemeMode) -> gpui::Background {
    match mode {
        ThemeMode::Light => page_gradient(parse_color("#ffeaf1"), parse_color("#ffffff")),
        ThemeMode::Dark => page_gradient(hsla(0.92156863, 0.5, 0.13333334, 1.0), hsla(0.0, 0.0, 0.043137256, 1.0)),
    }
}

fn page_gradient(start: Hsla, end: Hsla) -> gpui::Background {
    linear_gradient(180.0, linear_color_stop(start, 0.0), linear_color_stop(end, 1.0 / 3.0))
}

fn color_catalog_row(
    family: &xgpui_luma_look_radix::RawColorScale,
    swatches: &[Entity<Button>],
    muted: Hsla,
) -> gpui::Div {
    alpha_catalog_row(family.family, swatches, muted)
}

fn alpha_catalog_row(label: &'static str, swatches: &[Entity<Button>], muted: Hsla) -> gpui::Div {
    hstack! { gap=3; div().w(px(COLOR_MATRIX_LABEL_W)).flex_none().text_sm().text_color(muted).child(label) }
        .children(swatches.iter().cloned())
}

fn title_case(name: &str) -> String {
    let mut chars = name.chars();
    chars.next().map(|c| c.to_uppercase().collect::<String>() + chars.as_str()).unwrap_or_default()
}
