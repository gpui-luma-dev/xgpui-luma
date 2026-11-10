//! Color-lab reference comparisons and xgpui-color CPU API exploration.
//! The four tabs are `LabScreen`s (see `views.rs`) that share this page's state and controls.
mod display;
mod views;

use std::sync::Arc;
use gpui::{Context, Entity, SharedString};
use xgpui_luma::{
    controls::{button::Button, slider::Slider, textfield::TextField},
    prelude::HasPresenter,
};
use xgpui_luma_look_radix::{self as radix, Look};
use xgpui_color::{
    ColorInterpolation, EncodedRgb, GamutMapping, HueInterpolation, LinearGradient, LinearRgb, LinearRgba, Oklch,
    RgbSpace,
};
use crate::gamut::reference;

const SPACES: [RgbSpace; 3] = [RgbSpace::Srgb, RgbSpace::DisplayP3, RgbSpace::Rec2020];
const HUES: [HueInterpolation; 4] = [
    HueInterpolation::Shorter,
    HueInterpolation::Longer,
    HueInterpolation::Increasing,
    HueInterpolation::Decreasing,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Source,
    Destination,
    Preset,
    Mapping,
    Hue,
    Model,
    Samples,
    Reset,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Parameter {
    Red,
    Green,
    Blue,
    Position,
    /// OkLCh chroma scale applied to the input on Gamut Mapping.
    Saturation,
}
const PARAMETERS: [Parameter; 5] =
    [Parameter::Red, Parameter::Green, Parameter::Blue, Parameter::Position, Parameter::Saturation];

impl Parameter {
    fn config(self) -> (&'static str, f32, f32, f32) {
        match self {
            Self::Red => ("Red", 0., 1., 0.2),
            Self::Green => ("Green", 0., 1., 0.5),
            Self::Blue => ("Blue", 0., 1., 0.8),
            Self::Position => ("Position t", -0.25, 1.25, 0.5),
            Self::Saturation => ("Saturation", 0., 2., 1.),
        }
    }
    fn range_label(self) -> &'static str {
        match self {
            Self::Position => "−0.25–1.25",
            Self::Saturation => "×0–2",
            _ => "0–1",
        }
    }
    /// Fixed decimals per quantity: channels and `t` `0.000`, pixels `0`.
    fn format(self, value: f64) -> String {
        match self {
            Self::Saturation => format!("{value:.2}"),
            _ => format!("{value:.3}"),
        }
    }
    fn parse(self, text: &str) -> Option<f64> {
        let (_, min, max, _) = self.config();
        text.trim()
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite() && (min as f64..=max as f64).contains(value))
    }
}

/// Display name of an RGB space.
fn space_name(space: RgbSpace) -> &'static str {
    match space {
        RgbSpace::Srgb => "sRGB",
        RgbSpace::DisplayP3 => "Display P3",
        RgbSpace::Rec2020 => "Rec2020",
        _ => "RGB",
    }
}

fn mapping_name(policy: GamutMapping) -> &'static str {
    if policy == GamutMapping::Clip {
        "Clip"
    } else {
        "Reduce chroma"
    }
}

pub struct ColorPage {
    look: Arc<Look>,
    pub active_tab: SharedString,
    input: [f64; 3],
    source_index: usize,
    destination_index: usize,
    preset: usize,
    policy: GamutMapping,
    translucent: bool,
    hue_index: usize,
    model_index: usize,
    sample_count_index: usize,
    position: f64,
    /// Gradient axis end; the start is the origin. Samples follow the axis, so the bar does not
    /// depend on it, and the page keeps it fixed.
    axis: [f64; 2],
    /// OkLCh chroma scale for Gamut Mapping; 1 leaves the input unchanged.
    saturation: f64,
    buttons: Vec<(Action, Entity<Button>)>,
    choices: Vec<(Action, crate::lab_controls::Choice)>,
    sliders: Vec<(Parameter, Slider)>,
    fields: Vec<(Parameter, TextField)>,
    transparency: xgpui_luma::controls::switch::Switch,
    /// Draws colors as linear Display P3 on a wide-gamut window.
    painter: Result<Arc<xgpui_gamut::instanced::WidePainter>, String>,
}

