use std::{sync::Arc, rc::Rc, cell::Cell};
use gpui::{prelude::*, *};
use xgpui_luma_look_radix::{self as radix, Look};
use xgpui_luma::prelude::HasPresenter;
use xgpui_luma::controls::{slider::Slider, textfield::TextField, switch::Switch, button::Button};
use xgpui_color::GamutMapping;
use xgpui_luma_color::color_slider::{
    ColorSliderBuilder, ColorSliderDomainRenderer, GradientDelegate, primary_slider_value, refresh_color_slider,
};
use xgpui_shader::WindowShaderExt;
use super::chromaticity_shader::{self, Parameters};
use crate::{
    lab_controls::{Choice, labels},
    screen::{
        Hero, LabCard, LabMetrics, LabScreen, Legend, MONO_FAMILY, Mark, Param, ParametersPanel, PreviewPanel, data,
        labeled_control, value_field_look,
    },
    theme::ColorVizLookExt,
};
use super::chromaticity_model as model;

pub(crate) struct ChromaticityLab {
    look: Arc<Look>,
    rgb: [f32; 3],
    chart_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    drag_luminance: Option<f64>,
    inputs: Vec<TextField>,
    sliders: Vec<Slider>,
    slider_tracks: Vec<Arc<ColorSliderDomainRenderer>>,
    _subscriptions: Vec<Subscription>,
    presets: Vec<Entity<Button>>,
    hatch: bool,
    uv: bool,
    switches: Vec<Switch>,
    source: usize,
    patch: usize,
    simulated: usize,
    luminance: usize,
    mapping: GamutMapping,
    fill: bool,
    boundary: Arc<Vec<[f64; 2]>>,
    cells: Arc<Vec<model::Cell>>,
    error: Option<String>,
    choices: Vec<Choice>,
    shader: Result<xgpui_shader::instanced::ShaderProgram<Parameters>, String>,
}
impl ChromaticityLab {
    pub(crate) fn new(look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let choices = vec![
            Choice::new(
                "chromaticity-source",
                "Source",
                &look,
                labels(&model::NAMES),
                1,
                cx,
                |this: &mut Self, value, cx| {
                    this.source = value;
                    this.refresh_tracks(cx);
                    cx.notify();
                },
            ),
            Choice::new(
                "chromaticity-pixel",
                "Pixel",
                &look,
                model::PATCHES.iter().map(|p| p.0.to_owned()).collect(),
                0,
                cx,
                |this: &mut Self, value, cx| {
                    this.patch = value;
                    this.set_rgb(model::PATCHES[value].1, cx);
                    cx.notify();
                },
            ),
            Choice::new(
                "chromaticity-simulated",
                "Simulated gamut",
                &look,
                labels(&model::NAMES),
                0,
                cx,
                |this: &mut Self, value, cx| {
                    this.simulated = value;
                    this.rebuild();
                    cx.notify();
                },
            ),
            Choice::new(
                "chromaticity-luminance",
                "Fill Y",
                &look,
                labels(&["0.05", "0.18", "0.50"]),
                1,
                cx,
                |this: &mut Self, value, cx| {
                    this.luminance = value;
                    this.rebuild();
                    cx.notify();
                },
            ),
            Choice::new(
                "chromaticity-mapping",
                "Mapping",
                &look,
                labels(&["Clip", "Reduce chroma"]),
                0,
                cx,
                |this: &mut Self, value, cx| {
                    this.mapping = if value == 0 {
                        GamutMapping::Clip
                    } else {
                        GamutMapping::ReduceChroma
                    };
                    this.rebuild();
                    cx.notify();
                },
            ),
            Choice::new(
                "chromaticity-fill",
                "Fill",
                &look,
                labels(&["Visible", "Hidden"]),
                0,
                cx,
                |this: &mut Self, value, cx| {
                    this.fill = value == 0;
                    cx.notify();
                },
            ),
        ];
        let inputs = (0..3)
            .map(|index| {
                radix::TextField::new(format!("chroma-rgb-{index}"))
                    .look(&look)
                    .surface()
                    .look_override(value_field_look)
                    .full_width(true)
                    .value(format!("{:.3}", model::PATCHES[0].1[index]))
                    .validator(Arc::new(|text| parse_channel(text).is_some()))
                    .on_value_changed(cx, move |this: &mut Self, text, cx| {
                        if let Some(value) = parse_channel(text) {
                            this.rgb[index] = value;
                            this.sliders[index].update(cx, |slider, cx| slider.set_value(value, cx));
                            this.refresh_tracks(cx);
                            cx.notify();
                        }
                    })
                    .spawn(cx)
            })
            .collect();
        let mut sliders = Vec::new();
        let mut slider_tracks = Vec::new();
        let mut subscriptions = Vec::new();
        for index in 0..3 {
            let builder = ColorSliderBuilder::new(
                format!("chroma-slider-{index}"),
                model::PATCHES[0].1[index],
                channel_gradient(model::PATCHES[0].1, 1, index),
            )
            .horizontal()
            .range(0. ..1.)
            .step(0.001)
            .size(xgpui_luma::theme::ControlSize::Sm)
            .thumb_small();
            slider_tracks.push(builder.domain_renderer());
            let slider = builder.spawn(cx);
            subscriptions.push(cx.subscribe(&slider, move |this: &mut Self, _, event, cx| {
                if let Some(value) = primary_slider_value(event) {
                    this.rgb[index] = value;
                    this.inputs[index].update(cx, |input, cx| input.set_value(format!("{value:.3}"), cx));
                    this.refresh_tracks(cx);
                    cx.notify();
                }
            }));
            sliders.push(slider);
        }
        let presets = model::PATCHES
            .iter()
            .enumerate()
            .map(|(index, (label, _))| {
                radix::Button::new(format!("chroma-preset-{index}"))
                    .look(&look)
                    .radius(radix::Radius::Full)
                    .size(radix::ButtonSize::One)
                    .outline()
                    .label(*label)
                    .on_click(cx, move |this: &mut Self, cx| {
                        this.set_rgb(model::PATCHES[index].1, cx);
                    })
                    .spawn(cx)
            })
            .collect();
        let switches = vec![
            radix::Switch::new("chroma-hatching")
                .look(&look)
                .label("Hatching")
                .checked(true)
                .on_value_changed(cx, |this: &mut Self, value, cx| {
                    this.hatch = value;
                    cx.notify();
                })
                .spawn(cx),
            radix::Switch::new("chroma-uv")
                .look(&look)
                .label("u′v′")
                .checked(false)
                .on_value_changed(cx, |this: &mut Self, value, cx| {
                    this.uv = value;
                    cx.notify();
                })
                .spawn(cx),
        ];
        let mut this = Self {
            look,
            rgb: model::PATCHES[0].1,
            chart_bounds: Rc::new(Cell::new(None)),
            drag_luminance: None,
            inputs,
            sliders,
            slider_tracks,
            _subscriptions: subscriptions,
            presets,
            switches,
            hatch: true,
            uv: false,
            source: 1,
            patch: 0,
            simulated: 0,
            luminance: 1,
            mapping: GamutMapping::Clip,
            fill: true,
            boundary: Arc::new(model::locus()),
            cells: Arc::new(Vec::new()),
            error: None,
            choices,
            shader: chromaticity_shader::program().map_err(|error| error.to_string()),
        };
        this.rebuild();
        this
    }
    fn refresh_tracks(&self, cx: &mut Context<Self>) {
        for (index, track) in self.slider_tracks.iter().enumerate() {
            track.set_delegate(channel_gradient(self.rgb, self.source, index));
            refresh_color_slider(&self.sliders[index], cx);
        }
    }
    fn set_rgb(&mut self, rgb: [f32; 3], cx: &mut Context<Self>) {
        self.rgb = rgb;
        for (index, value) in rgb.into_iter().enumerate() {
            self.inputs[index].update(cx, |input, cx| input.set_value(format!("{value:.3}"), cx));
            self.sliders[index].update(cx, |slider, cx| slider.set_value(value, cx));
        }
        self.refresh_tracks(cx);
        cx.notify();
    }
    fn rebuild(&mut self) {
        match model::cells(
            model::SPACES[self.simulated],
            model::LUMINANCE[self.luminance],
            self.mapping,
            &self.boundary,
        ) {
            Ok(cells) => {
                self.cells = Arc::new(cells);
                self.error = None;
            }
            Err(error) => {
                self.cells = Arc::new(Vec::new());
                self.error = Some(error.to_string());
            }
        }
    }
    fn move_source(&mut self, position: Point<Pixels>, cx: &mut Context<Self>) {
        let Some(bounds) = self.chart_bounds.get() else {
            return;
        };
        let local = position - bounds.origin;
        let Some(xy) = model::pointer_xy(
            [f64::from(local.x), f64::from(local.y)],
            [f64::from(bounds.size.width), f64::from(bounds.size.height)],
            self.uv,
        ) else {
            return;
        };
        match model::rgb_at_xy(self.source, xy, self.drag_luminance.unwrap_or(0.18)) {
            Ok(rgb) => self.set_rgb(rgb, cx),
            Err(error) => {
                self.error = Some(error.to_string());
                cx.notify();
            }
        }
    }
    fn chart(&self, cx: &mut Context<Self>) -> AnyElement {
        let program = match &self.shader {
            Ok(program) => program.clone(),
            Err(error) => {
                return div().p_4().child(format!("Chromaticity shader unavailable: {error}")).into_any_element();
            }
        };
        let selected = model::source_xy(model::SPACES[self.source], self.rgb);
        let point = selected.unwrap_or([0., 0.]);
        let parameters = Parameters {
            simulated: self.simulated as f32,
            luminance: model::LUMINANCE[self.luminance] as f32,
            reduce_chroma: if self.mapping == GamutMapping::ReduceChroma {
                1.
            } else {
                0.
            },
            show_fill: if self.fill { 1. } else { 0. },
            source_x: point[0] as f32,
            source_y: point[1] as f32,
            source_visible: if selected.is_some() { 1. } else { 0. },
            overlays: 1.,
            hatch: if self.hatch { 1. } else { 0. },
            uv: if self.uv { 1. } else { 0. },
            ..bytemuck::Zeroable::zeroed()
        }
        .with_colors(&self.look);
        let chart_bounds = self.chart_bounds.clone();
        let axis_color = self.look.chrome().muted_text;
        let plot = canvas(
            move |bounds, _, _| {
                chart_bounds.set(Some(bounds));
            },
            move |bounds, _, window, cx| {
                if let Err(error) = window.paint_custom_shader(bounds, &program, parameters) {
                    tracing::error!("Chromaticity chart: {error:#}");
                }
                let extent = if parameters.uv > 0.5 { [0.65, 0.65] } else { [0.8, 0.9] };
                let width = bounds.size.width.as_f32();
                let height = bounds.size.height.as_f32();
                let scale = ((width - 48.) / extent[0]).min((height - 48.) / extent[1]).max(1.);
                let left = (width - scale * extent[0]) * 0.5;
                let bottom = height - 24.;
                let mut label = |text: String, x: f32, y: f32| {
                    let line = window.text_system().shape_line(
                        text.clone().into(),
                        LabMetrics::STANDARD.axis_tick,
                        &[TextRun {
                            len: text.len(),
                            font: font(MONO_FAMILY),
                            color: axis_color,
                            ..Default::default()
                        }],
                        None,
                    );
                    if let Err(error) = line.paint(
                        bounds.origin + gpui::point(px(x), px(y)),
                        px(14.),
                        TextAlign::Left,
                        None,
                        window,
                        cx,
                    ) {
                        tracing::error!("Chromaticity axis: {error:#}");
                    }
                };
                for tick in 0..=(extent[0] * 10.) as usize {
                    label(format!("{:.1}", tick as f32 / 10.), left + tick as f32 * scale / 10. - 8., bottom + 4.);
                }
                for tick in 0..=(extent[1] * 10.) as usize {
                    label(format!("{:.1}", tick as f32 / 10.), left - 24., bottom - tick as f32 * scale / 10. - 7.);
                }
                label(if parameters.uv > 0.5 { "u′" } else { "x" }.into(), left + extent[0] * scale + 16., bottom + 4.);
                label(
                    if parameters.uv > 0.5 { "v′" } else { "y" }.into(),
                    left - 24.,
                    bottom - extent[1] * scale - 24.,
                );
            },
        )
        .w_full()
        // Preserve the 48 px chart inset and enlarge the plotted area by 50%.
        .h(px(48. + (420. - 48.) * 1.5))
        .flex_shrink_0();
        div()
            .id("chromaticity-source-drag")
            .w_full()
            .cursor_crosshair()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, _, cx| {
                    this.drag_luminance = Some(
                        model::source_linear(this.source, this.rgb)
                            .and_then(|v| Ok(v.to_xyz()?.components()[1]))
                            .unwrap_or(0.18),
                    );
                    this.move_source(event.position, cx);
                    cx.stop_propagation();
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                if event.pressed_button != Some(MouseButton::Left) {
                    this.drag_luminance = None;
                }
                if this.drag_luminance.is_some() {
                    this.move_source(event.position, cx);
                }
            }))
            .on_mouse_up(MouseButton::Left, cx.listener(|this, _, _, _| this.drag_luminance = None))
            .on_mouse_up_out(MouseButton::Left, cx.listener(|this, _, _, _| this.drag_luminance = None))
            .child(plot)
            .into_any_element()
    }
}
impl Parameters {
    /// Overlay colors: grid, locus and the D65 marker follow the look; the source marker is data.
    fn with_colors(mut self, look: &Look) -> Self {
        let chrome = look.chrome();
        let rgb = |color: Hsla| {
            let color = Rgba::from(color);
            [color.r, color.g, color.b]
        };
        [self.grid_r, self.grid_g, self.grid_b] = rgb(chrome.border);
        [self.locus_r, self.locus_g, self.locus_b] = rgb(chrome.muted_text);
        [self.reference_r, self.reference_g, self.reference_b] = rgb(chrome.body_text);
        [self.reference_halo_r, self.reference_halo_g, self.reference_halo_b] = rgb(chrome.content_background);
        [self.subject_r, self.subject_g, self.subject_b] = rgb(data::color(data::MARKER_SUBJECT));
        [self.subject_halo_r, self.subject_halo_g, self.subject_halo_b] = rgb(data::color(data::MARKER_SUBJECT_HALO));
        self
    }
}

