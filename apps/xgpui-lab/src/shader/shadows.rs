use super::*;
use super::palette;
use crate::screen::Number;
use crate::screen::{ComparePane, Hero, LabScreen, PanelShell, ParametersPanel, TextRole, properties, text};
use gpui::{prelude::*, *};
use xgpui_shader::{
    instanced::{AnimatedBorder, AnimatedBorderStyle, ShadowPainter, ShadowStop, ShadowStyle},
    painted,
};

#[derive(Clone)]
pub struct ShadowDemo {
    painter: ShadowPainter,
    border: AnimatedBorder,
    pub blur: f32,
    pub spread: f32,
    pub falloff: f32,
    pub offset: f32,
    pub gradient: bool,
    pub inset: bool,
    pub card_dim: bool,
    pub clicks: usize,
}
impl ShadowDemo {
    pub fn new() -> Self {
        Self {
            border: AnimatedBorder::new().expect("border shader"),
            painter: ShadowPainter::new().expect("shadow shader"),
            blur: 24.,
            spread: 0.,
            falloff: 1.,
            offset: 12.,
            gradient: true,
            inset: false,
            card_dim: false,
            clicks: 0,
        }
    }
    pub fn border_button(&self, phase: f32, buttons: &[Entity<Button>]) -> impl IntoElement {
        div().child(
            painted()
                .rounded(px(999.))
                .bg(rgb(palette::BORDER_SURFACE))
                .animated_border(self.border.layer(AnimatedBorderStyle::default(), phase).expect("border frame"))
                .px_6()
                .py_3()
                .child(buttons[2].clone()),
        )
    }
    fn style(&self) -> ShadowStyle {
        ShadowStyle {
            stops: if self.gradient {
                palette::SHADOW_STOPS.into_iter().map(|(offset, color)| ShadowStop::new(offset, color)).collect()
            } else {
                vec![ShadowStop::new(0., palette::SHADOW_SOLID)]
            },
            angle: 90.,
            offset: point(px(self.offset * 0.35), px(self.offset)),
            blur_radius: px(self.blur),
            spread_radius: px(self.spread),
            falloff: self.falloff,
            inset: self.inset,
        }
    }
    /// Both cards are real styled elements; no canvas or manual shadow paint calls.
    pub fn render(&self, opacity: f32, buttons: &[Entity<Button>], look: &Look) -> Stateful<Div> {
        let native_color = if self.gradient {
            rgba(palette::SHADOW_NATIVE_GRADIENT)
        } else {
            rgba(palette::SHADOW_NATIVE_SOLID)
        };
        let native = BoxShadow {
            color: native_color.into(),
            offset: point(px(self.offset * 0.35), px(self.offset)),
            blur_radius: px(self.blur),
            spread_radius: px(self.spread),
            inset: self.inset,
        };
        let layer = self.painter.layer(self.style()).expect("validated demo shadow style");
        let card_opacity = if self.card_dim { 0.5 } else { 1. };
        let native_button = buttons[0].clone();
        let helper_button = buttons[1].clone();
        let native_card = div()
            .w_full()
            .max_w(px(240.))
            .h(px(160.))
            .rounded(px(16.))
            .bg(rgb(palette::SHADOW_CARD))
            .shadow(vec![native])
            .opacity(card_opacity)
            .overflow_hidden()
            .p_4()
            .flex()
            .flex_col()
            .justify_between()
            .text_color(look.chrome().body_text)
            .child(text(look, TextRole::Name, "Standard GPUI"))
            .child(text(look, TextRole::Caption, "A normal styled div"))
            .child(native_button);
        let helper_card = painted()
            .w_full()
            .max_w(px(240.))
            .h(px(160.))
            .rounded(px(16.))
            .bg(rgb(palette::SHADOW_CARD))
            .shadow_paint(layer)
            .opacity(card_opacity)
            .overflow_hidden()
            .p_4()
            .flex()
            .flex_col()
            .justify_between()
            .text_color(look.chrome().body_text)
            .child(text(look, TextRole::Name, "Extension helper"))
            .child(text(look, TextRole::Caption, "painted().shadow_paint(layer)"))
            .child(helper_button);
        div()
            .id("shadow-helper-demo")
            .w_full()
            .flex_1()
            .min_h(px(280.))
            .overflow_hidden()
            .bg(rgb(palette::SHADOW_STAGE))
            .opacity(opacity)
            .flex()
            .flex_wrap()
            .gap_y_8()
            .py_8()
            .items_center()
            .child(div().flex_1().min_w(px(240.)).px_6().flex().justify_center().child(native_card))
            .child(div().flex_1().min_w(px(240.)).px_6().flex().justify_center().child(helper_card))
    }
}

