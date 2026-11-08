use std::sync::Arc;

use gpui::{AnyElement, App, Entity, FontWeight, IntoElement, SharedString, TextRun, Window, div, font, prelude::*, px};
use xgpui_luma::controls::textarea::{
    TextAreaLineMetric, TextAreaRenderModel, TextAreaState, TextAreaTemplate, TextAreaTheme, ThemedTextAreaTemplate,
};
use xgpui_luma::controls::tabs::Tabs;
use xgpui_luma::theme::{ControlSize, StandardBoxScale};
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::style::shared::preview_handlers::input_textarea_handlers;
use crate::studio::style::shared::shadow_matrix::{ShadowPreviewShape, render_shadow_token_matrix};
use crate::studio::style::shared::shell::section_shell_with_width;
use crate::studio::style::variant_state_table::{VariantStateTable, VariantStateTableRow, VariantStateTableStyle};

// Six state columns must fit inside the 960px style-guide section.
const TEXTAREA_TABLE_STATE_COLUMN_WIDTH: f32 = 142.0;
const TEXTAREA_TABLE_VARIANT_COLUMN_WIDTH: f32 = 108.0;
const TEXTAREA_TABLE_HEADER_HEIGHT: f32 = 28.0;
/// Tall enough for 3-row previews + Primary elevation.
const TEXTAREA_TABLE_ROW_HEIGHT: f32 = 120.0;

#[derive(Clone, Copy)]
struct InputTextAreaSample {
    id: &'static str,
    label: &'static str,
    state: TextAreaState,
    enabled: bool,
}

#[derive(Clone, Copy)]
struct TextAreaStyleVariant {
    id: &'static str,
    label: &'static str,
}

/// Matches `ShadcnTextFieldStyle` / `ShadcnTextAreaExt` (same order as text field).
const TEXTAREA_STYLE_VARIANTS: [TextAreaStyleVariant; 3] = [
    TextAreaStyleVariant { id: "primary", label: "Primary" },
    TextAreaStyleVariant { id: "outline", label: "Outline" },
    TextAreaStyleVariant { id: "surface", label: "Surface" },
];

pub(crate) fn render_textarea_template_section(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<Tabs>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let active_tab =
        preview_tabs.read(cx).active_id().cloned().unwrap_or_else(|| SharedString::from("template-preview"));

    section_shell_with_width(
        960.0,
        "Text Area",
        "Template Preview: Primary / Outline / Surface × states, including invalid.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        render_textarea_preview_tabbed_content(look, preview_tabs, active_tab, chrome.border, window, cx),
    )
}

fn render_textarea_preview_tabbed_content(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<Tabs>,
    active_tab: SharedString,
    border: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let body = if matches!(active_tab.as_ref(), "shadows" | "textarea-shadows") {
        render_shadow_token_matrix(&look, ShadowPreviewShape::TextControl)
    } else {
        render_textarea_template_body(&look, window, cx)
    };

    div()
        .w_full()
        .flex()
        .flex_col()
        .child(div().w_full().flex().justify_start().child(preview_tabs))
        .child(div().w_full().h(px(1.0)).bg(border))
        .child(div().w_full().flex().justify_center().mt(px(16.0)).child(body))
        .into_any_element()
}

fn render_textarea_template_body(look: &Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let chrome = look.chrome();
    let samples = input_textarea_samples();

    let matrix = VariantStateTable::new(
        VariantStateTableStyle::from_chrome(&chrome)
            .variant_column_width(TEXTAREA_TABLE_VARIANT_COLUMN_WIDTH)
            .state_column_width(TEXTAREA_TABLE_STATE_COLUMN_WIDTH)
            .header_height(TEXTAREA_TABLE_HEADER_HEIGHT)
            .header_corner_padding_bottom(0.0)
            .row_height(TEXTAREA_TABLE_ROW_HEIGHT),
    )
    .column_headers(samples.iter().map(|sample| render_textarea_state_header_cell(*sample, chrome.muted_text)))
    .rows(TEXTAREA_STYLE_VARIANTS.iter().map(|style| {
        let (template, theme) = textarea_style_pair(look, style.id);
        VariantStateTableRow {
            label: SharedString::from(style.label),
            description: SharedString::from(""),
            cells: samples
                .iter()
                .copied()
                .map(|sample| render_textarea_cell(&template, theme.clone(), sample, style.id, window, cx))
                .collect(),
        }
    }))
    .build();

    div().w_full().flex().flex_col().gap(px(24.0)).child(matrix).into_any_element()
}

fn textarea_style_pair(look: &Arc<ShadcnLook>, style_id: &str) -> (Arc<dyn TextAreaTemplate>, Arc<dyn TextAreaTheme>) {
    match style_id {
        "surface" => {
            (Arc::new(ThemedTextAreaTemplate::new(look.surface_textarea_theme())), look.surface_textarea_theme())
        }
        "primary" => (look.primary_textarea_template(), look.primary_textarea_theme()),
        _ => (look.textarea_template(), look.textarea_theme()),
    }
}

