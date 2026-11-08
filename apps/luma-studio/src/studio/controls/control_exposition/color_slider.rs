//! Color slider control exposition — gallery-aligned hue/saturation/alpha, gradients, and channels.

use xgpui_luma::controls::slider::SliderEventExt;
use xgpui_luma::color::gpui_bridge::from_hsla;
use std::sync::Arc;

use gpui::{Context, Entity, FontWeight, Hsla, Render, Subscription, Window, div, prelude::*, px};
use xgpui_luma_color::ColorSwatch;
use xgpui_luma_color::color_slider::color_spec::{Hsl, RgbaSpec};
use xgpui_luma_color::color_slider::{
    AlphaDelegate, ChannelDelegate, ColorInterpolation, ColorSliderBuilder, ColorSliderDomainRenderer,
    ColorSpecification, refresh_color_slider, update_domain_delegate,
};
use xgpui_luma::controls::slider::{SliderControl, SliderEvent};
use xgpui_luma::theme::ControlSize;
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextRole, ShadcnTextSize};

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::color_chrome_exposition::{
    color_chrome_set_viewport_size, spawn_color_chrome_viewport, sync_color_chrome_viewport, ColorChromeViewportPane,
};
use super::color_chrome_inspector::ColorChromeInspector;
use super::color_exposition_common::slider_labeled_row;
use super::event_stream::ControlEventStream;
use super::inspector::color_chrome::COLOR_SLIDER_CHROME_PROFILES;
use super::inspector::slider_motion_report;
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::template::render_control_exposition_card;

const WIDE_CARD: f32 = 420.0;
const NARROW_CARD: f32 = 320.0;

pub struct ColorSliderControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<ColorSliderExpositionLeftPane>,
    chrome_inspector: Entity<ColorChromeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

struct ColorSliderExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    hue_slider: Entity<SliderControl>,
    saturation_slider: Entity<SliderControl>,
    alpha_slider: Entity<SliderControl>,
    saturation_domain: Arc<ColorSliderDomainRenderer>,
    alpha_domain: Arc<ColorSliderDomainRenderer>,
    gradient_rgb: Entity<SliderControl>,
    gradient_hsl: Entity<SliderControl>,
    gradient_lab: Entity<SliderControl>,
    red_slider: Entity<SliderControl>,
    green_slider: Entity<SliderControl>,
    blue_slider: Entity<SliderControl>,
    hsl: Hsl,
    event_stream: Entity<ControlEventStream>,
    _subscriptions: Vec<Subscription>,
}

impl ColorSliderExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for slider in [
            &self.hue_slider,
            &self.saturation_slider,
            &self.alpha_slider,
            &self.gradient_rgb,
            &self.gradient_hsl,
            &self.gradient_lab,
            &self.red_slider,
            &self.green_slider,
            &self.blue_slider,
        ] {
            slider.update(cx, |_, cx| cx.notify());
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }

    fn sync_delegates(&self, cx: &mut Context<Self>) {
        let hsl = self.hsl;
        update_domain_delegate(
            &self.saturation_domain,
            Arc::new(
                ChannelDelegate::new(hsl, Hsl::SATURATION.into()).expect("HSL saturation delegate should be valid"),
            ),
            self.saturation_domain.context(),
        );
        refresh_color_slider(&self.saturation_slider, cx);
        update_domain_delegate(&self.alpha_domain, Arc::new(AlphaDelegate { spec: hsl }), self.alpha_domain.context());
        refresh_color_slider(&self.alpha_slider, cx);
    }
}

