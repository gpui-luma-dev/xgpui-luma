use xgpui_luma::controls::slider::SliderEventExt;
use std::f32::consts::PI;
use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::button::{Button, HasPresenter};
use xgpui_luma::controls::slider::{Slider, SliderEvent, SliderThumbPolicy};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::inspector::{SliderInspectorAdapter, SLIDER_INSPECTOR_SPEC, slider_motion_report};
use super::standalone_theme_inspectors::SliderThemeInspector;
use super::template::render_control_exposition_card;

pub struct SliderControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<SliderExpositionLeftPane>,
    theme_inspector: Entity<SliderThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

struct SliderExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    fill_slider: Slider,
    blocked_slider: Slider,
    reversed_slider: Slider,
    angular_slider: Slider,
    wrapping_slider: Slider,
    stops_slider: Slider,
    animate_button: Entity<Button<()>>,
    event_stream: Entity<ControlEventStream>,
    fill_value: f32,
    blocked_value: f32,
    reversed_value: f32,
    angular_value: f32,
    wrapping_value: f32,
    stop_summary: String,
    stops_interaction_hint: Option<String>,
}

impl SliderExpositionLeftPane {
    fn sync_stop_summary(&mut self, cx: &mut Context<Self>) {
        let slider = self.stops_slider.read(cx);
        let mut stops = slider.thumbs().iter().map(|thumb| slider.range().value_at(thumb.position)).collect::<Vec<_>>();
        stops.sort_by(|left, right| left.total_cmp(right));

        let selected_value = slider
            .active_thumb_id()
            .and_then(|id| slider.thumb_value(id))
            .unwrap_or(stops.first().copied().unwrap_or(0.0));

        self.stop_summary = format!(
            "{} stops [{}] · selected {:.0}",
            stops.len(),
            stops.iter().map(|value| format!("{value:.0}")).collect::<Vec<_>>().join(", "),
            selected_value
        );
    }

    fn handle_stops_event(&mut self, _event: &SliderEvent, cx: &mut Context<Self>) {
        self.sync_stop_summary(cx);
        cx.notify();
    }

    fn toggle_animated_value(&mut self, cx: &mut Context<Self>) {
        let next = self.fill_slider.update(cx, |slider, cx| {
            let next = if slider.value() < 50.0 { 85.0 } else { 15.0 };
            slider.set_value(next, cx);
            next
        });
        self.fill_value = next;
        cx.notify();
    }

    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for slider in [
            &self.fill_slider,
            &self.blocked_slider,
            &self.reversed_slider,
            &self.angular_slider,
            &self.wrapping_slider,
            &self.stops_slider,
        ] {
            slider.update(cx, |_, cx| cx.notify());
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for SliderExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(24.0))
                .child(demo_section(
                    "Fill track",
                    chrome.muted_text,
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(8.0))
                        .child(div().w(px(320.0)).child(self.fill_slider.clone()))
                        .child(self.animate_button.clone())
                        .child(value_label(self.fill_value, chrome.body_text)),
                ))
                .child(demo_section(
                    "Reversed fill track",
                    chrome.muted_text,
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(8.0))
                        .child(div().w(px(320.0)).child(self.reversed_slider.clone()))
                        .child(value_label(self.reversed_value, chrome.body_text)),
                ))
                .child(demo_section(
                    "Blocked intervals",
                    chrome.muted_text,
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(8.0))
                        .child(div().w(px(320.0)).child(self.blocked_slider.clone()))
                        .child(value_label(self.blocked_value, chrome.body_text)),
                ))
                .child(demo_section(
                    "Multi-stop",
                    chrome.muted_text,
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(8.0))
                        .when_some(self.stops_interaction_hint.as_ref(), |this, hint| {
                            this.child(
                                div()
                                    .text_size(px(11.0))
                                    .line_height(px(14.0))
                                    .text_color(chrome.muted_text)
                                    .child(hint.clone()),
                            )
                        })
                        .child(div().w(px(320.0)).child(self.stops_slider.clone()))
                        .child(stops_label(&self.stop_summary, chrome.body_text)),
                ))
                .child(demo_section(
                    "Circular dials",
                    chrome.muted_text,
                    div()
                        .flex()
                        .flex_row()
                        .items_start()
                        .justify_center()
                        .gap(px(32.0))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .text_size(px(11.0))
                                        .line_height(px(14.0))
                                        .text_color(chrome.muted_text)
                                        .child("Angular dial"),
                                )
                                .child(self.angular_slider.clone())
                                .child(value_label(self.angular_value, chrome.body_text)),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .text_size(px(11.0))
                                        .line_height(px(14.0))
                                        .text_color(chrome.muted_text)
                                        .child("Wrapping dial"),
                                )
                                .child(self.wrapping_slider.clone())
                                .child(value_label(self.wrapping_value, chrome.body_text)),
                        ),
                ))
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-slider-left-pane")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .overflow_y_scroll()
                .child(render_control_exposition_card(
                    &self.look,
                    self.entry,
                    preview.into_any_element(),
                    None,
                    ControlExpositionLayout::BORDERLESS,
                ))
        })
    }
}

