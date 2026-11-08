//! Exercise smooth scrolling through the shared container's public API.
use gpui::{
    Context, Render, Window, TestAppContext, VisualTestContext, ScrollEvent, ScrollDelta, TouchPhase, div, px, point,
    prelude::*,
};
use xgpui_luma::controls::scroll_container::ScrollContainer;
use crate::ShadcnLook;

struct Harness {
    scroll: ScrollContainer,
}
impl Render for Harness {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.scroll.sync_scrollbar(cx);
        div()
            .w(px(250.0))
            .h(px(200.0))
            .child(self.scroll.render(div().h(px(2000.0)).w_full().into_any_element()))
    }
}

fn frames(cx: &mut VisualTestContext, count: usize) {
    cx.run_until_parked();
    for _ in 0..count {
        cx.executor().advance_clock(std::time::Duration::from_millis(16));
        cx.update(|window, app| window.simulate_next_frame(app));
        cx.run_until_parked();
    }
}

#[test]
fn scroll_container_smooth_offset_clamps_and_yields_to_wheel_and_immediate_requests() {
    let mut app = TestAppContext::single();
    let (view, cx) = app.add_window_view(|_, cx| Harness {
        scroll: ScrollContainer::new("scroll", ShadcnLook::built_in().scrollbar_template(), cx),
    });
    cx.update(|window, _| window.activate_window());
    frames(cx, 2);
    cx.update(|_, app| view.update(app, |host, cx| host.scroll.set_vertical_offset_smooth(1000.0, cx)));
    frames(cx, 2);
    let during = cx.update(|_, app| view.read(app).scroll.vertical_offset());
    assert!(during > px(0.0) && during < px(1000.0));
    frames(cx, 24);
    cx.update(|_, app| assert_eq!(view.read(app).scroll.vertical_offset(), px(1000.0)));
    cx.update(|_, app| view.update(app, |host, cx| host.scroll.set_vertical_offset_smooth(10000.0, cx)));
    frames(cx, 24);
    cx.update(|_, app| {
        let scroll = &view.read(app).scroll;
        assert_eq!(scroll.vertical_offset(), scroll.max_vertical_offset());
    });
    cx.update(|_, app| view.update(app, |host, cx| host.scroll.set_vertical_offset_smooth(0.0, cx)));
    frames(cx, 2);
    cx.simulate_event(ScrollEvent {
        position: point(px(30.0), px(30.0)),
        delta: ScrollDelta::Pixels(point(px(0.0), px(-12.0))),
        modifiers: Default::default(),
        source: gpui::ScrollSource::Pointer,
        touch_phase: TouchPhase::Moved,
    });
    cx.run_until_parked();
    let stopped = cx.update(|_, app| view.read(app).scroll.vertical_offset());
    frames(cx, 24);
    cx.update(|_, app| assert_eq!(view.read(app).scroll.vertical_offset(), stopped));
    cx.update(|_, app| view.update(app, |host, cx| host.scroll.set_vertical_offset_smooth(0.0, cx)));
    frames(cx, 2);
    cx.update(|_, app| view.update(app, |host, cx| host.scroll.set_vertical_offset(400.0, cx)));
    frames(cx, 24);
    cx.update(|_, app| assert_eq!(view.read(app).scroll.vertical_offset(), px(400.0)));
}
