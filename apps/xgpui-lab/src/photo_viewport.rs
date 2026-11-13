//! One retained photo, ordinary controls, and sampling-only GPU adjustments.
use crate::theme::ColorVizLookExt;
use crate::screen::{Hero, LabCard, LabScreen, PanelShell, properties};
use xgpui_luma::infra::ElementExt;
use anyhow::{Context as _, Result, ensure};
use gpui::{prelude::*, *};
use xgpui_luma::prelude::HasPresenter;
use xgpui_luma_look_radix::Look;
use image::ImageDecoder;
use std::{path::PathBuf, sync::Arc};
use xgpui_media::pipeline::*;

const MIB: usize = 1024 * 1024;
const SOURCE_BYTES: usize = 128 * MIB;

pub struct PhotoViewport {
    look: Arc<Look>,
    buttons: Vec<Entity<xgpui_luma::controls::button::Button>>,
    wide_switch: xgpui_luma::controls::switch::Switch,
    viewport_width: f64,
    source: Option<Arc<RasterSource>>,
    canvas: Option<TileCanvas>,
    loading: Option<Task<()>>,
    generation: u64,
    name: String,
    error: Option<String>,
    center: [f64; 2],
    zoom: f64,
    scale: f64,
    device_scale: f64,
    exposure: f64,
    wide: bool,
    active: bool,
    drag: Option<(Point<Pixels>, [f64; 2])>,
}

