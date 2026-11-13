// Shared native/Wasm view; the library owns provider scheduling and page caches.
use gpui::{prelude::*, *};
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;
use xgpui_media::pipeline::*;
use crate::theme::ColorVizLookExt;
use crate::screen::{Hero, LabCard, LabScreen, PanelShell, labeled_control, properties};
use xgpui_luma::infra::ElementExt;

pub struct TiledDemo {
    look: Arc<xgpui_luma_look_radix::Look>,
    buttons: Vec<Entity<xgpui_luma::controls::button::Button>>,
    blur_choice: crate::lab_controls::Choice,
    render_scale: f64,
    viewport_width: f64,
    pressure_switch: xgpui_luma::controls::switch::Switch,
    quality_switch: xgpui_luma::controls::switch::Switch,
    canvas: TileCanvas,
    blurs: [ImageBlur; 4],
    blur_index: usize,
    full_quality: bool,
    generation: u64,
    center: [f64; 2],
    zoom: f64,
    drag: Option<(Point<Pixels>, [f64; 2])>,
    paused: bool,
    pressure: bool,
    error: Option<String>,
    started: Instant,
    view_started: Instant,
    last_view: Option<CanvasView>,
    preview_ms: Option<f64>,
    complete_ms: Option<f64>,
    peak_cpu: u64,
    peak_gpu: u64,
}
impl TiledDemo {
    pub(super) fn requires_sdr(&self) -> bool {
        self.blur_index > 0
    }
    pub fn new(look: std::sync::Arc<xgpui_luma_look_radix::Look>, cx: &mut Context<Self>) -> Self {
        use xgpui_luma::prelude::HasPresenter;
        let blur_choice = crate::lab_controls::Choice::new(
            "tile-blur",
            "Blur radius",
            &look,
            crate::lab_controls::labels(&["0 px", "4 px", "8 px", "16 px"]),
            0,
            cx,
            |this: &mut Self, index, cx| {
                this.blur_index = index;
                cx.notify();
            },
        );
        let buttons = ["Fit", "1:1", "Zoom +", "Zoom −", "←", "→", "↑", "↓", "Replace source", "Cancel / Restart"]
            .into_iter()
            .enumerate()
            .map(|(index, label)| {
                xgpui_luma_look_radix::Button::new(format!("tile-control-{index}"))
                    .look(look.as_ref())
                    .outline()
                    .label(label)
                    .on_click(cx, move |s: &mut Self, cx| {
                        let scale = s.render_scale;
                        match index {
                            0 => {
                                s.zoom = 0.;
                                s.center = [6000., 3000.];
                            }
                            1 => s.zoom = 1.,
                            2 => s.zoom = (scale * 2.).min(4.),
                            3 => s.zoom = (scale / 2.).max(0.01),
                            4 => s.pan(-300., 0., scale),
                            5 => s.pan(300., 0., scale),
                            6 => s.pan(0., -180., scale),
                            7 => s.pan(0., 180., scale),
                            8 => s.replace(),
                            9 => {
                                if s.paused {
                                    s.replace();
                                } else {
                                    s.canvas.cancel();
                                    s.paused = true;
                                }
                            }
                            _ => {}
                        }
                        cx.notify();
                    })
                    .spawn(cx)
            })
            .collect();
        let pressure_switch = xgpui_luma_look_radix::Switch::new("tile-pressure")
            .look(look.as_ref())
            .label("2 MiB cache")
            .checked(false)
            .on_value_changed(cx, |this: &mut Self, on, cx| {
                this.pressure = on;
                this.canvas = Self::make_canvas(this.generation, on);
                this.started = Instant::now();
                this.last_view = None;
                this.preview_ms = None;
                this.complete_ms = None;
                this.peak_cpu = 0;
                this.peak_gpu = 0;
                this.paused = false;
                cx.notify();
            })
            .spawn(cx);
        let quality_switch = xgpui_luma_look_radix::Switch::new("tile-quality")
            .look(look.as_ref())
            .label("Full blur quality")
            .checked(false)
            .on_value_changed(cx, |this: &mut Self, on, cx| {
                this.full_quality = on;
                this.blurs = [0., 4., 8., 16.].map(|r| {
                    ImageBlur::new(px(r)).expect("bounded radius").with_quality(if on {
                        LayerBlurQuality::FullResolution
                    } else {
                        LayerBlurQuality::Adaptive
                    })
                });
                cx.notify();
            })
            .spawn(cx);
        Self {
            look,
            buttons,
            blur_choice,
            render_scale: 1.,
            viewport_width: 560.,
            pressure_switch,
            quality_switch,
            canvas: Self::make_canvas(0, false),
            blurs: [0., 4., 8., 16.].map(|r| ImageBlur::new(px(r)).unwrap()),
            blur_index: 0,
            full_quality: false,
            generation: 0,
            center: [6000., 3000.],
            zoom: 0.,
            drag: None,
            paused: false,
            pressure: false,
            error: None,
            started: Instant::now(),
            view_started: Instant::now(),
            last_view: None,
            preview_ms: None,
            complete_ms: None,
            peak_cpu: 0,
            peak_gpu: 0,
        }
    }
    fn make_canvas(generation: u64, pressure: bool) -> TileCanvas {
        let mut options = CanvasOptions::default();
        if pressure {
            options.budgets.cpu_bytes = 4 * 1024 * 1024;
            options.budgets.gpu_bytes = 2 * 1024 * 1024;
            options.max_cached_pages = 6;
            options.budgets.uploads_per_frame = 1;
        }
        TileCanvas::new(Arc::new(GeneratedCanvas::new(generation)), options).expect("generated canvas")
    }
    fn replace(&mut self) {
        self.generation += 1;
        self.canvas
            .replace(Arc::new(GeneratedCanvas::new(self.generation)))
            .expect("same generated descriptor");
        self.paused = false;
        self.error = None;
        self.started = Instant::now();
        self.last_view = None;
        self.preview_ms = None;
        self.complete_ms = None;
    }
    fn pan(&mut self, dx: f64, dy: f64, scale: f64) {
        self.center[0] = (self.center[0] + dx / scale).clamp(0., 12000.);
        self.center[1] = (self.center[1] + dy / scale).clamp(0., 6000.);
        self.complete_ms = None;
    }
}
impl Render for TiledDemo {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut viewport = [self.viewport_width, 380.];
        if self.blur_index > 0 {
            let side = (1020. / window.scale_factor()
                - 2. * (16. + 8. / window.scale_factor()) * MAX_LAYER_GROUPS as f32)
                .max(40.) as f64;
            viewport[0] = viewport[0].min(side);
            viewport[1] = viewport[1].min(side).min(300.);
        }
        let fit = (viewport[0] / 12000.).min(viewport[1] / 6000.);
        let scale = if self.zoom == 0. { fit } else { self.zoom };
        self.render_scale = scale;
        let view = CanvasView { center: self.center, scale, viewport, device_scale: f64::from(window.scale_factor()) };
        if self.last_view != Some(view) {
            self.last_view = Some(view);
            self.view_started = Instant::now();
            self.complete_ms = None;
        }
        if !self.paused
            && let Err(e) = self.canvas.advance(view)
        {
            self.error = Some(format!("{e:#}"));
        }
        let snapshot = self.canvas.snapshot();
        let stats = self.canvas.stats();
        if self.paused && (stats.gpu_bytes > 0 || stats.active_jobs > 0 || stats.upload_staging_bytes > 0) {
            window.request_animation_frame();
        }
        if snapshot.as_ref().is_some_and(|s| s.preview.is_some()) && self.preview_ms.is_none() {
            self.preview_ms = Some(self.started.elapsed().as_secs_f64() * 1000.);
        }
        if self.canvas.status() == &ProgressiveStatus::Ready
            && stats.ready_pages == stats.visible_pages
            && self.complete_ms.is_none()
        {
            self.complete_ms = Some(self.view_started.elapsed().as_secs_f64() * 1000.);
        }
        self.peak_cpu = self.peak_cpu.max(stats.cpu_bytes);
        self.peak_gpu = self.peak_gpu.max(stats.gpu_bytes);
        if !self.paused
            && self.error.is_none()
            && !matches!(self.canvas.status(), ProgressiveStatus::Failed(_) | ProgressiveStatus::Cancelled)
            && (self.canvas.status() != &ProgressiveStatus::Ready || stats.ready_pages < stats.visible_pages)
        {
            window.request_animation_frame();
        }
        let blur = self.blurs[self.blur_index].clone();
        let owner = cx.entity();
        let canvas = div()
            .id("tiled-viewport")
            .debug_selector(|| "media-tiled-viewport".into())
            .map(|view| {
                if self.blur_index > 0 {
                    view.w(px(viewport[0] as f32))
                } else {
                    view.w_full()
                }
            })
            .h(px(viewport[1] as f32))
            .flex_shrink_0()
            .bg(self.look.chrome().panel_background)
            .overflow_hidden()
            .cursor_crosshair()
            .on_scroll(cx.listener(move |s, event: &ScrollEvent, _, cx| {
                let delta = f64::from(f32::from(event.delta.pixel_delta(px(24.)).y));
                s.zoom = (scale * (delta * 0.002).clamp(-1., 1.).exp()).clamp(0.01, 4.);
                s.complete_ms = None;
                cx.stop_propagation();
                cx.notify();
            }))
            .on_pinch(cx.listener(move |s, event: &PinchEvent, _, cx| {
                s.zoom = (scale * f64::from((1. + event.delta).max(0.01))).clamp(0.01, 4.);
                s.complete_ms = None;
                cx.stop_propagation();
                cx.notify();
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|s, event: &MouseDownEvent, _, cx| {
                    s.drag = Some((event.position, s.center));
                    cx.stop_propagation();
                }),
            )
            .on_mouse_move(cx.listener(move |s, event: &MouseMoveEvent, _, cx| {
                if event.pressed_button == Some(MouseButton::Left) {
                    if let Some((anchor, center)) = s.drag {
                        let delta = event.position - anchor;
                        s.center = [
                            (center[0] - f64::from(f32::from(delta.x)) / scale).clamp(0., 12000.),
                            (center[1] - f64::from(f32::from(delta.y)) / scale).clamp(0., 6000.),
                        ];
                        s.complete_ms = None;
                        cx.notify();
                    }
                } else {
                    s.drag = None;
                }
            }))
            .on_mouse_up(MouseButton::Left, cx.listener(|s, _, _, _| s.drag = None))
            .on_mouse_up_out(MouseButton::Left, cx.listener(|s, _, _, _| s.drag = None))
            .child(
                gpui::canvas(
                    |_, _, _| (),
                    move |bounds, (), window, _| {
                        if let Some(snapshot) = &snapshot {
                            blur.paint(bounds, window, |window| snapshot.paint(bounds, window))
                                .expect("bounded blur viewport");
                        }
                    },
                )
                .size_full(),
            );
        let canvas = div()
            .w_full()
            .on_prepaint(move |bounds, _, cx| {
                owner.update(cx, |this, cx| {
                    let width = f64::from(f32::from(bounds.size.width));
                    if this.viewport_width != width {
                        this.viewport_width = width;
                        cx.notify();
                    }
                });
            })
            .child(canvas);
        let mib = |bytes: u64| bytes as f64 / (1024. * 1024.);
        let screen = LabScreen::new("media-tiled", self.look.clone())
            .crumb("xgpui-media", "Tiled Canvas")
            .title("Tiled Canvas")
            .lede("Explore a large image that sharpens as you zoom in and move around.")
            .hero(Hero::full(canvas))
            .caption(format!(
                "12,000×6,000 · generation {} · {:?} · zoom {:.1}% · mip {} · ready {}/{}",
                self.generation,
                self.canvas.status(),
                scale * 100.,
                stats.mip,
                stats.ready_pages,
                stats.visible_pages
            ))
            .control(labeled_control(&self.look, self.blur_choice.clone().into_element()))
            .control(self.pressure_switch.clone())
            .control(self.quality_switch.clone());
        self.buttons.iter().cloned().fold(screen, LabScreen::action)
            .notice(self.error.clone())
            .cards([
                LabCard::new("Pages").values([("cached", stats.cached_pages.to_string()), ("jobs", stats.active_jobs.to_string())]),
                LabCard::new("Payload reservations").values([("CPU", format!("{:.2} MiB", mib(stats.cpu_bytes))), ("GPU", format!("{:.2} MiB", mib(stats.gpu_bytes)))]),
                LabCard::new("Readiness").values([("preview", self.preview_ms.map_or("pending".into(), |v| format!("{v:.1} ms"))), ("view submitted", self.complete_ms.map_or("refining".into(), |v| format!("{v:.1} ms")))]),
            ])
            .panels(
                PanelShell::new("Transfers and cache").child(properties(&self.look, [
                    ("Cache budget", format!("{} MiB", if self.pressure {2} else {16})),
                    ("Pending transfer", format!("{:.2} MiB", mib(stats.pending_staging_bytes))),
                    ("Upload staging", format!("{:.2} MiB", mib(stats.upload_staging_bytes))),
                    ("Staging pool", format!("{:.2} MiB", mib(stats.staging_pool_bytes))),
                    ("Peak CPU / GPU", format!("{:.2} / {:.2} MiB", mib(self.peak_cpu), mib(self.peak_gpu))),
                ])),
                PanelShell::new("Provider activity").child(properties(&self.look, [
                    ("Requests", stats.requests.to_string()), ("Evictions", stats.evictions.to_string()),
                    ("Cancelled", stats.cancelled_requests.to_string()), ("Stale", stats.stale_results.to_string()),
                    ("Failures", stats.failures.to_string()), ("Presentation", if self.paused {"cancelled"} else {"active"}.into()),
                ])),
            )
            .note("Drag to pan; wheel or pinch to zoom. The preview stays visible while selected tiles refine. Blur runs once over assembled pages, with transparent pixels outside the viewport; retained blur targets are additional to page budgets.")
            .credits(if cfg!(target_arch="wasm32") {
                "Generated 72 megapixel source. Browser: one generated page per frame on the host thread. No file decoder or browser worker is measured. Grid boundaries expose seams; real assets need prefiltered pyramid data."
            } else {
                "Generated 72 megapixel source, with bounded provider workers and background staging. Statistics count payload reservations, not process RSS or driver VRAM. Old scenes remain charged until GPU completion. The provider samples mip centers; real assets need prefiltered pyramid data."
            })
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use xgpui_luma::{controls::button::ButtonEvent, theme::ThemeMode};

    #[::core::prelude::v1::test]
    fn tiled_screen_fits_both_themes_and_retains_zoom_controls() {
        for mode in [ThemeMode::Dark, ThemeMode::Light] {
            let mut app = TestAppContext::single();
            app.update(|cx| xgpui_luma::init(cx).unwrap());
            let look = crate::theme::default_look();
            look.set_mode(mode);
            let (page, cx) = app.add_window_view(|_, cx| {
                let mut page = TiledDemo::new(look, cx);
                // Layout/control test: do not schedule provider work or native uploads.
                page.canvas.cancel();
                page.paused = true;
                page
            });
            for width in [1280., 720.] {
                cx.simulate_resize(size(px(width), px(1600.)));
                cx.run_until_parked();
                let hero = cx.debug_bounds("media-tiled-hero").unwrap();
                assert!(hero.size.width <= px(width));
                assert!(cx.debug_bounds("media-tiled-panels").is_some());
                cx.update(|_, cx| {
                    assert!((page.read(cx).viewport_width - f64::from(f32::from(hero.size.width))).abs() < 1.)
                });
            }
            page.update(cx, |page, cx| {
                page.blur_index = 1;
                cx.notify();
            });
            cx.run_until_parked();
            let blurred_width = cx.debug_bounds("media-tiled-viewport").unwrap().size.width;
            page.update(cx, |page, cx| {
                page.blur_index = 0;
                cx.notify();
            });
            cx.run_until_parked();
            let restored = cx.debug_bounds("media-tiled-viewport").unwrap();
            assert!(restored.size.width >= blurred_width);
            assert_eq!(restored.size.width, cx.debug_bounds("media-tiled-hero").unwrap().size.width);
            let one_to_one = cx.update(|_, cx| page.read(cx).buttons[1].clone());
            one_to_one.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
            cx.run_until_parked();
            cx.update(|_, cx| assert_eq!(page.read(cx).render_scale, 1.));
            let fit = cx.update(|_, cx| page.read(cx).buttons[0].clone());
            fit.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
            cx.run_until_parked();
            cx.update(|_, cx| assert!(page.read(cx).render_scale < 1.));
        }
    }
}