impl Render for ColorSliderExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let selected = self.hsl.to_hsla();

            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(28.0))
                .child(render_demo_section(
                    &self.look,
                    "Core Delegates",
                    "Primary interactive color sliders backed by hue, saturation, and alpha delegates.",
                    render_demo_card(
                        &self.look,
                        WIDE_CARD,
                        div()
                            .w_full()
                            .flex()
                            .flex_col()
                            .gap(px(12.0))
                            .child(slider_labeled_row(&self.look, "Hue", self.hue_slider.clone()))
                            .child(slider_labeled_row(&self.look, "Saturation", self.saturation_slider.clone()))
                            .child(slider_labeled_row(&self.look, "Alpha", self.alpha_slider.clone()))
                            .child(
                                ColorSwatch::new(from_hsla(selected))
                                    .checkerboard(true)
                                    .height(px(44.0))
                                    .rounded(px(12.0))
                                    .into_any_element(),
                            )
                            .child(detail_row(&self.look, "Hex", format_hex_color(selected)))
                            .child(detail_row(&self.look, "HSLA", format_compact_hsla(selected))),
                    ),
                ))
                .child(render_demo_section(
                    &self.look,
                    "Interpolation",
                    "Gradient interpolation paths from the upstream slider demos.",
                    render_demo_card(
                        &self.look,
                        NARROW_CARD,
                        div()
                            .w_full()
                            .flex()
                            .flex_col()
                            .gap(px(12.0))
                            .child(slider_labeled_row(&self.look, "RGB", self.gradient_rgb.clone()))
                            .child(slider_labeled_row(&self.look, "HSL", self.gradient_hsl.clone()))
                            .child(slider_labeled_row(&self.look, "Lab", self.gradient_lab.clone())),
                    ),
                ))
                .child(render_demo_section(
                    &self.look,
                    "Channels",
                    "Representative RGB channel delegates.",
                    render_demo_card(
                        &self.look,
                        NARROW_CARD,
                        div()
                            .w_full()
                            .flex()
                            .flex_col()
                            .gap(px(12.0))
                            .child(slider_labeled_row(&self.look, "Red", self.red_slider.clone()))
                            .child(slider_labeled_row(&self.look, "Green", self.green_slider.clone()))
                            .child(slider_labeled_row(&self.look, "Blue", self.blue_slider.clone())),
                    ),
                ))
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-color-slider-left-pane")
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