fn decode(path: Option<PathBuf>) -> Result<RasterSource> {
    let mut reader = if let Some(path) = path {
        ensure!(std::fs::metadata(&path)?.len() <= 64 * MIB as u64, "encoded photo exceeds 64 MiB");
        image::ImageReader::new(std::io::Cursor::new(std::fs::read(path)?)).with_guessed_format()?
    } else {
        image::ImageReader::new(std::io::Cursor::new(
            include_bytes!("../assets/colorimetry/photos/Italy-sRGB.jpg").to_vec(),
        ))
        .with_guessed_format()?
    };
    let mut limits = image::Limits::default();
    limits.max_alloc = Some((256 * MIB) as u64);
    reader.limits(limits);
    let mut decoder = reader.into_decoder()?;
    ensure!(decoder.total_bytes() <= SOURCE_BYTES as u64, "decoded photo exceeds 128 MiB");
    ensure!(
        matches!(
            decoder.color_type(),
            image::ColorType::Rgb8 | image::ColorType::Rgba8 | image::ColorType::L8 | image::ColorType::La8
        ),
        "photo preview currently accepts 8-bit images"
    );
    let (w, h) = decoder.dimensions();
    ensure!(u64::from(w) * u64::from(h) * 4 <= SOURCE_BYTES as u64, "RGBA source exceeds 128 MiB");
    let color = if let Some(profile) = decoder.icc_profile()? {
        PixelColor::Custom {
            rgb: Arc::new(
                xgpui_color::icc::IccRgbProfile::parse(&profile)
                    .context("photo requires a supported RGB matrix/TRC ICC profile")?
                    .into_rgb_definition(),
            ),
            alpha: AlphaMode::Straight,
        }
    } else {
        ImageColor::SRGB.into()
    };
    let orientation = decoder.orientation()?;
    let mut image = image::DynamicImage::from_decoder(decoder)?;
    image.apply_orientation(orientation);
    RasterSource::new(image.into_rgba8(), color, SOURCE_BYTES)
}
fn make_canvas(source: Arc<RasterSource>) -> Result<TileCanvas> {
    TileCanvas::new(
        source,
        CanvasOptions {
            budgets: TileBudgets {
                cpu_bytes: 64 * MIB as u64,
                gpu_bytes: 128 * MIB as u64,
                staging_bytes: 16 * MIB as u64,
                upload_bytes_per_frame: 8 * MIB as u64,
                uploads_per_frame: 4,
                decode_jobs: 2,
            },
            max_cached_pages: 256,
            ..Default::default()
        },
    )
}
impl PhotoViewport {
    pub fn new(look: Arc<Look>, cx: &mut Context<Self>) -> Self {
        let buttons = ["Open photo…", "Fit", "1:1", "Exposure −", "Exposure +"]
            .into_iter()
            .enumerate()
            .map(|(index, label)| {
                xgpui_luma_look_radix::Button::new(format!("photo-{index}"))
                    .look(look.as_ref())
                    .outline()
                    .label(label)
                    .on_click(cx, move |this: &mut Self, cx| {
                        match index {
                            0 => {
                                this.open(cx);
                                return;
                            }
                            1 => {
                                this.zoom = 0.;
                                this.recenter();
                            }
                            2 => this.zoom = 1. / this.device_scale,
                            3 => this.exposure = (this.exposure - 0.25).max(-2.),
                            4 => this.exposure = (this.exposure + 0.25).min(2.),
                            _ => this.wide = !this.wide,
                        }
                        cx.notify();
                    })
                    .spawn(cx)
            })
            .collect();
        let wide_switch = xgpui_luma_look_radix::Switch::new("photo-wide")
            .look(look.as_ref())
            .label("Wide output")
            .checked(true)
            .on_value_changed(cx, |this: &mut Self, on, cx| {
                this.wide = on;
                cx.notify();
            })
            .spawn(cx);
        let mut this = Self {
            look,
            buttons,
            wide_switch,
            viewport_width: 560.,
            source: None,
            canvas: None,
            loading: None,
            generation: 0,
            name: String::new(),
            error: None,
            center: [0.; 2],
            zoom: 0.,
            scale: 1.,
            device_scale: 1.,
            exposure: 0.,
            wide: true,
            active: true,
            drag: None,
        };
        let args: Vec<_> = std::env::args().collect();
        let path = args.windows(2).find(|a| a[0] == "--photo").map(|a| PathBuf::from(&a[1]));
        this.load(path, cx);
        this
    }
    fn recenter(&mut self) {
        if let Some(source) = &self.source {
            self.center = [source.image().width() as f64 / 2., source.image().height() as f64 / 2.];
        }
    }
    fn load(&mut self, path: Option<PathBuf>, cx: &mut Context<Self>) {
        self.generation += 1;
        let generation = self.generation;
        let name = path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Italy — sRGB".into());
        self.error = None;
        let task = cx.background_spawn(async move { decode(path) });
        self.loading = Some(cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                if this.generation != generation {
                    return;
                }
                match result.and_then(|source| {
                    let source = Arc::new(source);
                    let canvas = if let Some(mut canvas) = this.canvas.take() {
                        canvas.replace(source.clone())?;
                        Some(canvas)
                    } else if this.active {
                        Some(make_canvas(source.clone())?)
                    } else {
                        None
                    };
                    Ok((source, canvas))
                }) {
                    Ok((source, canvas)) => {
                        this.source = Some(source);
                        this.canvas = canvas;
                        this.name = name;
                        this.zoom = 0.;
                        this.recenter();
                    }
                    Err(error) => this.error = Some(format!("{error:#}")),
                }
                this.loading = None;
                cx.notify();
            });
        }));
        cx.notify();
    }
    fn open(&mut self, cx: &mut Context<Self>) {
        let picker = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Open photo".into()),
        });
        cx.spawn(async move |this, cx| {
            if let Ok(Ok(Some(paths))) = picker.await
                && let Some(path) = paths.into_iter().next()
            {
                let _ = this.update(cx, |this, cx| this.load(Some(path), cx));
            }
        })
        .detach();
    }
    pub fn suspend(&mut self) {
        self.active = false;
        if let Some(mut canvas) = self.canvas.take() {
            canvas.cancel();
        }
    }
    pub fn activate(&mut self, cx: &mut Context<Self>) {
        self.active = true;
        if self.canvas.is_none()
            && let Some(source) = &self.source
        {
            match make_canvas(source.clone()) {
                Ok(canvas) => self.canvas = Some(canvas),
                Err(error) => self.error = Some(error.to_string()),
            }
        }
        cx.notify();
    }
}
impl Render for PhotoViewport {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.device_scale = window.scale_factor() as f64;
        let viewport =
            [self.viewport_width, (f32::from(window.viewport_size().height) - 340.).clamp(180., 560.) as f64];
        let mut status = if self.loading.is_some() {
            "Loading photo…".into()
        } else {
            self.name.clone()
        };
        let canvas_entity = cx.entity().downgrade();
        if let (Some(source), Some(canvas)) = (&self.source, &mut self.canvas) {
            let image = source.image();
            self.scale = if self.zoom == 0. {
                (viewport[0] / image.width() as f64).min(viewport[1] / image.height() as f64)
            } else {
                self.zoom
            };
            if let Err(error) = canvas.advance(CanvasView {
                center: self.center,
                scale: self.scale,
                viewport,
                device_scale: window.scale_factor() as f64,
            }) {
                self.error = Some(error.to_string());
            }
            let stats = canvas.stats();
            status = format!(
                "{} · {} × {} · {:.0}% · {:+.2} EV · {:.1} MiB source · {:.1} MiB pages",
                self.name,
                image.width(),
                image.height(),
                self.scale * self.device_scale * 100.,
                self.exposure,
                source.source_bytes() as f64 / MIB as f64,
                stats.gpu_bytes as f64 / MIB as f64
            );
            if !matches!(
                canvas.status(),
                ProgressiveStatus::Ready | ProgressiveStatus::Failed(_) | ProgressiveStatus::Cancelled
            ) || stats.ready_pages < stats.visible_pages
            {
                window.request_animation_frame();
            }
            if let ProgressiveStatus::Failed(error) = canvas.status() {
                self.error = Some(error.clone());
            }
        }
        let wide_available = window.color_status().effective() == WindowColorMode::WideGamutSdr;
        let paint = CanvasPaint {
            wide: self.wide && wide_available,
            opacity: 1.,
            adjustments: Some(
                ImageAdjustments::new(
                    ColorMatrix::identity(ImagePrimaries::DisplayP3).exposure(self.exposure).expect("bounded exposure"),
                    None,
                )
                .expect("P3 adjustments"),
            ),
        };
        if *self.wide_switch.read(cx).data() != self.wide {
            let wide = self.wide;
            self.wide_switch.update(cx, |switch, cx| switch.set_data(wide, cx));
        }
        let scale = self.scale;
        let owner = cx.entity();
        let canvas = div()
            .id("photo-viewport")
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
            .h(px(viewport[1] as f32))
            .overflow_hidden()
            .cursor_crosshair()
            .bg(self.look.chrome().panel_background)
            .on_scroll(cx.listener(move |this, event: &ScrollEvent, _, cx| {
                this.zoom = (scale
                    * (f64::from(f32::from(event.delta.pixel_delta(px(24.)).y)) * 0.002).clamp(-1., 1.).exp())
                .clamp(0.001, 16.);
                cx.stop_propagation();
                cx.notify();
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, _, _| this.drag = Some((event.position, this.center))),
            )
            .on_mouse_move(cx.listener(move |this, event: &MouseMoveEvent, _, cx| {
                if event.pressed_button == Some(MouseButton::Left) {
                    if let Some((anchor, center)) = this.drag {
                        let delta = event.position - anchor;
                        this.center = [
                            center[0] - f64::from(f32::from(delta.x)) / scale,
                            center[1] - f64::from(f32::from(delta.y)) / scale,
                        ];
                        cx.notify();
                    }
                } else {
                    this.drag = None;
                }
            }))
            .on_mouse_up(MouseButton::Left, cx.listener(|this, _, _, _| this.drag = None))
            .on_mouse_up_out(MouseButton::Left, cx.listener(|this, _, _, _| this.drag = None))
            .child(
                gpui::canvas(
                    |_, _, _| (),
                    move |bounds, (), window, cx| {
                        let _ = canvas_entity.update(cx, |this, cx| {
                            if let Some(canvas) = &mut this.canvas
                                && let Err(error) = canvas.paint(bounds, window, paint)
                            {
                                let message = error.to_string();
                                if this.error.as_ref() != Some(&message) {
                                    this.error = Some(message);
                                    cx.notify();
                                }
                            }
                        });
                    },
                )
                .size_full(),
            );
        let output = if paint.wide {
            "native Display P3"
        } else if self.wide && !wide_available {
            "sRGB preview · wide output unavailable"
        } else {
            "sRGB preview"
        };
        let screen = LabScreen::new("media-photo", self.look.clone())
            .crumb("xgpui-media", "Photo Viewer")
            .title("Photo Viewer")
            .lede("Explore a photo in detail and adjust its brightness without loading it again.")
            .hero(Hero::full(canvas))
            .caption(format!("{status} · {output}"))
            .control(self.wide_switch.clone());
        let screen = self.buttons.iter().cloned().fold(screen, LabScreen::action);
        let source = self.source.as_ref();
        let stats = self.canvas.as_ref().map(|canvas| canvas.stats());
        screen.notice(self.error.clone())
            .cards([
                LabCard::new("Photo").subtitle(self.name.clone()).values([("size", source.map_or("—".into(), |source| format!("{}×{}", source.image().width(), source.image().height())))]),
                LabCard::new("View").values([("physical scale", format!("{:.1}%", self.scale * self.device_scale * 100.)), ("exposure", format!("{:+.2} EV", self.exposure))]),
            ])
            .panels(
                PanelShell::new("Retained source").child(properties(&self.look, [
                    ("Source payload", source.map_or("—".into(), |source| format!("{:.2} MiB", source.source_bytes() as f64 / MIB as f64))),
                    ("Page GPU payload", stats.as_ref().map_or("—".into(), |stats| format!("{:.2} MiB", stats.gpu_bytes as f64 / MIB as f64))),
                    ("Ready pages", stats.as_ref().map_or("—".into(), |stats| format!("{}/{}", stats.ready_pages, stats.visible_pages))),
                ])),
                PanelShell::new("Presentation").child(properties(&self.look, [
                    ("Output", output.to_string()), ("Device scale", format!("{:.2}×", self.device_scale)),
                    ("Center", format!("{:.0}, {:.0} px", self.center[0], self.center[1])),
                ])),
            )
            .note("Drag to pan and wheel to zoom. Fit shows the whole photo; 1:1 maps each source pixel to one physical display pixel. Exposure changes sampling without decoding or uploading the source again.")
            .credits("Built-in Italy reference photo credited to WebKit (webkit.org/blog-files/color-gamut/), separate from the application code license. Imported 8-bit images use supported RGB matrix/TRC ICC profiles, or sRGB when untagged. Payload statistics do not measure process RSS or driver VRAM.")
    }
}

