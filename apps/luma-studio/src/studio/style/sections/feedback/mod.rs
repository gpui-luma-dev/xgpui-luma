use std::sync::Arc;

use gpui::{AnyElement, App, FontWeight, IntoElement, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::progress::{ProgressDirection, ProgressRenderModel, ProgressTemplate};
use xgpui_luma::infra::value::ControlRange;
use xgpui_luma::theme::ControlSize;
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn::{ShadcnLook};
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::style::shared::shell::section_shell_with_width;

const LINEAR_DEMO_WIDTH: f32 = 480.0;
const LINEAR_VERTICAL_HEIGHT: f32 = 200.0;
const LINEAR_VERTICAL_WIDTH: f32 = 48.0;
const LINEAR_DEMO_SIZE: ControlSize = ControlSize::Lg;

fn direction_label(direction: ProgressDirection) -> &'static str {
    match direction {
        ProgressDirection::LeftToRight => "Left to right",
        ProgressDirection::RightToLeft => "Right to left",
        ProgressDirection::BottomToTop => "Bottom to top",
        ProgressDirection::TopToBottom => "Top to bottom",
    }
}

#[derive(Clone, Copy)]
struct ProgressStateSample {
    id: &'static str,
    label: &'static str,
    value: f32,
    enabled: bool,
}

pub(crate) fn render_feedback_template_section(look: Arc<ShadcnLook>, window: &mut Window, cx: &mut App) -> AnyElement {
    let chrome = look.chrome();
    let circular_template = look.progress_template();
    let linear_template = look.linear_progress_template();

    section_shell_with_width(
        960.0,
        "Feedback",
        "Badge and progress previews.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        div()
            .flex()
            .flex_col()
            .gap(px(20.0))
            .child(render_feedback_heading("Badges", chrome.muted_text))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_start()
                    .gap(px(14.0))
                    .child(div().flex().flex_wrap().items_center().gap(px(10.0)).children([
                        look.badge("work").variant(BadgeVariant::Default).into_any_element(),
                        look.badge("budget").variant(BadgeVariant::Outline).into_any_element(),
                    ]))
                    .child(div().flex().flex_wrap().items_center().gap(px(10.0)).children([
                        look.badge("Default").variant(BadgeVariant::Default).into_any_element(),
                        look.badge("Secondary").variant(BadgeVariant::Secondary).into_any_element(),
                        look.badge("Outline").variant(BadgeVariant::Outline).into_any_element(),
                        look.badge("Ghost").variant(BadgeVariant::Ghost).into_any_element(),
                    ]))
                    .child(div().flex().flex_wrap().items_center().gap(px(10.0)).children([
                        look.badge("Small").size(ShadcnSize::Sm).variant(BadgeVariant::Secondary).into_any_element(),
                        look.badge("Medium").size(ShadcnSize::Md).variant(BadgeVariant::Secondary).into_any_element(),
                        look.badge("Large").size(ShadcnSize::Lg).variant(BadgeVariant::Secondary).into_any_element(),
                    ]))
                    .child(
                        div().flex().flex_wrap().items_center().gap(px(10.0)).children([
                            look.badge("Verified")
                                .variant(BadgeVariant::Default)
                                .start_icon(LucideIcon::BadgeCheck)
                                .into_any_element(),
                            look.badge("Updated")
                                .variant(BadgeVariant::Secondary)
                                .end_icon(LucideIcon::ArrowUpRight)
                                .into_any_element(),
                            look.badge("Draft")
                                .variant(BadgeVariant::Outline)
                                .start_icon(LucideIcon::Pencil)
                                .into_any_element(),
                            look.badge("Muted")
                                .variant(BadgeVariant::Ghost)
                                .end_icon(LucideIcon::Dot)
                                .into_any_element(),
                        ]),
                    ),
            )
            .child(render_feedback_heading("Progress — circular", chrome.muted_text))
            .child(div().flex().flex_wrap().items_start().gap(px(12.0)).children(
                feedback_progress_samples().into_iter().map(|sample| {
                    render_feedback_progress_sample(
                        &circular_template,
                        sample,
                        ControlSize::Md,
                        ProgressDirection::LeftToRight,
                        false,
                        chrome.muted_text,
                        window,
                        cx,
                    )
                }),
            ))
            .child(div().flex().flex_wrap().items_start().gap(px(12.0)).children([
                render_feedback_progress_sample(
                    &circular_template,
                    ProgressStateSample { id: "size-sm", label: "Sm", value: 50.0, enabled: true },
                    ControlSize::Sm,
                    ProgressDirection::LeftToRight,
                    false,
                    chrome.muted_text,
                    window,
                    cx,
                ),
                render_feedback_progress_sample(
                    &circular_template,
                    ProgressStateSample { id: "size-md", label: "Md", value: 50.0, enabled: true },
                    ControlSize::Md,
                    ProgressDirection::LeftToRight,
                    false,
                    chrome.muted_text,
                    window,
                    cx,
                ),
                render_feedback_progress_sample(
                    &circular_template,
                    ProgressStateSample { id: "size-lg", label: "Lg", value: 50.0, enabled: true },
                    ControlSize::Lg,
                    ProgressDirection::LeftToRight,
                    false,
                    chrome.muted_text,
                    window,
                    cx,
                ),
            ]))
            .child(render_feedback_heading("Progress — linear", chrome.muted_text))
            .child(div().flex().flex_col().items_stretch().gap(px(14.0)).children([
                render_feedback_linear_sample(
                    &linear_template,
                    "linear-ltr",
                    ProgressDirection::LeftToRight,
                    true,
                    chrome.muted_text,
                    window,
                    cx,
                ),
                render_feedback_linear_sample(
                    &linear_template,
                    "linear-rtl",
                    ProgressDirection::RightToLeft,
                    true,
                    chrome.muted_text,
                    window,
                    cx,
                ),
            ]))
            .child(div().flex().items_end().gap(px(32.0)).children([
                render_feedback_linear_vertical_sample(
                    &linear_template,
                    "linear-btt",
                    ProgressDirection::BottomToTop,
                    true,
                    chrome.muted_text,
                    window,
                    cx,
                ),
                render_feedback_linear_vertical_sample(
                    &linear_template,
                    "linear-ttb",
                    ProgressDirection::TopToBottom,
                    true,
                    chrome.muted_text,
                    window,
                    cx,
                ),
            ]))
            .child(div().flex().flex_col().items_stretch().gap(px(14.0)).children([
                render_feedback_linear_sample_with_label(
                    &linear_template,
                    "linear-thumb",
                    "With thumb",
                    ProgressDirection::LeftToRight,
                    true,
                    chrome.muted_text,
                    window,
                    cx,
                ),
                render_feedback_linear_sample_with_label(
                    &linear_template,
                    "linear-plain",
                    "Without thumb",
                    ProgressDirection::LeftToRight,
                    false,
                    chrome.muted_text,
                    window,
                    cx,
                ),
            ]))
            .into_any_element(),
    )
}

