//! LabScreen layouts for the retained analytic, coverage, WGSL and instancing examples.
use super::{Action, Parameter, ShaderPage};
use gpui::{AnyElement, Context, IntoElement, div, prelude::*, px};
use xgpui_luma::prelude::HasPresenter;
use xgpui_luma_look_radix as radix;
use crate::theme::ColorVizLookExt;
use crate::screen::{
    Hero, LabCard, LabScreen, PanelShell, Param, ParametersPanel, TextRole, labeled_control, properties, text,
    value_field_look,
};

const GRADIENT_PARAMETERS: [Parameter; 10] = [
    Parameter::Angle,
    Parameter::CenterX,
    Parameter::CenterY,
    Parameter::RadiusX,
    Parameter::RadiusY,
    Parameter::Offset,
    Parameter::Red,
    Parameter::Green,
    Parameter::Blue,
    Parameter::Alpha,
];
const WGSL_PARAMETERS: [Parameter; 2] = [Parameter::Progress, Parameter::ShaderAlpha];

impl Parameter {
    fn format(self, value: f32) -> String {
        if self == Self::Angle {
            format!("{value:.1}")
        } else {
            format!("{value:.3}")
        }
    }
    fn range_label(self) -> &'static str {
        match self {
            Self::Angle => "0–360°",
            Self::RadiusX | Self::RadiusY => "0.01–2",
            Self::Progress | Self::U | Self::V => "−1–1",
            _ => "0–1",
        }
    }
    fn parse(self, text: &str) -> Option<f32> {
        let (_, min, max) = self.config();
        text.trim().parse::<f32>().ok().filter(|value| value.is_finite() && (min..=max).contains(value))
    }
}

