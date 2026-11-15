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
    screen::{ComparePane, Hero, LabScreen, PanelShell, TextRole, text, properties, card_surface},
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
    threaded: bool,
    demo: bool,
    radius: f32,
    opacity: f32,
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
            threaded: true,
            demo: false,
            radius: 10.,
            opacity: 0.6,
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
    boids: boids::session::Session,
    error: Option<String>,
    buttons: Vec<Entity<Button>>,
    pause: Switch,
    choices: Vec<Choice>,
    sliders: Vec<Slider>,
    color: usize,
    swatches: Vec<Entity<Button>>,
    compare: bool,
    compare_active: bool,
    compare_control: Switch,
    preview: bool,
    zoom: f32,
    pan: Point<f32>,
    last_point: Option<Point<f32>>,
    stroke_id: u32,
    canvas_bounds: Rc<Cell<Bounds<Pixels>>>,
}
const COLORS: [[f32; 3]; 6] =
    [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.], [0.8, 0.3, 0.05], [0.004, 0.004, 0.004], [1., 1., 1.]];
const SWATCHES: [(&str, u32); 6] = [
    ("P3 red", 0xff2a1a),
    ("P3 green", 0x2aff3a),
    ("P3 blue", 0x2a40ff),
    ("Orange", 0xe89540),
    ("Near-black", 0x0e0e0e),
    ("White", 0xffffff),
];
impl Page {
    pub fn new(look: Arc<Look>, page: &str, cx: &mut Context<Self>) -> Self {
        let kind = Kind::for_page(page);
        let buttons = [
            "Start / Restart engine",
            "Reset",
            "Hold snapshot",
            "Release snapshots",
            "Lose device",
            "Fit view",
            "Reset to 1×",
        ]
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
                        6 => {
                            this.settings.speed = 1.;
                            this.sliders[0].update(cx, |slider, cx| slider.set_value(0., cx));
                            this.send_settings();
                        }
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
                this.check_boids_pause(cx);
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
            choices.push(Choice::new(
                "interop-engine-thread",
                "Engine on",
                &look,
                labels(&["Own thread", "UI thread"]),
                0,
                cx,
                |this: &mut Self, index, cx| {
                    this.settings.threaded = index == 0;
                    this.send_settings();
                    cx.notify();
                },
            ));
        } else {
            choices.push(Choice::new(
                "interop-presentation",
                "Show",
                &look,
                labels(&["Auto", "sRGB preview"]),
                0,
                cx,
                |this: &mut Self, index, cx| {
                    this.preview = index == 1;
                    cx.notify();
                },
            ));
        }
        let compare_control = radix::Switch::new("workbench-compare")
            .look(&look)
            .label("Compare wide | sRGB")
            .checked(false)
            .on_value_changed(cx, |this: &mut Self, compare, cx| {
                this.compare = compare;
                cx.notify();
            })
            .spawn(cx);
        let swatches = if kind == Kind::Workbench {
            SWATCHES
                .into_iter()
                .enumerate()
                .map(|(index, (name, color))| {
                    let owner = cx.weak_entity();
                    radix::Button::new(format!("workbench-swatch-{index}"))
                        .look(&look)
                        .outline()
                        .content(move |_, cx| {
                            let selected = owner.upgrade().is_some_and(|page| page.read(cx).color == index);
                            div().size(px(24.)).rounded(px(3.)).bg(gpui::rgb(color)).border_2().border_color(
                                if selected {
                                    gpui::white()
                                } else {
                                    gpui::transparent_black()
                                },
                            )
                        })
                        .with_template_modifier(move |row, _| {
                            row.size(px(34.))
                                .p_1()
                                .aria_label(name)
                                .debug_selector(move || format!("workbench-swatch-{index}"))
                        })
                        .on_click(cx, move |this: &mut Self, cx| {
                            this.color = index;
                            this.last_point = None;
                            for button in &this.swatches {
                                button.update(cx, |_, cx| cx.notify());
                            }
                            cx.notify();
                        })
                        .spawn(cx)
                })
                .collect()
        } else {
            Vec::new()
        };
        let sliders = if kind == Kind::Boids {
            vec![
                radix::Slider::new("interop-speed")
                    .look(&look)
                    .range(-2. ..2.)
                    .value(0.)
                    .step(0.01)
                    .on_value_changed(cx, |this: &mut Self, _, value, cx| {
                        let speed = value.exp2();
                        this.settings.speed = if (speed - 1.).abs() < 0.04 {
                            1.
                        } else {
                            (speed * 100.).round() / 100.
                        };
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
                        this.settings.radius = value;
                        this.send_settings();
                        cx.notify();
                    })
                    .spawn(cx),
                radix::Slider::new("interop-opacity")
                    .look(&look)
                    .range(0.05..1.)
                    .value(0.6)
                    .step(0.05)
                    .on_value_changed(cx, |this: &mut Self, _, value, cx| {
                        this.settings.opacity = value;
                        this.send_settings();
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
            boids: boids::session::Session::default(),
            error: None,
            buttons,
            pause,
            choices,
            sliders,
            color: 0,
            swatches,
            compare: false,
            compare_active: false,
            compare_control,
            preview: false,
            zoom: 1.,
            pan: point(0., 0.),
            last_point: None,
            stroke_id: 0,
            canvas_bounds: Rc::default(),
        }
    }
    fn start(&mut self, cx: &mut Context<Self>) {
        if self.kind == Kind::Boids {
            self.start_boids(cx);
            return;
        }
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
    fn send_settings(&mut self) {
        self.boids.apply(&self.settings);
        self.send(Command::Settings(self.settings.clone()));
    }
    fn check_boids_pause(&mut self, cx: &mut Context<Self>) {
        if self.kind != Kind::Boids {
            return;
        }
        self.boids.pause_check = None;
        if self.settings.paused && self.boids.engine.is_some() {
            self.boids.pause_check = Some(cx.spawn(async move |this, cx| {
                cx.background_executor().timer(std::time::Duration::from_millis(100)).await;
                let _ = this.update(cx, |page, _| {
                    if page.settings.paused {
                        page.boids.paused_baseline = page.boids.engine.as_ref().map(|engine| engine.status().steps);
                    }
                });
            }));
        }
    }
    fn start_boids(&mut self, cx: &mut Context<Self>) {
        self.updates = None;
        self.boids.release();
        self.boids = boids::session::Session::default();
        let particles = self.settings.particles;
        self.boids.starting = Some(cx.spawn(async move |this, cx| {
            let result =
                cx.background_executor().spawn(async move { boids::engine::Engine::new(particles, true) }).await;
            let _ = this.update(cx, |page, cx| {
                page.boids.starting = None;
                match result {
                    Ok(mut engine) => {
                        let (wake, mut updates) = futures::channel::mpsc::unbounded();
                        let failure = wake.clone();
                        engine.on_failure(move || {
                            let _ = failure.unbounded_send(());
                        });
                        engine.surface().on_present(move || {
                            let _ = wake.unbounded_send(());
                        });
                        page.boids.install(engine);
                        page.boids.apply(&page.settings);
                        page.check_boids_pause(cx);
                        page.updates = Some(cx.spawn(async move |this, cx| {
                            while updates.next().await.is_some() {
                                if this.update(cx, |_, cx| cx.notify()).is_err() {
                                    break;
                                }
                            }
                        }));
                    }
                    Err(error) => page.boids.error = Some(format!("{error:#}")),
                }
                cx.notify();
            });
        }));
    }
    pub fn suspend(&mut self) {
        self.active = false;
        self.boids.release();
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
        let x = self.view_x(position.x, bounds);
        Some(point(x / scale + self.pan.x, (position.y - bounds.top()).as_f32() / scale + self.pan.y))
    }
    fn view_x(&self, x: Pixels, bounds: Bounds<Pixels>) -> f32 {
        let mut x = (x - bounds.left()).as_f32();
        let half = bounds.size.width.as_f32() / 2.;
        if self.compare_active && x >= half {
            x -= half;
        }
        x
    }
    fn scroll_view(&mut self, event: &ScrollEvent) {
        let bounds = self.canvas_bounds.get();
        if bounds.size.width <= px(0.) {
            return;
        }
        let delta = event.delta.pixel_delta(px(24.));
        if event.modifiers.platform || event.modifiers.control {
            let Some(anchor) = self.slot_point(event.position) else {
                return;
            };
            self.zoom = (self.zoom * (delta.y.as_f32() * 0.005).exp()).clamp(1., 8.);
            let scale = bounds.size.width.as_f32() / WIDTH as f32 * self.zoom;
            self.pan = point(
                anchor.x - self.view_x(event.position.x, bounds) / scale,
                anchor.y - (event.position.y - bounds.top()).as_f32() / scale,
            );
        } else {
            let scale = bounds.size.width.as_f32() / WIDTH as f32 * self.zoom;
            self.pan.x -= delta.x.as_f32() / scale;
            self.pan.y -= delta.y.as_f32() / scale;
        }
        self.pan.x = self.pan.x.clamp(0., WIDTH as f32 * (1. - 1. / self.zoom));
        self.pan.y = self.pan.y.clamp(0., HEIGHT as f32 * (1. - 1. / self.zoom));
        self.last_point = None;
    }
    fn stroke_to(&mut self, to: Point<f32>) {
        let from = self.last_point.replace(to).unwrap_or_else(|| {
            self.stroke_id = (self.stroke_id.wrapping_add(1) & 0x7fff_ffff).max(1);
            to
        });
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let steps = ((dx * dx + dy * dy).sqrt() / (self.settings.radius / 3.).max(1.)).ceil().max(1.) as u32;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            self.send(Command::Dab(
                Dab {
                    stroke_id: self.stroke_id,
                    x: from.x + dx * t,
                    y: from.y + dy * t,
                    radius: self.settings.radius,
                    opacity: self.settings.opacity,
                    color: COLORS[self.color],
                },
                std::time::Instant::now(),
            ));
        }
    }
    fn placement(
        &self,
        surface: Option<EngineSurface>,
        preview: bool,
        interactive: bool,
        thumbnail: bool,
    ) -> AnyElement {
        let tracked = self.canvas_bounds.clone();
        let boids = self.kind == Kind::Boids;
        let compare = interactive && self.compare_active;
        let (zoom, pan) = if interactive {
            (self.zoom, self.pan)
        } else {
            (1., point(0., 0.))
        };
        let ui_work = self.boids.measure_from.map(|started| (self.boids.ui_work.clone(), started));
        canvas(
            move |bounds, _, _| {
                if interactive {
                    tracked.set(bounds);
                }
                bounds
            },
            move |bounds, fitted, window, _| {
                if let Some((samples, started)) = &ui_work {
                    boids::session::push_sample(&mut samples.borrow_mut(), started.elapsed().as_secs_f32() * 1000.);
                }
                if let Some(surface) = &surface {
                    let scale = fitted.size.width.as_f32() / WIDTH as f32 * zoom;
                    let placed = Bounds::new(
                        point(fitted.left() - px(pan.x * scale), fitted.top() - px(pan.y * scale)),
                        size(fitted.size.width * zoom, fitted.size.height * zoom),
                    );
                    window.with_content_mask(Some(ContentMask { bounds }), |window| {
                        if compare {
                            let half = bounds.size.width / 2.;
                            for (offset, content) in [(px(0.), surface.content()), (half, surface.preview_content())] {
                                if let Some(content) = content {
                                    let mask = Bounds::new(
                                        point(bounds.left() + offset, bounds.top()),
                                        size(half, bounds.size.height),
                                    );
                                    let at = Bounds::new(point(placed.left() + offset, placed.top()), placed.size);
                                    window.with_content_mask(Some(ContentMask { bounds: mask }), |window| {
                                        window.paint_custom(
                                            CustomGeometry::RoundedRect {
                                                bounds: at,
                                                corner_radii: Corners::all(px(6.)),
                                            },
                                            content,
                                        );
                                    });
                                }
                            }
                            window.paint_quad(gpui::fill(
                                Bounds::new(
                                    point(bounds.left() + half - px(0.5), bounds.top()),
                                    size(px(1.), bounds.size.height),
                                ),
                                gpui::black(),
                            ));
                            return;
                        }
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
        .when(!boids, |canvas| canvas.aspect_square())
        .when(boids, |canvas| canvas.h(px(500.)))
        .when(thumbnail, |canvas| canvas.size(px(110.)))
        .when(!thumbnail, |canvas| canvas.w_full())
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
        self.compare_active = self.kind == Kind::Workbench
            && self.compare
            && window.color_status().effective() == gpui::WindowColorMode::WideGamutSdr;
        let frame_started = std::time::Instant::now();
        let (boids_status, inline_more) = self.boids.frame(frame_started);
        if inline_more && self.active {
            window.request_animation_frame();
        }
        let mut snapshot = self.runtime.as_ref().map(Runtime::snapshot).unwrap_or_default();
        if let Some(engine) = &self.boids.engine {
            snapshot.surface = Some(engine.surface().clone());
        }
        if let Some(status) = &boids_status {
            snapshot.error =
                status.error.clone().or_else(|| snapshot.surface.as_ref().and_then(EngineSurface::unsupported));
        }
        if self.active && snapshot.error.is_none() && surface_is_waiting(&snapshot.surface) {
            window.request_animation_frame();
        }
        if self.kind == Kind::Workbench && (self.sliders[2].read(cx).value() - self.settings.exposure).abs() > 0.001 {
            self.sliders[2].update(cx, |slider, cx| slider.set_value(self.settings.exposure, cx));
        }
        let surface = snapshot.surface.clone();
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
                if self.runtime.is_some() || self.boids.starting.is_some() {
                    "Starting engine…".into()
                } else {
                    "Press Start / Restart engine. Navigating away releases its GPU resources.".into()
                }
            });
        let title = self.kind.title();
        let primary = self.placement(
            surface.clone(),
            self.kind == Kind::Workbench && self.preview,
            self.kind == Kind::Workbench,
            false,
        );
        let stage = div()
            .id("interop-drawing")
            .when(self.kind == Kind::Boids, |stage| stage.debug_selector(|| "boids-simulation".into()))
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
                if this.kind == Kind::Workbench
                    && event.pressed_button == Some(MouseButton::Left)
                    && this.last_point.is_some()
                {
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
                this.scroll_view(event);
                cx.stop_propagation();
                cx.notify();
            }));
        let hero = if self.kind == Kind::Boids {
            Hero::stage(ComparePane::new("Flock simulation", stage))
        } else if self.kind == Kind::Workbench {
            Hero::full(
                div()
                    .flex()
                    .flex_wrap()
                    .items_start()
                    .gap_6()
                    .child(div().w(px(640.)).max_w_full().child(card_surface(stage, &self.look)))
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(280.))
                            .max_w(px(400.))
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(text(&self.look, TextRole::Caption, "Same revision, second placement"))
                            .child(self.placement(surface.clone(), self.preview, false, true))
                            .child(self.workbench_controls())
                            .child(self.inspection_view(&snapshot)),
                    ),
            )
        } else {
            Hero::compare(
                ComparePane::new(format!("Native presentation ({:?})", window.color_status().effective()), stage),
                ComparePane::new(
                    "sRGB preview",
                    div().debug_selector(|| "interop-srgb-preview".into()).child(self.placement(
                        surface.clone(),
                        true,
                        false,
                        false,
                    )),
                ),
            )
        };
        let mut caption = if self.kind == Kind::Boids {
            detail
        } else {
            format!(
                "{detail} · Requested {:?} / Effective {:?} / Display wide gamut {:?}",
                window.color_status().requested(),
                window.color_status().effective(),
                window.color_status().display_wide_gamut()
            )
        };
        if self.kind == Kind::Workbench {
            let showing = if self.compare_active {
                "wide | sRGB comparison"
            } else if self.preview {
                "sRGB preview (forced)"
            } else if window.color_status().effective() == gpui::WindowColorMode::WideGamutSdr {
                "wide-gamut"
            } else {
                "sRGB preview (window fallback)"
            };
            caption.push_str(&format!(" · Showing {showing}"));
        }
        let mut screen = LabScreen::new("gpu-interop", self.look.clone()).crumb("xgpui-gpu-interop", title).title(title)
            .lede(match self.kind { Kind::Waves => "An independent wgpu device renders waves into a shared surface, composited beside ordinary sRGB controls.", Kind::Boids => "Explore compute capacity, compositor pacing, and UI responsiveness. Increase the flock, compare Own thread with UI thread, then change speed or resize the window under load.", Kind::Workbench => "Paint and inspect a persistent linear-P3 GPU image. Enable Compare wide | sRGB to compare the same region inside the interactive canvas." })
            .hero(hero)
            .caption(caption)
            .notice(self.error.clone().or(self.boids.error.clone()).or(snapshot.error.clone()))
            .credits("Ported from gpui-luma-dev/xgpui apps/xgpui-ext-lab. Boids derives from gfx-rs/wgpu (Apache-2.0 OR MIT). Timings are engine diagnostics, not confirmed displayed FPS.");
        if self.kind != Kind::Workbench {
            screen = screen.action(self.buttons[0].clone());
        }
        if self.kind == Kind::Workbench && self.compare && !self.compare_active {
            screen = screen
                .notice(Some("Compare wide | sRGB requires effective WideGamutSdr output; showing the single canvas."));
        }
        if let Some(status) = &boids_status {
            screen = screen.section_panel(
                PanelShell::new("Capacity, pacing & failure analysis")
                    .child(properties(&self.look, self.boids.diagnostics(status))),
            );
        } else if self.kind != Kind::Boids {
            screen = screen.section_panel(PanelShell::new("Engine diagnostics").child(text(
                &self.look,
                TextRole::Caption,
                snapshot.detail,
            )));
        }
        if self.kind == Kind::Boids {
            screen = screen.note("Try 100,000 boids on Own thread, then UI thread. UI-thread mode deliberately puts engine recording and GPU waits in the UI frame. Observe-only avoids compositor GPU waits for slow engine steps. Pausing stops GPU work; Restart engine retries after an engine failure. UI timings cover the latest 120 frames after warm-up and are not displayed FPS.");
        }
        if self.kind != Kind::Workbench {
            for choice in &self.choices {
                screen = screen.control(choice.clone());
            }
            screen = screen.control(self.pause.clone());
        }
        if self.kind == Kind::Boids {
            screen = screen.control(
                div()
                    .flex()
                    .items_end()
                    .gap_2()
                    .child(
                        div()
                            .w(px(160.))
                            .child(format!("Speed {:.2}×", self.settings.speed))
                            .child(self.sliders[0].clone()),
                    )
                    .child(self.buttons[6].clone()),
            );
        }
        if self.kind == Kind::Workbench {
            screen = screen.note(format!("Drag to paint · Right-click to inspect 4×4 · Scroll to pan · Ctrl/Cmd-scroll to zoom around the pointer · Zoom {:.2}×.", self.zoom));
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
                for slot in ["gpu-interop-crumb", "gpu-interop-hero"] {
                    assert!(cx.debug_bounds(slot).is_some(), "{page}: {slot}");
                }
                if page == "Interop Boids" {
                    assert!(cx.debug_bounds("boids-simulation").is_some());
                    assert!(cx.debug_bounds("interop-srgb-preview").is_none(), "Boids has no color comparison");
                } else if page == "GPU Canvas Workbench" {
                    assert!(cx.debug_bounds("interop-srgb-preview").is_none());
                    assert!(cx.debug_bounds("workbench-controls").is_some());
                    assert!(cx.debug_bounds("workbench-inspect").is_some());
                } else {
                    assert!(cx.debug_bounds("interop-srgb-preview").is_some());
                }
                cx.update(|_, cx| {
                    let page = view.read(cx);
                    assert!(
                        page.runtime.is_none() && page.boids.engine.is_none() && page.boids.starting.is_none(),
                        "navigation must not create devices"
                    );
                });
            }
        }
    }

