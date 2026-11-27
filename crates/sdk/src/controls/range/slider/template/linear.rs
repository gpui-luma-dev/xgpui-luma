use std::sync::{Arc, OnceLock};

use gpui::{App, Div, Pixels, Stateful, Window, div, px, prelude::*};

use super::{
    SliderBoundsHandler, SliderInteractionHandlers, SliderTemplate, SliderTemplateHandlers, attach_linear_interaction,
    attach_thumb_drag, effective_thumb_radius, render_domain_track_layer, render_linear_thumb, track_bounds_canvas,
    track_surface_background,
};
use crate::infra::StyledExt;
use crate::controls::slider::{SliderTheme, default_slider_theme};

use super::super::layout::{display_position, segment_corner_radii, segment_display_span};
use super::super::model::{SliderOrientation, SliderRenderModel, ThumbId, TrackPresentation, TrackSegmentKind};

pub struct ThemedSliderTemplate {
    theme: Arc<dyn SliderTheme>,
}

impl ThemedSliderTemplate {
    pub fn new(theme: Arc<dyn SliderTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_slider_template() -> Arc<dyn SliderTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn SliderTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedSliderTemplate::new(default_slider_theme()))).clone()
}

impl SliderTemplate for ThemedSliderTemplate {
    fn resolve_look(&self, state: crate::theme::InteractionState) -> super::super::SliderLook {
        self.theme.resolve(state)
    }

    fn render(
        &self,
        model: &SliderRenderModel<'_>,
        handlers: SliderTemplateHandlers,
        _primary_thumb_id: ThumbId,
        window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let look = self.theme.resolve(model.state);
        let long_axis = look.width;
        let short_axis = look.height;
        let cross_axis = look.track_height;
        let root_height = match model.orientation {
            SliderOrientation::Horizontal => short_axis,
            SliderOrientation::Vertical => long_axis,
        };
        let root_width = short_axis;
        let track_radius =
            model.corner_radius.map(|radius| radius.to_pixels(window.rem_size())).unwrap_or(px(look.radius));
        let thumb_radius = effective_thumb_radius(model, &look, window.rem_size());

        let SliderTemplateHandlers {
            track_bounds,
            hover,
            mouse_down,
            mouse_move,
            mouse_up,
            mouse_up_out,
            drag_move,
            thumb_mouse_down,
        } = handlers;
        let interaction = SliderInteractionHandlers {
            hover,
            mouse_down,
            mouse_move,
            mouse_up,
            mouse_up_out,
            drag_move: drag_move.clone(),
        };
        let model_id = model.id.clone();
        let enabled = model.enabled;

        let root = div()
            .id(model.id.clone())
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .when(model.orientation == SliderOrientation::Horizontal, |this| {
                this.w_full().min_w(px(0.0)).h(px(root_height)).px(px(look.thumb_size * 0.5))
            })
            .when(model.orientation == SliderOrientation::Vertical, |this| this.w(px(root_width)).h(px(root_height)));

        let root = match model.orientation {
            SliderOrientation::Horizontal => {
                let track_top = (short_axis - cross_axis) * 0.5;
                let thumb_top = (short_axis - look.thumb_size) * 0.5;
                let thumb_center_offset = -(look.thumb_size * 0.5);

                let track = render_horizontal_track(model, &look, track_top, cross_axis, track_radius, track_bounds);
                let outline = track_outline(&look, track_radius)
                    .left(px(0.0))
                    .right(px(0.0))
                    .top(px(track_top))
                    .h(px(cross_axis));

                let thumbs = ordered_thumbs(model).into_iter().map(|thumb| {
                    let display_percentage = display_position(thumb.position.clamp(0.0, 1.0), model.reversed);
                    let node = render_linear_thumb(
                        model,
                        &look,
                        ("thumb", thumb.id.as_u64()),
                        display_percentage,
                        thumb_top,
                        Some(thumb_center_offset),
                        None,
                        thumb_radius,
                        Some(thumb),
                    );
                    attach_thumb_drag(node, &model_id, thumb.id, thumb_mouse_down.clone(), drag_move.clone(), enabled)
                });

                root.child(track).child(outline).children(thumbs)
            }
            SliderOrientation::Vertical => {
                let track_left = (short_axis - cross_axis) * 0.5;
                let thumb_left = (short_axis - look.thumb_size) * 0.5;

                let track =
                    render_vertical_track(model, &look, track_left, long_axis, cross_axis, track_radius, track_bounds);
                let outline = track_outline(&look, track_radius)
                    .left(px(track_left))
                    .top(px(0.0))
                    .w(px(cross_axis))
                    .h(px(long_axis));

                let thumbs = ordered_thumbs(model).into_iter().map(|thumb| {
                    let display_percentage = display_position(thumb.position.clamp(0.0, 1.0), model.reversed);
                    let thumb_top = (long_axis - look.thumb_size).max(0.0) * (1.0 - display_percentage);
                    let node = render_linear_thumb(
                        model,
                        &look,
                        ("thumb", thumb.id.as_u64()),
                        display_percentage,
                        thumb_top,
                        None,
                        Some(thumb_left),
                        thumb_radius,
                        Some(thumb),
                    );
                    attach_thumb_drag(node, &model_id, thumb.id, thumb_mouse_down.clone(), drag_move.clone(), enabled)
                });

                root.child(track).child(outline).children(thumbs)
            }
        };

        attach_linear_interaction(root, model, interaction)
    }
}

/// An outline the track's shape (place it like the track), over the track's segments and under the thumbs;
/// empty when the look's track border width is zero.
fn track_outline(look: &crate::controls::slider::SliderLook, radius: Pixels) -> Div {
    div().absolute().rounded(radius).when(look.track_border_width > 0.0, |outline| {
        outline.border(px(look.track_border_width)).border_color(look.track_border)
    })
}

