//! Ports of xgpui-ext-lab's GPU interop, boids, and GPU canvas workbench.
//! Engines and shaders retain the upstream algorithms; chrome uses the Lab's Radix look.
use gpui::{
    prelude::*, AnyElement, Bounds, ContentMask, Context, Corners, CustomGeometry, Entity, MouseButton, MouseDownEvent,
    MouseMoveEvent, Pixels, Point, Render, ScrollEvent, Window, canvas, div, point, px, size,
};
use std::{cell::Cell, rc::Rc, sync::Arc};
use xgpui_luma::{
    controls::{button::Button, slider::Slider, switch::Switch},
    prelude::HasPresenter,
};
use xgpui_luma_look_radix::{self as radix, Look};
use xgpui_gpu_interop::EngineSurface;
use crate::{
    lab_controls::{Choice, labels},
    screen::{ComparePane, Hero, LabScreen, PanelShell, TextRole, text},
};
mod boids;
mod wave;
mod workbench;
mod runtime;
use runtime::{Command, Runtime};
use futures::StreamExt as _;
use workbench::oracle::{Dab, WIDTH, HEIGHT};

pub(crate) const PAGES: &[(&str, &str)] = &[
    ("GPU Interop Lab", "GPU Interop Lab"),
    ("Interop Boids", "Interop Boids"),
    ("GPU Canvas Workbench", "GPU Canvas Workbench"),
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Waves,
    Boids,
    Workbench,
}
impl Kind {
    fn for_page(page: &str) -> Self {
        match page {
            "Interop Boids" => Self::Boids,
            "GPU Canvas Workbench" => Self::Workbench,
            _ => Self::Waves,
        }
    }
    fn title(self) -> &'static str {
        match self {
            Self::Waves => PAGES[0].0,
            Self::Boids => PAGES[1].0,
            Self::Workbench => PAGES[2].0,
        }
    }
}
#[derive(Clone, Debug)]
struct Settings {
    paused: bool,
    demo: bool,
    particles: u32,
    speed: f32,
    wide_slots: bool,
    work: u32,
    bands: u32,
    exposure: f32,
    refresh: std::time::Duration,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            paused: false,
            demo: false,
            particles: boids::boids::DEFAULT_PARTICLES,
            speed: 1.,
            wide_slots: true,
            work: 0,
            bands: 1,
            exposure: 0.,
            refresh: std::time::Duration::from_secs_f32(1. / 60.),
        }
    }
}

