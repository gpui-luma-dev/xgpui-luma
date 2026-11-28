use crate::infra::focus_report::ReportsFocus as _;
use gpui::{
    Bounds, Context, DragMoveEvent, FocusOutEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, Window,
};

use super::super::input::{
    angle_for_percentage, angle_from_position, percentage_from_angle, percentage_from_position, unwrap_angle_near,
    SliderInputStrategy,
};
use super::super::model::ThumbId;
use super::super::thumbs::{insert_thumb, nearest_thumb, normalized_hit_radius};
use super::{SliderControl, SliderDrag, SliderEvent};
use crate::key_handling::{
    DecreaseValue, DecreaseValueLarge, IncreaseValue, IncreaseValueLarge, MoveToEnd, MoveToStart, RemoveValue,
};

impl SliderControl {
    pub(super) fn set_value_from_position(
        &mut self,
        position: Point<Pixels>,
        emit: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.model.enabled {
            return false;
        }

        let Some(bounds) = self.track_bounds else {
            return false;
        };

        let Some(percentage) = percentage_from_position(self.model.strategy, self.model.reversed, bounds, position)
        else {
            return false;
        };

        self.handle_linear_pointer_down(bounds, percentage, emit, cx)
    }

    pub(super) fn handle_linear_pointer_down(
        &mut self,
        bounds: Bounds<Pixels>,
        percentage: f32,
        emit: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        let hit_radius = self.normalized_hit_radius(bounds);
        let mut changed = false;

        let nearest_hit = nearest_thumb(&self.model.thumbs, percentage, hit_radius).map(|(thumb, _)| thumb.id);
        if let Some(thumb_id) = nearest_hit {
            let constrained = self.constrained_position_for_raw_position(percentage);
            changed |= self.select_thumb(thumb_id, true, cx);
            changed |= self.set_thumb_position_internal(thumb_id, constrained, emit, false, cx);
        } else if self.model.thumb_policy.allow_insert && insert_thumb(&mut self.model, percentage).is_some() {
            let thumb_id = self.model.thumbs.last().map(|thumb| thumb.id).expect("inserted thumb");
            changed = true;
            self.sync_transitions_with_thumbs();
            if let Some(transition) = self.transition_for_thumb_mut(thumb_id) {
                transition.snap_to(percentage);
            }
            self.select_thumb(thumb_id, true, cx);
            let value = self.thumb_value(thumb_id).unwrap_or(self.model.range.start);
            if emit {
                cx.emit(SliderEvent::ThumbAdded { thumb_id, value });
                cx.emit(SliderEvent::Change { thumb_id, value });
            }
            cx.notify();
        } else if let Some((thumb, _)) = nearest_thumb(&self.model.thumbs, percentage, f32::MAX) {
            let thumb_id = thumb.id;
            let constrained = self.constrained_position_for_raw_position(percentage);
            changed |= self.select_thumb(thumb_id, true, cx);
            changed |= self.set_thumb_position_internal(thumb_id, constrained, emit, false, cx);
        }

        changed
    }

    pub(super) fn normalized_hit_radius(&self, bounds: Bounds<Pixels>) -> f32 {
        normalized_hit_radius(bounds, self.thumb_size_px())
    }

    pub(super) fn thumb_size_px(&self) -> f32 {
        {
            use crate::style::PartRead;
            self.model.template.resolve_style(Default::default()).root().number("thumb_size")
        }
    }

    pub(super) fn handle_track_bounds(
        &mut self,
        bounds: &Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        self.track_bounds = Some(*bounds);
    }

    pub(super) fn handle_hover(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if self.interaction.handle_hover(*hovered) {
            cx.emit(SliderEvent::HoverChanged { hovered: self.interaction.hovered() });
            cx.notify();
        }
    }

    pub(super) fn handle_focus_in(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.model.enabled && self.emit_focus_changed(true, cx) {
            cx.notify();
        }
    }

    pub(super) fn handle_focus_out(&mut self, _: FocusOutEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.emit_focus_changed(false, cx) {
            cx.notify();
        }
    }