impl ColorPage {
    fn init_choices(&mut self, cx: &mut Context<Self>) {
        self.choices = [
            Action::Source,
            Action::Destination,
            Action::Preset,
            Action::Mapping,
            Action::Hue,
            Action::Model,
            Action::Samples,
        ]
        .into_iter()
        .map(|action| {
            let (label, options, selected) = self.choice_options(action);
            let choice = crate::lab_controls::Choice::new(
                format!("color_lab-{action:?}"),
                label,
                &self.look,
                options,
                selected,
                cx,
                move |this: &mut Self, index, cx| this.select_choice(action, index, cx),
            );
            (action, choice)
        })
        .collect();
    }
    fn sync_choices(&self, cx: &mut gpui::App) {
        for (action, choice) in &self.choices {
            let (_, options, selected) = self.choice_options(*action);
            choice.sync(options, selected, cx);
        }
    }
    fn choice_options(&self, action: Action) -> (&'static str, Vec<String>, usize) {
        match action {
            Action::Source => ("Source", SPACES.iter().map(|s| space_name(*s).to_owned()).collect(), self.source_index),
            Action::Destination => {
                ("Destination", SPACES.iter().map(|s| space_name(*s).to_owned()).collect(), self.destination_index)
            }
            Action::Preset => ("Color pair", reference::PRESETS.iter().map(|s| s.to_string()).collect(), self.preset),
            Action::Mapping => (
                "Mapping",
                crate::lab_controls::labels(&["Clip", "Reduce chroma"]),
                usize::from(self.policy != GamutMapping::Clip),
            ),
            Action::Hue => ("Hue", HUES.iter().map(|s| format!("{s:?}")).collect(), self.hue_index),
            Action::Model => {
                ("Interpolation", crate::lab_controls::labels(&["Linear RGB", "OkLab", "OkLCh"]), self.model_index)
            }
            Action::Samples => ("Samples", crate::lab_controls::labels(&["32", "64", "128"]), self.sample_count_index),
            _ => unreachable!("action has no selector"),
        }
    }
    fn select_choice(&mut self, action: Action, index: usize, cx: &mut Context<Self>) {
        match action {
            Action::Source => {
                self.source_index = index;
            }
            Action::Destination => {
                self.destination_index = index;
            }
            Action::Preset => {
                self.preset = index;
            }
            Action::Mapping => {
                self.policy = [GamutMapping::Clip, GamutMapping::ReduceChroma][index];
            }
            Action::Hue => {
                self.hue_index = index;
            }
            Action::Model => {
                self.model_index = index;
            }
            Action::Samples => {
                self.sample_count_index = index;
            }
            _ => unreachable!("action has no selector"),
        }

        cx.notify();
    }

