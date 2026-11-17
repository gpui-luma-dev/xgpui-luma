use gpui::{Bounds, Pixels};

use super::constraints::clamp_and_snap_value;
use super::input::wrap_and_snap;
use super::model::{SliderModel, SliderThumbPolicy, SliderThumbRole, SliderThumbValue, ThumbId, value_for_position};
use crate::infra::value::ControlRange;

const DEFAULT_HIT_RADIUS: f32 = 0.04;

pub fn normalized_hit_radius(bounds: Bounds<Pixels>, thumb_size_px: f32) -> f32 {
    let width = bounds.size.width.as_f32().max(bounds.size.height.as_f32());
    if width <= f32::EPSILON {
        return DEFAULT_HIT_RADIUS;
    }

    ((thumb_size_px * 0.55) / width).clamp(0.015, 0.15)
}

pub fn nearest_thumb(
    thumbs: &[SliderThumbValue],
    percentage: f32,
    max_distance: f32,
) -> Option<(&SliderThumbValue, f32)> {
    thumbs
        .iter()
        .map(|thumb| {
            let distance = (thumb.position - percentage).abs();
            (thumb, distance)
        })
        .filter(|(_, distance)| *distance <= max_distance)
        .min_by(|left, right| left.1.total_cmp(&right.1))
}

pub fn can_insert_at(thumbs: &[SliderThumbValue], percentage: f32, policy: SliderThumbPolicy) -> bool {
    if !policy.allow_insert || thumbs.len() >= policy.max_count {
        return false;
    }

    if policy.min_distance <= f32::EPSILON {
        return true;
    }

    thumbs.iter().all(|thumb| (thumb.position - percentage).abs() >= policy.min_distance - f32::EPSILON)
}

pub fn constrain_thumb_position(
    position: f32,
    thumb_id: ThumbId,
    thumbs: &[SliderThumbValue],
    policy: SliderThumbPolicy,
) -> f32 {
    constrain_position(position, Some(thumb_id), thumbs, policy)
}

pub fn constrain_insert_position(position: f32, thumbs: &[SliderThumbValue], policy: SliderThumbPolicy) -> f32 {
    constrain_position(position, None, thumbs, policy)
}

fn constrain_position(
    position: f32,
    thumb_id: Option<ThumbId>,
    thumbs: &[SliderThumbValue],
    policy: SliderThumbPolicy,
) -> f32 {
    let mut position = position.clamp(0.0, 1.0);
    if policy.allow_overlap || policy.min_distance <= f32::EPSILON {
        return position;
    }

    for _ in 0..thumbs.len() {
        let mut adjusted = false;
        for thumb in thumbs {
            if thumb_id == Some(thumb.id) {
                continue;
            }

            let delta = position - thumb.position;
            let distance = delta.abs();
            if distance < policy.min_distance - f32::EPSILON {
                position = if delta >= 0.0 {
                    (thumb.position + policy.min_distance).min(1.0)
                } else {
                    (thumb.position - policy.min_distance).max(0.0)
                };
                adjusted = true;
            }
        }

        if !adjusted {
            break;
        }
    }

    position.clamp(0.0, 1.0)
}

pub fn constrain_thumb_value(value: f32, model: &SliderModel) -> f32 {
    if model.wrapping
        && model.allowed_intervals.is_empty()
        && model.value_map.as_ref().is_none_or(|map| map.wraps_value())
    {
        wrap_and_snap(value, model.range, model.step)
    } else {
        clamp_and_snap_value(value, &model.allowed_intervals, model.range, model.step)
    }
}

pub fn thumb_value(thumb: &SliderThumbValue, range: ControlRange, model: &SliderModel) -> f32 {
    let _ = range;
    constrain_thumb_value(value_for_position(model, thumb.position), model)
}

pub fn insert_thumb(model: &mut SliderModel, percentage: f32) -> Option<ThumbId> {
    let policy = model.thumb_policy;
    if !can_insert_at(&model.thumbs, percentage, policy) {
        return None;
    }

    let position = constrain_insert_position(percentage, &model.thumbs, policy);
    let id = ThumbId::next();
    model.thumbs.push(SliderThumbValue { id, position, preview: None, role: SliderThumbRole::Value });
    Some(id)
}

