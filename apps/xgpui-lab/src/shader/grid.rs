//! App-only import of Christian Helgeson's Grid Shader (BSD-3-Clause).
//! Copyright 2019 WebGPU Samples Contributors; see licenses/ShaderDepot-BSD-3-Clause.txt.
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
pub(crate) struct GridInstance {
    pub grid_dimensions: f32,
    pub cell_origin_x: f32,
    pub cell_origin_y: f32,
    pub line_width: f32,
}
impl Default for GridInstance {
    fn default() -> Self {
        Self { grid_dimensions: 10., cell_origin_x: 0., cell_origin_y: 0., line_width: 1. }
    }
}
impl ShaderInstance for GridInstance {
    const FIELDS: &'static [InstanceField] = &[
        InstanceField { name: "grid_dimensions", offset: std::mem::offset_of!(Self, grid_dimensions), components: 1 },
        InstanceField { name: "cell_origin_x", offset: std::mem::offset_of!(Self, cell_origin_x), components: 1 },
        InstanceField { name: "cell_origin_y", offset: std::mem::offset_of!(Self, cell_origin_y), components: 1 },
        InstanceField { name: "line_width", offset: std::mem::offset_of!(Self, line_width), components: 1 },
    ];
}
pub(crate) struct GridPrograms {
    pub shader_paint: shader_paint::ShaderProgram,
    pub instanced: ShaderProgram<GridInstance>,
}
impl GridPrograms {
    pub fn new() -> anyhow::Result<Self> {
        let common = include_str!("grid.wgsl");
        Ok(Self {
            shader_paint: shader_paint::ShaderProgram::new(&format!(
                "{common}\nfn paint(input: PaintInput) -> vec4<f32> {{ return grid_color(input.uv, parameters); }}"
            ))?,
            instanced: ShaderProgram::new(&format!(
                "{common}\nfn shade(input: ShaderInput, p: Parameters) -> vec4<f32> {{ return grid_color(input.uv, p); }}"
            ))?,
        })
    }
    pub fn paint(&self, p: GridInstance) -> anyhow::Result<shader_paint::ShaderPaint> {
        self.shader_paint.paint(
            shader_paint::ShaderParameters::new()
                .float("grid_dimensions", p.grid_dimensions)
                .float("cell_origin_x", p.cell_origin_x)
                .float("cell_origin_y", p.cell_origin_y)
                .float("line_width", p.line_width),
        )
    }
}

