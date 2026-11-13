use super::*;
use xgpui_luma::infra::ElementExt;
use gpui::{prelude::*, *};
use xgpui_media::pipeline::*;
use anyhow::Context as _;

impl MediaPage {
    pub(super) fn camera_viewport(
        &self,
        index: usize,
        width: f32,
        height: f32,
        fill: bool,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let owner = cx.entity();
        // `fill` takes the parent's width; the measured width feeds the next frame's image fit.
        div()
            .id(format!("media-image-viewport-{index}"))
            .relative()
            .map(|view| if fill { view.w_full() } else { view.w(px(width)) })
            .h(px(height))
            .flex_shrink_0()
            .overflow_hidden()
            .cursor_crosshair()
            .child(super::advanced::checker())
            .on_prepaint(move |bounds, _, cx| {
                owner.update(cx, |this, cx| {
                    let resized = this.bounds[index].size.width != bounds.size.width;
                    this.bounds[index] = bounds;
                    if fill && resized {
                        cx.notify();
                    }
                });
            })
            .on_scroll(cx.listener(move |this, event: &ScrollEvent, _, cx| {
                let delta = f32::from(event.delta.pixel_delta(px(24.)).y);
                this.zoom_at(event.position - this.bounds[index].center(), (delta * 0.002).clamp(-1., 1.).exp(), cx);
                cx.stop_propagation();
            }))
            .on_pinch(cx.listener(move |this, event: &PinchEvent, _, cx| {
                this.zoom_at(event.position - this.bounds[index].center(), (1. + event.delta).max(0.01), cx);
                cx.stop_propagation();
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, _, cx| {
                    this.drag = Some((event.position, [this.values[1], this.values[2]]));
                    this.sweep = false;
                    cx.stop_propagation();
                }),
            )
    }
    pub(super) fn sync_camera(&self, cx: &mut Context<Self>) {
        for (parameter, slider) in self.sliders.iter().take(3) {
            slider.update(cx, |slider, cx| slider.set_value(self.values[*parameter as usize], cx));
            self.sync_field(*parameter, cx);
        }
    }
    pub(super) fn zoom_at(&mut self, anchor: Point<Pixels>, factor: f32, cx: &mut Context<Self>) {
        let old = self.values[0];
        let next = (old * factor).clamp(0.1, 4.);
        let pan = point(px(self.values[1]), px(self.values[2]));
        let pan = anchor - (anchor - pan) * (next / old);
        self.values[0] = next;
        self.values[1] = pan.x.into();
        self.values[2] = pan.y.into();
        self.sweep = false;
        self.drag = None;
        self.sync_camera(cx);
        cx.notify();
    }
    pub(super) fn exposure(&self) -> f64 {
        self.values[9] as f64
            + if self.animate {
                (self.started.elapsed().as_secs_f64() * 1.5).sin() * 0.8
            } else {
                0.
            }
    }
    pub(super) fn adjustments(&self, primaries: ImagePrimaries) -> anyhow::Result<ImageAdjustments> {
        ImageAdjustments::new(
            ColorMatrix::identity(primaries)
                .exposure(self.exposure())?
                .contrast(self.values[10] as f64, 0.18)?
                .saturation(self.values[11] as f64)?,
            Some(Vignette::new(self.values[12] as f64, 0.35, 0.65, 1.)?),
        )
    }
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub(super) fn blur(&self) -> anyhow::Result<ImageBlur> {
        Ok(ImageBlur::new(px([0., 4., 8., 16.][self.blur_index]))?.with_quality(if self.full_quality {
            LayerBlurQuality::FullResolution
        } else {
            LayerBlurQuality::Adaptive
        }))
    }
    pub(super) fn load_float(&mut self) {
        if self.float_image.is_some() || self.float_job.is_some() {
            return;
        }
        if let Ok(loader) = &self.float_loader {
            match loader.load_rgba16f(1280, 800, pixels(1280, 800), ImageColor::LINEAR_P3) {
                Ok(job) => self.float_job = Some(job),
                Err(error) => self.error = Some(error.to_string()),
            }
        }
    }
    pub(super) fn open_image(&mut self, cx: &mut Context<Self>) {
        self.import_request += 1;
        let request = self.import_request;
        let picker = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Open PNG, JPEG or WebP".into()),
        });
        cx.spawn(async move |this, cx| {
            let result: anyhow::Result<Option<Fixtures>> = async {
                let selected = picker.await.context("File picker closed")??;
                let Some(path) = selected.and_then(|paths| paths.into_iter().next()) else {
                    return Ok(None);
                };
                cx.background_spawn(async move { Fixtures::new(Some(path)) }).await.map(Some)
            }
            .await;
            let _ = this.update(cx, |this, cx| {
                if request != this.import_request {
                    return;
                }
                match result {
                    Ok(Some(fixtures)) => {
                        this.selected = fixtures.samples.len() - 1;
                        this.fixtures = Some(fixtures);
                        this.reload();
                    }
                    Ok(None) => (),
                    Err(error) => this.error = Some(error.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }
}

fn pixels(width: u32, height: u32) -> Vec<[f32; 4]> {
    (0..width * height)
        .map(|i| {
            let u = (i % width) as f32 / (width - 1) as f32;
            let v = (i / width) as f32 / (height - 1) as f32;
            let alpha = ((0.49 - (u - 0.5).abs().max((v - 0.5).abs())) * 60.).clamp(0., 1.);
            let rgb = if v < 0.33 {
                [1. - u, u, 0.02]
            } else if v < 0.66 {
                [u * 0.12, u * 0.12, u * 0.12]
            } else {
                [0.05, 1. - u, u]
            };
            [rgb[0], rgb[1], rgb[2], alpha]
        })
        .collect()
}

pub(crate) fn checker() -> impl IntoElement {
    canvas(
        |_, _, _| (),
        |bounds, (), window, _| {
            let tile = 24.;
            for y in 0..(f32::from(bounds.size.height) / tile).ceil() as i32 {
                for x in 0..(f32::from(bounds.size.width) / tile).ceil() as i32 {
                    let origin = bounds.origin + point(px(x as f32 * tile), px(y as f32 * tile));
                    let rect = Bounds::new(
                        origin,
                        size(px(tile).min(bounds.right() - origin.x), px(tile).min(bounds.bottom() - origin.y)),
                    );
                    window.paint_quad(fill(rect, rgb(super::palette::TRANSPARENCY_CHECKER[((x + y) % 2) as usize])));
                }
            }
        },
    )
    .absolute()
    .size_full()
}
