//! Fixed main-pass brush examples, independent of the legacy paint editor.
use gpui::{prelude::*, *};
use super::{palette::gallery as data, ShaderPage};
use xgpui_luma_look_radix::Look;
use crate::{
    screen::{Hero, ComparePane, LabScreen, PanelShell, TextRole, text},
};
fn stops(values: &[(f32, [f32; 4])]) -> Vec<BrushStop> {
    values.iter().map(|(offset, color)| BrushStop::new(*offset, *color)).collect()
}

use xgpui_shader::{BlendMode, Brush, BrushPoint, BrushStop, LayerEffect, Noise, painted};

pub fn gallery(lab: &super::ShaderPage, cx: &mut Context<super::ShaderPage>) -> impl IntoElement {
    let look = &lab.look;
    let colors = stops(&data::BRUSH_STOPS);
    let brushes = [
        ("Solid", Brush::solid(data::SOLID_BLUE).unwrap()),
        ("Linear", Brush::linear(60., colors.clone()).unwrap()),
        ("Radial", Brush::radial([0.4, 0.4], [0.65, 0.8], colors).unwrap()),
    ];
    let ring = Brush::linear(90., stops(&data::RING_STOPS)).unwrap();
    let material = Brush::linear(120., stops(&data::MATERIAL_STOPS)).unwrap();
    let typography = Brush::linear(90., stops(&data::TYPE_STOPS)).unwrap();
    let freeform = Brush::freeform(
        data::FREEFORM_POINTS
            .into_iter()
            .map(|(position, color)| BrushPoint::new(position, color))
            .collect(),
    )
    .unwrap();
    let mesh = Brush::mesh(data::MESH_COLORS).unwrap();
    let point_count = [4, 8, 16][lab.freeform_step as usize];
    let expanded = Brush::freeform(expanded_demo_points(point_count)).unwrap();
    let spatial = [
        ("Freeform · three color points", freeform),
        (
            match point_count {
                4 => "Freeform · four points",
                8 => "Freeform · eight points",
                _ => "Freeform · sixteen points",
            },
            expanded,
        ),
        ("Hermite mesh · regular 2×2", mesh),
    ];
    let procedural = [
        ("Simplex · 3 octaves", Noise::simplex(0.035, 3, 0.8)),
        ("Worley · F1", Noise::worley(0.045, 0.8)),
        ("Worley · F2", Noise::worley_f2(0.045, 0.8)),
        ("Blue-noise dither", Noise::dither(0.15)),
    ];
    let modes = [
        ("Normal", BlendMode::Normal),
        ("Multiply", BlendMode::Multiply),
        ("Screen", BlendMode::Screen),
        ("Overlay", BlendMode::Overlay),
        ("Color dodge", BlendMode::ColorDodge),
        ("Soft light", BlendMode::SoftLight),
        ("Linear burn", BlendMode::LinearBurn),
    ];
    let hero = div().flex().flex_wrap().gap_4().children(brushes.into_iter().map(|(label, brush)| {
        div()
            .flex_1()
            .min_w(px(200.))
            .flex()
            .flex_col()
            .gap_2()
            .child(text(look, TextRole::Name, label))
            .child(
                painted()
                    .w_full()
                    .h(px(200.))
                    .rounded(px(24.))
                    .border_2()
                    .bg_brush(brush)
                    .border_brush(ring.clone()),
            )
    }));
    let body = div()
        .w_full()
        .flex()
        .flex_col()
        .gap_3()
        .child(worley_demo(look))
        .child(blue_noise_demo(look))
        .child(path_brush_demo(lab, cx))
        .child(path_batch_demo(lab.direct_paths, look))
        .child(text(look, TextRole::Name, "Spatial brushes · backgrounds, borders, and glyphs"))
        .child(div().flex().flex_wrap().gap_4().children(spatial.into_iter().map(|(label, brush)| {
            div()
                .flex()
                .flex_col()
                .gap_3()
                .child(text(look, TextRole::Detail, label))
                .child(painted().w(px(320.)).h(px(160.)).rounded(px(24.)).bg_brush(brush.clone()))
                .child(
                    painted()
                        .w(px(320.))
                        .h(px(88.))
                        .rounded(px(24.))
                        .border_8()
                        .border_brush(brush.clone())
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(text(look, TextRole::Name, "One continuous border field")),
                )
                .child(
                    painted()
                        .w(px(320.))
                        .min_h(px(96.))
                        .map(|view| TextRole::Title.apply(look, view))
                        .text_brush(brush)
                        .child("Spatial color across glyphs"),
                )
        })))
        .child(text(look, TextRole::Name, "Brush-painted text · one continuous field across glyphs"))
        .child(
            painted()
                .w(px(640.))
                .max_w_full()
                .min_h(px(104.))
                .map(|view| TextRole::Title.apply(look, view))
                .text_brush(typography.clone())
                .child("Electric blue to vivid orange · wrapped typography"),
        )
        .child(
            painted()
                .w(px(640.))
                .max_w_full()
                .min_h(px(72.))
                .map(|view| TextRole::Title.apply(look, view))
                .text_brush(typography.clone().with_noise(Noise::simplex(0.08, 3, 0.6), BlendMode::Overlay).unwrap())
                .child("Textured glyphs · native emoji ✨"),
        )
        .child(text(look, TextRole::Name, "Procedural brushes · modulation of the same linear gradient"))
        .child(div().flex().flex_wrap().gap_4().children(procedural.into_iter().map(|(label, noise)| {
            painted()
                .w(px(208.))
                .h(px(120.))
                .rounded(px(24.))
                .bg_brush(material.clone().with_noise(noise, BlendMode::Overlay).unwrap())
                .flex()
                .items_center()
                .justify_center()
                .child(text(look, TextRole::Detail, label))
        })))
        .child(text(look, TextRole::Name, "Brush-local blends · same Simplex signal and strength"))
        .child(div().flex().flex_wrap().gap_4().children(modes.into_iter().map(|(label, mode)| {
            painted()
                .w(px(160.))
                .h(px(96.))
                .rounded(px(16.))
                .bg_brush(material.clone().with_noise(Noise::simplex(0.035, 3, 0.75), mode).unwrap())
                .flex()
                .items_center()
                .justify_center()
                .child(text(look, TextRole::Detail, label))
        })));
    LabScreen::new("shader-brushes", lab.look.clone()).crumb("xgpui-shader", "Brushes").title("Brushes")
        .lede("Compare color fields and textures across surfaces, borders, shapes and letters.")
        .hero(Hero::full(hero))
        .caption(format!("Solid, linear and radial · freeform {} points · {} paths", point_count, if lab.direct_paths {"direct"} else {"native"}))
        .control(lab.gallery_button("freeform-points", "Freeform points", cx))
        .control(lab.gallery_button("path-route", "Direct paths", cx))
        .section_panel(PanelShell::new("Brush comparisons").child(body))
        .note("Freeform points changes the larger spatial field. Direct paths uses prepared brush geometry; the native route paints the same paths through GPUI coverage. The noisy samples deliberately exaggerate grain to make it visible.")
        .credits("Adapted from xgpui paint-lab/brushes.rs. The blue-noise preview enlarges the exact bundled 16×16 shader rank tile, not a regenerated seed. Path batching labels describe the planned workload, not measured GPU submissions.")
}