pub(crate) struct Page {
    look: Arc<Look>,
    kind: Kind,
    active: bool,
    runtime: Option<Runtime>,
    updates: Option<gpui::Task<()>>,
    settings: Settings,
    error: Option<String>,
    buttons: Vec<Entity<Button>>,
    pause: Switch,
    choices: Vec<Choice>,
    sliders: Vec<Slider>,
    color: usize,
    radius: f32,
    opacity: f32,
    zoom: f32,
    pan: Point<f32>,
    last_point: Option<Point<f32>>,
    canvas_bounds: Rc<Cell<Bounds<Pixels>>>,
}
const COLORS: [[f32; 3]; 6] = [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.], [1., 1., 0.], [1., 1., 1.], [0., 0., 0.]];
impl Page {
    pub fn new(look: Arc<Look>, page: &str, cx: &mut Context<Self>) -> Self {
        let kind = Kind::for_page(page);
        let buttons =
            ["Start / Restart engine", "Reset", "Hold snapshot", "Release snapshots", "Lose device", "Fit view"]
                .into_iter()
                .enumerate()
                .map(|(index, label)| {
                    radix::Button::new(format!("interop-action-{index}"))
                        .look(&look)
                        .outline()
                        .label(label)
                        .on_click(cx, move |this: &mut Self, cx| {
                            match index {
                                0 => this.start(cx),
                                1 => {
                                    this.settings.exposure = 0.;
                                    this.last_point = None;
                                    this.send(Command::Reset);
                                    this.send_settings();
                                }
                                2 => this.send(Command::Hold),
                                3 => this.send(Command::Release),
                                4 => this.send(Command::LoseDevice),
                                _ => {
                                    this.zoom = 1.;
                                    this.pan = point(0., 0.);
                                }
                            }
                            cx.notify();
                        })
                        .spawn(cx)
                })
                .collect();
        let pause = radix::Switch::new("interop-pause")
            .look(&look)
            .label(if kind == Kind::Workbench {
                "Demo strokes"
            } else {
                "Paused"
            })
            .checked(false)
            .on_value_changed(cx, |this: &mut Self, paused, cx| {
                if this.kind == Kind::Workbench {
                    this.settings.demo = paused;
                } else {
                    this.settings.paused = paused;
                }
                this.send_settings();
                cx.notify();
            })
            .spawn(cx);
        let mut choices = Vec::new();
        if kind == Kind::Waves {
            choices.push(Choice::new(
                "interop-format",
                "Slots",
                &look,
                labels(&["Linear P3 / RGBA16F", "sRGB / BGRA8"]),
                0,
                cx,
                |this: &mut Self, index, cx| {
                    this.settings.wide_slots = index == 0;
                    if this.runtime.is_some() {
                        this.start(cx);
                    }
                    cx.notify();
                },
            ));
            choices.push(Choice::new(
                "interop-work",
                "Shader work",
                &look,
                labels(&["0", "32", "128", "512"]),
                0,
                cx,
                |this: &mut Self, index, cx| {
                    this.settings.work = [0, 32, 128, 512][index];
                    this.send_settings();
                    cx.notify();
                },
            ));
            choices.push(Choice::new(
                "interop-bands",
                "Submission bands",
                &look,
                labels(&["1", "4", "16"]),
                0,
                cx,
                |this: &mut Self, index, cx| {
                    this.settings.bands = [1, 4, 16][index];
                    this.send_settings();
                    cx.notify();
                },
            ));
        } else if kind == Kind::Boids {
            choices.push(Choice::new(
                "interop-particles",
                "Particles",
                &look,
                labels(&["500", "1,500", "15,000", "50,000", "100,000"]),
                1,
                cx,
                |this: &mut Self, index, cx| {
                    this.settings.particles = [500, 1500, 15000, 50000, 100000][index];
                    this.send_settings();
                    cx.notify();
                },
            ));
        } else {
            choices.push(Choice::new(
                "interop-brush-color",
                "Brush",
                &look,
                labels(&["P3 Red", "P3 Green", "P3 Blue", "Yellow", "White", "Black"]),
                0,
                cx,
                |this: &mut Self, index, cx| {
                    this.color = index;
                    cx.notify();
                },
            ));
        }
        let sliders = if kind == Kind::Boids {
            vec![
                radix::Slider::new("interop-speed")
                    .look(&look)
                    .range(0.25..4.)
                    .value(1.)
                    .step(0.05)
                    .on_value_changed(cx, |this: &mut Self, _, value, cx| {
                        this.settings.speed = value;
                        this.send_settings();
                        cx.notify();
                    })
                    .spawn(cx),
            ]
        } else if kind == Kind::Workbench {
            vec![
                radix::Slider::new("interop-radius")
                    .look(&look)
                    .range(1. ..40.)
                    .value(10.)
                    .step(1.)
                    .on_value_changed(cx, |this: &mut Self, _, value, cx| {
                        this.radius = value;
                        cx.notify();
                    })
                    .spawn(cx),
                radix::Slider::new("interop-opacity")
                    .look(&look)
                    .range(0.05..1.)
                    .value(0.6)
                    .step(0.05)
                    .on_value_changed(cx, |this: &mut Self, _, value, cx| {
                        this.opacity = value;
                        cx.notify();
                    })
                    .spawn(cx),
                radix::Slider::new("interop-exposure")
                    .look(&look)
                    .range(-1. ..1.)
                    .value(0.)
                    .step(0.1)
                    .on_value_changed(cx, |this: &mut Self, _, value, cx| {
                        this.settings.exposure = value;
                        this.send_settings();
                        cx.notify();
                    })
                    .spawn(cx),
            ]
        } else {
            Vec::new()
        };
        Self {
            look,
            kind,
            active: true,
            runtime: None,
            updates: None,
            settings: Settings::default(),
            error: None,
            buttons,
            pause,
            choices,
            sliders,
            color: 0,
            radius: 10.,
            opacity: 0.6,
            zoom: 1.,
            pan: point(0., 0.),
            last_point: None,
            canvas_bounds: Rc::default(),
        }
    }
    fn start(&mut self, cx: &mut Context<Self>) {
        self.updates = None;
        self.runtime = None;
        match Runtime::start(self.kind, self.settings.clone()) {
            Ok((runtime, mut updates)) => {
                self.updates = Some(cx.spawn(async move |this, cx| {
                    while updates.next().await.is_some() {
                        if this.update(cx, |_, cx| cx.notify()).is_err() {
                            break;
                        }
                    }
                }));
                self.runtime = Some(runtime);
                self.error = None;
            }
            Err(error) => self.error = Some(format!("{error:#}")),
        }
    }
    fn send(&self, command: Command) {
        if let Some(runtime) = &self.runtime {
            runtime.send(command);
        }
    }
    fn send_settings(&self) {
        self.send(Command::Settings(self.settings.clone()));
    }
    pub fn suspend(&mut self) {
        self.active = false;
        self.updates = None;
        self.last_point = None;
        self.runtime = None;
    }
    pub fn activate(&mut self) {
        self.active = true;
    }
    fn slot_point(&self, position: Point<Pixels>) -> Option<Point<f32>> {
        let bounds = self.canvas_bounds.get();
        if !bounds.contains(&position) || bounds.size.width <= px(0.) {
            return None;
        }
        let scale = bounds.size.width.as_f32() / WIDTH as f32 * self.zoom;
        Some(point(
            (position.x - bounds.left()).as_f32() / scale + self.pan.x,
            (position.y - bounds.top()).as_f32() / scale + self.pan.y,
        ))
    }
    fn stroke_to(&mut self, to: Point<f32>) {
        let from = self.last_point.replace(to).unwrap_or(to);
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let steps = ((dx * dx + dy * dy).sqrt() / (self.radius / 3.).max(1.)).ceil().max(1.) as u32;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            self.send(Command::Dab(
                Dab {
                    x: from.x + dx * t,
                    y: from.y + dy * t,
                    radius: self.radius,
                    opacity: self.opacity,
                    color: COLORS[self.color],
                },
                std::time::Instant::now(),
            ));
        }
    }
    fn placement(&self, surface: Option<EngineSurface>, preview: bool, interactive: bool, height: f32) -> AnyElement {
        let tracked = self.canvas_bounds.clone();
        let aspect = if self.kind == Kind::Boids {
            4. / 3.
        } else {
            WIDTH as f32 / HEIGHT as f32
        };
        let (zoom, pan) = if interactive {
            (self.zoom, self.pan)
        } else {
            (1., point(0., 0.))
        };
        canvas(
            move |bounds, _, _| {
                let width = bounds.size.width.as_f32();
                let h = bounds.size.height.as_f32();
                let w = width.min(h * aspect);
                let h = h.min(width / aspect);
                let bounds = Bounds::new(
                    point(
                        bounds.left() + px((width - w) / 2.),
                        bounds.top() + px((bounds.size.height.as_f32() - h) / 2.),
                    ),
                    size(px(w), px(h)),
                );
                if interactive {
                    tracked.set(bounds);
                }
                bounds
            },
            move |bounds, fitted, window, _| {
                if let Some(surface) = &surface {
                    let scale = fitted.size.width.as_f32() / WIDTH as f32 * zoom;
                    let placed = Bounds::new(
                        point(fitted.left() - px(pan.x * scale), fitted.top() - px(pan.y * scale)),
                        size(fitted.size.width * zoom, fitted.size.height * zoom),
                    );
                    window.with_content_mask(Some(ContentMask { bounds }), |window| {
                        let content = if preview {
                            surface.preview_content()
                        } else {
                            surface.content()
                        };
                        if let Some(content) = content {
                            window.paint_custom(
                                CustomGeometry::RoundedRect { bounds: placed, corner_radii: Corners::all(px(6.)) },
                                content,
                            );
                        }
                    });
                }
            },
        )
        .w_full()
        .h(px(height))
        .into_any_element()
    }
}
fn surface_is_waiting(surface: &Option<EngineSurface>) -> bool {
    surface.as_ref().is_some_and(|surface| surface.stats().waiting > 0)
}

