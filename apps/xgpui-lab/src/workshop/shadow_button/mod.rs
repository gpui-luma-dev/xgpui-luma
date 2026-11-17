//! Studio's Shadow Button prototype, copied locally and hosted with Lab chrome.
mod button;
use std::sync::Arc;
use gpui::{Context, Entity, Hsla, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::{
    button::{Button, HasPresenter},
    slider::{self, SliderEvent},
    textfield::{self, TextFieldEvent},
    toggle::{self, ToggleEvent},
};
use crate::theme::{ColorVizLookExt, Look, TextSize, TypographyExt};
use super::{
    event_stream::ControlEventStream,
    sample_layout::{card, page},
};
use button::{
    ShadowButtonControls, prototype_shadow_button_template, resolved_shadow_color, default_shadow_color_placeholder,
    format_css_box_shadow, format_rgb_triplet,
};

#[derive(Clone, Copy)]
enum Field {
    X,
    Y,
    Blur,
    Spread,
    Opacity,
}
impl Field {
    fn title(self) -> &'static str {
        match self {
            Self::X => "X offset",
            Self::Y => "Y offset",
            Self::Blur => "Blur",
            Self::Spread => "Spread",
            Self::Opacity => "Opacity (%)",
        }
    }
    fn value(self, controls: ShadowButtonControls) -> f32 {
        match self {
            Self::X => controls.offset_x,
            Self::Y => controls.offset_y,
            Self::Blur => controls.blur_radius,
            Self::Spread => controls.spread_radius,
            Self::Opacity => controls.opacity * 100.,
        }
    }
}
pub(super) struct ShadowButtonSample {
    look: Arc<Look>,
    demo: Entity<Button>,
    reset: Entity<Button>,
    bounds_toggle: toggle::Toggle,
    color_field: textfield::TextField,
    sliders: Vec<(Field, slider::Slider)>,
    controls: ShadowButtonControls,
    color_override: Option<Hsla>,
    color_valid: bool,
    show_bounds: bool,
    clicks: usize,
    events: Entity<ControlEventStream>,
}
impl ShadowButtonSample {
    pub(super) fn new(look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let controls = ShadowButtonControls::default();
        let demo = Button::new("workshop-shadow-demo")
            .content(|_, _| div().child(crate::l10n::ui("Floating Action")))
            .template(prototype_shadow_button_template(controls, None))
            .on_click(cx, |this: &mut Self, cx| {
                this.clicks += 1;
                this.record("SDK Button clicked", cx);
            })
            .spawn(cx);
        let reset = Button::new("workshop-shadow-reset")
            .content(|_, _| div().child(crate::l10n::ui("Reset Shadow")))
            .on_click(cx, |this: &mut Self, cx| this.reset_shadow(cx))
            .spawn(cx);
        let bounds_toggle = toggle::new("workshop-shadow-bounds")
            .with_data(true)
            .content(|_, _| div().child(crate::l10n::ui("Show Bounds")))
            .spawn(cx);
        cx.subscribe(&bounds_toggle, |this: &mut Self, _, event: &ToggleEvent, cx| {
            if let ToggleEvent::Change { selected } = event {
                this.show_bounds = *selected;
                cx.notify();
            }
        })
        .detach();
        let color_field = crate::l10n::localize_field(
            textfield::TextFieldBuilder::new("workshop-shadow-color")
                .placeholder(default_shadow_color_placeholder(resolved_shadow_color()))
                .validator(Arc::new(|value| value.trim().is_empty() || parse_shadow_color_input(value).is_some()))
                .spawn(cx),
            cx,
        );
        cx.subscribe(&color_field, |this: &mut Self, _, event: &TextFieldEvent, cx| {
            if let TextFieldEvent::Change { value } | TextFieldEvent::Submit { value } = event {
                let parsed = parse_shadow_color_input(value);
                this.color_valid = value.trim().is_empty() || parsed.is_some();
                if this.color_valid {
                    this.color_override = parsed;
                    this.apply_template(cx);
                }
                cx.notify();
            }
        })
        .detach();
        let sliders = [
            (Field::X, "workshop-shadow-x", -20., 20.),
            (Field::Y, "workshop-shadow-y", -8., 24.),
            (Field::Blur, "workshop-shadow-blur", 0., 32.),
            (Field::Spread, "workshop-shadow-spread", -12., 20.),
            (Field::Opacity, "workshop-shadow-opacity", 0., 100.),
        ]
        .into_iter()
        .map(|(field, id, min, max)| {
            let slider = slider::SliderBuilder::new(id).range(min..max).step(1.).value(field.value(controls)).spawn(cx);
            cx.subscribe(&slider, move |this: &mut Self, _, event: &SliderEvent, cx| {
                if let SliderEvent::Change { value, .. } = event {
                    match field {
                        Field::X => this.controls.offset_x = *value,
                        Field::Y => this.controls.offset_y = *value,
                        Field::Blur => this.controls.blur_radius = *value,
                        Field::Spread => this.controls.spread_radius = *value,
                        Field::Opacity => this.controls.opacity = (*value / 100.).clamp(0., 1.),
                    }
                    this.apply_template(cx);
                    this.record(&format!("{}: {value:.0}", field.title()), cx);
                }
            })
            .detach();
            (field, slider)
        })
        .collect();
        let events = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "workshop-shadow-events",
                "Tune the copied prototype or activate its SDK button.",
            )
        });
        Self {
            look,
            demo,
            reset,
            bounds_toggle,
            color_field,
            sliders,
            controls,
            color_override: None,
            color_valid: true,
            show_bounds: true,
            clicks: 0,
            events,
        }
    }
    fn apply_template(&mut self, cx: &mut Context<Self>) {
        let template = prototype_shadow_button_template(self.controls, self.color_override);
        self.demo.update(cx, |v, cx| v.set_template(template, cx));
        cx.notify();
    }
    fn reset_shadow(&mut self, cx: &mut Context<Self>) {
        self.controls = ShadowButtonControls::default();
        self.color_override = None;
        self.color_valid = true;
        self.color_field.update(cx, |v, cx| v.set_value("", cx));
        for (field, slider) in &self.sliders {
            slider.update(cx, |v, cx| v.set_value(field.value(self.controls), cx));
        }
        self.apply_template(cx);
        self.record("Shadow reset", cx);
    }
    fn record(&mut self, line: &str, cx: &mut Context<Self>) {
        self.events.update(cx, |events, cx| events.append_line(line, cx));
        cx.notify();
    }
    pub(super) fn sync_look(&mut self, look: Arc<Look>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.events.update(cx, |events, cx| events.sync_look(look, cx));
        cx.notify();
    }
}
impl Render for ShadowButtonSample {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let mut spec = self.controls.base_spec();
        if let Some(color) = self.color_override {
            spec.color = color;
        }
        let chrome = self.look.chrome();
        let tuning = div()
            .flex()
            .flex_col()
            .gap(px(14.))
            .child(self.reset.clone())
            .child(self.bounds_toggle.clone())
            .child(crate::l10n::ui(format!(
                "Shadow color: {} (RGB, rgb(...), or #hex)",
                format_rgb_triplet(spec.color)
            )))
            .child(self.color_field.clone())
            .child(div().text_color(chrome.muted_text).child(if self.color_valid {
                "Empty uses the SDK-default neutral shadow color."
            } else {
                "Invalid color. Use 255, 255, 255, rgb(255,255,255), or #ffffff."
            }))
            .children(self.sliders.iter().map(|(field, slider)| {
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(crate::l10n::ui(format!("{}: {:.0}", field.title(), field.value(self.controls))))
                    .child(slider.clone())
            }));
        page("workshop-shadow-page", div().w_full().flex().flex_col().items_start().gap(px(28.))
            .child(card(&self.look, "workshop-shadow-example", "Shadow Button Prototype", "Copied from Studio. A real SDK Button owns interaction; the local prototype template projects its shadow for hover, press, focus, and disabled states.", 900.,
                div().flex().flex_col().items_start().gap(px(16.))
                    .child(div().debug_selector(|| "workshop-shadow-stage".into())
                            .bg(xgpui_luma::theme::ThemeTokens::default().palette.surface.panel.background).border_1().border_color(if self.show_bounds { chrome.border } else { chrome.border.opacity(0.) }).p(px(32.)).child(self.demo.clone()))
                    .child(crate::l10n::ui(format!("Clicks: {}", self.clicks)))
                    .child(div().font_family(crate::theme::MONO_FAMILY).typography_style(self.look.typography_scale(TextSize::Xs)).child(format_css_box_shadow(spec)))))
            .child(card(&self.look, "workshop-shadow-tuning", "Shadow Tuning", "Adjust the prototype's shadow geometry and color.", 700., tuning))
            .child(self.events.clone()))
    }
}

