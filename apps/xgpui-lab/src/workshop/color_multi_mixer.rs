//! Multi mixer composition exposition — per-color-space channel mixers.

use xgpui_luma::controls::slider::SliderEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::slider::SliderEvent;
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn::ShadcnLook;

mod demo;

use demo::MultiMixerDemo;
use super::color_exposition_common::{format_slider_event, render_demo_section};
use super::event_stream::ControlEventStream;

pub struct ColorMultiMixerControlExposition {
    look: Arc<ShadcnLook>,
    demo: Entity<MultiMixerDemo>,
    event_stream: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}

impl ColorMultiMixerControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let demo = cx.new(|cx| MultiMixerDemo::new(look.clone(), cx));

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-color-multi-mixer-event-log",
                "Edit the HSVA hue slider; SliderEvent variants appear below.",
            )
        });

        let subscriptions = wire_multi_mixer_events(&demo, &event_stream, cx);

        Self { look, demo, event_stream, _subscriptions: subscriptions }
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.demo.update(cx, |demo, cx| demo.sync_look(look.clone(), cx));
        self.event_stream.update(cx, |stream, cx| stream.sync_look(self.look.clone(), cx));
        cx.notify();
    }
}

impl Render for ColorMultiMixerControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let look = &self.look;

            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(28.0))
                .child(render_demo_section(
                    look,
                    "Multi Mixer",
                    "Representative per-color-space channel mixers from the gallery inventory.",
                    div().child(self.demo.clone()).into_any_element(),
                ))
                .child(self.event_stream.clone());

            div().id("workshop-multi-mixer-scroll").size_full().overflow_y_scroll().p(px(16.0)).child(preview)
        })
    }
}

fn wire_multi_mixer_events(
    demo: &Entity<MultiMixerDemo>,
    event_stream: &Entity<ControlEventStream>,
    cx: &mut Context<ColorMultiMixerControlExposition>,
) -> Vec<Subscription> {
    let hue_slider = demo.read(cx).hsva_hue_slider(cx);

    {
        hue_slider.clone().on_event(cx, {
            let event_stream = event_stream.downgrade();
            move |_, event: &SliderEvent, cx| {
                let Some(event_stream) = event_stream.upgrade() else {
                    return;
                };
                if let Some(line) = format_slider_event("HSVA Hue", event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }
        });
        Vec::new()
    }
}
