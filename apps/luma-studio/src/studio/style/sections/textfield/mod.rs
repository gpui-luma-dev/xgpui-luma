use std::sync::Arc;

use gpui::{AnyElement, App, Entity, FontWeight, IntoElement, SharedString, TextRun, Window, div, font, prelude::*, px};
use xgpui_luma::controls::button::ControlIcon;
use xgpui_luma::controls::tabs::Tabs;
use xgpui_luma::controls::textfield::{
    TextFieldRenderModel, TextFieldState, TextFieldTemplate, TextFieldTheme, ThemedTextFieldTemplate,
};
use xgpui_luma::theme::{ControlSize, StandardBoxScale};
use xgpui_luma_look_shadcn::ShadcnLook;
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::style::shared::button_matrix::button_size_id;
use crate::studio::style::shared::preview_handlers::input_textfield_handlers;
use crate::studio::style::shared::shell::section_shell_with_width;
use crate::studio::style::shared::shadow_matrix::{ShadowPreviewShape, render_shadow_token_matrix};
use crate::studio::style::variant_state_table::{VariantStateTable, VariantStateTableRow, VariantStateTableStyle};

// Six state columns stay inside the 960px style-guide section.
// Preview width fits prefix icon + "Text" through Lg, including Primary elevation.
const TEXTFIELD_SAMPLE_TEXT: &str = "Text";
const TEXTFIELD_PREVIEW_WIDTH: f32 = 120.0;
const TEXTFIELD_TABLE_STATE_COLUMN_WIDTH: f32 = 132.0;
const TEXTFIELD_TABLE_SIZE_COLUMN_WIDTH: f32 = 136.0;
const TEXTFIELD_TABLE_VARIANT_COLUMN_WIDTH: f32 = 108.0;
const TEXTFIELD_TABLE_HEADER_HEIGHT: f32 = 28.0;
/// Extra height so Primary (`shadow-xs`) is not clipped by the row cell.
const TEXTFIELD_TABLE_ROW_HEIGHT: f32 = 64.0;

#[derive(Clone, Copy)]
struct InputTextFieldSample {
    id: &'static str,
    label: &'static str,
    state: TextFieldState,
    enabled: bool,
}

#[derive(Clone, Copy)]
struct TextFieldStyleVariant {
    id: &'static str,
    label: &'static str,
}

/// Matches `ShadcnTextFieldStyle` / `ShadcnTextFieldExt`.
const TEXTFIELD_STYLE_VARIANTS: [TextFieldStyleVariant; 3] = [
    TextFieldStyleVariant { id: "primary", label: "Primary" },
    TextFieldStyleVariant { id: "outline", label: "Outline" },
    TextFieldStyleVariant { id: "surface", label: "Surface" },
];

pub(crate) fn render_textfield_template_section(
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
        "Text Field",
        "Template Preview: Primary / Outline / Surface × states, including invalid. Sizes tab: same variants × Sm/Md/Lg.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        render_textfield_preview_tabbed_content(look, preview_tabs, active_tab, chrome.border, window, cx),
    )
}