fn backdrop_demo(lab: &super::ShaderPage, _cx: &mut Context<super::ShaderPage>) -> impl IntoElement {
    let look = &lab.look;
    let changed = lab.backdrop_changed;
    let moved = lab.backdrop_moved;
    let disabled = lab.backdrop_disabled;
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(text(look, TextRole::Name, "Backdrop blur · interactive visual check"))
        .child(text(
            look,
            TextRole::Name,
            "Swap: the frosted panel must change color. Move: the marker stays sharp. Toggle blur to compare.",
        ))
        .child(
            div()
                .relative()
                .w(px(360.))
                .h(px(144.))
                .child(
                    canvas(
                        |_, _, _| (),
                        move |bounds, (), window, _| {
                            for i in 0..45 {
                                let color = match (changed, i % 2 == 0) {
                                    (false, true) => rgb(data::BACKDROP_STRIPES[0][0]),
                                    (false, false) => rgb(data::BACKDROP_STRIPES[0][1]),
                                    (true, true) => rgb(data::BACKDROP_STRIPES[1][0]),
                                    (true, false) => rgb(data::BACKDROP_STRIPES[1][1]),
                                };
                                window.paint_quad(fill(
                                    Bounds::new(
                                        bounds.origin + point(px(i as f32 * 8.), px(0.)),
                                        size(px(8.), bounds.size.height),
                                    ),
                                    color,
                                ));
                            }
                        },
                    )
                    .absolute()
                    .size_full(),
                )
                .child(
                    painted()
                        .absolute()
                        .left(px(96.))
                        .top(px(16.))
                        .w(px(248.))
                        .h(px(112.))
                        .rounded(px(16.))
                        .backdrop_blur(xgpui_shader::BackdropBlur::new(px(if disabled { 0. } else { 16. })).unwrap())
                        .bg(gpui::rgba(data::FROST_FILL))
                        .border_1()
                        .border_color(gpui::rgba(data::FROST_BORDER))
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .text_color(white())
                        .child(div().map(|view| TextRole::Name.apply(look, view)).child(text(
                            look,
                            TextRole::Name,
                            "Frosted backdrop",
                        )))
                        .child(text(look, TextRole::Name, "Foreground stays sharp")),
                )
                .child(
                    div()
                        .absolute()
                        .left(px(if moved { 302. } else { 110. }))
                        .top(px(42.))
                        .w(px(18.))
                        .h(px(60.))
                        .bg(white()),
                ),
        )
}

