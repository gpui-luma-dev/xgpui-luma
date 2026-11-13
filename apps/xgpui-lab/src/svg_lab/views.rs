use super::*;
use gpui::{prelude::*, AnyElement, Bounds, ContentMask, Corners, Render, Window, canvas, div, point, px};
use xgpui_svg::{svg_image, path_effects::PathEffect};

impl Render for SvgPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_choices(cx);
        self.sync_bench_thumbnail(cx);
        self.density = window.scale_factor();
        self.motion_allowed = !cx.reduce_motion();
        self.gallery_columns =
            ((f32::from(window.viewport_size().width) - 96.).min(1192.) / 232.).floor().max(1.) as usize;
        if self.animate && self.motion_allowed && self.active_tab == "Path Effects" {
            window.request_animation_frame();
        }
        let actions: &[Action] = match self.active_tab.as_ref() {
            "Path Effects" => &[Action::Dash, Action::Animate, Action::Background, Action::Reset],
            "Scheduling & Cache" => &[
                Action::Gallery,
                Action::Builtin,
                Action::Eager,
                Action::Jobs,
                Action::Pool,
                Action::Tiles,
                Action::Budget,
                Action::Background,
            ],
            "Benchmarks" => &[Action::Open, Action::Builtin, Action::Benchmark, Action::Budget],
            "Scaling" => &[
                Action::Open,
                Action::Builtin,
                Action::Zoom1,
                Action::Zoom2,
                Action::Zoom4,
                Action::Zoom8,
                Action::Zoom16,
                Action::DevicePixels,
                Action::ScalingBudget,
                Action::Preview,
                Action::Settle,
                Action::Background,
                Action::Reset,
            ],
            _ => &[Action::Open, Action::Builtin, Action::Clip, Action::Shadow, Action::Background],
        };
        let parameters: &[Parameter] = match self.active_tab.as_ref() {
            "Path Effects" => &[Parameter::Trim, Parameter::Phase, Parameter::Width],
            "Benchmarks" | "Scheduling & Cache" => &[Parameter::Zoom],
            "Scaling" => &[Parameter::Zoom, Parameter::PanX, Parameter::PanY],
            _ => &[],
        };
        let result = match self.active_tab.as_ref() {
            "Path Effects" => self.path_view(),
            "Scheduling & Cache" => self.gallery(cx),
            "Benchmarks" => Ok(self.benchmark_view()),
            "Scaling" => self.scaling(cx),
            _ => self.rendering(cx),
        };
        let screen = self.screen(actions, parameters, result, cx);
        div()
            .on_mouse_move(cx.listener(|this, event: &gpui::MouseMoveEvent, _, cx| {
                if event.pressed_button != Some(gpui::MouseButton::Left) {
                    this.drag_position = None;
                } else if let Some(previous) = this.drag_position.replace(event.position) {
                    this.camera.pan += event.position - previous;
                    this.copy_camera(cx);
                    cx.notify();
                } else {
                    this.drag_position = None;
                }
            }))
            .on_mouse_up(gpui::MouseButton::Left, cx.listener(|this, _, _, _| this.drag_position = None))
            .on_mouse_up_out(gpui::MouseButton::Left, cx.listener(|this, _, _, _| this.drag_position = None))
            .size_full()
            .child(screen)
    }
}

