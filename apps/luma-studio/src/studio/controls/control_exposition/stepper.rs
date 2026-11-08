//! Stepper control exposition — horizontal/vertical samples and interactive navigation.

use std::sync::Arc;

use gpui::{App, Context, Entity, Render, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::button::Button;
use xgpui_luma::infra::icon::SelectionStatusIcons;
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma::controls::progress::ProgressDirection;
use xgpui_luma::controls::stepper::{StepState, Stepper, StepperLabelPlacement, StepperRenderModel, StepperTemplate};
use xgpui_luma::theme::ControlSize;
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::standalone_theme_inspectors::StepperThemeInspector;
use super::inspector::{StepperInspectorAdapter, STEPPER_INSPECTOR_SPEC, stepper_motion_report};
use super::template::render_control_exposition_card;

const DEMO_STEP_COUNT: usize = 4;
const DEMO_LABELS: [&str; 4] = ["Account", "Shipping", "Payment", "Review"];
const DEMO_STEP_BODIES: [&str; 4] = [
    "Create your account credentials and profile basics.",
    "Choose a shipping address and delivery preferences.",
    "Enter payment details to continue the checkout.",
    "Confirm your order summary before finishing.",
];
const SECTION_WIDTH: f32 = 520.0;
const VERTICAL_DEMO_HEIGHT: f32 = 260.0;
const VERTICAL_DEMO_WIDTH: f32 = 200.0;
const DEMO_CONTENT_HEIGHT: f32 = 96.0;
const DEMO_SIZE: ControlSize = ControlSize::Md;

pub struct StepperControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<StepperExpositionLeftPane>,
    theme_inspector: Entity<StepperThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

struct StepperExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    stepper: Stepper,
    start_button: Entity<Button>,
    previous_button: Entity<Button>,
    next_button: Entity<Button>,
    enable_toggle_button: Entity<Button>,
    enabled: bool,
    awaiting_start: bool,
}

impl StepperExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.stepper.update(cx, |stepper, cx| {
            stepper.set_template(look.stepper_template(), cx);
        });
        self.enable_toggle_button.update(cx, |_, cx| cx.notify());
        cx.notify();
    }

    fn handle_previous(&mut self, cx: &mut Context<Self>) {
        let current = self.stepper.read(cx).current_step();
        if current > 0 {
            self.stepper.update(cx, |stepper, cx| stepper.set_current_step(current - 1, cx));
        }
    }

    fn handle_next(&mut self, cx: &mut Context<Self>) {
        if self.awaiting_start {
            self.awaiting_start = false;
            self.stepper.update(cx, |stepper, cx| {
                let step_count = stepper.step_count();
                for index in 0..step_count {
                    stepper.clear_step_state(index, cx);
                }
                stepper.set_current_step(0, cx);
            });
            return;
        }

        let current = self.stepper.read(cx).current_step();
        let last = self.stepper.read(cx).step_count().saturating_sub(1);
        if current < last {
            self.stepper.update(cx, |stepper, cx| stepper.set_current_step(current + 1, cx));
        } else if !self.stepper.read(cx).is_complete() {
            self.stepper.update(cx, |stepper, cx| stepper.complete(cx));
        }
    }

    fn handle_start(&mut self, cx: &mut Context<Self>) {
        self.awaiting_start = true;
        self.stepper.update(cx, |stepper, cx| {
            let step_count = stepper.step_count();
            stepper.set_current_step(0, cx);
            for index in 0..step_count {
                stepper.set_step_state(index, StepState::Incomplete, cx);
            }
        });
    }

    fn toggle_enabled(&mut self, cx: &mut Context<Self>) {
        self.enabled = !self.enabled;
        self.stepper.update(cx, |stepper, cx| stepper.set_enabled(self.enabled, cx));
        let label = if self.enabled { "Disable" } else { "Enable" };
        self.enable_toggle_button.update(cx, |button, cx| button.set_label(label, cx));
        cx.notify();
    }
}

