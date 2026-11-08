//! App-owned, interruption-aware slide using public SDK APIs.
use std::collections::HashMap;
use gpui::{AnyElement, App, SharedString, Window, div, prelude::*, relative};
use xgpui_luma::controls::tabs::{
    TabsContentTransition, TabsTransitionChange, TabsTransitionDirection, TabsTransitionFrame, TabsTransitionRenderer,
};

#[derive(Default)]
struct DirectionalSlide {
    // Per-panel start/end offsets in viewport widths.
    poses: HashMap<SharedString, (f32, f32)>,
}
impl TabsTransitionRenderer for DirectionalSlide {
    fn selection_changed(&mut self, change: TabsTransitionChange) {
        if !change.animated {
            self.poses.clear();
            if let Some(id) = change.active_id {
                self.poses.insert(id, (0.0, 0.0));
            }
            return;
        }
        if change.previous_progress >= 1.0 {
            self.poses.retain(|id, _| Some(id) == change.previous_id.as_ref());
        }
        let direction = match change.direction {
            TabsTransitionDirection::Forward => 1.0,
            TabsTransitionDirection::Backward => -1.0,
        };
        for (id, (start, end)) in &mut self.poses {
            let current = *start + (*end - *start) * change.previous_progress;
            *start = current;
            *end = if Some(id) == change.active_id.as_ref() {
                0.0
            } else if current.abs() > 0.0001 {
                current.signum()
            } else {
                -direction
            };
        }
        if let Some(id) = change.active_id {
            self.poses.entry(id).or_insert((direction, 0.0));
        }
    }
    fn render(&mut self, frame: TabsTransitionFrame, _: &mut Window, _: &mut App) -> AnyElement {
        if !frame.animating {
            self.poses.retain(|id, _| frame.incoming.as_ref().is_some_and(|panel| &panel.id == id));
        }
        let progress = frame.progress;
        div()
            .relative()
            .size_full()
            .children(frame.outgoing.into_iter().chain(frame.incoming).map(|panel| {
                let (start, end) = self.poses.get(&panel.id).copied().unwrap_or((0.0, 0.0));
                div()
                    .id(panel.id)
                    .absolute()
                    .top_0()
                    .left(relative(start + (end - start) * progress))
                    .size_full()
                    .flex()
                    .flex_col()
                    .child(panel.element)
            }))
            .into_any_element()
    }
}
pub(crate) fn directional_slide() -> TabsContentTransition {
    TabsContentTransition::with_factory(DirectionalSlide::default)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn change(active: &str, progress: f32, direction: TabsTransitionDirection, animated: bool) -> TabsTransitionChange {
        TabsTransitionChange {
            previous_id: animated.then(|| if active == "b" { "a" } else { "b" }.into()),
            active_id: Some(active.into()),
            previous_progress: progress,
            direction,
            animated,
        }
    }
    #[test]
    fn rapid_forward_selection_preserves_all_current_positions() {
        let mut slide = DirectionalSlide::default();
        slide.selection_changed(change("a", 1.0, TabsTransitionDirection::Forward, false));
        slide.selection_changed(change("b", 1.0, TabsTransitionDirection::Forward, true));
        slide.selection_changed(change("c", 0.4, TabsTransitionDirection::Forward, true));
        assert!((slide.poses["a"].0 + 0.4).abs() < 0.0001);
        assert!((slide.poses["b"].0 - 0.6).abs() < 0.0001);
        assert_eq!(slide.poses["c"], (1.0, 0.0));
        // Old panels leave on their current side instead of crossing the viewport.
        assert_eq!(slide.poses["a"].1, -1.0);
        assert_eq!(slide.poses["b"].1, 1.0);
    }
    #[test]
    fn reversal_retargets_from_the_sampled_poses_and_disable_settles() {
        let mut slide = DirectionalSlide::default();
        slide.selection_changed(change("a", 1.0, TabsTransitionDirection::Forward, false));
        slide.selection_changed(change("b", 1.0, TabsTransitionDirection::Forward, true));
        slide.selection_changed(change("a", 0.4, TabsTransitionDirection::Backward, true));
        assert!((slide.poses["a"].0 + 0.4).abs() < 0.0001);
        assert_eq!(slide.poses["a"].1, 0.0);
        assert!((slide.poses["b"].0 - 0.6).abs() < 0.0001);
        assert_eq!(slide.poses["b"].1, 1.0);
        slide.selection_changed(change("a", 1.0, TabsTransitionDirection::Backward, false));
        assert_eq!(slide.poses.len(), 1);
        assert_eq!(slide.poses["a"], (0.0, 0.0));
    }
}