impl SvgPage {
    fn rendering(&self, cx: &mut Context<Self>) -> anyhow::Result<AnyElement> {
        let asset = self.asset.as_ref().map_err(|error| anyhow::anyhow!("{error:#}"))?;
        let camera = camera::Camera { scene_size: asset.logical, ..Default::default() };
        let fixed = asset.fixed.size(0);
        let view = div().id("svg-rendering-comparison").debug_selector(|| "svg-rendering-comparison".into()).w_full();
        let mut panels = div().flex().flex_wrap().gap_4();
        for (mode, label) in [
            format!("Fixed raster · {}×{} px", i32::from(fixed.width), i32::from(fixed.height)),
            "SVG renderer · display-density raster".into(),
            "Native GPUI paths".into(),
        ]
        .into_iter()
        .enumerate()
        {
            let panel = div().size_full().overflow_hidden().bg(self.backdrop.color());
            let panel = if mode == 1 {
                panel.child(
                    svg_image("svg-rendering-progressive", asset.document.clone())
                        .logical_size(asset.logical)
                        .policy(RasterPolicy::default())
                        .on_status_change(cx.listener(|this, status: &SvgStatus, _, cx| {
                            this.rendering_status = Some(status.clone());
                            cx.notify();
                        })),
                )
            } else if mode == 2 && asset.source.is_some() {
                panel.child("Imported artwork has no automatic SVG-to-native-path conversion")
            } else {
                panel.child(self.texture_or_native(camera, mode == 2))
            };
            let panel = radix::Card::new(&self.look)
                .style_override(|style| {
                    style.padding = 1.;
                    style.radius = 0.;
                    style.background = None;
                })
                .child(panel)
                .into_element()
                .w_full()
                .aspect_ratio(1.)
                .flex_shrink_0()
                .overflow_hidden();
            panels = panels.child(
                div()
                    .flex_1()
                    .min_w(px(280.))
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(crate::screen::text(&self.look, crate::screen::TextRole::Name, label))
                    .child(panel),
            );
        }
        Ok(view.child(panels).into_any_element())
    }

    pub(super) fn source_label(&self) -> String {
        self.asset
            .as_ref()
            .ok()
            .and_then(|asset| asset.source.as_ref())
            .map_or_else(|| "Built-in ribbon/leaf".to_owned(), |path| path.display().to_string())
    }

