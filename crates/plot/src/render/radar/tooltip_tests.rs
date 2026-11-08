use crate::{Dataset, Series, Sample, RadarChartBuilder, RadarPresentation, RadarSpoke};
use gpui::{Context, IntoElement, Render, TestAppContext, Window, div, prelude::*, px, point};
use xgpui_luma::controls::tooltip::{Tooltip, TooltipEvent};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
    time::Duration,
};
struct Harness {
    created: Rc<RefCell<Vec<RadarSpoke>>>,
    events: Arc<Mutex<Vec<TooltipEvent>>>,
    selections: Rc<RefCell<Vec<crate::SelectionEvent<RadarSpoke>>>>,
}
impl Render for Harness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let selections = self.selections.clone();
        let created = self.created.clone();
        let events = self.events.clone();
        let data = Dataset::new()
            .with_series(Series::new(
                "a",
                "A",
                [Sample::new(1.0, 30.0), Sample::new(0.0, 40.0), Sample::new(2.0, 20.0), Sample::new(3.0, 10.0)],
            ))
            .unwrap()
            .with_series(Series::new("b", "B", [Sample::new(0.0, 10.0), Sample::new(1.0, 20.0)]))
            .unwrap();
        div().w(px(300.0)).h(px(300.0)).child(
            RadarChartBuilder::new(data, "a")
                .series("b")
                .categories(["Top", "Right", "Bottom", "Left"])
                .maximum(50.0)
                .style(|| RadarPresentation { padding: px(0.0), label_gap: px(0.0), ..Default::default() })
                .spoke_tooltip(move |spoke| {
                    created.borrow_mut().push(spoke.clone());
                    let events = events.clone();
                    Tooltip::new(spoke.category.clone())
                        .delay(Duration::from_millis(250))
                        .hide_delay(Duration::ZERO)
                        .on_event(move |event| events.lock().unwrap().push(event))
                })
                .on_spoke_event(move |event, _| selections.borrow_mut().push(event))
                .tooltip_motion(Duration::from_millis(250))
                .build(),
        )
    }
}
#[gpui::test]
fn delayed_radar_tooltip_cancels_quick_pass_and_reports_all_original_values(cx: &mut TestAppContext) {
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
    draw(point(px(150.0), px(80.0)), 5, cx);
    cx.simulate_click(point(px(150.0), px(80.0)), gpui::Modifiers::default());
    assert!(selections.borrow().iter().any(
        |e| matches!(e,crate::SelectionEvent::Select{target} if target.category_index==0 && target.points.len()==2)
    ));
    assert_eq!(created.borrow()[0].category, "Top");
    assert_eq!(created.borrow()[0].points.len(), 2);
    assert_eq!(created.borrow()[0].points[0].sample_index, 1);
    assert_eq!(created.borrow()[0].points[0].sample.y, 40.0);
    assert!(!events.lock().unwrap().iter().any(|event| matches!(event, TooltipEvent::Shown)));
    draw(point(px(150.0), px(150.0)), 30, cx);
    assert!(!events.lock().unwrap().iter().any(|event| matches!(event, TooltipEvent::Shown)));
    draw(point(px(150.0), px(80.0)), 30, cx);
    assert!(events.lock().unwrap().iter().any(|event| matches!(event, TooltipEvent::Shown)));
    draw(point(px(220.0), px(150.0)), 30, cx);
    assert_eq!(created.borrow().last().unwrap().category, "Right");
    draw(point(px(0.0), px(0.0)), 4, cx);
    assert!(events.lock().unwrap().iter().any(|event| matches!(event, TooltipEvent::Hidden)));
}
