use std::sync::{Arc, Mutex};

use gpui::{Bounds, Hsla, Pixels};

use crate::motion::{Transition, VisualTransition};
use crate::infra::lock;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TabsIndicatorRect {
    pub left: f32,
    pub width: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TabsIndicatorPaint {
    pub left: f32,
    pub width: f32,
    pub height: f32,
    pub color: Hsla,
}

#[derive(Debug)]
struct TabsIndicatorMotionInner {
    left: VisualTransition,
    width: VisualTransition,
    to: TabsIndicatorRect,
    list_bounds: Option<Bounds<Pixels>>,
    pending_item_bounds: Option<Bounds<Pixels>>,
    padding_x: f32,
    height: f32,
    color: Option<Hsla>,
    motion_enabled: bool,
    has_geometry: bool,
}

impl TabsIndicatorMotionInner {
    fn new(motion_enabled: bool, definition: Transition) -> Self {
        Self {
            left: VisualTransition::with_transition(0.0, definition.clone()),
            width: VisualTransition::with_transition(0.0, definition.clone()),
            to: TabsIndicatorRect::default(),
            list_bounds: None,
            pending_item_bounds: None,
            padding_x: 0.0,
            height: crate::theme::stylesheet::CommonStylesheet::sdk_default()
                .tabs
                .geometry
                .indicator_height
                .unwrap_or_default(),
            color: None,
            motion_enabled,
            has_geometry: false,
        }
    }

    fn set_motion_enabled(&mut self, motion_enabled: bool) {
        if self.motion_enabled == motion_enabled {
            return;
        }
        self.motion_enabled = motion_enabled;
        if !motion_enabled {
            self.left.snap_to(self.to.left);
            self.width.snap_to(self.to.width);
        }
    }

    fn set_metrics(&mut self, padding_x: f32, height: f32, color: Option<Hsla>) {
        let inset_changed = self.padding_x != padding_x;
        self.padding_x = padding_x;
        self.height = height;
        self.color = color;
        if inset_changed && let Some(bounds) = self.pending_item_bounds {
            self.apply_item_bounds(bounds, false);
        }
    }

    fn set_list_bounds(&mut self, bounds: Bounds<Pixels>) {
        let changed = self.list_bounds != Some(bounds);
        self.list_bounds = Some(bounds);
        if changed && let Some(item_bounds) = self.pending_item_bounds {
            self.apply_item_bounds(item_bounds, false);
        }
    }

    fn clear_geometry(&mut self) {
        self.list_bounds = None;
        self.pending_item_bounds = None;
        self.has_geometry = false;
        self.to = TabsIndicatorRect::default();
        self.left.snap_to(self.to.left);
        self.width.snap_to(self.to.width);
    }

    fn apply_item_bounds(&mut self, item_bounds: Bounds<Pixels>, animate: bool) {
        self.pending_item_bounds = Some(item_bounds);
        let Some(list_bounds) = self.list_bounds else {
            return;
        };
        let next = indicator_rect_from_bounds(item_bounds, list_bounds, self.padding_x);
        self.retarget(next, animate);
    }

    fn retarget(&mut self, next: TabsIndicatorRect, animate: bool) {
        if !self.has_geometry || !animate || !self.motion_enabled {
            self.to = next;
            self.left.snap_to(self.to.left);
            self.width.snap_to(self.to.width);
            self.has_geometry = true;
            return;
        }

        if (self.to.left - next.left).abs() <= f32::EPSILON && (self.to.width - next.width).abs() <= f32::EPSILON {
            return;
        }

        self.to = next;
        self.left.set_target(next.left);
        self.width.set_target(next.width);
        self.has_geometry = true;
    }

    fn display_rect(&self) -> TabsIndicatorRect {
        TabsIndicatorRect { left: self.left.progress(), width: self.width.progress() }
    }

    fn sync(&mut self) -> bool {
        let left = self.left.sync();
        self.width.sync() || left
    }

    fn is_animating(&self) -> bool {
        self.left.is_animating() || self.width.is_animating()
    }

    fn paint(&self) -> Option<TabsIndicatorPaint> {
        let color = self.color?;
        if !self.has_geometry {
            return None;
        }
        let rect = self.display_rect();
        if rect.width <= f32::EPSILON {
            return None;
        }
        Some(TabsIndicatorPaint { left: rect.left, width: rect.width, height: self.height, color })
    }
}

/// Shared indicator motion state between [`super::Tabs`] and the list template.
#[derive(Clone, Debug)]
pub struct TabsIndicatorMotion {
    inner: Arc<Mutex<TabsIndicatorMotionInner>>,
}

impl TabsIndicatorMotion {
    pub fn new(motion_enabled: bool) -> Self {
        Self::with_transition(motion_enabled, Transition::default())
    }

    pub fn with_transition(motion_enabled: bool, definition: Transition) -> Self {
        Self { inner: Arc::new(Mutex::new(TabsIndicatorMotionInner::new(motion_enabled, definition))) }
    }

    fn inner(&self) -> std::sync::MutexGuard<'_, TabsIndicatorMotionInner> {
        lock::mutex(&self.inner)
    }

    pub fn set_motion_enabled(&self, motion_enabled: bool) {
        self.inner().set_motion_enabled(motion_enabled);
    }

    pub fn set_metrics(&self, padding_x: f32, height: f32, color: Option<Hsla>) {
        self.inner().set_metrics(padding_x, height, color);
    }

    pub fn set_list_bounds(&self, bounds: Bounds<Pixels>) {
        self.inner().set_list_bounds(bounds);
    }

    pub(crate) fn list_bounds(&self) -> Option<Bounds<Pixels>> {
        self.inner().list_bounds
    }

    pub fn clear_geometry(&self) {
        self.inner().clear_geometry();
    }

    pub fn apply_item_bounds(&self, item_bounds: Bounds<Pixels>, animate: bool) {
        self.inner().apply_item_bounds(item_bounds, animate);
    }

    pub fn sync(&self) -> bool {
        self.inner().sync()
    }

    pub fn is_animating(&self) -> bool {
        self.inner().is_animating()
    }

    pub fn schedule_frame<T>(&self, window: &mut gpui::Window, cx: &mut gpui::Context<T>)
    where
        T: 'static,
    {
        let animating = self.is_animating();
        if !animating {
            return;
        }
        cx.on_next_frame(window, |_, _, cx| {
            cx.notify();
        });
    }

    pub fn paint(&self) -> Option<TabsIndicatorPaint> {
        self.inner().paint()
    }

    pub fn display_rect_for_test(&self) -> TabsIndicatorRect {
        self.inner().display_rect()
    }
}

