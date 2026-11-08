//! Upstream silk-ribbon composition, with retained Luma controls.
use gpui::{prelude::*, *};
use std::{sync::Arc, time::Instant, rc::Rc, cell::RefCell};
use xgpui_luma::{
    controls::{button::Button, slider::Slider},
    prelude::HasPresenter,
};
use xgpui_luma_look_radix::{self as radix, Look};
use crate::theme::ColorVizLookExt;
use xgpui_shader::{BlendMode, Brush, BrushPoint, BrushStop, Noise, painted};

pub struct PastelWaves {
    look: Arc<Look>,
    scene: Rc<RefCell<WaveScene>>,
    buttons: Vec<Entity<Button>>,
    scrub: Slider,
    active: bool,
    playing: bool,
    grain: bool,
    phase: f32,
    direction: f32,
    updated: Instant,
}
impl PastelWaves {
    pub fn new(look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let buttons = ["Animate / pause", "Reverse", "Grain on / off", "Reset"]
            .into_iter()
            .enumerate()
            .map(|(i, label)| {
                radix::Button::new(format!("waves-{i}"))
                    .look(&look)
                    .label(label)
                    .on_click(cx, move |this: &mut Self, cx| {
                        match i {
                            0 => {
                                this.playing = !this.playing;
                                this.scrub.update(cx, |slider, cx| slider.set_value(this.phase, cx));
                            }
                            1 => this.direction = -this.direction,
                            2 => this.grain = !this.grain,
                            _ => {
                                this.phase = 0.;
                                this.direction = 1.;
                                this.grain = true;
                                this.playing = false;
                                this.scrub.update(cx, |slider, cx| slider.set_value(0., cx));
                            }
                        }
                        this.updated = Instant::now();
                        cx.notify();
                    })
                    .spawn(cx)
            })
            .collect();
        let scrub = radix::Slider::new("waves-phase")
            .look(&look)
            .range(0.0..std::f32::consts::PI)
            .step(0.001)
            .value(0.)
            .on_value_changed(cx, |this: &mut Self, _, value, cx| {
                this.phase = value;
                this.playing = false;
                cx.notify();
            })
            .spawn(cx);
        Self {
            look,
            scene: Rc::new(RefCell::new(WaveScene::new(true))),
            buttons,
            scrub,
            active: false,
            playing: false,
            grain: true,
            phase: 0.,
            direction: 1.,
            updated: Instant::now(),
        }
    }
    #[cfg(all(test, feature = "test-support"))]
    pub(crate) fn play_button(&self) -> Entity<Button> {
        self.buttons[0].clone()
    }
    #[cfg(all(test, feature = "test-support"))]
    pub(crate) fn is_playing(&self) -> bool {
        self.playing
    }
    pub fn activate(&mut self, cx: &mut Context<Self>) {
        self.scrub.update(cx, |slider, cx| slider.set_value(self.phase, cx));
        self.active = true;
        self.updated = Instant::now();
        cx.notify();
    }
    pub fn suspend(&mut self) {
        self.active = false;
        self.playing = false;
        self.updated = Instant::now();
    }
}
impl Render for PastelWaves {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let now = Instant::now();
        if self.active && self.playing && !cx.reduce_motion() {
            self.phase += (now.duration_since(self.updated).as_secs_f32().min(0.05)) * 0.22 * self.direction;
            let end = std::f32::consts::PI;
            if self.phase > end {
                self.phase = 2. * end - self.phase;
                self.direction = -1.;
            } else if self.phase < 0. {
                self.phase = -self.phase;
                self.direction = 1.;
            }

            window.request_animation_frame();
        }
        self.updated = now;
        let chrome = self.look.chrome();
        div()
            .id("pastel-waves-pane")
            .overflow_y_scroll()
            .p_6()
            .gap_4()
            .bg(chrome.panel_background)
            .text_color(chrome.body_text)
            .size_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(div().flex().flex_wrap().gap_2().children(self.buttons.iter().cloned()))
                    .child(
                        div()
                            .flex()
                            .gap_3()
                            .items_center()
                            .child("Scrub ribbon motion")
                            .child(div().w(px(260.)).child(self.scrub.clone())),
                    )
                    .child(format!(
                        "Pastel Waves · {} · grain {} · phase {:.2}",
                        if cx.reduce_motion() {
                            "reduced motion"
                        } else if self.playing {
                            "playing"
                        } else {
                            "paused"
                        },
                        self.grain,
                        self.phase
                    )),
            )
            .child(
                div()
                    .relative()
                    .w(px(900.))
                    .max_w_full()
                    .aspect_ratio(16. / 9.)
                    .flex_shrink_0()
                    .overflow_hidden()
                    .child(
                        composition(
                            std::f32::consts::PI * 0.5 * (1. - self.phase.cos()),
                            self.grain,
                            self.scene.clone(),
                        )
                        .absolute()
                        .size_full(),
                    ),
            )
    }
}

