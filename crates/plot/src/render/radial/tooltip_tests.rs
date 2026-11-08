use crate::{Dataset, Series, Sample, RadialChartBuilder, RadialBar};
use gpui::{Context, IntoElement, Render, TestAppContext, Window, div, prelude::*, px, point};
use xgpui_luma::controls::tooltip::{Tooltip, TooltipEvent};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
    time::Duration,
};
struct Harness {
    created: Rc<RefCell<Vec<RadialBar>>>,
    events: Arc<Mutex<Vec<TooltipEvent>>>,
    selections: Rc<RefCell<Vec<crate::SelectionEvent<RadialBar>>>>,
}
impl Render for Harness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let selections = self.selections.clone();
        let created = self.created.clone();
        let events = self.events.clone();
        let data = Dataset::new()
            .with_series(Series::new("a", "A", [Sample::new(0.0, 100.0), Sample::new(1.0, 50.0)]))
            .unwrap();
        div().w(px(300.0)).h(px(300.0)).child(
            RadialChartBuilder::new(data, "a")
                .bar_names(["First", "Second"])
                .maximum(100.0)
                .bar_tooltip(move |bar| {
                    created.borrow_mut().push(bar.clone());
                    let events = events.clone();
                    Tooltip::new(bar.label.clone())
                        .delay(Duration::from_millis(250))
                        .hide_delay(Duration::ZERO)
                        .on_event(move |event| events.lock().unwrap().push(event))
                })
                .on_bar_event(move |event, _| selections.borrow_mut().push(event))
                .tooltip_motion(Duration::from_millis(250))
                .build(),
        )
    }
}
#[gpui::test]
fn delayed_radial_tooltip_cancels_quick_pass_and_retargets_visible_bands(cx: &mut TestAppContext) {
    let created = Rc::new(RefCell::new(Vec::new()));
    let events = Arc::new(Mutex::new(Vec::new()));
    let selections = Rc::new(RefCell::new(Vec::new()));
    let (_, cx) = cx.add_window_view(|_, _| Harness {
        created: created.clone(),
        events: events.clone(),
        selections: selections.clone(),
    });
    cx.update(|window, _| window.activate_window());
    let draw = |pointer, frames, cx: &mut gpui::VisualTestContext| {
        for _ in 0..frames {
            cx.executor().advance_clock(Duration::from_millis(10));
            cx.update(|window, cx| {
                window.simulate_mouse_move(pointer, cx);
                window.draw(cx).clear(cx);
            });
        }
    };
    draw(point(px(150.0), px(85.0)), 5, cx);
    cx.simulate_click(point(px(150.0), px(85.0)), gpui::Modifiers::default());
    assert!(selections.borrow().iter().any(
        |e| matches!(e,crate::SelectionEvent::Select{target} if target.sample_index==0 && target.sample.y==100.0)
    ));
    assert_eq!(created.borrow()[0].label, "First");
    assert_eq!(created.borrow()[0].sample_index, 0);
    assert_eq!(created.borrow()[0].sample.y, 100.0);
    assert!(!events.lock().unwrap().iter().any(|event| matches!(event, TooltipEvent::Shown)));
    draw(point(px(150.0), px(150.0)), 30, cx);
    assert!(!events.lock().unwrap().iter().any(|event| matches!(event, TooltipEvent::Shown)));
    draw(point(px(150.0), px(85.0)), 30, cx);
    assert!(events.lock().unwrap().iter().any(|event| matches!(event, TooltipEvent::Shown)));
    draw(point(px(150.0), px(35.0)), 30, cx);
    assert_eq!(created.borrow().last().unwrap().label, "Second");
    draw(point(px(0.0), px(0.0)), 4, cx);
    assert!(events.lock().unwrap().iter().any(|event| matches!(event, TooltipEvent::Hidden)));
}
