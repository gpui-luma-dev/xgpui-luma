//! Embedded adaptations of xgpui/apps/gamut-lab/{main,color_main}.rs.
use super::*;
use super::palette;
use crate::screen::{Hero, LabScreen, TextRole, labeled_control, text};
use gpui::{AnyElement, IntoElement, rgba, rgb};
use xgpui_gamut::{
    color::{ColorInterpolation, EncodedRgb, HueInterpolation, LinearRgba},
    instanced::WideGradient,
};

fn strip(colors: &[[f32; 4]], p3: bool) -> anyhow::Result<AnyElement> {
    // Use direct native shader instances for reference cells. WideColor custom
    // draws allocate one uncommitted Metal command buffer per cell; dense strips
    // can exhaust the queue before the renderer reaches its submission phase.
    let painter = p3.then(WidePainter::new).transpose()?.map(Arc::new);
    let cells = colors
        .iter()
        .map(|&[r, g, b, a]| {
            let cell = div().flex_1().min_w_0().h_full();
            if p3 {
                let [r, g, b] =
                    EncodedRgb::new(RgbSpace::DisplayP3, [r as f64, g as f64, b as f64])?.decode()?.components();
                let color = WideColor::linear_p3([r as f32, g as f32, b as f32, a])?;
                let painter = painter.as_ref().unwrap().clone();
                Ok(cell
                    .child(
                        canvas(
                            |_, _, _| (),
                            move |bounds, _, window, _| {
                                if let Err(error) = painter.paint_color(window, bounds, px(0.), &color) {
                                    tracing::error!("Gamut reference strip: {error:#}");
                                }
                            },
                        )
                        .size_full(),
                    )
                    .into_any_element())
            } else {
                Ok(cell.bg(Rgba { r, g, b, a }).into_any_element())
            }
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    Ok(div().flex().w_full().h(px(48.0)).children(cells).into_any_element())
}

impl GamutPage {
    pub(super) fn lab_screen(
        &self,
        wide: bool,
        display: &str,
        fallback: &str,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> LabScreen {
        self.sync_choices(cx);
        let presentation = self.active_tab == "Overlays";
        let mapping = self.active_tab == "Gamut Mapping";
        let (id, lede, note) = if presentation {
            (
                "gamut-overlays",
                "Explore how vivid colors, translucent overlays, and overlapping surfaces are combined.",
                "Change the presentation path to compare texture, instanced solid, and OkLab gradient rendering. Overlays are drawn between and above the two surfaces.",
            )
        } else if mapping {
            (
                "gamut-mapping",
                "Compare two ways to fit a color blend into the colors a Display P3 screen can show.",
                "Clip limits each channel separately; Reduce chroma lowers vividness until the color fits. On wide-gamut output, each shader bar is followed by CPU reference samples. Numeric conversions are on xgpui-color → Gamut Mapping.",
            )
        } else {
            (
                "gamut-gradients",
                "Compare how different ways of blending the same two colors change the colors in between.",
                "Each blend is followed by its sampled sRGB reference. Encoded sRGB uses sampled bands; the other native P3 rows use instanced shaders.",
            )
        };
        let actions: &[Action] = if presentation {
            &[
                Action::Presentation,
                Action::Overlay,
                Action::Opacity,
                Action::Order,
                Action::Background,
                Action::Route,
            ]
        } else if mapping {
            &[Action::Preset, Action::Alpha]
        } else {
            &[Action::Preset, Action::Mapping, Action::Alpha]
        };
        let output = if wide {
            "native Display P3"
        } else {
            "sRGB fallback preview"
        };
        let caption = if presentation {
            format!(
                "{} · {output} · opacity {:.2} · {} background · {} · {}",
                ["Texture", "Instanced solid", "OkLab gradient"][self.route],
                if self.half_opacity { 0.5 } else { 1.0 },
                self.background.label(),
                if self.overlay {
                    "overlays shown"
                } else {
                    "overlays hidden"
                },
                if self.reverse {
                    "reversed order"
                } else {
                    "original order"
                }
            )
        } else {
            format!(
                "{} · {} · end alpha {} · {output}",
                reference::PRESETS[self.preset],
                if mapping {
                    "Clip vs Reduce chroma"
                } else if self.policy == GamutMapping::Clip {
                    "Clip"
                } else {
                    "Reduce chroma"
                },
                if self.translucent { "25%" } else { "100%" }
            )
        };
        let mut screen = LabScreen::new(id, self.look.clone())
            .crumb("xgpui-gamut", self.active_tab.as_ref()).title(self.active_tab.clone()).lede(lede)
            .caption(caption).section_panel(self.presentation_panel(display, fallback, window)).note(note)
            .credits("Native P3 output requires a wide-gamut window and display. Fallback previews do not validate native output; screenshots do not measure display fidelity.");
        for (_, choice) in self.choices.iter().filter(|(action, _)| actions.contains(action)) {
            screen = screen.control(labeled_control(&self.look, choice.clone().into_element()));
        }
        for (_, switch) in self.switches.iter().filter(|(action, _)| actions.contains(action)) {
            screen = screen.control(switch.clone());
        }
        for (_, button) in self.controls.iter().filter(|(action, _)| actions.contains(action)) {
            screen = screen.action(button.clone());
        }
        let result = if presentation {
            self.presentation(wide)
        } else if mapping {
            self.mapping(wide)
        } else {
            self.gradients(wide)
        };
        match result {
            Ok(view) => screen.hero(Hero::full(view)),
            Err(error) => {
                self.paint_errors.borrow_mut().push(format!("{error:#}"));
                screen.notice(Some(format!("Lab unavailable: {error:#}")))
            }
        }
    }

    fn gradients(&self, wide: bool) -> anyhow::Result<AnyElement> {
        // Same endpoint presets and mapping policies as color-lab.
        let srgb = reference::rows(self.preset, self.policy, self.translucent)?;
        let p3 = reference::rows_in_space(self.preset, self.policy, self.translucent, RgbSpace::DisplayP3)?;
        let mut view = div().flex().flex_col().gap_4();
        for (index, (srgb, p3)) in srgb.iter().zip(&p3).enumerate() {
            let mut row = div().flex().flex_col().gap_2().child(text(&self.look, TextRole::Label, srgb.label));
            if wide && index > 0 {
                let painter = self.painter.as_ref().map_err(|error| anyhow::anyhow!("{error:#}"))?;
                let decode = |rgba: [f32; 4]| -> anyhow::Result<LinearRgba> {
                    Ok(LinearRgba::new(
                        EncodedRgb::new(RgbSpace::DisplayP3, [rgba[0] as f64, rgba[1] as f64, rgba[2] as f64])?
                            .decode()?,
                        rgba[3] as f64,
                    )?)
                };
                let model = match index {
                    1 => ColorInterpolation::LinearRgb,
                    2 => ColorInterpolation::Oklab,
                    _ => ColorInterpolation::Oklch(HueInterpolation::Shorter),
                };
                let gradient = WideGradient::linear(
                    [0.0, 0.0],
                    [1.0, 0.0],
                    [decode(p3.rgba[0])?, decode(p3.rgba[reference::SAMPLE_COUNT - 1])?],
                    model,
                    self.policy,
                )?;
                let painter = painter.clone();
                row = row.child(
                    div().w_full().h(px(48.0)).child(
                        canvas(
                            |_, _, _| (),
                            move |bounds, _, window, _| {
                                if let Err(error) = painter.paint_gradient(window, bounds, px(0.0), &gradient) {
                                    eprintln!("Gamut gradient paint failed: {error:#}");
                                }
                            },
                        )
                        .size_full(),
                    ),
                );
            } else {
                row = row.child(strip(if wide { &p3.rgba } else { &srgb.rgba }, wide)?);
            }
            view = view.child(
                row.child(text(&self.look, TextRole::Label, "sRGB reference · 32 samples"))
                    .child(strip(&srgb.rgba, false)?),
            );
        }
        Ok(view.into_any_element())
    }

    fn mapping(&self, wide: bool) -> anyhow::Result<AnyElement> {
        let mut view = div().flex().flex_col().gap_3();
        for policy in [GamutMapping::Clip, GamutMapping::ReduceChroma] {
            let rows = reference::rows_in_space(self.preset, policy, self.translucent, RgbSpace::DisplayP3)?;
            for (index, row) in rows.iter().enumerate().skip(1) {
                view = view.child(text(
                    &self.look,
                    TextRole::Detail,
                    format!(
                        "{} · {policy:?} · {} source samples outside Display P3",
                        row.label,
                        row.outside.iter().filter(|v| **v).count()
                    ),
                ));
                if wide {
                    let decode = |rgba: [f32; 4]| -> anyhow::Result<LinearRgba> {
                        Ok(LinearRgba::new(
                            EncodedRgb::new(RgbSpace::DisplayP3, [rgba[0] as f64, rgba[1] as f64, rgba[2] as f64])?
                                .decode()?,
                            rgba[3] as f64,
                        )?)
                    };
                    let model = match index {
                        1 => ColorInterpolation::LinearRgb,
                        2 => ColorInterpolation::Oklab,
                        _ => ColorInterpolation::Oklch(HueInterpolation::Shorter),
                    };
                    let gradient = WideGradient::linear(
                        [0., 0.],
                        [1., 0.],
                        [decode(row.rgba[0])?, decode(*row.rgba.last().unwrap())?],
                        model,
                        policy,
                    )?;
                    let painter = self.painter.as_ref().map_err(|e| anyhow::anyhow!("{e:#}"))?.clone();
                    let paint_errors = self.paint_errors.clone();
                    view = view
                        .child(
                            div().h(px(48.)).child(
                                canvas(
                                    |_, _, _| (),
                                    move |bounds, _, window, _| {
                                        if let Err(error) = painter.paint_gradient(window, bounds, px(0.), &gradient) {
                                            paint_errors.borrow_mut().push(format!("P3 mapping: {error:#}"));
                                            eprintln!("P3 mapping: {error:#}");
                                        }
                                    },
                                )
                                .size_full(),
                            ),
                        )
                        .child(text(&self.look, TextRole::Label, "CPU P3 reference samples"))
                        .child(strip(&row.rgba, true)?);
                } else {
                    let colors = row
                        .rgba
                        .iter()
                        .map(|rgba| -> anyhow::Result<[f32; 4]> {
                            let rgb =
                                EncodedRgb::new(RgbSpace::DisplayP3, [rgba[0] as f64, rgba[1] as f64, rgba[2] as f64])?
                                    .decode()?
                                    .map_gamut(RgbSpace::Srgb, GamutMapping::ReduceChroma)?
                                    .encode()?
                                    .components();
                            Ok([rgb[0] as f32, rgb[1] as f32, rgb[2] as f32, rgba[3]])
                        })
                        .collect::<anyhow::Result<Vec<_>>>()?;
                    view = view
                        .child(text(&self.look, TextRole::Label, "P3 → sRGB preview"))
                        .child(strip(&colors, false)?);
                }
            }
        }
        Ok(view.into_any_element())
    }

    fn presentation(&self, wide: bool) -> anyhow::Result<AnyElement> {
        let opacity = if self.half_opacity { 0.5 } else { 1.0 };
        let surface = |left: f32, top: f32, source: [f32; 4], fallback| -> anyhow::Result<AnyElement> {
            let color = Arc::new(WideColor::linear_p3(source)?);
            let route = self.route;
            let painter = if wide && route > 0 {
                Some(self.painter.as_ref().map_err(|error| anyhow::anyhow!("{error:#}"))?.clone())
            } else {
                None
            };
            let paint_errors = self.paint_errors.clone();
            let endpoints = [source, [source[2], source[0], source[1], source[3]]].map(|rgba| {
                LinearRgba::new(
                    LinearRgb::new(RgbSpace::DisplayP3, [rgba[0] as f64, rgba[1] as f64, rgba[2] as f64]).unwrap(),
                    rgba[3] as f64,
                )
                .unwrap()
            });
            let gradient = WideGradient::linear(
                [0., 0.],
                [1., 0.],
                endpoints,
                ColorInterpolation::Oklab,
                GamutMapping::ReduceChroma,
            )?;
            Ok(div()
                .absolute()
                .left(gpui::relative(left / 600.0))
                .top(px(top))
                .w(gpui::relative(0.6))
                .h(px(220.0))
                .opacity(opacity)
                .when(!wide, |view| view.bg(rgb(fallback)))
                .when(wide, |view| {
                    view.child(
                        canvas(
                            |_, _, _| (),
                            move |bounds, _, window, _| {
                                if route > 0
                                    && let Some(painter) = &painter
                                {
                                    let result = if route == 2 {
                                        painter.paint_gradient(window, bounds, px(0.), &gradient)
                                    } else {
                                        painter.paint_color(window, bounds, px(0.), &color)
                                    };
                                    if let Err(error) = result {
                                        paint_errors.borrow_mut().push(format!("P3 comparison route: {error:#}"));
                                        eprintln!("P3 comparison route: {error:#}");
                                    }
                                } else {
                                    window.paint_custom(
                                        CustomGeometry::RoundedRect { bounds, corner_radii: Default::default() },
                                        color.clone(),
                                    );
                                }
                            },
                        )
                        .size_full(),
                    )
                })
                .into_any_element())
        };
        let red = surface(25.0, 25.0, [1.0, 0.0, 0.0, 1.0], palette::RED_SRGB)?;
        let green = surface(200.0, 155.0, [0.0, 1.0, 0.0, 1.0], palette::GREEN_SRGB)?;
        let (first, second) = if self.reverse { (green, red) } else { (red, green) };
        let scene = div()
            .relative()
            .w_full()
            .debug_selector(|| "gamut-overlays-scene".into())
            .h(px(400.0))
            .overflow_hidden()
            .when(self.background != Background::Transparent, |scene| {
                scene.bg(rgb(if self.background == Background::Dark {
                    palette::DARK_STAGE
                } else {
                    palette::LIGHT_STAGE
                }))
            })
            .child(first)
            .when(self.overlay, |view| {
                view.child(
                    div()
                        .absolute()
                        .left(gpui::relative(0.15))
                        .top(px(110.0))
                        .w(gpui::relative(0.7))
                        .p_4()
                        .bg(rgba(palette::BETWEEN_OVERLAY))
                        .text_color(rgb(palette::DARK_TEXT))
                        .child("sRGB overlay between P3 surfaces"),
                )
            })
            .child(second)
            .when(self.overlay, |view| {
                view.child(
                    div()
                        .absolute()
                        .left(gpui::relative(0.5))
                        .right(gpui::relative(0.05))
                        .top(px(270.0))
                        .p_3()
                        .bg(rgba(palette::TOP_OVERLAY))
                        .text_color(rgb(palette::LIGHT_TEXT))
                        .child("sRGB overlay above both surfaces"),
                )
            });
        Ok(scene.into_any_element())
    }
}
