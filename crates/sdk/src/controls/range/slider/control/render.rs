use std::sync::Arc;

use gpui::{App, Context, IntoElement, MouseDownEvent, MouseMoveEvent, Render, Window, canvas, div, prelude::*};

use super::super::model::{build_render_segments_at, SliderRenderModel, ThumbId, TrackPresentation};
use super::super::SliderTemplateHandlers;
use super::SliderControl;
use crate::key_handling::ControlKeyProfile;

impl SliderControl {
    pub(super) fn render_model<'a>(&'a self, window: &Window) -> SliderRenderModel<'a> {
        let presentation = if self.model.thumb_policy.is_multi_thumb() {
            TrackPresentation::Domain
        } else {
            self.model.presentation
        };
        let display_thumb_position = self.display_thumbs.first().map(|thumb| thumb.position).unwrap_or(0.0);

        SliderRenderModel {
            id: &self.model.id,
            strategy: self.model.strategy,
            orientation: self.model.strategy.orientation(),
            presentation,
            range: self.model.range,
            step: self.model.step,
            thumbs: &self.display_thumbs,
            track_segments: build_render_segments_at(&self.model, display_thumb_position),
            reversed: self.model.reversed,
            wrapping: self.model.wrapping,
            enabled: self.model.enabled,
            corner_radius: self.model.corner_radius,
            thumb_radius: self.model.thumb_radius,
            thumb_policy: self.model.thumb_policy,
            active_thumb_id: self.active_thumb_id,
            state: self.interaction.render_state(self.model.enabled, window),
            domain_track: self.model.domain_track.clone(),
        }
    }

    pub(super) fn template_handlers(&self, cx: &mut Context<Self>) -> SliderTemplateHandlers {
        let entity = cx.entity().clone();
        SliderTemplateHandlers {
            track_bounds: Box::new(cx.listener(Self::handle_track_bounds)),
            hover: Box::new(cx.listener(Self::handle_hover)),
            mouse_down: Box::new(cx.listener(Self::handle_mouse_down)),
            mouse_move: Box::new(cx.listener(Self::handle_mouse_move)),
            mouse_up: Box::new(cx.listener(Self::handle_mouse_up)),
            mouse_up_out: Box::new(cx.listener(Self::handle_mouse_up)),
            drag_move: Arc::new(cx.listener(Self::handle_drag_move)),
            thumb_mouse_down: Arc::new(
                move |thumb_id: &ThumbId, event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                    entity.update(cx, |this, cx| {
                        this.handle_thumb_mouse_down(thumb_id, event, window, cx);
                    });
                },
            ),
        }
    }
}

impl Render for SliderControl {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.interaction.focus_handle().clone();
        self.focus_subscriptions
            .ensure(&focus_handle, window, cx, Self::handle_focus_in, Self::handle_focus_out);

        self.sync_transitions_with_thumbs();
        let mut was_animating = false;
        let mut is_animating = false;
        for transition in &mut self.thumb_transitions {
            was_animating |= transition.is_animating();
            is_animating |= transition.sync();
        }
        self.refresh_display_thumbs();
        for transition in &self.thumb_transitions {
            transition.schedule_frame(window, cx);
        }
        if was_animating || is_animating {
            cx.notify();
        }

        let model = self.render_model(window);
        let handlers = self.template_handlers(cx);
        let active_thumb_id = self
            .active_thumb_id
            .or_else(|| self.model.thumbs.first().map(|thumb| thumb.id))
            .expect("slider always has a thumb");

        div()
            .when(self.interaction.is_pressed() && !self.model.strategy.is_angular(), |root| {
                let entity = cx.entity();
                // Track presses do not start GPUI's thumb drag. Keep following the pointer outside the track.
                root.child(
                    canvas(
                        |_, _, _| (),
                        move |_, _, window, _| {
                            window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                                if phase.bubble() {
                                    entity.update(cx, |slider, cx| slider.handle_mouse_move(event, window, cx));
                                }
                            });
                        },
                    )
                    .absolute(),
                )
            })
            .child(
                self.attachments
                    .render(self.model.template.render(&model, handlers, active_thumb_id, window, cx), model.state, cx)
                    .track_focus(self.interaction.focus_handle())
                    .key_context(ControlKeyProfile::RangeValue.context())
                    .when(self.model.keyboard_position_step.is_some(), |root| {
                        root.on_key_down(cx.listener(Self::handle_position_step_key_down))
                    })
                    .on_action(cx.listener(Self::handle_decrease_value))
                    .on_action(cx.listener(Self::handle_increase_value))
                    .on_action(cx.listener(Self::handle_decrease_value_large))
                    .on_action(cx.listener(Self::handle_increase_value_large))
                    .on_action(cx.listener(Self::handle_move_to_start))
                    .on_action(cx.listener(Self::handle_move_to_end))
                    .on_action(cx.listener(Self::handle_remove_value)),
            )
            .into_any_element()
    }
}
