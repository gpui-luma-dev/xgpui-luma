mod angular_dial;
mod circular_ring;
mod linear;

use std::sync::Arc;
use std::f32::consts::FRAC_PI_2;

use gpui::{
    App, BorderStyle, Bounds, Corners, Div, DragMoveEvent, Edges, Hsla, IntoElement, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, PaintQuad, Pixels, Point, SharedString, Stateful, Window, canvas, div, point, px,
    size, transparent_black, prelude::*,
};

use crate::infra::{Arc as ShapeArc, ArcData};

use super::model::{SliderRenderModel, SliderThumbValue, ThumbId};
use super::SliderDrag;

pub use angular_dial::{ThemedAngularDialTemplate, default_angular_dial_template};
pub use circular_ring::{ThemedCircularRingTemplate, default_circular_ring_template};
pub use linear::{ThemedSliderTemplate, default_slider_template};

pub use crate::controls::slider::domain::render_domain_track_layer;

pub type SliderBoundsHandler = Box<dyn Fn(&Bounds<Pixels>, &mut Window, &mut App) + Send + Sync>;
pub type SliderHoverHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + Send + Sync>;
pub type SliderMouseDownHandler = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + Send + Sync>;
pub type SliderMouseMoveHandler = Box<dyn Fn(&MouseMoveEvent, &mut Window, &mut App) + Send + Sync>;
pub type SliderMouseUpHandler = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + Send + Sync>;
pub type SliderDragMoveHandler = Arc<dyn Fn(&DragMoveEvent<SliderDrag>, &mut Window, &mut App) + Send + Sync>;
pub type SliderThumbMouseDownHandler = Arc<dyn Fn(&ThumbId, &MouseDownEvent, &mut Window, &mut App) + Send + Sync>;

pub struct SliderTemplateHandlers {
    pub track_bounds: SliderBoundsHandler,
    pub hover: SliderHoverHandler,
    pub mouse_down: SliderMouseDownHandler,
    pub mouse_move: SliderMouseMoveHandler,
    pub mouse_up: SliderMouseUpHandler,
    pub mouse_up_out: SliderMouseUpHandler,
    pub drag_move: SliderDragMoveHandler,
    pub thumb_mouse_down: SliderThumbMouseDownHandler,
}

pub type SliderTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &SliderRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static>;

pub struct SliderInteractionHandlers {
    pub hover: SliderHoverHandler,
    pub mouse_down: SliderMouseDownHandler,
    pub mouse_move: SliderMouseMoveHandler,
    pub mouse_up: SliderMouseUpHandler,
    pub mouse_up_out: SliderMouseUpHandler,
    pub drag_move: SliderDragMoveHandler,
}

impl From<SliderTemplateHandlers> for (SliderBoundsHandler, SliderInteractionHandlers) {
    fn from(handlers: SliderTemplateHandlers) -> Self {
        (
            handlers.track_bounds,
            SliderInteractionHandlers {
                hover: handlers.hover,
                mouse_down: handlers.mouse_down,
                mouse_move: handlers.mouse_move,
                mouse_up: handlers.mouse_up,
                mouse_up_out: handlers.mouse_up_out,
                drag_move: handlers.drag_move,
            },
        )
    }
}

pub trait SliderTemplate: Send + Sync {
    /// Resolve the same geometry for pointer hit testing and painted chrome.
    fn resolve_look(&self, state: crate::theme::InteractionState) -> super::SliderLook {
        super::theme::default_slider_theme().resolve(state)
    }