impl Render for StepperExpositionLeftPane {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let look = &self.look;
            let chrome = look.chrome();
            let template = look.stepper_template();

            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_stretch()
                .gap(px(28.0))
                .child(section_block(
                    "Interactive wizard",
                    chrome.muted_text,
                    div()
                        .flex()
                        .flex_col()
                        .items_stretch()
                        .gap(px(12.0))
                        .child(div().w_full().child(self.stepper.clone()))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_center()
                                .gap(px(8.0))
                                .child(self.start_button.clone())
                                .child(self.previous_button.clone())
                                .child(self.next_button.clone())
                                .child(self.enable_toggle_button.clone()),
                        )
                        .into_any_element(),
                    window,
                    cx,
                    &template,
                ))
                .child(
                    div()
                        .w_full()
                        .max_w(px(SECTION_WIDTH))
                        .flex()
                        .flex_col()
                        .items_stretch()
                        .gap(px(8.0))
                        .child(section_label("Vertical directions", chrome.muted_text))
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_start()
                                .justify_center()
                                .gap(px(40.0))
                                .child(vertical_sample_column(
                                    &template,
                                    "btt-start",
                                    "Bottom to top · labels start",
                                    ProgressDirection::BottomToTop,
                                    2,
                                    StepperLabelPlacement::Start,
                                    chrome.muted_text,
                                    window,
                                    cx,
                                ))
                                .child(vertical_sample_column(
                                    &template,
                                    "ttb-end",
                                    "Top to bottom · labels end",
                                    ProgressDirection::TopToBottom,
                                    2,
                                    StepperLabelPlacement::End,
                                    chrome.muted_text,
                                    window,
                                    cx,
                                )),
                        ),
                );

            div()
                .id("controls-doc-stepper-left-pane")
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

impl StepperControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("stepper").expect("stepper catalog entry");
        let mut stepper_builder = shadcn::Stepper::new("controls-doc-stepper", DEMO_STEP_COUNT)
            .look(look.as_ref())
            .current_step(1)
            .labels(DEMO_LABELS.to_vec())
            .size(ShadcnSize::Md)
            .content_height(DEMO_CONTENT_HEIGHT);
        for (index, body) in DEMO_STEP_BODIES.iter().enumerate() {
            let body = *body;
            let title = DEMO_LABELS[index];
            stepper_builder = stepper_builder.step_content(index, move |_, _| {
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .gap(px(4.0))
                    .px(px(12.0))
                    .child(div().text_size(px(14.0)).font_weight(gpui::FontWeight::SEMIBOLD).child(title))
                    .child(div().text_size(px(12.0)).opacity(0.72).child(body))
                    .into_any_element()
            });
        }
        let stepper = stepper_builder.spawn(cx);
        let start_button = shadcn::Button::new("controls-doc-stepper-start")
            .look(look.as_ref())
            .secondary()
            .label("Start")
            .spawn(cx);
        let previous_button = shadcn::Button::new("controls-doc-stepper-previous")
            .look(look.as_ref())
            .secondary()
            .label("Previous")
            .spawn(cx);
        let next_button = shadcn::Button::new("controls-doc-stepper-next")
            .look(look.as_ref())
            .secondary()
            .label("Next")
            .spawn(cx);
        let enable_toggle_button = shadcn::Button::new("controls-doc-stepper-enable-toggle")
            .look(look.as_ref())
            .outline()
            .label("Disable")
            .spawn(cx);

        let left_pane = cx.new(|_| StepperExpositionLeftPane {
            look: look.clone(),
            entry,
            stepper: stepper.clone(),
            start_button: start_button.clone(),
            previous_button: previous_button.clone(),
            next_button: next_button.clone(),
            enable_toggle_button: enable_toggle_button.clone(),
            enabled: true,
            awaiting_start: false,
        });

        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-stepper-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &STEPPER_INSPECTOR_SPEC,
            StepperInspectorAdapter::shared(),
        );
        theme_inspector.update(cx, |inspector, cx| {
            inspector.set_motion_report(stepper_motion_report([("Interactive wizard", stepper)]), cx);
        });

        start_button.clone().on_click(cx, {
            let left_pane = left_pane.downgrade();
            move |_, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };

                left_pane.update(cx, |pane, cx| pane.handle_start(cx));
            }
        });
        previous_button.clone().on_click(cx, {
            let left_pane = left_pane.downgrade();
            move |_, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };

                left_pane.update(cx, |pane, cx| pane.handle_previous(cx));
            }
        });
        next_button.clone().on_click(cx, {
            let left_pane = left_pane.downgrade();
            move |_, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };

                left_pane.update(cx, |pane, cx| pane.handle_next(cx));
            }
        });
        enable_toggle_button.clone().on_click(cx, {
            let left_pane = left_pane.downgrade();
            move |_, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };

                left_pane.update(cx, |pane, cx| pane.toggle_enabled(cx));
            }
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