// Vary one encoded channel while retaining the other two, then preview in sRGB.
fn channel_gradient(rgb: [f32; 3], source: usize, channel: usize) -> Arc<GradientDelegate> {
    let colors = (0..=16)
        .map(|step| {
            let mut sample = rgb;
            sample[channel] = step as f32 / 16.;
            let color = model::preview(source, sample, 0, GamutMapping::Clip).unwrap_or_else(|_| sample.map(f64::from));
            xgpui_luma::color::ColorValue::srgb(color[0] as f32, color[1] as f32, color[2] as f32, 1.)
        })
        .collect();
    Arc::new(GradientDelegate::from_colors(colors).expect("finite bounded channel gradient"))
}
fn parse_channel(text: &str) -> Option<f32> {
    text.trim().parse::<f32>().ok().filter(|v| v.is_finite() && (0. ..=1.).contains(v))
}

fn swatch(rgb: [f64; 3]) -> Rgba {
    Rgba { r: rgb[0] as f32, g: rgb[1] as f32, b: rgb[2] as f32, a: 1. }
}
fn hex(rgb: [f64; 3]) -> String {
    let [r, g, b] = rgb.map(|v| (v.clamp(0., 1.) * 255.).round() as u8);
    format!("#{r:02X}{g:02X}{b:02X}")
}