pub fn indicator_rect_from_bounds(
    item_bounds: Bounds<Pixels>,
    list_bounds: Bounds<Pixels>,
    padding_x: f32,
) -> TabsIndicatorRect {
    TabsIndicatorRect {
        left: (item_bounds.left() - list_bounds.left()).as_f32() + padding_x,
        width: (item_bounds.size.width.as_f32() - padding_x * 2.0).max(0.0),
    }
}

#[cfg(test)]
mod tests {
    use gpui::{point, px, size, Bounds};

    use super::*;

    #[cfg(feature = "test-support")]
    #[test]
    fn spring_geometry_retarget_preserves_velocity_and_disable_settles() {
        use std::time::Duration;
        let mut motion = TabsIndicatorMotionInner::new(true, Transition::spring(200.0, 18.0));
        motion.retarget(TabsIndicatorRect { left: 0.0, width: 40.0 }, false);
        motion.left.elapse_for_test(Duration::ZERO);
        motion.width.elapse_for_test(Duration::ZERO);
        motion.retarget(TabsIndicatorRect { left: 200.0, width: 80.0 }, true);
        motion.left.elapse_for_test(Duration::from_millis(100));
        motion.width.elapse_for_test(Duration::from_millis(100));
        motion.sync();
        let position = motion.left.state();
        let width = motion.width.state();
        motion.retarget(TabsIndicatorRect { left: -50.0, width: 50.0 }, true);
        assert_eq!(motion.left.state(), position);
        assert_eq!(motion.width.state(), width);
        motion.left.elapse_for_test(Duration::from_secs(10));
        motion.width.elapse_for_test(Duration::from_secs(10));
        assert!(!motion.sync());
        assert_eq!(motion.display_rect(), TabsIndicatorRect { left: -50.0, width: 50.0 });
        motion.retarget(TabsIndicatorRect { left: 100.0, width: 90.0 }, true);
        motion.set_motion_enabled(false);
        assert_eq!(motion.display_rect(), TabsIndicatorRect { left: 100.0, width: 90.0 });
        assert!(!motion.is_animating());
        motion.set_motion_enabled(true);
        motion.retarget(TabsIndicatorRect { left: 0.0, width: 40.0 }, true);
        assert!(motion.is_animating());
    }

