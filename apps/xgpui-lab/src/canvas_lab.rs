//! Canvas and Mandelbrot adapted from xgpui-ext-lab 74b8e4e; Luma controls and retained app lifecycle.
//! Both pages are `LabScreen`s (see `screen.rs`) around one canvas viewport.
mod interaction;
mod mandelbrot;
mod minimap;
mod overview_map;
mod palette;
mod profile;
mod screen;
use interaction::Gesture;

use gpui::{prelude::*, *};
use std::{collections::BTreeSet, path::PathBuf, sync::Arc, time::Instant};
use xgpui_luma_look_radix::Look;

use crate::screen::TextRole;
use crate::theme::ColorVizLookExt;
use xgpui_canvas::{
    Camera2D, Canvas as SpatialCanvas, GridPainter, GridStyle, LevelOfDetail, Point as WorldPoint, Scene, SceneItem,
    WorldBounds,
};

pub struct Lab {
    look: Arc<Look>,
    controls: screen::Controls,
    active: bool,
    results: Option<PathBuf>,
    /// Sweep progress and errors, shown as a notice.
    status: Option<String>,
    fractal: mandelbrot::Mandelbrot,
    profile: Option<profile::Profile>,
    smoke: Option<(PathBuf, usize, bool)>,
    grid: GridPainter,
    grid_style: GridStyle,
    scene: Scene<usize>,
    preview: minimap::Preview,
    overview: Arc<xgpui_canvas::Overview>,
    minimap_image: Arc<RenderImage>,
    overview_key: (usize, u64),
    map_thumbnail: bool,
    map_drag: bool,
    whole_scene_map: bool,
    map_bounds: Bounds<Pixels>,
    show_minimap: bool,
    focus: FocusHandle,
    space_down: bool,
    camera: Camera2D,
    bounds: Bounds<Pixels>,
    drag: xgpui_canvas::PanGesture,
    selected: BTreeSet<usize>,
    gesture: Option<Gesture>,
    snap_guides: Vec<xgpui_canvas::AlignmentGuide>,
}

impl Lab {
    fn viewport(&self) -> WorldPoint {
        WorldPoint::new(f64::from(f32::from(self.bounds.size.width)), f64::from(f32::from(self.bounds.size.height)))
    }
}

impl Lab {
    fn reset_view(&mut self) {
        self.cancel_interaction();
        self.space_down = false;
        self.camera = Camera2D::default();
        self.invalidate_preview();
    }
}