impl ColorSliderControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("color-slider").expect("color-slider catalog entry");
        let hsl = Hsl { h: 210.0, s: 0.72, l: 0.52, a: 0.85 };

        let hue_slider = ColorSliderBuilder::hue("controls-doc-color-slider-hue", hsl.h)
            .size(ControlSize::Sm)
            .thumb_medium()
            .spawn(cx);
        let saturation_builder =
            ColorSliderBuilder::channel("controls-doc-color-slider-saturation", hsl.s, hsl, Hsl::SATURATION)
                .expect("HSL saturation delegate should be valid")
                .size(ControlSize::Sm)
                .thumb_medium()
                .edge_to_edge();
        let saturation_domain = saturation_builder.domain_renderer();
        let saturation_slider = saturation_builder.spawn(cx);
        let alpha_builder = ColorSliderBuilder::alpha("controls-doc-color-slider-alpha", hsl.a, hsl)
            .size(ControlSize::Sm)
            .thumb_medium()
            .edge_to_edge();
        let alpha_domain = alpha_builder.domain_renderer();
        let alpha_slider = alpha_builder.spawn(cx);

        let red = gpui::hsla(0.0, 1.0, 0.5, 1.0);
        let blue = gpui::hsla(240.0 / 360.0, 1.0, 0.5, 1.0);
        let gradient_rgb = ColorSliderBuilder::gradient(
            "controls-doc-color-slider-gradient-rgb",
            0.5,
            vec![red, blue].into_iter().map(xgpui_luma::color::gpui_bridge::from_hsla).collect(),
        )
        .expect("valid demo gradient colors")
        .interpolation(ColorInterpolation::Rgb)
        .spawn(cx);
        let gradient_hsl = ColorSliderBuilder::gradient(
            "controls-doc-color-slider-gradient-hsl",
            0.5,
            vec![red, blue].into_iter().map(xgpui_luma::color::gpui_bridge::from_hsla).collect(),
        )
        .expect("valid demo gradient colors")
        .interpolation(ColorInterpolation::Hsl)
        .spawn(cx);
        let gradient_lab = ColorSliderBuilder::gradient(
            "controls-doc-color-slider-gradient-lab",
            0.5,
            vec![red, blue].into_iter().map(xgpui_luma::color::gpui_bridge::from_hsla).collect(),
        )
        .expect("valid demo gradient colors")
        .interpolation(ColorInterpolation::Lab)
        .spawn(cx);

        let rgba = RgbaSpec { r: 255.0, g: 128.0, b: 0.0, a: 1.0 };
        let red_slider = ColorSliderBuilder::channel("controls-doc-color-slider-red", rgba.r, rgba, RgbaSpec::RED)
            .expect("RGBA red delegate should be valid")
            .min(0.0)
            .max(255.0)
            .spawn(cx);
        let green_slider =
            ColorSliderBuilder::channel("controls-doc-color-slider-green", rgba.g, rgba, RgbaSpec::GREEN)
                .expect("RGBA green delegate should be valid")
                .min(0.0)
                .max(255.0)
                .spawn(cx);
        let blue_slider = ColorSliderBuilder::channel("controls-doc-color-slider-blue", rgba.b, rgba, RgbaSpec::BLUE)
            .expect("RGBA blue delegate should be valid")
            .min(0.0)
            .max(255.0)
            .spawn(cx);

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-color-slider-event-log",
                "Drag the hue, saturation, and alpha sliders; SliderEvent variants appear in the stream below.",
            )
        });

        let left_pane = cx.new(|cx| {
            let mut pane_subscriptions = Vec::new();
            for (slider, label) in [(&hue_slider, "Hue"), (&saturation_slider, "Saturation"), (&alpha_slider, "Alpha")]
            {
                pane_subscriptions.extend(subscribe_slider(slider, label, event_stream.clone(), cx));
            }
            hue_slider.clone().on_event(cx, |this, event, cx| {
                if let Some(value) = slider_release_or_change_value(event) {
                    this.hsl.h = value;
                    this.sync_delegates(cx);
                    cx.notify();
                }
            });
            saturation_slider.clone().on_event(cx, |this, event, cx| {
                if let Some(value) = slider_release_or_change_value(event) {
                    this.hsl.s = value;
                    this.sync_delegates(cx);
                    cx.notify();
                }
            });
            alpha_slider.clone().on_event(cx, |this, event, cx| {
                if let Some(value) = slider_release_or_change_value(event) {
                    this.hsl.a = value;
                    this.sync_delegates(cx);
                    cx.notify();
                }
            });

            ColorSliderExpositionLeftPane {
                look: look.clone(),
                entry,
                hue_slider,
                saturation_slider,
                alpha_slider,
                saturation_domain,
                alpha_domain,
                gradient_rgb,
                gradient_hsl,
                gradient_lab,
                red_slider,
                green_slider,
                blue_slider,
                hsl,
                event_stream,
                _subscriptions: pane_subscriptions,
            }
        });

        let ColorChromeViewportPane { chrome_inspector, inspector_split } = spawn_color_chrome_viewport(
            cx,
            look.clone(),
            "controls-doc-color-slider-pane",
            "controls-doc-color-slider-chrome",
            COLOR_SLIDER_CHROME_PROFILES,
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
        );
        chrome_inspector.update(cx, |inspector, cx| {
            let pane = left_pane.read(cx);
            let samples = [
                ("Hue", pane.hue_slider.clone()),
                ("Saturation", pane.saturation_slider.clone()),
                ("Alpha", pane.alpha_slider.clone()),
                ("RGB", pane.gradient_rgb.clone()),
                ("HSL", pane.gradient_hsl.clone()),
                ("Lab", pane.gradient_lab.clone()),
                ("Red", pane.red_slider.clone()),
                ("Green", pane.green_slider.clone()),
                ("Blue", pane.blue_slider.clone()),
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

impl Render for ColorSliderControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-color-slider-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn subscribe_slider(
    slider: &Entity<SliderControl>,
    label: &'static str,
    event_stream: Entity<ControlEventStream>,
    cx: &mut Context<ColorSliderExpositionLeftPane>,
) -> Vec<Subscription> {
    let label = label.to_string();
    {
        slider.clone().on_event(cx, move |_, event: &SliderEvent, cx| {
            let Some(line) = format_slider_event(&label, event) else {
                return;
            };
            event_stream.update(cx, |stream, cx| {
                stream.append_line(&line, cx);
                cx.notify();
            });
        });
        Vec::new()
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
        SliderEvent::Change { value, .. } => Some(format!("SliderEvent::Change - {source} ({value:.3})")),
        SliderEvent::Release { value, .. } => Some(format!("SliderEvent::Release - {source} ({value:.3})")),
        SliderEvent::DragStart { .. } => Some(format!("SliderEvent::DragStart - {source}")),
        SliderEvent::DragEnd { value, .. } => Some(format!("SliderEvent::DragEnd - {source} ({value:.3})")),
        SliderEvent::FocusChanged { focused } => Some(format!("SliderEvent::FocusChanged {{ focused: {focused} }}")),
        SliderEvent::HoverChanged { hovered } => Some(format!("SliderEvent::HoverChanged {{ hovered: {hovered} }}")),
        SliderEvent::EnabledChanged { enabled } => {
            Some(format!("SliderEvent::EnabledChanged {{ enabled: {enabled} }}"))
        }
        _ => None,
    }
}

fn render_demo_section(
    look: &ShadcnLook,
    title: &'static str,
    description: &'static str,
    content: gpui::AnyElement,
) -> gpui::AnyElement {
    let chrome = look.chrome();
    let title_style = look.typography_role(ShadcnTextRole::H4);
    let description_style = look.typography_scale(ShadcnTextSize::Sm);

    div()
        .flex()
        .flex_col()
        .gap(px(12.0))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .typography_style(title_style)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(chrome.title_text)
                        .child(title),
                )
                .child(div().typography_style(description_style).text_color(chrome.muted_text).child(description)),
        )
        .child(content)
        .into_any_element()
}

fn render_demo_card(look: &ShadcnLook, width_px: f32, content: impl IntoElement) -> gpui::AnyElement {
    let chrome = look.chrome();

    div()
        .w(px(width_px))
        .max_w_full()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .rounded(px(16.0))
        .border_1()
        .border_color(chrome.border)
        .bg(chrome.panel_background)
        .p(px(18.0))
        .child(content)
        .into_any_element()
}

fn detail_row(look: &ShadcnLook, label: &'static str, value: String) -> gpui::AnyElement {
    let chrome = look.chrome();
    let label_style = look.typography_scale(ShadcnTextSize::Xs);

    div()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.0))
        .child(
            div()
                .typography_style(label_style)
                .font_weight(FontWeight::MEDIUM)
                .text_color(chrome.muted_text)
                .child(label),
        )
        .child(div().typography_style(label_style).text_color(chrome.body_text).child(value))
        .into_any_element()
}

