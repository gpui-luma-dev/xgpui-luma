//! Style Guide · Text Field
//!
//! In-section tabs:
//! - **Template Preview** — variants × interaction states (incl. invalid)
//! - **Colors** — accents × variants
//! - **All Sizes** — Sm / Md / Lg per variant

use std::sync::Arc;

use gpui::{
    AnyElement, App, Entity, FontWeight, Hsla, IntoElement, SharedString, TextRun, Window, div, font, prelude::*, px,
};
use xgpui_luma::controls::tabs::Tabs;
use xgpui_luma::controls::textfield::{
    TextFieldLook, TextFieldRenderModel, TextFieldState, TextFieldTemplate, TextFieldTheme,
};
use xgpui_luma::theme::{ControlSize, StandardBoxScale};
use xgpui_luma_look_radix::{Accent, Gray, Look, LookControlExt, TextFieldVariant, ScaleFamily, textfield_theme_with};

use super::matrix_grid::{
    COL_GAP, centered, column_header, empty_corner, equal_data_columns, fixed_grid, preview_tabbed, row_label,
};
use super::preview_handlers;
use super::table::{TableRow, TableStyle};
use crate::assets::{icon_named, react_icon};

const STATE_COLUMN_WIDTH: f32 = 132.0;
const VARIANT_COLUMN_WIDTH: f32 = 188.0;
const ROW_LABEL_WIDTH: f32 = 96.0;
const COLOR_COLUMN_WIDTH: f32 = 160.0;
const SIZE_COLUMN_WIDTH: f32 = 148.0;
const SIZE_COL_GAP: f32 = 24.0;
const SIZE_ROW_GAP: f32 = 20.0;
const PREVIEW_WIDTH: f32 = 120.0;
const SAMPLE_TEXT: &str = "Text";
const HEADER_ICON_SIZE: f32 = 15.0;

pub struct VariantDef {
    pub id: &'static str,
    pub label: &'static str,
    pub variant: TextFieldVariant,
}

pub const RADIX_VARIANTS: [VariantDef; 2] = [
    VariantDef { id: "surface", label: "Surface", variant: TextFieldVariant::Surface },
    VariantDef { id: "soft", label: "Soft", variant: TextFieldVariant::Soft },
];

#[derive(Clone, Copy)]
struct FieldSample {
    id: &'static str,
    label: &'static str,
    icon: &'static str,
    state: TextFieldState,
    enabled: bool,
}

