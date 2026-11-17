//! UI bridge to the retained producer. Coalesce edits while one background job runs.
use crate::gradient_builder::color_field::{ColorField, portable::Engine};
use gpui::{Context, Corners, Render, Task, Window, canvas, div, prelude::*};
use std::time::Duration;
use xgpui_gpu_interop::EngineSurface;

#[derive(Clone, PartialEq)]
struct Request {
    field: ColorField,
    wide: bool,
    extent: [u32; 2],
}

pub(super) struct FieldView {
    input: (ColorField, bool),
    desired: Option<Request>,
    completed: Option<Request>,
    engine: Option<Engine>,
    surface: Option<EngineSurface>,
    task: Option<Task<()>>,
    error: Option<String>,
    check_presentation: bool,
}
impl FieldView {
    pub(super) fn new(field: ColorField, wide: bool) -> Self {
        Self {
            input: (field, wide),
            desired: None,
            completed: None,
            engine: None,
            surface: None,
            task: None,
            error: None,
            check_presentation: false,
        }
    }
    pub(super) fn set_input(&mut self, field: ColorField, wide: bool, cx: &mut Context<Self>) {
        if self.input != (field.clone(), wide) {
            self.input = (field, wide);
            cx.notify();
        }
    }
    pub(super) fn failed(&self) -> bool {
        self.error.is_some() || self.surface.as_ref().is_some_and(|s| s.unsupported().is_some())
    }
    fn request(&mut self, extent: [u32; 2], cx: &mut Context<Self>) -> bool {
        if extent.contains(&0) {
            return false;
        }
        if let Some(error) = self.surface.as_ref().and_then(EngineSurface::unsupported)
            && self.error.is_none()
        {
            self.error = Some(format!("{error}. Switch away and back to wgpu to retry."));
            cx.notify();
        }
        self.desired = Some(Request { field: self.input.0.clone(), wide: self.input.1, extent });
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
                        engine.as_mut().ok_or_else(|| anyhow::anyhow!("Freeform engine unavailable"))?.render(
                            &submitted.field,
                            submitted.wide,
                            submitted.extent,
                        )
                    })();
                    (engine, result)
                })
                .await;
            // Backpressure is temporary; retry without spinning or occupying the UI thread.
            if matches!(result, Ok(None)) {
                cx.background_executor().timer(Duration::from_millis(16)).await;
            }
            let _ = this.update(cx, |this, cx| {
                this.task = None;
                this.engine = engine;
                match result {
                    Ok(Some(surface)) => {
                        this.surface = Some(surface);
                        this.completed = Some(request);
                        this.check_presentation = true;
                    }
                    Ok(None) => {}
                    Err(error) => this.error = Some(format!("{error:#}. Switch away and back to wgpu to retry.")),
                }
                // The next prepaint schedules only the latest edit and physical extent.
                cx.notify();
            });
        }));
    }
}
impl Render for FieldView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let surface = self.surface.clone();
        div()
            .size_full()
            .relative()
            .child(
                canvas(
                    move |bounds, window, cx| {
                        let scale = window.scale_factor();
                        let check = entity.update(cx, |this, cx| {
                            this.request(
                                [
                                    (bounds.size.width.as_f32() * scale).ceil() as u32,
                                    (bounds.size.height.as_f32() * scale).ceil() as u32,
                                ],
                                cx,
                            )
                        });
                        // One follow-up frame observes any latched compositor/adapter failure.
                        if check {
                            window.request_animation_frame();
                        }
                    },
                    move |bounds, _, window, cx| {
                        if let Some(surface) = &surface {
                            surface.paint(window, cx, bounds, Corners::default());
                            if surface.stats().waiting > 0 {
                                window.request_animation_frame();
                            }
                        }
                    },
                )
                .size_full(),
            )
            .when_some(self.error.clone(), |el, error| el.child(div().absolute().inset_0().p_3().child(error)))
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::TestAppContext;
    use crate::gradient_builder::color_field::tests::fixture;
    #[test]
    fn pending_work_coalesces_edits_and_physical_resize() {
        let mut app = TestAppContext::single();
        let view = app.update(|cx| cx.new(|_| FieldView::new(fixture(), false)));
        view.update(&mut app, |view, cx| {
            // Represent an in-flight job without accessing a GPU.
            view.task = Some(Task::ready(()));
            view.request([100, 50], cx);
            let mut latest = fixture();
            latest.points[0].radius = 1.1;
            view.set_input(latest.clone(), true, cx);
            view.request([200, 100], cx);
            let request = view.desired.as_ref().unwrap();
            assert_eq!(request.field, latest);
            assert!(request.wide);
            assert_eq!(request.extent, [200, 100]);
            assert!(view.completed.is_none());
            assert!(view.engine.is_none());
            view.task = None;
            view.completed = view.desired.clone();
            view.start(cx);
            assert!(view.task.is_none(), "unchanged field and extent reuse the last frame");
            view.completed = None;
            view.error = Some("unsupported adapter".into());
            view.start(cx);
            assert!(view.task.is_none(), "permanent errors must not retry every frame");
        });
    }
}