/// Native correctness probe. Readback is confined to this explicit diagnostic.
pub fn run_check(cx: &mut App, output: PathBuf) -> Result<()> {
    let handle = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(None, size(px(1200.), px(850.)), cx))),
            window_background: WindowBackgroundAppearance::Opaque,
            show: false,
            focus: false,
            ..Default::default()
        },
        |window, cx| {
            window.set_color_mode(WindowColorMode::WideGamutSdr).expect("native wide photo check");
            cx.new(|cx| PhotoViewport::new(crate::theme::default_look(), cx))
        },
    )?;
    cx.spawn(async move |cx| {
        let result: Result<()> = async {
            async fn settled(handle: WindowHandle<PhotoViewport>, cx: &mut AsyncApp) -> Result<image::RgbaImage> {
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(90);
                loop {
                    let ready = handle.update(cx, |photo, _, _| -> Result<bool> {
                        ensure!(photo.error.is_none(), "photo error: {:?}", photo.error);
                        Ok(photo.loading.is_none() && photo.canvas.as_ref().is_some_and(|canvas| {
                            let stats = canvas.stats(); canvas.status() == &ProgressiveStatus::Ready && stats.ready_pages == stats.visible_pages
                        }))
                    })??;
                    let image = cx.update_window(handle.into(), |_, window, cx| -> Result<_> {
                        window.refresh(); window.draw(cx).clear(cx); window.render_to_image()
                    })??;
                    if ready { return Ok(image); }
                    ensure!(std::time::Instant::now() < deadline, "photo readiness timeout");
                    cx.background_executor().timer(std::time::Duration::from_millis(5)).await;
                }
            }
            std::fs::create_dir_all(&output)?;
            let baseline = settled(handle, cx).await?;
            baseline.save(output.join("photo.png"))?;
            let before = handle.update(cx, |photo, _, _| {
                let canvas = photo.canvas.as_ref().unwrap();
                (canvas.stats().requests, photo.source.as_ref().unwrap().source())
            })?;
            handle.update(cx, |photo, _, cx| { photo.exposure = 1.; cx.notify(); })?;
            let adjusted = settled(handle, cx).await?;
            adjusted.save(output.join("exposure.png"))?;
            ensure!(baseline.as_raw() != adjusted.as_raw(), "exposure must change pixels");
            let after = handle.update(cx, |photo, _, _| {
                (photo.canvas.as_ref().unwrap().stats().requests, photo.source.as_ref().unwrap().source())
            })?;
            ensure!(before == after, "sampling adjustment regenerated source/pages");
            handle.update(cx, |photo, _, cx| { photo.zoom = 1. / photo.device_scale; cx.notify(); })?;
            let one_to_one = settled(handle, cx).await?;
            one_to_one.save(output.join("one-to-one.png"))?;
            handle.update(cx, |photo, _, cx| { photo.wide = false; cx.notify(); })?;
            let srgb = settled(handle, cx).await?;
            ensure!(image::imageops::crop_imm(&one_to_one, 0, 0, one_to_one.width(), 100).to_image()
                == image::imageops::crop_imm(&srgb, 0, 0, srgb.width(), 100).to_image(), "preview output altered ordinary controls");
            handle.update(cx, |photo, _, cx| { photo.wide = true; cx.notify(); })?;
            handle.update(cx, |photo, _, cx| { photo.suspend(); photo.activate(cx); })?;
            settled(handle, cx).await?;
            let stats = handle.update(cx, |photo, _, _| {
                let source = photo.source.as_ref().unwrap(); let stats = photo.canvas.as_ref().unwrap().stats();
                serde_json::json!({"width": source.image().width(), "height":source.image().height(),
                    "source_bytes":source.source_bytes(),"page_gpu_bytes":stats.gpu_bytes,"page_cpu_bytes":stats.cpu_bytes,
                    "exposure_reused_source_and_pages":true,"native_readback_verified":true,"physical_pixel_one_to_one":true,"srgb_controls_independent":true,"displayed_fps_measured":false})
            })?;
            // A replacement must keep the prior overview during its first frame.
            let previous_source = handle.update(cx, |photo, _, _| photo.canvas.as_ref().unwrap().displayed_source())?;
            handle.update(cx, |photo, _, cx| -> Result<()> {
                let source = Arc::new(RasterSource::new(image::RgbaImage::from_pixel(8,8,image::Rgba([255,0,0,255])), ImageColor::SRGB, 256)?);
                photo.canvas.as_mut().unwrap().replace(source.clone())?;
                photo.source = Some(source); photo.zoom = 0.; photo.recenter(); cx.notify(); Ok(())
            })??;
            let retained = cx.update_window(handle.into(), |_, window, cx| -> Result<_> {
                window.refresh(); window.draw(cx).clear(cx); window.render_to_image()
            })??;
            ensure!(handle.update(cx, |photo, _, _| photo.canvas.as_ref().unwrap().displayed_source())? == previous_source,
                "replacement retired the displayed source before its successor was ready");
            let replacement = settled(handle, cx).await?;
            let x = replacement.width() * 45 / 100; let y = replacement.height() / 2;
            ensure!(retained.get_pixel(x,y) != replacement.get_pixel(x,y), "replacement covered prior image before upload readiness");
            let pixel = replacement.get_pixel(x,y).0;
            ensure!(pixel[0] > 240 && pixel[1] < 10 && pixel[2] < 10, "replacement pixels: {pixel:?}");
            std::fs::write(output.join("report.json"), serde_json::to_vec_pretty(&stats)?)?;
            println!("PASS: photo pixels, exposure reuse, physical 1:1, independent controls, retained replacement, suspend/resume");
            Ok(())
        }.await;
        let _ = cx.update_window(handle.into(), |_, window, _| window.remove_window());
        match result { Ok(()) => { cx.update(|cx| cx.quit()); }, Err(error) => { eprintln!("Photo check: {error:#}"); std::process::exit(1); } }
    }).detach();
    Ok(())
}

