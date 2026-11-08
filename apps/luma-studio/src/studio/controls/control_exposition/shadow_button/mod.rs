mod button;

use xgpui_luma::controls::textfield::TextFieldEventExt;
use xgpui_luma::controls::slider::SliderEventExt;
use xgpui_luma::controls::toggle::ToggleEventExt;
use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Entity, FontWeight, Hsla, IntoElement, Render, SharedString, Window, div, prelude::*, px,
};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::button_family::{ButtonFamilyRole, ButtonSize};
use xgpui_luma::controls::button::{Button, ButtonContentContext, ButtonEvent, ButtonRenderModel, ButtonTemplate};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma::controls::slider::{Slider, SliderEvent};
use xgpui_luma::controls::textfield::{TextField, TextFieldEvent, Validator};
use xgpui_luma::controls::toggle::{Toggle, ToggleEvent};
use xgpui_luma::theme::{InteractionState, LumaChrome};
use xgpui_luma::{hstack, vstack};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::color_exposition_common::format_compact_hsla;
use super::model::ControlExpositionLayout;
use super::template::render_control_exposition_card;

use button::{
    ButtonVisualState, ShadowButtonControls, ShadowButtonSpec, default_shadow_color_placeholder, format_css_box_shadow,
    format_rgb_triplet, prototype_shadow_button_template, resolve_shadow_spec, resolved_shadow_color,
};

#[derive(Clone, Copy, Debug)]
enum ShadowSliderField {
    OffsetX,
    OffsetY,
    Blur,
    Spread,
    Opacity,
}

pub struct ShadowButtonControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    demo_button: Entity<Button>,
    state_preview: Entity<ButtonStatePreview>,
    reset_button: Entity<Button>,
    bounds_toggle: Toggle,
    color_field: TextField,
    offset_x_slider: Slider,
    offset_y_slider: Slider,
    blur_slider: Slider,
    spread_slider: Slider,
    opacity_slider: Slider,
    demo_clicks: usize,
    shadow: ShadowButtonControls,
    color_input: SharedString,
    color_override: Option<Hsla>,
    color_input_valid: bool,
    show_bounds: bool,
    suppress_control_events: bool,
}

