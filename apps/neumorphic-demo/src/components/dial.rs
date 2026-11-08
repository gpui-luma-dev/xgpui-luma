use std::f32::consts::PI;
use std::sync::Arc;

use gpui::{
    App, Bounds, BoxShadow, Corners, Div, FontFeatures, PathBuilder, Pixels, Point, SharedString, Stateful, Window,
    canvas, div, fill, hsla, point, prelude::*, px, rgb, size,
};
use xgpui_luma::controls::slider::{
    self, SliderBuilder, SliderRenderModel, SliderTemplate, SliderTemplateHandlers, ThumbId,
};

const MIN_ANGLE: f32 = -1.25 * PI;
const MAX_ANGLE: f32 = 0.25 * PI;
const ANGLE_SPAN: f32 = MAX_ANGLE - MIN_ANGLE;
const DISABLED_OPACITY: f32 = 0.56;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialSize {
    Large,
    Medium,
    Small,
}

impl DialSize {
    fn frame_diameter(self) -> Pixels {
        match self {
            Self::Large => px(186.0),
            Self::Medium => px(126.0),
            Self::Small => px(80.0),
        }
    }

    fn face_diameter(self) -> Pixels {
        match self {
            Self::Large => px(150.0),
            Self::Medium => px(102.0),
            Self::Small => px(64.0),
        }
    }

    fn label_size(self) -> Pixels {
        match self {
            Self::Large => px(14.0),
            Self::Medium => px(12.0),
            Self::Small => px(10.0),
        }
    }

    fn label_line_height(self) -> Pixels {
        match self {
            Self::Large => px(18.0),
            Self::Medium => px(16.0),
            Self::Small => px(12.0),
        }
    }

    fn border_width(self) -> Pixels {
        match self {
            Self::Large => px(2.5),
            Self::Medium => px(2.0),
            Self::Small => px(1.25),
        }
    }

    fn inner_border_width(self) -> Pixels {
        match self {
            Self::Large => px(1.75),
            Self::Medium => px(1.35),
            Self::Small => px(1.0),
        }
    }

    fn tick_count(self) -> usize {
        match self {
            Self::Large => 40,
            Self::Medium => 32,
            Self::Small => 24,
        }
    }

    fn tick_inner_length(self) -> Pixels {
        match self {
            Self::Large => px(6.0),
            Self::Medium => px(4.0),
            Self::Small => px(3.0),
        }
    }

    fn tick_outer_length(self) -> Pixels {
        match self {
            Self::Large => px(12.0),
            Self::Medium => px(8.0),
            Self::Small => px(6.0),
        }
    }

    fn tick_ring_gap(self) -> Pixels {
        match self {
            Self::Large => px(10.0),
            Self::Medium => px(6.0),
            Self::Small => px(4.0),
        }
    }

    fn tick_stroke_width(self) -> Pixels {
        match self {
            Self::Large => px(1.6),
            Self::Medium => px(1.25),
            Self::Small => px(1.0),
        }
    }

    fn indicator_length(self) -> Pixels {
        match self {
            Self::Large => px(16.0),
            Self::Medium => px(12.0),
            Self::Small => px(8.0),
        }
    }

    fn indicator_width(self) -> Pixels {
        match self {
            Self::Large => px(8.0),
            Self::Medium => px(6.0),
            Self::Small => px(4.5),
        }
    }

    fn indicator_corner_radius(self) -> Pixels {
        match self {
            Self::Large => px(2.0),
            Self::Medium => px(1.6),
            Self::Small => px(1.25),
        }
    }
}

pub fn dial(id: impl Into<SharedString>, label: impl Into<SharedString>, value: f32, size: DialSize) -> SliderBuilder {
    slider::new(id)
        .range(0.0..=1.0)
        .step(0.001)
        .value(value)
        .angular(MIN_ANGLE, MAX_ANGLE)
        .template(Arc::new(NeumorphicDialTemplate { size, label: label.into() }))
}

struct NeumorphicDialTemplate {
    size: DialSize,
    label: SharedString,
}