/// Inactive thumbs first; active thumb last so it paints on top while dragging.
fn ordered_thumbs<'a>(model: &'a SliderRenderModel<'_>) -> Vec<&'a super::super::model::SliderThumbValue> {
    let mut thumbs: Vec<_> = model.thumbs.iter().collect();
    thumbs.sort_by_key(|thumb| model.active_thumb_id == Some(thumb.id));
    thumbs
}

fn render_horizontal_track(
    model: &SliderRenderModel<'_>,
    look: &crate::controls::slider::SliderLook,
    track_top: f32,
    cross_axis: f32,
    track_radius: Pixels,
    track_bounds: SliderBoundsHandler,
) -> Stateful<Div> {
    let uses_sibling_segments = uses_partitioned_track(model);

    let mut track = div()
        .id("track")
        .absolute()
        .left(px(0.0))
        .right(px(0.0))
        .top(px(track_top))
        .h(px(cross_axis))
        .rounded(track_radius)
        .overflow_hidden()
        .when(model.domain_track.is_none() && !uses_sibling_segments, |this| {
            this.bg(track_surface_background(model, look))
                .map(|track| crate::style::fade(track, look.track_opacity))
        });

    if let Some(layer) = render_domain_track_layer(model, track_radius) {
        track = track.child(layer);
    } else if model.presentation == TrackPresentation::Domain {
        track =
            track.children(
                model.track_segments.iter().filter(|segment| segment.kind == TrackSegmentKind::Blocked).map(
                    |segment| render_fill_segment(segment, model.orientation, model.reversed, look, track_radius, true),
                ),
            );
    } else {
        track =
            track.children(model.track_segments.iter().map(|segment| {
                render_fill_segment(segment, model.orientation, model.reversed, look, track_radius, true)
            }));
    }

    track.child(track_bounds_canvas(track_bounds))
}

fn render_vertical_track(
    model: &SliderRenderModel<'_>,
    look: &crate::controls::slider::SliderLook,
    track_left: f32,
    long_axis: f32,
    cross_axis: f32,
    track_radius: Pixels,
    track_bounds: SliderBoundsHandler,
) -> Stateful<Div> {
    let uses_sibling_segments = uses_partitioned_track(model);

    let mut track = div()
        .id("track")
        .absolute()
        .left(px(track_left))
        .top(px(0.0))
        .w(px(cross_axis))
        .h(px(long_axis))
        .rounded(track_radius)
        .overflow_hidden()
        .when(model.domain_track.is_none() && !uses_sibling_segments, |this| {
            this.bg(track_surface_background(model, look))
                .map(|track| crate::style::fade(track, look.track_opacity))
        });

    if let Some(layer) = render_domain_track_layer(model, track_radius) {
        track = track.child(layer);
    } else if model.presentation == TrackPresentation::Domain {
        track =
            track.children(
                model.track_segments.iter().filter(|segment| segment.kind == TrackSegmentKind::Blocked).map(
                    |segment| {
                        render_fill_segment(segment, model.orientation, model.reversed, look, track_radius, false)
                    },
                ),
            );
    } else {
        track =
            track.children(model.track_segments.iter().map(|segment| {
                render_fill_segment(segment, model.orientation, model.reversed, look, track_radius, false)
            }));
    }

    track.child(track_bounds_canvas(track_bounds))
}

fn render_fill_segment(
    segment: &super::super::model::TrackSegment,
    orientation: SliderOrientation,
    reversed: bool,
    look: &crate::controls::slider::SliderLook,
    track_radius: Pixels,
    horizontal: bool,
) -> Div {
    use gpui::relative;

    let (display_start, display_span) = segment_display_span(segment, reversed);
    let background = segment_background(segment.kind, look);
    let corner_radii = segment_corner_radii(display_start, display_span, orientation, track_radius);
    let opacity = segment_opacity(segment.kind, look);

    if horizontal {
        let segment = div()
            .absolute()
            .left(relative(display_start))
            .top(px(0.0))
            .h_full()
            .w(relative(display_span))
            .bg(background)
            .corner_radii(corner_radii);
        crate::style::fade(segment, opacity)
    } else {
        let segment = div()
            .absolute()
            .left(px(0.0))
            .bottom(relative(display_start))
            .w_full()
            .h(relative(display_span))
            .bg(background)
            .corner_radii(corner_radii);
        crate::style::fade(segment, opacity)
    }
}

/// A segment fades with the part whose color it draws (`:disabled { opacity }` on `fill` or `track`).
fn segment_opacity(kind: TrackSegmentKind, look: &crate::controls::slider::SliderLook) -> f32 {
    match kind {
        TrackSegmentKind::Active | TrackSegmentKind::Domain => look.fill_opacity,
        TrackSegmentKind::Inactive | TrackSegmentKind::Blocked => look.track_opacity,
    }
}

fn segment_background(kind: TrackSegmentKind, look: &crate::controls::slider::SliderLook) -> gpui::Hsla {
    match kind {
        TrackSegmentKind::Active | TrackSegmentKind::Domain => look.fill_background,
        TrackSegmentKind::Inactive | TrackSegmentKind::Blocked => look.track_background,
    }
}

fn uses_partitioned_track(model: &SliderRenderModel<'_>) -> bool {
    match model.presentation {
        TrackPresentation::Fill => !model.track_segments.is_empty(),
        TrackPresentation::Domain => {
            model.track_segments.iter().any(|segment| segment.kind == TrackSegmentKind::Blocked)
        }
    }
}
