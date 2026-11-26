use std::sync::{Arc, OnceLock};

use gpui::{App, Div, Stateful, Window, canvas, div, px, prelude::*};

use super::{
    SliderTemplate, SliderTemplateHandlers, attach_radial_interaction, effective_thumb_radius, paint_radial_annulus,
    paint_radial_fill_track, paint_radial_track_outline, render_slider_thumb_at, track_bounds_canvas,
};
use crate::controls::slider::{SliderTheme, default_slider_theme};

use super::super::input::{SliderInputStrategy, angle_for_percentage};
use super::super::layout::display_position;
use super::super::model::{SliderRenderModel, ThumbId, TrackPresentation};

pub struct ThemedAngularDialTemplate {
    theme: Arc<dyn SliderTheme>,
}

impl ThemedAngularDialTemplate {
    pub fn new(theme: Arc<dyn SliderTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_angular_dial_template() -> Arc<dyn SliderTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn SliderTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedAngularDialTemplate::new(default_slider_theme()))).clone()
}

impl SliderTemplate for ThemedAngularDialTemplate {
    fn resolve_look(&self, state: crate::theme::InteractionState) -> super::super::SliderLook {
        self.theme.resolve(state)
    }

    fn render(
        &self,
        model: &SliderRenderModel<'_>,
        handlers: SliderTemplateHandlers,
        primary_thumb_id: ThumbId,
        window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        let look = self.theme.resolve(model.state);
        let SliderInputStrategy::Angular { min_angle, max_angle } = model.strategy else {
            return div().id(model.id.clone());
        };

        let primary_thumb =
            model.thumbs.iter().find(|thumb| thumb.id == primary_thumb_id).or_else(|| model.thumbs.first());
        let thumb_position = primary_thumb.map(|thumb| thumb.position).unwrap_or(0.0);
        let display_percentage = display_position(thumb_position, model.reversed);
        let angle = angle_for_percentage(min_angle, max_angle, display_percentage);
        let dial_size = look.decoration.dial_size;
        let track_radius = look.decoration.track_radius;
        let center = dial_size * 0.5;
        let thumb_x = center + track_radius * angle.cos();
        let thumb_y = center + track_radius * angle.sin();
        let arc_thickness = look.track_height;
        let presentation = model.presentation;
        let track_background = look.track_background;
        let fill_background = look.fill_background;
        let (track_border, track_border_width) = (look.track_border, look.track_border_width);
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
                                    track_background,
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