fn group_opacity_demo(look: &Look) -> impl IntoElement {
    let shapes = || {
        div()
            .relative()
            .w(px(176.))
            .h(px(112.))
            .child(
                div()
                    .absolute()
                    .left(px(16.))
                    .top(px(16.))
                    .w(px(96.))
                    .h(px(72.))
                    .rounded(px(12.))
                    .bg(rgb(data::OPACITY_SHAPES[0])),
            )
            .child(
                div()
                    .absolute()
                    .left(px(64.))
                    .top(px(32.))
                    .w(px(96.))
                    .h(px(64.))
                    .rounded(px(12.))
                    .bg(rgb(data::OPACITY_SHAPES[1])),
            )
    };
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(text(look, TextRole::Name, "Group opacity · compare the overlap"))
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_4()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(text(look, TextRole::Name, "Ordinary opacity: 50% per shape"))
                        .child(div().w(px(176.)).h(px(112.)).bg(white()).child(div().opacity(0.5).child(shapes()))),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(text(look, TextRole::Name, "Layer effect: 50% after overlap"))
                        .child(
                            div().w(px(176.)).h(px(112.)).bg(white()).child(
                                painted()
                                    .w(px(176.))
                                    .h(px(112.))
                                    .layer_effect(LayerEffect::opacity(0.5).unwrap())
                                    .child(shapes()),
                            ),
                        ),
                ),
        )
        .child(div().map(|view| TextRole::Fine.apply(look, view)).child(text(
            look,
            TextRole::Name,
            "The blue shape fully covers red before the captured group fades.",
        )))
}

