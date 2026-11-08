//! Color ring control exposition — linked HSL rings and renderer comparison.

use xgpui_luma::controls::slider::SliderEventExt;
use std::sync::Arc;

use gpui::{App, AppContext, Context, Entity, Hsla, Render, Window, div, hsla, prelude::*, px};
use xgpui_luma_color::color_ring::{
    ColorRingBuilder, ColorRingDomainRenderer, ColorRingRenderer, ColorRingTrackContext, HueRingDelegate,
    LightnessRingDelegate, RasterRingDelegate, SaturationRingDelegate, primary_slider_value, refresh_color_ring,
    update_ring_delegate,
};
use xgpui_luma_color::color_slider::color_spec::{Hsl, Hsv};
use xgpui_luma_color::color_slider::ColorSpecification;
use xgpui_luma_color::style::Size;
use xgpui_luma::controls::slider::{SliderControl, SliderEvent};
use xgpui_luma::{vstack, wrappanel};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::color_chrome_exposition::{
    color_chrome_set_viewport_size, spawn_color_chrome_viewport, sync_color_chrome_viewport, ColorChromeViewportPane,
};
use super::color_chrome_inspector::ColorChromeInspector;
use super::color_exposition_common::{detail_row, format_slider_event, render_demo_card, render_demo_section};
use super::event_stream::ControlEventStream;
use super::inspector::color_chrome::COLOR_RING_CHROME_PROFILES;
use super::inspector::slider_motion_report;
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::template::render_control_exposition_card;

const SWATCH_SIZE: f32 = 220.0;
const RING_FRAME_PX: f32 = 256.0;
const RING_MEDIUM_PX: f32 = 220.0;
const SLIDER_COLUMN_WIDTH: f32 = 280.0;
const READOUT_WIDTH: f32 = 160.0;
const WIDE_CARD: f32 = 1120.0;

struct RingDemo {
    slider: Entity<SliderControl>,
    renderer: Arc<ColorRingDomainRenderer>,
    track_context: ColorRingTrackContext,
}

impl RingDemo {
    fn spawn(builder: ColorRingBuilder, cx: &mut impl AppContext) -> Self {
        let renderer = builder.domain_renderer();
        let track_context = builder.track_context();
        let slider = builder.spawn(cx);
        Self { slider, renderer, track_context }
    }

    fn sync_saturation(&self, hue: f32, hsv_value: f32, cx: &mut App) {
        update_ring_delegate(
            &self.renderer,
            Arc::new(SaturationRingDelegate { hue, hsv_value }),
            self.track_context.clone(),
        );
        refresh_color_ring(&self.slider, cx);
    }

    fn sync_lightness(&self, hue: f32, saturation: f32, cx: &mut App) {
        update_ring_delegate(
            &self.renderer,
            Arc::new(LightnessRingDelegate { hue, saturation }),
            self.track_context.clone(),
        );
        refresh_color_ring(&self.slider, cx);
    }

    fn sync_hue_raster(&self, saturation: f32, lightness: f32, cx: &mut App) {
        update_ring_delegate(
            &self.renderer,
            Arc::new(RasterRingDelegate::hue(saturation, lightness)),
            self.track_context.clone(),
        );
        refresh_color_ring(&self.slider, cx);
    }

    fn sync_hue_vector(&self, saturation: f32, lightness: f32, cx: &mut App) {
        update_ring_delegate(
            &self.renderer,
            Arc::new(HueRingDelegate { saturation, lightness }),
            self.track_context.clone(),
        );
        refresh_color_ring(&self.slider, cx);
    }

    fn set_value(&self, value: f32, cx: &mut App) {
        self.slider.update(cx, |slider, cx| slider.set_value(value, cx));
    }
}

pub struct ColorRingControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<ColorRingExpositionLeftPane>,
    chrome_inspector: Entity<ColorChromeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

struct ColorRingExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    color_ring: RingDemo,
    color_ring_vector: RingDemo,
    color_ring_raster: RingDemo,
    ring_saturation: RingDemo,
    ring_lightness: RingDemo,
    ring_hsl: Hsl,
    ring_color: Hsla,
    event_stream: Entity<ControlEventStream>,
}

impl ColorRingExpositionLeftPane {
    fn handle_hue_event(&mut self, event: &SliderEvent, cx: &mut Context<Self>) {
        if let Some(value) = primary_slider_value(event) {
            self.ring_hsl.h = value;
            self.sync_ring(cx, SyncSource::HueRing);
        }
    }

    fn handle_saturation_event(&mut self, event: &SliderEvent, cx: &mut Context<Self>) {
        if let Some(value) = primary_slider_value(event) {
            self.ring_hsl.s = value;
            self.sync_ring(cx, SyncSource::SaturationRing);
        }
    }

