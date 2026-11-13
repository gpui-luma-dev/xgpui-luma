//! App-side host for TDM's Seascape. The adapted WGSL retains CC BY-NC-SA 3.0.
use super::*;
use crate::screen::Number;
use crate::screen::{ComparePane, Hero, LabScreen, PanelShell, ParametersPanel, properties};
use xgpui_shader::{
    shader_paint,
    instanced::{InstanceField, ShaderInstance},
    painted, WindowShaderExt,
};

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct SeaInstance {
    pub time: f32,
    pub sea_height: f32,
    pub choppiness: f32,
    pub sea_speed: f32,
    pub frequency: f32,
    pub camera: f32,
    pub exposure: f32,
    pub detail: f32,
}
impl Default for SeaInstance {
    fn default() -> Self {
        Self {
            time: 0.,
            sea_height: 0.6,
            choppiness: 4.,
            sea_speed: 0.8,
            frequency: 0.16,
            camera: 0.,
            exposure: 1.,
            detail: 1.,
        }
    }
}
impl ShaderInstance for SeaInstance {
    const FIELDS: &'static [InstanceField] = &[
        InstanceField { name: "time", offset: std::mem::offset_of!(Self, time), components: 1 },
        InstanceField { name: "sea_height", offset: std::mem::offset_of!(Self, sea_height), components: 1 },
        InstanceField { name: "choppiness", offset: std::mem::offset_of!(Self, choppiness), components: 1 },
        InstanceField { name: "sea_speed", offset: std::mem::offset_of!(Self, sea_speed), components: 1 },
        InstanceField { name: "frequency", offset: std::mem::offset_of!(Self, frequency), components: 1 },
        InstanceField { name: "camera", offset: std::mem::offset_of!(Self, camera), components: 1 },
        InstanceField { name: "exposure", offset: std::mem::offset_of!(Self, exposure), components: 1 },
        InstanceField { name: "detail", offset: std::mem::offset_of!(Self, detail), components: 1 },
    ];
}
impl SeaInstance {
    pub fn values(self) -> [f32; 8] {
        [
            self.time,
            self.sea_height,
            self.choppiness,
            self.sea_speed,
            self.frequency,
            self.camera,
            self.exposure,
            self.detail,
        ]
    }
    fn edit(&mut self, index: usize, value: f32) {
        match index {
            0 => self.time = value,
            1 => self.sea_height = value,
            2 => self.choppiness = value,
            3 => self.sea_speed = value,
            4 => self.frequency = value,
            5 => self.camera = value,
            6 => self.exposure = value,
            _ => self.detail = value.round(),
        }
    }
}
/// Expand only the two marked, constant-bound loops. Each block retains its own scope.
fn expand_shader_paint_loops(mut source: String) -> String {
    for (marker, count) in [("OCTAVES", 5), ("TRACE", 32)] {
        let begin = format!("// BEGIN_{marker}");
        let end = format!("// END_{marker}");
        let start = source.find(&begin).expect("bounded loop marker");
        let finish = source.find(&end).expect("bounded loop end");
        let loop_source = &source[start + begin.len()..finish];
        let body = &loop_source[loop_source.find('{').unwrap() + 1..loop_source.rfind('}').unwrap()];
        let expanded = (0..count).map(|i| format!("{{ let iteration = {i}u; {body} }}\n")).collect::<String>();
        source.replace_range(start..finish + end.len(), &expanded);
    }
    source
}
pub(crate) struct SeaPrograms {
    pub shader_paint: shader_paint::ShaderProgram,
    pub instanced: ShaderProgram<SeaInstance>,
}
impl SeaPrograms {
    pub fn new() -> anyhow::Result<Self> {
        let source = include_str!("seascape.wgsl");
        Ok(Self {
            instanced: ShaderProgram::new(&format!(
                "{source}\nfn shade(i: ShaderInput, p: Parameters) -> vec4<f32> {{ return seascape_color(i.uv, i.size, p); }}"
            ))?,
            shader_paint: shader_paint::ShaderProgram::new(&format!(
                "{}\nfn paint(i: PaintInput) -> vec4<f32> {{ return seascape_color(i.uv, i.size, parameters); }}",
                expand_shader_paint_loops(source.into())
            ))?,
        })
    }
    pub fn paint(&self, p: SeaInstance) -> anyhow::Result<shader_paint::ShaderPaint> {
        let mut fields = shader_paint::ShaderParameters::new();
        for (field, value) in SeaInstance::FIELDS.iter().zip(p.values()) {
            fields = fields.float(field.name, value);
        }
        self.shader_paint.paint(fields)
    }
}
const CONTROLS: [(&str, f32, f32, f32); 8] = [
    ("Time (paused scrub)", 0., 60., 0.1),
    ("Wave height", 0.1, 1.5, 0.05),
    ("Choppiness", 0.5, 8., 0.1),
    ("Wave speed", 0., 2., 0.05),
    ("Frequency", 0.05, 0.4, 0.01),
    ("Camera phase", -3., 3., 0.05),
    ("Exposure", 0.5, 2., 0.05),
    ("Detailed normals (0 / 1)", 0., 1., 1.),
];
pub struct SeaPane {
    look: Arc<Look>,
    programs: anyhow::Result<SeaPrograms>,
    parameters: SeaInstance,
    sliders: Vec<Slider>,
    numbers: Vec<Number>,
    switches: Vec<Switch>,
    buttons: Vec<Entity<Button>>,
    active: bool,
    animated: bool,
    compare: bool,
    last: Instant,
}
impl SeaPane {
    pub fn new(look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let sliders = CONTROLS
            .iter()
            .enumerate()
            .map(|(i, (_, min, max, step))| {
                radix::Slider::new(format!("seascape-{i}"))
                    .look(look.as_ref())
                    .range(*min..*max)
                    .step(*step)
                    .on_value_changed(cx, move |this: &mut Self, _, value, cx| {
                        this.parameters.edit(i, value);
                        if i == 0 {
                            this.animated = false;
                        }
                        cx.notify();
                    })
                    .spawn(cx)
            })
            .collect();
        let buttons = ["Animate / Pause", "Compare rendering models", "Reset", "Original · TDM", "CC BY-NC-SA 3.0"]
            .iter()
            .enumerate()
            .map(|(i, label)| {
                radix::Button::new(format!("seascape-action-{i}"))
                    .look(look.as_ref())
                    .outline()
                    .label(*label)
                    .on_click(cx, move |this: &mut Self, cx| {
                        match i {
                            0 => {
                                this.animated = !this.animated;
                                this.last = Instant::now();
                                this.sync(cx);
                            }
                            1 => this.compare = !this.compare,
                            2 => {
                                this.parameters = SeaInstance::default();
                                this.animated = false;
                                this.last = Instant::now();
                                this.sync(cx);
                            }
                            3 => cx.open_url("https://www.shadertoy.com/view/Ms2SD1"),
                            _ => cx.open_url("https://creativecommons.org/licenses/by-nc-sa/3.0/"),
                        }
                        cx.notify();
                    })
                    .spawn(cx)
            })
            .collect();
        let switches = ["Animate", "Compare rendering models", "Detailed normals"]
            .into_iter()
            .enumerate()
            .map(|(i, label)| {
                radix::Switch::new(format!("seascape-switch-{i}"))
                    .look(&look)
                    .label(label)
                    .checked(i == 2)
                    .on_value_changed(cx, move |this: &mut Self, on, cx| {
                        match i {
                            0 => {
                                this.animated = on;
                                this.last = Instant::now();
                                this.sync(cx);
                            }
                            1 => this.compare = on,
                            _ => this.parameters.detail = if on { 1. } else { 0. },
                        }
                        cx.notify();
                    })
                    .spawn(cx)
            })
            .collect();
        let mut pane = Self {
            look: look.clone(),
            numbers: Vec::new(),
            switches,
            programs: SeaPrograms::new(),
            parameters: SeaInstance::default(),
            sliders,
            buttons,
            active: false,
            animated: false,
            compare: false,
            last: Instant::now(),
        };
        pane.numbers = pane
            .sliders
            .iter()
            .take(7)
            .enumerate()
            .map(|(i, slider)| {
                let (label, min, max, step) = CONTROLS[i];
                Number::attach(
                    &format!("seascape-{i}"),
                    label,
                    (min, max, step),
                    pane.parameters.values()[i],
                    slider.clone(),
                    &look,
                    cx,
                    move |this: &mut Self, value, _| {
                        this.parameters.edit(i, value);
                        if i == 0 {
                            this.animated = false;
                        }
                        this.parameters.values()[i]
                    },
                )
            })
            .collect();
        pane.sync(cx);
        pane
    }
    fn sync(&self, cx: &mut Context<Self>) {
        for (slider, value) in self.sliders.iter().zip(self.parameters.values()) {
            slider.update(cx, |slider, cx| slider.set_value(value, cx));
        }
        for (number, value) in self.numbers.iter().zip(self.parameters.values()) {
            number.sync(value, cx);
        }
    }
    pub fn activate(&mut self) {
        if !self.active {
            self.last = Instant::now();
        }
        self.active = true;
    }
    pub fn suspend(&mut self) {
        self.active = false;
        self.animated = false;
    }
    fn preview(&self) -> anyhow::Result<Hero> {
        let programs = self.programs.as_ref().map_err(|e| anyhow::anyhow!("{e:#}"))?;
        let typed = programs.instanced.clone();
        let p = self.parameters;
        let native = ComparePane::new(
            "Instanced shader",
            canvas(
                |_, _, _| (),
                move |bounds, _, window, _| {
                    if let Err(e) = window.paint_custom_shader(bounds, &typed, p) {
                        tracing::error!("Seascape: {e:#}");
                    }
                },
            )
            .w_full()
            .h(px(300.)),
        )
        .subtitle("Native WGSL loops · 32-byte instance");
        Ok(if self.compare {
            Hero::compare(
                ComparePane::new("Shader paint", painted().w_full().h(px(300.)).bg_paint(programs.paint(p)?))
                    .subtitle("Expanded bounded loops"),
                native,
            )
        } else {
            Hero::stage(native)
        })
    }
}
impl Render for SeaPane {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let now = Instant::now();
        if cx.reduce_motion() {
            self.animated = false;
        }
        if self.active && self.animated {
            self.parameters.time += now.duration_since(self.last).as_secs_f32().min(0.1);
            window.request_animation_frame();
        }
        self.last = now;
        for (number, value) in self.numbers.iter().zip(self.parameters.values()) {
            number.observe(value, cx);
        }
        for (switch, on) in self.switches.iter().zip([self.animated, self.compare, self.parameters.detail > 0.5]) {
            if *switch.read(cx).data() != on {
                switch.update(cx, |switch, cx| switch.set_data(on, cx));
            }
        }
        let mut screen = LabScreen::new("shader-seascape", self.look.clone())
            .crumb("xgpui-shader", "Shadertoy: Seascape")
            .title("Shadertoy: Seascape")
            .lede("Explore a moving ocean and compare two ways to draw the same waves.")
            .caption(format!(
                "{} · time {:.2} s · 32 trace steps · 3 geometry / {} normal octaves",
                if self.animated { "Animating" } else { "Paused" },
                self.parameters.time,
                if self.parameters.detail > 0.5 { 5 } else { 3 }
            ));
        for switch in &self.switches {
            screen = screen.control(switch.clone());
        }
        for button in self.buttons.iter().skip(2) {
            screen = screen.action(button.clone());
        }
        let screen = screen.panels(ParametersPanel::new("Ocean parameters").subtitle("Scrubbing time pauses animation.").params(self.numbers.iter().map(Number::param)),
            PanelShell::new("Rendering models").child(properties(&self.look, [("Instanced", "Native bounded WGSL loops"), ("Shader paint", "Expanded constant-bound loops"), ("Comparison", "Same time, camera and wave inputs")])))
            .note("Compare rendering models adds the shader-paint preview beside the typed shader. Detailed normals use five octaves instead of three. Leaving the page pauses motion.")
            .credits("Seascape © 2014 Alexander Alekseev / TDM. WGSL adaptation under CC BY-NC-SA 3.0: noncommercial, share alike. Original work and license are linked in the controls.");
        match self.preview() {
            Ok(hero) => screen.hero(hero),
            Err(e) => screen.notice(Some(format!("Seascape shader error: {e:#}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[::core::prelude::v1::test]
    fn public_apis_compile_looped_and_expanded_seascape() {
        assert_eq!(std::mem::size_of::<SeaInstance>(), 32);
        assert_eq!(SeaInstance::FIELDS.iter().map(|f| f.offset).collect::<Vec<_>>(), [0, 4, 8, 12, 16, 20, 24, 28]);
        let programs = SeaPrograms::new().unwrap();
        programs.paint(SeaInstance::default()).unwrap();
        assert!(programs.paint(SeaInstance { time: f32::INFINITY, ..Default::default() }).is_err());
        let source = include_str!("seascape.wgsl");
        let result = shader_paint::ShaderProgram::new(&format!(
            "{source}\nfn paint(i: PaintInput) -> vec4<f32> {{ return seascape_color(i.uv, i.size, parameters); }}"
        ));
        assert!(result.unwrap_err().to_string().contains("loops are not supported"));
        let expanded = expand_shader_paint_loops(source.into());
        assert!(!expanded.contains("for ("));
        assert_eq!(expanded.matches("state = trace_step(state, origin, direction, p);").count(), 32);
        assert_eq!(expanded.matches("let d = sea_octave").count(), 5);
    }
    #[cfg(feature = "test-support")]
    #[::core::prelude::v1::test]
    fn controls_scrub_reset_and_suspend_animation() {
        use xgpui_luma::controls::button::ButtonEvent;
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (pane, cx) = app.add_window_view(|_, cx| SeaPane::new(crate::theme::default_look(), cx));
        cx.run_until_parked();
        let slider = cx.update(|_, cx| pane.read(cx).sliders[0].clone());
        slider.update(cx, |slider, cx| slider.set_thumb_position(slider.primary_thumb_id(), 0.5, cx));
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert_eq!(pane.read(cx).parameters.time, 30.);
            assert!(!pane.read(cx).animated);
        });
        let compare = cx.update(|_, cx| pane.read(cx).buttons[1].clone());
        compare.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        cx.run_until_parked();
        cx.update(|_, cx| assert!(pane.read(cx).compare));
        let reset = cx.update(|_, cx| pane.read(cx).buttons[2].clone());
        reset.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        cx.run_until_parked();
        pane.update(cx, |pane, _| {
            assert_eq!(pane.parameters.values(), SeaInstance::default().values());
            pane.animated = true;
            pane.activate();
            pane.suspend();
            assert!(!pane.active);
            assert!(!pane.animated);
        });
    }
    #[cfg(feature = "test-support")]
    #[::core::prelude::v1::test]
    fn numeric_scrubbing_pauses_and_switches_follow_the_original_reset() {
        use xgpui_luma::controls::{textfield::TextFieldEvent, switch::SwitchEvent, button::ButtonEvent};
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (page, cx) = app.add_window_view(|_, cx| SeaPane::new(crate::theme::default_look(), cx));
        let (time, animate, compare, detail) = cx.update(|_, cx| {
            let page = page.read(cx);
            (
                page.numbers[0].field.clone(),
                page.switches[0].clone(),
                page.switches[1].clone(),
                page.switches[2].clone(),
            )
        });
        animate.update(cx, |switch, cx| {
            switch.set_data(true, cx);
            cx.emit(SwitchEvent::Change { on: true });
        });
        time.update(cx, |field, cx| {
            field.set_value("12.5", cx);
            cx.emit(TextFieldEvent::Change { value: "12.5".into() });
        });
        for switch in [&compare, &detail] {
            switch.update(cx, |switch, cx| {
                switch.set_data(false, cx);
                cx.emit(SwitchEvent::Change { on: false });
            });
        }
        compare.update(cx, |switch, cx| {
            switch.set_data(true, cx);
            cx.emit(SwitchEvent::Change { on: true });
        });
        cx.run_until_parked();
        cx.update(|_, cx| {
            let page = page.read(cx);
            assert_eq!(page.parameters.time, 12.5);
            assert!(!page.animated);
            assert!(!*animate.read(cx).data());
            assert!(page.compare);
            assert_eq!(page.parameters.detail, 0.);
        });
        let reset = cx.update(|_, cx| page.read(cx).buttons[2].clone());
        reset.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        cx.run_until_parked();
        cx.update(|_, cx| {
            let page = page.read(cx);
            assert_eq!(page.parameters.values(), SeaInstance::default().values());
            assert!(page.compare, "upstream reset preserves comparison mode");
            assert!(*detail.read(cx).data());
        });
    }
}
