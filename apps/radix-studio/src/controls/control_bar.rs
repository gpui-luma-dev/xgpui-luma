//! Independently rendered navigation chrome driven by actual page offsets.
use std::{sync::Arc, time::Duration};
use gpui::{Context, Entity, Hsla, Render, Window, div, prelude::*};
use xgpui_luma::motion::{Easing, Transition, VisualTransition};
use xgpui_luma_look_radix::{Look, SemanticRole};
use super::ScreenNav;
use crate::tabs::RadixStudioTab;

const FADE_DISTANCE: f32 = 96.0;

pub struct ControlBar {
    look: Arc<Look>,
    nav: Entity<ScreenNav>,
    active: RadixStudioTab,
    opacity: VisualTransition,
    target: f32,
}
impl ControlBar {
    pub fn new(look: &Arc<Look>, nav: Entity<ScreenNav>) -> Self {
        Self {
            look: look.clone(),
            nav,
            active: RadixStudioTab::CustomPalette,
            opacity: VisualTransition::with_transition(
                0.0,
                Transition::timed(Duration::from_millis(120), Easing::OutCubic),
            ),
            target: 0.0,
        }
    }
    fn retarget(&mut self, offset: f32, duration: Duration) -> bool {
        let distance = (-offset / FADE_DISTANCE).clamp(0.0, 1.0);
        let target = distance * distance * (3.0 - 2.0 * distance);
        if (target - self.target).abs() < 0.0001 {
            return false;
        }
        // Retarget from the last painted sample, including during rapid navigation.
        self.opacity.set_transition(Transition::timed(duration, Easing::OutCubic));
        self.opacity.set_target(target);
        self.target = target;
        true
    }
    pub fn select(&mut self, screen: RadixStudioTab, offset: f32, duration: Duration, cx: &mut Context<Self>) {
        self.active = screen;
        if self.retarget(offset, duration) {
            cx.notify();
        }
    }
    /// Called after layout; wheel events at a scroll boundary cause no updates.
    pub fn scroll_changed(&mut self, screen: RadixStudioTab, offset: f32) -> bool {
        screen == self.active && self.retarget(offset, Duration::from_millis(120))
    }
}
impl Render for ControlBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.opacity.sync();
        self.opacity.schedule_frame(window, cx);
        let opacity = self.opacity.progress().clamp(0.0, 1.0);
        let background = self.look.resolve_role(SemanticRole::Background).hsla();
        let border = self.look.resolve_role(SemanticRole::Border).hsla();
        div()
            .id("radix-studio-control-bar")
            .w_full()
            .flex_none()
            .px_8()
            .py_2()
            .border_b_1()
            .border_color(Hsla { a: border.a * opacity, ..border })
            .bg(Hsla { a: background.a * opacity, ..background })
            .child(self.nav.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opacity_tracks_scroll_distance_and_navigation_retargets_without_snapping() {
        let app = gpui::TestAppContext::single();
        app.update(|cx| {
            let look = Arc::new(Look::built_in());
            let nav = cx.new(|cx| {
                let tabs = xgpui_luma_look_radix::Tabs::new("bar-pages").look(&look).spawn(cx);
                ScreenNav::new(&look, &look, tabs, cx)
            });
            let mut bar = ControlBar::new(&look, nav);
            assert!(!bar.scroll_changed(RadixStudioTab::CustomPalette, 0.0));
            assert!(!bar.scroll_changed(RadixStudioTab::Colors, -96.0));
            assert!(bar.scroll_changed(RadixStudioTab::CustomPalette, -48.0));
            assert_eq!(bar.target, 0.5);
            assert_eq!(bar.opacity.progress(), 0.0);
            bar.opacity.snap_to(0.5);
            assert!(bar.retarget(0.0, Duration::from_millis(400)));
            assert_eq!(bar.opacity.progress(), 0.5);
            assert!(bar.opacity.is_animating());
            assert!(!bar.retarget(0.0, Duration::from_millis(120)));
        });
    }
}