const INTERACTION_NOTE: &str = "Click or drag the diagram to move the source pixel. Positions outside the source gamut are constrained to its RGB bounds. Changing source keeps the encoded RGB values and reinterprets them, so the gold cross moves while the controls stay the same. The simulated gamut sets hatching, mapping, and preview colors. Cards report source-pixel membership in each reference gamut.";
const CIE_LOCUS_CREDIT: &str = "CIE 1931 2° spectral locus: CIE 2019, DOI 10.25039/CIE.DS.mifmy4x4, CC BY-SA 4.0. The plotted locus retains that license. Fill colors are an sRGB preview, even on a wide-gamut window. Coverage percentages describe reference xy area. Hatching describes the selected simulated gamut, not a measurement of your monitor.";

impl ChromaticityLab {
    fn legend(&self) -> Legend {
        let legend = model::NAMES.into_iter().enumerate().fold(Legend::new(), |legend, (index, name)| {
            let coverage = model::coverage(model::SPACES[index], &self.boundary)
                .map(|fraction| format!("{:.0}%", fraction * 100.))
                .unwrap_or_else(|_| "–".into());
            legend.item(Mark::Line(data::color(data::GAMUTS[index])), name, coverage)
        });
        legend
            .note("% of visible xy area covered")
            .marker(Mark::Cross { color: None, halo: None }, "D65 white point")
            .marker(
                Mark::Cross {
                    color: Some(data::color(data::MARKER_SUBJECT)),
                    halo: Some(data::color(data::MARKER_SUBJECT_HALO)),
                },
                "Source pixel",
            )
            .marker(Mark::Hatch, "Outside simulated gamut")
    }

