//! Retained field producer; GPU setup and submission stay off the UI thread.
use std::sync::{Arc, Mutex};
use std::time::Duration;
use gpui::{Bounds, Context, Pixels, Task};
use xgpui_gpu_interop::EngineSurface;
use xgpui_luma_color::color_field::ColorFieldBackgroundPainter;
use super::{interop_field_engine::Engine, source_field::FieldSettings};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Request {
    pub source: usize,
    pub saturation: f32,
    pub contour: bool,
    pub extent: [u32; 2],
    pub scale: f32,
}

#[derive(Default)]
pub(super) struct Field {
    desired: Option<Request>,
    completed: Option<Request>,
    engine: Option<Engine>,
    surface: Arc<Mutex<Option<EngineSurface>>>,
    task: Option<Task<()>>,
    check_presentation: bool,
    pub error: Option<String>,
}

impl Field {
    pub fn retry(&mut self) {
        if self.error.take().is_some() {
            self.engine = None;
            self.completed = None;
            *self.surface.lock().unwrap_or_else(|error| error.into_inner()) = None;
        }
    }

    pub fn painter(&self) -> ColorFieldBackgroundPainter {
        let shared = self.surface.clone();
        Arc::new(move |bounds, _, window, _| {
            let surface = shared.lock().unwrap_or_else(|error| error.into_inner()).clone();
            if let Some(surface) = surface {
                surface.paint(window, bounds, Default::default());
                if surface.stats().waiting > 0 {
                    window.request_animation_frame();
                }
            }
            Ok(())
        })
    }

    pub fn request(
        &mut self,
        settings: &FieldSettings,
        bounds: Bounds<Pixels>,
        scale: f32,
        cx: &mut Context<Self>,
    ) -> bool {
        let extent = [
            (bounds.size.width.as_f32() * scale).ceil() as u32,
            (bounds.size.height.as_f32() * scale).ceil() as u32,
        ];
        if extent.contains(&0) {
            return false;
        }
        let unsupported = self
            .surface
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .as_ref()
            .and_then(EngineSurface::unsupported);
        if self.error.is_none() && unsupported.is_some() {
            self.error = unsupported;
            cx.notify();
        }
        self.desired = Some(Request {
            source: settings.source_index(),
            saturation: settings.saturation(),
            contour: settings.contour(),
            extent,
            scale,
        });
        self.start(cx);
        std::mem::take(&mut self.check_presentation)
    }

    fn start(&mut self, cx: &mut Context<Self>) {
        if self.task.is_some() || self.error.is_some() || self.desired == self.completed {
            return;
        }
        let Some(request) = self.desired.clone() else {
            return;
        };
        let mut engine = self.engine.take();
        self.task = Some(cx.spawn(async move |this, cx| {
            let submitted = request.clone();
            let (engine, result) = cx
                .background_executor()
                .spawn(async move {
                    let result = (|| {
                        if engine.is_none() {
                            engine = Some(Engine::new()?);
                        }
                        engine.as_mut().ok_or_else(|| anyhow::anyhow!("Field engine unavailable"))?.render(&submitted)
                    })();
                    (engine, result)
                })
                .await;
            if matches!(result, Ok(None)) {
                cx.background_executor().timer(Duration::from_millis(16)).await;
            }
            let _ = this.update(cx, |this, cx| {
                this.task = None;
                this.engine = engine;
                match result {
                    Ok(Some(surface)) => {
                        *this.surface.lock().unwrap_or_else(|error| error.into_inner()) = Some(surface);
                        this.completed = Some(request);
                        this.check_presentation = true;
                    }
                    Ok(None) => {}
                    Err(error) => this.error = Some(format!("wgpu interop: {error:#}")),
                }
                cx.notify();
            });
        }));
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{AppContext, TestAppContext, px, point, size};
    #[test]
    fn edits_and_resize_coalesce_while_work_is_pending() {
        let mut app = TestAppContext::single();
        let field = app.update(|cx| cx.new(|_| Field::default()));
        field.update(&mut app, |field, cx| {
            field.task = Some(Task::ready(()));
            let settings = FieldSettings::new(0);
            let bounds = Bounds::new(point(px(0.), px(0.)), size(px(100.), px(50.)));
            field.request(&settings, bounds, 1., cx);
            settings.set_source(2);
            settings.set_saturation(0.62);
            settings.set_contour(false);
            field.request(&settings, bounds, 2., cx);
            let latest = field.desired.clone().unwrap();
            assert_eq!(latest.source, 2);
            assert_eq!(latest.saturation, 0.62);
            assert!(!latest.contour);
            assert_eq!(latest.extent, [200, 100]);
            assert!(field.engine.is_none());
            field.task = None;
            field.completed = Some(latest);
            field.start(cx);
            assert!(field.task.is_none(), "unchanged input must reuse the completed frame");
            field.completed = None;
            field.error = Some("unsupported adapter".into());
            field.start(cx);
            assert!(field.task.is_none(), "permanent failures must not loop");
            field.retry();
            assert!(field.error.is_none());
        });
    }
}