impl SliderControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("slider").expect("slider catalog entry");
        let stops_policy = SliderThumbPolicy::multi_stop();
        let stops_interaction_hint = multi_stop_interaction_hint(stops_policy);

        let fill_slider = shadcn::Slider::new("controls-doc-slider-fill")
            .look(look.as_ref())
            .range(1..100)
            .step(1)
            .value(41)
            .spawn(cx);
        let blocked_slider = shadcn::Slider::new("controls-doc-slider-blocked")
            .look(look.as_ref())
            .range(0.0..360.0)
            .step(10.0)
            .value(100.0)
            .allowed_intervals(vec![0.0..=120.0, 180.0..=240.0, 300.0..=360.0])
            .spawn(cx);
        let reversed_slider = shadcn::Slider::new("controls-doc-slider-reversed")
            .look(look.as_ref())
            .reversed(true)
            .range(1..100)
            .step(1)
            .value(41)
            .spawn(cx);
        let angular_slider = shadcn::Slider::new("controls-doc-slider-angular")
            .look(look.as_ref())
            .angular(-1.25 * PI, 0.25 * PI)
            .template(look.slider_angular_template())
            .range(0..100)
            .step(1)
            .value(50)
            .spawn(cx);
        let wrapping_slider = shadcn::Slider::new("controls-doc-slider-wrapping")
            .look(look.as_ref())
            .angular(0.0, 2.0 * PI)
            .wrapping(true)
            .domain()
            .template(look.slider_circular_ring_template())
            .range(0.0..360.0)
            .step(1.0)
            .value(180.0)
            .spawn(cx);
        let stops_slider = shadcn::Slider::new("controls-doc-slider-stops")
            .look(look.as_ref())
            .multi_stop()
            .range(0.0..360.0)
            .step(1.0)
            .thumb_values([(0.0, None), (180.0, None), (300.0, None)])
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-slider-event-log",
                "Drag sliders; SliderEvent variants from the fill track appear below.",
            )
        });
        let animate_button = shadcn::Button::new("controls-doc-slider-animate")
            .look(look.as_ref())
            .secondary()
            .label("Animate value")
            .spawn(cx);

        let left_pane = cx.new(|cx| {
            let mut pane = SliderExpositionLeftPane {
                look: look.clone(),
                entry,
                fill_slider: fill_slider.clone(),
                blocked_slider: blocked_slider.clone(),
                reversed_slider: reversed_slider.clone(),
                angular_slider: angular_slider.clone(),
                wrapping_slider: wrapping_slider.clone(),
                stops_slider: stops_slider.clone(),
                animate_button: animate_button.clone(),
                event_stream: event_stream.clone(),
                fill_value: 41.0,
                blocked_value: 100.0,
                reversed_value: 41.0,
                angular_value: 50.0,
                wrapping_value: 180.0,
                stop_summary: String::new(),
                stops_interaction_hint,
            };
            pane.sync_stop_summary(cx);
            pane
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
            "controls-doc-slider-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &SLIDER_INSPECTOR_SPEC,
            SliderInspectorAdapter::shared(),
        );
        theme_inspector.update(cx, |inspector, cx| {
            inspector.set_motion_report(
                slider_motion_report([
                    ("Fill track", fill_slider.clone()),
                    ("Reversed fill track", reversed_slider.clone()),
                    ("Blocked intervals", blocked_slider.clone()),
                    ("Multi-stop", stops_slider.clone()),
                    ("Angular dial", angular_slider.clone()),
                    ("Wrapping dial", wrapping_slider.clone()),
                ]),
                cx,
            );
        });

        wire_value_slider(cx, &fill_slider, "fill", event_stream.clone(), left_pane.clone(), SliderValueTarget::Fill);
        wire_value_slider(
            cx,
            &blocked_slider,
            "blocked",
            event_stream.clone(),
            left_pane.clone(),
            SliderValueTarget::Blocked,
        );
        wire_value_slider(
            cx,
            &reversed_slider,
            "reversed",
            event_stream.clone(),
            left_pane.clone(),
            SliderValueTarget::Reversed,
        );
        wire_value_slider(
            cx,
            &angular_slider,
            "angular",
            event_stream.clone(),
            left_pane.clone(),
            SliderValueTarget::Angular,
        );
        wire_value_slider(
            cx,
            &wrapping_slider,
            "wrapping",
            event_stream.clone(),
            left_pane.clone(),
            SliderValueTarget::Wrapping,
        );
        stops_slider.clone().on_event(cx, {
            let event_stream = event_stream.downgrade();
            let left_pane = left_pane.downgrade();
            move |_, event: &SliderEvent, cx| {
                let Some(event_stream) = event_stream.upgrade() else {
                    return;
                };
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };
                left_pane.update(cx, |pane, cx| pane.handle_stops_event(event, cx));
                if let Some(line) = format_slider_event("stops", event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
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

#[derive(Clone, Copy)]
enum SliderValueTarget {
    Fill,
    Blocked,
    Reversed,
    Angular,
    Wrapping,
}

fn wire_value_slider(
    cx: &mut Context<SliderControlExposition>,
    slider: &Slider,
    label: &'static str,
    event_stream: Entity<ControlEventStream>,
    left_pane: Entity<SliderExpositionLeftPane>,
    target: SliderValueTarget,
) {
    let left_pane = left_pane.downgrade();
    slider.clone().on_event(cx, move |_, event: &SliderEvent, cx| {
        if let Some(value) = slider_release_or_change_value(event) {
            let Some(left_pane) = left_pane.upgrade() else {
                return;
            };
            left_pane.update(cx, |pane, cx| {
                match target {
                    SliderValueTarget::Fill => pane.fill_value = value,
                    SliderValueTarget::Blocked => pane.blocked_value = value,
                    SliderValueTarget::Reversed => pane.reversed_value = value,
                    SliderValueTarget::Angular => pane.angular_value = value,
                    SliderValueTarget::Wrapping => pane.wrapping_value = value,
                }
                cx.notify();
            });
        }
        if matches!(target, SliderValueTarget::Fill)
            && let Some(line) = format_slider_event(label, event)
        {
            event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
        }
    });
}

impl Render for SliderControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-slider-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn slider_release_or_change_value(event: &SliderEvent) -> Option<f32> {
    match event {
        SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } => Some(*value),
        _ => None,
    }
}

fn format_slider_event(source: &str, event: &SliderEvent) -> Option<String> {
    match event {
        SliderEvent::Change { value, .. } => Some(format!("SliderEvent::Change - {source} ({value:.0})")),
        SliderEvent::Release { value, .. } => Some(format!("SliderEvent::Release - {source} ({value:.0})")),
        SliderEvent::DragStart { .. } => Some(format!("SliderEvent::DragStart - {source}")),
        SliderEvent::DragEnd { value, .. } => Some(format!("SliderEvent::DragEnd - {source} ({value:.0})")),
        SliderEvent::ThumbSelected { thumb_id } => {
            Some(format!("SliderEvent::ThumbSelected {{ thumb_id: {thumb_id:?} }}"))
        }
        SliderEvent::FocusChanged { focused } => Some(format!("SliderEvent::FocusChanged {{ focused: {focused} }}")),
        _ => None,
    }
}

fn demo_section(title: &'static str, title_color: gpui::Hsla, content: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(4.0))
        .child(section_label(title, title_color))
        .child(content)
}

fn section_label(label: &'static str, color: gpui::Hsla) -> impl IntoElement {
    div().text_size(px(12.0)).line_height(px(16.0)).text_color(color).child(label)
}

fn value_label(value: f32, color: gpui::Hsla) -> impl IntoElement {
    div()
        .text_size(px(12.0))
        .line_height(px(16.0))
        .text_color(color)
        .child(format!("Value: {:.0}", value))
}

fn stops_label(summary: &str, color: gpui::Hsla) -> impl IntoElement {
    div().text_size(px(12.0)).line_height(px(16.0)).text_color(color).child(summary.to_string())
}

fn multi_stop_interaction_hint(policy: SliderThumbPolicy) -> Option<String> {
    let mut parts = Vec::new();
    if policy.supports_click_to_add() {
        parts.push("Click empty track to add");
    }
    if policy.supports_delete_to_remove() {
        parts.push("Delete/Backspace to remove selected");
    }
    (!parts.is_empty()).then(|| parts.join(" · "))
}