    pub fn new(look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let buttons = [(Action::Reset, "Reset input")]
            .map(|(action, label)| {
                (
                    action,
                    radix::Button::new(format!("color-lab-{action:?}"))
                        .look(look.as_ref())
                        .outline()
                        .label(label)
                        .on_click(cx, move |this: &mut Self, cx| this.action(action, cx))
                        .spawn(cx),
                )
            })
            .to_vec();
        let transparency = radix::Switch::new("color-lab-transparency")
            .look(look.as_ref())
            .label("Endpoint transparency")
            .on_value_changed(cx, |this: &mut Self, on, cx| {
                this.translucent = on;
                cx.notify();
            })
            .spawn(cx);
        let sliders = PARAMETERS
            .map(|parameter| {
                let (_, min, max, value) = parameter.config();
                (
                    parameter,
                    radix::Slider::new(format!("color-lab-{parameter:?}"))
                        .look(look.as_ref())
                        .size(radix::SliderSize::One)
                        .range(min..max)
                        .step(0.01)
                        .value(value as f64)
                        .on_value_changed(cx, move |this: &mut Self, _, value, cx| {
                            this.set_value(parameter, value as f64);
                            this.sync_field(parameter, cx);
                            cx.notify();
                        })
                        .spawn(cx),
                )
            })
            .to_vec();
        // Each value field is declared with its callbacks: a valid edit moves the value and its
        // slider; leaving the field rewrites it in the parameter's fixed format.
        let fields = PARAMETERS
            .map(|parameter| {
                let field = radix::TextField::new(format!("color-lab-field-{parameter:?}"))
                    .look(look.as_ref())
                    .surface()
                    .look_override(crate::screen::value_field_look(&look))
                    .full_width(true)
                    .value(parameter.format(parameter.config().3 as f64))
                    .select_all_on_tab_focus(true)
                    .validator(Arc::new(move |text| parameter.parse(text).is_some()))
                    .on_value_changed(cx, move |this: &mut Self, text, cx| {
                        if let Some(value) = parameter.parse(text) {
                            this.set_value(parameter, value);
                            if let Some((_, slider)) = this.sliders.iter().find(|(id, _)| *id == parameter) {
                                slider.update(cx, |slider, cx| slider.set_value(value as f32, cx));
                            }
                            cx.notify();
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
            .to_vec();
        let mut page = Self {
            look,
            active_tab: "Conversions".into(),
            input: [0.2, 0.5, 0.8],
            // Display P3 into sRGB: the source is wider than the destination, so mapping has work to do.
            source_index: 1,
            destination_index: 0,
            preset: 0,
            policy: GamutMapping::Clip,
            translucent: false,
            hue_index: 0,
            model_index: 1,
            sample_count_index: 0,
            position: 0.5,
            axis: [320., 0.],
            saturation: 1.,
            buttons,
            choices: Vec::new(),
            transparency,
            sliders,
            fields,
            painter: xgpui_gamut::instanced::WidePainter::new().map(Arc::new).map_err(|error| format!("{error:#}")),
        };
        page.init_choices(cx);
        page
    }

    fn action(&mut self, action: Action, cx: &mut Context<Self>) {
        match action {
            Action::Source
            | Action::Destination
            | Action::Preset
            | Action::Mapping
            | Action::Hue
            | Action::Model
            | Action::Samples => {
                unreachable!("selector setting")
            }
            Action::Reset => {
                self.input = [0.2, 0.5, 0.8];
                self.position = 0.5;
                self.axis = [320., 0.];
                self.saturation = 1.;
                for (parameter, slider) in &self.sliders {
                    slider.update(cx, |slider, cx| slider.set_value(parameter.config().3, cx));
                }
                for parameter in PARAMETERS {
                    self.sync_field(parameter, cx);
                }
            }
        }
        cx.notify();
    }

    fn value(&self, parameter: Parameter) -> f64 {
        match parameter {
            Parameter::Red => self.input[0],
            Parameter::Green => self.input[1],
            Parameter::Blue => self.input[2],
            Parameter::Position => self.position,
            Parameter::Saturation => self.saturation,
        }
    }

    fn set_value(&mut self, parameter: Parameter, value: f64) {
        match parameter {
            Parameter::Red => self.input[0] = value,
            Parameter::Green => self.input[1] = value,
            Parameter::Blue => self.input[2] = value,
            Parameter::Position => self.position = value,
            Parameter::Saturation => self.saturation = value,
        }
    }

    fn sync_field(&self, parameter: Parameter, cx: &mut gpui::App) {
        if let Some((_, field)) = self.fields.iter().find(|(id, _)| *id == parameter) {
            let text = parameter.format(self.value(parameter));
            field.update(cx, |field, cx| field.set_value(text, cx));
        }
    }

    fn input_color(&self) -> Result<LinearRgb, xgpui_color::ColorError> {
        EncodedRgb::new(SPACES[self.source_index], self.input)?.decode()
    }

    /// The input with its OkLCh chroma scaled by the Saturation parameter, in the source space.
    /// Scaling past 1 can push it outside the source gamut; that is the point of the control.
    fn saturated_input(&self) -> Result<LinearRgb, xgpui_color::ColorError> {
        let source = SPACES[self.source_index];
        let input = self.input_color()?;
        // The OkLab round trip is not bit-exact, so the neutral value returns the input itself.
        if self.saturation == 1. {
            return Ok(input);
        }
        let [l, c, h] = input.to_oklab()?.to_oklch()?.components();
        Oklch::new(l, c * self.saturation, h)?.to_oklab()?.to_linear(source)
    }

    fn model(&self) -> ColorInterpolation {
        match self.model_index {
            0 => ColorInterpolation::LinearRgb,
            1 => ColorInterpolation::Oklab,
            _ => ColorInterpolation::Oklch(HUES[self.hue_index]),
        }
    }

    fn endpoints(&self) -> Result<[LinearRgba; 2], xgpui_color::ColorError> {
        let (a, b) = match self.preset {
            0 => ([0., 0., 1.], [1., 1., 0.]),
            1 => ([1., 0., 0.], [0., 1., 0.]),
            _ => ([1., 0.18, 0.], [0.45, 0., 1.]),
        };
        Ok([
            LinearRgba::new(LinearRgb::new(RgbSpace::DisplayP3, a)?, 1.)?,
            LinearRgba::new(LinearRgb::new(RgbSpace::DisplayP3, b)?, if self.translucent { 0.25 } else { 1. })?,
        ])
    }

    fn gradient(&self, model: ColorInterpolation) -> Result<LinearGradient, xgpui_color::ColorError> {
        LinearGradient::new([0., 0.], self.axis, self.endpoints()?, SPACES[self.destination_index], model)
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::{Action, ColorPage, Parameter, SPACES};
    use gpui::TestAppContext;

    #[test]
    fn color_tabs_render_and_luma_controls_retain_independent_lab_state() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).expect("initialize SDK"));
        let (page, cx) = app.add_window_view(|_, cx| ColorPage::new(crate::theme::default_look(), cx));
        for tab in ["Conversions", "Interpolation", "Gamut Mapping", "CPU Gradients"] {
            page.update(cx, |page, cx| {
                page.active_tab = tab.into();
                cx.notify();
            });
            cx.run_until_parked();
        }
        let (source, red, preset) = cx.update(|_, cx| {
            let page = page.read(cx);
            (
                page.choices.iter().find(|(action, _)| *action == Action::Source).unwrap().1.clone(),
                page.sliders.iter().find(|(parameter, _)| *parameter == Parameter::Red).unwrap().1.clone(),
                page.choices.iter().find(|(action, _)| *action == Action::Preset).unwrap().1.clone(),
            )
        });
        let transparency = cx.update(|_, cx| page.read(cx).transparency.clone());
        transparency.update(cx, |switch, cx| {
            switch.set_data(true, cx);
            cx.emit(xgpui_luma::controls::switch::SwitchEvent::Change { on: true });
        });
        cx.update(|_, cx| source.select(2, cx));
        red.update(cx, |slider, cx| {
            slider.set_thumb_position(slider.primary_thumb_id(), 0.9, cx);
        });
        cx.update(|_, cx| preset.select(1, cx));
        cx.run_until_parked();
        cx.update(|_, cx| {
            let page = page.read(cx);
            assert!(page.translucent);
            assert!(*page.transparency.read(cx).data());
            assert_eq!(page.source_index, 2);
            assert_eq!(page.preset, 1);
            assert!((page.input[0] - 0.9).abs() < 0.001);
            let roundtrip = page
                .input_color()
                .unwrap()
                .convert(SPACES[2])
                .unwrap()
                .convert(SPACES[page.source_index])
                .unwrap()
                .encode()
                .unwrap()
                .components();
            for (actual, expected) in roundtrip.into_iter().zip(page.input) {
                assert!((actual - expected).abs() < 1e-6);
            }
        });
    }

    #[test]
    fn cpu_gradient_projects_geometry_extends_endpoints_and_handles_coincident_axis() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).expect("initialize SDK"));
        let (page, cx) = app.add_window_view(|_, cx| ColorPage::new(crate::theme::default_look(), cx));
        page.update(cx, |page, _| {
            page.translucent = true;
            page.axis = [120., 80.];
            let gradient = page.gradient(page.model()).unwrap();
            let first = gradient.sample_at(0.).unwrap();
            let last = gradient.sample_at(1.).unwrap();
            assert_eq!(gradient.sample([-120., -80.]).unwrap(), first);
            assert_eq!(gradient.sample([240., 160.]).unwrap(), last);
            assert!((gradient.sample([60., 40.]).unwrap().alpha() - 0.625).abs() < 1e-12);
            for count in 0..3 {
                page.sample_count_index = count;
                assert!(page.cpu_samples().is_ok(), "all CPU sample counts must render");
            }
            page.axis = [0., 0.];
            assert_eq!(page.gradient(page.model()).unwrap().sample([0., 0.]).unwrap(), last);
        });
    }
}

#[cfg(all(test, feature = "test-support"))]
mod saturation_tests {
    use super::ColorPage;
    use gpui::TestAppContext;

    #[test]
    fn saturation_pushes_a_p3_input_out_of_srgb_and_one_leaves_it_unchanged() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).expect("initialize SDK"));
        let (page, cx) = app.add_window_view(|_, cx| ColorPage::new(crate::theme::default_look(), cx));
        page.update(cx, |page, _| {
            let input = page.input_color().unwrap().components();
            let neutral = page.saturated_input().unwrap().components();
            for (a, b) in input.into_iter().zip(neutral) {
                assert_eq!(a, b, "saturation 1 is the input");
            }
            page.saturation = 2.;
            let pushed = page.saturated_input().unwrap().convert(xgpui_color::RgbSpace::Srgb).unwrap();
            assert!(!pushed.is_in_gamut(), "doubling chroma leaves sRGB");
        });
    }
}