impl SliderTemplate for NeumorphicDialTemplate {
    fn render(
        &self,
        model: &SliderRenderModel<'_>,
        handlers: SliderTemplateHandlers,
        primary_thumb_id: ThumbId,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let SliderTemplateHandlers { track_bounds, hover, mouse_down, mouse_up, mouse_up_out, drag_move, .. } =
            handlers;
        let primary = model
            .thumbs
            .iter()
            .find(|thumb| thumb.id == primary_thumb_id)
            .or_else(|| model.thumbs.first())
            .expect("slider thumb");
        let value = model.range.value_at(primary.position);
        let dial_size = self.size;
        let label = self.label.clone();

        let interactive_area = div()
            .id(model.id.clone())
            .relative()
            .size(dial_size.frame_diameter())
            .on_hover(hover)
            .on_mouse_down(gpui::MouseButton::Left, mouse_down)
            .on_mouse_up(gpui::MouseButton::Left, mouse_up)
            .on_mouse_up_out(gpui::MouseButton::Left, mouse_up_out)
            .on_drag(
                xgpui_luma::controls::slider::SliderDrag::new(model.id.clone(), primary_thumb_id),
                |drag, _, _, cx| {
                    cx.stop_propagation();
                    cx.new(|_| drag.clone())
                },
            )
            .on_drag_move({
                let drag_move = drag_move.clone();
                move |event, window, cx| drag_move(event, window, cx)
            })
            .child(
                canvas(
                    move |bounds, window, cx| track_bounds(&bounds, window, cx),
                    move |bounds, _, window, _| paint_dial(bounds, value, dial_size, window),
                )
                .size_full(),
            );

        let mut root = div()
            .id(format!("{}-shell", model.id))
            .flex_col()
            .items_center()
            .gap(px(12.0))
            .child(interactive_area)
            .child(
                div()
                    .w(dial_size.frame_diameter())
                    .flex()
                    .justify_center()
                    .text_size(dial_size.label_size())
                    .line_height(dial_size.label_line_height())
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .font_features(FontFeatures(Arc::new(vec![("smcp".into(), 1)])))
                    .text_color(rgb(0x596273))
                    .child(label),
            );

        if model.enabled {
            root = root.cursor_pointer();
        } else {
            root = root.opacity(DISABLED_OPACITY);
        }

        root
    }
}

fn angle_for_value(value: f32) -> f32 {
    MIN_ANGLE + value.clamp(0.0, 1.0) * ANGLE_SPAN
}

fn paint_dial(bounds: Bounds<Pixels>, value: f32, dial_size: DialSize, window: &mut Window) {
    let center = bounds.center();
    let diameter = dial_size.face_diameter();
    let radius = diameter / 2.0;
    let face_bounds = Bounds { origin: point(center.x - radius, center.y - radius), size: size(diameter, diameter) };
    let face_corners = Corners::all(radius);

    window.paint_drop_shadows(face_bounds, face_corners, &raised_shadows(dial_size));
    window.paint_quad(fill(face_bounds, rgb(0xE9EDF2)).corner_radii(face_corners));
    paint_face_noise(window, center, radius, dial_size);

    paint_circle_outline(
        window,
        center,
        radius - (dial_size.border_width() / 2.0),
        dial_size.border_width(),
        rgb(0x1F2530).into(),
    );
    paint_lower_inner_bevel(
        window,
        center,
        radius - dial_size.border_width() - (dial_size.inner_border_width() / 2.0),
        dial_size.inner_border_width(),
    );
    paint_tick_ring(window, center, radius, value, dial_size);
    paint_indicator(window, center, radius, value, dial_size);
}