impl Render for Page {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(refresh) = window.refresh_interval(cx)
            && refresh != self.settings.refresh
        {
            self.settings.refresh = refresh;
            self.send_settings();
        }
        let snapshot = self.runtime.as_ref().map(Runtime::snapshot).unwrap_or_default();
        if self.active && surface_is_waiting(&snapshot.surface) {
            window.request_animation_frame();
        }
        if self.kind == Kind::Workbench && (self.sliders[2].read(cx).value() - self.settings.exposure).abs() > 0.001 {
            self.sliders[2].update(cx, |slider, cx| slider.set_value(self.settings.exposure, cx));
        }
        let surface = snapshot.surface;
        let detail = surface
            .as_ref()
            .map(|s| {
                let stats = s.stats();
                format!(
                    "{:?} · {} presents · {} composites · {} ring-full · {} waiting",
                    s.desc().format,
                    stats.presents,
                    stats.composited,
                    stats.would_block,
                    stats.waiting
                )
            })
            .unwrap_or_else(|| {
                if self.runtime.is_some() {
                    "Starting engine…".into()
                } else {
                    "Press Start / Restart engine. Navigating away releases its GPU resources.".into()
                }
            });
        let title = self.kind.title();
        let primary = self.placement(surface.clone(), false, self.kind == Kind::Workbench, 300.);
        let preview = self.placement(surface.clone(), true, false, 300.);
        let stage = div()
            .id("interop-drawing")
            .child(primary)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, _, cx| {
                    if this.kind == Kind::Workbench
                        && let Some(at) = this.slot_point(event.position)
                    {
                        this.last_point = None;
                        this.stroke_to(at);
                        cx.notify();
                    }
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                if this.kind == Kind::Workbench && event.pressed_button == Some(MouseButton::Left) {
                    if let Some(at) = this.slot_point(event.position) {
                        this.stroke_to(at);
                        cx.notify();
                    }
                } else {
                    this.last_point = None;
                }
            }))
            .on_mouse_up(MouseButton::Left, cx.listener(|this, _, _, _| this.last_point = None))
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, event: &MouseDownEvent, _, cx| {
                    if this.kind == Kind::Workbench
                        && let Some(at) = this.slot_point(event.position)
                    {
                        this.send(Command::Inspect(at.x as u32, at.y as u32));
                        cx.notify();
                    }
                }),
            )
            .on_scroll(cx.listener(|this, event: &ScrollEvent, _, cx| {
                if this.kind != Kind::Workbench {
                    return;
                }
                let delta = event.delta.pixel_delta(px(24.));
                if event.modifiers.platform || event.modifiers.control {
                    this.zoom = (this.zoom * (delta.y.as_f32() * 0.005).exp()).clamp(1., 8.);
                } else {
                    let scale = this.canvas_bounds.get().size.width.as_f32() / WIDTH as f32 * this.zoom;
                    if scale > 0. {
                        this.pan.x -= delta.x.as_f32() / scale;
                        this.pan.y -= delta.y.as_f32() / scale;
                    }
                }
                this.pan.x = this.pan.x.clamp(0., WIDTH as f32 * (1. - 1. / this.zoom));
                this.pan.y = this.pan.y.clamp(0., HEIGHT as f32 * (1. - 1. / this.zoom));
                this.last_point = None;
                cx.stop_propagation();
                cx.notify();
            }));
        let mut screen = LabScreen::new("gpu-interop", self.look.clone()).crumb("xgpui-gpu-interop", title).title(title)
            .lede(match self.kind { Kind::Waves => "An independent wgpu device renders waves into a shared surface, composited beside ordinary sRGB controls.", Kind::Boids => "wgpu's boids compute and render example, with persistent particle buffers on an independent device and engine thread.", Kind::Workbench => "Paint a persistent linear-P3 GPU image. Published revisions are immutable; both placements show the same snapshot." })
            .hero(Hero::compare(ComparePane::new(format!("Native presentation ({:?})", window.color_status().effective()), stage), ComparePane::new("sRGB preview", preview)))
            .caption(format!("{detail} · Requested {:?} / Effective {:?} / Display wide gamut {:?}", window.color_status().requested(), window.color_status().effective(), window.color_status().display_wide_gamut()))
            .section_panel(PanelShell::new("Engine diagnostics").child(text(&self.look, TextRole::Caption, snapshot.detail)))
            .action(self.buttons[0].clone())
            .notice(self.error.clone().or(snapshot.error))
            .credits("Ported from gpui-luma-dev/xgpui apps/xgpui-ext-lab. Boids derives from gfx-rs/wgpu (Apache-2.0 OR MIT). Timings are engine diagnostics, not confirmed displayed FPS.");
        for choice in &self.choices {
            screen = screen.control(choice.clone());
        }
        screen = screen.control(self.pause.clone());
        let names: &[String] = if self.kind == Kind::Boids {
            &[format!("Speed {:.2}×", self.settings.speed)]
        } else {
            &[
                format!("Radius {:.0} px", self.radius),
                format!("Opacity {:.0}%", self.opacity * 100.),
                format!("Exposure {:+.1}", self.settings.exposure),
            ]
        };
        for (label, slider) in names.iter().zip(&self.sliders) {
            screen = screen.control(div().w(px(160.)).child(label.clone()).child(slider.clone()));
        }
        if self.kind == Kind::Workbench {
            for index in 1..6 {
                screen = screen.action(self.buttons[index].clone());
            }
            screen = screen.section(self.placement(surface, false, false, 110.)).note(format!("Drag to paint · Right-click to inspect 4×4 · Scroll to pan · Ctrl/Cmd-scroll to zoom · Zoom {:.2}×. {}", self.zoom, snapshot.inspection));
        }
        screen
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::TestAppContext;
    use xgpui_luma::controls::button::ButtonEvent;
    #[test]
    fn section_routes_are_alphabetical_and_each_page_is_lazy_in_both_themes() {
        let routes = crate::app::Route::all();
        let gamut = routes.iter().rposition(|route| route.starts_with("xgpui-gamut/")).unwrap();
        let interop = routes.iter().position(|route| route.starts_with("xgpui-gpu-interop/")).unwrap();
        let media = routes.iter().position(|route| route.starts_with("xgpui-media/")).unwrap();
        assert!(gamut < interop && interop < media);
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        for mode in [xgpui_luma::theme::ThemeMode::Light, xgpui_luma::theme::ThemeMode::Dark] {
            for &(page, _) in PAGES {
                assert!(crate::app::Route::resolve(&format!("xgpui-gpu-interop/{page}")).is_ok());
                let look = crate::theme::default_look();
                look.set_mode(mode);
                let (view, cx) = app.add_window_view(|_, cx| Page::new(look, page, cx));
                cx.run_until_parked();
                for slot in ["gpu-interop-crumb", "gpu-interop-hero", "gpu-interop-controls"] {
                    assert!(cx.debug_bounds(slot).is_some(), "{page}: {slot}");
                }
                cx.update(|_, cx| assert!(view.read(cx).runtime.is_none(), "navigation must not create devices"));
            }
        }
    }

    #[test]
    fn controls_forward_particle_and_brush_edits_and_navigation_disconnects_workers() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (boids, cx) = app.add_window_view(|_, cx| Page::new(crate::theme::default_look(), "Interop Boids", cx));
        cx.run_until_parked();
        let (runtime, received) = Runtime::recording();
        boids.update(cx, |page, _| page.runtime = Some(runtime));
        let choice = cx.update(|_, cx| boids.read(cx).choices[0].clone());
        cx.update(|_, cx| choice.select(4, cx));
        let Command::Settings(settings) = received.try_recv().unwrap() else {
            panic!("particle command")
        };
        assert_eq!(settings.particles, 100_000);
        boids.update(cx, |page, _| page.suspend());
        assert!(matches!(received.try_recv(), Err(std::sync::mpsc::TryRecvError::Disconnected)));

        let (view, cx) =
            app.add_window_view(|_, cx| Page::new(crate::theme::default_look(), "GPU Canvas Workbench", cx));
        cx.run_until_parked();
        let (runtime, received) = Runtime::recording();
        view.update(cx, |page, _| page.runtime = Some(runtime));
        // Use the actual fitted canvas, so letterboxing and pointer mapping are exercised.
        let bounds = cx.update(|_, cx| view.read(cx).canvas_bounds.get());
        cx.simulate_mouse_down(bounds.center(), MouseButton::Left, Default::default());
        cx.simulate_mouse_up(bounds.center(), MouseButton::Left, Default::default());
        let Command::Dab(dab, _) = received.try_recv().unwrap() else {
            panic!("brush command")
        };
        assert!((dab.x - WIDTH as f32 / 2.).abs() < 1.);
        assert!((dab.y - HEIGHT as f32 / 2.).abs() < 1.);
        assert_eq!(dab.color, COLORS[0]);
        cx.simulate_mouse_down(bounds.center(), MouseButton::Right, Default::default());
        let Command::Inspect(x, y) = received.try_recv().unwrap() else {
            panic!("inspection command")
        };
        assert_eq!((x, y), (WIDTH / 2, HEIGHT / 2));
        let reset = cx.update(|_, cx| view.read(cx).buttons[1].clone());
        reset.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        assert!(matches!(received.try_recv(), Ok(Command::Reset)));
        assert!(matches!(received.try_recv(), Ok(Command::Settings(_))));
        view.update(cx, |page, _| page.suspend());
        assert!(matches!(received.try_recv(), Err(std::sync::mpsc::TryRecvError::Disconnected)));
    }
}