impl Render for StepperControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-stepper-exposition")
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

fn section_block(
    title: &'static str,
    label_color: gpui::Hsla,
    content: gpui::AnyElement,
    _window: &mut Window,
    _cx: &mut App,
    _template: &Arc<dyn StepperTemplate>,
) -> gpui::AnyElement {
    div()
        .w_full()
        .max_w(px(SECTION_WIDTH))
        .flex()
        .flex_col()
        .items_stretch()
        .gap(px(10.0))
        .child(section_label(title, label_color))
        .child(content)
        .into_any_element()
}

fn vertical_sample_column(
    template: &Arc<dyn StepperTemplate>,
    id: &str,
    caption: &'static str,
    direction: ProgressDirection,
    current_step: usize,
    label_placement: StepperLabelPlacement,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> gpui::AnyElement {
    div()
        .w(px(VERTICAL_DEMO_WIDTH))
        .flex()
        .flex_col()
        .items_stretch()
        .gap(px(8.0))
        .child(section_label(caption, label_color))
        .child(div().w_full().h(px(VERTICAL_DEMO_HEIGHT)).child(render_stepper_sample(
            template,
            id,
            current_step,
            true,
            false,
            direction,
            label_placement,
            true,
            window,
            cx,
        )))
        .into_any_element()
}

fn render_stepper_sample(
    template: &Arc<dyn StepperTemplate>,
    id: &str,
    current_step: usize,
    enabled: bool,
    workflow_complete: bool,
    direction: ProgressDirection,
    label_placement: StepperLabelPlacement,
    with_labels: bool,
    window: &mut Window,
    cx: &mut App,
) -> gpui::AnyElement {
    let sample_id = SharedString::from(format!("controls-doc-stepper-{id}"));
    let step_states = if workflow_complete {
        vec![StepState::Complete; DEMO_STEP_COUNT]
    } else {
        derive_sample_states(DEMO_STEP_COUNT, current_step)
    };
    let labels: Vec<SharedString> = if with_labels {
        DEMO_LABELS.iter().map(|label| SharedString::from(*label)).collect()
    } else {
        Vec::new()
    };
    let empty_contents = Vec::new();
    let model = StepperRenderModel {
        id: &sample_id,
        step_count: DEMO_STEP_COUNT,
        current_step,
        display_step: current_step as f32,
        transition_progress: 1.0,
        from_step: current_step as f32,
        to_step: current_step,
        step_states: &step_states,
        labels: &labels,
        label_placement,
        direction,
        size: DEMO_SIZE,
        enabled,
        step_contents: &empty_contents,
        content_height: None,
        icons: &SelectionStatusIcons::default(),
    };

    template.render(&model, window, cx).into_any_element()
}

fn derive_sample_states(step_count: usize, current_step: usize) -> Vec<StepState> {
    (0..step_count)
        .map(|index| {
            if index < current_step {
                StepState::Complete
            } else if index == current_step {
                StepState::InProgress
            } else {
                StepState::Incomplete
            }
        })
        .collect()
}
