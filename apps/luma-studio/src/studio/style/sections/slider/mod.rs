pub(crate) mod customization;

use std::sync::Arc;

use gpui::{AnyElement, App, Entity, FontWeight, IntoElement, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::slider::{
    SliderInputStrategy, SliderRenderModel, SliderTemplate, SliderThumbPolicy, SliderThumbRole, SliderThumbValue,
    ThumbId, TrackPresentation, build_track_segments,
};
use xgpui_luma::controls::tabs::Tabs;
use xgpui_luma::infra::value::ControlRange;
use xgpui_luma::theme::{ControlSize, InteractionState};
use xgpui_luma_look_shadcn::{ButtonRadiusPreset, ShadcnButtonStyle, ShadcnLook};

use crate::studio::style::sections::slider::customization::SliderCustomizationPreview;
use crate::studio::style::shared::button_matrix::{
    BUTTON_TABLE_SIZE_HEADER_HEIGHT, BUTTON_TABLE_SIZE_RADIUS_ROW_HEIGHT, BUTTON_TABLE_SIZE_VARIANT_COLUMN_WIDTH,
    CHOICE_STYLE_VARIANTS, render_button_radius_header_cell,
};
use crate::studio::style::shared::input_samples::{InputInteractionSample, input_interaction_samples};
use crate::studio::style::shared::preview_handlers::input_slider_handlers;
use crate::studio::style::shared::shell::section_shell_with_width;
use crate::studio::style::variant_state_table::{VariantStateTable, VariantStateTableRow, VariantStateTableStyle};

const SLIDER_DEMO_WIDTH: f32 = 320.0;
const SLIDER_DEMO_MAX_WIDTH: f32 = 360.0;
/// Template preview keeps table width; sliders are shorter and centered in state columns.
const SLIDER_TEMPLATE_DEMO_WIDTH: f32 = 120.0;
const SLIDER_TABLE_STATE_COLUMN_WIDTH: f32 = 152.0;
const SLIDER_TEMPLATE_TABLE_HEADER_HEIGHT: f32 = 28.0;
const SLIDER_TEMPLATE_TABLE_ROW_HEIGHT: f32 = 52.0;
/// Match template preview state columns so 120px sliders are not clipped.
const SLIDER_TABLE_RADIUS_COLUMN_WIDTH: f32 = 152.0;
const SLIDER_TABLE_SIZE_RADIUS_ROW_HEIGHT: f32 = BUTTON_TABLE_SIZE_RADIUS_ROW_HEIGHT;
const SLIDER_TABLE_SIZE_VARIANT_COLUMN_WIDTH: f32 = BUTTON_TABLE_SIZE_VARIANT_COLUMN_WIDTH;
const SLIDER_TABLE_SIZE_HEADER_HEIGHT: f32 = BUTTON_TABLE_SIZE_HEADER_HEIGHT;
const SLIDER_TABLE_SIZE_HEADER_CORNER_PADDING_BOTTOM: f32 = 8.0;

const SLIDER_SIZES: [(ControlSize, &str); 3] =
    [(ControlSize::Sm, "Small"), (ControlSize::Md, "Medium"), (ControlSize::Lg, "Large")];

pub(crate) fn render_slider_template_section(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<Tabs>,
    customization_preview: Entity<SliderCustomizationPreview>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let active_tab =
        preview_tabs.read(cx).active_id().cloned().unwrap_or_else(|| SharedString::from("template-preview"));

    section_shell_with_width(
        960.0,
        "Slider",
        "Horizontal slider with primary/secondary styles, Sm / Md / Lg sizing, and radius presets.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        render_slider_preview_tabbed_content(
            look,
            preview_tabs,
            customization_preview,
            active_tab,
            chrome.border,
            window,
            cx,
        ),
    )
}

