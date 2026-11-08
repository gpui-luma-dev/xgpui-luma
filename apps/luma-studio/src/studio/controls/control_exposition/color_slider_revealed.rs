//! Color slider revealed exposition — styling survey for the color-slider primitive.

use xgpui_luma::controls::slider::SliderEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use xgpui_luma_color::color_slider::color_spec::{Hsl, RgbaSpec};
use xgpui_luma_color::color_slider::{ColorInterpolation, ColorSliderBuilder, SliderThumbSize};
use xgpui_luma::controls::slider::{SliderControl, SliderEvent};
use xgpui_luma::theme::ControlSize;
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::color_exposition_common::{format_slider_event, render_demo_section, render_field_card, slider_grid_stack};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{ViewportInspectorPane, spawn_viewport_inspector, sync_viewport_inspector};
use super::inspector::{MotionOnlyInspector, slider_motion_report};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::template::render_control_exposition_card;
use super::theme_inspector::ThemeInspector;

const CARD_WIDTH: f32 = 360.0;

pub struct ColorSliderRevealedControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<ColorSliderRevealedPane>,
    theme_inspector: Entity<ThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

impl ColorSliderRevealedControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("color-slider-revealed").expect("color-slider-revealed catalog entry");
        let left_pane = cx.new(|cx| ColorSliderRevealedPane::new(cx, look.clone()));
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-color-slider-revealed-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &super::inspector::SLIDE_PANEL_INSPECTOR_SPEC,
            MotionOnlyInspector::shared(),
        );
        theme_inspector.update(cx, |inspector, cx| {
            let pane = left_pane.read(cx);
            let samples = [
                ("XSmall", pane.size_xsmall.clone()),
                ("Small", pane.size_small.clone()),
                ("Medium", pane.size_medium.clone()),
                ("Large", pane.size_large.clone()),
                ("Rounded", pane.rounded_full.clone()),
                ("Rounded 8", pane.rounded_8.clone()),
                ("Square", pane.rounded_square.clone()),
                ("Thumb default", pane.thumb_default.clone()),
                ("Thumb square", pane.thumb_square.clone()),
                ("Thumb bar", pane.thumb_bar.clone()),
                ("Edge default", pane.edge_default.clone()),
                ("Edge square", pane.edge_square.clone()),
                ("Edge large", pane.edge_large.clone()),
                ("RGB", pane.interp_rgb.clone()),
                ("HSL", pane.interp_hsl.clone()),
                ("Lab", pane.interp_lab.clone()),
                ("Hue", pane.delegate_hue.clone()),
                ("Alpha", pane.delegate_alpha.clone()),
                ("Red", pane.delegate_red.clone()),
                ("Saturation", pane.delegate_saturation.clone()),
            ];
            inspector.set_motion_report(slider_motion_report(samples), cx);
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

impl Render for ColorSliderRevealedControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let inspector_split = self.inspector_split.clone();
        with_look(&self.look, || {
            div()
                .id("controls-doc-color-slider-revealed-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(inspector_split)
        })
    }
}

struct ColorSliderRevealedPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    size_xsmall: Entity<SliderControl>,
    size_small: Entity<SliderControl>,
    size_medium: Entity<SliderControl>,
    size_large: Entity<SliderControl>,
    rounded_full: Entity<SliderControl>,
    rounded_8: Entity<SliderControl>,
    rounded_square: Entity<SliderControl>,
    thumb_default: Entity<SliderControl>,
    thumb_square: Entity<SliderControl>,
    thumb_bar: Entity<SliderControl>,
    edge_default: Entity<SliderControl>,
    edge_square: Entity<SliderControl>,
    edge_large: Entity<SliderControl>,
    interp_rgb: Entity<SliderControl>,
    interp_hsl: Entity<SliderControl>,
    interp_lab: Entity<SliderControl>,
    delegate_hue: Entity<SliderControl>,
    delegate_alpha: Entity<SliderControl>,
    delegate_red: Entity<SliderControl>,
    delegate_saturation: Entity<SliderControl>,
    event_stream: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}

impl ColorSliderRevealedPane {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("color-slider-revealed").expect("color-slider-revealed catalog entry");

        let size_xsmall = ColorSliderBuilder::hue("controls-doc-color-slider-revealed-size-xsmall", 180.0)
            .size(ControlSize::Sm)
            .thumb_xsmall()
            .spawn(cx);
        let size_small = ColorSliderBuilder::hue("controls-doc-color-slider-revealed-size-small", 180.0)
            .size(ControlSize::Sm)
            .spawn(cx);
        let size_medium = ColorSliderBuilder::hue("controls-doc-color-slider-revealed-size-medium", 180.0).spawn(cx);
        let size_large = ColorSliderBuilder::hue("controls-doc-color-slider-revealed-size-large", 180.0)
            .size(ControlSize::Lg)
            .spawn(cx);