// Copied Studio color-input behavior, with checked UTF-8 slicing for arbitrary input.
fn parse_shadow_color_input(raw: &str) -> Option<Hsla> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(color) = parse_hex_color(trimmed) {
        return Some(color);
    }
    let source = if trimmed.get(..4).is_some_and(|s| s.eq_ignore_ascii_case("rgb(")) && trimmed.ends_with(')') {
        trimmed.get(4..trimmed.len() - 1)?
    } else {
        trimmed
    };
    let parts: Vec<&str> =
        source.split(|c: char| c == ',' || c.is_ascii_whitespace()).filter(|s| !s.is_empty()).collect();
    if parts.len() != 3 {
        return None;
    }
    Some(gpui::rgb(rgb_hex(parts[0].parse().ok()?, parts[1].parse().ok()?, parts[2].parse().ok()?)).into())
}
fn parse_hex_color(raw: &str) -> Option<Hsla> {
    let hex = raw.trim().trim_start_matches('#');
    let (r, g, b) = match hex.len() {
        3 => (
            u8::from_str_radix(&hex.get(0..1)?.repeat(2), 16).ok()?,
            u8::from_str_radix(&hex.get(1..2)?.repeat(2), 16).ok()?,
            u8::from_str_radix(&hex.get(2..3)?.repeat(2), 16).ok()?,
        ),
        6 => (
            u8::from_str_radix(hex.get(0..2)?, 16).ok()?,
            u8::from_str_radix(hex.get(2..4)?, 16).ok()?,
            u8::from_str_radix(hex.get(4..6)?, 16).ok()?,
        ),
        _ => return None,
    };
    Some(gpui::rgb(rgb_hex(r, g, b)).into())
}
fn rgb_hex(r: u8, g: u8, b: u8) -> u32 {
    (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{Focusable, TestAppContext};
    use crate::workshop::{Sample, Workshop};
    #[test]
    fn copied_prototype_uses_native_input_retains_tuning_and_resets() {
        let mut app = TestAppContext::single();
        app.update(|cx| {
            xgpui_luma::init(cx).unwrap();
            xgpui_luma::key_handling::bind_default_control_keys(cx);
        });
        let (workshop, cx) = app.add_window_view(|window, cx| {
            window.activate_window();
            Workshop::new(crate::theme::default_look(), cx)
        });
        workshop.update(cx, |v, cx| v.select("shadow-button", cx));
        cx.run_until_parked();
        let sample = cx.update(|window, cx| {
            let Sample::ShadowButton(sample) = &workshop.read(cx).samples["shadow-button"] else {
                panic!("missing prototype")
            };
            let sample = sample.clone();
            let focus = sample.read(cx).demo.read(cx).focus_handle(cx);
            focus.focus(window, cx);
            sample
        });
        cx.run_until_parked();
        cx.simulate_keystrokes("space");
        cx.run_until_parked();
        cx.update(|window, cx| {
            assert_eq!(sample.read(cx).clicks, 1);
            let focus = sample.read(cx).sliders[0].1.read(cx).focus_handle(cx);
            focus.focus(window, cx);
        });
        cx.run_until_parked();
        cx.simulate_keystrokes("end");
        cx.run_until_parked();
        cx.update(|window, cx| {
            assert_eq!(sample.read(cx).controls.offset_x, 20.);
            let focus = sample.read(cx).color_field.read(cx).focus_handle(cx);
            focus.focus(window, cx);
        });
        cx.run_until_parked();
        cx.simulate_input("#f00");
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert!(sample.read(cx).color_valid);
            assert_eq!(format_rgb_triplet(sample.read(cx).color_override.unwrap()), "255, 0, 0");
        });
        cx.simulate_keystrokes("cmd-a");
        cx.simulate_input("彩色");
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert!(!sample.read(cx).color_valid);
            assert_eq!(format_rgb_triplet(sample.read(cx).color_override.unwrap()), "255, 0, 0");
        });
        workshop.update(cx, |v, cx| {
            v.select("button", cx);
            v.sync_theme(xgpui_luma::theme::ThemeMode::Light, cx);
            v.select("shadow-button", cx);
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            let Sample::ShadowButton(retained) = &workshop.read(cx).samples["shadow-button"] else {
                unreachable!()
            };
            assert_eq!(retained, &sample);
            assert_eq!(sample.read(cx).controls.offset_x, 20.);
            assert_eq!(sample.read(cx).clicks, 1);
            let focus = sample.read(cx).reset.read(cx).focus_handle(cx);
            focus.focus(window, cx);
        });
        cx.run_until_parked();
        cx.simulate_keystrokes("space");
        cx.run_until_parked();
        cx.update(|_, cx| {
            let sample = sample.read(cx);
            assert_eq!(sample.controls.offset_x, 0.);
            assert_eq!(sample.controls.offset_y, 5.);
            assert!(sample.color_valid);
            assert!(sample.color_override.is_none());
            assert_eq!(sample.color_field.read(cx).value(), "");
            for (field, slider) in &sample.sliders {
                assert_eq!(slider.read(cx).value(), field.value(ShadowButtonControls::default()));
            }
        });
    }
    #[test]
    fn copied_color_input_accepts_supported_formats_and_rejects_unicode_safely() {
        for input in ["#f00", "#ff0000", "255, 0, 0", "rgb(255,0,0)"] {
            let color = parse_shadow_color_input(input).unwrap();
            assert_eq!(format_rgb_triplet(color), "255, 0, 0");
        }
        for input in ["彩色", "🎨red", "rgb(彩色)", "256, 0, 0", "#zzzzzz"] {
            assert!(parse_shadow_color_input(input).is_none());
        }
    }
}