fn render_slider_preview_tabbed_content(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<Tabs>,
    customization_preview: Entity<SliderCustomizationPreview>,
    active_tab: SharedString,
    border: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let body = match active_tab.as_ref() {
        "sizes" => render_input_slider_size_matrix(&look, window, cx),
        "customization" => customization::render_slider_customization_body(customization_preview),
        _ => render_input_slider_states_body(&look, window, cx),
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

fn render_input_slider_states_body(look: &Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let chrome = look.chrome();
    let samples = input_interaction_samples();

    div()
        .flex()
        .flex_col()
        .items_center()
        .child(
            VariantStateTable::new(
                VariantStateTableStyle::from_chrome(&chrome)
                    .state_column_width(SLIDER_TABLE_STATE_COLUMN_WIDTH)
                    .header_height(SLIDER_TEMPLATE_TABLE_HEADER_HEIGHT)
                    .header_corner_padding_bottom(0.0)
                    .row_height(SLIDER_TEMPLATE_TABLE_ROW_HEIGHT),
            )
            .row_group_label("")
            .column_headers(
                samples.iter().map(|sample| render_input_slider_state_header_cell(*sample, chrome.muted_text)),
            )
            .rows(CHOICE_STYLE_VARIANTS.iter().map(|variant| {
                VariantStateTableRow {
                    label: SharedString::from(match variant.style {
                        ShadcnButtonStyle::Secondary => "Secondary*",
                        _ => variant.label,
                    }),
                    description: SharedString::from(variant.description),
                    cells: samples
                        .iter()
                        .copied()
                        .map(|sample| {
                            render_input_slider_state_cell(
                                look,
                                InputInteractionSample { label: "", ..sample },
                                variant.style,
                                ControlSize::Md,
                                None,
                                Some(SLIDER_TEMPLATE_DEMO_WIDTH),
                                1.0,
                                window,
                                cx,
                            )
                        })
                        .collect(),
                }
            }))
            .build(),
        )
        .into_any_element()
}

fn render_input_slider_state_header_cell(sample: InputInteractionSample, muted_text: gpui::Hsla) -> AnyElement {
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

fn render_input_slider_size_matrix(look: &Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let chrome = look.chrome();

    div()
        .flex()
        .flex_col()
        .items_center()
        .child(
            VariantStateTable::new(
                VariantStateTableStyle::from_chrome(&chrome)
                    .variant_column_width(SLIDER_TABLE_SIZE_VARIANT_COLUMN_WIDTH)
                    .state_column_width(SLIDER_TABLE_RADIUS_COLUMN_WIDTH)
                    .header_height(SLIDER_TABLE_SIZE_HEADER_HEIGHT)
                    .header_corner_padding_bottom(SLIDER_TABLE_SIZE_HEADER_CORNER_PADDING_BOTTOM)
                    .row_height(SLIDER_TABLE_SIZE_RADIUS_ROW_HEIGHT),
            )
            .row_group_label("SIZE")
            .column_headers(
                ButtonRadiusPreset::ALL
                    .iter()
                    .map(|preset| render_button_radius_header_cell(preset.label(), chrome.muted_text)),
            )
            .rows(SLIDER_SIZES.iter().map(|(size, label)| {
                VariantStateTableRow {
                    label: SharedString::from(*label),
                    description: SharedString::from(""),
                    cells: ButtonRadiusPreset::ALL
                        .iter()
                        .map(|radius| render_input_slider_size_radius_cell(look, *size, *radius, window, cx))
                        .collect(),
                }
            }))
            .build(),
        )
        .into_any_element()
}

fn render_input_slider_state_cell(
    look: &Arc<ShadcnLook>,
    sample: InputInteractionSample,
    style: ShadcnButtonStyle,
    size: ControlSize,
    radius: Option<ButtonRadiusPreset>,
    demo_width: Option<f32>,
    demo_scale: f32,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let template = look.slider_template_with_style(style);

    div()
        .w_full()
        .h_full()
        .flex()
        .items_center()
        .justify_center()
        .py(px(4.0))
        .child(render_input_slider_sample(
            look,
            &template,
            sample,
            style,
            size,
            radius,
            demo_width,
            demo_scale,
            gpui::transparent_black(),
            window,
            cx,
        ))
        .into_any_element()
}

fn render_input_slider_size_radius_cell(
    look: &Arc<ShadcnLook>,
    size: ControlSize,
    radius: ButtonRadiusPreset,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let sample_id = match size {
        ControlSize::Sm => "size-sm",
        ControlSize::Md => "size-md",
        ControlSize::Lg => "size-lg",
    };
    let sample = InputInteractionSample { id: sample_id, label: "", state: InteractionState::default() };

    render_input_slider_state_cell(
        look,
        sample,
        ShadcnButtonStyle::Primary,
        size,
        Some(radius),
        Some(SLIDER_TEMPLATE_DEMO_WIDTH),
        1.0,
        window,
        cx,
    )
}

fn render_input_slider_sample(
    look: &Arc<ShadcnLook>,
    template: &Arc<dyn SliderTemplate>,
    sample: InputInteractionSample,
    style: ShadcnButtonStyle,
    size: ControlSize,
    radius: Option<ButtonRadiusPreset>,
    demo_width: Option<f32>,
    demo_scale: f32,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!(
        "luma-studio-slider-preview-{}-{}-{}-{}",
        sample.id,
        slider_style_id(style),
        slider_size_id(size),
        radius.map(slider_radius_id).unwrap_or("default")
    ));
    let range = ControlRange::from(1..100);
    let value = 41.0;
    let position = range.percentage(value);
    let thumb_id = ThumbId::next();
    let thumbs = [SliderThumbValue { id: thumb_id, position, preview: None, role: SliderThumbRole::Value }];
    let track_segments = build_track_segments(TrackPresentation::Fill, position, &[], range);
    let corner_radius = radius.map(|preset| px(slider_track_radius_px(look, size, preset)).into());
    let thumb_radius = radius.map(|preset| px(slider_thumb_radius_px(look, size, preset)).into());
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
        corner_radius,
        thumb_radius,
        thumb_policy: SliderThumbPolicy::default(),
        active_thumb_id: Some(thumb_id),
        state: sample.state,
        domain_track: None,
    };

    let demo_width = demo_width.unwrap_or(SLIDER_DEMO_WIDTH * demo_scale);
    let demo_max_width = demo_width * (SLIDER_DEMO_MAX_WIDTH / SLIDER_DEMO_WIDTH);

    let mut root = div().flex().flex_col().items_center().gap(px(6.0)).child(
        div().w(px(demo_width)).max_w(px(demo_max_width)).child(template.render(
            &model,
            input_slider_handlers(),
            thumb_id,
            window,
            cx,
        )),
    );

    if !sample.label.is_empty() {
        root = root.child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label));
    }

    root.into_any_element()
}