    fn caption_line(&self) -> String {
        let outside = self.cells.iter().filter(|cell| cell.outside).count();
        let fraction = 100. * outside as f64 / self.cells.len().max(1) as f64;
        format!(
            "{} · fill Y {} · {} · sampled area outside {}: {fraction:.1}%",
            if self.uv { "CIE 1976 u′v′" } else { "CIE 1931 xy" },
            model::LUMINANCE[self.luminance],
            if self.mapping == GamutMapping::Clip {
                "clip"
            } else {
                "reduce chroma"
            },
            model::NAMES[self.simulated]
        )
    }

    fn space_card(&self, index: usize) -> LabCard {
        let name = model::NAMES[index];
        let card = LabCard::new(name).mark(data::color(data::GAMUTS[index]));
        let values = model::source_linear(self.source, self.rgb)
            .and_then(|v| Ok(v.convert(model::SPACES[index])?.encode()?.components()));
        let values = match values {
            Ok(values) => values,
            Err(error) => return card.subtitle(error.to_string()),
        };
        let inside = values.iter().all(|v| (-1e-7..=1. + 1e-7).contains(v));
        let subtitle = if self.source == index {
            "Source · D65"
        } else {
            ["IEC 61966-2-1", "P3 · D65", "γ 2.2", "ITU-R BT.2020"][index]
        };
        let badge = radix::Badge::new(&self.look, if inside { "In" } else { "Out" }).variant(if inside {
            radix::BadgeVariant::Soft
        } else {
            radix::BadgeVariant::Outline
        });
        card.when_some(model::preview(self.source, self.rgb, index, GamutMapping::Clip).ok(), |card, color| {
            card.swatch_color(swatch(color))
        })
        .subtitle(subtitle)
        .badge(badge)
        .values(["R", "G", "B"].into_iter().zip(values.map(|value| format!("{value:.3}"))))
        .debug_selector(move || format!("chromaticity-card-{name}-{}", if inside { "in" } else { "out" }))
    }

