//! Color field control exposition — rectangular, triangle, wheel, and renderer samples.

use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use xgpui_luma_color::color_field::{ColorFieldEvent, ColorFieldState};
use xgpui_luma_color::color_slider::color_spec::Hsv;
use xgpui_luma_color::color_slider::sizing;
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::color_chrome_exposition::{
    color_chrome_set_viewport_size, spawn_color_chrome_viewport, sync_color_chrome_viewport, ColorChromeViewportPane,
};
use super::color_exposition_common::{
    detail_row, format_color_field_event, format_compact_hsla, format_hex_color, render_demo_section, render_field_card,
};
use super::event_stream::ControlEventStream;
use super::inspector::color_chrome::COLOR_FIELD_CHROME_PROFILES;
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::template::render_control_exposition_card;
use super::color_chrome_inspector::ColorChromeInspector;

const FIELD_SIZE: f32 = 220.0;
const CARD_WIDTH: f32 = 320.0;

pub struct ColorFieldControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<ColorFieldExpositionLeftPane>,
    chrome_inspector: Entity<ColorChromeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

struct ColorFieldExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    rect_field: Entity<ColorFieldState>,
    raster_field: Entity<ColorFieldState>,
    triangle_field: Entity<ColorFieldState>,
    wheel_field: Entity<ColorFieldState>,
    hue_value_field: Entity<ColorFieldState>,
    hsv: Hsv,
    event_stream: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}

impl ColorFieldExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for field in
            [&self.rect_field, &self.raster_field, &self.triangle_field, &self.wheel_field, &self.hue_value_field]
        {
            field.update(cx, |_, cx| cx.notify());
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }

    fn handle_event(&mut self, event: &ColorFieldEvent, cx: &mut Context<Self>) {
        self.hsv = match event {
            ColorFieldEvent::Change(hsv) | ColorFieldEvent::Release(hsv) => *hsv,
            _ => return,
        };
        cx.notify();
    }
}

impl Render for ColorFieldExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
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
                    "Planes",
                    "Representative field models from the gallery inventory.",
                    div()
                        .flex()
                        .flex_wrap()
                        .items_start()
                        .gap(px(16.0))
                        .child(render_field_card(
                            look,
                            "Saturation / Value",
                            "Primary rectangular HSV field.",
                            CARD_WIDTH,
                            div().size(px(FIELD_SIZE)).child(self.rect_field.clone()),
                        ))
                        .child(render_field_card(
                            look,
                            "Hue / Value",
                            "Alternate rectangular domain at fixed saturation.",
                            CARD_WIDTH,
                            div().size(px(FIELD_SIZE)).child(self.hue_value_field.clone()),
                        ))
                        .child(render_field_card(
                            look,
                            "Triangle Domain",
                            "Triangle selection surface using the same color model.",
                            CARD_WIDTH,
                            div().size(px(FIELD_SIZE)).child(self.triangle_field.clone()),
                        ))
                        .child(render_field_card(
                            look,
                            "Hue / Saturation Wheel",
                            "Circular field variant.",
                            CARD_WIDTH,
                            div().size(px(FIELD_SIZE)).child(self.wheel_field.clone()),
                        ))
                        .into_any_element(),
                ))
                .child(render_demo_section(
                    look,
                    "Renderer Compare",
                    "Vector field next to a prewarmed raster image field.",
                    div()
                        .flex()
                        .flex_wrap()
                        .items_start()
                        .gap(px(16.0))
                        .child(render_field_card(
                            look,
                            "Raster Image",
                            "Raster-backed field using the same HSV model.",
                            CARD_WIDTH,
                            div().size(px(FIELD_SIZE)).child(self.raster_field.clone()),
                        ))
                        .child(render_field_card(
                            look,
                            "Selection Readout",
                            "Readout from the primary saturation/value field.",
                            CARD_WIDTH,
                            div()
                                .w_full()
                                .flex()
                                .flex_col()
                                .gap(px(10.0))
                                .child(
                                    div()
                                        .h(px(44.0))
                                        .rounded(px(12.0))
                                        .border_1()
                                        .border_color(look.chrome().border)
                                        .bg(selected),
                                )
                                .child(detail_row(look, "Hex", format_hex_color(selected)))
                                .child(detail_row(look, "HSLA", format_compact_hsla(selected)))
                                .child(detail_row(look, "Hue", format!("{:.1} deg", self.hsv.h)))
                                .child(detail_row(look, "Saturation", format!("{:.3}", self.hsv.s)))
                                .child(detail_row(look, "Value", format!("{:.3}", self.hsv.v))),
                        ))
                        .into_any_element(),
                ))
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-color-field-left-pane")
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

impl ColorFieldControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("color-field").expect("color-field catalog entry");
        let hsv = Hsv { h: 320.0, s: 0.72, v: 0.84, a: 1.0 };

        let rect_field = cx.new(|_| {
            ColorFieldState::saturation_value_rect("controls-doc-color-field-rect", hsv, sizing::THUMB_SIZE_MEDIUM)
                .vector()
                .rounded(px(12.0))
        });
        let raster_field = cx.new(|_| {
            ColorFieldState::saturation_value_rect("controls-doc-color-field-raster", hsv, sizing::THUMB_SIZE_MEDIUM)
                .rounded(px(12.0))
                .raster_image_prewarmed_square(FIELD_SIZE)
        });
        let triangle_field = cx.new(|_| {
            ColorFieldState::saturation_value_triangle(
                "controls-doc-color-field-triangle",
                hsv,
                sizing::THUMB_SIZE_MEDIUM,
            )
            .vector()
        });
        let wheel_field = cx.new(|_| {
            ColorFieldState::hue_saturation_wheel("controls-doc-color-field-wheel", hsv, sizing::THUMB_SIZE_MEDIUM)
                .raster_image_prewarmed_square(FIELD_SIZE)
                .inside_field()
        });
        let hue_value_field = cx.new(|_| {
            ColorFieldState::hue_value("controls-doc-color-field-hue-value", hsv, sizing::THUMB_SIZE_MEDIUM)
                .rounded(px(12.0))
                .vector()
        });

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-color-field-event-log",
                "Drag inside a color field; ColorFieldEvent variants appear below.",
            )
        });

        let left_pane = cx.new(|cx| {
            let subscriptions = vec![cx.subscribe(&rect_field, {
                let event_stream = event_stream.clone();
                move |this: &mut ColorFieldExpositionLeftPane, _, event: &ColorFieldEvent, cx| {
                    this.handle_event(event, cx);
                    if let Some(line) = format_color_field_event(event) {
                        event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                    }
                }
            })];

            ColorFieldExpositionLeftPane {
                look: look.clone(),
                entry,
                rect_field,
                raster_field,
                triangle_field,
                wheel_field,
                hue_value_field,
                hsv,
                event_stream,
                _subscriptions: subscriptions,
            }
        });

        let ColorChromeViewportPane { chrome_inspector, inspector_split } = spawn_color_chrome_viewport(
            cx,
            look.clone(),
            "controls-doc-color-field-pane",
            "controls-doc-color-field-chrome",
            COLOR_FIELD_CHROME_PROFILES,
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
        );

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

impl Render for ColorFieldControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-color-field-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}
