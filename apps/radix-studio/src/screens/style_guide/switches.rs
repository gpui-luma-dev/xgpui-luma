//! Style Guide · Switches
//!
//! In-section tabs:
//! - **Template Preview** — variants × interaction states (`table` + `states`)
//! - **Colors** — accents × variants (default + high-contrast pair per cell)
//! - **All Sizes** — per-variant size × radius grids

use std::sync::Arc;

use gpui::{AnyElement, App, Entity, Hsla, IntoElement, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::button::{ButtonRenderModel, ButtonTemplate};
use xgpui_luma::controls::button_family::{ButtonFamilyRole, ButtonSize};
use xgpui_luma::controls::switch::SwitchData;
use xgpui_luma::controls::tabs::Tabs;
use xgpui_luma::{hstack, vstack};
use xgpui_luma_look_radix::{Accent, Paint, Gray, Look, LookControlExt, Radius, SwitchSize, SwitchVariant, ScaleFamily};

use super::matrix_grid::{
    COL_GAP, centered, column_header, corner_label, empty_corner, equal_data_columns, fixed_grid, preview_tabbed,
    row_label,
};
use super::states::{self, StateSample};
use super::table::{TableRow, TableStyle};

const STATE_COLUMN_WIDTH: f32 = 168.0;
const VARIANT_COLUMN_WIDTH: f32 = 220.0;
const ROW_LABEL_WIDTH: f32 = 96.0;
const COLOR_COLUMN_WIDTH: f32 = 220.0;
const SIZE_COLUMN_WIDTH: f32 = 128.0;
const SIZE_COL_GAP: f32 = 24.0;
const SIZE_ROW_GAP: f32 = 20.0;
const SIZE_TABLE_GAP: f32 = 40.0;
const PAIR_GAP: f32 = 16.0;

pub struct VariantDef {
    pub id: &'static str,
    pub label: &'static str,
    description: &'static str,
    pub variant: SwitchVariant,
}

/// Style Guide switch variants (Classic deferred until elevation lands).
pub const RADIX_VARIANTS: [VariantDef; 2] = [
    VariantDef {
        id: "surface",
        label: "Surface",
        description: "Gray track with an accent fill when on",
        variant: SwitchVariant::Surface,
    },
    VariantDef {
        id: "soft",
        label: "Soft",
        description: "Accent tint when on, no hard edge",
        variant: SwitchVariant::Soft,
    },
];

pub fn tabbed(
    look: &Arc<Look>,
    preview_tabs: Entity<Tabs>,
    fg: Hsla,
    muted: Hsla,
    border: Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let active = preview_tabs.read(cx).active_id().cloned().unwrap_or_else(|| SharedString::from("template-preview"));

    let body = match active.as_ref() {
        "colors" => colors_matrix(look, fg, muted, window, cx),
        "all-sizes" => sizes_matrix(look, fg, muted, window, cx),
        _ => matrix(look, &RADIX_VARIANTS, fg, muted, window, cx),
    };

    preview_tabbed(preview_tabs, border, body)
}

/// Size 1–3 × theme radius for every switch variant.
pub fn sizes_matrix(look: &Arc<Look>, fg: Hsla, muted: Hsla, window: &mut Window, cx: &mut App) -> AnyElement {
    vstack! { gap=SIZE_TABLE_GAP; }
        .children(RADIX_VARIANTS.iter().map(|def| sizes_variant_table(look, def, fg, muted, window, cx)))
        .into_any_element()
}

fn sizes_variant_table(
    look: &Arc<Look>,
    def: &VariantDef,
    fg: Hsla,
    muted: Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let mut grid = fixed_grid(
        equal_data_columns(ROW_LABEL_WIDTH, SIZE_COLUMN_WIDTH, Radius::ALL.len()),
        SIZE_COL_GAP,
        SIZE_ROW_GAP,
    );

    grid = grid.child(corner_label(def.label, muted), 0, 0);
    for (col, radius) in Radius::ALL.iter().enumerate() {
        grid = grid.child(column_header(radius.label(), muted), 0, col + 1);
    }

    for (row, size) in SwitchSize::ALL.iter().enumerate() {
        let row = row + 1;
        grid = grid.child(row_label(size.label(), fg), row, 0);
        for (col, radius) in Radius::ALL.iter().enumerate() {
            grid = grid.child(sizes_cell(look, def, *size, *radius, window, cx), row, col + 1);
        }
    }

    grid.into_any_element()
}

fn sizes_cell(
    look: &Arc<Look>,
    def: &VariantDef,
    size: SwitchSize,
    radius: Radius,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let template = look.switch_template_for(def.variant, Paint::accent(), size, radius);
    centered(switch_pair(
        &template,
        &format!("sizes-{}-{}-{}", def.id, size.as_str(), radius.as_str()),
        size.control_size(),
        window,
        cx,
    ))
}

/// Accents × variants: each cell pairs default + high-contrast (off + on each).
pub fn colors_matrix(base_look: &Arc<Look>, fg: Hsla, muted: Hsla, window: &mut Window, cx: &mut App) -> AnyElement {
    let mut grid =
        fixed_grid(equal_data_columns(ROW_LABEL_WIDTH, COLOR_COLUMN_WIDTH, RADIX_VARIANTS.len()), COL_GAP, COL_GAP);

    grid = grid.child(empty_corner(), 0, 0);
    for (col, def) in RADIX_VARIANTS.iter().enumerate() {
        grid = grid.child(column_header(def.label, muted), 0, col + 1);
    }

    for (row, accent) in Accent::ALL.iter().enumerate() {
        let row = row + 1;
        let row_look = Arc::new(base_look.fork());
        row_look.set_palettes(*accent, Gray::Auto);
        let accent_id = accent.as_str();
        let label = super::palettes::title_case(accent_id);

        grid = grid.child(row_label(label, fg), row, 0);
        for (col, def) in RADIX_VARIANTS.iter().enumerate() {
            grid = grid.child(colors_variant_cell(&row_look, accent_id, def, window, cx), row, col + 1);
        }
    }

    grid.into_any_element()
}

fn colors_variant_cell(
    look: &Arc<Look>,
    accent_id: &str,
    def: &VariantDef,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let default = look.switch_template_with(def.variant, Paint::accent());
    let high_contrast = look.switch_template_with(def.variant, Paint::accent().high_contrast());

    centered(
        hstack! { gap=12 align=center; }
            .child(switch_pair(&default, &format!("colors-{accent_id}-{}-default", def.id), ButtonSize::Md, window, cx))
            .child(switch_pair(
                &high_contrast,
                &format!("colors-{accent_id}-{}-hc", def.id),
                ButtonSize::Md,
                window,
                cx,
            )),
    )
}

pub fn matrix(
    look: &Arc<Look>,
    variants: &[VariantDef],
    fg: Hsla,
    muted: Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let samples = states::samples();
    let style = TableStyle {
        variant_column_width: VARIANT_COLUMN_WIDTH,
        state_column_width: STATE_COLUMN_WIDTH,
        ..TableStyle::new(fg, muted)
    };
    let accent = super::palettes::title_case(look.palette_label(ScaleFamily::Color));
    let gray = super::palettes::title_case(look.palette_label(ScaleFamily::Gray));

    let headers = samples.iter().map(|sample| states::header_cell(sample, muted)).collect();
    let rows = variants
        .iter()
        .map(|def| {
            let template = look.switch_template(def.variant);
            TableRow {
                label: SharedString::from(def.label),
                description: SharedString::from(description(def, &accent, &gray)),
                cells: samples.iter().map(|sample| state_cell(&template, def.id, sample, window, cx)).collect(),
            }
        })
        .collect();

    super::table::render(&style, "VARIANTS", headers, rows)
}

fn description(def: &VariantDef, accent: &str, gray: &str) -> String {
    match def.variant {
        SwitchVariant::Soft => format!("{accent} tint when on"),
        SwitchVariant::Surface => format!("{gray} track, {accent} when on"),
        _ => def.description.to_string(),
    }
}

fn state_cell(
    template: &Arc<dyn ButtonTemplate<SwitchData>>,
    variant_id: &str,
    sample: &StateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    centered(div().flex().items_center().justify_center().gap(px(PAIR_GAP)).children([false, true].map(|on| {
        render_switch(
            template,
            &format!("style-guide-switch-{variant_id}-{}-{}", sample.id, if on { "on" } else { "off" }),
            on,
            ButtonSize::Md,
            sample.state,
            window,
            cx,
        )
    })))
}

fn switch_pair(
    template: &Arc<dyn ButtonTemplate<SwitchData>>,
    id_prefix: &str,
    size: ButtonSize,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(PAIR_GAP))
        .children([false, true].map(|on| {
            render_switch(
                template,
                &format!("{id_prefix}-{}", if on { "on" } else { "off" }),
                on,
                size,
                Default::default(),
                window,
                cx,
            )
        }))
        .into_any_element()
}

fn render_switch(
    template: &Arc<dyn ButtonTemplate<SwitchData>>,
    id: &str,
    on: bool,
    size: ButtonSize,
    state: xgpui_luma::theme::InteractionState,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let model = ButtonRenderModel {
        id: SharedString::from(id),
        data: SwitchData::new(on),
        content: Arc::new(|_, _| div().into_any_element()),
        role: ButtonFamilyRole::Icon,
        size,
        state,
        ..Default::default()
    };
    template.render(&model, window, cx).into_any_element()
}