    fn pixel_parameters(&self) -> ParametersPanel {
        ParametersPanel::new("Pixel parameters")
            .subtitle(format!("Encoded {} values (0–1)", model::NAMES[self.source]))
            .params(
                ["Red", "Green", "Blue"]
                    .into_iter()
                    .enumerate()
                    .map(|(index, label)| Param::new(label, self.inputs[index].clone(), self.sliders[index].clone())),
            )
            .footer(div().flex().flex_wrap().gap_1().children(self.presets.clone()))
    }

    fn pixel_preview(&self) -> PreviewPanel {
        let panel = PreviewPanel::new("Pixel preview").subtitle("Source pixel mapped into the simulated gamut");
        let panel = [("Clip", GamutMapping::Clip), ("Reduce chroma", GamutMapping::ReduceChroma)].into_iter().fold(
            panel,
            |panel, (label, mapping)| match model::preview(self.source, self.rgb, self.simulated, mapping) {
                Ok(color) => panel.labeled_color(swatch(color), label, hex(color)),
                Err(_) => panel.labeled(div(), label, "—"),
            },
        );
        // Undefined coordinates (black, or a failed conversion) read as dashes, not prose.
        let xyz = model::source_linear(self.source, self.rgb).and_then(|v| Ok(v.to_xyz()?.components())).ok();
        let fixed = |value: Option<f64>| value.map_or_else(|| "—".to_owned(), |value| format!("{value:.4}"));
        let xy = xyz.and_then(model::xy);
        let uv = xyz.filter(|_| xy.is_some()).map(|[x, y, z]| {
            let d = x + 15. * y + 3. * z;
            [4. * x / d, 9. * y / d]
        });
        panel.reference_values(
            "Source",
            [
                ("x", fixed(xy.map(|p| p[0]))),
                ("y", fixed(xy.map(|p| p[1]))),
                ("u′", fixed(uv.map(|p| p[0]))),
                ("v′", fixed(uv.map(|p| p[1]))),
                ("Y", fixed(xyz.map(|v| v[1]))),
            ],
        )
    }
}

