//! Multi mixer composition — per-color-space channel mixers.

use xgpui_luma::controls::slider::SliderEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, FontWeight, Render, Window, div, prelude::*, px};
use xgpui_luma_color::ColorSwatch;
use xgpui_luma_color::color_slider::color_spec::{
    ColorChannel, ColorSpecification, Hsl, Hsv, HueAlpha, Lab, Oklch, RgbaSpec, slider_step_for_channel,
};
use xgpui_luma_color::color_slider::{
    AlphaDelegate, ChannelDelegate, ColorSliderBuilder, ColorSliderDelegate, ColorSliderDomainRenderer, HueDelegate,
    primary_slider_value, refresh_color_slider, update_domain_delegate,
};
use xgpui_luma::controls::slider::{SliderControl, SliderEvent};
use xgpui_luma::theme::ControlSize;
use xgpui_luma::{wrappanel};
use xgpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextRole, ShadcnTextSize};

use super::super::color_exposition_common::{
    detail_row, format_compact_hsla, format_hex_color, composition_inset_radius, render_field_card,
    slider_labeled_row_wide,
};

pub struct MultiMixerDemo {
    look: Arc<ShadcnLook>,
    hue_alpha: Entity<ColorSpaceMixerState<HueAlpha>>,
    rgb: Entity<ColorSpaceMixerState<RgbaSpec>>,
    hsla: Entity<ColorSpaceMixerState<Hsl>>,
    hsva: Entity<ColorSpaceMixerState<Hsv>>,
    lab: Entity<ColorSpaceMixerState<Lab>>,
    lab_auto: Entity<ColorSpaceMixerState<Lab>>,
    lab_dynamic: Entity<ColorSpaceMixerState<Lab>>,
    oklch: Entity<ColorSpaceMixerState<Oklch>>,
}

impl MultiMixerDemo {
    pub fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let initial = gpui::hsla(0.55, 1.0, 0.5, 1.0);
        let hue_alpha = cx.new(|cx| {
            ColorSpaceMixerState::new("hue-alpha", None, true, look.clone(), HueAlpha::from_hsla(initial), cx)
        });
        let rgb =
            cx.new(|cx| ColorSpaceMixerState::new("rgb", None, true, look.clone(), RgbaSpec::from_hsla(initial), cx));
        let hsla =
            cx.new(|cx| ColorSpaceMixerState::new("hsla", None, true, look.clone(), Hsl::from_hsla(initial), cx));
        let hsva =
            cx.new(|cx| ColorSpaceMixerState::new("hsva", None, true, look.clone(), Hsv::from_hsla(initial), cx));
        let lab = cx.new(|cx| {
            ColorSpaceMixerState::new("lab", Some("Unclamped"), true, look.clone(), Lab::from_hsla(initial), cx)
        });
        let lab_auto = cx.new(|cx| {
            let mut spec = Lab::from_hsla(initial);
            spec.set_auto_clamp(true);
            ColorSpaceMixerState::new("lab-auto", Some("Auto-clamped"), true, look.clone(), spec, cx)
        });
        let lab_dynamic = cx.new(|cx| {
            let mut spec = Lab::from_hsla(initial);
            spec.set_dynamic_range(true);
            ColorSpaceMixerState::new("lab-dynamic", Some("Dynamic range"), true, look.clone(), spec, cx)
        });
        let oklch = cx.new(|cx| {
            ColorSpaceMixerState::new(
                "oklch",
                Some("Gamut-clamped"),
                false,
                look.clone(),
                Oklch::from_hsla(initial),
                cx,
            )
        });