    fn handle_lightness_event(&mut self, event: &SliderEvent, cx: &mut Context<Self>) {
        if let Some(value) = primary_slider_value(event) {
            self.ring_hsl.l = value;
            self.sync_ring(cx, SyncSource::LightnessRing);
        }
    }

    fn sync_ring(&mut self, cx: &mut Context<Self>, source: SyncSource) {
        let hsl = self.ring_hsl;
        let hsv_value = hsv_value_for_saturation_ring(hsl);
        self.ring_color = hsl.to_hsla();
        let lightness = self.ring_color.l;

        if !matches!(source, SyncSource::HueRing) {
            self.color_ring.set_value(hsl.h, cx);
        }
        for ring in [&self.color_ring_vector, &self.color_ring_raster] {
            ring.set_value(hsl.h, cx);
        }
        for ring in [&self.color_ring, &self.color_ring_vector] {
            ring.sync_hue_vector(hsl.s, lightness, cx);
        }
        self.color_ring_raster.sync_hue_raster(hsl.s, lightness, cx);

        if !matches!(source, SyncSource::SaturationRing) {
            self.ring_saturation.set_value(hsl.s, cx);
        }
        self.ring_saturation.sync_saturation(hsl.h, hsv_value, cx);

        if !matches!(source, SyncSource::LightnessRing) {
            self.ring_lightness.set_value(hsl.l, cx);
        }
        self.ring_lightness.sync_lightness(hsl.h, hsl.s, cx);

        cx.notify();
    }

    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for ring in [
            &self.color_ring,
            &self.color_ring_vector,
            &self.color_ring_raster,
            &self.ring_saturation,
            &self.ring_lightness,
        ] {
            ring.slider.update(cx, |_, cx| cx.notify());
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for ColorRingExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let look = &self.look;
            let chrome = look.chrome();

            let interactive = wrappanel! {
                gap=24 align=center;
                div()
                    .size(px(SWATCH_SIZE))
                    .flex_shrink_0()
                    .rounded(px(8.0))
                    .bg(self.ring_color)
                    .border_1()
                    .border_color(chrome.border),
                div()
                    .size(px(RING_FRAME_PX))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(self.color_ring.slider.clone()),
                div()
                    .w(px(SLIDER_COLUMN_WIDTH))
                    .flex_shrink_0()
                    .child(vstack! {
                        gap=16;
                        div()
                            .size(px(RING_MEDIUM_PX))
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(self.ring_saturation.slider.clone()),
                        div()
                            .size(px(RING_MEDIUM_PX))
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(self.ring_lightness.slider.clone()),
                    }),
                div()
                    .w(px(READOUT_WIDTH))
                    .flex_shrink_0()
                    .child(vstack! {
                        gap=8;
                        detail_row(look, "H", format!("{:.1}°", self.ring_color.h * 360.0)),
                        detail_row(look, "S", format!("{:.3}", self.ring_color.s)),
                        detail_row(look, "L", format!("{:.3}", self.ring_color.l)),
                        detail_row(look, "A", format!("{:.3}", self.ring_color.a)),
                    }),
            };

            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(28.0))
                .child(render_demo_section(
                    look,
                    "Interactive HSL Mixer",
                    "Primary hue, saturation, and lightness rings linked to a shared swatch.",
                    render_demo_card(look, WIDE_CARD, interactive),
                ))
                .child(render_demo_section(
                    look,
                    "Renderer Compare",
                    "Vector paths vs raster pre-image for the same hue ring.",
                    render_demo_card(
                        look,
                        WIDE_CARD,
                        wrappanel! {
                            gap=24 align=center;
                            render_ring_variant(look, "Vector", self.color_ring_vector.slider.clone(), RING_MEDIUM_PX),
                            render_ring_variant(look, "Raster", self.color_ring_raster.slider.clone(), RING_MEDIUM_PX),
                        },
                    ),
                ))
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-color-ring-left-pane")
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

impl ColorRingControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("color-ring").expect("color-ring catalog entry");
        let hsla_color = hsla(0.0, 1.0, 0.5, 1.0);
        let hsl = Hsl::from_hsla(hsla_color);

