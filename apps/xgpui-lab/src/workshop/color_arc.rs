//! Color arc control exposition — channel arcs, renderer compare, and thickness variants.

use crate::theme::{ColorVizLookExt, Look};
use xgpui_luma::controls::slider::SliderEventExt;
use std::sync::Arc;

use gpui::{App, AppContext, Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use xgpui_luma_color::color_arc::{
    ColorArcBuilder, ColorArcDomainRenderer, ColorArcRenderer, ColorArcTrackContext, HueArcDelegate,
    LightnessArcDelegate, RasterArcDelegate, SaturationArcDelegate, refresh_color_arc, update_arc_delegate,
};
use xgpui_luma_color::color_slider::color_spec::Hsv;
use xgpui_luma_color::style::Size;
use xgpui_luma::controls::slider::{SliderControl, SliderEvent};

use super::color_exposition_common::{
    centered_field, detail_row, format_compact_hsla, format_hex_color, format_slider_event, render_demo_section,
    render_field_card,
};
use super::event_stream::ControlEventStream;

const ARC_CARD_WIDTH: f32 = 320.0;

struct ArcDemo {
    slider: Entity<SliderControl>,
    renderer: Arc<ColorArcDomainRenderer>,
    track_context: ColorArcTrackContext,
}

impl ArcDemo {
    fn spawn(builder: ColorArcBuilder, cx: &mut impl AppContext) -> Self {
        let renderer = builder.domain_renderer();
        let track_context = builder.track_context();
        let slider = builder.spawn(cx);
        Self { slider, renderer, track_context }
    }

    fn sync_saturation(&self, hue: f32, hsv_value: f32, cx: &mut App) {
        update_arc_delegate(
            &self.renderer,
            Arc::new(SaturationArcDelegate { hue, hsv_value }),
            self.track_context.clone(),
        );
        refresh_color_arc(&self.slider, cx);
    }

    fn sync_lightness(&self, hue: f32, saturation: f32, cx: &mut App) {
        update_arc_delegate(
            &self.renderer,
            Arc::new(LightnessArcDelegate { hue, saturation }),
            self.track_context.clone(),
        );
        refresh_color_arc(&self.slider, cx);
    }

    fn sync_hue_raster(&self, saturation: f32, lightness: f32, cx: &mut App) {
        update_arc_delegate(
            &self.renderer,
            Arc::new(RasterArcDelegate::hue(saturation, lightness)),
            self.track_context.clone(),
        );
        refresh_color_arc(&self.slider, cx);
    }

    fn sync_hue_vector(&self, saturation: f32, lightness: f32, cx: &mut App) {
        update_arc_delegate(
            &self.renderer,
            Arc::new(HueArcDelegate { saturation, lightness }),
            self.track_context.clone(),
        );
        refresh_color_arc(&self.slider, cx);
    }

    fn set_value(&self, value: f32, cx: &mut App) {
        self.slider.update(cx, |slider, cx| slider.set_value(value, cx));
    }
}

pub(super) struct ColorArcExpositionLeftPane {
    look: Arc<Look>,
    hue_arc: ArcDemo,
    saturation_arc: ArcDemo,
    lightness_arc: ArcDemo,
    vector_arc: ArcDemo,
    raster_arc: ArcDemo,
    thickness_small_arc: ArcDemo,
    thickness_large_arc: ArcDemo,
    hsv: Hsv,
    last_event: String,
    event_stream: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}

impl ColorArcExpositionLeftPane {
    fn handle_event(&mut self, event: &SliderEvent, cx: &mut Context<Self>) {
        let hue = match event {
            SliderEvent::Change { value, .. } => {
                self.last_event = format!("Change {:.1}", value);
                *value
            }
            SliderEvent::Release { value, .. } => {
                self.last_event = format!("Release {:.1}", value);
                *value
            }
            _ => return,
        };

        self.hsv.h = hue;
        self.sync_dependent_controls(cx);
        cx.notify();
    }

    fn sync_dependent_controls(&self, cx: &mut Context<Self>) {
        let hue = self.hsv.h;
        let saturation = self.hsv.s;
        let value = self.hsv.v;
        let lightness = self.hsv.to_hsla_ext().l;

        self.saturation_arc.sync_saturation(hue, value, cx);
        self.lightness_arc.sync_lightness(hue, saturation, cx);
        self.vector_arc.set_value(hue, cx);
        self.vector_arc.sync_hue_vector(saturation, lightness, cx);
        self.raster_arc.set_value(hue, cx);
        self.raster_arc.sync_hue_raster(saturation, lightness, cx);
        self.thickness_small_arc.set_value(hue, cx);
        self.thickness_small_arc.sync_hue_raster(saturation, lightness, cx);
        self.thickness_large_arc.set_value(hue, cx);
        self.thickness_large_arc.sync_hue_raster(saturation, lightness, cx);
    }

    pub(super) fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for arc in [
            &self.hue_arc,
            &self.saturation_arc,
            &self.lightness_arc,
            &self.vector_arc,
            &self.raster_arc,
            &self.thickness_small_arc,
            &self.thickness_large_arc,
        ] {
            arc.slider.update(cx, |_, cx| cx.notify());
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for ColorArcExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let look = &self.look;
        let selected = self.hsv.to_hsla_ext();

        let preview = div()
            .w_full()
            .flex()
            .flex_col()
            .items_start()
            .gap(px(28.0))
            .child(render_demo_section(
                look,
                "Channels",
                "Hue, saturation, and lightness delegates rendered as 270-degree arcs.",
                div()
                    .flex()
                    .flex_wrap()
                    .items_start()
                    .gap(px(16.0))
                    .child(render_field_card(
                        look,
                        "Hue Arc",
                        "Primary interactive hue arc.",
                        ARC_CARD_WIDTH,
                        centered_field(self.hue_arc.slider.clone()),
                    ))
                    .child(render_field_card(
                        look,
                        "Saturation Arc",
                        "Mirrored saturation arc delegate.",
                        ARC_CARD_WIDTH,
                        centered_field(self.saturation_arc.slider.clone()),
                    ))
                    .child(render_field_card(
                        look,
                        "Lightness Arc",
                        "Mirrored lightness arc delegate.",
                        ARC_CARD_WIDTH,
                        centered_field(self.lightness_arc.slider.clone()),
                    ))
                    .into_any_element(),
            ))
            .child(render_demo_section(
                look,
                "Renderer Compare",
                "Vector and raster hue arcs rendered with the same geometry.",
                div()
                    .flex()
                    .flex_wrap()
                    .items_start()
                    .gap(px(16.0))
                    .child(render_field_card(
                        look,
                        "Vector",
                        "Segmented vector arc.",
                        ARC_CARD_WIDTH,
                        centered_field(self.vector_arc.slider.clone()),
                    ))
                    .child(render_field_card(
                        look,
                        "Raster",
                        "Raster-backed arc.",
                        ARC_CARD_WIDTH,
                        centered_field(self.raster_arc.slider.clone()),
                    ))
                    .child(render_field_card(
                        look,
                        "Live Readout",
                        "Current value from the primary hue arc.",
                        ARC_CARD_WIDTH,
                        div()
                            .w_full()
                            .flex()
                            .flex_col()
                            .gap(px(10.0))
                            .child(
                                div()
                                    .h(px(44.0))
                                    .rounded(px(look.metrics().radius.lg))
                                    .border_1()
                                    .border_color(look.chrome().border)
                                    .bg(selected),
                            )
                            .child(detail_row(look, "Hex", format_hex_color(selected)))
                            .child(detail_row(look, "HSLA", format_compact_hsla(selected)))
                            .child(detail_row(look, "Hue", format!("{:.1} deg", self.hsv.h)))
                            .child(detail_row(look, "Last Event", self.last_event.clone())),
                    ))
                    .into_any_element(),
            ))
            .child(render_demo_section(
                look,
                "Thickness",
                "Thinner and thicker variants of the same hue arc.",
                render_field_card(
                    look,
                    "Thickness Variants",
                    "Small and large arc track thickness on a half arc.",
                    ARC_CARD_WIDTH,
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(14.0))
                        .child(self.thickness_small_arc.slider.clone())
                        .child(self.thickness_large_arc.slider.clone()),
                ),
            ))
            .child(self.event_stream.clone());

        div()
            .id("controls-doc-color-arc-left-pane")
            .size_full()
            .min_h(px(0.0))
            .min_w(px(0.0))
            .overflow_y_scroll()
            .p(px(16.0))
            .child(preview)
    }
}

