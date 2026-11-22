//! Keyed post-drop layout transitions. Only mounted, clipped-in items are tracked.
use std::{cell::RefCell, collections::HashMap, hash::Hash, rc::Rc, time::Duration};
use gpui::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement, LayoutId, Pixels,
    Point, Window, point,
};
use gpui::prelude::*;
use super::{VisualTransition, Transition};

/// Shared retained geometry for one collection. Data commits and lifecycle events
/// remain synchronous; this animates existing keyed elements, never cloned views.
pub struct DropLayoutMotion<K>(Rc<RefCell<State<K>>>);
impl<K> Clone for DropLayoutMotion<K> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
struct State<K> {
    current: HashMap<K, Bounds<Pixels>>,
    from: HashMap<K, Bounds<Pixels>>,
    targets: HashMap<K, Bounds<Pixels>>,
    viewport: Option<Bounds<Pixels>>,
    scroll: Point<Pixels>,
    pending: bool,
    channels: HashMap<K, (VisualTransition, VisualTransition)>,
    definition: Transition,
    #[cfg(all(test, feature = "test-support"))]
    elapsed: Option<Duration>,
    enabled: bool,
}
impl<K> Default for DropLayoutMotion<K> {
    fn default() -> Self {
        Self(Rc::new(RefCell::new(State {
            current: HashMap::new(),
            from: HashMap::new(),
            targets: HashMap::new(),
            viewport: None,
            scroll: Point::default(),
            pending: false,
            channels: HashMap::new(),
            definition: Transition::default(),
            #[cfg(all(test, feature = "test-support"))]
            elapsed: None,
            enabled: true,
        })))
    }
}
impl<K: Clone + 'static> DropLayoutMotion<K> {
    /// Change policy while retaining the captured timing and role.
    pub fn set_motion_enabled(&self, enabled: bool) {
        let mut s = self.0.borrow_mut();
        if s.enabled != enabled {
            Self::settle(&mut s);
            s.enabled = enabled;
        }
    }
    /// Apply the control's frozen role without reconfiguring trajectories on render.
    pub fn configure_transition(&self, definition: &Transition) {
        let mut s = self.0.borrow_mut();
        if !s.definition.same_definition(definition) {
            Self::settle(&mut s);
            s.definition = definition.clone();
        }
    }

    /// Call after a changed atomic commit, before the next render. A no-op,
    /// rejection or cancellation must not start a layout transition.
    pub fn start(&self) {
        let mut s = self.0.borrow_mut();
        if !s.enabled {
            Self::settle(&mut s);
            return;
        }
        let definition = s.definition.clone();
        if definition.start(super::MotionState::default(), 1.0).sample(Duration::ZERO).complete {
            Self::settle(&mut s);
            return;
        }
        // Retain per-key channel velocity when another drop interrupts this one.
        s.from = s.current.clone();
        s.targets.clear();
        s.pending = !s.from.is_empty();
        #[cfg(all(test, feature = "test-support"))]
        if s.elapsed.is_some() {
            s.elapsed = Some(Duration::ZERO);
        }
    }

    /// Settle on new input/data/presentation changes. Keep last measured geometry
    /// available for a synchronous host-owned drop commit in the same turn.
    pub fn cancel(&self) {
        Self::settle(&mut self.0.borrow_mut());
    }
    fn settle(s: &mut State<K>) {
        s.from.clear();
        s.targets.clear();
        s.pending = false;
        s.channels.clear();
    }
}
impl<K: Clone + Eq + Hash + 'static> DropLayoutMotion<K> {
    /// Wrap the collection viewport once. Offsets are sampled during prepaint,
    /// after layout/scroll clamping; scrolling and resize settle active motion.
    pub fn scope(&self, content: impl IntoElement, scroll: impl Fn() -> Point<Pixels> + 'static) -> AnyElement {
        Scope { child: content.into_any_element(), motion: self.clone(), scroll: Box::new(scroll) }.into_any_element()
    }
    /// Wrap one real keyed row/button. Layout remains at its committed position;
    /// paint and hit testing move together from the previously visible position.
    pub fn item(&self, key: K, content: impl IntoElement) -> AnyElement {
        Item { child: content.into_any_element(), key, motion: self.clone() }.into_any_element()
    }
    /// Select an explicit elapsed time, independent of machine speed. Setting
    /// zero before a drop freezes its first frame until advanced by the test.
    #[cfg(all(test, feature = "test-support"))]
    pub(crate) fn elapse_for_test(&self, elapsed: Duration) {
        let mut s = self.0.borrow_mut();
        s.elapsed = Some(elapsed);
        for (x, y) in s.channels.values_mut() {
            x.elapse_for_test(elapsed);
            y.elapse_for_test(elapsed);
        }
    }
    /// Number of visible measured identities retained (at most 512).
    pub fn tracked_items(&self) -> usize {
        self.0.borrow().current.len()
    }

    pub fn is_animating(&self) -> bool {
        let s = self.0.borrow();
        s.pending || s.channels.values().any(|(x, y)| x.is_animating() || y.is_animating())
    }
    // Preserve stateful template roots (focus, style modifiers and native IDs).
    pub(crate) fn begin_surface(
        &self,
        surface: gpui::Stateful<gpui::Div>,
        scroll: impl Fn() -> Point<Pixels> + 'static,
    ) -> gpui::Stateful<gpui::Div> {
        let motion = self.clone();
        surface.child(
            gpui::canvas(move |bounds, _, cx| motion.begin(bounds, scroll(), cx.reduce_motion()), |_, _, _, _| {})
                .absolute()
                .inset_0()
                .size_full(),
        )
    }
    pub(crate) fn end_surface(&self, surface: gpui::Stateful<gpui::Div>) -> gpui::Stateful<gpui::Div> {
        let motion = self.clone();
        surface.child(
            gpui::canvas(move |_, window, _| motion.finish(window), |_, _, _, _| {})
                .absolute()
                .inset_0()
                .size_full(),
        )
    }
    fn finish(&self, window: &mut Window) {
        {
            let mut s = self.0.borrow_mut();
            if s.pending
                || !s
                    .targets
                    .iter()
                    .any(|(key, target)| s.from.get(key).is_some_and(|from| from.origin != target.origin))
            {
                Self::settle(&mut s);
            }
        }
        if self.is_animating() {
            window.on_next_frame(|window, _| window.refresh());
        }
    }
    fn begin(&self, bounds: Bounds<Pixels>, scroll: Point<Pixels>, reduced: bool) {
        let mut s = self.0.borrow_mut();
        if reduced || s.viewport.is_some_and(|old| old != bounds) || s.scroll != scroll {
            Self::settle(&mut s);
        }
        s.viewport = Some(bounds);
        s.scroll = scroll;
        let mounted = s.current.clone();
        s.channels.retain(|key, _| mounted.contains_key(key));
        s.current.clear();
        let mut active = false;
        for (x, y) in s.channels.values_mut() {
            active |= x.sync();
            active |= y.sync();
        }
        if !s.pending && !active {
            s.from.clear();
            s.targets.clear();
            s.channels.clear();
        }
    }
    fn offset(&self, key: &K, bounds: Bounds<Pixels>, mask: Bounds<Pixels>) -> Point<Pixels> {
        let mut s = self.0.borrow_mut();
        if !bounds.intersects(&mask) || !s.viewport.is_some_and(|v| v.intersects(&bounds)) {
            return Point::default();
        }
        if s.current.len() >= 512 {
            return Point::default();
        }
        s.current.insert(key.clone(), bounds);
        let Some(from) = s.from.get(key).copied() else {
            return Point::default();
        };
        if s.targets.get(key).is_some_and(|target| *target != bounds) {
            Self::settle(&mut s);
            return Point::default();
        }
        s.targets.insert(key.clone(), bounds);
        s.pending = false;
        let definition = s.definition.clone();
        #[cfg(all(test, feature = "test-support"))]
        let elapsed = s.elapsed;
        if !s.channels.contains_key(key) && s.channels.len() >= 512 {
            return Point::default();
        }
        let (x, y) = s.channels.entry(key.clone()).or_insert_with(|| {
            (
                VisualTransition::with_transition(from.origin.x.as_f32(), definition.clone()),
                VisualTransition::with_transition(from.origin.y.as_f32(), definition),
            )
        });
        x.set_target(bounds.origin.x.as_f32());
        y.set_target(bounds.origin.y.as_f32());
        #[cfg(all(test, feature = "test-support"))]
        if let Some(elapsed) = elapsed {
            x.elapse_for_test(elapsed);
            y.elapse_for_test(elapsed);
            x.sync();
            y.sync();
        }
        let origin = point(gpui::px(x.progress()), gpui::px(y.progress()));
        s.current.insert(key.clone(), Bounds { origin, ..bounds });
        origin - bounds.origin
    }
}
struct Item<K> {
    child: AnyElement,
    key: K,
    motion: DropLayoutMotion<K>,
}
struct Scope<K> {
    child: AnyElement,
    motion: DropLayoutMotion<K>,
    scroll: Box<dyn Fn() -> Point<Pixels>>,
}
macro_rules! transparent_layout {
    () => {
        type RequestLayoutState = ();
        type PrepaintState = ();
        fn id(&self) -> Option<ElementId> {
            None
        }
        fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
            None
        }
        fn request_layout(
            &mut self,
            _: Option<&GlobalElementId>,
            _: Option<&InspectorElementId>,
            window: &mut Window,
            cx: &mut App,
        ) -> (LayoutId, ()) {
            (self.child.request_layout(window, cx), ())
        }
        fn paint(
            &mut self,
            _: Option<&GlobalElementId>,
            _: Option<&InspectorElementId>,
            _: Bounds<Pixels>,
            _: &mut (),
            _: &mut (),
            window: &mut Window,
            cx: &mut App,
        ) {
            self.child.paint(window, cx);
        }
    };
}
impl<K: Clone + Eq + Hash + 'static> Element for Item<K> {
    transparent_layout!();
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let offset = self.motion.offset(&self.key, bounds, window.content_mask().bounds);
        window.with_element_offset(offset, |window| {
            self.child.prepaint(window, cx);
        });
    }
}
impl<K: Clone + Eq + Hash + 'static> Element for Scope<K> {
    transparent_layout!();
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.motion.begin(bounds, (self.scroll)(), cx.reduce_motion());
        self.child.prepaint(window, cx);
        self.motion.finish(window);
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::{Context, Entity, Render, TestAppContext, VisualTestContext, div, px};

    #[test]
    fn visible_geometry_cache_is_bounded_with_typed_motion() {
        let motion = DropLayoutMotion::<u32>::default();
        motion.configure_transition(&Transition::spring(300.0, 28.0));
        let bounds = Bounds::new(point(px(0.0), px(0.0)), gpui::size(px(120.0), px(90.0)));
        motion.begin(bounds, Point::default(), false);
        for key in 0..10_000 {
            motion.offset(&key, bounds, bounds);
        }
        assert_eq!(motion.tracked_items(), 512);
    }

    #[test]
    fn keyed_spring_channels_retain_velocity_across_interrupted_drops() {
        use gpui::size;
        let motion = DropLayoutMotion::<u32>::default();
        let viewport = Bounds::new(point(px(0.), px(0.)), size(px(200.), px(200.)));
        let bounds = |y| Bounds::new(point(px(0.), px(y)), size(px(100.), px(20.)));
        motion.configure_transition(&Transition::spring(200., 18.));
        motion.elapse_for_test(Duration::ZERO);
        motion.begin(viewport, Point::default(), false);
        motion.offset(&1, bounds(0.), viewport);
        motion.start();
        motion.begin(viewport, Point::default(), false);
        motion.offset(&1, bounds(80.), viewport);
        motion.elapse_for_test(Duration::from_millis(100));
        motion.begin(viewport, Point::default(), false);
        motion.offset(&1, bounds(80.), viewport);
        let before = motion.0.borrow().channels[&1].1.state();
        assert!(before.velocity > 0.);
        motion.start();
        motion.begin(viewport, Point::default(), false);
        motion.offset(&1, bounds(20.), viewport);
        assert_eq!(motion.0.borrow().channels[&1].1.state(), before);
        motion.elapse_for_test(Duration::from_secs(10));
        motion.begin(viewport, Point::default(), false);
        assert!(!motion.is_animating());
    }
    struct Page {
        order: Vec<u32>,
        motion: DropLayoutMotion<u32>,
        scroll: Point<Pixels>,
        width: f32,
    }
    impl Render for Page {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let rows = div().w(gpui::relative(1.)).flex().flex_col().children(self.order.iter().map(|key| {
                self.motion.item(
                    *key,
                    div()
                        .id(("row", *key))
                        .w_full()
                        .h(px(30.))
                        .bg(if *key == 0 {
                            gpui::rgb(0xff0000)
                        } else {
                            gpui::rgb(0x0000ff)
                        })
                        .debug_selector({
                            let key = *key;
                            move || format!("row-{key}")
                        }),
                )
            }));
            let scroll = self.scroll;
            div().w(px(self.width)).h(px(90.)).overflow_hidden().child(self.motion.scope(rows, move || scroll))
        }
    }
    fn setup(app: &mut TestAppContext) -> (Entity<Page>, &mut VisualTestContext) {
        let (page, cx) = app.add_window_view(|_, _| Page {
            order: vec![0, 1, 2],
            motion: DropLayoutMotion::default(),
            scroll: Point::default(),
            width: 120.,
        });
        cx.run_until_parked();
        (page, cx)
    }
    fn frame(cx: &mut VisualTestContext) {
        cx.update(|window, cx| {
            window.simulate_next_frame(cx);
        });
        cx.run_until_parked();
    }
    fn red_y(cx: &mut VisualTestContext) -> f32 {
        cx.update(|window, _| {
            window
                .painted_quads()
                .iter()
                .find(|q| q.background.as_solid() == Some(gpui::Hsla::from(gpui::rgb(0xff0000))))
                .unwrap()
                .bounds
                .origin
                .y
                .as_f32()
                / window.scale_factor()
        })
    }
    #[test]
    fn painted_positions_interpolate_and_finish_without_cloning_rows() {
        let mut app = TestAppContext::single();
        let (page, cx) = setup(&mut app);
        assert_eq!(red_y(cx), 0.);
        page.update(cx, |page, cx| {
            page.motion.start();
            page.order = vec![1, 2, 0];
            cx.notify();
        });
        cx.run_until_parked();
        assert_eq!(red_y(cx), 0.);
        page.update(cx, |p, _| p.motion.elapse_for_test(Duration::from_millis(100)));
        frame(cx);
        let y = red_y(cx);
        assert!(y > 0. && y < 60., "intermediate {y}");
        cx.update(|window, _| {
            assert_eq!(
                window
                    .painted_quads()
                    .iter()
                    .filter(|q| q.background.as_solid() == Some(gpui::Hsla::from(gpui::rgb(0xff0000))))
                    .count(),
                1
            )
        });
        page.update(cx, |p, _| p.motion.elapse_for_test(Duration::from_millis(250)));
        frame(cx);
        assert_eq!(red_y(cx), 60.);
        cx.update(|_, cx| assert!(!page.read(cx).motion.is_animating()));
    }
    #[test]
    fn reduced_disabled_scroll_resize_and_cancel_settle() {
        for mode in 0..5 {
            let mut app = TestAppContext::single();
            let (page, cx) = setup(&mut app);
            if mode == 0 {
                cx.update(|_, cx| cx.set_reduce_motion(true));
            }
            page.update(cx, |p, cx| {
                if mode == 1 {
                    p.motion.configure_transition(&Transition::instant());
                }
                p.motion.start();
                p.order = vec![1, 2, 0];
                cx.notify();
            });
            cx.run_until_parked();
            page.update(cx, |p, cx| {
                match mode {
                    2 => p.scroll.y = px(-10.),
                    3 => p.width = 130.,
                    4 => p.motion.cancel(),
                    _ => {}
                }
                cx.notify();
            });
            frame(cx);
            assert_eq!(red_y(cx), 60., "mode {mode}");
            cx.update(|_, cx| assert!(!page.read(cx).motion.is_animating()));
        }
    }
    #[test]
    fn unchanged_layout_and_unseen_items_do_not_schedule_motion() {
        let mut app = TestAppContext::single();
        let (page, cx) = setup(&mut app);
        page.update(cx, |p, cx| {
            p.motion.start();
            cx.notify();
        });
        cx.run_until_parked();
        cx.update(|_, cx| assert!(!page.read(cx).motion.is_animating()));
        page.update(cx, |p, cx| {
            p.motion.start();
            p.order = vec![3, 4, 5];
            cx.notify();
        });
        cx.run_until_parked();
        cx.update(|_, cx| assert!(!page.read(cx).motion.is_animating()));
    }
}
