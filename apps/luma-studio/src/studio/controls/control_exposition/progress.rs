//! Progress control exposition — circular ring and linear bar samples.

use std::sync::Arc;

use gpui::{App, Context, Entity, Render, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::button::{Button, HasPresenter};
use xgpui_luma::controls::progress::{Progress, ProgressDirection, ProgressRenderModel, ProgressTemplate};
use xgpui_luma::infra::value::ControlRange;
use xgpui_luma::theme::ControlSize;
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::inspector::{ProgressInspectorAdapter, PROGRESS_INSPECTOR_SPEC, progress_motion_report};
use super::standalone_theme_inspectors::ProgressThemeInspector;
use super::template::render_control_exposition_card;

const LINEAR_DEMO_WIDTH: f32 = 420.0;
const LINEAR_VERTICAL_HEIGHT: f32 = 200.0;
const LINEAR_VERTICAL_WIDTH: f32 = 48.0;
const LINEAR_TWO_COLUMN_GAP: f32 = 32.0;
const LINEAR_SECTION_MAX_WIDTH: f32 = LINEAR_DEMO_WIDTH + LINEAR_TWO_COLUMN_GAP + 160.0;
const LINEAR_DEMO_SIZE: ControlSize = ControlSize::Lg;

pub struct ProgressControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<ProgressExpositionLeftPane>,
    theme_inspector: Entity<ProgressThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

struct ProgressExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    linear_progress: Progress,
    indeterminate_linear: Progress,
    indeterminate_circular: Progress,
    animate_button: Entity<Button<()>>,
}

impl ProgressExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.linear_progress.update(cx, |progress, cx| {
            progress.set_template(look.linear_progress_template(), cx);
        });
        self.indeterminate_linear.update(cx, |progress, cx| {
            progress.set_template(look.linear_progress_template(), cx);
        });
        self.indeterminate_circular.update(cx, |progress, cx| {
            progress.set_template(look.progress_template(), cx);
        });
        cx.notify();
    }

    fn toggle_animated_value(&mut self, cx: &mut Context<Self>) {
        self.linear_progress.update(cx, |progress, cx| {
            let next = if progress.value() < 50.0 { 88.0 } else { 18.0 };
            progress.set_value(next, cx);
        });
    }
}

impl Render for ProgressExpositionLeftPane {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let look = &self.look;
            let chrome = look.chrome();
            let circular_template = look.progress_template();
            let linear_template = look.linear_progress_template();

            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(16.0))
                .child(
                    div()
                        .w_full()
                        .max_w(px(LINEAR_SECTION_MAX_WIDTH))
                        .flex()
                        .flex_col()
                        .items_stretch()
                        .gap(px(8.0))
                        .child(section_label("Linear", chrome.muted_text))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_start()
                                .gap(px(LINEAR_TWO_COLUMN_GAP))
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .items_stretch()
                                        .gap(px(14.0))
                                        .child(div().w(px(LINEAR_DEMO_WIDTH)).child(self.linear_progress.clone()))
                                        .child(self.animate_button.clone())
                                        .child(section_label("Indeterminate", chrome.muted_text))
                                        .child(div().w(px(LINEAR_DEMO_WIDTH)).child(self.indeterminate_linear.clone()))
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap(px(12.0))
                                                .child(self.indeterminate_circular.clone()),
                                        )
                                        .child(render_linear_direction_sample(
                                            &linear_template,
                                            "rtl",
                                            ProgressDirection::RightToLeft,
                                            true,
                                            chrome.muted_text,
                                            window,
                                            cx,
                                        ))
                                        .child(section_label("Hide Thumb", chrome.muted_text))
                                        .child(render_linear_direction_sample_with_label(
                                            &linear_template,
                                            "thumb-off",
                                            None,
                                            ProgressDirection::LeftToRight,
                                            false,
                                            chrome.muted_text,
                                            window,
                                            cx,
                                        )),
                                )
                                .child(render_vertical_direction_row(&linear_template, chrome.muted_text, window, cx)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(8.0))
                        .child(section_label("Circular value samples", chrome.muted_text))
                        .child(
                            div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                                [
                                    ("empty", "Empty", 0.0, true),
                                    ("quarter", "25%", 25.0, true),
                                    ("half", "50%", 50.0, true),
                                    ("complete", "Complete", 100.0, true),
                                    ("disabled", "Disabled", 50.0, false),
                                ]
                                .into_iter()
                                .map(|(id, label, value, enabled)| {
                                    render_progress_sample(
                                        ProgressSample {
                                            template: &circular_template,
                                            id,
                                            label,
                                            value,
                                            enabled,
                                            label_color: chrome.muted_text,
                                            direction: ProgressDirection::LeftToRight,
                                            show_thumb: false,
                                        },
                                        window,
                                        cx,
                                    )
                                }),
                            ),
                        ),
                );

            div()
                .id("controls-doc-progress-left-pane")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .overflow_y_scroll()
                .child(render_control_exposition_card(
                    look,
                    self.entry,
                    preview.into_any_element(),
                    None,
                    ControlExpositionLayout::BORDERLESS,
                ))
        })
    }
}