#[cfg(all(test, feature = "test-support"))]
mod screen_tests {
    use super::*;
    use xgpui_luma::{
        controls::{button::ButtonEvent, switch::SwitchEvent},
        theme::ThemeMode,
    };

    #[::core::prelude::v1::test]
    fn photo_screen_resizes_and_controls_reuse_the_retained_source() {
        for mode in [ThemeMode::Dark, ThemeMode::Light] {
            let mut app = TestAppContext::single();
            app.update(|cx| xgpui_luma::init(cx).unwrap());
            let look = crate::theme::default_look();
            look.set_mode(mode);
            let (page, cx) = app.add_window_view(|_, cx| {
                let mut photo = PhotoViewport::new(look, cx);
                photo.loading = None;
                photo.source = Some(Arc::new(
                    RasterSource::new(
                        image::RgbaImage::from_pixel(8, 8, image::Rgba([64, 128, 192, 255])),
                        ImageColor::SRGB,
                        256,
                    )
                    .unwrap(),
                ));
                photo.name = "Test photo".into();
                photo.recenter();
                photo
            });
            for width in [1280., 720.] {
                cx.simulate_resize(size(px(width), px(1400.)));
                cx.run_until_parked();
                let hero = cx.debug_bounds("media-photo-hero").unwrap();
                assert!(hero.size.width <= px(width));
                assert!(cx.debug_bounds("media-photo-panels").is_some());
            }
            let (source, brighter, one_to_one, wide) = cx.update(|_, cx| {
                let page = page.read(cx);
                (
                    page.source.clone().unwrap(),
                    page.buttons[4].clone(),
                    page.buttons[2].clone(),
                    page.wide_switch.clone(),
                )
            });
            brighter.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
            one_to_one.update(cx, |_, cx| cx.emit(ButtonEvent::Click));
            wide.update(cx, |switch, cx| {
                switch.set_data(false, cx);
                cx.emit(SwitchEvent::Change { on: false });
            });
            cx.run_until_parked();
            cx.update(|_, cx| {
                let page = page.read(cx);
                assert!(Arc::ptr_eq(&source, page.source.as_ref().unwrap()));
                assert_eq!(page.exposure, 0.25);
                assert_eq!(page.zoom, 1. / page.device_scale);
                assert!(!page.wide);
            });
        }
    }
}