fn slider_size_id(size: ControlSize) -> &'static str {
    match size {
        ControlSize::Sm => "sm",
        ControlSize::Md => "md",
        ControlSize::Lg => "lg",
    }
}

fn slider_style_id(style: ShadcnButtonStyle) -> &'static str {
    match style {
        ShadcnButtonStyle::Primary => "primary",
        ShadcnButtonStyle::Secondary => "secondary",
        ShadcnButtonStyle::Outline => "outline",
        ShadcnButtonStyle::Ghost => "ghost",
        ShadcnButtonStyle::ContentOnly => "content-only",
    }
}

fn slider_radius_id(radius: ButtonRadiusPreset) -> &'static str {
    match radius {
        ButtonRadiusPreset::None => "none",
        ButtonRadiusPreset::Small => "small",
        ButtonRadiusPreset::Medium => "medium",
        ButtonRadiusPreset::Large => "large",
        ButtonRadiusPreset::Full => "full",
    }
}

fn slider_thumb_radius_px(look: &ShadcnLook, size: ControlSize, preset: ButtonRadiusPreset) -> f32 {
    let (metrics, look) = slider_size_metrics(look, size);
    xgpui_luma_look_shadcn::paint::resolve_slider_thumb_radius_preset(preset, &metrics, look.thumb_size)
}

fn slider_track_radius_px(look: &ShadcnLook, size: ControlSize, preset: ButtonRadiusPreset) -> f32 {
    let (metrics, look) = slider_size_metrics(look, size);
    xgpui_luma_look_shadcn::paint::resolve_slider_track_radius_preset(preset, &metrics, look.track_height)
}

fn slider_size_metrics(
    look: &ShadcnLook,
    size: ControlSize,
) -> (xgpui_luma::theme::MetricTokens, xgpui_luma::controls::slider::SliderLook) {
    use xgpui_luma_look_shadcn::paint::slider_look;

    let tokens = look.mode_tokens();
    let slider_look =
        slider_look(tokens.as_ref(), look.mode(), ShadcnButtonStyle::Primary, size, None, InteractionState::default());
    (tokens.metrics, slider_look)
}
