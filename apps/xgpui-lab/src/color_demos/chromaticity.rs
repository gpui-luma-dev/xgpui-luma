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
    theme::{ColorVizLookExt, TypographyExt},
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
                    .look_override(|mut look| {
                        look.typography.size = 10.;
                        look.typography.line_height = 14.;
                        look.font_family = "monospace".into();
                        look
                    })
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
        };
        let chart_bounds = self.chart_bounds.clone();
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
                        px(10.),
                        &[TextRun {
                            len: text.len(),
                            font: font("monospace"),
                            color: rgb(0x92959c).into(),
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
                label(if parameters.uv > 0.5 { "u′" } else { "x" }.into(), left + extent[0] * scale + 10., bottom + 4.);
                label(
                    if parameters.uv > 0.5 { "v′" } else { "y" }.into(),
                    left - 24.,
                    bottom - extent[1] * scale - 16.,
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
impl Render for ChromaticityLab {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let typography = self.look.common_stylesheet().tokens.typography.text;
        let outside = self.cells.iter().filter(|cell| cell.outside).count();
        let fraction = 100. * outside as f64 / self.cells.len().max(1) as f64;
        let panel = || {
            div()
                .p_4()
                .rounded_lg()
                .border_1()
                .border_color(chrome.border)
                .bg(chrome.panel_background)
                .flex()
                .flex_col()
                .gap_2()
        };
        let legend = div()
            .w(px(220.))
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap_3()
            .children(model::NAMES.into_iter().zip(model::COLORS).zip(["34%", "46%", "45%", "64%"]).map(
                |((name, color), area)| {
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().w(px(16.)).h(px(2.)).bg(rgb(color)))
                        .child(div().flex_1().child(name))
                        .child(div().text_color(chrome.muted_text).child(area))
                },
            ))
            .child(div().text_sm().text_color(chrome.muted_text).child("% of visible xy area covered"))
            .child(div().h(px(1.)).bg(chrome.border))
            .child("+ D65 white point")
            .child(div().text_color(rgb(0xffdf62)).child("+ Source pixel"))
            .child("//// Outside simulated gamut");
        let cards = model::NAMES
            .into_iter()
            .enumerate()
            .map(|(index, name)| {
                let values = model::source_linear(self.source, self.rgb)
                    .and_then(|v| Ok(v.convert(model::SPACES[index])?.encode()?.components()));
                let mut card = div()
                    .p_3()
                    .rounded_lg()
                    .border_1()
                    .border_color(chrome.border)
                    .bg(chrome.panel_background)
                    .flex()
                    .flex_col()
                    .gap_2()
                    .text_size(px(12.))
                    .line_height(px(16.))
                    .flex_1()
                    .min_w(px(205.));
                match values {
                    Ok(values) => {
                        let inside = values.iter().all(|v| (-1e-7..=1. + 1e-7).contains(v));
                        let preview = model::preview(self.source, self.rgb, index, GamutMapping::Clip);
                        let mut title = div().flex().items_center().gap_2();
                        if let Ok(color) = preview {
                            title = title.child(div().size(px(34.)).flex_shrink_0().rounded_md().bg(swatch(color)));
                        }
                        let metadata = if self.source == index {
                            "Source · D65"
                        } else {
                            ["IEC 61966-2-1", "P3 · D65", "γ 2.2", "ITU-R BT.2020"][index]
                        };
                        let label = div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .child(
                                        div().size(px(6.)).flex_shrink_0().rounded_sm().bg(rgb(model::COLORS[index])),
                                    )
                                    .child(name),
                            )
                            .child(div().text_size(px(11.)).text_color(chrome.muted_text).child(metadata));
                        let status_color = if inside { rgb(0x79b97c) } else { rgb(0xe6a06c) };
                        let badge = div()
                            .flex_shrink_0()
                            .px_2()
                            .rounded_full()
                            .border_1()
                            .border_color(status_color.opacity(0.3))
                            .text_color(status_color)
                            .text_size(px(10.))
                            .line_height(px(16.))
                            .child(if inside { "In" } else { "Out" });
                        card = card.child(title.child(label).child(badge)).child(
                            div()
                                .font_family("monospace")
                                .text_size(px(10.))
                                .line_height(px(14.))
                                .child(format!("R {:.3}  G {:.3}  B {:.3}", values[0], values[1], values[2])),
                        );
                    }
                    Err(error) => card = card.child(error.to_string()),
                }
                card
            })
            .collect::<Vec<_>>();
        let parameters = panel()
            .flex_grow(1.)
            .flex_basis(px(500.))
            .min_w(px(400.))
            .text_size(px(12.))
            .line_height(px(16.))
            .gap_2()
            .child(div().text_size(px(12.)).font_weight(FontWeight::MEDIUM).child("Pixel parameters"))
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(chrome.muted_text)
                    .child(format!("Encoded {} values (0–1)", model::NAMES[self.source])),
            )
            .child(div().flex().gap_2().children((0..3).map(|index| {
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(["Red", "Green", "Blue"][index])
                    .child(self.inputs[index].clone())
                    .child(self.sliders[index].clone())
            })))
            .child(div().flex().flex_wrap().gap_1().children(self.presets.clone()));
        let previews = panel()
            .flex_grow(1.)
            .flex_basis(px(300.))
            .min_w(px(300.))
            .text_size(px(12.))
            .line_height(px(16.))
            .gap_2()
            .child(div().text_size(px(12.)).font_weight(FontWeight::MEDIUM).child("Pixel preview"))
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(chrome.muted_text)
                    .child("Source pixel mapped into the simulated gamut"),
            )
            .child(div().flex().gap(px(6.)).children(
                [("Clip", GamutMapping::Clip), ("Reduce chroma", GamutMapping::ReduceChroma)].into_iter().map(
                    |(label, mapping)| {
                        let mut view = div().flex_1().flex().flex_col().gap_2();
                        match model::preview(self.source, self.rgb, self.simulated, mapping) {
                            Ok(color) => {
                                view = view.child(div().w_full().h(px(64.)).rounded_md().bg(swatch(color))).child(
                                    div()
                                        .flex()
                                        .flex_wrap()
                                        .items_baseline()
                                        .gap_1()
                                        .text_size(px(10.))
                                        .line_height(px(14.))
                                        .child(div().text_color(chrome.muted_text).child(format!("{label} ·")))
                                        .child(div().font_family("monospace").child(hex(color))),
                                )
                            }
                            Err(error) => view = view.child(error.to_string()),
                        }
                        view
                    },
                ),
            ))
            .child(
                div()
                    .flex()
                    .items_baseline()
                    .gap_1()
                    .text_size(px(10.))
                    .line_height(px(14.))
                    .child(div().text_color(chrome.muted_text).child("Source"))
                    .child(div().flex_1().min_w_0().font_family("monospace").child(
                        match model::source_linear(self.source, self.rgb).and_then(|v| Ok(v.to_xyz()?.components())) {
                            Ok(xyz) => match model::xy(xyz) {
                                Some([x, y]) => {
                                    let d = xyz[0] + 15. * xyz[1] + 3. * xyz[2];
                                    format!(
                                        "x {x:.4}  y {y:.4} · u′ {:.4}  v′ {:.4} · Y {:.4}",
                                        4. * xyz[0] / d,
                                        9. * xyz[1] / d,
                                        xyz[1]
                                    )
                                }
                                None => "Black · chromaticity undefined".into(),
                            },
                            Err(error) => error.to_string(),
                        },
                    )),
            );
        div().id("chromaticity-lab-scroll").size_full().overflow_y_scroll().p_4()
            .bg(chrome.content_background).text_color(chrome.body_text)
            .child(div().w_full().max_w(px(1280.)).mx_auto().flex().flex_col().gap_5()
                .child(div().text_sm().text_color(chrome.muted_text).child("xgpui-gamut / Chromaticity"))
                .child(div().flex().flex_col().gap_1()
                    .child(div().typography_style(typography.role.h2).child("Chromaticity & Gamut"))
                    .child(div().w_full().typography_style(typography.scale.sm).text_color(chrome.muted_text).child("Keep one pixel fixed and change how it is interpreted. Compare reference gamut boundaries, luminance, and mapping on the CIE 1931 diagram.")))
                .child(div().flex().flex_wrap().items_center().gap_5().child(div().flex_1().min_w(px(400.)).child(self.chart(cx))).child(legend))
                .child(format!("{} · fill Y {} · {} · sampled area outside {}: {fraction:.1}%", if self.uv { "CIE 1976 u′v′" } else { "CIE 1931 xy" }, model::LUMINANCE[self.luminance], if self.mapping == GamutMapping::Clip { "clip" } else { "reduce chroma" }, model::NAMES[self.simulated]))
                .child(div().flex().flex_wrap().gap_3().children(self.choices.iter().enumerate().filter(|(i, _)| *i != 1).map(|(_, c)| c.clone())).children(self.switches.clone()))
                .child(div().flex().flex_wrap().gap_3().children(cards))
                .child(div().flex().flex_wrap().items_stretch().gap_4().child(parameters).child(previews))
                .when_some(self.error.clone(), |view, error| view.child(error))
                .child(div().text_sm().text_color(chrome.muted_text).child("Click or drag the diagram to move the source pixel. Positions outside the source gamut are constrained to its RGB bounds. Changing source keeps the encoded RGB values and reinterprets them, so the gold cross moves while the controls stay the same. The simulated gamut sets hatching, mapping, and preview colors. Cards report source-pixel membership in each reference gamut."))
                .child(div().text_size(px(12.)).text_color(chrome.muted_text).child("CIE 1931 2° spectral locus: CIE 2019, DOI 10.25039/CIE.DS.mifmy4x4, CC BY-SA 4.0. The plotted locus retains that license. Fill colors are an sRGB preview, even on a wide-gamut window. Coverage percentages describe reference xy area. Hatching describes the selected simulated gamut, not a measurement of your monitor.")))
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