impl ShadowButtonControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("shadow-button").expect("shadow-button catalog entry");
        let shadow = ShadowButtonControls::default();
        let template = prototype_shadow_button_template(look.clone(), shadow, None);
        let default_shadow_color = resolved_shadow_color(&look);

        let demo_button = Button::new("controls-doc-shadow-demo-button")
            .label("Floating Action")
            .template(template.clone())
            .spawn(cx);
        let state_preview = cx.new(|_| ButtonStatePreview::new(look.clone(), template.clone()));
        let reset_button = shadcn::Button::new("controls-doc-shadow-reset")
            .look(look.as_ref())
            .secondary()
            .label("Reset Shadow")
            .spawn(cx);
        let bounds_toggle = shadcn::Toggle::new("controls-doc-shadow-show-bounds")
            .look(look.as_ref())
            .secondary()
            .with_data(true)
            .content(|_, _| div().child("Show Bounds").into_any_element())
            .spawn(cx);
        let color_field = shadcn::TextField::new("controls-doc-shadow-color")
            .look(look.as_ref())
            .placeholder(default_shadow_color_placeholder(default_shadow_color))
            .full_width(true)
            .clean_on_escape(true)
            .validator(shadow_color_validator())
            .spawn(cx);

        let offset_x_slider = shadcn::Slider::new("controls-doc-shadow-offset-x")
            .look(look.as_ref())
            .range(-20..20)
            .step(1)
            .value(shadow.offset_x)
            .spawn(cx);
        let offset_y_slider = shadcn::Slider::new("controls-doc-shadow-offset-y")
            .look(look.as_ref())
            .range(-8..24)
            .step(1)
            .value(shadow.offset_y)
            .spawn(cx);
        let blur_slider = shadcn::Slider::new("controls-doc-shadow-blur")
            .look(look.as_ref())
            .range(0..32)
            .step(1)
            .value(shadow.blur_radius)
            .spawn(cx);
        let spread_slider = shadcn::Slider::new("controls-doc-shadow-spread")
            .look(look.as_ref())
            .range(-12..20)
            .step(1)
            .value(shadow.spread_radius)
            .spawn(cx);
        let opacity_slider = shadcn::Slider::new("controls-doc-shadow-opacity")
            .look(look.as_ref())
            .range(0..100)
            .step(1)
            .value(shadow.opacity * 100.0)
            .spawn(cx);

        demo_button.clone().on_event(cx, move |this, event: &ButtonEvent, cx| {
            this.handle_button_event(event, cx);
        });
        reset_button.clone().on_event(cx, move |this, event: &ButtonEvent, cx| {
            this.handle_reset(event, cx);
        });
        bounds_toggle.clone().on_event(cx, move |this, event: &ToggleEvent, cx| {
            this.handle_bounds_toggle(event, cx);
        });
        color_field.clone().on_event(cx, move |this, event: &TextFieldEvent, cx| {
            this.handle_color_field_event(event, cx);
        });
        for (slider, field) in [
            (offset_x_slider.clone(), ShadowSliderField::OffsetX),
            (offset_y_slider.clone(), ShadowSliderField::OffsetY),
            (blur_slider.clone(), ShadowSliderField::Blur),
            (spread_slider.clone(), ShadowSliderField::Spread),
            (opacity_slider.clone(), ShadowSliderField::Opacity),
        ] {
            slider.clone().on_event(cx, move |this, event: &SliderEvent, cx| {
                this.handle_slider_event(field, event, cx);
            });
        }

        Self {
            look,
            entry,
            demo_button,
            state_preview,
            reset_button,
            bounds_toggle,
            color_field,
            offset_x_slider,
            offset_y_slider,
            blur_slider,
            spread_slider,
            opacity_slider,
            demo_clicks: 0,
            shadow,
            color_input: SharedString::default(),
            color_override: None,
            color_input_valid: true,
            show_bounds: true,
            suppress_control_events: false,
        }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.demo_button.update(cx, |_, cx| cx.notify());
        self.state_preview.update(cx, |preview, cx| preview.sync_look(look.clone(), cx));
        self.reset_button.update(cx, |_, cx| cx.notify());
        self.bounds_toggle.update(cx, |_, cx| cx.notify());
        self.suppress_control_events = true;
        self.sync_color_field(cx);
        self.suppress_control_events = false;
        self.color_field.update(cx, |_, cx| cx.notify());
        for slider in [
            &self.offset_x_slider,
            &self.offset_y_slider,
            &self.blur_slider,
            &self.spread_slider,
            &self.opacity_slider,
        ] {
            slider.update(cx, |_, cx| cx.notify());
        }
        cx.notify();
    }

    fn handle_button_event(&mut self, event: &ButtonEvent, cx: &mut Context<Self>) {
        if self.suppress_control_events {
            return;
        }
        if matches!(event, ButtonEvent::Click) {
            self.demo_clicks += 1;
            let label = format!("Floating Action ({})", self.demo_clicks);
            self.demo_button.update(cx, |button, cx| button.set_label(label, cx));
        }
    }

    fn handle_reset(&mut self, event: &ButtonEvent, cx: &mut Context<Self>) {
        if self.suppress_control_events {
            return;
        }
        if matches!(event, ButtonEvent::Click) {
            self.shadow = ShadowButtonControls::default();
            self.color_input = SharedString::default();
            self.color_override = None;
            self.color_input_valid = true;
            self.suppress_control_events = true;
            self.sync_sliders(cx);
            self.sync_color_field(cx);
            self.suppress_control_events = false;
            self.apply_template(cx);
        }
    }

    fn handle_bounds_toggle(&mut self, event: &ToggleEvent, cx: &mut Context<Self>) {
        if self.suppress_control_events {
            return;
        }
        if let ToggleEvent::Change { selected } = event {
            self.show_bounds = *selected;
            cx.notify();
        }
    }

    fn handle_color_field_event(&mut self, event: &TextFieldEvent, cx: &mut Context<Self>) {
        if self.suppress_control_events {
            return;
        }
        match event {
            TextFieldEvent::Change { value } | TextFieldEvent::Submit { value } => {
                self.color_input = value.clone().into();
                let trimmed = value.trim();

                if trimmed.is_empty() {
                    self.color_override = None;
                    self.color_input_valid = true;
                    self.apply_template(cx);
                } else if let Some(color) = parse_shadow_color_input(trimmed) {
                    self.color_override = Some(color);
                    self.color_input_valid = true;
                    self.apply_template(cx);
                } else {
                    self.color_input_valid = false;
                    cx.notify();
                }
            }
            TextFieldEvent::FocusChanged { .. } => {}
            _ => {}
        }
    }

    fn handle_slider_event(&mut self, field: ShadowSliderField, event: &SliderEvent, cx: &mut Context<Self>) {
        if self.suppress_control_events {
            return;
        }
        let value = match event {
            SliderEvent::Change { value, .. } | SliderEvent::Release { value, .. } => *value,
            _ => return,
        };
        match field {
            ShadowSliderField::OffsetX => self.shadow.offset_x = value,
            ShadowSliderField::OffsetY => self.shadow.offset_y = value,
            ShadowSliderField::Blur => self.shadow.blur_radius = value,
            ShadowSliderField::Spread => self.shadow.spread_radius = value,
            ShadowSliderField::Opacity => self.shadow.opacity = (value / 100.0).clamp(0.0, 1.0),
        }

        self.apply_template(cx);
    }

    fn sync_sliders(&mut self, cx: &mut Context<Self>) {
        self.offset_x_slider.update(cx, |slider, cx| slider.set_value(self.shadow.offset_x, cx));
        self.offset_y_slider.update(cx, |slider, cx| slider.set_value(self.shadow.offset_y, cx));
        self.blur_slider.update(cx, |slider, cx| slider.set_value(self.shadow.blur_radius, cx));
        self.spread_slider.update(cx, |slider, cx| slider.set_value(self.shadow.spread_radius, cx));
        self.opacity_slider.update(cx, |slider, cx| slider.set_value(self.shadow.opacity * 100.0, cx));
    }

    fn sync_color_field(&mut self, cx: &mut Context<Self>) {
        let placeholder = default_shadow_color_placeholder(resolved_shadow_color(&self.look));
        let value = self.color_input.clone();

        self.color_field.update(cx, |text_field, cx| {
            text_field.set_placeholder(placeholder, cx);
            text_field.set_value(value.as_ref(), cx);
        });
    }

    fn apply_template(&mut self, cx: &mut Context<Self>) {
        let template = prototype_shadow_button_template(self.look.clone(), self.shadow, self.color_override);
        self.demo_button.update(cx, |button, cx| {
            button.set_template(template.clone(), cx);
        });
        self.state_preview.update(cx, |preview, cx| {
            preview.set_template(template.clone(), cx);
        });
        cx.notify();
    }

    fn base_spec(&self, look: &ShadcnLook) -> ShadowButtonSpec {
        let mut spec = self.shadow.base_spec(look);
        if let Some(color) = self.color_override {
            spec.color = color;
        }
        spec
    }

    fn color_help_text(&self, look: &ShadcnLook) -> String {
        if self.color_input.is_empty() {
            format!("Theme default: {}", format_rgb_triplet(resolved_shadow_color(look)))
        } else if let Some(color) = self.color_override {
            format!("Using: {}", format_rgb_triplet(color))
        } else {
            String::from("Invalid color. Use 255, 255, 255, rgb(255,255,255), or #ffffff.")
        }
    }

    fn render_demo_column(&self, look: &ShadcnLook) -> AnyElement {
        let chrome = look.chrome();
        let foreground = look.token_color("foreground").unwrap_or(chrome.title_text);
        let base_spec = self.base_spec(look);
        let pressed_spec = resolve_shadow_spec(base_spec, ButtonVisualState::Pressed).unwrap_or(base_spec);

        vstack! {
            gap=18.0;
            render_demo_stage(
                self.demo_button.clone(),
                self.demo_clicks,
                self.show_bounds,
                foreground,
                chrome,
            ),
            render_shadow_summary_card(chrome, base_spec, pressed_spec),
            self.state_preview.clone(),
        }
        .w(px(420.0))
        .max_w_full()
        .items_stretch()
        .into_any_element()
    }

    fn render_controls_column(&self, look: &ShadcnLook) -> AnyElement {
        let chrome = look.chrome();
        let base_spec = self.base_spec(look);
        let color_help_text = self.color_help_text(look);
        let color_help_color = if self.color_input_valid {
            chrome.muted_text
        } else {
            look.token_color("destructive").unwrap_or(chrome.body_text)
        };

        vstack! {
            gap=12.0;
            vstack! {
                gap=14.0;
                render_controls_intro(chrome),
                render_color_chip_row(base_spec.color, self.reset_button.clone(), chrome),
                hstack! { justify=end; self.bounds_toggle.clone() },
                render_text_field_row(
                    "Color",
                    "RGB, rgb(...), or #hex",
                    self.color_field.clone(),
                    color_help_text,
                    color_help_color,
                    chrome,
                ),
                render_slider_row(
                    "X offset",
                    format!("{:.0}px", self.shadow.offset_x),
                    self.offset_x_slider.clone(),
                    chrome,
                ),
                render_slider_row(
                    "Y offset",
                    format!("{:.0}px", self.shadow.offset_y),
                    self.offset_y_slider.clone(),
                    chrome,
                ),
                render_slider_row(
                    "Blur",
                    format!("{:.0}px", self.shadow.blur_radius),
                    self.blur_slider.clone(),
                    chrome,
                ),
                render_slider_row(
                    "Spread",
                    format!("{:.0}px", self.shadow.spread_radius),
                    self.spread_slider.clone(),
                    chrome,
                ),
                render_slider_row(
                    "Opacity",
                    format!("{:.0}%", self.shadow.opacity * 100.0),
                    self.opacity_slider.clone(),
                    chrome,
                ),
                render_css_shadow_value(base_spec, chrome),
            }
            .border_1()
            .border_color(chrome.border)
            .bg(chrome.panel_background)
            .rounded(px(10.0))
            .p(px(14.0))
        }
        .w(px(420.0))
        .max_w_full()
        .into_any_element()
    }
}

