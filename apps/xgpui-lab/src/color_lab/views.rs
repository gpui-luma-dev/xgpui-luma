//! The xgpui-color tabs (Conversions, Interpolation, Gamut Mapping, CPU Gradients), laid out with `LabScreen`.
use super::*;
use gpui::{AnyElement, Div, IntoElement, ParentElement, Render, Styled, Window, WindowColorMode, div, prelude::*, px, rgb};
use lucide_svg_static::Icon;
use xgpui_luma::{hstack, vstack};
use xgpui_luma_look_radix::{Badge, BadgeVariant};

use super::display::{Display, srgb_bar};

use crate::screen::{
    ComparePane, Hero, LabCard, LabScreen, PanelShell, Param, ParametersPanel, PreviewPanel, TextRole, data,
    labeled_control, properties, readout, text,
};
use crate::theme::ColorVizLookExt;

/// Height of a hero swatch and of a gradient bar.
const SWATCH_HEIGHT: f32 = 120.;
const BAR_HEIGHT: f32 = 48.;
const CHANNELS: [&str; 3] = ["R", "G", "B"];

const PREVIEW_CREDITS: &str = "On a wide-gamut window, swatches and the sampled bar are drawn in Display P3, and colors outside P3 are reduced in chroma for display only; on other windows they are sRGB previews. The numbers are xgpui-color's CPU reference math and keep extended values.";
const SRGB_REFERENCE_CREDITS: &str = "These reference rows map into sRGB, so they are drawn as sRGB on any window. The numbers are xgpui-color's CPU reference math.";
const INTERPOLATION_NOTE: &str = "32 CPU reference samples per row. The strip under each bar marks samples outside sRGB before mapping. Display paints interpolate between the mapped samples; color interpolation uses premultiplied components.";

/// Encoded sRGB with straight alpha, for reference rows that target sRGB.
fn preview(color: LinearRgb, policy: GamutMapping, alpha: f64) -> anyhow::Result<[f32; 4]> {
    let [r, g, b] = color.map_gamut(RgbSpace::Srgb, policy)?.encode()?.components();
    Ok([r as f32, g as f32, b as f32, alpha as f32])
}