fn group_filter_demo(lab: &super::ShaderPage, _cx: &mut Context<super::ShaderPage>) -> impl IntoElement {
    let look = &lab.look;
    let modes = [
        ("Normal", BlendMode::Normal),
        ("Multiply", BlendMode::Multiply),
        ("Screen", BlendMode::Screen),
        ("Overlay", BlendMode::Overlay),
        ("Color dodge", BlendMode::ColorDodge),
        ("Soft light", BlendMode::SoftLight),
        ("Linear burn", BlendMode::LinearBurn),
    ];
    let (blend_label, blend_mode) = modes[lab.layer_blend_step];
    let blur_radius = [0., 4., 8., 16.][lab.layer_blur_step];
    let blur_quality = if lab.layer_blur_full_resolution {
        LayerBlurQuality::FullResolution
    } else {
        LayerBlurQuality::Adaptive
    };

    let (label, effect) = match lab.layer_filter_step {
        0 => ("Invert 100%", LayerEffect::invert(1.).unwrap()),
        1 => ("Invert 50%", LayerEffect::invert(0.5).unwrap()),
        _ => ("Saturation 2", LayerEffect::saturation(2.).unwrap()),
    };
    let sample = |continuous_background: bool| {
        div()
            .relative()
            .w(px(176.))
            .h(px(112.))
            .bg(rgb(data::FILTER_BACKGROUND))
            .children(continuous_background.then(|| {
                div()
                    .absolute()
                    .left(px(-64.))
                    .top(px(-64.))
                    .w(px(304.))
                    .h(px(240.))
                    .bg(rgb(data::FILTER_BACKGROUND))
            }))
            .child(
                div()
                    .absolute()
                    .left(px(12.))
                    .top(px(12.))
                    .w(px(100.))
                    .h(px(76.))
                    .rounded(px(12.))
                    .bg(rgb(data::FILTER_SHAPES[0])),
            )
            .child(
                div()
                    .absolute()
                    .left(px(64.))
                    .top(px(32.))
                    .w(px(100.))
                    .h(px(68.))
                    .rounded(px(12.))
                    .bg(rgb(data::FILTER_SHAPES[1])),
            )
            .child(
                div()
                    .absolute()
                    .left(px(18.))
                    .top(px(42.))
                    .text_color(rgb(data::FILTER_GLYPHS))
                    .map(|view| TextRole::Name.apply(look, view))
                    .child(text(look, TextRole::Name, "Color + text")),
            )
    };
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(text(look, TextRole::Name, "Captured color filters · the whole card and text"))
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_4()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(text(look, TextRole::Name, "Original · saturation 1"))
                        .child(sample(false)),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(text(look, TextRole::Name, "Grayscale · saturation 0"))
                        .child(
                            painted()
                                .w(px(176.))
                                .h(px(112.))
                                .layer_effect(LayerEffect::grayscale(1.).unwrap())
                                .child(sample(false)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(text(look, TextRole::Caption, label))
                        .child(painted().w(px(176.)).h(px(112.)).layer_effect(effect).child(sample(false))),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(text(look, TextRole::Caption, format!("Blend: {blend_label}")))
                        .child(text(look, TextRole::Caption, format!("Subtree blur: {blur_radius}px")))
                        .child(
                            div()
                                .relative()
                                .w(px(176.))
                                .h(px(112.))
                                .bg(rgb(data::BLEND_BACKDROP[0]))
                                .child(
                                    div()
                                        .absolute()
                                        .left(px(88.))
                                        .top_0()
                                        .w(px(88.))
                                        .h_full()
                                        .bg(rgb(data::BLEND_BACKDROP[1])),
                                )
                                .child(
                                    painted()
                                        .relative()
                                        .w(px(176.))
                                        .h(px(112.))
                                        .layer_effect(
                                            LayerEffect::blend(blend_mode)
                                                .unwrap()
                                                .with_blur(px(blur_radius))
                                                .unwrap()
                                                .with_blur_quality(blur_quality),
                                        )
                                        .child(sample(true)),
                                ),
                        ),
                ),
        )
}

fn path_brush_demo(lab: &super::ShaderPage, _cx: &mut Context<super::ShaderPage>) -> impl IntoElement {
    let look = &lab.look;
    let direct = lab.direct_paths;
    let gradient = Brush::linear(65., stops(&data::PATH_STOPS)).unwrap();
    let mesh = Brush::mesh(data::PATH_MESH)
        .unwrap()
        .with_noise(Noise::simplex(0.07, 2, 0.4), BlendMode::Overlay)
        .unwrap();
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(text(look, TextRole::Name, "Vector brushes · curved fill, a hole, and a stroked path"))
        .child(div().flex().flex_wrap().gap_4().children([(false, gradient), (true, mesh)].into_iter().map(
            |(stroke, brush)| {
                let mut p = if stroke {
                    PathBuilder::stroke(px(12.))
                } else {
                    PathBuilder::fill()
                };
                p.move_to(point(px(16.), px(26.)));
                p.cubic_bezier_to(point(px(188.), px(32.)), point(px(62.), px(-4.)), point(px(162.), px(4.)));
                p.curve_to(point(px(166.), px(114.)), point(px(208.), px(112.)));
                p.line_to(point(px(38.), px(108.)));
                p.close();
                if !stroke {
                    p.move_to(point(px(78.), px(48.)));
                    p.line_to(point(px(78.), px(82.)));
                    p.line_to(point(px(130.), px(82.)));
                    p.line_to(point(px(130.), px(48.)));
                    p.close();
                }

                let path = p.build().unwrap();
                let prepared = brush.prepare_path(&path).unwrap();
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(if stroke {
                        "Noisy mesh stroke"
                    } else {
                        "Blue/orange fill with a hole"
                    })
                    .child(
                        canvas(
                            |_, _, _| (),
                            move |bounds, (), window, _| {
                                if direct {
                                    prepared.paint(window, bounds.origin).unwrap();
                                } else {
                                    let mut path = path.clone();
                                    path.bounds.origin += bounds.origin;
                                    for vertex in &mut path.vertices {
                                        vertex.xy_position += bounds.origin;
                                    }
                                    brush.paint_path_native(window, path).unwrap();
                                }
                            },
                        )
                        .w(px(224.))
                        .h(px(144.)),
                    )
            },
        )))
}