impl Render for ShadowButtonControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let look = &self.look;
            let preview = hstack! {
                justify=center;
                div()
                    .w_full()
                    .max_w(px(980.0))
                    .flex()
                    .flex_wrap()
                    .items_start()
                    .justify_center()
                    .gap(px(32.0))
                    .child(self.render_demo_column(look))
                    .child(self.render_controls_column(look))
            }
            .w_full()
            .into_any_element();

            render_control_exposition_card(look, self.entry, preview, None, ControlExpositionLayout::BORDERLESS)
        })
    }
}

#[derive(Clone)]
struct ButtonStatePreview {
    look: Arc<ShadcnLook>,
    template: Arc<dyn ButtonTemplate<()>>,
}

struct ButtonStateSample {
    id: &'static str,
    label: &'static str,
    state: InteractionState,
}

impl ButtonStatePreview {
    fn new(look: Arc<ShadcnLook>, template: Arc<dyn ButtonTemplate<()>>) -> Self {
        Self { look, template }
    }

    fn set_template(&mut self, template: Arc<dyn ButtonTemplate<()>>, cx: &mut Context<Self>) {
        self.template = template;
        cx.notify();
    }

    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        cx.notify();
    }
}

impl Render for ButtonStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let samples = [
            ButtonStateSample { id: "default", label: "Default", state: InteractionState::default() },
            ButtonStateSample {
                id: "hover",
                label: "Hover",
                state: InteractionState { hovered: true, ..InteractionState::default() },
            },
            ButtonStateSample {
                id: "focus",
                label: "Focus",
                state: InteractionState { focused: true, ..InteractionState::default() },
            },
            ButtonStateSample {
                id: "active",
                label: "Pressed",
                state: InteractionState { hovered: true, pressed: true, focused: true, ..InteractionState::default() },
            },
            ButtonStateSample {
                id: "disabled",
                label: "Disabled",
                state: InteractionState { disabled: true, ..InteractionState::default() },
            },
        ];

        vstack! {
            gap=10.0;
            div()
                .text_size(px(12.0))
                .line_height(px(16.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(chrome.muted_text)
                .child("Template state preview"),
            hstack! {}
                .w_full()
                .flex_wrap()
                .items_start()
                .justify_start()
                .gap(px(16.0))
                .children(samples.into_iter().map(|sample| {
                    render_button_state_sample(&self.template, sample, chrome.muted_text, window, cx)
                })),
        }
        .w_full()
        .mt(px(2.0))
    }
}