// Brush parameters are independent of animation phase. Retain their programs
// instead of searching/rebuilding the specialization cache on every frame.
#[derive(Default)]
struct Materials {
    brushes: Vec<Brush>,
    cursor: usize,
    created: usize,
}
impl Materials {
    fn get(&mut self, create: impl FnOnce() -> Brush) -> Brush {
        if self.cursor == self.brushes.len() {
            self.brushes.push(create());
            self.created += 1;
        }
        let brush = self.brushes[self.cursor].clone();
        self.cursor += 1;
        brush
    }
}
struct Geometry {
    bounds: Bounds<Pixels>,
    phase: f32,
    grain: bool,
    paths: Vec<(Brush, Path<Pixels>)>,
}
struct WaveScene {
    background: Brush,
    materials: [Materials; 2],
    geometry: Option<Geometry>,
    retain: bool,
    geometry_builds: usize,
}
impl WaveScene {
    fn new(retain: bool) -> Self {
        let background = Brush::freeform(vec![
            BrushPoint::new([0.03, 0.08], [0.95, 0.90, 0.99, 1.]),
            BrushPoint::new([0.58, 0.02], [1., 0.98, 0.96, 1.]),
            BrushPoint::new([0.98, 0.20], [1., 0.95, 0.87, 1.]),
            BrushPoint::new([0.07, 0.96], [0.73, 0.91, 1., 1.]),
            BrushPoint::new([0.96, 0.94], [0.95, 0.76, 0.96, 1.]),
        ])
        .unwrap();
        Self { background, materials: Default::default(), geometry: None, retain, geometry_builds: 0 }
    }
    fn paint(&mut self, bounds: Bounds<Pixels>, phase: f32, grain: bool, window: &mut Window) {
        let materials = &mut self.materials[usize::from(grain)];
        materials.cursor = 0;
        if !self.retain {
            materials.brushes.clear();
            self.geometry = None;
        }
        paint_background_fields(bounds, window, materials);
        if self.geometry.as_ref().is_none_or(|g| g.bounds != bounds || g.phase != phase || g.grain != grain) {
            self.geometry =
                Some(Geometry { bounds, phase, grain, paths: ribbon_paths(bounds, phase, grain, materials) });
            self.geometry_builds += 1;
        }
        for (brush, path) in &self.geometry.as_ref().unwrap().paths {
            brush.paint_path_native(window, path.clone()).unwrap();
        }
    }
}
fn composition(phase: f32, grain: bool, scene: Rc<RefCell<WaveScene>>) -> impl IntoElement + Styled {
    let background = scene.borrow().background.clone();
    painted().relative().w_full().overflow_hidden().bg_brush(background).child(
        canvas(
            |_, _, _| (),
            move |bounds, (), window, _| {
                window.with_content_mask(Some(ContentMask { bounds }), |window| {
                    scene.borrow_mut().paint(bounds, phase, grain, window)
                });
            },
        )
        .size_full(),
    )
}