    #[test]
    fn indicator_rect_insets_by_padding() {
        let list = Bounds::new(point(px(10.0), px(20.0)), size(px(400.0), px(40.0)));
        let item = Bounds::new(point(px(30.0), px(20.0)), size(px(80.0), px(40.0)));
        let rect = indicator_rect_from_bounds(item, list, 8.0);
        assert_eq!(rect.left, 28.0);
        assert_eq!(rect.width, 64.0);
    }

    #[test]
    fn changing_indicator_inset_updates_existing_geometry() {
        let motion = TabsIndicatorMotion::new(false);
        motion.set_list_bounds(Bounds::new(point(px(10.0), px(20.0)), size(px(400.0), px(40.0))));
        motion.set_metrics(8.0, 2.0, Some(gpui::black()));
        motion.apply_item_bounds(Bounds::new(point(px(30.0), px(20.0)), size(px(80.0), px(40.0))), false);
        assert_eq!(motion.display_rect_for_test(), TabsIndicatorRect { left: 28.0, width: 64.0 });
        motion.set_metrics(0.0, 3.0, Some(gpui::black()));
        assert_eq!(motion.display_rect_for_test(), TabsIndicatorRect { left: 20.0, width: 80.0 });
        assert_eq!(motion.paint().unwrap().height, 3.0);
    }

    #[test]
    fn first_geometry_snaps_without_animation() {
        let motion = TabsIndicatorMotion::new(true);
        let list = Bounds::new(point(px(0.0), px(0.0)), size(px(400.0), px(40.0)));
        let item = Bounds::new(point(px(20.0), px(0.0)), size(px(60.0), px(40.0)));
        motion.set_list_bounds(list);
        motion.set_metrics(4.0, 2.0, Some(gpui::hsla(0.0, 0.0, 0.0, 1.0)));
        motion.apply_item_bounds(item, true);
        assert!(!motion.is_animating());
        let paint = motion.paint().expect("paint");
        assert_eq!(paint.left, 24.0);
        assert_eq!(paint.width, 52.0);
    }

    #[test]
    fn poisoned_motion_lock_does_not_abort_paint() {
        let motion = TabsIndicatorMotion::new(false);
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = motion.inner.lock().unwrap();
            panic!("poison");
        }));
        assert!(motion.paint().is_none());
        assert!(!motion.is_animating());
    }

    #[test]
    fn retarget_animates_between_rects() {
        let motion = TabsIndicatorMotion::new(true);
        let list = Bounds::new(point(px(0.0), px(0.0)), size(px(400.0), px(40.0)));
        motion.set_list_bounds(list);
        motion.set_metrics(0.0, 2.0, Some(gpui::hsla(0.0, 0.0, 0.0, 1.0)));
        motion.apply_item_bounds(Bounds::new(point(px(0.0), px(0.0)), size(px(40.0), px(40.0))), true);
        motion.apply_item_bounds(Bounds::new(point(px(100.0), px(0.0)), size(px(50.0), px(40.0))), true);
        assert!(motion.is_animating());
        let current = motion.display_rect_for_test();
        assert_eq!(current.left, 0.0);
        assert_eq!(current.width, 40.0);
    }
}
