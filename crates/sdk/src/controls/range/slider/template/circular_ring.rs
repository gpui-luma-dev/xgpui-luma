use crate::style::PartRead;
use std::sync::{Arc, OnceLock};

use gpui::{App, Div, Stateful, Window, canvas, div, px, prelude::*};

use super::{
    SliderTemplate, SliderTemplateHandlers, attach_radial_interaction, effective_thumb_radius, paint_radial_annulus,
    paint_radial_fill_track, paint_radial_track_outline, render_slider_thumb_at, track_bounds_canvas,
    track_muted_background, track_surface_background, uses_static_track_surface,
};
use crate::controls::slider::{SliderTheme, default_slider_theme};

use super::super::input::{SliderInputStrategy, angle_for_percentage};
use super::super::layout::display_position;
use super::super::model::{SliderRenderModel, ThumbId, TrackPresentation};

pub struct ThemedCircularRingTemplate {
    theme: Arc<dyn SliderTheme>,
}

impl ThemedCircularRingTemplate {
    pub fn new(theme: Arc<dyn SliderTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_circular_ring_template() -> Arc<dyn SliderTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn SliderTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedCircularRingTemplate::new(default_slider_theme()))).clone()
}

impl SliderTemplate for ThemedCircularRingTemplate {
    fn resolve_style(&self, state: crate::theme::InteractionState) -> std::sync::Arc<xgpui_luma_style::Resolved> {
        super::super::resolve_slider(&self.theme.style(), state)
    }

    fn render(
        &self,
        model: &SliderRenderModel<'_>,
        handlers: SliderTemplateHandlers,
        primary_thumb_id: ThumbId,
        window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let look = super::super::resolve_slider(&self.theme.style(), model.state);
        let SliderInputStrategy::Angular { min_angle, max_angle } = model.strategy else {
            return div().id(model.id.clone());
        };

        let primary_thumb =
            model.thumbs.iter().find(|thumb| thumb.id == primary_thumb_id).or_else(|| model.thumbs.first());
        let thumb_position = primary_thumb.map(|thumb| thumb.position).unwrap_or(0.0);
        let display_percentage = display_position(thumb_position, model.reversed);
        let angle = angle_for_percentage(min_angle, max_angle, display_percentage);
        let dial_size = look.root().number("dial_size");
        let track_radius = look.root().number("dial_track_radius");
        let center = dial_size * 0.5;
        let thumb_x = center + track_radius * angle.cos();
        let thumb_y = center + track_radius * angle.sin();
        let arc_thickness = look.root().number("track_height");
        let presentation = model.presentation;
        let track_background = track_muted_background(&look);
        let fill_background = look.part(xgpui_luma_style::PartId::new("fill")).color("background");
        let surface_background = track_surface_background(model, &look);
        let static_surface = uses_static_track_surface(model);
        let (track_border, track_border_width) = (
            look.part(xgpui_luma_style::PartId::new("track")).color("border"),
            look.part(xgpui_luma_style::PartId::new("track")).number("border_width"),
        );
        let thumb_radius = effective_thumb_radius(model, &look, window.rem_size());

        let (track_bounds, interaction) = handlers.into();

        let root = div()
            .id(model.id.clone())
            .relative()
            .size(px(dial_size))
            .child(
                canvas(
                    move |_, _, _| (),
                    move |bounds, _, window, _| {
                        // The outline first; the track and fill are drawn inside it.
                        let arc_thickness = paint_radial_track_outline(
                            bounds,
                            track_radius,
                            min_angle,
                            max_angle,
                            track_border,
                            track_border_width,
                            arc_thickness,
                            window,
                        );
                        if static_surface {
                            paint_radial_annulus(
                                bounds,
                                track_radius,
                                min_angle,
                                max_angle,
                                surface_background,
                                arc_thickness,
                                window,
                            );
                            return;
                        }

                        match presentation {
                            TrackPresentation::Fill => paint_radial_fill_track(
                                bounds,
                                track_radius,
                                min_angle,
                                max_angle,
                                display_percentage,
                                track_background,
                                fill_background,
                                arc_thickness,
                                window,
                            ),
                            TrackPresentation::Domain => {
                                paint_radial_annulus(
                                    bounds,
                                    track_radius,
                                    min_angle,
                                    max_angle,
                                    surface_background,
                                    arc_thickness,
                                    window,
                                );
                            }
                        }
                    },
                )
                .absolute()
                .size_full(),
            )
            .child(render_slider_thumb_at(&look, "thumb", thumb_x, thumb_y, thumb_radius, primary_thumb))
            .child(track_bounds_canvas(track_bounds));

        attach_radial_interaction(root, model, interaction, primary_thumb_id)
    }
}