/// Keep the normal, subtle material beside an intentionally exaggerated signal.
/// Enlarged cells read the exact asset embedded in the GPU shader, not a new seed.
fn blue_noise_demo(look: &Look) -> impl IntoElement {
    let gradient = Brush::linear(90., stops(&data::NOISE_RAMP)).unwrap();
    let swatches = [
        ("No noise", gradient.clone()),
        ("Visible grain · strength 0.6", gradient.with_noise(Noise::dither(0.6), BlendMode::Overlay).unwrap()),
        (
            "Noise only · strength 1.0",
            Brush::solid(data::NOISE_BASE).unwrap().with_noise(Noise::dither(1.), BlendMode::Normal).unwrap(),
        ),
    ];
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(text(look, TextRole::Name, "Blue-noise visual check · exaggerated for visibility"))
        .child(div().flex().flex_wrap().gap_4().children(swatches.into_iter().map(|(label, brush)| {
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(text(look, TextRole::Detail, label))
                .child(painted().w(px(200.)).h(px(144.)).bg_brush(brush))
        })))
        .child(text(
            look,
            TextRole::Name,
            "Below: one 16×16 tile enlarged 8×. The actual shader uses one sample per physical pixel.",
        ))
        .child(div().flex().flex_wrap().gap_4().children([false, true].into_iter().map(|threshold| {
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(if threshold {
                    "50% threshold · dispersed dots"
                } else {
                    "Full grayscale rank tile"
                })
                .child(
                    canvas(
                        |_, _, _| (),
                        move |bounds, (), window, _| {
                            let ranks = include_bytes!("../../assets/samples/shader/blue_noise.bin");
                            for (i, rank) in ranks.iter().enumerate() {
                                let value = if threshold {
                                    if *rank < 128 { 0 } else { 255 }
                                } else {
                                    *rank as u32
                                };
                                window.paint_quad(fill(
                                    Bounds::new(
                                        bounds.origin + point(px((i % 16) as f32 * 8.), px((i / 16) as f32 * 8.)),
                                        size(px(8.), px(8.)),
                                    ),
                                    rgb(value * 0x010101),
                                ));
                            }
                        },
                    )
                    .w(px(128.))
                    .h(px(128.)),
                )
        })))
}

fn worley_demo(look: &Look) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(text(look, TextRole::Name, "Worley cellular noise · same features, different distance"))
        .child(text(
            look,
            TextRole::Name,
            "F1 measures the closest point; F2 measures the second closest. Both use the same scale and seed.",
        ))
        .child(
            div().flex().flex_wrap().gap_4().children(
                [("F1 · closest", Noise::worley(0.035, 1.)), ("F2 · second closest", Noise::worley_f2(0.035, 1.))]
                    .into_iter()
                    .map(|(label, noise)| {
                        div().flex().flex_col().gap_2().child(text(look, TextRole::Detail, label)).child(
                            painted().w(px(288.)).h(px(176.)).bg_brush(
                                Brush::solid(data::NOISE_BASE).unwrap().with_noise(noise, BlendMode::Normal).unwrap(),
                            ),
                        )
                    }),
            ),
        )
}