#[derive(Clone, Copy)]
struct Settings {
    grid: GridInstance,
    pulse_width: f32,
    pulse_speed: f32,
    pulse: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self { grid: GridInstance::default(), pulse_width: 2., pulse_speed: 10., pulse: true }
    }
}
impl Settings {
    fn frame(self, time: f32) -> GridInstance {
        let mut grid = self.grid;
        if self.pulse {
            // Upstream main.ts and utils/interpolate.ts: cosine interpolation.
            let phase = time - time / 10. * (self.pulse_speed + 10.);
            let mix = (1. - (phase * std::f32::consts::PI).cos()) * 0.5;
            grid.line_width = grid.line_width * (1. - mix) + self.pulse_width * mix;
        }
        grid
    }
    fn values(self) -> [f32; 6] {
        [
            self.grid.grid_dimensions,
            self.grid.cell_origin_x,
            self.grid.cell_origin_y,
            self.grid.line_width,
            self.pulse_width,
            self.pulse_speed,
        ]
    }
    fn edit(&mut self, index: usize, value: f32) {
        match index {
            0 => self.grid.grid_dimensions = value.round(),
            1 => self.grid.cell_origin_x = value,
            2 => self.grid.cell_origin_y = value,
            3 => self.grid.line_width = value,
            4 => self.pulse_width = value,
            _ => self.pulse_speed = value,
        }
    }
}
const CONTROLS: [(&str, f32, f32, f32); 6] = [
    ("Grid dimensions", 1., 100., 1.),
    ("Cell origin X", -1., 1., 0.1),
    ("Cell origin Y", -1., 1., 0.1),
    ("Line width", 1., 8., 0.1),
    ("Pulse width", 2., 9., 0.1),
    ("Pulse speed", 0., 100., 10.),
];
pub struct GridPane {
    look: Arc<Look>,
    programs: anyhow::Result<GridPrograms>,
    settings: Settings,
    sliders: Vec<Slider>,
    numbers: Vec<Number>,
    pulse_switch: Switch,
    buttons: Vec<Entity<Button>>,
    active: bool,
    time: f32,
    last: Instant,
}
impl GridPane {
    pub fn new(look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let sliders = CONTROLS
            .iter()
            .enumerate()
            .map(|(i, (_, min, max, step))| {
                radix::Slider::new(format!("grid-{i}"))
                    .look(look.as_ref())
                    .range(*min..*max)
                    .step(*step)
                    .on_value_changed(cx, move |this: &mut Self, _, value, cx| {
                        this.settings.edit(i, value);
                        cx.notify();
                    })
                    .spawn(cx)
            })
            .collect();
        let buttons = ["Toggle pulse", "Reset", "Original source · Christian Helgeson"]
            .iter()
            .enumerate()
            .map(|(i, label)| {
                radix::Button::new(format!("grid-action-{i}"))
                    .look(look.as_ref())
                    .outline()
                    .label(*label)
                    .on_click(cx, move |this: &mut Self, cx| {
                        match i {
                            0 => {
                                this.settings.pulse = !this.settings.pulse;
                                this.last = Instant::now();
                            }
                            1 => {
                                this.settings = Settings::default();
                                this.time = 0.;
                                this.sync(cx);
                            }
                            _ => cx.open_url("https://github.com/cmhhelgeson/WGSL_Shader_Depot"),
                        }
                        cx.notify();
                    })
                    .spawn(cx)
            })
            .collect();
        let pulse_switch = radix::Switch::new("grid-pulse")
            .look(&look)
            .label("Pulse width")
            .checked(true)
            .on_value_changed(cx, |this: &mut Self, on, cx| {
                this.settings.pulse = on;
                this.last = Instant::now();
                cx.notify();
            })
            .spawn(cx);
        let mut pane = Self {
            look: look.clone(),
            numbers: Vec::new(),
            pulse_switch,
            programs: GridPrograms::new(),
            settings: Settings::default(),
            sliders,
            buttons,
            active: false,
            time: 0.,
            last: Instant::now(),
        };
        pane.numbers = pane
            .sliders
            .iter()
            .enumerate()
            .map(|(i, slider)| {
                let (label, min, max, step) = CONTROLS[i];
                Number::attach(
                    &format!("grid-{i}"),
                    label,
                    (min, max, step),
                    pane.settings.values()[i],
                    slider.clone(),
                    &look,
                    cx,
                    move |this: &mut Self, value, _| {
                        this.settings.edit(i, value);
                        this.settings.values()[i]
                    },
                )
            })
            .collect();
        pane.sync(cx);
        pane
    }
    fn sync(&self, cx: &mut Context<Self>) {
        for (slider, value) in self.sliders.iter().zip(self.settings.values()) {
            slider.update(cx, |slider, cx| slider.set_value(value, cx));
        }
        for (number, value) in self.numbers.iter().zip(self.settings.values()) {
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
        self.settings.pulse = false;
    }
    fn preview(&self, p: GridInstance) -> anyhow::Result<Hero> {
        let programs = self.programs.as_ref().map_err(|e| anyhow::anyhow!("{e:#}"))?;
        let paint = programs.paint(p)?;
        let typed = programs.instanced.clone();
        Ok(Hero::compare(
            ComparePane::new("Shader paint", painted().w_full().h(px(280.)).bg_paint(paint))
                .subtitle("Declarative paint"),
            ComparePane::new(
                "Instanced shader",
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, _| {
                        if let Err(e) = window.paint_custom_shader(bounds, &typed, p) {
                            tracing::error!("Grid: {e:#}");
                        }
                    },
                )
                .w_full()
                .h(px(280.)),
            )
            .subtitle("Typed main-pass instance"),
        ))
    }
}
impl Render for GridPane {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let now = Instant::now();
        if cx.reduce_motion() {
            self.settings.pulse = false;
        }
        if self.active && self.settings.pulse {
            self.time += now.duration_since(self.last).as_secs_f32().min(0.1);
            window.request_animation_frame();
        }
        self.last = now;
        for (number, value) in self.numbers.iter().zip(self.settings.values()) {
            number.observe(value, cx);
        }
        if *self.pulse_switch.read(cx).data() != self.settings.pulse {
            let pulse = self.settings.pulse;
            self.pulse_switch.update(cx, |switch, cx| switch.set_data(pulse, cx));
        }
        let p = self.settings.frame(self.time);
        let screen = LabScreen::new("shader-grid", self.look.clone()).crumb("xgpui-shader", "Shader Depot: Grid").title("Shader Depot: Grid")
            .lede("Compare the same pulsing grid drawn through two shader interfaces.")
            .caption(format!("{} · effective line width {:.2} · {:.0} cells · 16-byte instance", if self.settings.pulse {"Pulsing"} else {"Static"}, p.line_width, p.grid_dimensions))
            .control(self.pulse_switch.clone()).action(self.buttons[1].clone()).action(self.buttons[2].clone())
            .panels(ParametersPanel::new("Grid parameters").params(self.numbers.iter().map(Number::param)),
                PanelShell::new("Rendering").child(properties(&self.look, [("Comparison", "Same grid parameters and phase"), ("Pulse", "Cosine interpolation of line width"), ("Instance", "Four scalar inputs / 16 bytes")])))
            .note("Pulse moves between Line width and Pulse width. Editing the grid updates both previews; Reset restores the original settings. Leaving the page stops the pulse.")
            .credits("Adapted from Christian Helgeson's WGSL Shader Depot. Copyright 2019 WebGPU Samples Contributors; BSD-3-Clause, licenses/ShaderDepot-BSD-3-Clause.txt. No GPU timing is measured.");
        match self.preview(p) {
            Ok(hero) => screen.hero(hero),
            Err(e) => screen.notice(Some(format!("Grid shader error: {e:#}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[::core::prelude::v1::test]
    fn public_apis_accept_grid_and_validate_layout_and_parameters() {
        assert_eq!(std::mem::size_of::<GridInstance>(), 16);
        assert_eq!(GridInstance::FIELDS.iter().map(|f| f.offset).collect::<Vec<_>>(), [0, 4, 8, 12]);
        let programs = GridPrograms::new().unwrap();
        programs.paint(GridInstance::default()).unwrap();
        assert!(programs.shader_paint.paint(shader_paint::ShaderParameters::new()).is_err());
        assert!(programs.paint(GridInstance { line_width: f32::NAN, ..Default::default() }).is_err());
        let wrong = include_str!("grid.wgsl").replace("grid_dimensions: f32", "dimensions: f32");
        assert!(
            ShaderProgram::<GridInstance>::new(&format!(
                "{wrong}\nfn shade(i: ShaderInput, p: Parameters) -> vec4<f32> {{ return grid_color(i.uv, p); }}"
            ))
            .is_err()
        );
    }
    #[::core::prelude::v1::test]
    fn pulse_matches_upstream_endpoints_and_pause() {
        let mut s = Settings::default();
        assert_eq!(s.frame(0.).line_width, 1.);
        assert_eq!(s.frame(1.).line_width, 2.);
        s.pulse = false;
        assert_eq!(s.frame(1.).line_width, 1.);
        s.edit(0, 23.4);
        assert_eq!(s.grid.grid_dimensions, 23.);
    }
    #[cfg(feature = "test-support")]
    #[::core::prelude::v1::test]
    fn controls_edit_reset_and_suspend_retained_grid_state() {
        use xgpui_luma::controls::button::ButtonEvent;
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (pane, cx) = app.add_window_view(|_, cx| GridPane::new(crate::theme::default_look(), cx));
        cx.run_until_parked();
        let slider = cx.update(|_, cx| pane.read(cx).sliders[0].clone());
        slider.update(cx, |slider, cx| slider.set_thumb_position(slider.primary_thumb_id(), 0.5, cx));
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(pane.read(cx).settings.grid.grid_dimensions, 51.));
        let toggle = cx.update(|_, cx| pane.read(cx).buttons[0].clone());
        toggle.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        cx.run_until_parked();
        cx.update(|_, cx| assert!(!pane.read(cx).settings.pulse));
        let reset = cx.update(|_, cx| pane.read(cx).buttons[1].clone());
        reset.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        cx.run_until_parked();
        pane.update(cx, |pane, _| {
            assert_eq!(pane.settings.values(), Settings::default().values());
            assert!(pane.settings.pulse);
            pane.activate();
            assert!(pane.active);
            pane.suspend();
            assert!(!pane.active);
            assert!(!pane.settings.pulse, "leaving stops pulse playback");
        });
    }
    #[cfg(feature = "test-support")]
    #[::core::prelude::v1::test]
    fn numeric_fields_round_grid_dimensions_and_reset_the_pulse_switch() {
        use xgpui_luma::controls::{textfield::TextFieldEvent, switch::SwitchEvent, button::ButtonEvent};
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (page, cx) = app.add_window_view(|_, cx| GridPane::new(crate::theme::default_look(), cx));
        let field = cx.update(|_, cx| page.read(cx).numbers[0].field.clone());
        field.update(cx, |field, cx| {
            field.set_value("12.8", cx);
            cx.emit(TextFieldEvent::Change { value: "12.8".into() });
        });
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(page.read(cx).settings.grid.grid_dimensions, 13.));
        field.update(cx, |_, cx| cx.emit(TextFieldEvent::Submit { value: "12.8".into() }));
        cx.update(|_, cx| assert_eq!(field.read(cx).value(), "13.0"));
        let pulse = cx.update(|_, cx| page.read(cx).pulse_switch.clone());
        pulse.update(cx, |switch, cx| {
            switch.set_data(false, cx);
            cx.emit(SwitchEvent::Change { on: false });
        });
        let reset = cx.update(|_, cx| page.read(cx).buttons[1].clone());
        reset.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        cx.run_until_parked();
        cx.update(|_, cx| {
            assert_eq!(field.read(cx).value(), "10.0");
            assert!(*pulse.read(cx).data());
        });
    }
}