        Self { look, hue_alpha, rgb, hsla, hsva, lab, lab_auto, lab_dynamic, oklch }
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.hue_alpha.update(cx, |mixer, cx| mixer.sync_look(look.clone(), cx));
        self.rgb.update(cx, |mixer, cx| mixer.sync_look(look.clone(), cx));
        self.hsla.update(cx, |mixer, cx| mixer.sync_look(look.clone(), cx));
        self.hsva.update(cx, |mixer, cx| mixer.sync_look(look.clone(), cx));
        self.lab.update(cx, |mixer, cx| mixer.sync_look(look.clone(), cx));
        self.lab_auto.update(cx, |mixer, cx| mixer.sync_look(look.clone(), cx));
        self.lab_dynamic.update(cx, |mixer, cx| mixer.sync_look(look.clone(), cx));
        self.oklch.update(cx, |mixer, cx| mixer.sync_look(look.clone(), cx));
        cx.notify();
    }

    pub fn hsva_hue_slider(&self, cx: &gpui::App) -> Entity<SliderControl> {
        self.hsva.read(cx).primary_slider()
    }
}

impl Render for MultiMixerDemo {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let look = &self.look;

        div().w_full().flex().flex_col().gap(px(28.0)).child(
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
                                .typography_style(look.typography_role(ShadcnTextRole::H4))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(look.chrome().title_text)
                                .child("Mixer Modes"),
                        )
                        .child(
                            div()
                                .typography_style(look.typography_scale(ShadcnTextSize::Sm))
                                .text_color(look.chrome().muted_text)
                                .child("Representative mixers for the upstream mode set."),
                        ),
                )
                .child(
                    wrappanel! {
                        orientation=horizontal gap=16.0 align=start;
                        mixer_card(look, "Hue + Alpha", self.hue_alpha.clone()),
                        mixer_card(look, "RGB", self.rgb.clone()),
                        mixer_card(look, "HSLA", self.hsla.clone()),
                        mixer_card(look, "HSVA", self.hsva.clone()),
                        mixer_card(look, "Lab", self.lab.clone()),
                        mixer_card(look, "Lab Auto-clamped", self.lab_auto.clone()),
                        mixer_card(look, "Lab Dynamic Range", self.lab_dynamic.clone()),
                        mixer_card(look, "OKLCH", self.oklch.clone())
                    }
                    .w_full(),
                ),
        )
    }
}

fn mixer_card(look: &ShadcnLook, title: &'static str, mixer: Entity<impl gpui::Render>) -> gpui::AnyElement {
    render_field_card(look, title, "Interactive per-space channel mixer.", 340.0, mixer)
}

struct MixerSliderRow {
    channel: ColorChannel,
    slider: Entity<SliderControl>,
    domain_renderer: Arc<ColorSliderDomainRenderer>,
}

struct ColorSpaceMixerState<S: ColorSpecification> {
    subtitle: Option<&'static str>,
    show_gamut_warning: bool,
    look: Arc<ShadcnLook>,
    spec: S,
    sliders: Vec<MixerSliderRow>,
}

