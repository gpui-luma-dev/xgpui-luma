use crate::{
    BarLayer, BarOrientation, CartesianChartBuilder, CategoryScale, Dataset, NumericScale, PlotPoint, Series, Sample,
};
use gpui::{Context, IntoElement, Render, TestAppContext, Window, div, prelude::*, px, point};
use xgpui_luma::controls::tooltip::{Tooltip, TooltipEvent};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
    time::Duration,
};

struct Harness {
    created: Rc<RefCell<Vec<PlotPoint>>>,
    events: Arc<Mutex<Vec<TooltipEvent>>>,
}
impl Render for Harness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let created = self.created.clone();
        let events = self.events.clone();
        let data = Dataset::new()
            .with_series(Series::new("a", "A", [Sample::new(0.0, 2.0), Sample::new(1.0, 4.0)]))
            .unwrap();
        div().w(px(300.0)).h(px(200.0)).child(
            CartesianChartBuilder::new(data)
                .x_scale(CategoryScale::new(["First", "Second"]))
                .y_scale(NumericScale::new().domain(0.0..=5.0))
                .axes([])
                .style_override(|style| {
                    style.legend.visible = false;
                    style.metrics.plot_padding = px(0.0);
                })
                .bar_orientation(BarOrientation::Right)
                .bar(BarLayer::new("a"))
                .bar_tooltip(move |point| {
                    created.borrow_mut().push(point.clone());
                    let events = events.clone();
                    Tooltip::new(point.sample.y.to_string())
                        .delay(Duration::ZERO)
                        .hide_delay(Duration::ZERO)
                        .on_event(move |event| events.lock().unwrap().push(event))
                })
                .tooltip_motion(Duration::from_millis(250))
                .build(),
        )
    }
}
#[gpui::test]
fn motion_tooltip_opens_on_bar_body_retargets_and_dismisses_in_gap(cx: &mut TestAppContext) {
    let created = Rc::new(RefCell::new(Vec::new()));
    let events = Arc::new(Mutex::new(Vec::new()));
    let (_, cx) = cx.add_window_view(|_, _| Harness { created: created.clone(), events: events.clone() });
    cx.update(|window, _| window.activate_window());
    // Draw with the pointer reapplied: the mock platform resets it during scheduled frames.
    let draw = |pointer, cx: &mut gpui::VisualTestContext| {
        for _ in 0..3 {
            cx.executor().advance_clock(Duration::from_millis(10));
            cx.update(|window, cx| {
                window.simulate_mouse_move(pointer, cx);
                window.draw(cx).clear(cx);
            });
        }
    };
    draw(point(px(20.0), px(50.0)), cx);
    assert_eq!(created.borrow().len(), 1);
    assert_eq!(created.borrow()[0].sample, Sample::new(0.0, 2.0));
    assert!(events.lock().unwrap().iter().any(|event| matches!(event, TooltipEvent::Shown)));
    draw(point(px(30.0), px(50.0)), cx);
    assert_eq!(created.borrow().len(), 1);
    draw(point(px(30.0), px(150.0)), cx);
    assert_eq!(created.borrow().last().unwrap().sample, Sample::new(1.0, 4.0));
    draw(point(px(30.0), px(100.0)), cx);
    assert!(events.lock().unwrap().iter().any(|event| matches!(event, TooltipEvent::Hidden)));
}

struct SiblingHarness {
    events: Arc<Mutex<Vec<TooltipEvent>>>,
}
impl Render for SiblingHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().flex().w(px(600.0)).h(px(200.0)).children((0_usize..2).map(|index| {
            let events = self.events.clone();
            let data = Dataset::new().with_series(Series::new("a", "A", [Sample::new(0.0, 4.0)])).unwrap();
            div().w(px(300.0)).h(px(200.0)).child(
                CartesianChartBuilder::new(data)
                    .id(("sibling-chart", index))
                    .x_scale(CategoryScale::new(["Only"]))
                    .y_scale(NumericScale::new().domain(0.0..=5.0))
                    .axes([])
                    .style_override(|style| {
                        style.legend.visible = false;
                        style.metrics.plot_padding = px(0.0);
                    })
                    .bar_orientation(BarOrientation::Right)
                    .bar(BarLayer::new("a"))
                    .bar_tooltip(move |_| {
                        let events = events.clone();
                        Tooltip::new("Value")
                            .delay(Duration::ZERO)
                            .on_event(move |event| events.lock().unwrap().push(event))
                    })
                    .tooltip_motion(Duration::from_millis(250))
                    .build(),
            )
        }))
    }
}
#[gpui::test]
fn neighboring_charts_do_not_dismiss_each_others_tooltips(cx: &mut TestAppContext) {
    let events = Arc::new(Mutex::new(Vec::new()));
    let (_, cx) = cx.add_window_view(|_, _| SiblingHarness { events: events.clone() });
    cx.update(|window, _| window.activate_window());
    for _ in 0..4 {
        cx.executor().advance_clock(Duration::from_millis(10));
        cx.update(|window, cx| {
            window.simulate_mouse_move(point(px(30.0), px(100.0)), cx);
            window.draw(cx).clear(cx);
        });
    }
    let first_count = events.lock().unwrap().iter().filter(|event| matches!(event, TooltipEvent::Shown)).count();
    assert!(first_count > 0);
    for _ in 0..4 {
        cx.executor().advance_clock(Duration::from_millis(10));
        cx.update(|window, cx| {
            window.simulate_mouse_move(point(px(330.0), px(100.0)), cx);
            window.draw(cx).clear(cx);
        });
    }
    assert!(events.lock().unwrap().iter().filter(|event| matches!(event, TooltipEvent::Shown)).count() > first_count);
}