        let rounded_full = ColorSliderBuilder::hue("controls-doc-color-slider-revealed-rounded-full", 180.0).spawn(cx);
        let rounded_8 = ColorSliderBuilder::hue("controls-doc-color-slider-revealed-rounded-8", 180.0)
            .rounded(px(8.0))
            .spawn(cx);
        let rounded_square = ColorSliderBuilder::hue("controls-doc-color-slider-revealed-rounded-square", 180.0)
            .rounded(px(0.0))
            .spawn(cx);

        let thumb_default =
            ColorSliderBuilder::hue("controls-doc-color-slider-revealed-thumb-default", 180.0).spawn(cx);
        let thumb_square = ColorSliderBuilder::hue("controls-doc-color-slider-revealed-thumb-square", 180.0)
            .thumb_square()
            .spawn(cx);
        let thumb_bar =
            ColorSliderBuilder::hue("controls-doc-color-slider-revealed-thumb-bar", 180.0).thumb_bar().spawn(cx);

        let edge_default = ColorSliderBuilder::hue("controls-doc-color-slider-revealed-edge-default", 180.0)
            .edge_to_edge()
            .spawn(cx);
        let edge_square = ColorSliderBuilder::hue("controls-doc-color-slider-revealed-edge-square", 180.0)
            .rounded(px(0.0))
            .thumb_square()
            .edge_to_edge()
            .spawn(cx);
        let edge_large = ColorSliderBuilder::hue("controls-doc-color-slider-revealed-edge-large", 180.0)
            .size(ControlSize::Lg)
            .edge_to_edge()
            .spawn(cx);

        let red = gpui::hsla(0.0, 1.0, 0.5, 1.0);
        let blue = gpui::hsla(240.0 / 360.0, 1.0, 0.5, 1.0);
        let interp_rgb = ColorSliderBuilder::gradient(
            "controls-doc-color-slider-revealed-interp-rgb",
            0.5,
            vec![red, blue].into_iter().map(xgpui_luma::color::gpui_bridge::from_hsla).collect(),
        )
        .expect("valid demo gradient colors")
        .interpolation(ColorInterpolation::Rgb)
        .spawn(cx);
        let interp_hsl = ColorSliderBuilder::gradient(
            "controls-doc-color-slider-revealed-interp-hsl",
            0.5,
            vec![red, blue].into_iter().map(xgpui_luma::color::gpui_bridge::from_hsla).collect(),
        )
        .expect("valid demo gradient colors")
        .interpolation(ColorInterpolation::Hsl)
        .spawn(cx);
        let interp_lab = ColorSliderBuilder::gradient(
            "controls-doc-color-slider-revealed-interp-lab",
            0.5,
            vec![red, blue].into_iter().map(xgpui_luma::color::gpui_bridge::from_hsla).collect(),
        )
        .expect("valid demo gradient colors")
        .interpolation(ColorInterpolation::Lab)
        .spawn(cx);

        let delegate_hue = ColorSliderBuilder::hue("controls-doc-color-slider-revealed-delegate-hue", 180.0)
            .thumb_size(SliderThumbSize::Md)
            .spawn(cx);
        let delegate_alpha = {
            let hsl = Hsl { h: 200.0, s: 0.8, l: 0.5, a: 1.0 };
            ColorSliderBuilder::alpha("controls-doc-color-slider-revealed-delegate-alpha", 0.5, hsl).spawn(cx)
        };
        let delegate_red = {
            let rgba = RgbaSpec { r: 255.0, g: 128.0, b: 0.0, a: 1.0 };
            ColorSliderBuilder::channel("controls-doc-color-slider-revealed-delegate-red", rgba.r, rgba, RgbaSpec::RED)
                .expect("RGBA red delegate should be valid")
                .min(0.0)
                .max(255.0)
                .spawn(cx)
        };
        let delegate_saturation = {
            let hsl = Hsl { h: 240.0, s: 0.5, l: 0.52, a: 1.0 };
            ColorSliderBuilder::channel(
                "controls-doc-color-slider-revealed-delegate-saturation",
                hsl.s,
                hsl,
                Hsl::SATURATION,
            )
            .expect("HSL saturation delegate should be valid")
            .spawn(cx)
        };

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-color-slider-revealed-event-log",
                "Edit the hue delegate slider; SliderEvent variants appear below.",
            )
        });

        let subscriptions = wire_slider_revealed_events(&delegate_hue, &event_stream, cx);

        Self {
            look,
            entry,
            size_xsmall,
            size_small,
            size_medium,
            size_large,
            rounded_full,
            rounded_8,
            rounded_square,
            thumb_default,
            thumb_square,
            thumb_bar,
            edge_default,
            edge_square,
            edge_large,
            interp_rgb,
            interp_hsl,
            interp_lab,
            delegate_hue,
            delegate_alpha,
            delegate_red,
            delegate_saturation,
            event_stream,
            _subscriptions: subscriptions,
        }
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for slider in self.all_sliders() {
            slider.update(cx, |_, cx| cx.notify());
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(self.look.clone(), cx));
        cx.notify();
    }

    fn all_sliders(&self) -> [&Entity<SliderControl>; 20] {
        [
            &self.size_xsmall,
            &self.size_small,
            &self.size_medium,
            &self.size_large,
            &self.rounded_full,
            &self.rounded_8,
            &self.rounded_square,
            &self.thumb_default,
            &self.thumb_square,
            &self.thumb_bar,
            &self.edge_default,
            &self.edge_square,
            &self.edge_large,
            &self.interp_rgb,
            &self.interp_hsl,
            &self.interp_lab,
            &self.delegate_hue,
            &self.delegate_alpha,
            &self.delegate_red,
            &self.delegate_saturation,
        ]
    }
}