impl<S: ColorSpecification> ColorSpaceMixerState<S> {
    fn new(
        id_prefix: &'static str,
        subtitle: Option<&'static str>,
        show_gamut_warning: bool,
        look: Arc<ShadcnLook>,
        spec: S,
        cx: &mut Context<Self>,
    ) -> Self {
        let channels = spec.channels().to_vec();
        let mut sliders = Vec::with_capacity(channels.len());

        for channel in &channels {
            let channel_meta = *channel;
            let initial_value = spec.get_value(channel_meta.name);
            let builder = if channel_meta.name == "alpha" {
                ColorSliderBuilder::alpha(
                    format!("controls-doc-multi-mixer-{id_prefix}-{}", channel_meta.name),
                    initial_value,
                    spec,
                )
            } else if channel_meta.name == "hue" && spec.uses_rainbow_hue_track() {
                ColorSliderBuilder::hue(
                    format!("controls-doc-multi-mixer-{id_prefix}-{}", channel_meta.name),
                    initial_value,
                )
            } else {
                ColorSliderBuilder::channel(
                    format!("controls-doc-multi-mixer-{id_prefix}-{}", channel_meta.name),
                    initial_value,
                    spec,
                    channel_meta.name,
                )
                .expect("color mixer channel delegate should be valid")
            };
            let domain_renderer = builder.domain_renderer();
            let slider = builder
                .size(ControlSize::Sm)
                .thumb_medium()
                .edge_to_edge()
                .range(channel_meta.min..channel_meta.max)
                .step(slider_step_for_channel(&channel_meta))
                .spawn(cx);
            sliders.push(MixerSliderRow { channel: channel_meta, slider, domain_renderer });
        }

        sliders.iter().for_each(|row| {
            let channel_name = row.channel.name;
            row.slider.clone().on_event(cx, move |this, event: &SliderEvent, cx| {
                if let Some(value) = primary_slider_value(event) {
                    this.handle_slider_event(channel_name, value, cx);
                }
            });
        });

        let state = Self { subtitle, show_gamut_warning, look, spec, sliders };
        state.sync_sliders(cx);
        state
    }

    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        for row in &self.sliders {
            row.slider.update(cx, |_, cx| cx.notify());
        }
        cx.notify();
    }

    fn handle_slider_event(&mut self, channel_name: &'static str, proposed: f32, cx: &mut Context<Self>) {
        self.spec.apply_channel_slider_value(channel_name, proposed);
        // This existing Studio demo explicitly opts into gamut-clamped interaction.
        if self.subtitle == Some("Gamut-clamped") {
            self.spec.clamp_spec_to_gamut();
        }
        self.sync_sliders(cx);
        cx.notify();
    }

    fn sync_sliders(&self, cx: &mut Context<Self>) {
        let spec = self.spec;
        for row in &self.sliders {
            let channel = row.channel;
            let (min, max) = spec.channel_bounds(channel.name);
            let mut context = row.domain_renderer.context();
            context.range = min..max;
            update_domain_delegate(&row.domain_renderer, mixer_delegate(spec, channel.name), context);
            row.slider.update(cx, |slider, cx| {
                slider.set_range(min..max, cx);
                let track_intervals = spec.channel_track_intervals(channel.name);
                if track_intervals.is_empty() {
                    slider.clear_track_intervals(cx);
                } else {
                    slider.set_track_intervals(track_intervals, cx);
                }
                if spec.uses_continuous_channel_interaction(channel.name) {
                    slider.clear_allowed_intervals(cx);
                } else {
                    let intervals = spec.channel_allowed_intervals(channel.name);
                    if intervals.is_empty() {
                        slider.clear_allowed_intervals(cx);
                    } else {
                        slider.set_allowed_intervals(intervals, cx);
                    }
                }
                slider.set_value(spec.get_value(channel.name), cx);
            });
            refresh_color_slider(&row.slider, cx);
        }
    }

    fn primary_slider(&self) -> Entity<SliderControl> {
        self.sliders.first().expect("mixer has sliders").slider.clone()
    }
}

fn mixer_delegate<S: ColorSpecification>(spec: S, channel_name: &'static str) -> Arc<dyn ColorSliderDelegate> {
    if channel_name == "alpha" {
        Arc::new(AlphaDelegate { spec })
    } else if channel_name == "hue" && spec.uses_rainbow_hue_track() {
        Arc::new(HueDelegate)
    } else {
        Arc::new(ChannelDelegate::new(spec, channel_name.into()).expect("color mixer channel delegate should be valid"))
    }
}

impl<S: ColorSpecification> Render for ColorSpaceMixerState<S> {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let color = self.spec.to_hsla();
        let look = &self.look;

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .when_some(self.subtitle, |container, subtitle| {
                container.child(
                    div()
                        .typography_style(look.typography_scale(ShadcnTextSize::Xs))
                        .text_color(look.chrome().muted_text)
                        .child(subtitle),
                )
            })
            .child(
                ColorSwatch::new(self.spec.to_color_value())
                    .checkerboard(true)
                    .height(px(44.0))
                    .rounded(px(composition_inset_radius(look))),
            )
            .child(detail_row(look, "Hex", format_hex_color(color)))
            .child(detail_row(look, "HSLA", format_compact_hsla(color)))
            .child(detail_row(look, "Spec", self.spec.summary()))
            .when(self.show_gamut_warning && self.spec.is_out_of_gamut(), |div| {
                div.child(detail_row(look, "Gamut", "Out of gamut".to_string()))
            })
            .children(
                self.sliders.iter().map(|row| slider_labeled_row_wide(look, row.channel.label, row.slider.clone())),
            )
    }
}