impl ProgressControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("progress").expect("progress catalog entry");
        let linear_progress = shadcn::Progress::linear("controls-doc-progress")
            .look(look.as_ref())
            .range(1..100)
            .value(41)
            .size(ShadcnSize::Lg)
            .show_thumb(true)
            .spawn(cx);
        let indeterminate_linear = shadcn::Progress::linear("controls-doc-progress-indeterminate-linear")
            .look(look.as_ref())
            .size(ShadcnSize::Lg)
            .indeterminate(true)
            .spawn(cx);
        let indeterminate_circular = shadcn::Progress::new("controls-doc-progress-indeterminate-circular")
            .look(look.as_ref())
            .indeterminate(true)
            .spawn(cx);
        let animate_button = shadcn::Button::new("controls-doc-progress-animate")
            .look(look.as_ref())
            .secondary()
            .label("Animate value")
            .spawn(cx);
        let left_pane = cx.new(|_| ProgressExpositionLeftPane {
            look: look.clone(),
            entry,
            linear_progress: linear_progress.clone(),
            indeterminate_linear: indeterminate_linear.clone(),
            indeterminate_circular: indeterminate_circular.clone(),
            animate_button: animate_button.clone(),
        });

        animate_button.clone().on_click(cx, {
            let left_pane = left_pane.downgrade();
            move |_, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };

                left_pane.update(cx, |pane, cx| pane.toggle_animated_value(cx));
            }
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-progress-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &PROGRESS_INSPECTOR_SPEC,
            ProgressInspectorAdapter::shared(),
        );
        theme_inspector.update(cx, |inspector, cx| {
            inspector.set_motion_report(
                progress_motion_report([
                    ("Linear", linear_progress),
                    ("Indeterminate", indeterminate_linear),
                    ("Indeterminate circular", indeterminate_circular),
                ]),
                cx,
            );
        });

        Self { look, entry, left_pane, theme_inspector, inspector_split }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn fills_viewport(&self) -> bool {
        true
    }

    pub fn request_layout_refresh(&mut self, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.request_layout_refresh(cx));
    }

    pub fn set_viewport_size(&mut self, size: gpui::Size<gpui::Pixels>, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.set_viewport_size(size, cx));
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        sync_viewport_inspector(look, &self.theme_inspector, &self.inspector_split, cx);
        cx.notify();
    }
}

impl Render for ProgressControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-progress-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn section_label(label: &'static str, color: gpui::Hsla) -> impl IntoElement {
    div()
        .text_size(px(12.0))
        .line_height(px(16.0))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(color)
        .child(label)
}

struct ProgressSample<'a> {
    template: &'a Arc<dyn ProgressTemplate>,
    id: &'a str,
    label: &'static str,
    value: f32,
    enabled: bool,
    label_color: gpui::Hsla,
    direction: ProgressDirection,
    show_thumb: bool,
}

fn render_progress_sample(sample: ProgressSample<'_>, window: &mut Window, cx: &mut App) -> gpui::AnyElement {
    let ProgressSample { template, id, label, value, enabled, label_color, direction, show_thumb } = sample;
    let id = SharedString::from(format!("controls-doc-progress-{id}"));
    let range = ControlRange::from(0..100);
    let model = ProgressRenderModel {
        id: &id,
        range,
        value,
        percentage: range.percentage(value),
        size: ControlSize::Md,
        enabled,
        direction,
        show_thumb,
        indeterminate: false,
        phase: 0.0,
    };

    div()
        .w(px(120.0))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(label))
        .into_any_element()
}

fn direction_label(direction: ProgressDirection) -> &'static str {
    match direction {
        ProgressDirection::LeftToRight => "Left to right",
        ProgressDirection::RightToLeft => "Right to left",
        ProgressDirection::BottomToTop => "Bottom to top",
        ProgressDirection::TopToBottom => "Top to bottom",
    }
}

fn render_linear_direction_sample(
    template: &Arc<dyn ProgressTemplate>,
    id: &str,
    direction: ProgressDirection,
    show_thumb: bool,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> gpui::AnyElement {
    render_linear_direction_sample_with_label(
        template,
        id,
        Some(direction_label(direction)),
        direction,
        show_thumb,
        label_color,
        window,
        cx,
    )
}

fn render_linear_direction_sample_with_label(
    template: &Arc<dyn ProgressTemplate>,
    id: &str,
    label: Option<&'static str>,
    direction: ProgressDirection,
    show_thumb: bool,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> gpui::AnyElement {
    let sample_id = SharedString::from(format!("controls-doc-progress-linear-{id}"));
    let range = ControlRange::from(0..100);
    let value = 41.0;
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

    let track = if direction.orientation() == xgpui_luma::controls::progress::ProgressOrientation::Vertical {
        div()
            .w(px(LINEAR_VERTICAL_WIDTH))
            .h(px(LINEAR_VERTICAL_HEIGHT))
            .child(template.render(&model, window, cx))
    } else {
        div().w(px(LINEAR_DEMO_WIDTH)).child(template.render(&model, window, cx))
    };

    let mut sample = div().flex().flex_col().gap(px(6.0));
    if let Some(label) = label {
        sample = sample.child(div().text_size(px(12.0)).line_height(px(16.0)).text_color(label_color).child(label));
    }
    sample.child(track).into_any_element()
}

fn render_vertical_direction_row(
    template: &Arc<dyn ProgressTemplate>,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> gpui::AnyElement {
    div()
        .flex()
        .flex_row()
        .items_end()
        .gap(px(32.0))
        .child(render_linear_direction_sample(
            template,
            "btt",
            ProgressDirection::BottomToTop,
            true,
            label_color,
            window,
            cx,
        ))
        .child(render_linear_direction_sample(
            template,
            "ttb",
            ProgressDirection::TopToBottom,
            true,
            label_color,
            window,
            cx,
        ))
        .into_any_element()
}