fn render_shadow_summary_card(
    chrome: LumaChrome,
    base_spec: ShadowButtonSpec,
    pressed_spec: ShadowButtonSpec,
) -> AnyElement {
    vstack! {
        gap=8.0;
        div()
            .text_size(px(11.0))
            .line_height(px(15.0))
            .font_family("Monaco")
            .text_color(chrome.muted_text)
            .child("effective shadow spec"),
        render_summary_row("color", format_compact_hsla(base_spec.color), chrome),
        render_summary_row("x", format!("{:.0}px", base_spec.offset_x), chrome),
        render_summary_row("y", format!("{:.0}px", base_spec.offset_y), chrome),
        render_summary_row("blur", format!("{:.0}px", base_spec.blur_radius), chrome),
        render_summary_row("spread", format!("{:.0}px", base_spec.spread_radius), chrome),
        render_summary_row("pressed y", format!("{:.0}px", pressed_spec.offset_y), chrome),
        render_summary_row("pressed opacity", format!("{:.0}%", pressed_spec.opacity * 100.0), chrome),
    }
    .w_full()
    .border_1()
    .border_color(chrome.border)
    .rounded(px(10.0))
    .bg(chrome.panel_background)
    .p(px(12.0))
    .into_any_element()
}

fn render_demo_stage(
    demo_button: Entity<Button>,
    demo_clicks: usize,
    show_bounds: bool,
    foreground: Hsla,
    chrome: LumaChrome,
) -> AnyElement {
    vstack! {
        gap=10.0;
        div()
            .relative()
            .border_1()
            .border_color(if show_bounds { foreground } else { foreground.opacity(0.0) })
            .px(px(36.0))
            .pt(px(8.0))
            .pb(px(10.0))
            .child(demo_button),
        div()
            .text_size(px(12.0))
            .line_height(px(16.0))
            .text_color(chrome.muted_text)
            .child(format!("Clicks: {demo_clicks}")),
    }
    .w_full()
    .items_center()
    .into_any_element()
}