impl Render for Lab {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.active {
            return div().into_any_element();
        }
        let started = Instant::now();
        self.check_mandelbrot(window, cx);
        let smoke_viewport = self.viewport();
        if let Some((directory, stored_tick, pending)) = &mut self.smoke
            && !*pending
            && !self.preview.capturing
        {
            *pending = true;
            let tick = *stored_tick + 1;
            let scenario = (tick - 1) / 8;
            if (tick - 1) % 8 == 0 {
                self.camera = Camera2D::default();
                match scenario {
                    1 => self.camera.zoom_at(WorldPoint::new(137., 247.), smoke_viewport, 1e-10),
                    2 => self.camera.zoom_at(WorldPoint::new(137., 247.), smoke_viewport, 1e10),
                    3 => self.camera.pan_by(WorldPoint::new(1e10 + 13., -1e10 + 7.)),
                    4 => window.resize(size(px(800.), px(600.))),
                    5 => self.camera.zoom_at(
                        WorldPoint::new(smoke_viewport.x / 2., smoke_viewport.y / 2.),
                        smoke_viewport,
                        0.1,
                    ),
                    _ => (),
                }
            }
            let capture = (tick % 8 == 0).then(|| directory.join(format!("{scenario}.png")));
            let preview = (tick % 8 == 3 && (6..=7).contains(&scenario)).then_some(scenario == 6);
            let done = tick >= 64;
            let overview_capture = directory.join("whole-canvas.png");
            let entity = cx.entity();
            window.on_next_frame(move |window, cx| {
                if let Some(marquee) = preview {
                    interaction::smoke_preview(&entity, window, cx, marquee);
                }
                if let Some(path) = capture {
                    smoke_click(&entity, window, cx);
                    if scenario == 0 || scenario == 5 {
                        interaction::smoke_marquee(&entity, window, cx);
                    }
                    window.render_to_image().expect("capture smoke frame").save(path).expect("save smoke frame");
                }
                if done {
                    entity.update(cx, |lab, _| {
                        lab.whole_scene_map = true;
                        lab.refresh_overview();
                    });
                    window.draw(cx).clear(cx);
                    overview_map::smoke_navigation(&entity, window, cx);
                    overview_map::smoke_invalidation_and_modes(&entity, window, cx);
                    window.draw(cx).clear(cx);
                    window.render_to_image().expect("capture minimap").save(overview_capture).expect("save minimap");
                    cx.quit();
                } else {
                    entity.update(cx, |this, cx| {
                        let smoke = this.smoke.as_mut().unwrap();
                        smoke.1 = tick;
                        smoke.2 = false;
                        cx.notify();
                    });
                }
            });
            window.request_animation_frame();
        }
        if let Some(profile) = &mut self.profile
            && !self.preview.capturing
        {
            let count = profile.begin(started);
            if self.scene.len() != count {
                self.scene = make_scene(count);
            }
            if profile.frame == 301 {
                self.camera = Camera2D::default();
                self.camera.zoom_at(
                    WorldPoint::new(smoke_viewport.x / 2., smoke_viewport.y / 2.),
                    smoke_viewport,
                    0.01,
                );
            }
            self.grid_style = if self.scene.len() == 10_000 {
                GridStyle::Lines
            } else {
                GridStyle::Dots
            };
            self.camera.pan_by(WorldPoint::new(1., 0.5));
        }
        let viewport = self.viewport();
        let zoom = self.camera.zoom() as f32;
        let mut cards = Vec::new();
        let mut proxies = Vec::new();
        let mut point_count = 0;
        for entry in self.scene.visible(self.camera, viewport).filter(|_| !self.fractal.enabled) {
            let id = entry.item.data;
            let screen = entry.screen_bounds.min;
            if entry.detail != LevelOfDetail::Full {
                let mut bounds = Bounds::new(
                    point(px(screen.x as f32), px(screen.y as f32)),
                    size(
                        px((entry.screen_bounds.max.x - screen.x) as f32),
                        px((entry.screen_bounds.max.y - screen.y) as f32),
                    ),
                );
                if entry.detail == LevelOfDetail::Point {
                    point_count += 1;
                    // A small filled proxy at the object's center, never a 1px border.
                    let center = bounds.center();
                    bounds.size = size(px(1.), px(1.));
                    bounds.origin = center - point(px(0.5), px(0.5));
                }
                proxies.push((bounds, self.selected.contains(&id)));
                continue;
            }
            cards.push(
                div()
                    .id(id)
                    .absolute()
                    .left(px(screen.x as f32))
                    .top(px(screen.y as f32))
                    .w(px(160. * zoom))
                    .h(px(80. * zoom))
                    .overflow_hidden()
                    .bg(rgb(if self.selected.contains(&id) {
                        palette::CARD_SELECTED
                    } else {
                        palette::CARD
                    }))
                    .border_1()
                    .border_color(rgb(palette::CARD_EDGE))
                    .text_size(px(16. * zoom))
                    .text_color(rgb(palette::CARD_TEXT))
                    .child(div().absolute().top_0().left_0().child(format!("Card {id}"))),
            );
        }
        let full_count = cards.len();
        let proxy_count = proxies.len();
        let count = full_count + proxy_count;
        if let Some(profile) = &mut self.profile
            && !self.preview.capturing
        {
            profile.record_build(started.elapsed().as_secs_f64() * 1000., count, full_count);
            let capture = match profile.frame {
                20 => Some(profile.output.with_extension("10k.png")),
                170 => Some(profile.output.with_extension("100k.png")),
                320 => Some(profile.output.with_extension("overview.png")),
                _ => None,
            };
            let done = profile.frame >= 450;
            let entity = cx.entity();
            window.on_next_frame(move |window, cx| {
                if let Some(path) = capture {
                    window.render_to_image().expect("capture canvas frame").save(path).expect("save canvas screenshot");
                }
                entity.update(cx, |this, cx| {
                    if done {
                        this.profile.as_ref().unwrap().finish(
                            [f32::from(this.bounds.size.width), f32::from(this.bounds.size.height)],
                            window.scale_factor(),
                        );
                        this.status = Some("Canvas sweep completed; results include captures and timing JSON.".into());
                        this.profile = None;
                        cx.notify();
                    } else {
                        cx.notify();
                    }
                });
            });
            window.request_animation_frame();
        }
        let entity = cx.entity();
        let marquee = self.marquee();
        let guides = self.snap_guides.clone();
        self.schedule_mandelbrot(window, cx);
        let fractal = self.fractal.enabled.then(|| self.fractal_element());
        if self.fractal.ready_for_capture()
            && let Some(path) = self.fractal.capture.take()
        {
            window.on_next_frame(move |window, cx| {
                window.render_to_image().expect("capture Mandelbrot").save(path).expect("save Mandelbrot");
                cx.quit();
            });
            window.request_animation_frame();
        }
        if self.show_minimap && self.whole_scene_map && self.overview_key != self.scene.cache_key() {
            self.refresh_overview();
        }
        minimap::schedule(self, window, cx);
        let minimap = (self.show_minimap && !self.preview.hidden).then(|| {
            if self.whole_scene_map {
                self.overview_map(cx)
            } else {
                self.minimap(cx)
            }
        });
        let wide = window.color_status().effective() == WindowColorMode::WideGamutSdr;
        let viewport = self.viewport_element(entity, cards, proxies, fractal, minimap, marquee, guides, cx);
        let drawn = screen::Drawn { cards: full_count, proxies: proxy_count, points: point_count };
        let screen = self.screen(viewport, drawn, wide, cx);
        // Keys act anywhere on the page, including while one of its controls has focus.
        div()
            .id("canvas-lab")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                if event.keystroke.key == "space" {
                    this.space_down = true;
                    cx.stop_propagation();
                }
                if event.keystroke.key == "g" {
                    this.grid_style = match this.grid_style {
                        GridStyle::Lines => GridStyle::Dots,
                        GridStyle::Dots => GridStyle::Lines,
                    };
                    cx.notify();
                }
                // No page-swap key: Spatial Canvas and Mandelbrot each have a navigation entry,
                // and a key that swapped them would leave the navigation showing the wrong page.
                if event.keystroke.key == "t" && !this.fractal.enabled {
                    this.cancel_interaction();
                    this.whole_scene_map = !this.whole_scene_map;
                    cx.notify();
                }
                if event.keystroke.key == "m" {
                    this.cancel_interaction();
                    this.show_minimap = !this.show_minimap;
                    cx.notify();
                }
                if event.keystroke.key == "escape" {
                    this.cancel_interaction();
                    cx.notify();
                }
                if event.keystroke.key == "r" {
                    this.reset_view();
                    cx.notify();
                }
                if event.keystroke.key == "o" {
                    this.cancel_interaction();
                    this.overview_zoom();
                    cx.notify();
                }
                if event.keystroke.key == "b" {
                    this.cancel_interaction();
                    this.set_scene_size(if this.scene.len() == 10_000 { 100_000 } else { 10_000 });
                    cx.notify();
                }
            }))
            .on_key_up(cx.listener(|this, event: &KeyUpEvent, _, cx| {
                if event.keystroke.key == "space" {
                    this.space_down = false;
                    this.drag.end();
                    cx.stop_propagation();
                }
            }))
            .size_full()
            .child(screen)
            .into_any_element()
    }
}