fn samples() -> [FieldSample; 6] {
    [
        FieldSample { id: "default", label: "Default", icon: "home", state: TextFieldState::default(), enabled: true },
        FieldSample {
            id: "hover",
            label: "Hover",
            icon: "cursor-arrow",
            state: TextFieldState { hovered: true, ..TextFieldState::default() },
            enabled: true,
        },
        FieldSample {
            id: "focus",
            label: "Focus",
            icon: "border-dashed",
            state: TextFieldState {
                focused: true,
                focus_visible: true,
                cursor: SAMPLE_TEXT.chars().count(),
                ..TextFieldState::default()
            },
            enabled: true,
        },
        FieldSample {
            id: "active",
            label: "Active",
            icon: "arrow-down",
            state: TextFieldState {
                hovered: true,
                focused: true,
                focus_visible: true,
                cursor: SAMPLE_TEXT.chars().count(),
                selection_anchor: Some(0),
                ..TextFieldState::default()
            },
            enabled: true,
        },
        FieldSample {
            id: "invalid",
            label: "Invalid",
            icon: "exclamation-triangle",
            state: TextFieldState { invalid: true, ..TextFieldState::default() },
            enabled: true,
        },
        FieldSample {
            id: "disabled",
            label: "Disabled",
            icon: "circle-backslash",
            state: TextFieldState::default(),
            enabled: false,
        },
    ]
}

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
        let template = look.textfield_template(def.variant);
        let theme = textfield_theme_with(look, def.variant);
        for (col, (size, _)) in sizes.iter().enumerate() {
            let sample = FieldSample {
                id: "size",
                label: "Size",
                icon: "home",
                state: TextFieldState::default(),
                enabled: true,
            };
            grid = grid.child(
                centered(render_field(&template, theme.clone(), sample, def.id, *size, window, cx)),
                row,
                col + 1,
            );
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
            let template = row_look.textfield_template(def.variant);
            let theme = textfield_theme_with(&row_look, def.variant);
            let sample = FieldSample {
                id: "default",
                label: "Default",
                icon: "home",
                state: TextFieldState::default(),
                enabled: true,
            };
            grid = grid.child(
                centered(render_field(
                    &template,
                    theme,
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
    let samples = samples();
    let style = TableStyle {
        variant_column_width: VARIANT_COLUMN_WIDTH,
        state_column_width: STATE_COLUMN_WIDTH,
        row_height: 64.0,
        ..TableStyle::new(fg, muted)
    };
    let accent = super::palettes::title_case(look.palette_label(ScaleFamily::Color));
    let gray = super::palettes::title_case(look.palette_label(ScaleFamily::Gray));

    let headers = samples.iter().map(|sample| header_cell(sample, muted)).collect();
    let rows = variants
        .iter()
        .map(|def| {
            let template = look.textfield_template(def.variant);
            let theme = textfield_theme_with(look, def.variant);
            TableRow {
                label: SharedString::from(def.label),
                description: SharedString::from(description(def, &accent, &gray)),
                cells: samples
                    .iter()
                    .copied()
                    .map(|sample| {
                        centered(render_field(&template, theme.clone(), sample, def.id, ControlSize::Md, window, cx))
                    })
                    .collect(),
            }
        })
        .collect();

    super::table::render(&style, "VARIANTS", headers, rows)
}

fn description(def: &VariantDef, accent: &str, gray: &str) -> String {
    match def.variant {
        TextFieldVariant::Soft => format!("{accent} tint, no hard edge"),
        TextFieldVariant::Surface => format!("{gray} rim on surface"),
        _ => String::new(),
    }
}

fn header_cell(sample: &FieldSample, muted: Hsla) -> AnyElement {
    div()
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(4.0))
        .children(icon_named(sample.icon).map(|icon| react_icon(icon, muted, HEADER_ICON_SIZE)))
        .child(
            div()
                .text_xs()
                .line_height(px(15.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(muted)
                .child(sample.label),
        )
        .into_any_element()
}

fn render_field(
    template: &Arc<dyn TextFieldTemplate>,
    theme: Arc<dyn TextFieldTheme>,
    sample: FieldSample,
    variant_id: &str,
    size: ControlSize,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("style-guide-textfield-{variant_id}-{}-{}", sample.id, size_id(size)));
    let placeholder = SharedString::from("Placeholder");
    let value = SharedString::from(SAMPLE_TEXT);
    let look = field_look(&theme, sample.state, sample.enabled, size, window);
    let character_offsets = character_offsets(value.as_ref(), theme, sample.state, sample.enabled, size, window);
    let model = TextFieldRenderModel {
        id: &id,
        placeholder: &placeholder,
        value: &value,
        prefix_icon: None,
        size,
        enabled: sample.enabled,
        full_width: true,
        state: sample.state,
        marked_range: None,
        caret_visible: sample.state.focused && sample.enabled,
        horizontal_scroll: 0.0,
        character_offsets,
        look,
    };

    div()
        .w(px(PREVIEW_WIDTH))
        .flex()
        .items_center()
        .justify_center()
        .child(template.render(&model, preview_handlers::textfield_handlers(), window, cx))
        .into_any_element()
}

fn field_look(
    theme: &Arc<dyn TextFieldTheme>,
    state: TextFieldState,
    enabled: bool,
    size: ControlSize,
    window: &Window,
) -> TextFieldLook {
    let scale = StandardBoxScale::compute(size, &theme.metrics(), window.scale_factor());
    theme.resolve_look(state, enabled, size, &scale)
}

fn character_offsets(
    value: &str,
    theme: Arc<dyn TextFieldTheme>,
    state: TextFieldState,
    enabled: bool,
    size: ControlSize,
    window: &mut Window,
) -> Vec<f32> {
    let look = field_look(&theme, state, enabled, size, window);
    let run = TextRun {
        len: value.len(),
        font: {
            let mut font = font(".SystemUIFont");
            font.weight = look.typography.weight;
            font
        },
        color: look.foreground,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let line =
        window
            .text_system()
            .shape_line(SharedString::from(value.to_owned()), px(look.typography.size), &[run], None);
    let chars = value.chars().count();
    let mut offsets = Vec::with_capacity(chars + 1);
    for char_offset in 0..=chars {
        let byte_offset = value.chars().take(char_offset).map(char::len_utf8).sum();
        offsets.push(line.x_for_index(byte_offset).as_f32());
    }
    offsets
}

fn size_id(size: ControlSize) -> &'static str {
    match size {
        ControlSize::Sm => "sm",
        ControlSize::Md => "md",
        ControlSize::Lg => "lg",
    }
}

/// Render the same field template inside composite selection-control previews.
pub(super) fn selection_field(
    look: &Look,
    id: &SharedString,
    size: ControlSize,
    state: TextFieldState,
    enabled: bool,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let theme = textfield_theme_with(look, TextFieldVariant::Surface);
    let value = SharedString::from("Option one");
    let placeholder = SharedString::from("Choose an option");
    let model = TextFieldRenderModel {
        id,
        placeholder: &placeholder,
        value: &value,
        prefix_icon: None,
        size,
        enabled,
        full_width: true,
        state,
        marked_range: None,
        caret_visible: state.focused && enabled,
        horizontal_scroll: 0.0,
        character_offsets: character_offsets(&value, theme.clone(), state, enabled, size, window),
        look: field_look(&theme, state, enabled, size, window),
    };
    look.textfield_template(TextFieldVariant::Surface)
        .render(&model, preview_handlers::textfield_handlers(), window, cx)
        .into_any_element()
}