pub fn remove_thumb(model: &mut SliderModel, thumb_id: ThumbId) -> bool {
    let policy = model.thumb_policy;
    if !policy.allow_remove || model.thumbs.len() <= policy.min_count {
        return false;
    }

    let Some(index) = model.thumbs.iter().position(|thumb| thumb.id == thumb_id) else {
        return false;
    };

    model.thumbs.remove(index);
    true
}

pub fn set_thumb_position(model: &mut SliderModel, thumb_id: ThumbId, percentage: f32) -> bool {
    let policy = model.thumb_policy;
    let position = constrain_thumb_position(percentage, thumb_id, &model.thumbs, policy);
    let Some(thumb) = model.thumbs.iter_mut().find(|thumb| thumb.id == thumb_id) else {
        return false;
    };

    if (thumb.position - position).abs() <= f32::EPSILON {
        return false;
    }

    thumb.position = position;
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::slider::input::SliderInputStrategy;
    use crate::controls::slider::model::SliderModel;
    use crate::controls::slider::template::default_slider_template;
    use crate::infra::value::ControlRange;
    use gpui::SharedString;

    fn test_model(thumbs: Vec<SliderThumbValue>, policy: SliderThumbPolicy) -> SliderModel {
        SliderModel {
            motion: Default::default(),
            id: SharedString::from("test"),
            strategy: SliderInputStrategy::Horizontal,
            presentation: super::super::model::TrackPresentation::Fill,
            size: crate::theme::ControlSize::Md,
            thumb_size: None,
            range: ControlRange::new(0.0, 100.0),
            step: 1.0,
            keyboard_position_step: None,
            thumbs,
            allowed_intervals: Vec::new(),
            track_intervals: None,
            reversed: false,
            reversed_explicit: false,
            wrapping: false,
            enabled: true,
            corner_radius: None,
            thumb_radius: None,
            template: default_slider_template(),
            thumb_policy: policy,
            domain_track: None,
            value_map: None,
            radial_hit_target: None,
        }
    }

    fn thumb(position: f32) -> SliderThumbValue {
        SliderThumbValue { id: ThumbId::next(), position, preview: None, role: SliderThumbRole::Value }
    }

    #[test]
    fn nearest_thumb_respects_hit_radius() {
        let thumbs = vec![thumb(0.2), thumb(0.8)];
        assert_eq!(nearest_thumb(&thumbs, 0.22, 0.05).map(|(thumb, _)| thumb.position), Some(0.2));
        assert!(nearest_thumb(&thumbs, 0.5, 0.05).is_none());
    }

    #[test]
    fn insert_respects_min_distance() {
        let policy = SliderThumbPolicy {
            min_count: 1,
            max_count: 4,
            min_distance: 0.1,
            allow_insert: true,
            ..SliderThumbPolicy::multi_stop()
        };
        let mut model = test_model(vec![thumb(0.5)], policy);
        assert!(insert_thumb(&mut model, 0.52).is_none());
        assert!(insert_thumb(&mut model, 0.7).is_some());
        assert_eq!(model.thumbs.len(), 2);
    }

    #[test]
    fn remove_respects_min_count() {
        let policy = SliderThumbPolicy { min_count: 2, ..SliderThumbPolicy::multi_stop() };
        let first = thumb(0.2);
        let first_id = first.id;
        let mut model = test_model(vec![first, thumb(0.8)], policy);
        assert!(!remove_thumb(&mut model, first_id));
        assert_eq!(model.thumbs.len(), 2);
    }

    #[test]
    fn constrain_thumb_position_enforces_min_distance() {
        let policy = SliderThumbPolicy { min_distance: 0.1, allow_overlap: false, ..SliderThumbPolicy::multi_stop() };
        let left = thumb(0.2);
        let left_id = left.id;
        let thumbs = vec![left, thumb(0.8)];
        let position = constrain_thumb_position(0.75, left_id, &thumbs, policy);
        assert!((position - 0.7).abs() <= f32::EPSILON);
    }

    #[test]
    fn crossover_preserves_identity_in_vec_order() {
        let policy = SliderThumbPolicy::multi_stop();
        let first = thumb(0.2);
        let first_id = first.id;
        let mut model = test_model(vec![first, thumb(0.8)], policy);
        set_thumb_position(&mut model, first_id, 0.9);
        assert_eq!(model.thumbs[0].id, first_id);
        assert!(model.thumbs[0].position > model.thumbs[1].position);
    }
}