    #[test]
    fn controls_forward_particle_and_brush_edits_and_navigation_disconnects_workers() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (boids, cx) = app.add_window_view(|_, cx| Page::new(crate::theme::default_look(), "Interop Boids", cx));
        cx.run_until_parked();
        let choice = cx.update(|_, cx| boids.read(cx).choices[0].clone());
        cx.update(|_, cx| choice.select(4, cx));
        cx.update(|_, cx| assert_eq!(boids.read(cx).settings.particles, 100_000));
        let mode = cx.update(|_, cx| boids.read(cx).choices[1].clone());
        cx.update(|_, cx| mode.select(1, cx));
        cx.update(|_, cx| assert!(!boids.read(cx).settings.threaded));
        cx.update(|_, cx| mode.select(0, cx));
        cx.update(|_, cx| assert!(boids.read(cx).settings.threaded));
        let speed = cx.update(|_, cx| boids.read(cx).sliders[0].clone());
        speed.update(cx, |slider, cx| {
            slider.set_thumb_position(slider.primary_thumb_id(), 1., cx);
        });
        cx.update(|_, cx| assert_eq!(boids.read(cx).settings.speed, 4.));
        let reset_speed = cx.update(|_, cx| boids.read(cx).buttons[6].clone());
        reset_speed.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        cx.update(|_, cx| assert_eq!(boids.read(cx).settings.speed, 1.));
        boids.update(cx, |page, _| page.suspend());

