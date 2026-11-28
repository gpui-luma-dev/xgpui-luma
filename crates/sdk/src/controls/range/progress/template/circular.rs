use crate::style::PartRead;
use std::f32::consts::PI;
use std::sync::{Arc, OnceLock};

use gpui::{App, Div, PathBuilder, Stateful, Window, canvas, div, point, px, prelude::*};

use super::{ProgressTemplate, ProgressTemplateModifier, apply_template_modifiers};
use super::super::ProgressRenderModel;
use crate::controls::progress::{ProgressTheme, default_progress_theme};

pub struct CircularProgressTemplate {
    theme: Arc<dyn ProgressTheme>,
    modifiers: Vec<ProgressTemplateModifier>,
}

pub type ThemedProgressTemplate = CircularProgressTemplate;

impl CircularProgressTemplate {
    pub fn new(theme: Arc<dyn ProgressTheme>) -> Self {
        Self { theme, modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ProgressRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }
}

pub fn default_circular_progress_template() -> Arc<dyn ProgressTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ProgressTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(CircularProgressTemplate::new(default_progress_theme()))).clone()
}

impl ProgressTemplate for CircularProgressTemplate {
    fn render(&self, model: &ProgressRenderModel<'_>, _window: &mut Window, _cx: &mut App) -> Stateful<Div> {
        let look = super::super::resolve_progress(&self.theme.style(), model.enabled);
        let geometry = crate::theme::stylesheet::ProgressGeometry::from_resolved(&look);
        let percentage = model.percentage.clamp(0.0, 1.0);
        let indeterminate = model.indeterminate;
        let phase = model.phase.clamp(0.0, 1.0);
        let size = px(look.root().number("ring_size"));
        let stroke_width = px(look.root().number("stroke_width"));
        let track_color = look.part(xgpui_luma_style::PartId::new("track")).color("background");
        let progress_color = look.part(xgpui_luma_style::PartId::new("fill")).color("background");
        let sweep_fraction = geometry.indeterminate_fraction;

        let progress_canvas = canvas(
            |_, _, _| {},
            move |bounds, _, window, _cx| {
                let center_x = bounds.origin.x + bounds.size.width / 2.0;
                let center_y = bounds.origin.y + bounds.size.height / 2.0;
                let radius = (size / 2.0) - stroke_width;

                paint_circle(center_x, center_y, radius, stroke_width, track_color, window);
                if indeterminate {
                    paint_sweep_arc(
                        center_x,
                        center_y,
                        radius,
                        stroke_width,
                        phase,
                        sweep_fraction,
                        progress_color,
                        window,
                    );
                } else {
                    paint_progress_arc(center_x, center_y, radius, stroke_width, percentage, progress_color, window);
                }
            },
        )
        .size_full();

        let root = div().id(model.id.clone()).relative().size(size).child(progress_canvas);
        apply_template_modifiers(&self.modifiers, root, model)
    }
}

fn paint_circle(
    center_x: gpui::Pixels,
    center_y: gpui::Pixels,
    radius: gpui::Pixels,
    stroke_width: gpui::Pixels,
    color: gpui::Hsla,
    window: &mut Window,
) {
    let mut builder = PathBuilder::stroke(stroke_width);

    builder.move_to(point(center_x + radius, center_y));
    builder.arc_to(point(radius, radius), px(0.0), false, true, point(center_x - radius, center_y));
    builder.arc_to(point(radius, radius), px(0.0), false, true, point(center_x + radius, center_y));
    builder.close();

    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }
}

fn paint_progress_arc(
    center_x: gpui::Pixels,
    center_y: gpui::Pixels,
    radius: gpui::Pixels,
    stroke_width: gpui::Pixels,
    percentage: f32,
    color: gpui::Hsla,
    window: &mut Window,
) {
    if percentage <= 0.0 {
        return;
    }

    if percentage >= 0.999 {
        paint_circle(center_x, center_y, radius, stroke_width, color, window);
        return;
    }

    let mut builder = PathBuilder::stroke(stroke_width);
    let start_x = center_x;
    let start_y = center_y - radius;
    let angle = -PI / 2.0 + percentage * 2.0 * PI;
    let end_x = center_x + radius * angle.cos();
    let end_y = center_y + radius * angle.sin();

    builder.move_to(point(start_x, start_y));
    builder.arc_to(point(radius, radius), px(0.0), percentage > 0.5, true, point(end_x, end_y));

    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_sweep_arc(
    center_x: gpui::Pixels,
    center_y: gpui::Pixels,
    radius: gpui::Pixels,
    stroke_width: gpui::Pixels,
    phase: f32,
    sweep: f32,
    color: gpui::Hsla,
    window: &mut Window,
) {
    let sweep = sweep.clamp(0.05, 0.95);
    let start_angle = -PI / 2.0 + phase * 2.0 * PI;
    let end_angle = start_angle + sweep * 2.0 * PI;

    let start_x = center_x + radius * start_angle.cos();
    let start_y = center_y + radius * start_angle.sin();
    let end_x = center_x + radius * end_angle.cos();
    let end_y = center_y + radius * end_angle.sin();

    let mut builder = PathBuilder::stroke(stroke_width);
    builder.move_to(point(start_x, start_y));
    builder.arc_to(point(radius, radius), px(0.0), sweep > 0.5, true, point(end_x, end_y));

    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }
}