    fn render(
        &self,
        model: &SliderRenderModel<'_>,
        handlers: SliderTemplateHandlers,
        primary_thumb_id: ThumbId,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div>;
}

struct ModifiedSliderTemplate {
    base: Arc<dyn SliderTemplate>,
    modifiers: Vec<SliderTemplateModifier>,
}

impl ModifiedSliderTemplate {
    fn new(base: Arc<dyn SliderTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier(mut self, modifier: SliderTemplateModifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    fn apply_modifiers(&self, mut root: Stateful<Div>, model: &SliderRenderModel<'_>) -> Stateful<Div> {
        for modifier in &self.modifiers {
            root = (modifier)(root, model);
        }
        root
    }
}

pub(super) fn modified_slider_template<F>(template: Arc<dyn SliderTemplate>, modifier: F) -> Arc<dyn SliderTemplate>
where
    F: Fn(Stateful<Div>, &SliderRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
{
    Arc::new(ModifiedSliderTemplate::new(template).with_modifier(Box::new(modifier)))
}

impl SliderTemplate for ModifiedSliderTemplate {
    fn resolve_look(&self, state: crate::theme::InteractionState) -> super::SliderLook {
        self.base.resolve_look(state)
    }

    fn render(
        &self,
        model: &SliderRenderModel<'_>,
        handlers: SliderTemplateHandlers,
        primary_thumb_id: ThumbId,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let root = self.base.render(model, handlers, primary_thumb_id, window, cx);
        self.apply_modifiers(root, model)
    }
}

pub(crate) fn uses_static_track_surface(model: &SliderRenderModel<'_>) -> bool {
    model.thumb_policy.is_multi_thumb() || model.presentation == super::model::TrackPresentation::Domain
}

pub(crate) fn track_surface_background(
    model: &SliderRenderModel<'_>,
    look: &crate::controls::slider::SliderLook,
) -> Hsla {
    if uses_static_track_surface(model) {
        look.fill_background
    } else {
        look.track_background
    }
}

pub(crate) fn track_muted_background(look: &crate::controls::slider::SliderLook) -> Hsla {
    look.track_background
}

pub(crate) fn effective_thumb_radius(
    model: &SliderRenderModel<'_>,
    look: &crate::controls::slider::SliderLook,
    rem_size: Pixels,
) -> f32 {
    model.thumb_radius.map(|radius| f32::from(radius.to_pixels(rem_size))).unwrap_or(look.radius)
}

pub(crate) fn render_slider_thumb_at(
    look: &crate::controls::slider::SliderLook,
    id: impl Into<gpui::ElementId>,
    center_x: f32,
    center_y: f32,
    thumb_radius: f32,
    thumb: Option<&SliderThumbValue>,
) -> Stateful<Div> {
    let thumb_fill = thumb.and_then(|thumb| thumb.preview).unwrap_or(look.thumb_background);
    let offset = look.thumb_size * 0.5;

    div()
        .id(id)
        .absolute()
        .left(px(center_x - offset))
        .top(px(center_y - offset))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(thumb_radius))
        .child(
            div()
                .size(px(look.thumb_size))
                .bg(thumb_fill)
                .border(px(look.thumb_border_width))
                .border_color(look.thumb_border)
                .rounded(px(thumb_radius))
                .shadow(look.thumb_shadow.clone()),
        )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_linear_thumb(
    model: &SliderRenderModel<'_>,
    look: &crate::controls::slider::SliderLook,
    id: impl Into<gpui::ElementId>,
    display_percentage: f32,
    top: f32,
    main_offset: Option<f32>,
    cross_offset: Option<f32>,
    thumb_radius: f32,
    thumb: Option<&SliderThumbValue>,
) -> Stateful<Div> {
    use super::model::SliderOrientation;
    use gpui::relative;

    let thumb_fill = thumb.and_then(|thumb| thumb.preview).unwrap_or(look.thumb_background);
    let mut thumb = div()
        .id(id)
        .absolute()
        .top(px(top))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(thumb_radius))
        .child(
            div()
                .size(px(look.thumb_size))
                .bg(thumb_fill)
                .border(px(look.thumb_border_width))
                .border_color(look.thumb_border)
                .rounded(px(thumb_radius))
                .shadow(look.thumb_shadow.clone()),
        );

    thumb = match model.orientation {
        SliderOrientation::Horizontal => {
            let mut node = thumb.left(relative(display_percentage));
            if let Some(offset) = main_offset {
                node = node.ml(px(offset));
            }
            node
        }
        SliderOrientation::Vertical => {
            if let Some(offset) = cross_offset {
                thumb = thumb.left(px(offset));
            }
            thumb
        }
    };

    thumb
}

pub fn track_bounds_canvas(track_bounds: SliderBoundsHandler) -> impl IntoElement {
    canvas(move |bounds, window, cx| track_bounds(&bounds, window, cx), |_, _, _, _| {})
        .absolute()
        .size_full()
}

pub fn attach_radial_interaction(
    mut root: Stateful<Div>,
    model: &SliderRenderModel<'_>,
    handlers: SliderInteractionHandlers,
    primary_thumb_id: ThumbId,
) -> Stateful<Div> {
    let SliderInteractionHandlers { hover, mouse_down, mouse_move, mouse_up, mouse_up_out, drag_move, .. } = handlers;

    root = root
        .on_hover(hover)
        .on_mouse_down(MouseButton::Left, mouse_down)
        .on_mouse_move(mouse_move)
        .on_mouse_up(MouseButton::Left, mouse_up)
        .on_mouse_up_out(MouseButton::Left, mouse_up_out)
        .on_drag(SliderDrag::new(model.id.clone(), primary_thumb_id), |drag, _, _, cx| {
            cx.stop_propagation();
            cx.new(|_| drag.clone())
        })
        .on_drag_move({
            let drag_move = drag_move.clone();
            move |event, window, cx| drag_move(event, window, cx)
        });

    if model.enabled {
        root.cursor_pointer()
    } else {
        root.cursor_not_allowed()
    }
}

pub fn attach_linear_interaction(
    mut root: Stateful<Div>,
    model: &SliderRenderModel<'_>,
    handlers: SliderInteractionHandlers,
) -> Stateful<Div> {
    let SliderInteractionHandlers { hover, mouse_down, mouse_move, mouse_up, mouse_up_out, drag_move, .. } = handlers;

    root = root
        .on_hover(hover)
        .on_mouse_down(MouseButton::Left, mouse_down)
        .on_mouse_move(mouse_move)
        .on_mouse_up(MouseButton::Left, mouse_up)
        .on_mouse_up_out(MouseButton::Left, mouse_up_out)
        .on_drag_move({
            let drag_move = drag_move.clone();
            move |event, window, cx| drag_move(event, window, cx)
        });

    if model.enabled {
        root.cursor_pointer()
    } else {
        root.cursor_not_allowed()
    }
}

pub fn attach_thumb_drag(
    mut thumb: Stateful<Div>,
    model_id: &SharedString,
    thumb_id: ThumbId,
    mouse_down: SliderThumbMouseDownHandler,
    drag_move: SliderDragMoveHandler,
    enabled: bool,
) -> Stateful<Div> {
    thumb = thumb
        .on_mouse_down(MouseButton::Left, {
            let mouse_down = mouse_down.clone();
            move |event, window, cx| mouse_down(&thumb_id, event, window, cx)
        })
        .on_drag(SliderDrag::new(model_id.clone(), thumb_id), |drag, _, _, cx| {
            cx.stop_propagation();
            cx.new(|_| drag.clone())
        })
        .on_drag_move({
            let drag_move = drag_move.clone();
            move |event, window, cx| drag_move(event, window, cx)
        });

    if enabled {
        thumb = thumb.cursor_pointer();
    }

    thumb
}

pub(crate) fn arc_annulus_radii(arc_thickness: f32, track_radius: f32) -> (f32, f32) {
    let outer = track_radius + arc_thickness * 0.5;
    let inner = (track_radius - arc_thickness * 0.5).max(0.0);
    (inner, outer)
}

fn arc_shape_angles(start_angle: f32, end_angle: f32) -> (f32, f32) {
    // `Arc` path math applies -π/2 internally; offset so angles match thumb polar coords.
    (start_angle + FRAC_PI_2, end_angle + FRAC_PI_2)
}

pub(crate) fn paint_radial_annulus(
    bounds: Bounds<Pixels>,
    track_radius: f32,
    start_angle: f32,
    end_angle: f32,
    color: Hsla,
    arc_thickness: f32,
    window: &mut Window,
) {
    if end_angle <= start_angle + f32::EPSILON || arc_thickness <= f32::EPSILON {
        return;
    }

    let (inner, outer) = arc_annulus_radii(arc_thickness, track_radius);
    let (start_angle, end_angle) = arc_shape_angles(start_angle, end_angle);
    let arc_shape = ShapeArc::new().inner_radius(inner).outer_radius(outer);
    let data = ArcData { data: &(), index: 0, value: 1.0, start_angle, end_angle, pad_angle: 0.0 };
    arc_shape.paint(&data, color, None, None, &bounds, window);
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_radial_fill_track(
    bounds: Bounds<Pixels>,
    track_radius: f32,
    min_angle: f32,
    max_angle: f32,
    percentage: f32,
    track_background: Hsla,
    fill_background: Hsla,
    arc_thickness: f32,
    window: &mut Window,
) {
    let span = max_angle - min_angle;
    let percentage = percentage.clamp(0.0, 1.0);

    paint_radial_annulus(bounds, track_radius, min_angle, max_angle, track_background, arc_thickness, window);

    if percentage > f32::EPSILON {
        let fill_end = min_angle + span * percentage;
        paint_radial_annulus(bounds, track_radius, min_angle, fill_end, fill_background, arc_thickness, window);
    }

    let center = bounds.center();
    let fill_segments = usize::from(percentage > f32::EPSILON);
    let segment_count = if percentage >= 1.0 - f32::EPSILON { 1 } else { 2 };
    let (start_color, end_color) =
        arc_fill_end_cap_colors(fill_segments, segment_count, track_background, fill_background);
    paint_arc_open_end_caps(center, track_radius, min_angle, max_angle, start_color, end_color, arc_thickness, window);
}

pub(crate) fn paint_arc_round_cap(
    center: Point<Pixels>,
    radius: f32,
    angle: f32,
    diameter: f32,
    color: Hsla,
    window: &mut Window,
) {
    if diameter <= f32::EPSILON {
        return;
    }

    let cap_center = polar_point(center, radius, angle);
    let cap_radius = diameter * 0.5;
    window.paint_quad(PaintQuad {
        bounds: Bounds {
            origin: point(cap_center.x - px(cap_radius), cap_center.y - px(cap_radius)),
            size: size(px(diameter), px(diameter)),
        },
        corner_radii: Corners::all(px(cap_radius)),
        background: color.into(),
        border_widths: Edges::default(),
        border_color: transparent_black(),
        border_style: BorderStyle::default(),
    });
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_arc_open_end_caps(
    center: Point<Pixels>,
    radius: f32,
    min_angle: f32,
    max_angle: f32,
    start_color: Hsla,
    end_color: Hsla,
    stroke_width: f32,
    window: &mut Window,
) {
    use std::f32::consts::TAU;

    let span = max_angle - min_angle;
    if span <= f32::EPSILON || span >= TAU - f32::EPSILON {
        return;
    }

    paint_arc_round_cap(center, radius, min_angle, stroke_width, start_color, window);
    paint_arc_round_cap(center, radius, max_angle, stroke_width, end_color, window);
}

pub(crate) fn arc_fill_end_cap_colors(
    fill_segments: usize,
    segment_count: usize,
    track_background: Hsla,
    fill_background: Hsla,
) -> (Hsla, Hsla) {
    let start_color = if fill_segments > 0 {
        fill_background
    } else {
        track_background
    };
    let end_color = if fill_segments >= segment_count {
        fill_background
    } else {
        track_background
    };
    (start_color, end_color)
}

pub(crate) fn polar_point(center: Point<Pixels>, radius: f32, angle: f32) -> Point<Pixels> {
    gpui::point(center.x + px(radius * angle.cos()), center.y + px(radius * angle.sin()))
}

#[cfg(test)]
mod tests {
    use gpui::hsla;

    use super::*;

    #[test]
    fn arc_fill_end_cap_colors_match_outer_segment_ends() {
        let track = hsla(0.0, 0.0, 0.5, 1.0);
        let fill = hsla(0.6, 0.8, 0.5, 1.0);

        assert_eq!(arc_fill_end_cap_colors(0, 64, track, fill), (track, track));
        assert_eq!(arc_fill_end_cap_colors(32, 64, track, fill), (fill, track));
        assert_eq!(arc_fill_end_cap_colors(64, 64, track, fill), (fill, fill));
    }
}