        let (view, cx) =
            app.add_window_view(|_, cx| Page::new(crate::theme::default_look(), "GPU Canvas Workbench", cx));
        cx.run_until_parked();
        let (runtime, received) = Runtime::recording();
        view.update(cx, |page, _| page.runtime = Some(runtime));
        let sliders = cx.update(|_, cx| view.read(cx).sliders.clone());
        sliders[0].update(cx, |slider, cx| {
            slider.set_thumb_position(slider.primary_thumb_id(), 1., cx);
        });
        let Command::Settings(settings) = received.try_recv().unwrap() else {
            panic!("radius setting")
        };
        assert_eq!(settings.radius, 40.);
        sliders[1].update(cx, |slider, cx| {
            slider.set_thumb_position(slider.primary_thumb_id(), 0., cx);
        });
        let Command::Settings(settings) = received.try_recv().unwrap() else {
            panic!("opacity setting")
        };
        assert_eq!(settings.opacity, 0.05);
        let blue = cx.update(|_, cx| view.read(cx).swatches[2].clone());
        blue.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        cx.update(|_, cx| {
            let page = view.read(cx);
            assert_eq!(page.color, 2);
            assert_eq!(
                (page.settings.radius, page.settings.opacity),
                (40., 0.05),
                "swatches share the brush settings as in the source"
            );
        });
        // Use the laid-out square canvas to exercise pointer mapping.
        let bounds = cx.update(|_, cx| view.read(cx).canvas_bounds.get());
        assert_eq!(bounds.size.width, bounds.size.height, "the demo surface is square");
        cx.simulate_mouse_down(bounds.center(), MouseButton::Left, Default::default());
        cx.simulate_mouse_up(bounds.center(), MouseButton::Left, Default::default());
        let Command::Dab(dab, _) = received.try_recv().unwrap() else {
            panic!("brush command")
        };
        assert!((dab.x - WIDTH as f32 / 2.).abs() < 1.);
        assert!((dab.y - HEIGHT as f32 / 2.).abs() < 1.);
        assert_eq!(dab.color, COLORS[2]);
        assert_eq!((dab.radius, dab.opacity), (40., 0.05));
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
    #[test]
    fn workbench_zoom_fit_compare_and_inspection_follow_the_main_canvas() {
        use gpui::{Modifiers, ScrollDelta};
        use xgpui_luma::controls::switch::SwitchEvent;
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (view, cx) =
            app.add_window_view(|_, cx| Page::new(crate::theme::default_look(), "GPU Canvas Workbench", cx));
        cx.run_until_parked();
        let bounds = cx.update(|_, cx| view.read(cx).canvas_bounds.get());
        let at = point(bounds.left() + bounds.size.width / 4., bounds.top() + bounds.size.height / 4.);
        let before = cx.update(|_, cx| view.read(cx).slot_point(at).unwrap());
        cx.simulate_event(ScrollEvent {
            position: at,
            delta: ScrollDelta::Pixels(point(px(0.), px(160.))),
            modifiers: Modifiers { control: true, ..Default::default() },
            ..Default::default()
        });
        cx.run_until_parked();
        cx.update(|_, cx| {
            let page = view.read(cx);
            let after = page.slot_point(at).unwrap();
            assert!(page.zoom > 1.);
            assert!(
                (before.x - after.x).abs() < 0.01 && (before.y - after.y).abs() < 0.01,
                "zoom stays anchored under the pointer"
            );
        });
        let fit = cx.update(|_, cx| view.read(cx).buttons[5].clone());
        fit.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        cx.update(|_, cx| assert_eq!((view.read(cx).zoom, view.read(cx).pan), (1., point(0., 0.))));
        let compare = cx.update(|_, cx| view.read(cx).compare_control.clone());
        compare.update(cx, |_, cx| cx.emit(SwitchEvent::Change { on: true }));
        cx.update(|_, cx| assert!(view.read(cx).compare));
        view.update(cx, |page, _| {
            // Test both mirrored halves against the same source coordinates.
            page.compare_active = true;
            let bounds = page.canvas_bounds.get();
            let left = point(bounds.left() + bounds.size.width / 4., bounds.center().y);
            let right = point(left.x + bounds.size.width / 2., left.y);
            assert_eq!(page.slot_point(left), page.slot_point(right));
            let snapshot = runtime::Snapshot {
                revision: 9,
                inspection_data: Some(workbench::engine::Inspection {
                    x: 20,
                    y: 40,
                    revision: 7,
                    generation: 2,
                    pixels: vec![[0.25, 0.5, 0.75, 1.]; 16],
                }),
                ..Default::default()
            };
            let properties = page.inspection_properties(&snapshot);
            assert!(properties.iter().any(|(label, value)| *label == "Revision" && value == "7 (now 9)"));
            assert!(
                properties
                    .iter()
                    .any(|(label, value)| *label == "Working linear P3" && value.contains("0.2500 0.5000 0.7500"))
            );
        });
    }
}