impl Render for ChromaticityLab {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // The pixel selector stays in the model (tests drive it); presets replace it on screen.
        let choices = self.choices.iter().enumerate().filter(|(index, _)| *index != 1);
        let controls = choices.map(|(_, choice)| labeled_control(&self.look, choice.clone().into_element()));
        let screen = LabScreen::new("chromaticity", self.look.clone())
            .crumb("xgpui-gamut", "Chromaticity")
            .title("Chromaticity & Gamut")
            .lede("Keep one pixel fixed and change how it is interpreted. Compare reference gamut boundaries, luminance, and mapping on the CIE 1931 diagram.")
            .hero(Hero::with_legend(self.chart(cx), self.legend()))
            .caption(self.caption_line());
        controls
            .fold(screen, LabScreen::control)
            .control(self.switches[0].clone())
            .control(self.switches[1].clone())
            .notice(self.error.clone())
            .cards((0..model::NAMES.len()).map(|index| self.space_card(index)))
            .panels(self.pixel_parameters(), self.pixel_preview())
            .note(INTERACTION_NOTE)
            .credits(CIE_LOCUS_CREDIT)
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::{ChromaticityLab, GamutMapping};
    use gpui::TestAppContext;

    #[test]
    fn native_selectors_update_simulation_and_source_independently() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).expect("initialize SDK"));
        let (view, cx) = app.add_window_view(|_, cx| ChromaticityLab::new(crate::theme::default_look(), cx));
        cx.run_until_parked();
        cx.update(|_, cx| view.read(cx).choices[2].clone().select(1, cx));
        cx.run_until_parked();
        let (choices, outside) = cx.update(|_, cx| {
            let lab = view.read(cx);
            (lab.choices.clone(), lab.cells.iter().filter(|cell| cell.outside).count())
        });
        cx.update(|_, cx| choices[2].select(0, cx));
        cx.run_until_parked();
        cx.update(|_, cx| assert!(view.read(cx).cells.iter().filter(|cell| cell.outside).count() > outside));
        cx.update(|_, cx| choices[0].select(3, cx));
        cx.update(|_, cx| choices[1].select(5, cx));
        cx.update(|_, cx| choices[4].select(1, cx));
        cx.update(|_, cx| choices[5].select(1, cx));
        cx.run_until_parked();
        cx.update(|_, cx| {
            let lab = view.read(cx);
            assert_eq!((lab.source, lab.patch, lab.simulated), (3, 5, 0));
            assert_eq!(lab.mapping, GamutMapping::ReduceChroma);
            assert!(!lab.fill);
            assert!(lab.error.is_none());
        });
    }
}
