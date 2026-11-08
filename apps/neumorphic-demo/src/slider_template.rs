use std::sync::{Arc, OnceLock};

use gpui::{App, Div, MouseButton, Stateful, Window, canvas, div, hsla, px, prelude::*};
use xgpui_luma::controls::slider::{SliderDrag, SliderRenderModel, SliderTemplate, SliderTemplateHandlers, ThumbId};

pub fn neumorphic_slider_template() -> Arc<dyn SliderTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn SliderTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(NeumorphicSliderTemplate)).clone()
}

struct NeumorphicSliderTemplate;

impl SliderTemplate for NeumorphicSliderTemplate {
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
        let percentage = if model.reversed {
            1.0 - primary.position
        } else {
            primary.position
        }
        .clamp(0.0, 1.0);
        let root_width = 56.0;
        let rail_height = 154.0;
        let rail_width = 12.0;
        let thumb_width = 34.0;
        let thumb_height = 20.0;
        let rail_top = 4.0;
        let lower_padding = 8.0;
        let root_height = rail_top + rail_height + lower_padding;
        let rail_left = (root_width - rail_width) * 0.5;
        let thumb_left = (root_width - thumb_width) * 0.5;
        let thumb_travel = rail_height - thumb_height;
        let thumb_top = rail_top + ((1.0 - percentage) * thumb_travel);

        let active_start = thumb_top + (thumb_height * 0.5);
        let active_height = (rail_top + rail_height - active_start).max(0.0_f32);

        let track_bg = if model.enabled {
            hsla(220.0 / 360.0, 0.12, 0.80, 0.96)
        } else {
            hsla(220.0 / 360.0, 0.08, 0.84, 0.82)
        };
        let track_fg = if model.enabled {
            hsla(220.0 / 360.0, 0.10, 0.26, 0.82)
        } else {
            hsla(220.0 / 360.0, 0.08, 0.55, 0.58)
        };

        let thumb_shadow = if model.enabled {
            vec![
                gpui::BoxShadow {
                    color: hsla(220.0 / 360.0, 0.18, 0.14, 0.18),
                    offset: gpui::point(px(0.0), px(1.0)),
                    blur_radius: px(2.0),
                    spread_radius: px(0.0),
                    inset: false,
                },
                gpui::BoxShadow {
                    color: hsla(220.0 / 360.0, 0.24, 0.08, 0.24),
                    offset: gpui::point(px(0.0), px(3.0)),
                    blur_radius: px(5.0),
                    spread_radius: px(0.0),
                    inset: false,
                },
            ]
        } else {
            Vec::new()
        };

        let rail = div()
            .absolute()
            .top(px(rail_top))
            .left(px(rail_left))
            .w(px(rail_width))
            .h(px(rail_height))
            .bg(track_bg)
            .rounded(px(rail_width * 0.5))
            .child(
                div()
                    .absolute()
                    .left(px((rail_width - 3.0) * 0.5))
                    .bottom(px(0.0))
                    .w(px(3.0))
                    .h(px(active_height))
                    .bg(track_fg)
                    .rounded(px(1.5)),
            );

        let thumb = div()
            .absolute()
            .top(px(thumb_top))
            .left(px(thumb_left))
            .w(px(thumb_width))
            .h(px(thumb_height))
            .bg(hsla(220.0 / 360.0, 0.08, 0.94, 0.98))
            .border_1()
            .border_color(hsla(220.0 / 360.0, 0.08, 0.74, 0.52))
            .rounded(px(thumb_height * 0.5))
            .shadow(thumb_shadow)
            .overflow_hidden()
            .child(
                div()
                    .absolute()
                    .top(px(1.0))
                    .left(px(1.0))
                    .right(px(1.0))
                    .h(px(thumb_height * 0.34))
                    .bg(hsla(0.0, 0.0, 1.0, 0.52))
                    .rounded_t(px((thumb_height * 0.5) - 1.0)),
            )
            .child(
                div()
                    .absolute()
                    .left(px(0.0))
                    .right(px(0.0))
                    .bottom(px(1.0))
                    .h(px(thumb_height * 0.56))
                    .bg(hsla(220.0 / 360.0, 0.16, 0.78, 0.42))
                    .rounded_b(px(thumb_height * 0.5)),
            )
            .child(
                div()
                    .absolute()
                    .top(px((thumb_height - 8.0) * 0.5))
                    .left(px((thumb_width - 22.0) * 0.5))
                    .w(px(22.0))
                    .h(px(8.0))
                    .bg(hsla(220.0 / 360.0, 0.08, 0.22, 0.90))
                    .rounded(px(4.0)),
            );

        let thumb_shadow_pool = div()
            .absolute()
            .top(px(thumb_top + 7.0))
            .left(px(thumb_left + 5.0))
            .w(px(thumb_width - 8.0))
            .h(px(thumb_height - 6.0))
            .bg(hsla(220.0 / 360.0, 0.24, 0.14, if model.enabled { 0.16 } else { 0.0 }))
            .rounded(px((thumb_height - 6.0) * 0.5))
            .shadow(if model.enabled {
                vec![gpui::BoxShadow {
                    color: hsla(220.0 / 360.0, 0.30, 0.08, 0.28),
                    offset: gpui::point(px(1.0), px(6.0)),
                    blur_radius: px(9.0),
                    spread_radius: px(0.0),
                    inset: false,
                }]
            } else {
                Vec::new()
            });

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .w(px(root_width))
            .h(px(root_height))
            .on_hover(hover)
            .on_mouse_down(MouseButton::Left, mouse_down)
            .on_mouse_up(MouseButton::Left, mouse_up)
            .on_mouse_up_out(MouseButton::Left, mouse_up_out)
            .on_drag(SliderDrag::new(model.id.clone(), primary_thumb_id), |drag, _, _, cx| {
                cx.stop_propagation();
                cx.new(|_| drag.clone())
            })
            .on_drag_move({
                let drag_move = drag_move.clone();
                move |event, window, cx| drag_move(event, window, cx)
            })
            .child(rail)
            .child(thumb_shadow_pool)
            .child(thumb)
            .child(
                canvas(move |bounds, window, cx| track_bounds(&bounds, window, cx), |_, _, _, _| {})
                    .absolute()
                    .size_full(),
            );

        if model.enabled {
            root = root.cursor_pointer();
        } else {
            root = root.opacity(0.56);
        }

        root
    }
}