impl Lab {
    /// The canvas viewport: the hero of both pages. Pointer input here pans, zooms and edits the scene.
    #[allow(clippy::too_many_arguments)]
    fn viewport_element(
        &self,
        entity: Entity<Self>,
        cards: Vec<Stateful<Div>>,
        proxies: Vec<(Bounds<Pixels>, bool)>,
        fractal: Option<AnyElement>,
        minimap: Option<AnyElement>,
        marquee: Option<WorldBounds>,
        guides: Vec<xgpui_canvas::AlignmentGuide>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let camera = self.camera;
        div()
            .id("viewport")
            .relative()
            .w_full()
            .h(px(screen::CANVAS_HEIGHT))
            .flex_shrink_0()
            .overflow_hidden()
            .bg(rgb(palette::STAGE))
            .border_1()
            .border_color(self.look.chrome().border)
            .on_scroll(cx.listener(|this, event: &ScrollEvent, _, cx| {
                this.cancel_interaction();
                let delta = event.delta.pixel_delta(px(24.));
                this.zoom_view(event.position, (f64::from(f32::from(delta.y)) * 0.005).clamp(-1., 1.).exp());
                cx.stop_propagation();
                cx.notify();
            }))
            .on_pinch(cx.listener(|this, event: &PinchEvent, _, cx| {
                this.cancel_interaction();
                this.zoom_view(event.position, (1. + f64::from(event.delta)).max(0.01));
                cx.stop_propagation();
                cx.notify();
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, window, cx| {
                    this.focus.focus(window, cx);
                    let hit_card = this.scene.hit_test(this.world_at(event.position)).is_some();
                    if interaction::primary_drag_pans(
                        this.fractal.enabled,
                        this.space_down,
                        event.modifiers.shift,
                        hit_card,
                    ) {
                        this.cancel_interaction();
                        if !this.fractal.enabled && !this.space_down && !hit_card {
                            this.selected.clear();
                        }
                        this.drag.begin(WorldPoint::new(
                            f32::from(event.position.x) as f64,
                            f32::from(event.position.y) as f64,
                        ));
                    } else {
                        this.start_selection(event);
                    }
                    cx.notify();
                }),
            )
            .on_mouse_down(
                MouseButton::Middle,
                cx.listener(|this, event: &MouseDownEvent, _, _| {
                    this.cancel_interaction();
                    this.drag
                        .begin(WorldPoint::new(f32::from(event.position.x) as f64, f32::from(event.position.y) as f64));
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                if this.gesture.is_some() && event.pressed_button == Some(MouseButton::Left) {
                    this.update_selection(event.position, event.modifiers.alt);
                    cx.notify();
                    return;
                }
                if event.pressed_button != Some(MouseButton::Middle) && event.pressed_button != Some(MouseButton::Left)
                {
                    this.drag.end();
                } else if let Some(delta) = this
                    .drag
                    .update(WorldPoint::new(f32::from(event.position.x) as f64, f32::from(event.position.y) as f64))
                {
                    this.camera.pan_by(delta);
                    cx.notify();
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, event: &MouseUpEvent, _, cx| {
                    this.finish_selection(event);
                    cx.notify();
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, event: &MouseUpEvent, _, cx| {
                    this.finish_selection(event);
                    cx.notify();
                }),
            )
            .on_mouse_up(MouseButton::Middle, cx.listener(|this, _, _, _| this.drag.end()))
            .on_mouse_up_out(MouseButton::Middle, cx.listener(|this, _, _, _| this.drag.end()))
            .child(
                SpatialCanvas::new(self.camera, &self.grid, self.grid_style)
                    .size_full()
                    .child(
                        canvas(
                            move |bounds, _, cx| {
                                entity.update(cx, |this, cx| {
                                    if this.bounds != bounds {
                                        this.bounds = bounds;
                                        cx.notify();
                                    }
                                });
                            },
                            |_, _, _, _| {},
                        )
                        .absolute()
                        .size_full(),
                    )
                    .child(
                        canvas(
                            |_, _, _| (),
                            move |bounds, _, window, _| {
                                for (mut proxy, selected) in proxies {
                                    proxy.origin += bounds.origin;
                                    window.paint_quad(fill(
                                        proxy,
                                        rgb(if selected {
                                            palette::PROXY_SELECTED
                                        } else {
                                            palette::PROXY
                                        }),
                                    ));
                                }
                            },
                        )
                        .absolute()
                        .size_full(),
                    )
                    .children(cards)
                    .children(fractal)
                    .child(
                        canvas(
                            |_, _, _| (),
                            move |bounds, _, window, _| {
                                interaction::paint_overlay(window, bounds, camera, marquee, &guides)
                            },
                        )
                        .absolute()
                        .size_full(),
                    ),
            )
            .children(minimap)
            .into_any_element()
    }
}

// Dispatch real GPUI input against the rendered viewport in native smoke mode.
fn smoke_click(entity: &Entity<Lab>, window: &mut Window, cx: &mut App) {
    let target = {
        let lab = entity.read(cx);
        let viewport = lab.viewport();
        lab.scene.visible(lab.camera, viewport).next().map(|entry| {
            let rect = entry.screen_bounds;
            let local = WorldPoint::new(
                (rect.min.x.max(0.) + rect.max.x.min(viewport.x)) / 2.,
                (rect.min.y.max(0.) + rect.max.y.min(viewport.y)) / 2.,
            );
            let expected = lab.scene.hit_test(lab.camera.screen_to_world(local, viewport)).unwrap().data;
            (lab.bounds.origin + point(px(local.x as f32), px(local.y as f32)), expected)
        })
    };
    if let Some((position, expected)) = target {
        window.dispatch_event(PlatformInput::MouseMove(MouseMoveEvent { position, ..Default::default() }), cx);
        window.dispatch_event(
            PlatformInput::MouseDown(MouseDownEvent {
                position,
                button: MouseButton::Left,
                click_count: 1,
                ..Default::default()
            }),
            cx,
        );
        assert!(entity.read(cx).selected.contains(&expected), "native viewport selection");
        window.dispatch_event(
            PlatformInput::MouseUp(MouseUpEvent { position, button: MouseButton::Left, ..Default::default() }),
            cx,
        );
    }
}

fn make_scene(count: usize) -> Scene<usize> {
    Scene::new((0..count).map(|id| {
        let min = WorldPoint::new((id % 100) as f64 * 200. - 10000., (id / 100) as f64 * 120. - 6000.);
        SceneItem { bounds: WorldBounds { min, max: WorldPoint::new(min.x + 160., min.y + 80.) }, data: id }
    }))
}

impl Lab {
    pub fn new(look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let controls = screen::Controls::new(&look, cx);
        let scene = make_scene(10_000);
        let overview = Arc::new(xgpui_canvas::Overview::new(&scene));
        let minimap_image = overview_map::thumbnail(&scene, &overview);
        Self {
            look,
            controls,
            active: true,
            results: None,
            status: None,
            fractal: mandelbrot::Mandelbrot::new(false, None, false, None),
            preview: minimap::Preview::default(),
            overview,
            minimap_image,
            overview_key: scene.cache_key(),
            map_thumbnail: true,
            map_drag: false,
            whole_scene_map: true,
            map_bounds: Bounds::default(),
            show_minimap: true,
            profile: None,
            smoke: None,
            grid: GridPainter::new().expect("canvas grid"),
            grid_style: GridStyle::Lines,
            scene,
            focus: cx.focus_handle(),
            space_down: false,
            camera: Camera2D::default(),
            bounds: Bounds::default(),
            drag: xgpui_canvas::PanGesture::default(),
            selected: BTreeSet::new(),
            gesture: None,
            snap_guides: Vec::new(),
        }
    }
    pub fn activate(&mut self, fractal: bool, cx: &mut Context<Self>) {
        self.active = true;
        if self.profile.is_none() {
            self.status = None;
        }
        if self.fractal.enabled != fractal {
            self.fractal.toggle();
            self.reset_view();
            self.show_minimap = !fractal;
            self.whole_scene_map = !fractal;
        }
        cx.notify();
    }
    pub fn ready_for_capture(&self) -> bool {
        !self.fractal.enabled || self.fractal.ready_for_capture()
    }
    pub fn suspend(&mut self) {
        self.active = false;
        self.cancel_interaction();
        self.space_down = false;
        if self.profile.take().is_some() {
            self.status = Some("Sweep cancelled when leaving the pane; partial captures retained.".into());
        }
        self.fractal.suspend();
    }
}

#[cfg(all(test, feature = "test-support"))]
mod luma_tests {
    use super::*;
    use xgpui_luma::controls::button::ButtonEvent;
    #[::core::prelude::v1::test]
    fn luma_canvas_controls_change_scene_and_suspend_pending_work() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (page, cx) = app.add_window_view(|_, cx| Lab::new(crate::theme::default_look(), cx));
        cx.run_until_parked();
        let (scene, reset) =
            cx.update(|_, cx| (page.read(cx).controls.scene.clone(), page.read(cx).controls.reset.clone()));
        cx.update(|_, cx| scene.select(1, cx));
        page.update(cx, |page, _| assert_eq!(page.scene.len(), 100_000));
        page.update(cx, |page, cx| {
            page.activate(true, cx);
            assert!(page.fractal.enabled);
            page.suspend();
            assert!(!page.active);
            assert!(page.gesture.is_none());
        });
        reset.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        page.update(cx, |page, _| assert_eq!(page.camera.zoom(), 1.));
    }

    #[::core::prelude::v1::test]
    fn both_pages_render_as_lab_screens() {
        let mut app = TestAppContext::single();
        app.update(|cx| xgpui_luma::init(cx).unwrap());
        let (page, cx) = app.add_window_view(|_, cx| Lab::new(crate::theme::default_look(), cx));
        cx.run_until_parked();
        for slot in [
            "spatial-canvas-crumb",
            "spatial-canvas-title",
            "spatial-canvas-hero",
            "spatial-canvas-caption",
            "spatial-canvas-controls",
            "spatial-canvas-cards",
            "spatial-canvas-note",
            "spatial-canvas-credits",
        ] {
            assert!(cx.debug_bounds(slot).is_some(), "{slot}");
        }
        let hero = cx.debug_bounds("spatial-canvas-hero").unwrap();
        assert_eq!(hero.size.height, px(screen::CANVAS_HEIGHT));
        page.update(cx, |page, cx| page.activate(true, cx));
        cx.run_until_parked();
        for slot in [
            "mandelbrot-title",
            "mandelbrot-hero",
            "mandelbrot-caption",
            "mandelbrot-controls",
            "mandelbrot-cards",
        ] {
            assert!(cx.debug_bounds(slot).is_some(), "{slot}");
        }
        assert!(cx.debug_bounds("spatial-canvas-hero").is_none());
    }
}