fn raised_shadows(dial_size: DialSize) -> [BoxShadow; 2] {
    match dial_size {
        DialSize::Large => [
            BoxShadow {
                color: hsla(220.0 / 360.0, 0.16, 0.14, 0.18),
                offset: point(px(0.0), px(4.0)),
                blur_radius: px(6.0),
                spread_radius: px(0.0),
                inset: false,
            },
            BoxShadow {
                color: hsla(220.0 / 360.0, 0.22, 0.09, 0.62),
                offset: point(px(4.0), px(22.0)),
                blur_radius: px(27.0),
                spread_radius: px(-6.0),
                inset: false,
            },
        ],
        DialSize::Medium => [
            BoxShadow {
                color: hsla(220.0 / 360.0, 0.16, 0.14, 0.16),
                offset: point(px(0.0), px(3.0)),
                blur_radius: px(5.0),
                spread_radius: px(0.0),
                inset: false,
            },
            BoxShadow {
                color: hsla(220.0 / 360.0, 0.22, 0.09, 0.46),
                offset: point(px(2.0), px(13.0)),
                blur_radius: px(16.0),
                spread_radius: px(-3.0),
                inset: false,
            },
        ],
        DialSize::Small => [
            BoxShadow {
                color: hsla(220.0 / 360.0, 0.16, 0.14, 0.16),
                offset: point(px(0.0), px(2.0)),
                blur_radius: px(4.0),
                spread_radius: px(0.0),
                inset: false,
            },
            BoxShadow {
                color: hsla(220.0 / 360.0, 0.22, 0.09, 0.46),
                offset: point(px(2.0), px(11.0)),
                blur_radius: px(13.0),
                spread_radius: px(-2.5),
                inset: false,
            },
        ],
    }
}

fn paint_face_noise(window: &mut Window, center: Point<Pixels>, face_radius: Pixels, dial_size: DialSize) {
    let step = match dial_size {
        DialSize::Large => 4_i32,
        DialSize::Medium => 4_i32,
        DialSize::Small => 3_i32,
    };
    let center_x = center.x.as_f32();
    let center_y = center.y.as_f32();
    let radius = face_radius.as_f32();
    let min_x = (center_x - radius) as i32;
    let max_x = (center_x + radius) as i32;
    let min_y = (center_y - radius) as i32;
    let max_y = (center_y + radius) as i32;
    let radius_sq = radius * radius;

    for y in (min_y..=max_y).step_by(step as usize) {
        for x in (min_x..=max_x).step_by(step as usize) {
            let sample_x = x as f32 + (step as f32 * 0.5);
            let sample_y = y as f32 + (step as f32 * 0.5);
            let dx = sample_x - center_x;
            let dy = sample_y - center_y;

            if (dx * dx) + (dy * dy) > radius_sq {
                continue;
            }

            let noise = hash_noise(x / step, y / step);
            let (size_px, color) = if noise < 0.060 {
                (1.0, hsla(220.0 / 360.0, 0.10, 0.28, 0.042))
            } else if noise > 0.952 {
                (1.0, hsla(0.0, 0.0, 1.0, 0.034))
            } else {
                continue;
            };

            let speck_size = px(size_px);
            let speck_origin = point(px(sample_x - (size_px * 0.5)), px(sample_y - (size_px * 0.5)));
            let speck_bounds = Bounds { origin: speck_origin, size: size(speck_size, speck_size) };
            window.paint_quad(fill(speck_bounds, color).corner_radii(Corners::all(px(0.5))));
        }
    }
}

fn hash_noise(x: i32, y: i32) -> f32 {
    let mut n = x.wrapping_mul(374_761_393).wrapping_add(y.wrapping_mul(668_265_263));
    n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
    let bits = (n ^ (n >> 16)) as u32;
    bits as f32 / u32::MAX as f32
}

fn paint_tick_ring(window: &mut Window, center: Point<Pixels>, face_radius: Pixels, value: f32, dial_size: DialSize) {
    let tick_radius = face_radius + dial_size.tick_ring_gap();
    let tick_count = dial_size.tick_count();

    for index in 0..=tick_count {
        let t = index as f32 / tick_count as f32;
        let angle = MIN_ANGLE + ANGLE_SPAN * t;
        let inner = point(
            center.x + angle.cos() * (tick_radius + dial_size.tick_inner_length()),
            center.y + angle.sin() * (tick_radius + dial_size.tick_inner_length()),
        );
        let outer = point(
            center.x + angle.cos() * (tick_radius + dial_size.tick_outer_length()),
            center.y + angle.sin() * (tick_radius + dial_size.tick_outer_length()),
        );

        let color = if t <= value {
            hsla(220.0 / 360.0, 0.14, 0.12, 0.9)
        } else {
            hsla(220.0 / 360.0, 0.16, 0.67, 0.72)
        };

        paint_segment(window, inner, outer, dial_size.tick_stroke_width(), color);
    }
}