impl ShaderPage {
    pub(super) fn init_core_controls(&mut self, cx: &mut Context<Self>) {
        let look = self.look.clone();
        self.fields = GRADIENT_PARAMETERS
            .into_iter()
            .chain(WGSL_PARAMETERS)
            .map(|parameter| {
                let field = radix::TextField::new(format!("shader-field-{parameter:?}"))
                    .look(&look)
                    .surface()
                    .look_override(value_field_look(&look))
                    .full_width(true)
                    .value(parameter.format(self.value(parameter)))
                    .select_all_on_tab_focus(true)
                    .validator(std::sync::Arc::new(move |text| parameter.parse(text).is_some()))
                    .on_value_changed(cx, move |this: &mut Self, text, cx| {
                        if let Some(value) = parameter.parse(text) {
                            this.edit(parameter, value, cx);
                        }
                    })
                    .on_focus_changed(cx, move |this: &mut Self, focused, cx| {
                        if !focused {
                            this.sync_field(parameter, cx);
                        }
                    })
                    .on_submit(cx, move |this: &mut Self, _, cx| this.sync_field(parameter, cx))
                    .spawn(cx);
                (parameter, field)
            })
            .collect();
        self.core_switches = [
            (Action::Animate, "Animate"),
            (Action::Opacity, "50% opacity"),
            (Action::Clip, "Clip preview"),
            (Action::Native, "Native coverage"),
        ]
        .map(|(action, label)| {
            let switch = radix::Switch::new(format!("shader-switch-{action:?}"))
                .look(&look)
                .label(label)
                .checked(false)
                .on_value_changed(cx, move |this: &mut Self, on, cx| {
                    match action {
                        Action::Animate => {
                            this.animated = on;
                            this.last_frame = std::time::Instant::now();
                        }
                        Action::Opacity => this.half_opacity = on,
                        Action::Clip => this.clipped = on,
                        Action::Native => this.native = on,
                        _ => {}
                    }
                    cx.notify();
                })
                .spawn(cx);
            (action, switch)
        })
        .to_vec();
    }
    pub(super) fn sync_field(&self, parameter: Parameter, cx: &mut gpui::App) {
        if let Some((_, field)) = self.fields.iter().find(|(id, _)| *id == parameter) {
            let value = parameter.format(self.value(parameter));
            field.update(cx, |field, cx| field.set_value(value, cx));
        }
    }
    pub(super) fn sync_fields_except(&self, except: Option<Parameter>, cx: &mut gpui::App) {
        for (parameter, _) in &self.fields {
            if Some(*parameter) != except {
                self.sync_field(*parameter, cx);
            }
        }
    }
    fn sync_core_switches(&self, cx: &mut gpui::App) {
        for (action, switch) in &self.core_switches {
            let value = match action {
                Action::Animate => self.animated,
                Action::Opacity => self.half_opacity,
                Action::Clip => self.clipped,
                Action::Native => self.native,
                _ => false,
            };
            if *switch.read(cx).data() != value {
                switch.update(cx, |switch, cx| switch.set_data(value, cx));
            }
        }
    }
    fn core_parameters(&self, title: &'static str, parameters: &[Parameter]) -> ParametersPanel {
        ParametersPanel::new(title).params(
            parameters.iter().filter(|parameter| self.parameter_visible(**parameter)).filter_map(|parameter| {
                let field = self.fields.iter().find(|(id, _)| id == parameter)?.1.clone();
                let slider = self.sliders.iter().find(|(id, _)| id == parameter)?.1.clone();
                Some(Param::new(parameter.config().0, field, slider).range(parameter.range_label()))
            }),
        )
    }
    fn core_choices(&self, screen: LabScreen, actions: &[Action]) -> LabScreen {
        actions
            .iter()
            .filter_map(|action| self.choices.iter().find(|(id, _)| id == action))
            .fold(screen, |screen, (_, choice)| {
                screen.control(labeled_control(&self.look, choice.clone().into_element()))
            })
    }
    pub(super) fn render_core(&self, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_core_switches(cx);
        let gradient = matches!(self.active_tab.as_ref(), "Gradients" | "Gradient Paints");
        let (id, title, lede, hero) = match self.active_tab.as_ref() {
            "Gradient Paints" => (
                "shader-gradient-paints",
                "Gradient Paints",
                "Compare the same gradient filling a rounded surface, a border and a curved shape.",
                self.coverage(true).map(Hero::full),
            ),
            "WGSL Paints" => (
                "shader-wgsl",
                "WGSL Paints",
                "Compare two shader examples and explore how their parameters change the picture.",
                self.wgsl(),
            ),
            "Instancing" => (
                "shader-instancing",
                "Instancing",
                "Explore repeated glowing shapes, moving particles and overlapping layers.",
                self.instances().map(Hero::full),
            ),
            _ => (
                "shader-gradients",
                "Gradients",
                "Explore how gradient shapes, color stops and transparency change a smooth color field.",
                self.gradients().map(Hero::full),
            ),
        };
        let mut screen = LabScreen::new(id, self.look.clone())
            .crumb("xgpui-shader", title)
            .title(title)
            .lede(lede)
            .caption(self.core_caption());
        screen = match hero {
            Ok(hero) => screen.hero(hero),
            Err(error) => screen
                .hero(Hero::full(div().w_full().h(px(280.)).child(text(
                    &self.look,
                    TextRole::Caption,
                    "Preview unavailable",
                ))))
                .notice(Some(format!("Shader lab unavailable: {error:#}"))),
        };
        if gradient {
            screen = self.core_choices(screen, &[Action::Effect, Action::Stop]);
        }
        if self.active_tab == "Instancing" {
            screen = self.core_choices(screen, &[Action::Mode]);
            if self.mode == 1 {
                screen = self.core_choices(screen, &[Action::Count]);
            }
        }
        for (action, switch) in &self.core_switches {
            if (*action != Action::Native || gradient)
                && (*action != Action::Clip || self.active_tab != "Instancing" || self.mode == 2)
            {
                screen = screen.control(switch.clone());
            }
        }
        if gradient {
            for action in [Action::Add, Action::Duplicate, Action::Remove, Action::Reset] {
                if let Some((_, button)) = self.buttons.iter().find(|(id, _)| *id == action) {
                    screen = screen.action(button.clone());
                }
            }
            let settings = &self.settings[self.effect_index];
            let stop = settings.stops[settings.selected];
            let [r, g, b, a] = stop.color;
            screen = screen.panels(
                self.core_parameters("Gradient parameters", &GRADIENT_PARAMETERS).subtitle("Editing a value pauses the current animation."),
                PanelShell::new("Selected stop").child(properties(&self.look, [
                    ("Stop", format!("{} / {}", settings.selected + 1, settings.stops.len())),
                    ("Position", format!("{:.3}", stop.offset)), ("RGBA", format!("{r:.3}, {g:.3}, {b:.3}, {a:.3}")),
                    ("Paint", if self.native {"Native white coverage"} else {"Gradient producer"}.into()),
                ])),
            ).section_panel(PanelShell::new("Color stops").child(self.stop_strip()))
                .note("Choose a stop to edit its position and color. Add inserts an interpolated color in the largest gap; Duplicate preserves the selected color. Each gradient type keeps its own stops and geometry.")
                .credits("Adapted from xgpui paint-lab. Native coverage replaces the gradient with white to inspect the same geometry; switch it off to see color edits. Clipping shifts the preview inside its bounds. Stop values and rendering paths retain the source example's color interpretation.");
        } else if self.active_tab == "WGSL Paints" {
            screen = screen.panels(self.core_parameters("Shader parameters", &WGSL_PARAMETERS),
                PanelShell::new("Shader inputs").child(properties(&self.look, [
                    ("Wash", "Progress, fixed tint, alpha"), ("Typed fragment", "Color and effect instance"),
                    ("Coverage", "Rounded background / direct canvas"),
                ])))
                .note("Progress changes the wash; Alpha changes both examples. Animate changes the wash progress and the typed shader's phase. Opacity applies to each preview as a whole.")
                .credits("Portable wash adapted from xgpui paint-lab/portable.rs; typed fragment adapted from shader-examples/minimal.wgsl. The examples share controls but render different shader programs. No GPU timing is measured.");
        } else {
            let details = if self.mode == 2 {
                vec![
                    ("1", "Red shader".into()),
                    ("2", "Gold native quad".into()),
                    ("3", "Green shader".into()),
                    ("4", "Blue shader".into()),
                ]
            } else {
                vec![
                    (
                        "Pattern",
                        if self.mode == 0 {
                            "10×10 glow grid"
                        } else {
                            "Deterministic particle seeds"
                        }
                        .into(),
                    ),
                    ("Phase", format!("{:.2} s", self.time)),
                ]
            };
            screen = screen.cards([LabCard::new("Workload").subtitle(["Glow", "Particles", "Composition"][self.mode])
                .values([("instances", self.instance_count().to_string()), ("opacity", if self.half_opacity {"50%"} else {"100%"}.into())])])
                .panels(PanelShell::new(if self.mode == 2 {"Paint order"} else {"Animation"}).child(properties(&self.look, details)),
                    PanelShell::new("Rendering").child(properties(&self.look, [("Path", "Typed instances in the main pass"), ("Measurement", "Workload count only")])))
                .note(if self.mode == 2 {"The cyan frame marks the optional clip boundary. The gold native quad covers red; green and blue cover gold where they overlap."} else {"Glow uses one hundred cells; Particles uses the selected workload. Pausing retains the phase, and changing tabs stops animation."})
                .credits("Adapted from xgpui shader-examples. Instance counts describe the workload; they are not GPU submission counts or timings. Use a GPU capture to measure rendering cost.");
        }
        screen
    }
    fn core_caption(&self) -> String {
        let state = if self.animated { "Animating" } else { "Paused" };
        let opacity = if self.half_opacity { "50%" } else { "100%" };
        match self.active_tab.as_ref() {
            "Instancing" => format!(
                "{} · {} instances · {state} · opacity {opacity}",
                ["Glow", "Particles", "Composition"][self.mode],
                self.instance_count()
            ),
            "WGSL Paints" => format!(
                "Wash progress {:.3} · alpha {:.3} · {state} · opacity {opacity}",
                self.progress + if self.animated { 0.25 * self.time.sin() } else { 0. },
                self.shader_alpha
            ),
            _ => {
                let settings = &self.settings[self.effect_index];
                format!(
                    "{} · stop {} of {} · {} · {state} · opacity {opacity}",
                    Self::EFFECTS[self.effect_index].label(),
                    settings.selected + 1,
                    settings.stops.len(),
                    if self.native {
                        "native white coverage"
                    } else {
                        "gradient producer"
                    }
                )
            }
        }
    }
    pub(super) fn instance_count(&self) -> usize {
        match self.mode {
            0 => 100,
            1 => [100, 1000, 5000, 10000][self.count_index],
            _ => 3,
        }
    }
    fn stop_strip(&self) -> AnyElement {
        let chrome = self.look.chrome();
        let settings = &self.settings[self.effect_index];
        div()
            .flex()
            .flex_wrap()
            .gap_2()
            .children(settings.stops.iter().enumerate().map(|(index, stop)| {
                let [r, g, b, a] = stop.color;
                div()
                    .flex_1()
                    .min_w(px(40.))
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().h(px(28.)).rounded_sm().bg(gpui::Rgba { r, g, b, a }).border_2().border_color(
                        if index == settings.selected {
                            chrome.body_text
                        } else {
                            chrome.border
                        },
                    ))
                    .child(text(&self.look, TextRole::Value, format!("{:.3}", stop.offset)))
            }))
            .into_any_element()
    }
}
