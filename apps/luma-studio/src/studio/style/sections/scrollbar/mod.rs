use std::sync::Arc;

use gpui::{AnyElement, App, Entity, FontWeight, IntoElement, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::scrollbar::{ScrollbarOrientation, ScrollbarRenderModel, ScrollbarStyle, ScrollbarTemplate};
use xgpui_luma::controls::tabs::Tabs;
use xgpui_luma::infra::value::ControlRange;
use xgpui_luma::theme::ControlSize;
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::style::shared::input_samples::{InputInteractionSample, input_interaction_samples};
use crate::studio::style::shared::preview_handlers::input_scrollbar_handlers;
use crate::studio::style::shared::shell::section_shell_with_width;
use crate::studio::style::variant_state_table::{VariantStateTable, VariantStateTableRow, VariantStateTableStyle};

const SCROLLBAR_HORIZONTAL_COLUMN_WIDTH: f32 = 280.0;
const SCROLLBAR_TEMPLATE_HORIZONTAL_COLUMN_WIDTH: f32 = 160.0;
const SCROLLBAR_TEMPLATE_HORIZONTAL_LENGTH: f32 = 130.0;
const SCROLLBAR_TEMPLATE_TABLE_HEADER_HEIGHT: f32 = 28.0;
const SCROLLBAR_TEMPLATE_TABLE_ROW_HEIGHT: f32 = 52.0;
const SCROLLBAR_TEMPLATE_VARIANT_COLUMN_WIDTH: f32 = 120.0;
const SCROLLBAR_VERTICAL_COLUMN_WIDTH: f32 = 120.0;

#[derive(Clone, Copy)]
struct ScrollbarPreviewSample {
    size: ControlSize,
    id_suffix: &'static str,
    label: &'static str,
    state: xgpui_luma::theme::InteractionState,
    enabled: bool,
    length: Option<f32>,
    style: ScrollbarStyle,
}

pub(crate) fn render_scrollbar_template_section(
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
        "Scrollbar",
        "Horizontal and vertical scrollbar interaction states. Sizes tab: Sm/Md/Lg horizontal thickness.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        render_scrollbar_preview_tabbed_content(look, preview_tabs, active_tab, chrome.border, window, cx),
    )
}