    pub(super) fn texture_or_native(&self, camera: camera::Camera, native: bool) -> AnyElement {
        let fixed = self.asset.as_ref().expect("validated SVG asset").fixed.clone();
        let clipped = self.clipped;
        let shadow = self.shadow;
        canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                let origin = camera.origin(bounds);
                window.with_content_mask(Some(ContentMask { bounds }), |window| {
                    if native {
                        scene::native(origin, camera.zoom, clipped, shadow, window);
                    } else {
                        let image_bounds = Bounds::new(
                            origin,
                            size(px(camera.scene_size.width * camera.zoom), px(camera.scene_size.height * camera.zoom)),
                        );
                        if let Err(error) =
                            window.paint_image(image_bounds, image_bounds, Corners::default(), fixed.clone(), 0, false)
                        {
                            eprintln!("SVG baseline paint: {error:#}");
                        }
                    }
                });
            },
        )
        .size_full()
        .into_any_element()
    }

    fn gallery(&self, cx: &mut Context<Self>) -> anyhow::Result<AnyElement> {
        let asset = self.asset.as_ref().map_err(|error| anyhow::anyhow!("{error:#}"))?;
        let scheduler = self.scheduler.as_ref().map_err(|error| anyhow::anyhow!("{error:#}"))?;
        let columns = self.gallery_columns;
        let row_height = 210.;
        let viewport_height = 520.;
        let offset = -f32::from(self.gallery_scroll.offset().y);
        let mut rows = Vec::new();
        for first in (0..self.tiles).step_by(columns) {
            let visible = self.eager
                || ((first / columns) as f32 * row_height >= offset - row_height
                    && (first / columns) as f32 * row_height < offset + viewport_height + row_height);
            let mut row = div().h(px(row_height)).flex_shrink_0().flex().gap_3();
            for index in first..(first + columns).min(self.tiles) {
                let (name, document) = if self.gallery_docs.is_empty() {
                    ("Built-in ribbon/leaf", asset.document.clone())
                } else {
                    let (name, doc) = &self.gallery_docs[index % self.gallery_docs.len()];
                    (name.as_str(), doc.clone())
                };
                let mut tile = div().w(px(220.)).flex_shrink_0().flex().flex_col().gap_1().child(crate::screen::text(
                    &self.look,
                    crate::screen::TextRole::Detail,
                    name.to_string(),
                ));
                if visible {
                    tile = tile.child(
                        svg_image(format!("svg-scheduled-{index}"), document)
                            .h(px(180.))
                            .zoom(self.values[0])
                            .policy(self.policy)
                            .scheduler(scheduler.clone())
                            .bg(self.backdrop.color())
                            .on_status_change(cx.listener(|_, _: &SvgStatus, _, cx| cx.notify())),
                    );
                } else {
                    tile = tile.child(div().h(px(180.)).child("Outside visible + overscan rows"));
                }
                row = row.child(tile);
            }
            rows.push(row);
        }
        Ok(div()
            .w_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .id("svg-gallery-scroll")
                    .h(px(viewport_height))
                    .flex_shrink_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.gallery_scroll)
                    .on_scroll(cx.listener(|this, event: &gpui::ScrollEvent, _, cx| {
                        if event.modifiers.platform {
                            let delta = f32::from(event.delta.pixel_delta(px(16.)).y);
                            this.camera.zoom_at(point(px(0.), px(0.)), (-delta * 0.005).exp());
                            this.copy_camera(cx);
                            cx.stop_propagation();
                        }
                        cx.notify();
                    }))
                    .on_pinch(cx.listener(|this, event: &gpui::PinchEvent, _, cx| {
                        this.zoom_at(point(px(0.), px(0.)), (1. + event.delta).max(0.01), cx);
                        cx.stop_propagation();
                    }))
                    .children(rows),
            )
            .into_any_element())
    }

    fn path_view(&self) -> anyhow::Result<AnyElement> {
        let paths = self.paths.as_ref().map_err(|error| anyhow::anyhow!("{error:#}"))?;
        let mut view = div().w_full().flex().flex_col().gap_4();
        for (index, path) in paths.iter().enumerate() {
            view = view
                .child(crate::screen::text(
                    &self.look,
                    crate::screen::TextRole::Name,
                    format!(
                        "{} · Measured length {:.2}px",
                        ["Open Bézier curve", "Closed heart", "Multiple contours"][index],
                        f32::from(path.length())
                    ),
                ))
                .child(
                    div().flex().flex_wrap().gap_3().children(
                        [
                            ("Original", PathEffect::new()),
                            ("Trimmed", paths::effect(self.effective_trim(), 0, 0.)?),
                            (
                                "Trimmed + dashed",
                                paths::effect(self.effective_trim(), self.dash, self.effective_phase())?,
                            ),
                        ]
                        .into_iter()
                        .map(|(label, effect)| {
                            div()
                                .flex_1()
                                .min_w(px(240.))
                                .flex()
                                .flex_col()
                                .gap_2()
                                .child(crate::screen::text(&self.look, crate::screen::TextRole::Label, label))
                                .child(
                                    paths::panel(&self.look, path, effect, self.values[5], palette::PATH, 170.)
                                        .bg(self.backdrop.color()),
                                )
                        }),
                    ),
                );
        }
        Ok(view.into_any_element())
    }

    fn benchmark_view(&self) -> AnyElement {
        let mut view=div().flex().flex_col().gap_3().child("svg-lab CPU parse/raster observations")
            .child("Runs five bounded parse + raster samples in a background task. Times exclude GPU upload and presentation. These in-process observations are not the standalone lab's isolated-process corpus benchmark or GPU timing.");
        if self.benchmarking {
            view = view.child(self.bench_progress.snapshot().0);
        }
        if let Some(result) = &self.bench_result {
            match result {
                Ok(_) => {}
                Err(error) => view = view.child(format!("Benchmark failed: {error}")),
            }
        }
        view.child(crate::benchmarks::svg_table::view(self.bench_table.clone())).into_any_element()
    }
}
