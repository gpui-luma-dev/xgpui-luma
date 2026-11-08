use xgpui_luma::controls::slider::SliderEventExt;
use std::f32::consts::PI;
use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, IntoElement, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::slider::{Slider, SliderEvent, SliderThumbPolicy, ThumbId};
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;

/// Vertical gap between label, control, and value within each customization demo.
const DEMO_STACK_GAP: f32 = 4.0;
/// Partial compensation for empty space above the painted dial track
/// (`DIAL_SIZE / 2 - TRACK_RADIUS` ≈ 28). Keep some inset so dials aren't
/// as tight as a full pull-up, closer to linear demo label spacing.
const DIAL_TOP_INSET: f32 = 12.0;

pub(crate) struct SliderCustomizationPreview {
    look: Arc<ShadcnLook>,
    reversed_slider: Slider,
    blocked_slider: Slider,
    stops_slider: Slider,
    angular_slider: Slider,
    wrapping_slider: Slider,
    reversed_value: f32,
    blocked_value: f32,
    angular_value: f32,
    wrapping_value: f32,
    selected_stop: Option<ThumbId>,
    stop_summary: String,
    stops_interaction_hint: Option<String>,
    _subscriptions: Vec<gpui::Subscription>,
}

impl SliderCustomizationPreview {
    pub(crate) fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let stops_policy = SliderThumbPolicy::multi_stop();
        let reversed_slider = shadcn::Slider::new("luma-studio-slider-reversed")
            .look(look.as_ref())
            .reversed(true)
            .range(1..100)
            .step(1)
            .value(41)
            .spawn(cx);
        let blocked_slider = shadcn::Slider::new("luma-studio-slider-blocked")
            .look(look.as_ref())
            .range(0.0..360.0)
            .step(10.0)
            .value(100.0)
            .allowed_intervals(vec![0.0..=120.0, 180.0..=240.0, 300.0..=360.0])
            .spawn(cx);
        let stops_slider = shadcn::Slider::new("luma-studio-slider-stops")
            .look(look.as_ref())
            .multi_stop()
            .range(0.0..360.0)
            .step(1.0)
            .thumb_values([(0.0, None), (180.0, None), (300.0, None)])
            .spawn(cx);
        let angular_slider = shadcn::Slider::new("luma-studio-slider-angular")
            .look(look.as_ref())
            .angular(-1.25 * PI, 0.25 * PI)
            .template(look.slider_angular_template())
            .range(0..100)
            .step(1)
            .value(50)
            .spawn(cx);
        let wrapping_slider = shadcn::Slider::new("luma-studio-slider-wrapping")
            .look(look.as_ref())
            .angular(0.0, 2.0 * PI)
            .wrapping(true)
            .domain()
            .template(look.slider_circular_ring_template())
            .range(0.0..360.0)
            .step(1.0)
            .value(180.0)
            .spawn(cx);