// Fixed, overlapping veils make the pale background carry color even above
// the ribbons. Each radial field fades completely into the underlying surface.
fn paint_background_fields(bounds: Bounds<Pixels>, window: &mut Window, materials: &mut Materials) {
    let fields = [
        ([0.09, 0.13], [0.42, 0.48], [0.82, 0.68, 0.97], 0.20),
        ([0.29, 0.32], [0.37, 0.30], [0.95, 0.72, 0.86], 0.18),
        ([0.44, 0.06], [0.30, 0.32], [1., 0.84, 0.77], 0.16),
        ([0.76, 0.18], [0.42, 0.43], [1., 0.92, 0.70], 0.23),
        ([0.02, 0.48], [0.36, 0.35], [0.69, 0.80, 1.], 0.19),
        ([0.54, 0.50], [0.42, 0.29], [0.96, 0.70, 0.90], 0.17),
        ([0.87, 0.58], [0.33, 0.39], [1., 0.76, 0.84], 0.18),
        ([0.28, 0.89], [0.51, 0.29], [0.59, 0.88, 1.], 0.23),
        ([0.73, 0.92], [0.40, 0.37], [0.85, 0.68, 0.98], 0.20),
    ];
    for (center, radii, color, alpha) in fields {
        materials
            .get(|| {
                Brush::radial(
                    center,
                    radii,
                    vec![
                        BrushStop::new(0., [color[0], color[1], color[2], alpha]),
                        BrushStop::new(0.48, [color[0], color[1], color[2], alpha * 0.45]),
                        BrushStop::new(1., [color[0], color[1], color[2], 0.]),
                    ],
                )
                .unwrap()
            })
            .paint_background(window, bounds, Corners::default())
            .unwrap();
    }
}