fn paint_indicator(window: &mut Window, center: Point<Pixels>, face_radius: Pixels, value: f32, dial_size: DialSize) {
    let angle = angle_for_value(value);
    let distance = match dial_size {
        DialSize::Large => face_radius - px(18.0),
        DialSize::Medium => face_radius - px(14.0),
        DialSize::Small => face_radius - px(12.0),
    };
    let indicator_center = point(center.x + angle.cos() * distance, center.y + angle.sin() * distance);
    paint_rounded_indicator(window, indicator_center, angle.to_degrees(), dial_size, rgb(0x2B3039).into());
}

fn paint_circle_outline(
    window: &mut Window,
    center: Point<Pixels>,
    radius: Pixels,
    stroke_width: Pixels,
    color: gpui::Hsla,
) {
    let mut builder = PathBuilder::stroke(stroke_width);
    let start = point(center.x + radius, center.y);
    let mid = point(center.x - radius, center.y);

    builder.move_to(start);
    builder.arc_to(point(radius, radius), px(0.0), false, true, mid);
    builder.arc_to(point(radius, radius), px(0.0), false, true, start);
    builder.close();

    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }
}

fn paint_lower_inner_bevel(window: &mut Window, center: Point<Pixels>, radius: Pixels, stroke_width: Pixels) {
    let start_angle = 0.05 * PI;
    let end_angle = 0.95 * PI;
    let segment_count = 48;
    let layers = [
        (px(0.0), stroke_width * 1.35, 0.12, 0.58),
        (px(1.2), stroke_width * 1.95, 0.08, 0.30),
        (px(2.4), stroke_width * 2.45, 0.05, 0.16),
    ];

    for (radius_offset, layer_width, edge_alpha, center_alpha) in layers {
        let layer_radius = radius - radius_offset;

        for index in 0..segment_count {
            let t0 = index as f32 / segment_count as f32;
            let t1 = (index + 1) as f32 / segment_count as f32;
            let angle0 = start_angle + (end_angle - start_angle) * t0;
            let angle1 = start_angle + (end_angle - start_angle) * t1;

            let start = point(center.x + angle0.cos() * layer_radius, center.y + angle0.sin() * layer_radius);
            let end = point(center.x + angle1.cos() * layer_radius, center.y + angle1.sin() * layer_radius);

            let mid_t = (t0 + t1) * 0.5;
            let emphasis = (1.0 - ((mid_t - 0.5).abs() / 0.5)).clamp(0.0, 1.0);
            let feather = emphasis * emphasis * (3.0 - (2.0 * emphasis));
            let alpha = edge_alpha + ((center_alpha - edge_alpha) * feather);
            let lightness = 0.50 + (0.11 * (1.0 - feather));
            let color = hsla(220.0 / 360.0, 0.10, lightness, alpha);

            paint_segment(window, start, end, layer_width, color);
        }
    }
}

fn paint_segment(
    window: &mut Window,
    start: Point<Pixels>,
    end: Point<Pixels>,
    stroke_width: Pixels,
    color: gpui::Hsla,
) {
    let mut builder = PathBuilder::stroke(stroke_width);
    builder.move_to(start);
    builder.line_to(end);

    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }
}

fn paint_rounded_indicator(
    window: &mut Window,
    center: Point<Pixels>,
    angle_degrees: f32,
    dial_size: DialSize,
    color: gpui::Hsla,
) {
    let length = dial_size.indicator_length();
    let width = dial_size.indicator_width();
    let radius = dial_size.indicator_corner_radius();
    let half_length = length / 2.0;
    let half_width = width / 2.0;

    let left = -half_length;
    let right = half_length;
    let top = -half_width;
    let bottom = half_width;

    let mut builder = PathBuilder::fill();
    builder.move_to(point(left + radius, top));
    builder.line_to(point(right - radius, top));
    builder.arc_to(point(radius, radius), px(0.0), false, true, point(right, top + radius));
    builder.line_to(point(right, bottom - radius));
    builder.arc_to(point(radius, radius), px(0.0), false, true, point(right - radius, bottom));
    builder.line_to(point(left + radius, bottom));
    builder.arc_to(point(radius, radius), px(0.0), false, true, point(left, bottom - radius));
    builder.line_to(point(left, top + radius));
    builder.arc_to(point(radius, radius), px(0.0), false, true, point(left + radius, top));
    builder.close();
    builder.rotate(angle_degrees);
    builder.translate(center);

    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }
}