fn render_controls_intro(chrome: LumaChrome) -> AnyElement {
    vstack! {
        gap=2.0;
        div()
            .text_size(px(16.0))
            .line_height(px(22.0))
            .font_weight(FontWeight::MEDIUM)
            .text_color(chrome.title_text)
            .child("Shadow Tuning"),
        div()
            .text_size(px(12.0))
            .line_height(px(17.0))
            .text_color(chrome.muted_text)
            .child("CSS-style shadow prototype: x, y, blur, spread, and a theme-aware neutral shadow tone."),
    }
    .into_any_element()
}

fn render_color_chip_row(color: Hsla, reset_button: Entity<Button>, chrome: LumaChrome) -> AnyElement {
    hstack! {
        gap=12.0;
        hstack! {
            gap=8.0;
            div().size(px(28.0)).rounded_full().bg(color),
            div()
                .font_family("Monaco")
                .text_size(px(11.0))
                .line_height(px(15.0))
                .text_color(chrome.body_text)
                .child(format_compact_hsla(color)),
        },
        reset_button,
    }
    .items_center()
    .justify_between()
    .into_any_element()
}

fn render_summary_row(label: &'static str, value: String, chrome: LumaChrome) -> AnyElement {
    hstack! {
        gap=8.0;
        div().text_size(px(12.0)).line_height(px(16.0)).text_color(chrome.body_text).child(label),
        div()
            .font_family("Monaco")
            .text_size(px(11.0))
            .line_height(px(15.0))
            .text_color(chrome.muted_text)
            .child(value),
    }
    .items_center()
    .justify_between()
    .into_any_element()
}