impl Render for ColorSliderRevealedPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let look = &self.look;

            let preview = div()
                .w_full()
                .max_w(px(1120.0))
                .flex()
                .flex_col()
                .items_start()
                .gap(px(28.0))
                .child(render_demo_section(
                    look,
                    "Scale",
                    "Track sizes and default density mappings.",
                    render_field_card(
                        look,
                        "Sizes",
                        "XSmall through large.",
                        CARD_WIDTH,
                        slider_grid_stack(
                            look,
                            [
                                ("XSmall", self.size_xsmall.clone()),
                                ("Small", self.size_small.clone()),
                                ("Medium", self.size_medium.clone()),
                                ("Large", self.size_large.clone()),
                            ],
                        ),
                    ),
                ))
                .child(render_demo_section(
                    look,
                    "Corners and Thumb Shapes",
                    "Surface radius and thumb-shape variations.",
                    div()
                        .flex()
                        .flex_wrap()
                        .items_start()
                        .gap(px(16.0))
                        .child(render_field_card(
                            look,
                            "Corner Radius",
                            "Default rounded, softened, and square corners.",
                            CARD_WIDTH,
                            slider_grid_stack(
                                look,
                                [
                                    ("Rounded", self.rounded_full.clone()),
                                    ("Rounded 8", self.rounded_8.clone()),
                                    ("Square", self.rounded_square.clone()),
                                ],
                            ),
                        ))
                        .child(render_field_card(
                            look,
                            "Thumb Shapes",
                            "Default, square, and bar thumb treatments.",
                            CARD_WIDTH,
                            slider_grid_stack(
                                look,
                                [
                                    ("Default", self.thumb_default.clone()),
                                    ("Square", self.thumb_square.clone()),
                                    ("Bar", self.thumb_bar.clone()),
                                ],
                            ),
                        ))
                        .into_any_element(),
                ))
                .child(render_demo_section(
                    look,
                    "Edge Cases",
                    "Edge-to-edge and interpolation variants.",
                    div()
                        .flex()
                        .flex_wrap()
                        .items_start()
                        .gap(px(16.0))
                        .child(render_field_card(
                            look,
                            "Edge To Edge",
                            "Track ends aligned to the thumb centerline.",
                            CARD_WIDTH,
                            slider_grid_stack(
                                look,
                                [
                                    ("Default", self.edge_default.clone()),
                                    ("Square", self.edge_square.clone()),
                                    ("Large", self.edge_large.clone()),
                                ],
                            ),
                        ))
                        .child(render_field_card(
                            look,
                            "Interpolation",
                            "RGB, HSL, and Lab interpolation over the same two-color gradient.",
                            CARD_WIDTH,
                            slider_grid_stack(
                                look,
                                [
                                    ("RGB", self.interp_rgb.clone()),
                                    ("HSL", self.interp_hsl.clone()),
                                    ("Lab", self.interp_lab.clone()),
                                ],
                            ),
                        ))
                        .into_any_element(),
                ))
                .child(render_demo_section(
                    look,
                    "Delegate Samples",
                    "Representative delegate families from the original page.",
                    render_field_card(
                        look,
                        "Delegates",
                        "Hue, alpha, red-channel, and HSL saturation delegates.",
                        CARD_WIDTH,
                        slider_grid_stack(
                            look,
                            [
                                ("Hue", self.delegate_hue.clone()),
                                ("Alpha", self.delegate_alpha.clone()),
                                ("Red", self.delegate_red.clone()),
                                ("Saturation", self.delegate_saturation.clone()),
                            ],
                        ),
                    ),
                ))
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-color-slider-revealed-left-pane")
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

fn wire_slider_revealed_events(
    delegate_hue: &Entity<SliderControl>,
    event_stream: &Entity<ControlEventStream>,
    cx: &mut Context<ColorSliderRevealedPane>,
) -> Vec<Subscription> {
    {
        delegate_hue.clone().on_event(cx, {
            let event_stream = event_stream.downgrade();
            move |_, event: &SliderEvent, cx| {
                let Some(event_stream) = event_stream.upgrade() else {
                    return;
                };
                if let Some(line) = format_slider_event("Hue Delegate", event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }
        });
        Vec::new()
    }
}