// Seven samples across each boundary. Cubic interpolation gives the ribbons
// independent crests and folds instead of a stack of parallel sine waves.
type Edge = [f32; 7];
struct Ribbon {
    top: Edge,
    bottom: Edge,
    color: [f32; 3],
    alpha: f32,
}
const RIBBONS: &[Ribbon] = &[
    Ribbon {
        top: [0.65, 0.47, 0.54, 0.59, 0.42, 0.34, 0.45],
        bottom: [0.82, 0.62, 0.72, 0.70, 0.64, 0.52, 0.61],
        color: [0.65, 0.69, 0.94],
        alpha: 0.21,
    },
    Ribbon {
        top: [0.72, 0.59, 0.65, 0.49, 0.33, 0.27, 0.44],
        bottom: [0.88, 0.79, 0.83, 0.64, 0.49, 0.49, 0.51],
        color: [0.92, 0.47, 0.72],
        alpha: 0.23,
    },
    Ribbon {
        top: [0.69, 0.67, 0.64, 0.24, 0.29, 0.47, 0.52],
        bottom: [0.88, 0.83, 0.79, 0.55, 0.41, 0.56, 0.48],
        color: [0.80, 0.40, 0.78],
        alpha: 0.30,
    },
    Ribbon {
        top: [0.57, 0.44, 0.58, 0.63, 0.46, 0.62, 0.48],
        bottom: [0.77, 0.66, 0.77, 0.78, 0.61, 0.76, 0.66],
        color: [0.58, 0.66, 0.94],
        alpha: 0.26,
    },
    Ribbon {
        top: [0.53, 0.66, 0.72, 0.77, 0.53, 0.68, 0.76],
        bottom: [0.82, 0.84, 0.95, 0.98, 0.74, 0.84, 0.92],
        color: [0.69, 0.49, 0.87],
        alpha: 0.26,
    },
    Ribbon {
        top: [0.78, 0.63, 0.72, 0.81, 0.59, 0.68, 0.68],
        bottom: [0.90, 0.78, 0.83, 0.94, 0.73, 0.80, 0.79],
        color: [0.33, 0.63, 0.91],
        alpha: 0.55,
    },
    Ribbon {
        top: [0.89, 0.69, 0.66, 0.77, 0.44, 0.59, 0.50],
        bottom: [1.02, 0.82, 0.84, 0.79, 0.55, 0.68, 0.63],
        color: [0.49, 0.40, 0.77],
        alpha: 0.55,
    },
    Ribbon {
        top: [0.87, 0.66, 0.73, 0.80, 0.66, 0.61, 0.68],
        bottom: [1.03, 0.78, 0.84, 0.89, 0.77, 0.74, 0.76],
        color: [0.64, 0.38, 0.80],
        alpha: 0.27,
    },
    Ribbon {
        top: [0.96, 0.73, 0.66, 0.79, 0.64, 0.69, 0.45],
        bottom: [1.09, 0.82, 0.81, 0.83, 0.69, 0.78, 0.58],
        color: [0.41, 0.79, 0.95],
        alpha: 0.40,
    },
];
fn edge_y(edge: Edge, x: f32) -> f32 {
    let t = x.clamp(0., 1.) * 6.;
    let i = (t.floor() as usize).min(5);
    let t = t - i as f32;
    let a = edge[i.saturating_sub(1)];
    let b = edge[i];
    let c = edge[i + 1];
    let d = edge[(i + 2).min(6)];
    0.5 * ((2. * b) + (-a + c) * t + (2. * a - 5. * b + 4. * c - d) * t * t + (-a + 3. * b - 3. * c + d) * t * t * t)
}
fn ribbon_paths(
    bounds: Bounds<Pixels>,
    phase: f32,
    grain: bool,
    materials: &mut Materials,
) -> Vec<(Brush, Path<Pixels>)> {
    let mut paths = Vec::new();
    for (index, ribbon) in RIBBONS.iter().enumerate() {
        let position = |x: f32, y: f32| {
            let motion = 0.012 * ((phase + x * 5. + index as f32 * 0.7).sin() - (x * 5. + index as f32 * 0.7).sin());
            bounds.origin + point(bounds.size.width * x, bounds.size.height * (y + motion))
        };
        // Keep each ribbon's translucent gradient associated with its geometry.
        let color = ribbon.color;
        let brush = materials.get(|| {
            let mut brush = Brush::mesh([
                [color[0].min(0.75), (color[1] + 0.14).min(0.90), (color[2] + 0.10).min(1.), ribbon.alpha * 0.40],
                [(color[0] + 0.25).min(1.), color[1] * 0.82, (color[2] + 0.03).min(1.), ribbon.alpha * 0.65],
                [color[0] * 0.65, color[1] * 0.93, color[2], ribbon.alpha],
                [(color[0] + 0.13).min(1.), color[1] * 0.67, color[2] * 0.94, ribbon.alpha * 0.90],
            ])
            .unwrap();
            if grain {
                brush = brush.with_noise(Noise::dither(0.035), BlendMode::SoftLight).unwrap();
            }
            brush
        });
        // Native coverage reuses brush programs as the geometry animates;
        // specializing direct shaders per moving ribbon would cause compilation churn.
        let path = |fraction_a: f32, fraction_b: f32| {
            let boundary = |x: f32, fraction: f32| {
                let top = edge_y(ribbon.top, x);
                let bottom = edge_y(ribbon.bottom, x);
                position(x, top + (bottom - top) * fraction)
            };
            let mut p = PathBuilder::fill();
            p.move_to(boundary(0., fraction_a));
            for step in 1..=256 {
                p.line_to(boundary(step as f32 / 256., fraction_a));
            }
            for step in (0..=256).rev() {
                p.line_to(boundary(step as f32 / 256., fraction_b));
            }
            p.close();
            p.build().unwrap()
        };
        paths.push((brush, path(0., 1.)));
        // Build a separate variable-width strip along the actual edge. Offsets
        // follow its screen-space normal so steep folds do not appear thinner.
        if index >= 5 {
            let crest = [0.18, 0.67, 0.69, 0.35][index - 5];
            let strength = |x: f32| {
                let e = 0.008;
                let bend = (edge_y(ribbon.top, x + e) - 2. * edge_y(ribbon.top, x) + edge_y(ribbon.top, x - e)).abs()
                    / (e * e);
                let curvature = bend / (bend + 22.);
                let fold = (-0.5 * ((x - crest) / 0.105).powi(2)).exp();
                (0.68 * fold + 0.32 * curvature).clamp(0., 1.)
            };
            // Overlapping translucent veils follow the ribbon's own cross-section.
            // Their accumulated opacity falls smoothly from the rim into the fill,
            // unlike a screen-space gradient or a blurred outline outside the sheet.
            for layer in (1..=24).rev() {
                let t = layer as f32 / 24.;
                let shade_edge = |x: f32| {
                    let top = edge_y(ribbon.top, x);
                    let thickness = (edge_y(ribbon.bottom, x) - top).max(0.);
                    let depth = (0.12 + 0.43 * strength(x)) * t;
                    position(x, top + thickness * depth)
                };
                let mut veil = PathBuilder::fill();
                veil.move_to(position(0., edge_y(ribbon.top, 0.)));
                for step in 1..=256 {
                    let x = step as f32 / 256.;
                    veil.line_to(position(x, edge_y(ribbon.top, x)));
                }
                for step in (0..=256).rev() {
                    veil.line_to(shade_edge(step as f32 / 256.));
                }
                veil.close();
                // Difference of the desired exponential profile yields each
                // veil's contribution; the widest, faintest layers draw first.
                let profile = |v: f32| ((-3.5 * v).exp() - (-3.5_f32).exp()) / (1. - (-3.5_f32).exp());
                let weight = profile((layer - 1) as f32 / 24.) - profile(t);
                let points = (0..=12)
                    .map(|step| {
                        let x = step as f32 / 12.;
                        let dark = strength(x);
                        let cyan = [0.20, 0.43, 0.70];
                        let violet = [0.43, 0.19, 0.60];
                        let color: [f32; 3] = std::array::from_fn(|i| cyan[i] * (1. - x) + violet[i] * x);
                        // Optical-depth accumulation keeps the center rich while
                        // the last layers disappear into the translucent material.
                        let alpha = 1. - (-weight * (0.32 + 0.95 * dark)).exp();
                        BrushPoint::new([x, 0.5], [color[0], color[1], color[2], alpha])
                    })
                    .collect();
                paths.push((materials.get(|| Brush::freeform(points).unwrap()), veil.build().unwrap()));
            }
            let edge = |x: f32, side: f32| {
                let e = 0.001;
                let lo = (x - e).max(0.);
                let hi = (x + e).min(1.);
                let a = position(lo, edge_y(ribbon.top, lo));
                let b = position(hi, edge_y(ribbon.top, hi));
                let dx = f32::from(b.x - a.x);
                let dy = f32::from(b.y - a.y);
                let length = dx.hypot(dy).max(0.001);
                let width = 0.35 + 2.25 * strength(x);
                position(x, edge_y(ribbon.top, x))
                    + point(px(-dy / length * width * side), px(dx / length * width * side))
            };
            let mut rim = PathBuilder::fill();
            rim.move_to(edge(0., -0.2));
            for step in 1..=256 {
                rim.line_to(edge(step as f32 / 256., -0.2));
            }
            for step in (0..=256).rev() {
                rim.line_to(edge(step as f32 / 256., 0.8));
            }
            rim.close();
            // Opacity and pigment vary independently of width. Closely spaced
            // field points keep the strongest dark accents local to each fold.
            let colors = (0..=12)
                .map(|step| {
                    let x = step as f32 / 12.;
                    let dark = strength(x);
                    let cyan = [0.18, 0.46, 0.66];
                    let rose = [0.63, 0.20, 0.53];
                    let color: [f32; 3] =
                        std::array::from_fn(|i| (cyan[i] * (1. - x) + rose[i] * x) * (1. - dark * 0.28));
                    BrushPoint::new([x, 0.5], [color[0], color[1], color[2], 0.12 + 0.65 * dark])
                })
                .collect();
            paths.push((materials.get(|| Brush::freeform(colors).unwrap()), rim.build().unwrap()));
        }
        let lip = materials.get(|| Brush::solid([0.98, 0.98, 1., if index >= 5 { 0.10 } else { 0.20 }]).unwrap());
        paths.push((lip, path(0.006, 0.016)));
        if grain {
            let thread = materials.get(|| Brush::solid([1., 0.96, 1., 0.045]).unwrap());
            for f in [0.15, 0.32, 0.52, 0.73, 0.88] {
                paths.push((thread.clone(), path(f, f + 0.004)));
            }
        }
    }
    paths
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use xgpui_luma::controls::button::ButtonEvent;
    #[::core::prelude::v1::test]
    fn luma_controls_scrub_reset_and_stop_hidden_animation() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (page, cx) = app.add_window_view(|_, cx| PastelWaves::new(crate::theme::default_look(), cx));
        page.update(cx, |page, cx| page.activate(cx));
        cx.run_until_parked();
        let buttons = cx.update(|_, cx| page.read(cx).buttons.clone());
        buttons[0].update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        page.update(cx, |page, _| {
            assert!(page.playing);
            page.suspend();
            assert!(!page.active);
        });
        let scrub = cx.update(|_, cx| page.read(cx).scrub.clone());
        scrub.update(cx, |slider, cx| slider.set_thumb_position(slider.primary_thumb_id(), 0.5, cx));
        page.update(cx, |page, _| {
            assert!(!page.playing);
            assert!((page.phase - std::f32::consts::FRAC_PI_2).abs() < 0.02);
        });
        buttons[2].update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        page.update(cx, |page, _| assert!(!page.grain));
        buttons[3].update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        page.update(cx, |page, _| assert!(page.grain && page.phase == 0. && !page.playing));
    }
}