fn render_scrollbar_preview_tabbed_content(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<Tabs>,
    active_tab: SharedString,
    border: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let body = match active_tab.as_ref() {
        "sizes" => render_input_scrollbar_sizes_body(&look, window, cx),
        _ => render_input_scrollbar_body(&look, window, cx),
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

fn render_input_scrollbar_body(look: &Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let chrome = look.chrome();
    let samples = input_interaction_samples();
    let template = look.scrollbar_template();

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(12.0))
        .child(render_input_scrollbar_style_table(&template, "Horizontal", &samples, &chrome, window, cx))
        .child(render_input_scrollbar_row(
            &template,
            "Vertical",
            ScrollbarOrientation::Vertical,
            &samples,
            ScrollbarStyle::Soft,
            chrome.muted_text,
            window,
            cx,
        ))
        .into_any_element()
}

fn render_input_scrollbar_style_table(
    template: &Arc<dyn ScrollbarTemplate>,
    group_label: &'static str,
    samples: &[InputInteractionSample],
    chrome: &xgpui_luma::theme::LumaChrome,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .child(
            VariantStateTable::new(
                VariantStateTableStyle::from_chrome(chrome)
                    .variant_column_width(SCROLLBAR_TEMPLATE_VARIANT_COLUMN_WIDTH)
                    .state_column_width(SCROLLBAR_TEMPLATE_HORIZONTAL_COLUMN_WIDTH)
                    .header_height(SCROLLBAR_TEMPLATE_TABLE_HEADER_HEIGHT)
                    .header_corner_padding_bottom(0.0)
                    .row_height(SCROLLBAR_TEMPLATE_TABLE_ROW_HEIGHT),
            )
            .row_group_label(group_label)
            .column_headers(
                samples.iter().map(|sample| render_input_scrollbar_state_header_cell(*sample, chrome.muted_text)),
            )
            .rows([
                render_input_scrollbar_style_row(template, "Ghost", ScrollbarStyle::Ghost, samples, window, cx),
                render_input_scrollbar_style_row(template, "Soft", ScrollbarStyle::Soft, samples, window, cx),
            ])
            .build(),
        )
        .into_any_element()
}

fn render_input_scrollbar_style_row(
    template: &Arc<dyn ScrollbarTemplate>,
    label: &'static str,
    style: ScrollbarStyle,
    samples: &[InputInteractionSample],
    window: &mut Window,
    cx: &mut App,
) -> VariantStateTableRow {
    VariantStateTableRow {
        label: SharedString::from(label),
        description: SharedString::from(""),
        cells: samples
            .iter()
            .copied()
            .map(|sample| {
                render_input_scrollbar_sample_control(
                    template,
                    ScrollbarOrientation::Horizontal,
                    ScrollbarPreviewSample {
                        size: ControlSize::Md,
                        id_suffix: sample.id,
                        label: sample.label,
                        state: sample.state,
                        enabled: !sample.state.disabled,
                        length: Some(SCROLLBAR_TEMPLATE_HORIZONTAL_LENGTH),
                        style,
                    },
                    window,
                    cx,
                )
            })
            .collect(),
    }
}

fn render_input_scrollbar_state_header_cell(sample: InputInteractionSample, muted_text: gpui::Hsla) -> AnyElement {
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

fn render_input_scrollbar_sizes_body(look: &Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let template = look.scrollbar_template();

    div()
        .w_full()
        .max_w(px(900.0))
        .child(render_input_scrollbar_size_row(
            &template,
            ScrollbarOrientation::Horizontal,
            look.chrome().muted_text,
            window,
            cx,
        ))
        .into_any_element()
}

fn render_input_scrollbar_size_row(
    template: &Arc<dyn ScrollbarTemplate>,
    orientation: ScrollbarOrientation,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .items_start()
        .justify_center()
        .gap(px(16.0))
        .children([(ControlSize::Sm, "Sm"), (ControlSize::Md, "Md"), (ControlSize::Lg, "Lg")].into_iter().map(
            |(size, label)| {
                render_input_scrollbar_sample_with_size(
                    template,
                    orientation,
                    ScrollbarPreviewSample {
                        size,
                        id_suffix: label,
                        label,
                        state: Default::default(),
                        enabled: true,
                        length: None,
                        style: ScrollbarStyle::Soft,
                    },
                    SCROLLBAR_HORIZONTAL_COLUMN_WIDTH,
                    label_color,
                    window,
                    cx,
                )
            },
        ))
        .into_any_element()
}

fn render_input_scrollbar_row(
    template: &Arc<dyn ScrollbarTemplate>,
    row_label: &'static str,
    orientation: ScrollbarOrientation,
    samples: &[InputInteractionSample],
    style: ScrollbarStyle,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let column_width = match orientation {
        ScrollbarOrientation::Horizontal => SCROLLBAR_TEMPLATE_HORIZONTAL_COLUMN_WIDTH,
        ScrollbarOrientation::Vertical => SCROLLBAR_VERTICAL_COLUMN_WIDTH,
    };
    let length = match orientation {
        ScrollbarOrientation::Horizontal => Some(SCROLLBAR_TEMPLATE_HORIZONTAL_LENGTH),
        ScrollbarOrientation::Vertical => None,
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(8.0))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(row_label))
        .child(div().flex().items_start().justify_center().gap(px(12.0)).children(samples.iter().copied().map(
            |sample| {
                render_input_scrollbar_sample(
                    template,
                    orientation,
                    sample,
                    ControlSize::Md,
                    column_width,
                    length,
                    style,
                    label_color,
                    window,
                    cx,
                )
            },
        )))
        .into_any_element()
}

fn render_input_scrollbar_sample(
    template: &Arc<dyn ScrollbarTemplate>,
    orientation: ScrollbarOrientation,
    sample: InputInteractionSample,
    size: ControlSize,
    column_width: f32,
    length: Option<f32>,
    style: ScrollbarStyle,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    render_input_scrollbar_sample_with_size(
        template,
        orientation,
        ScrollbarPreviewSample {
            size,
            id_suffix: sample.id,
            label: sample.label,
            state: sample.state,
            enabled: !sample.state.disabled,
            length,
            style,
        },
        column_width,
        label_color,
        window,
        cx,
    )
    .into_any_element()
}

fn render_input_scrollbar_sample_control(
    template: &Arc<dyn ScrollbarTemplate>,
    orientation: ScrollbarOrientation,
    sample: ScrollbarPreviewSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let orientation_id = match orientation {
        ScrollbarOrientation::Horizontal => "horizontal",
        ScrollbarOrientation::Vertical => "vertical",
    };
    let style_id = match sample.style {
        ScrollbarStyle::Ghost => "ghost",
        ScrollbarStyle::Soft => "soft",
    };
    let id = SharedString::from(format!(
        "luma-studio-scrollbar-preview-{}-{}-{}",
        orientation_id, style_id, sample.id_suffix
    ));
    let range = ControlRange::from(0..220);
    let value = 40.0;
    let model = ScrollbarRenderModel {
        id: &id,
        orientation,
        range,
        step: 20.0,
        page_step: 80.0,
        value,
        percentage: range.percentage(value),
        thumb_fraction: 0.54,
        length: sample.length,
        size: sample.size,
        style: sample.style,
        enabled: sample.enabled,
        state: sample.state,
    };

    template.render(&model, input_scrollbar_handlers(), window, cx).into_any_element()
}

fn render_input_scrollbar_sample_with_size(
    template: &Arc<dyn ScrollbarTemplate>,
    orientation: ScrollbarOrientation,
    sample: ScrollbarPreviewSample,
    column_width: f32,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .justify_start()
        .w(px(column_width))
        .gap(px(6.0))
        .child(render_input_scrollbar_sample_control(template, orientation, sample, window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}
