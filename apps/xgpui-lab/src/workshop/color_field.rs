//! Color field control exposition — rectangular, triangle, wheel, and renderer samples.

use crate::theme::{ColorVizLookExt, Look};
use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use xgpui_luma_color::color_field::{ColorFieldEvent, ColorFieldState};
use xgpui_luma_color::color_slider::color_spec::Hsv;
use xgpui_luma_color::color_slider::sizing;

use super::color_exposition_common::{
    detail_row, format_color_field_event, format_compact_hsla, format_hex_color, render_demo_section, render_field_card,
};
use super::event_stream::ControlEventStream;

const FIELD_SIZE: f32 = 220.0;
const CARD_WIDTH: f32 = 320.0;

pub(super) struct ColorFieldExpositionLeftPane {
    look: Arc<Look>,
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
    pub(super) fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
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
                                    .rounded(px(look.radii().large))
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
            .p(px(16.0))
            .child(preview)
    }
}

impl ColorFieldExpositionLeftPane {
    pub(super) fn new(cx: &mut Context<Self>, look: Arc<Look>) -> Self {
        let hsv = Hsv { h: 320.0, s: 0.72, v: 0.84, a: 1.0 };

        let rect_field = cx.new(|_| {
            ColorFieldState::saturation_value_rect("controls-doc-color-field-rect", hsv, sizing::THUMB_SIZE_MEDIUM)
                .vector()
        });
        let raster_field = cx.new(|_| {
            ColorFieldState::saturation_value_rect("controls-doc-color-field-raster", hsv, sizing::THUMB_SIZE_MEDIUM)
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
            ColorFieldState::hue_value("controls-doc-color-field-hue-value", hsv, sizing::THUMB_SIZE_MEDIUM).vector()
        });

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-color-field-event-log",
                "Drag inside a color field; ColorFieldEvent variants appear below.",
            )
        });

        let subscriptions = vec![cx.subscribe(&rect_field, {
            let event_stream = event_stream.clone();
            move |this: &mut ColorFieldExpositionLeftPane, _, event: &ColorFieldEvent, cx| {
                this.handle_event(event, cx);
                if let Some(line) = format_color_field_event(event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }
        })];

        Self {
            look: look.clone(),
            rect_field,
            raster_field,
            triangle_field,
            wheel_field,
            hue_value_field,
            hsv,
            event_stream,
            _subscriptions: subscriptions,
        }
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    #[test]
    fn workshop_field_change_updates_readout_and_survives_theme_sync() {
        let mut app = gpui::TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (workshop, cx) =
            app.add_window_view(|_, cx| crate::workshop::Workshop::new(crate::theme::default_look(), cx));
        let pane = cx.update(|_, cx| match &workshop.read(cx).samples["color-field"] {
            crate::workshop::Sample::Field(pane) => pane.clone(),
            _ => unreachable!(),
        });
        let field = cx.update(|_, cx| pane.read(cx).rect_field.clone());
        let changed = Hsv { h: 110., s: 0.35, v: 0.6, a: 1. };
        field.update(cx, |_, cx| cx.emit(ColorFieldEvent::Change(changed)));
        cx.run_until_parked();
        workshop.update(cx, |workshop, cx| {
            workshop.select("color-ring", cx);
            workshop.select("button", cx);
            workshop.sync_theme(xgpui_luma::theme::ThemeMode::Light, cx);
            workshop.select("color-field", cx);
        });
        cx.update(|_, cx| {
            let pane = pane.read(cx);
            assert_eq!(pane.hsv.h, changed.h);
            assert_eq!(pane.hsv.s, changed.s);
            assert_eq!(pane.hsv.v, changed.v);
            assert_eq!(pane.rect_field, field);
        });
    }
}