fn render_textfield_preview_tabbed_content(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<Tabs>,
    active_tab: SharedString,
    border: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let body = match active_tab.as_ref() {
        "sizes" => render_textfield_sizes_body(&look, window, cx),
        "shadows" => render_shadow_token_matrix(&look, ShadowPreviewShape::TextControl),
        _ => render_textfield_template_body(&look, window, cx),
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

fn render_textfield_template_body(look: &Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let chrome = look.chrome();
    let samples = input_textfield_samples();

    let matrix = VariantStateTable::new(
        VariantStateTableStyle::from_chrome(&chrome)
            .variant_column_width(TEXTFIELD_TABLE_VARIANT_COLUMN_WIDTH)
            .state_column_width(TEXTFIELD_TABLE_STATE_COLUMN_WIDTH)
            .header_height(TEXTFIELD_TABLE_HEADER_HEIGHT)
            .header_corner_padding_bottom(0.0)
            .row_height(TEXTFIELD_TABLE_ROW_HEIGHT),
    )
    .column_headers(samples.iter().map(|sample| render_textfield_state_header_cell(*sample, chrome.muted_text)))
    .rows(TEXTFIELD_STYLE_VARIANTS.iter().map(|style| {
        let (template, theme) = textfield_style_pair(look, style.id);
        VariantStateTableRow {
            label: SharedString::from(style.label),
            description: SharedString::from(""),
            cells: samples
                .iter()
                .copied()
                .map(|sample| {
                    render_textfield_cell(&template, theme.clone(), sample, style.id, ControlSize::Md, window, cx)
                })
                .collect(),
        }
    }))
    .build();

    div().w_full().flex().flex_col().gap(px(24.0)).child(matrix).into_any_element()
}

fn render_textfield_sizes_body(look: &Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let chrome = look.chrome();
    let sizes = [(ControlSize::Sm, "Sm"), (ControlSize::Md, "Md"), (ControlSize::Lg, "Lg")];
    let sample = InputTextFieldSample { id: "size", label: "Size", state: TextFieldState::default(), enabled: true };

    VariantStateTable::new(
        VariantStateTableStyle::from_chrome(&chrome)
            .variant_column_width(TEXTFIELD_TABLE_VARIANT_COLUMN_WIDTH)
            .state_column_width(TEXTFIELD_TABLE_SIZE_COLUMN_WIDTH)
            .header_height(TEXTFIELD_TABLE_HEADER_HEIGHT)
            .header_corner_padding_bottom(0.0)
            .row_height(TEXTFIELD_TABLE_ROW_HEIGHT),
    )
    .column_headers(sizes.iter().map(|(_, label)| render_textfield_size_header_cell(label, chrome.muted_text)))
    .rows(TEXTFIELD_STYLE_VARIANTS.iter().map(|style| {
        let (template, theme) = textfield_style_pair(look, style.id);
        VariantStateTableRow {
            label: SharedString::from(style.label),
            description: SharedString::from(""),
            cells: sizes
                .iter()
                .copied()
                .map(|(size, _)| render_textfield_cell(&template, theme.clone(), sample, style.id, size, window, cx))
                .collect(),
        }
    }))
    .build()
}

fn textfield_style_pair(
    look: &Arc<ShadcnLook>,
    style_id: &str,
) -> (Arc<dyn TextFieldTemplate>, Arc<dyn TextFieldTheme>) {
    match style_id {
        "primary" => (look.primary_textfield_template(), look.primary_textfield_theme()),
        "surface" => (
            Arc::new(ThemedTextFieldTemplate::new(look.surface_textfield_theme())),
            look.surface_textfield_theme(),
        ),
        _ => (look.textfield_template(), look.textfield_theme()),
    }
}

fn render_textfield_state_header_cell(sample: InputTextFieldSample, muted_text: gpui::Hsla) -> AnyElement {
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

fn render_textfield_size_header_cell(label: &'static str, muted_text: gpui::Hsla) -> AnyElement {
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
        .child(label)
        .into_any_element()
}

fn render_textfield_cell(
    template: &Arc<dyn TextFieldTemplate>,
    theme: Arc<dyn TextFieldTheme>,
    sample: InputTextFieldSample,
    style_id: &str,
    size: ControlSize,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!(
        "luma-studio-textfield-preview-{}-{}-{}",
        style_id,
        sample.id,
        button_size_id(size)
    ));
    let placeholder = SharedString::from("Placeholder");
    let value = SharedString::from(TEXTFIELD_SAMPLE_TEXT);
    let look = input_textfield_look(&theme, sample.state, sample.enabled, size, window);
    let character_offsets =
        input_textfield_character_offsets(value.as_ref(), theme, sample.state, sample.enabled, size, window);
    let prefix_icon = ControlIcon::Lucide(LucideIcon::Search);
    let model = TextFieldRenderModel {
        id: &id,
        placeholder: &placeholder,
        value: &value,
        prefix_icon: Some(&prefix_icon),
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
        .w(px(TEXTFIELD_PREVIEW_WIDTH))
        .flex()
        .items_center()
        .justify_center()
        .child(template.render(&model, input_textfield_handlers(), window, cx))
        .into_any_element()
}

fn input_textfield_samples() -> [InputTextFieldSample; 6] {
    [
        InputTextFieldSample { id: "default", label: "Default", state: TextFieldState::default(), enabled: true },
        InputTextFieldSample {
            id: "hover",
            label: "Hover",
            state: TextFieldState { hovered: true, ..TextFieldState::default() },
            enabled: true,
        },
        InputTextFieldSample {
            id: "focus",
            label: "Focus",
            state: TextFieldState {
                focused: true,
                focus_visible: true,
                cursor: TEXTFIELD_SAMPLE_TEXT.chars().count(),
                ..TextFieldState::default()
            },
            enabled: true,
        },
        InputTextFieldSample {
            id: "active",
            label: "Active",
            state: TextFieldState {
                hovered: true,
                focused: true,
                focus_visible: true,
                cursor: TEXTFIELD_SAMPLE_TEXT.chars().count(),
                selection_anchor: Some(0),
                ..TextFieldState::default()
            },
            enabled: true,
        },
        InputTextFieldSample {
            id: "invalid",
            label: "Invalid",
            state: TextFieldState { invalid: true, ..TextFieldState::default() },
            enabled: true,
        },
        InputTextFieldSample { id: "disabled", label: "Disabled", state: TextFieldState::default(), enabled: false },
    ]
}

fn input_textfield_look(
    theme: &Arc<dyn TextFieldTheme>,
    state: TextFieldState,
    enabled: bool,
    size: ControlSize,
    window: &Window,
) -> xgpui_luma::controls::textfield::TextFieldLook {
    let scale = StandardBoxScale::compute(size, &theme.metrics(), window.scale_factor());
    theme.resolve_look(state, enabled, size, &scale)
}

fn input_textfield_character_offsets(
    value: &str,
    theme: Arc<dyn TextFieldTheme>,
    state: TextFieldState,
    enabled: bool,
    size: ControlSize,
    window: &mut Window,
) -> Vec<f32> {
    let look = input_textfield_look(&theme, state, enabled, size, window);
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
    let mut character_offsets = Vec::with_capacity(chars + 1);
    for char_offset in 0..=chars {
        let byte_offset = value.chars().take(char_offset).map(char::len_utf8).sum();
        character_offsets.push(line.x_for_index(byte_offset).as_f32());
    }
    character_offsets
}