fn render_slider_row(label: &'static str, value: String, slider: Slider, chrome: LumaChrome) -> AnyElement {
    vstack! {
        gap=6.0;
        hstack! {
            gap=8.0;
            div()
                .text_size(px(12.0))
                .line_height(px(16.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(chrome.body_text)
                .child(label),
            div()
                .font_family("Monaco")
                .text_size(px(11.0))
                .line_height(px(15.0))
                .text_color(chrome.muted_text)
                .child(value),
        }
        .items_center()
        .justify_between(),
        slider,
    }
    .into_any_element()
}

fn render_text_field_row(
    label: &'static str,
    value_hint: &'static str,
    text_field: TextField,
    assistive_text: String,
    assistive_color: Hsla,
    chrome: LumaChrome,
) -> AnyElement {
    vstack! {
        gap=6.0;
        hstack! {
            gap=8.0;
            div()
                .text_size(px(12.0))
                .line_height(px(16.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(chrome.body_text)
                .child(label),
            div()
                .font_family("Monaco")
                .text_size(px(11.0))
                .line_height(px(15.0))
                .text_color(chrome.muted_text)
                .child(value_hint),
        }
        .items_center()
        .justify_between(),
        text_field,
        div()
            .text_size(px(11.0))
            .line_height(px(15.0))
            .text_color(assistive_color)
            .child(assistive_text),
    }
    .into_any_element()
}

fn render_css_shadow_value(spec: ShadowButtonSpec, chrome: LumaChrome) -> AnyElement {
    vstack! {
        gap=6.0;
        div()
            .text_size(px(12.0))
            .line_height(px(16.0))
            .font_weight(FontWeight::MEDIUM)
            .text_color(chrome.body_text)
            .child("CSS box-shadow"),
        div()
            .w_full()
            .border_1()
            .border_color(chrome.border)
            .bg(chrome.content_background)
            .rounded(px(8.0))
            .p(px(10.0))
            .font_family("Monaco")
            .text_size(px(11.0))
            .line_height(px(16.0))
            .text_color(chrome.body_text)
            .child(format_css_box_shadow(spec)),
    }
    .into_any_element()
}

fn render_button_state_sample(
    template: &Arc<dyn ButtonTemplate<()>>,
    sample: ButtonStateSample,
    label_color: Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("controls-doc-shadow-preview-{}", sample.id));
    let label = SharedString::from("Button");
    let content: xgpui_luma::controls::button::ControlPresenter<ButtonContentContext<()>> =
        Arc::new(move |_: &ButtonContentContext<()>, _| div().child(label.clone()).into_any_element());
    let model = ButtonRenderModel {
        id,
        data: (),
        content,
        role: ButtonFamilyRole::Text,
        size: ButtonSize::Md,
        state: sample.state,
        round: false,
        radius_override: std::cell::Cell::new(None),
        elevation: true,
        compact: false,
        look: None,
        ..Default::default()
    };

    vstack! {
        gap=6.0;
        template.render(&model, window, cx),
        div()
            .text_size(px(11.0))
            .line_height(px(15.0))
            .text_color(label_color)
            .child(sample.label),
    }
    .w(px(120.0))
    .items_center()
    .into_any_element()
}

fn shadow_color_validator() -> Validator {
    Arc::new(|value: &str| value.trim().is_empty() || parse_shadow_color_input(value).is_some())
}

fn parse_shadow_color_input(raw: &str) -> Option<Hsla> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(color) = parse_hex_color(trimmed) {
        return Some(color);
    }

    let component_source = if trimmed.len() >= 5 && trimmed[..4].eq_ignore_ascii_case("rgb(") && trimmed.ends_with(')')
    {
        &trimmed[4..trimmed.len() - 1]
    } else {
        trimmed
    };

    let components: Vec<&str> = component_source
        .split(|ch: char| ch == ',' || ch.is_ascii_whitespace())
        .filter(|part| !part.is_empty())
        .collect();

    if components.len() != 3 {
        return None;
    }

    let r = components[0].parse::<u8>().ok()?;
    let g = components[1].parse::<u8>().ok()?;
    let b = components[2].parse::<u8>().ok()?;
    Some(gpui::rgb(rgb_hex(r, g, b)).into())
}

fn parse_hex_color(raw: &str) -> Option<Hsla> {
    let hex = raw.trim().trim_start_matches('#');
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some(gpui::rgb(rgb_hex(r, g, b)).into())
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(gpui::rgb(rgb_hex(r, g, b)).into())
        }
        _ => None,
    }
}

fn rgb_hex(r: u8, g: u8, b: u8) -> u32 {
    (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
}