fn expanded_demo_points(count: usize) -> Vec<BrushPoint> {
    let colors = data::EXPANDED_COLORS;
    (0..count)
        .map(|i| {
            let position = if count == 8 {
                let angle = i as f32 * std::f32::consts::TAU / 8.;
                [0.5 + 0.42 * angle.cos(), 0.5 + 0.42 * angle.sin()]
            } else {
                let side = if count == 4 { 2 } else { 4 };
                [
                    0.08 + (i % side) as f32 * 0.84 / (side - 1) as f32,
                    0.08 + (i / side) as f32 * 0.84 / (side - 1) as f32,
                ]
            };
            BrushPoint::new(position, colors[i % 8])
        })
        .collect()
}

fn path_batch_demo(direct: bool, look: &Look) -> impl IntoElement {
    let brush = Brush::linear(45., stops(&data::BATCH_STOPS)).unwrap();
    let mut path = PathBuilder::fill();
    path.move_to(point(px(4.), px(4.)));
    path.cubic_bezier_to(point(px(40.), px(36.)), point(px(48.), px(0.)), point(px(0.), px(44.)));
    path.line_to(point(px(4.), px(36.)));
    path.close();
    let path = path.build().unwrap();
    let prepared = brush.prepare_path(&path).unwrap();
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(text(look, TextRole::Name, "Path batching · 24 separate curved shapes"))
        .child(if direct {
            "Direct: 24 shapes · 0 coverage passes"
        } else {
            "Native: 24 paths · 1 planned coverage batch"
        })
        .child(
            canvas(
                |_, _, _| (),
                move |bounds, (), window, _| {
                    for index in 0..24 {
                        let offset = bounds.origin + point(px((index % 8) as f32 * 56.), px((index / 8) as f32 * 48.));
                        if direct {
                            prepared.paint(window, offset).unwrap();
                        } else {
                            let mut path = path.clone();
                            path.bounds.origin += offset;
                            for vertex in &mut path.vertices {
                                vertex.xy_position += offset;
                            }
                            brush.paint_path_native(window, path).unwrap();
                        }
                    }
                },
            )
            .w(px(448.))
            .h(px(144.)),
        )
}

pub fn layers(lab: &ShaderPage, cx: &mut Context<ShaderPage>) -> impl IntoElement {
    let look = &lab.look;
    let mut screen = LabScreen::new("shader-layers", lab.look.clone())
        .crumb("xgpui-shader", "Layer Effects")
        .title("Layer Effects")
        .lede("Compare effects on a group of shapes with effects applied to each shape separately.")
        .hero(Hero::stage(ComparePane::new("Captured color filters", group_filter_demo(lab, cx))))
        .caption(format!(
            "{} · {} blend · {} px subtree blur · {} quality",
            ["Invert 100%", "Invert 50%", "Saturation 2"][lab.layer_filter_step],
            ["Normal", "Multiply", "Screen", "Overlay", "Color dodge", "Soft light", "Linear burn"]
                [lab.layer_blend_step],
            [0, 4, 8, 16][lab.layer_blur_step],
            if lab.layer_blur_full_resolution {
                "full"
            } else {
                "adaptive"
            }
        ));
    for id in [
        "layer-filter-cycle",
        "layer-blend-cycle",
        "layer-blur-cycle",
        "layer-blur-quality",
        "backdrop-toggle",
    ] {
        screen = screen.control(lab.gallery_button(id, "", cx));
    }
    for (id, label) in [("backdrop-source", "Swap stripe colors"), ("backdrop-foreground", "Move marker")] {
        screen = screen.action(lab.gallery_button(id, label, cx));
    }
    screen.section_panel(PanelShell::new("Backdrop blur").child(backdrop_demo(lab,cx)))
        .section_panel(PanelShell::new("Group opacity").child(group_opacity_demo(look)))
        .note("The filter and blur controls act on the captured card and its text. Backdrop blur samples only the stripes behind the frosted panel; moving the foreground marker leaves it sharp. Compare the overlap before and after group opacity.")
        .credits("Adapted from xgpui paint-lab/brushes.rs. The original sample geometry, blend modes, blur radii and adaptive/full-resolution quality settings are retained. Captured effects use bounded SDR targets; no GPU timing is measured.")
}