/// A hero swatch with the values it shows underneath.
fn swatch_with(look: &Look, fill: AnyElement, values: [(&'static str, String); 3]) -> Div {
    vstack! {
        div().w_full().h(px(SWATCH_HEIGHT)).child(fill),
        readout(look, values),
    }
    .gap_3()
}

/// R, G and B keys paired with values in a fixed format.
fn channels(values: [f64; 3], decimals: usize) -> [(&'static str, String); 3] {
    [0, 1, 2].map(|index| (CHANNELS[index], format!("{:.decimals$}", values[index])))
}

/// Three values on one line for a properties list.
fn triple(values: [f64; 3]) -> String {
    format!("{:.4}  {:.4}  {:.4}", values[0], values[1], values[2])
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

/// A labeled gradient bar, with a strip marking samples outside sRGB before mapping.
fn bar(look: &Look, label: impl Into<SharedString>, display: AnyElement, outside: &[bool]) -> Div {
    let inside = look.chrome().border;
    let strip = hstack! {}.h(px(4.)).w_full().rounded_sm().overflow_hidden().children(outside.iter().map(|outside| {
        let color = if *outside {
            rgb(data::OUT_OF_GAMUT).into()
        } else {
            inside
        };
        div().flex_1().h_full().bg(color)
    }));
    vstack! { text(look, TextRole::Detail, label.into()), display, strip }.gap_2().w_full()
}

/// Marks position `t` on a bar that spans t 0–1: a 2 px foreground line over a 4 px background
/// halo, like the diagram markers, so it reads on any color. Positions outside 0–1 pin to the
/// nearer end, where the gradient holds its end color.
fn with_marker(look: &Look, display: AnyElement, t: f64) -> Div {
    let chrome = look.chrome();
    let marker = div()
        .absolute()
        .top(px(-4.))
        .bottom(px(-4.))
        .left(gpui::relative(t.clamp(0., 1.) as f32))
        .ml(px(-2.))
        .w(px(4.))
        .rounded_sm()
        .bg(chrome.content_background)
        .child(div().ml(px(1.)).w(px(2.)).h_full().bg(chrome.body_text));
    div().relative().w_full().child(display).child(marker)
}

/// A labeled bar of encoded sRGB reference samples.
fn reference_bar(
    look: &Look,
    label: impl Into<SharedString>,
    colors: &[[f32; 4]],
    outside: &[bool],
) -> anyhow::Result<Div> {
    Ok(bar(look, label, srgb_bar(colors, px(BAR_HEIGHT))?, outside))
}

impl Render for ColorPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_choices(cx);
        // Draw at the window's real output: linear P3 when it presents wide-gamut SDR.
        let wide = window.color_status().effective() == WindowColorMode::WideGamutSdr;
        let display = Display::new(self.painter.as_ref().ok().filter(|_| wide).cloned());
        let screen = self.base();
        let screen = match self.active_tab.as_ref() {
            "Conversions" => self.conversions(screen, &display),
            "Interpolation" => self.interpolation(screen),
            "Gamut Mapping" => self.mapping(screen, &display),
            _ => self.cpu_gradient_screen(screen, &display),
        }
        .map(|screen| {
            let error = self.painter.as_ref().err().filter(|_| wide);
            screen.notice(error.map(|error| format!("Wide-gamut painting unavailable, showing sRGB previews: {error}")))
        });
        // The color math failed: keep the tab's controls and say what happened.
        screen.unwrap_or_else(|error| self.base().notice(Some(format!("Color lab unavailable: {error:#}"))))
    }
}

impl ColorPage {
    /// Breadcrumb, title, lede and the tab's controls. Content comes from the tab's own builder.
    fn base(&self) -> LabScreen {
        let tab = self.active_tab.as_ref();
        let (id, lede, selectors, transparency, reset): (_, _, &[Action], _, _) = match tab {
            "Conversions" => (
                "color-conversions",
                "Explore how one color is written as numbers in different color systems: the numbers change, but the color you see stays the same.",
                &[Action::Source, Action::Destination],
                false,
                true,
            ),
            "Interpolation" => (
                "color-interpolation",
                "Compare ways of blending one color into another: the same two end colors can pass through gray, stay vivid, or travel around the color wheel.",
                &[Action::Preset, Action::Mapping, Action::Hue],
                true,
                false,
            ),
            "Gamut Mapping" => (
                "color-gamut-mapping",
                "Compare two ways to show a color that a screen cannot display: cut it off at the screen's limit, or make it a little less vivid until it fits.",
                &[Action::Source, Action::Destination],
                false,
                true,
            ),
            _ => (
                "color-cpu-gradients",
                "Explore how a smooth color blend is worked out point by point, and look up the exact color at any point along it.",
                &[Action::Destination, Action::Preset, Action::Mapping, Action::Model, Action::Hue, Action::Samples],
                true,
                true,
            ),
        };
        // The hue policy only applies to OkLCh on the gradient tab.
        let shown = |action: &&Action| !(**action == Action::Hue && tab == "CPU Gradients" && self.model_index != 2);
        let screen = selectors
            .iter()
            .filter(shown)
            .filter_map(|action| self.choices.iter().find(|(id, _)| id == action))
            .fold(
                LabScreen::new(id, self.look.clone()).crumb("xgpui-color", tab).title(tab.to_owned()).lede(lede),
                |screen, (_, choice)| screen.control(labeled_control(&self.look, choice.clone().into_element())),
            );
        let screen = if transparency {
            screen.control(self.transparency.clone())
        } else {
            screen
        };
        self.buttons
            .iter()
            .filter(|_| reset)
            .fold(screen, |screen, (_, button)| screen.action(button.clone()))
    }

    fn param(&self, parameter: Parameter) -> Option<Param> {
        let field = self.fields.iter().find(|(id, _)| *id == parameter)?.1.clone();
        let slider = self.sliders.iter().find(|(id, _)| *id == parameter)?.1.clone();
        Some(Param::new(parameter.config().0, field, slider).range(parameter.range_label()))
    }

    fn params<'a>(&'a self, parameters: &'a [Parameter]) -> impl Iterator<Item = Param> + 'a {
        parameters.iter().filter_map(|parameter| self.param(*parameter))
    }

    /// The encoded input channels, and on Gamut Mapping the saturation that pushes them outward.
    fn input_parameters(&self, saturation: bool) -> ParametersPanel {
        let source = space_name(SPACES[self.source_index]);
        let (subtitle, parameters): (_, &[Parameter]) = if saturation {
            (
                format!("{source} channels, then OkLCh chroma scaled by Saturation"),
                &[Parameter::Red, Parameter::Green, Parameter::Blue, Parameter::Saturation],
            )
        } else {
            (format!("{source} channels"), &[Parameter::Red, Parameter::Green, Parameter::Blue])
        };
        ParametersPanel::new("Encoded input").subtitle(subtitle).params(self.params(parameters))
    }

    /// `Soft` for a positive state, `Outline` otherwise.
    fn badge(&self, label: &'static str, positive: bool) -> Badge {
        Badge::new(&self.look, label).variant(if positive {
            BadgeVariant::Soft
        } else {
            BadgeVariant::Outline
        })
    }

    fn gamut_badge(&self, inside: bool) -> Badge {
        self.badge(if inside { "In gamut" } else { "Out of gamut" }, inside)
    }

    fn conversions(&self, screen: LabScreen, display: &Display) -> anyhow::Result<LabScreen> {
        let look = &self.look;
        let (source, destination) = (SPACES[self.source_index], SPACES[self.destination_index]);
        let linear = self.input_color()?;
        let converted = linear.convert(destination)?;
        let lab = linear.to_oklab()?;
        let lch = lab.to_oklch()?.components();
        let xyz = linear.to_xyz()?.components();
        let encoded = converted.encode()?.components();
        let roundtrip = converted.convert(source)?.encode()?.components();
        let error = roundtrip.into_iter().zip(self.input).map(|(a, b)| (a - b).abs()).fold(0., f64::max);
        let inside = converted.is_in_gamut();
        let source_pane = ComparePane::new(
            space_name(source),
            swatch_with(look, display.fill(linear, 1., GamutMapping::ReduceChroma, px(6.))?, channels(self.input, 3)),
        )
        .subtitle("Encoded input");
        let destination_pane = ComparePane::new(
            space_name(destination),
            swatch_with(look, display.fill(converted, 1., GamutMapping::ReduceChroma, px(6.))?, channels(encoded, 3)),
        )
        .subtitle("Encoded, unclipped")
        .badge(self.gamut_badge(inside));
        let card = |name: &'static str, subtitle: &'static str, keys: [&'static str; 3], values: [f64; 3]| {
            LabCard::new(name)
                .subtitle(subtitle)
                .values(keys.into_iter().zip(values).map(|(key, value)| (key, format!("{value:.4}"))))
        };
        let round_trip = PanelShell::new("Round trip")
            .subtitle(format!("{} back to {}", space_name(destination), space_name(source)))
            .child(properties(
                look,
                [
                    ("Linear", triple(converted.components())),
                    ("Encoded", triple(roundtrip)),
                    ("Max error", format!("{error:.1e}")),
                ],
            ));
        Ok(screen
            .hero(Hero::compare(source_pane, destination_pane))
            .caption(format!(
                "{} → {} · {} · round-trip error {error:.1e} · {}",
                space_name(source),
                space_name(destination),
                if inside { "in gamut" } else { "outside the destination gamut" },
                display.label(),
            ))
            .cards([
                card("Decoded source", "Linear light", CHANNELS, linear.components()),
                card("XYZ", "CIE 1931, D65", ["X", "Y", "Z"], xyz),
                card("OkLab", "Perceptual", ["L", "a", "b"], lab.components()),
                LabCard::new("OkLCh").subtitle("Hue in degrees").values([
                    ("L", format!("{:.4}", lch[0])),
                    ("C", format!("{:.4}", lch[1])),
                    ("H", format!("{:.1}°", lch[2])),
                ]),
            ])
            .panels(self.input_parameters(false), round_trip)
            .note("Converting never clips: values outside 0–1 are kept. Changing the source space reinterprets the same encoded numbers.")
            .credits(PREVIEW_CREDITS))
    }

    fn interpolation(&self, screen: LabScreen) -> anyhow::Result<LabScreen> {
        let rows = reference::rows(self.preset, self.policy, self.translucent)?;
        let hue = HUES[self.hue_index];
        let mut view = vstack! {}.gap_4().w_full();
        for (index, sample) in rows.iter().enumerate() {
            if index != 3 {
                view = view.child(reference_bar(&self.look, sample.label, &sample.rgba, &sample.outside)?);
                continue;
            }
            let model = ColorInterpolation::Oklch(hue);
            let gradient = LinearGradient::new([0., 0.], [1., 0.], self.endpoints()?, RgbSpace::Srgb, model)?;
            let mut colors = Vec::new();
            let mut outside = Vec::new();
            for i in 0..reference::SAMPLE_COUNT {
                let color = gradient.sample_at(i as f64 / (reference::SAMPLE_COUNT - 1) as f64)?;
                outside.push(!color.rgb().is_in_gamut());
                colors.push(preview(color.rgb(), self.policy, color.alpha())?);
            }
            view = view.child(reference_bar(&self.look, format!("OkLCh · {hue:?} hue"), &colors, &outside)?);
        }
        Ok(screen
            .hero(Hero::full(view))
            .caption(format!(
                "{} · {} · end alpha {} · OkLCh {hue:?} hue",
                reference::PRESETS[self.preset],
                mapping_name(self.policy),
                if self.translucent { "25%" } else { "100%" },
            ))
            .note(INTERPOLATION_NOTE)
            .credits(SRGB_REFERENCE_CREDITS))
    }

    fn mapping(&self, screen: LabScreen, display: &Display) -> anyhow::Result<LabScreen> {
        let look = &self.look;
        let destination = SPACES[self.destination_index];
        let input = self.saturated_input()?;
        let extended = input.convert(destination)?;
        // How far the unmapped color is outside 0–1 in the destination, in linear channel units.
        let excess = extended.components().into_iter().map(|v| (-v).max(v - 1.).max(0.)).fold(0., f64::max);
        let pane = |policy: GamutMapping, verb: &'static str| -> anyhow::Result<ComparePane> {
            let mapped = input.map_gamut(destination, policy)?;
            let changed = mapped.components().into_iter().zip(extended.components()).any(|(a, b)| (a - b).abs() > 1e-9);
            let swatch = display.fill(mapped, 1., GamutMapping::ReduceChroma, px(6.))?;
            Ok(ComparePane::new(mapping_name(policy), swatch_with(look, swatch, channels(mapped.components(), 4)))
                .subtitle(format!("Linear {}", space_name(destination)))
                .badge(self.badge(if changed { verb } else { "Unchanged" }, changed)))
        };
        // The color before mapping, drawn as far as the display allows (P3 on a wide window).
        let unmapped = PanelShell::new("Unmapped")
            .subtitle(format!("Converted to {}, before mapping", space_name(destination)))
            .child(div().w_full().h(px(80.)).child(display.fill(extended, 1., GamutMapping::ReduceChroma, px(6.))?))
            .child(properties(
                look,
                [
                    ("Linear", triple(extended.components())),
                    ("In gamut", yes_no(extended.is_in_gamut()).into()),
                    ("Outside by", format!("{excess:.4}")),
                ],
            ));
        Ok(screen
            .hero(Hero::compare(pane(GamutMapping::Clip, "Clipped")?, pane(GamutMapping::ReduceChroma, "Reduced")?))
            .caption(format!(
                "{} → {} · saturation ×{:.3} · {} · {}",
                space_name(SPACES[self.source_index]),
                space_name(destination),
                self.saturation,
                if extended.is_in_gamut() {
                    "in gamut, so both policies agree".to_owned()
                } else {
                    format!("outside by {excess:.4}")
                },
                display.label(),
            ))
            .panels(self.input_parameters(true), unmapped)
.note("Raise Saturation to push the color out of the destination gamut and watch the two policies part: Clip moves each channel into 0–1 on its own, Reduce chroma lowers chroma until the color fits. Whole gradients through both policies, drawn natively in P3, are on xgpui-gamut → Gamut Mapping.")
            .credits(PREVIEW_CREDITS))
    }

    /// CPU samples along the gradient axis, mapped into the destination, and whether each
    /// sample was outside the destination gamut before mapping.
    pub(super) fn cpu_samples(&self) -> anyhow::Result<(Vec<LinearRgba>, Vec<bool>)> {
        let destination = SPACES[self.destination_index];
        let gradient = self.gradient(self.model())?;
        let count = [32, 64, 128][self.sample_count_index];
        let mut colors = Vec::with_capacity(count);
        let mut outside = Vec::with_capacity(count);
        for i in 0..count {
            let t = i as f64 / (count - 1) as f64;
            let color = gradient.sample([self.axis[0] * t, self.axis[1] * t])?;
            outside.push(!color.rgb().is_in_gamut());
            colors.push(color.map_gamut(destination, self.policy)?);
        }
        Ok((colors, outside))
    }

    fn cpu_gradient_screen(&self, screen: LabScreen, display: &Display) -> anyhow::Result<LabScreen> {
        let look = &self.look;
        let destination = SPACES[self.destination_index];
        let count = [32, 64, 128][self.sample_count_index];
        let (colors, outside) = self.cpu_samples()?;
        let point = self.axis.map(|end| end * self.position);
        let sample = self.gradient(self.model())?.sample(point)?;
        let mapped = sample.map_gamut(destination, self.policy)?;
        let premultiplied = mapped.premultiplied();
        let label =
            format!("{count} samples, mapped into {} · marker at t = {:.3}", space_name(destination), self.position);
        let model = match self.model() {
            ColorInterpolation::Oklch(hue) => format!("OkLCh {hue:?} hue"),
            ColorInterpolation::Oklab => "OkLab".to_owned(),
            _ => "Linear RGB".to_owned(),
        };
        let sample_preview = PreviewPanel::new("Sample")
            .subtitle(format!("At t = {:.3} · {}", self.position, display.label()))
            .swatch(display.fill(mapped.rgb(), mapped.alpha(), self.policy, px(6.))?);
        let sampling = ParametersPanel::new("Sample position")
            .subtitle("Where along the gradient to sample: 0 is the first color, 1 the second")
            .params(self.params(&[Parameter::Position]));
        Ok(screen
            .hero(Hero::full(bar(
                look,
                label,
                with_marker(look, display.samples(&colors, self.policy, px(BAR_HEIGHT))?, self.position).into_any_element(),
                &outside,
            )))
            .caption(format!(
                "{} · {model} · output {} · {} · {}",
                reference::PRESETS[self.preset],
                space_name(destination),
                mapping_name(self.policy),
                display.label(),
            ))
            .cards([
                LabCard::new("Unmapped sample")
                    .icon(Icon::Crosshair)
                    .subtitle(format!("Linear {} · alpha {:.3}", space_name(destination), sample.alpha()))
                    .badge(self.gamut_badge(sample.rgb().is_in_gamut()))
                    .values(channels(sample.rgb().components(), 4)),
                LabCard::new("Mapped sample")
                    .icon(Icon::Target)
                    .subtitle(mapping_name(self.policy))
                    .badge(self.gamut_badge(mapped.rgb().is_in_gamut()))
                    .values(channels(mapped.rgb().components(), 4)),
                LabCard::new("Premultiplied")
                    .icon(Icon::Layers)
                    .subtitle("Output components")
                    .values(["R", "G", "B", "A"].into_iter().zip(premultiplied).map(|(key, v)| (key, format!("{v:.3}")))),
            ])
            .panels(sampling, sample_preview)
            .note("Positions outside 0–1 extend the end colors. This page checks xgpui-color's CPU gradient API: sampling, extension, mapping and premultiplied output.")
            .credits("Sampling is CPU reference math; these previews do not measure GPU performance. On a wide-gamut window the bar and sample are drawn in Display P3, with colors outside P3 fitted using the selected mapping; otherwise they are sRGB previews. Bars with more than 32 samples draw every sample as its own band."))
    }
}