fn format_compact_hsla(color: Hsla) -> String {
    format!(
        "hsla({} {}% {}% / {})",
        rounded_channel(color.h * 360.0),
        rounded_channel(color.s * 100.0),
        rounded_channel(color.l * 100.0),
        compact_alpha(color.a)
    )
}

fn format_hex_color(color: Hsla) -> String {
    let (r, g, b) = hsla_to_rgb8(color);
    format!("#{r:02x}{g:02x}{b:02x}")
}

fn rounded_channel(value: f32) -> i32 {
    value.round() as i32
}

fn compact_alpha(alpha: f32) -> String {
    let rounded = (alpha * 100.0).round() / 100.0;
    if (rounded - rounded.round()).abs() < f32::EPSILON {
        format!("{}", rounded.round() as i32)
    } else {
        format!("{rounded:.2}")
    }
}

fn hsla_to_rgb8(color: Hsla) -> (u8, u8, u8) {
    let h = color.h.fract() * 6.0;
    let s = color.s.clamp(0.0, 1.0);
    let l = color.l.clamp(0.0, 1.0);

    if s <= f32::EPSILON {
        let gray = (l * 255.0).round() as u8;
        return (gray, gray, gray);
    }

    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let r = hue_to_channel(p, q, h + 2.0);
    let g = hue_to_channel(p, q, h);
    let b = hue_to_channel(p, q, h - 2.0);
    ((r * 255.0).round() as u8, (g * 255.0).round() as u8, (b * 255.0).round() as u8)
}

fn hue_to_channel(p: f32, q: f32, t: f32) -> f32 {
    let mut t = t;
    if t < 0.0 {
        t += 6.0;
    }
    if t >= 6.0 {
        t -= 6.0;
    }
    if t < 1.0 {
        p + (q - p) * t
    } else if t < 3.0 {
        q
    } else if t < 4.0 {
        p + (q - p) * (4.0 - t)
    } else {
        p
    }
}
