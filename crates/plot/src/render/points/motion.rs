use xgpui_luma::motion::{Transition, Easing};
use std::time::Duration;
use gpui::{Bounds, Context, Pixels, Point, Window, point, px};
use xgpui_luma::motion::VisualTransition;

/// Animate only the anchor. Hit regions, observations and events never lag it.
pub(crate) struct AnchorMotion {
    from: Point<Pixels>,
    target: Option<Point<Pixels>>,
    bounds: Option<Bounds<Pixels>>,
    duration: Duration,
    transition: VisualTransition,
}

impl Default for AnchorMotion {
    fn default() -> Self {
        Self {
            from: Point::default(),
            target: None,
            bounds: None,
            duration: Duration::ZERO,
            transition: VisualTransition::with_transition(1.0, Transition::timed(Duration::ZERO, Easing::OutCubic)),
        }
    }
}

impl AnchorMotion {
    pub(crate) fn clear(&mut self) {
        *self = Self::default();
    }

    fn position(&self) -> Point<Pixels> {
        let target = self.target.unwrap_or(self.from);
        point(
            self.transition.interpolate_pixels(self.from.x, target.x),
            self.transition.interpolate_pixels(self.from.y, target.y),
        )
    }

    pub(crate) fn update(
        &mut self,
        target: Point<Pixels>,
        bounds: Bounds<Pixels>,
        duration: Duration,
    ) -> Point<Pixels> {
        self.transition.sync();
        if duration.is_zero() {
            self.from = target;
            self.target = Some(target);
            self.transition.snap_to(1.0);
        } else if self.target.is_none() || self.bounds != Some(bounds) {
            // Enter near the final anchor; never fly in from a stale scrolled plot.
            self.from = target + point(px(0.0), px(8.0));
            self.target = Some(target);
            self.transition = VisualTransition::with_transition(0.0, Transition::timed(duration, Easing::OutCubic));
            self.transition.set_target(1.0);
        } else if self.target != Some(target) || self.duration != duration {
            self.from = self.position();
            self.target = Some(target);
            self.transition = VisualTransition::with_transition(0.0, Transition::timed(duration, Easing::OutCubic));
            self.transition.set_target(1.0);
        }
        self.bounds = Some(bounds);
        self.duration = duration;
        self.position()
    }

    pub(crate) fn schedule_frame<T: 'static>(&self, window: &mut Window, cx: &mut Context<T>) {
        self.transition.schedule_frame(window, cx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::size;

    #[test]
    fn retargets_from_displayed_position_and_can_snap_or_reset() {
        let bounds = Bounds::new(Point::default(), size(px(300.0), px(200.0)));
        let a = point(px(60.0), px(80.0));
        let b = point(px(240.0), px(120.0));
        let duration = Duration::from_millis(250);
        let mut motion = AnchorMotion::default();
        assert_eq!(motion.update(a, bounds, Duration::ZERO), a);
        assert_eq!(motion.update(b, bounds, duration), a);
        motion.transition.snap_to(0.5);
        let midpoint = point(px(150.0), px(100.0));
        assert_eq!(motion.position(), midpoint);
        // Reversing midway starts from the displayed position, not the old target.
        assert_eq!(motion.update(a, bounds, duration), midpoint);
        assert_eq!(motion.update(b, bounds, Duration::ZERO), b);
        assert!(!motion.transition.is_animating());
        motion.clear();
        assert_eq!(motion.update(a, bounds, duration), a + point(px(0.0), px(8.0)));
        motion.transition.snap_to(1.0);
        assert_eq!(motion.update(a, bounds, duration), a);
        let shifted = Bounds::new(point(px(0.0), px(500.0)), bounds.size);
        let c = point(px(60.0), px(580.0));
        assert_eq!(motion.update(c, shifted, duration), c + point(px(0.0), px(8.0)));
    }
}
