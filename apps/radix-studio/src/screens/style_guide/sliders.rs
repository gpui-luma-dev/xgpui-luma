//! Style Guide · Slider
//!
//! In-section tabs:
//! - **Template Preview** — variants × interaction states
//! - **Colors** — accents × variants
//! - **All Sizes** — Sm / Md / Lg per variant

use std::sync::Arc;

use gpui::{AnyElement, App, Entity, Hsla, IntoElement, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::slider::{
    SliderInputStrategy, SliderRenderModel, SliderTemplate, SliderThumbPolicy, SliderThumbRole, SliderThumbValue,
    ThumbId, TrackPresentation, build_track_segments,
};
use xgpui_luma::controls::tabs::Tabs;
use xgpui_luma::infra::value::ControlRange;
use xgpui_luma::theme::ControlSize;
use xgpui_luma_look_radix::{Accent, Gray, Look, LookControlExt, SliderVariant, ScaleFamily};

use super::matrix_grid::{
    COL_GAP, centered, column_header, empty_corner, equal_data_columns, fixed_grid, preview_tabbed, row_label,
};
use super::preview_handlers;
use super::states::{self, StateSample};
use super::table::{TableRow, TableStyle};

const STATE_COLUMN_WIDTH: f32 = 152.0;
const VARIANT_COLUMN_WIDTH: f32 = 188.0;
const ROW_LABEL_WIDTH: f32 = 96.0;
const COLOR_COLUMN_WIDTH: f32 = 160.0;
const SIZE_COLUMN_WIDTH: f32 = 152.0;
const SIZE_COL_GAP: f32 = 24.0;
const SIZE_ROW_GAP: f32 = 20.0;
const DEMO_WIDTH: f32 = 120.0;

pub struct VariantDef {
    pub id: &'static str,
    pub label: &'static str,
    pub variant: SliderVariant,
}

pub const RADIX_VARIANTS: [VariantDef; 2] = [
    VariantDef { id: "surface", label: "Surface", variant: SliderVariant::Surface },
    VariantDef { id: "soft", label: "Soft", variant: SliderVariant::Soft },
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

pub fn sizes_matrix(look: &Arc<Look>, fg: Hsla, muted: Hsla, window: &mut Window, cx: &mut App) -> AnyElement {
    let sizes = [(ControlSize::Sm, "Sm"), (ControlSize::Md, "Md"), (ControlSize::Lg, "Lg")];
    let mut grid =
        fixed_grid(equal_data_columns(ROW_LABEL_WIDTH, SIZE_COLUMN_WIDTH, sizes.len()), SIZE_COL_GAP, SIZE_ROW_GAP);

    grid = grid.child(empty_corner(), 0, 0);
    for (col, (_, label)) in sizes.iter().enumerate() {
        grid = grid.child(column_header(*label, muted), 0, col + 1);
    }

    for (row, def) in RADIX_VARIANTS.iter().enumerate() {
        let row = row + 1;
        grid = grid.child(row_label(def.label, fg), row, 0);
        let template = look.slider_template(def.variant);
        for (col, (size, _)) in sizes.iter().enumerate() {
            let sample = StateSample { id: "size", label: "", icon: "home", state: Default::default() };
            grid = grid.child(centered(render_slider(&template, sample, def.id, *size, window, cx)), row, col + 1);
        }
    }

    grid.into_any_element()
}

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
            let template = row_look.slider_template(def.variant);
            let sample = StateSample { id: "default", label: "", icon: "home", state: Default::default() };
            grid = grid.child(
                centered(render_slider(
                    &template,
                    sample,
                    &format!("colors-{accent_id}-{}", def.id),
                    ControlSize::Md,
                    window,
                    cx,
                )),
                row,
                col + 1,
            );
        }
    }

    grid.into_any_element()
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
            let template = look.slider_template(def.variant);
            TableRow {
                label: SharedString::from(def.label),
                description: SharedString::from(description(def, &accent, &gray)),
                cells: samples
                    .iter()
                    .map(|sample| centered(render_slider(&template, *sample, def.id, ControlSize::Md, window, cx)))
                    .collect(),
            }
        })
        .collect();

    super::table::render(&style, "VARIANTS", headers, rows)
}

fn description(def: &VariantDef, accent: &str, gray: &str) -> String {
    match def.variant {
        SliderVariant::Soft => format!("{gray} track, {accent}-6 fill"),
        SliderVariant::Surface => format!("{gray} track, flat thumb"),
        _ => String::new(),
    }
}

fn render_slider(
    template: &Arc<dyn SliderTemplate>,
    sample: StateSample,
    variant_id: &str,
    size: ControlSize,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("style-guide-slider-{variant_id}-{}-{}", sample.id, size_id(size)));
    let range = ControlRange::from(1..100);
    let value = 41.0;
    let position = range.percentage(value);
    let thumb_id = ThumbId::next();
    let thumbs = [SliderThumbValue { id: thumb_id, position, preview: None, role: SliderThumbRole::Value }];
    let track_segments = build_track_segments(TrackPresentation::Fill, position, &[], range);
    let model = SliderRenderModel {
        id: &id,
        strategy: SliderInputStrategy::Horizontal,
        orientation: SliderInputStrategy::Horizontal.orientation(),
        presentation: TrackPresentation::Fill,
        size,
        thumb_size: None,
        range,
        step: 10.0,
        thumbs: &thumbs,
        track_segments,
        reversed: false,
        wrapping: false,
        enabled: !sample.state.disabled,
        corner_radius: None,
        thumb_radius: None,
        thumb_policy: SliderThumbPolicy::default(),
        active_thumb_id: Some(thumb_id),
        state: sample.state,
        domain_track: None,
    };

    div()
        .w(px(DEMO_WIDTH))
        .flex()
        .items_center()
        .justify_center()
        .py(px(4.0))
        .child(template.render(&model, preview_handlers::slider_handlers(), thumb_id, window, cx))
        .into_any_element()
}

fn size_id(size: ControlSize) -> &'static str {
    match size {
        ControlSize::Sm => "sm",
        ControlSize::Md => "md",
        ControlSize::Lg => "lg",
    }
}