/// Native pixel parity and CPU draw measurements for retained versus rebuilt
/// ribbon materials. No timing threshold is used as a correctness assertion.
pub(crate) fn run_check(cx: &mut App, output: std::path::PathBuf) -> anyhow::Result<()> {
    struct Surface {
        scene: Rc<RefCell<WaveScene>>,
        phase: f32,
        grain: bool,
    }
    impl Render for Surface {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            composition(self.phase, self.grain, self.scene.clone()).size_full()
        }
    }
    std::fs::create_dir_all(&output)?;
    cx.open_window(WindowOptions {window_bounds:Some(WindowBounds::Windowed(Bounds::centered(None,size(px(1100.),px(750.)),cx))),inactive_frame_interval:None,..Default::default()},|window,cx| {
        let surface=cx.new(|_|Surface {scene:Rc::new(RefCell::new(WaveScene::new(true))),phase:0.,grain:true});
        let entity=surface.clone();
        window.on_next_frame(move|window,cx| {
            let result=(||->anyhow::Result<()> {
                let mut records=Vec::new();
                for grain in [false,true] {
                    let rebuilt=Rc::new(RefCell::new(WaveScene::new(false)));
                    let retained=Rc::new(RefCell::new(WaveScene::new(true)));
                    let mut references=Vec::new();
                    for (mode,scene) in [("rebuilt",&rebuilt),("retained",&retained)] {
                        entity.update(cx,|view,cx|{view.scene=scene.clone();view.phase=0.;view.grain=grain;cx.notify();});
                        window.draw(cx).clear(cx);
                        let warmed=retained.borrow().materials[usize::from(grain)].created;
                        for (frame,phase) in [0.,0.15,0.4,0.7,1.2,1.8].into_iter().enumerate() {
                            entity.update(cx,|view,cx|{view.scene=scene.clone();view.phase=phase;view.grain=grain;cx.notify();});
                            let started=Instant::now();window.draw(cx).clear(cx);let draw_ms=started.elapsed().as_secs_f64()*1000.;
                            let started=Instant::now();let image=window.render_to_image()?;let readback_ms=started.elapsed().as_secs_f64()*1000.;
                            if mode=="rebuilt" {references.push(image);} else {
                                anyhow::ensure!(references[frame].as_raw()==image.as_raw(),"retained ribbons changed pixels at grain={grain}, phase={phase}");
                                anyhow::ensure!(scene.borrow().materials[usize::from(grain)].created==warmed,"animation rebuilt immutable brushes");
                                if frame==5 {image.save(output.join(format!("grain-{grain}.png")))?;}
                            }
                            records.push(serde_json::json!({"grain":grain,"phase":phase,"mode":mode,"draw_ms":draw_ms,"readback_ms":readback_ms}));
                        }
                    }
                    let builds=retained.borrow().geometry_builds;
                    for _ in 0..3 {entity.update(cx,|_,cx|cx.notify());window.draw(cx).clear(cx);}
                    anyhow::ensure!(retained.borrow().geometry_builds==builds,"paused redraw rebuilt ribbon geometry");
                }
                std::fs::write(output.join("report.json"),serde_json::to_vec_pretty(&serde_json::json!({"profile":if cfg!(debug_assertions){"debug"}else{"release"},"scope":"Native window draw CPU wall time and separate readback wall time; not GPU-only timing or presented FPS. Reference rebuilds ribbon materials/geometry each frame; retained path reuses materials and unchanged geometry.","checks":["exact native pixels across grain modes and phases","no new ribbon brushes after warmup","paused geometry reuse"],"samples":records,"errors":[]}))?)?;
                Ok(())
            })();
            if let Err(error)=result {eprintln!("Waves check: {error:#}");std::process::exit(1);}
            cx.quit();
        });surface
    })?;
    cx.activate(true);
    Ok(())
}