fn render_feedback_heading(title: &'static str, muted_text: gpui::Hsla) -> AnyElement {
    div()
        .text_size(px(12.0))
        .line_height(px(16.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(muted_text)
        .child(title)
        .into_any_element()
}

fn render_feedback_progress_sample(
    template: &Arc<dyn ProgressTemplate>,
    sample: ProgressStateSample,
    size: ControlSize,
    direction: ProgressDirection,
    show_thumb: bool,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("luma-studio-feedback-progress-{}", sample.id));
    let range = ControlRange::from(0..100);
    let percentage = range.percentage(sample.value);
    let model = ProgressRenderModel {
        id: &id,
        range,
        value: sample.value,
        percentage,
        size,
        enabled: sample.enabled,
        direction,
        show_thumb,
        indeterminate: false,
        phase: 0.0,
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}

fn render_feedback_linear_sample(
    template: &Arc<dyn ProgressTemplate>,
    id: &str,
    direction: ProgressDirection,
    show_thumb: bool,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    render_feedback_linear_sample_with_label(
        template,
        id,
        direction_label(direction),
        direction,
        show_thumb,
        label_color,
        window,
        cx,
    )
}

fn render_feedback_linear_sample_with_label(
    template: &Arc<dyn ProgressTemplate>,
    id: &str,
    label: &'static str,
    direction: ProgressDirection,
    show_thumb: bool,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let sample_id = SharedString::from(format!("luma-studio-feedback-{id}"));
    let range = ControlRange::from(0..100);
    let value = 50.0;
    let model = ProgressRenderModel {
        id: &sample_id,
        range,
        value,
        percentage: range.percentage(value),
        size: LINEAR_DEMO_SIZE,
        enabled: true,
        direction,
        show_thumb,
        indeterminate: false,
        phase: 0.0,
    };

    div()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(div().text_size(px(12.0)).line_height(px(16.0)).text_color(label_color).child(label))
        .child(div().w(px(LINEAR_DEMO_WIDTH)).child(template.render(&model, window, cx)))
        .into_any_element()
}

fn render_feedback_linear_vertical_sample(
    template: &Arc<dyn ProgressTemplate>,
    id: &str,
    direction: ProgressDirection,
    show_thumb: bool,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    render_feedback_linear_vertical_sample_with_label(
        template,
        id,
        direction_label(direction),
        direction,
        show_thumb,
        label_color,
        window,
        cx,
    )
}

fn render_feedback_linear_vertical_sample_with_label(
    template: &Arc<dyn ProgressTemplate>,
    id: &str,
    label: &'static str,
    direction: ProgressDirection,
    show_thumb: bool,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let sample_id = SharedString::from(format!("luma-studio-feedback-{id}"));
    let range = ControlRange::from(0..100);
    let value = 50.0;
    let model = ProgressRenderModel {
        id: &sample_id,
        range,
        value,
        percentage: range.percentage(value),
        size: LINEAR_DEMO_SIZE,
        enabled: true,
        direction,
        show_thumb,
        indeterminate: false,
        phase: 0.0,
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(div().text_size(px(12.0)).line_height(px(16.0)).text_color(label_color).child(label))
        .child(
            div()
                .w(px(LINEAR_VERTICAL_WIDTH))
                .h(px(LINEAR_VERTICAL_HEIGHT))
                .child(template.render(&model, window, cx)),
        )
        .into_any_element()
}

fn feedback_progress_samples() -> [ProgressStateSample; 5] {
    [
        ProgressStateSample { id: "empty", label: "Empty", value: 0.0, enabled: true },
        ProgressStateSample { id: "quarter", label: "25%", value: 25.0, enabled: true },
        ProgressStateSample { id: "half", label: "50%", value: 50.0, enabled: true },
        ProgressStateSample { id: "complete", label: "Complete", value: 100.0, enabled: true },
        ProgressStateSample { id: "disabled", label: "Disabled", value: 50.0, enabled: false },
    ]
}