pub(super) struct ShadowPane {
    look: Arc<Look>,
    sample_look: Arc<Look>,
    shadow: ShadowDemo,
    numbers: Vec<Number>,
    switches: Vec<Switch>,
    pub borders: bool,
    pub animated: bool,
    phase: f32,
    last: std::time::Instant,
    buttons: Vec<Entity<Button>>,
    sliders: Vec<Slider>,
    last_clicks: usize,
}
impl ShadowPane {
    pub fn new(look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let light = Arc::new(xgpui_luma_look_radix::Look::built_in());
        light.set_mode(xgpui_luma::theme::ThemeMode::Light);
        let buttons = [
            "Native child",
            "Extension child",
            "Generate",
            "Animate / Pause",
            "Gradient / solid",
            "Inset / outset",
            "Card opacity",
        ]
        .into_iter()
        .enumerate()
        .map(|(id, label)| {
            radix::Button::new(format!("shadow-{id}"))
                .look(if id < 3 { light.as_ref() } else { look.as_ref() })
                .outline()
                .label(label)
                .on_click(cx, move |this: &mut Self, cx| {
                    match id {
                        0..=2 => this.shadow.clicks += 1,
                        3 => {
                            this.animated = !this.animated;
                            this.last = std::time::Instant::now();
                        }
                        4 => this.shadow.gradient = !this.shadow.gradient,
                        5 => this.shadow.inset = !this.shadow.inset,
                        _ => this.shadow.card_dim = !this.shadow.card_dim,
                    }
                    cx.notify();
                })
                .spawn(cx)
        })
        .collect();
        let sliders = [(0., 64., 24.), (-12., 24., 0.), (0.5, 4., 1.), (0., 24., 12.)]
            .into_iter()
            .enumerate()
            .map(|(id, (min, max, value))| {
                radix::Slider::new(format!("shadow-value-{id}"))
                    .look(look.as_ref())
                    .range(min..max)
                    .value(value)
                    .step(0.5)
                    .on_value_changed(cx, move |this: &mut Self, _, value, cx| {
                        match id {
                            0 => this.shadow.blur = value,
                            1 => this.shadow.spread = value,
                            2 => this.shadow.falloff = value,
                            _ => this.shadow.offset = value,
                        }
                        cx.notify();
                    })
                    .spawn(cx)
            })
            .collect();
        let switches = [("Animate", false), ("Gradient shadow", true), ("Inset", false), ("50% card opacity", false)]
            .into_iter()
            .enumerate()
            .map(|(i, (label, checked))| {
                radix::Switch::new(format!("shadow-switch-{i}"))
                    .look(&look)
                    .label(label)
                    .checked(checked)
                    .on_value_changed(cx, move |this: &mut Self, on, cx| {
                        match i {
                            0 => {
                                this.animated = on;
                                this.last = Instant::now();
                            }
                            1 => this.shadow.gradient = on,
                            2 => this.shadow.inset = on,
                            _ => this.shadow.card_dim = on,
                        }
                        cx.notify();
                    })
                    .spawn(cx)
            })
            .collect();
        let mut pane = Self {
            look: look.clone(),
            sample_look: light,
            numbers: Vec::new(),
            switches,
            shadow: ShadowDemo::new(),
            borders: false,
            animated: false,
            phase: 0.,
            last: std::time::Instant::now(),
            buttons,
            sliders,
            last_clicks: usize::MAX,
        };
        pane.numbers = pane
            .sliders
            .iter()
            .enumerate()
            .map(|(i, slider)| {
                let (label, min, max, value) = [
                    ("Blur (px)", 0., 64., 24.),
                    ("Spread (px)", -12., 24., 0.),
                    ("Falloff", 0.5, 4., 1.),
                    ("Offset (px)", 0., 24., 12.),
                ][i];
                Number::attach(
                    &format!("shadow-{i}"),
                    label,
                    (min, max, 0.5),
                    value,
                    slider.clone(),
                    &look,
                    cx,
                    move |this: &mut Self, value, _| {
                        match i {
                            0 => this.shadow.blur = value,
                            1 => this.shadow.spread = value,
                            2 => this.shadow.falloff = value,
                            _ => this.shadow.offset = value,
                        };
                        value
                    },
                )
            })
            .collect();
        pane
    }
}
impl Render for ShadowPane {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let now = std::time::Instant::now();
        if self.borders && self.animated && !cx.reduce_motion() {
            self.phase += (now - self.last).as_secs_f32().min(0.1);
            window.request_animation_frame();
        }
        self.last = now;
        if self.last_clicks != self.shadow.clicks {
            self.last_clicks = self.shadow.clicks;
            for button in self.buttons.iter().take(3) {
                button.update(cx, |button, cx| button.set_label(format!("Child clicks: {}", self.shadow.clicks), cx));
            }
        }
        if cx.reduce_motion() {
            self.animated = false;
        }
        for (number, value) in
            self.numbers
                .iter()
                .zip([self.shadow.blur, self.shadow.spread, self.shadow.falloff, self.shadow.offset])
        {
            number.observe(value, cx);
        }
        for (switch, on) in
            self.switches
                .iter()
                .zip([self.animated, self.shadow.gradient, self.shadow.inset, self.shadow.card_dim])
        {
            if *switch.read(cx).data() != on {
                switch.update(cx, |switch, cx| switch.set_data(on, cx));
            }
        }
        if self.borders {
            LabScreen::new("shader-borders", self.look.clone()).crumb("xgpui-shader", "Borders").title("Borders")
                .lede("Explore moving highlights around an ordinary interactive button.")
                .hero(Hero::stage(ComparePane::new("Animated border", div().w_full().min_h(px(280.)).p_8().flex().items_center().justify_center().bg(rgb(palette::BORDER_STAGE)).child(self.shadow.border_button(self.phase, &self.buttons)))))
                .caption(format!("{} · phase {:.2} · child clicks {}", if self.animated {"Animating"} else {"Paused"}, self.phase, self.shadow.clicks))
                .control(self.switches[0].clone())
                .panels(PanelShell::new("Motion").child(properties(&self.look, [("Period", "2.5 s"), ("Highlights", "Soft blue and orange"), ("Interaction", "Retained Luma button")])), PanelShell::new("Child interaction").child(properties(&self.look, [("Clicks", self.shadow.clicks.to_string())])))
                .note("Click the decorated button to verify child interaction. Pausing retains the phase; reduced motion freezes automatic animation.")
                .credits("Adapted from xgpui shader-examples/shadows.rs. The fixed light reference surface reveals the border signal in both themes. This page does not measure GPU time.")
        } else {
            let mut screen = LabScreen::new("shader-shadows", self.look.clone())
                .crumb("xgpui-shader", "Shadows")
                .title("Shadows")
                .lede("Compare a standard shadow with a gradient shadow around the same card.")
                .hero(Hero::full(self.shadow.render(1., &self.buttons, &self.sample_look)))
                .caption(format!(
                    "{} · {} · blur {:.1} px · spread {:.1} px · offset {:.1} px · card opacity {} · child clicks {}",
                    if self.shadow.gradient { "Gradient" } else { "Solid" },
                    if self.shadow.inset { "inset" } else { "outset" },
                    self.shadow.blur,
                    self.shadow.spread,
                    self.shadow.offset,
                    if self.shadow.card_dim { "50%" } else { "100%" },
                    self.shadow.clicks
                ));
            for switch in self.switches.iter().skip(1) {
                screen = screen.control(switch.clone());
            }
            screen.panels(ParametersPanel::new("Shadow parameters").params(self.numbers.iter().map(Number::param)),
                PanelShell::new("Comparison").child(properties(&self.look, [("Shared", "Blur, spread, offset and inset"), ("Extension", "Falloff and gradient stops"), ("Interaction", "Both child buttons are Luma controls")])))
                .note("Click either child button. Both cards receive the same geometric settings; extension falloff and gradient stops add capabilities beyond the native shadow.")
                .credits("Adapted from xgpui shader-examples/shadows.rs. Fixed light reference surfaces and shadow colors are preserved in both themes. Analytic softness differs from CSS blur; no GPU timing is measured.")
        }
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use xgpui_luma::controls::{button::ButtonEvent, switch::SwitchEvent, textfield::TextFieldEvent};
    #[::core::prelude::v1::test]
    fn shadow_numeric_controls_and_children_remain_live_in_both_views() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (page, cx) = app.add_window_view(|_, cx| ShadowPane::new(crate::theme::default_look(), cx));
        let blur = cx.update(|_, cx| page.read(cx).numbers[0].field.clone());
        blur.update(cx, |field, cx| {
            field.set_value("40", cx);
            cx.emit(TextFieldEvent::Change { value: "40".into() });
        });
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(page.read(cx).shadow.style().blur_radius, px(40.)));
        blur.update(cx, |field, cx| {
            field.set_value("1000", cx);
            cx.emit(TextFieldEvent::Change { value: "1000".into() });
        });
        cx.update(|_, cx| assert_eq!(page.read(cx).shadow.blur, 40.));
        for index in [0, 1] {
            let button = cx.update(|_, cx| page.read(cx).buttons[index].clone());
            button.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        }
        let gradient = cx.update(|_, cx| page.read(cx).switches[1].clone());
        gradient.update(cx, |switch, cx| {
            switch.set_data(false, cx);
            cx.emit(SwitchEvent::Change { on: false });
        });
        cx.run_until_parked();
        cx.update(|_, cx| {
            let page = page.read(cx);
            assert_eq!(page.shadow.clicks, 2);
            assert_eq!(page.shadow.style().stops.len(), 1);
        });
        page.update(cx, |page, cx| {
            page.borders = true;
            page.phase = 1.25;
            cx.notify();
        });
        let generate = cx.update(|_, cx| page.read(cx).buttons[2].clone());
        generate.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        cx.run_until_parked();
        assert!(cx.debug_bounds("shader-borders-hero").is_some());
        cx.update(|_, cx| {
            assert_eq!(page.read(cx).shadow.clicks, 3);
            assert_eq!(page.read(cx).phase, 1.25);
        });
    }
}