        let color_ring = RingDemo::spawn(
            ColorRingBuilder::hue("controls-doc-color-ring", hsl.h, hsl.s, hsl.l)
                .size(Size::Medium)
                .allow_inner_target(true),
            cx,
        );
        let color_ring_vector = RingDemo::spawn(
            ColorRingBuilder::hue_with_renderer(
                "controls-doc-color-ring-vector",
                hsl.h,
                hsl.s,
                hsl.l,
                ColorRingRenderer::Vector,
            )
            .size(Size::Medium),
            cx,
        );
        let color_ring_raster = RingDemo::spawn(
            ColorRingBuilder::hue_with_renderer(
                "controls-doc-color-ring-raster",
                hsl.h,
                hsl.s,
                hsl.l,
                ColorRingRenderer::Raster,
            )
            .size(Size::Medium),
            cx,
        );
        let ring_saturation = RingDemo::spawn(
            ColorRingBuilder::saturation(
                "controls-doc-color-ring-saturation",
                hsl.s,
                hsl.h,
                hsv_value_for_saturation_ring(hsl),
            )
            .size(Size::Medium)
            .allow_inner_target(true),
            cx,
        );
        let ring_lightness = RingDemo::spawn(
            ColorRingBuilder::lightness("controls-doc-color-ring-lightness", hsl.l, hsl.h, hsl.s)
                .size(Size::Medium)
                .allow_inner_target(true),
            cx,
        );

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-color-ring-event-log",
                "Drag hue, saturation, or lightness rings; SliderEvent variants appear below.",
            )
        });

        let left_pane = cx.new(|cx| {
            color_ring.slider.clone().on_event(cx, {
                let event_stream = event_stream.clone();
                move |this: &mut ColorRingExpositionLeftPane, event: &SliderEvent, cx| {
                    this.handle_hue_event(event, cx);
                    append_slider_event(&event_stream, "Hue", event, cx);
                }
            });
            ring_saturation.slider.clone().on_event(cx, {
                let event_stream = event_stream.clone();
                move |this: &mut ColorRingExpositionLeftPane, event: &SliderEvent, cx| {
                    this.handle_saturation_event(event, cx);
                    append_slider_event(&event_stream, "Saturation", event, cx);
                }
            });
            ring_lightness.slider.clone().on_event(cx, {
                let event_stream = event_stream.clone();
                move |this: &mut ColorRingExpositionLeftPane, event: &SliderEvent, cx| {
                    this.handle_lightness_event(event, cx);
                    append_slider_event(&event_stream, "Lightness", event, cx);
                }
            });

            let mut pane = ColorRingExpositionLeftPane {
                look: look.clone(),
                entry,
                color_ring,
                color_ring_vector,
                color_ring_raster,
                ring_saturation,
                ring_lightness,
                ring_hsl: hsl,
                ring_color: hsla_color,
                event_stream,
            };
            pane.sync_ring(cx, SyncSource::HueRing);
            pane
        });

        let ColorChromeViewportPane { chrome_inspector, inspector_split } = spawn_color_chrome_viewport(
            cx,
            look.clone(),
            "controls-doc-color-ring-pane",
            "controls-doc-color-ring-chrome",
            COLOR_RING_CHROME_PROFILES,
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
        );
        chrome_inspector.update(cx, |inspector, cx| {
            let pane = left_pane.read(cx);
            let samples = [
                ("Hue", pane.color_ring.slider.clone()),
                ("Saturation", pane.ring_saturation.slider.clone()),
                ("Lightness", pane.ring_lightness.slider.clone()),
                ("Vector", pane.color_ring_vector.slider.clone()),
                ("Raster", pane.color_ring_raster.slider.clone()),
            ];
            inspector.set_motion_report(slider_motion_report(samples), cx);
        });

        Self { look, entry, left_pane, chrome_inspector, inspector_split }
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
        color_chrome_set_viewport_size(&self.inspector_split, size, cx);
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        sync_color_chrome_viewport(look, &self.chrome_inspector, &self.inspector_split, cx);
        cx.notify();
    }
}

impl Render for ColorRingControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-color-ring-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

#[derive(Clone, Copy)]
#[allow(clippy::enum_variant_names)]
enum SyncSource {
    HueRing,
    SaturationRing,
    LightnessRing,
}

fn hsv_value_for_saturation_ring(hsl: Hsl) -> f32 {
    Hsv::from_hsla_ext(hsl.to_hsla()).v
}

fn append_slider_event(event_stream: &Entity<ControlEventStream>, source: &str, event: &SliderEvent, cx: &mut App) {
    if let Some(line) = format_slider_event(source, event) {
        event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
    }
}

fn render_ring_variant(
    look: &ShadcnLook,
    label: &'static str,
    ring: Entity<SliderControl>,
    frame_px: f32,
) -> impl IntoElement {
    let label_style = look.typography_scale(xgpui_luma_look_shadcn::ShadcnTextSize::Xs);

    vstack! {
        gap=8 align=center;
        div()
            .typography_style(label_style)
            .text_color(look.chrome().muted_text)
            .child(label),
        div()
            .size(px(frame_px))
            .flex()
            .items_center()
            .justify_center()
            .child(ring),
    }
}