impl ColorArcExpositionLeftPane {
    pub(super) fn new(cx: &mut Context<Self>, look: Arc<Look>) -> Self {
        let hsv = Hsv { h: 28.0, s: 0.74, v: 0.92, a: 1.0 };
        let lightness = hsv.to_hsla_ext().l;

        let hue_arc = ArcDemo::spawn(
            ColorArcBuilder::hue("controls-doc-color-arc-hue", hsv.h, hsv.s, lightness)
                .size(Size::Medium)
                .start_degrees(-45.0)
                .sweep_degrees(270.0),
            cx,
        );
        let saturation_arc = ArcDemo::spawn(
            ColorArcBuilder::saturation("controls-doc-color-arc-saturation", hsv.s, hsv.h, hsv.v)
                .size(Size::Medium)
                .start_degrees(-45.0)
                .sweep_degrees(270.0),
            cx,
        );
        let lightness_arc = ArcDemo::spawn(
            ColorArcBuilder::lightness("controls-doc-color-arc-lightness", lightness, hsv.h, hsv.s)
                .size(Size::Medium)
                .start_degrees(-45.0)
                .sweep_degrees(270.0),
            cx,
        );
        let vector_arc = ArcDemo::spawn(
            ColorArcBuilder::hue_with_renderer(
                "controls-doc-color-arc-vector",
                hsv.h,
                hsv.s,
                lightness,
                ColorArcRenderer::Vector,
            )
            .size(Size::Medium)
            .start_degrees(0.0)
            .sweep_degrees(180.0),
            cx,
        );
        let raster_arc = ArcDemo::spawn(
            ColorArcBuilder::hue_with_renderer(
                "controls-doc-color-arc-raster",
                hsv.h,
                hsv.s,
                lightness,
                ColorArcRenderer::Raster,
            )
            .size(Size::Medium)
            .start_degrees(0.0)
            .sweep_degrees(180.0),
            cx,
        );
        let thickness_small_arc = ArcDemo::spawn(
            ColorArcBuilder::hue("controls-doc-color-arc-thickness-small", hsv.h, hsv.s, lightness)
                .size(Size::Medium)
                .arc_thickness_size(Size::Small)
                .start_degrees(0.0)
                .sweep_degrees(180.0),
            cx,
        );
        let thickness_large_arc = ArcDemo::spawn(
            ColorArcBuilder::hue("controls-doc-color-arc-thickness-large", hsv.h, hsv.s, lightness)
                .size(Size::Medium)
                .arc_thickness_size(Size::Large)
                .start_degrees(0.0)
                .sweep_degrees(180.0),
            cx,
        );

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-color-arc-event-log",
                "Drag the hue arc; SliderEvent variants appear below.",
            )
        });

        let subscriptions = {
            hue_arc.slider.clone().on_event(cx, {
                let event_stream = event_stream.downgrade();
                move |this: &mut ColorArcExpositionLeftPane, event: &SliderEvent, cx| {
                    let Some(event_stream) = event_stream.upgrade() else {
                        return;
                    };
                    this.handle_event(event, cx);
                    if let Some(line) = format_slider_event("Hue Arc", event) {
                        event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                    }
                }
            });
            Vec::new()
        };

        Self {
            look: look.clone(),
            hue_arc,
            saturation_arc,
            lightness_arc,
            vector_arc,
            raster_arc,
            thickness_small_arc,
            thickness_large_arc,
            hsv,
            last_event: "Release".to_string(),
            event_stream,
            _subscriptions: subscriptions,
        }
    }
}