fn render_textarea_state_header_cell(sample: InputTextAreaSample, muted_text: gpui::Hsla) -> AnyElement {
    div()
        .w_full()
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .text_xs()
        .line_height(px(15.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(muted_text)
        .child(sample.label)
        .into_any_element()
}

fn render_textarea_cell(
    template: &Arc<dyn TextAreaTemplate>,
    theme: Arc<dyn TextAreaTheme>,
    sample: InputTextAreaSample,
    style_id: &str,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("luma-studio-textarea-preview-{}-{}", style_id, sample.id));
    let placeholder = SharedString::from("Placeholder");
    let value = SharedString::from("Preview\nText area");
    let line_metrics = input_textarea_line_metrics(value.as_ref(), theme, sample.state, sample.enabled, window);
    let model = TextAreaRenderModel {
        id: &id,
        placeholder: &placeholder,
        value: &value,
        enabled: sample.enabled,
        full_width: true,
        size: ControlSize::Md,
        rows: 3,
        state: sample.state,
        caret_visible: sample.state.focused && sample.enabled,
        vertical_scroll: 0.0,
        line_metrics,
    };

    div()
        .w(px(127.0))
        .flex()
        .items_center()
        .justify_center()
        .child(template.render(&model, input_textarea_handlers(), window, cx))
        .into_any_element()
}

fn input_textarea_samples() -> [InputTextAreaSample; 6] {
    [
        InputTextAreaSample { id: "default", label: "Default", state: TextAreaState::default(), enabled: true },
        InputTextAreaSample {
            id: "hover",
            label: "Hover",
            state: TextAreaState { hovered: true, ..TextAreaState::default() },
            enabled: true,
        },
        InputTextAreaSample {
            id: "focus",
            label: "Focus",
            state: TextAreaState { focused: true, focus_visible: true, cursor: 7, ..TextAreaState::default() },
            enabled: true,
        },
        InputTextAreaSample {
            id: "active",
            label: "Active",
            state: TextAreaState {
                hovered: true,
                focused: true,
                focus_visible: true,
                cursor: 18,
                selection_anchor: Some(4),
                ..TextAreaState::default()
            },
            enabled: true,
        },
        InputTextAreaSample {
            id: "invalid",
            label: "Invalid",
            state: TextAreaState { invalid: true, ..TextAreaState::default() },
            enabled: true,
        },
        InputTextAreaSample { id: "disabled", label: "Disabled", state: TextAreaState::default(), enabled: false },
    ]
}

fn input_textarea_line_metrics(
    value: &str,
    theme: Arc<dyn TextAreaTheme>,
    state: TextAreaState,
    enabled: bool,
    window: &mut Window,
) -> Vec<TextAreaLineMetric> {
    let look = input_textarea_look(&theme, state, enabled, window);
    let mut metrics = Vec::new();
    let mut start = 0usize;
    let mut current = String::new();

    for (ix, ch) in value.chars().enumerate() {
        if ch == '\n' {
            metrics.push(input_textarea_shape_metric(
                start,
                ix,
                std::mem::take(&mut current),
                metrics.len(),
                &look,
                window,
            ));
            start = ix + 1;
        } else {
            current.push(ch);
        }
    }
    metrics.push(input_textarea_shape_metric(start, value.chars().count(), current, metrics.len(), &look, window));
    metrics
}

fn input_textarea_shape_metric(
    start: usize,
    end: usize,
    text: String,
    line_ix: usize,
    look: &xgpui_luma::controls::textarea::TextAreaLook,
    window: &mut Window,
) -> TextAreaLineMetric {
    let run = TextRun {
        len: text.len(),
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
            .shape_line(SharedString::from(text.to_owned()), px(look.typography.size), &[run], None);
    let chars = text.chars().count();
    let mut character_offsets = Vec::with_capacity(chars + 1);
    for char_offset in 0..=chars {
        let byte_offset = text.chars().take(char_offset).map(char::len_utf8).sum();
        character_offsets.push(line.x_for_index(byte_offset).as_f32());
    }

    TextAreaLineMetric {
        start,
        end,
        text,
        y: line_ix as f32 * look.typography.line_height,
        height: look.typography.line_height,
        character_offsets,
    }
}

fn input_textarea_look(
    theme: &Arc<dyn TextAreaTheme>,
    state: TextAreaState,
    enabled: bool,
    window: &Window,
) -> xgpui_luma::controls::textarea::TextAreaLook {
    let scale = StandardBoxScale::compute(ControlSize::Md, &theme.metrics(), window.scale_factor());
    theme.resolve_look(state, enabled, ControlSize::Md, &scale)
}