    pub(super) fn handle_thumb_mouse_down(
        &mut self,
        thumb_id: &ThumbId,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let was_pressed = self.interaction.is_pressed();
        let interaction_changed = self.interaction.handle_mouse_down(self.model.enabled, window, cx);
        let selection_changed = self.select_thumb(*thumb_id, true, cx);
        self.emit_drag_start_if_needed(was_pressed, Some(*thumb_id), cx);

        if interaction_changed || selection_changed {
            cx.stop_propagation();
            cx.notify();
        }
    }

    pub(super) fn handle_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        match self.model.strategy {
            SliderInputStrategy::Angular { min_angle, max_angle } => {
                let Some(bounds) = self.track_bounds else {
                    return;
                };
                if !self.angular_accepts_pointer(bounds, event.position) {
                    return;
                }
                let was_pressed = self.interaction.is_pressed();
                let interaction_changed = self.interaction.handle_mouse_down(self.model.enabled, window, cx);
                let thumb_id = self.active_thumb_id.or_else(|| self.model.thumbs.first().map(|thumb| thumb.id));
                let mut changed = false;
                if self.model.enabled
                    && let Some(thumb_id) = thumb_id
                {
                    self.select_thumb(thumb_id, false, cx);
                    if let Some(percentage) =
                        percentage_from_position(self.model.strategy, self.model.reversed, bounds, event.position)
                    {
                        let constrained = self.constrained_position_for_raw_position(percentage);
                        changed |= self.set_thumb_position_internal(thumb_id, constrained, true, false, cx);
                    }

                    let handle_position = self
                        .model
                        .thumbs
                        .iter()
                        .find(|thumb| thumb.id == thumb_id)
                        .map(|thumb| thumb.position)
                        .unwrap_or(0.0);
                    let handle_angle = angle_for_percentage(min_angle, max_angle, handle_position);
                    let pointer_angle = angle_from_position(bounds, event.position)
                        .map(|pointer_angle| unwrap_angle_near(handle_angle, pointer_angle));
                    self.angular_drag_angle_offset = Some(0.0);
                    self.angular_drag_pointer_angle = pointer_angle;
                }
                self.emit_drag_start_if_needed(was_pressed, thumb_id, cx);
                if interaction_changed || changed {
                    cx.stop_propagation();
                    cx.notify();
                }
            }
            SliderInputStrategy::Horizontal | SliderInputStrategy::Vertical => {
                let was_pressed = self.interaction.is_pressed();
                let interaction_changed = self.interaction.handle_mouse_down(self.model.enabled, window, cx);
                let changed = self.set_value_from_position(event.position, true, cx);
                self.emit_drag_start_if_needed(was_pressed, self.active_thumb_id, cx);
                if interaction_changed || changed {
                    cx.stop_propagation();
                    cx.notify();
                }
            }
        }
    }

    pub(super) fn angular_accepts_pointer(&self, bounds: Bounds<Pixels>, pointer: Point<Pixels>) -> bool {
        let Some(target) = self.model.radial_hit_target.as_ref() else {
            return true;
        };
        let thumb_position = self
            .active_thumb_id
            .and_then(|thumb_id| self.model.thumbs.iter().find(|thumb| thumb.id == thumb_id))
            .or_else(|| self.model.thumbs.first())
            .map(|thumb| thumb.position)
            .unwrap_or(0.0);
        target.accepts_pointer(bounds, pointer, thumb_position, self.model.reversed)
    }

    pub(super) fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.reset_drag_tracking();
        if self.interaction.handle_mouse_up() {
            if let Some(thumb_id) = self.active_thumb_id {
                let value = self.thumb_value(thumb_id).unwrap_or(self.model.range.start);
                cx.emit(SliderEvent::Release { thumb_id, value });
                cx.emit(SliderEvent::DragEnd { thumb_id, value });
            }
            cx.notify();
        }
    }

    pub(super) fn handle_mouse_move(&mut self, event: &MouseMoveEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.interaction.is_pressed() || !self.model.enabled {
            return;
        }

        let Some(bounds) = self.track_bounds else {
            return;
        };

        let Some(thumb_id) = self.active_or_primary_thumb_id() else {
            return;
        };
        let SliderInputStrategy::Angular { min_angle, max_angle } = self.model.strategy else {
            if let Some(percentage) =
                percentage_from_position(self.model.strategy, self.model.reversed, bounds, event.position)
            {
                let constrained = self.constrained_position_for_raw_position(percentage);
                if self.set_thumb_position_internal(thumb_id, constrained, true, false, cx) {
                    cx.stop_propagation();
                }
            }
            return;
        };
        if let Some(pointer_angle) = angle_from_position(bounds, event.position) {
            let previous = self.angular_drag_pointer_angle.unwrap_or(pointer_angle);
            let pointer_angle = unwrap_angle_near(previous, pointer_angle);
            self.angular_drag_pointer_angle = Some(pointer_angle);
            let offset = self.angular_drag_angle_offset.unwrap_or(0.0);
            if let Some(raw_percentage) = percentage_from_angle(pointer_angle - offset, min_angle, max_angle)
                .map(|raw| if self.model.reversed { 1.0 - raw } else { raw })
            {
                let constrained = self.constrained_position_for_raw_position(raw_percentage);
                if self.set_thumb_position_internal(thumb_id, constrained, true, false, cx) {
                    cx.stop_propagation();
                }
            }
        }
    }

    pub(super) fn handle_drag_move(
        &mut self,
        event: &DragMoveEvent<SliderDrag>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let drag = event.drag(cx);
        if drag.id != self.model.id {
            return;
        }

        let thumb_id = drag.thumb_id;
        self.select_thumb(thumb_id, false, cx);

        match self.model.strategy {
            SliderInputStrategy::Angular { min_angle, max_angle } => {
                let bounds = self.track_bounds.unwrap_or(event.bounds);
                if let (Some(pointer_angle), Some(offset), Some(previous_pointer_angle)) = (
                    angle_from_position(bounds, event.event.position),
                    self.angular_drag_angle_offset,
                    self.angular_drag_pointer_angle,
                ) {
                    let pointer_angle = unwrap_angle_near(previous_pointer_angle, pointer_angle);
                    self.angular_drag_pointer_angle = Some(pointer_angle);
                    if let Some(raw_percentage) = percentage_from_angle(pointer_angle - offset, min_angle, max_angle)
                        .map(|raw| if self.model.reversed { 1.0 - raw } else { raw })
                    {
                        let constrained = self.constrained_position_for_raw_position(raw_percentage);
                        self.set_thumb_position_internal(thumb_id, constrained, true, false, cx);
                    }
                }
            }
            SliderInputStrategy::Horizontal | SliderInputStrategy::Vertical => {
                if self.track_bounds.is_none() {
                    self.track_bounds = Some(event.bounds);
                }

                let bounds = self.track_bounds.unwrap_or(event.bounds);

                if let Some(percentage) =
                    percentage_from_position(self.model.strategy, self.model.reversed, bounds, event.event.position)
                {
                    let constrained = self.constrained_position_for_raw_position(percentage);
                    self.set_thumb_position_internal(thumb_id, constrained, true, false, cx);
                }
            }
        }
    }
    pub(super) fn reset_drag_tracking(&mut self) {
        self.angular_drag_angle_offset = None;
        self.angular_drag_pointer_angle = None;
    }
    pub(super) fn handle_decrease_value(&mut self, _: &DecreaseValue, _window: &mut Window, cx: &mut Context<Self>) {
        self.adjust_value(-self.model.step, cx);
    }

    pub(super) fn handle_increase_value(&mut self, _: &IncreaseValue, _window: &mut Window, cx: &mut Context<Self>) {
        self.adjust_value(self.model.step, cx);
    }

    pub(super) fn handle_decrease_value_large(
        &mut self,
        _: &DecreaseValueLarge,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.adjust_value(-(self.model.step * 10.0), cx);
    }

    pub(super) fn handle_increase_value_large(
        &mut self,
        _: &IncreaseValueLarge,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.adjust_value(self.model.step * 10.0, cx);
    }

    pub(super) fn handle_move_to_start(&mut self, _: &MoveToStart, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_to_value(self.model.range.start, cx);
    }

    pub(super) fn handle_move_to_end(&mut self, _: &MoveToEnd, _window: &mut Window, cx: &mut Context<Self>) {
        self.move_to_value(self.model.range.end, cx);
    }

    pub(super) fn handle_remove_value(&mut self, _: &RemoveValue, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.model.enabled || !self.model.thumb_policy.allow_remove {
            return;
        }

        if let Some(thumb_id) = self.active_thumb_id {
            self.remove_thumb(thumb_id, cx);
        }
    }

    pub(super) fn emit_drag_start_if_needed(
        &mut self,
        was_pressed: bool,
        thumb_id: Option<ThumbId>,
        cx: &mut Context<Self>,
    ) {
        if was_pressed || !self.interaction.is_pressed() {
            return;
        }

        if let Some(thumb_id) = thumb_id {
            cx.emit(SliderEvent::DragStart { thumb_id });
        }
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Bounds, Pixels, point, px, size};

    use crate::controls::slider::input::{SliderInputStrategy, percentage_from_position};
    use crate::controls::slider::layout;

    fn test_bounds() -> Bounds<Pixels> {
        Bounds { origin: point(px(10.0), px(20.0)), size: size(px(200.0), px(120.0)) }
    }

    #[test]
    fn horizontal_percentage_tracks_x_position() {
        let percentage =
            percentage_from_position(SliderInputStrategy::Horizontal, false, test_bounds(), point(px(60.0), px(30.0)));

        assert_eq!(percentage, Some(0.25));
    }

    #[test]
    fn horizontal_reversed_pointer_mirrors_x_position() {
        let percentage =
            percentage_from_position(SliderInputStrategy::Horizontal, true, test_bounds(), point(px(60.0), px(30.0)));

        assert_eq!(percentage, Some(0.75));
    }

    #[test]
    fn vertical_percentage_inverts_y_position() {
        let percentage =
            percentage_from_position(SliderInputStrategy::Vertical, false, test_bounds(), point(px(20.0), px(50.0)));

        assert_eq!(percentage, Some(0.75));
    }

    #[test]
    fn vertical_reversed_pointer_mirrors_y_position() {
        let percentage =
            percentage_from_position(SliderInputStrategy::Vertical, true, test_bounds(), point(px(20.0), px(50.0)));

        assert_eq!(percentage, Some(0.25));
    }

    #[test]
    fn reversed_pointer_round_trips_display_position() {
        let raw = layout::position_from_pointer(0.2, true);
        assert!((layout::display_position(raw, true) - 0.2).abs() <= f32::EPSILON);
    }

    #[cfg(feature = "test-support")]
    #[test]
    fn vertical_track_press_continues_dragging_outside_track() {
        use super::SliderControl;
        use crate::controls::slider::SliderBuilder;
        use gpui::{MouseButton, MouseDownEvent, MouseMoveEvent, TestAppContext};

        for reversed in [false, true] {
            let mut app = TestAppContext::single();
            let (slider, cx) = app.add_window_view(|window, cx| {
                window.activate_window();
                SliderControl::from_builder(
                    SliderBuilder::new("vertical").vertical().reversed(reversed).range(0.0..100.0).value(10.0),
                    cx,
                )
            });
            let bounds = cx.update(|_, app| slider.read(app).track_bounds.unwrap());
            let at = |fraction| point(bounds.center().x, bounds.bottom() - bounds.size.height * fraction);
            cx.simulate_event(MouseDownEvent {
                button: MouseButton::Left,
                position: at(0.4),
                click_count: 1,
                ..Default::default()
            });
            let mut outside = at(0.7);
            outside.x += px(80.0);
            cx.simulate_event(MouseMoveEvent {
                position: outside,
                pressed_button: Some(MouseButton::Left),
                ..Default::default()
            });
            cx.update(|_, app| {
                let expected = if reversed { 30.0 } else { 70.0 };
                assert!((slider.read(app).value() - expected).abs() < 0.01);
            });
        }
    }
}
