use super::*;
use gpui::{prelude::*, App, Render, Window, div, img, px};
use xgpui_media::image_view;

impl Render for MediaPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.active_tab == "Benchmarks" {
            return div().size_full().child(self.benchmarks.clone()).into_any_element();
        }
        if let Some(mut run) = self.cadence.take()
            && run.tick(self, window)
        {
            self.cadence = Some(run);
        }
        self.sync_choices(cx);
        if cx.reduce_motion() {
            self.sweep = false;
            self.animate = false;
        }
        if (self.sweep || self.animate) && !cx.reduce_motion() {
            window.request_animation_frame();
        }
        if let Some(job) = &mut self.float_job {
            if let Some(image) = job.try_take() {
                self.float_image = Some(image);
                self.float_job = None;
            } else if let LoadStatus::Failed(error) = job.status() {
                self.error = Some(error);
                self.float_job = None;
            }
        }
        if self.float_job.is_some() {
            window.request_animation_frame();
        }
        if self.requires_sdr(cx) && window.color_status().effective() != gpui::WindowColorMode::Default {
            return div().p_4().child("Preparing SDR presentation for media blur…").into_any_element();
        }
        let video = self.active_tab == "Video Frames";
        if !video {
            self.ensure_fixtures(cx);
        }
        self.poll();
        if self.job.is_some() || (self.active_tab == "Resource Limits" && self.resources.needs_redraw()) {
            window.request_animation_frame();
        }
        if self.active_tab == "Image Filters" {
            return self.render_image_filters(cx).into_any_element();
        }
        if matches!(self.active_tab.as_ref(), "Images" | "Textures & Mipmaps") {
            return self.render_image_screen(window, cx).into_any_element();
        }
        match self.active_tab.as_ref() {
            "Linear Adjustments" | "Float / P3" => self.render_adjustments(window, cx).into_any_element(),
            "Resource Limits" => self.render_resources(cx).into_any_element(),
            "Video Frames" => self.render_video(window, cx).into_any_element(),
            "Tiled Canvas" => div().size_full().children(self.tiled.clone()).into_any_element(),
            _ => self.render_image_screen(window, cx).into_any_element(),
        }
    }
}

impl MediaPage {
    /// Display scale of the current sample in a `width` × `height` viewport (fit × zoom, with sweep).
    pub(super) fn display_scale(&self, width: f32, height: f32, cx: &App) -> Option<f32> {
        let sample = &self.fixtures.as_ref()?.samples[self.selected];
        let zoom = self.values[0]
            * if self.sweep && !cx.reduce_motion() {
                1. + 0.1 * (self.started.elapsed().as_secs_f32() * 0.9).sin()
            } else {
                1.
            };
        Some((width / sample.width as f32).min(height / sample.height as f32) * zoom)
    }

    /// One comparison viewport: the GPUI image atlas (`native == false`) or the media texture.
    pub(super) fn compare_view(
        &self,
        native: bool,
        width: f32,
        height: f32,
        fill: bool,
        cx: &mut Context<Self>,
    ) -> gpui::Stateful<gpui::Div> {
        let mut viewport = self.camera_viewport(usize::from(native), width, height, fill, cx);
        let (Some(fixtures), Some(fit)) = (&self.fixtures, self.display_scale(width, height, cx)) else {
            return viewport;
        };
        let sample = &fixtures.samples[self.selected];
        let w = sample.width as f32 * fit;
        let h = sample.height as f32 * fit;
        let x = (width - w) * 0.5 + self.values[1];
        let y = (height - h) * 0.5 + self.values[2];
        if native || self.mode == 1 {
            if let Some(image) = &self.image {
                let filter = if self.active_tab == "Image Filters" {
                    self.filter()
                } else {
                    ImageFilter::new()
                };
                let view = image_view(image.clone());
                let view = if native && self.mode == 1 {
                    match self.adjustments(xgpui_media::pipeline::ImagePrimaries::Srgb) {
                        Ok(adjustments) => view.adjustments(adjustments),
                        Err(_) => view,
                    }
                } else {
                    view.filter(filter)
                };
                viewport = viewport.child(
                    view.absolute()
                        .left(px(x))
                        .top(px(y))
                        .w(px(w))
                        .h(px(h))
                        .rounded(px(self.values[4]))
                        .opacity(self.values[3]),
                );
            }
        } else {
            viewport = viewport.child(
                img(sample.path.clone())
                    .absolute()
                    .left(px(x))
                    .top(px(y))
                    .w(px(w))
                    .h(px(h))
                    .rounded(px(self.values[4]))
                    .opacity(self.values[3]),
            );
        }
        viewport
    }
}