        let mut preview = Self {
            look,
            reversed_slider,
            blocked_slider,
            stops_slider,
            angular_slider,
            wrapping_slider,
            reversed_value: 41.0,
            blocked_value: 100.0,
            angular_value: 50.0,
            wrapping_value: 180.0,
            selected_stop: None,
            stop_summary: String::new(),
            stops_interaction_hint: multi_stop_interaction_hint(stops_policy),
            _subscriptions: Vec::new(),
        };
        preview.sync_stop_summary(cx);
        preview.subscribe(cx);
        preview
    }

    pub(crate) fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.reversed_slider.update(cx, |slider, cx| {
            slider.set_template(look.slider_template(), cx);
        });
        self.blocked_slider.update(cx, |slider, cx| {
            slider.set_template(look.slider_template(), cx);
        });
        self.stops_slider.update(cx, |slider, cx| {
            slider.set_template(look.slider_template(), cx);
        });
        self.angular_slider.update(cx, |slider, cx| {
            slider.set_template(look.slider_angular_template(), cx);
        });
        self.wrapping_slider.update(cx, |slider, cx| {
            slider.set_template(look.slider_circular_ring_template(), cx);
        });
        cx.notify();
    }

    fn subscribe(&mut self, cx: &mut Context<Self>) {
        self.reversed_slider.clone().on_event(cx, |this, event: &SliderEvent, cx| {
            if let SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } = event {
                this.reversed_value = *value;
                cx.notify();
            }
        });
        self.blocked_slider.clone().on_event(cx, |this, event: &SliderEvent, cx| {
            if let SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } = event {
                this.blocked_value = *value;
                cx.notify();
            }
        });
        self.angular_slider.clone().on_event(cx, |this, event: &SliderEvent, cx| {
            if let SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } = event {
                this.angular_value = *value;
                cx.notify();
            }
        });
        self.wrapping_slider.clone().on_event(cx, |this, event: &SliderEvent, cx| {
            if let SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } = event {
                this.wrapping_value = *value;
                cx.notify();
            }
        });
        self.stops_slider.clone().on_event(cx, |this, event: &SliderEvent, cx| {
            if let SliderEvent::ThumbSelected { thumb_id } = event {
                this.selected_stop = Some(*thumb_id);
            }
            this.sync_stop_summary(cx);
            cx.notify();
        });
    }

    fn sync_stop_summary(&mut self, cx: &mut Context<Self>) {
        let slider = self.stops_slider.read(cx);
        let mut stops = slider.thumbs().iter().map(|thumb| slider.range().value_at(thumb.position)).collect::<Vec<_>>();
        stops.sort_by(|left, right| left.total_cmp(right));

        self.selected_stop = slider.active_thumb_id();
        let selected_value = self
            .selected_stop
            .and_then(|id| slider.thumb_value(id))
            .unwrap_or(stops.first().copied().unwrap_or(0.0));

        self.stop_summary = format!(
            "{} stops [{}] · selected {:.0}",
            stops.len(),
            stops.iter().map(|value| format!("{value:.0}")).collect::<Vec<_>>().join(", "),
            selected_value
        );
    }
}

impl Render for SliderCustomizationPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let stops_hint = self.stops_interaction_hint.clone();
        let stop_summary = self.stop_summary.clone();

        div()
            .w_full()
            .flex()
            .flex_row()
            .items_start()
            .justify_center()
            .gap_8()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_4()
                    .child(demo_block(
                        "Reversed fill track",
                        chrome.muted_text,
                        div().w(px(320.0)).child(self.reversed_slider.clone()),
                        value_label(self.reversed_value, chrome.body_text),
                        None,
                    ))
                    .child(demo_block(
                        "Blocked intervals",
                        chrome.muted_text,
                        div().w(px(320.0)).child(self.blocked_slider.clone()),
                        value_label(self.blocked_value, chrome.body_text),
                        None,
                    ))
                    .child(demo_block(
                        "Multi-stop",
                        chrome.muted_text,
                        div().w(px(320.0)).child(self.stops_slider.clone()),
                        stops_label(&stop_summary, chrome.body_text),
                        stops_hint.map(|hint| {
                            div()
                                .text_size(px(11.0))
                                .line_height(px(14.0))
                                .text_color(chrome.muted_text)
                                .child(hint)
                                .into_any_element()
                        }),
                    )),
            )
            .child(demo_block(
                "Angular dial",
                chrome.muted_text,
                div().mt(px(-DIAL_TOP_INSET)).child(self.angular_slider.clone()),
                value_label(self.angular_value, chrome.body_text),
                None,
            ))
            .child(demo_block(
                "Wrapping dial",
                chrome.muted_text,
                div().mt(px(-DIAL_TOP_INSET)).child(self.wrapping_slider.clone()),
                value_label(self.wrapping_value, chrome.body_text),
                None,
            ))
    }
}

fn demo_block(
    label: &'static str,
    label_color: gpui::Hsla,
    control: impl IntoElement,
    value: impl IntoElement,
    hint: Option<AnyElement>,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(DEMO_STACK_GAP))
        .child(section_label(label, label_color))
        .when_some(hint, |this, hint| this.child(hint))
        .child(control)
        .child(value)
        .into_any_element()
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

pub(crate) fn render_slider_customization_body(preview: Entity<SliderCustomizationPreview>) -> AnyElement {
    preview.into_any_element()
}
