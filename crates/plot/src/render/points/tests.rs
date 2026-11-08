use super::*;
use crate::{
    AreaLayer, AreaPresentation, CartesianChartBuilder, CartesianPlot, ChartMetrics, Dataset, Decimation,
    Interpolation, LegendPresentation, LineLayer, LinePresentation, MarkerPresentation, MarkerVisibility, NumericScale,
    Sample, Series,
};
use gpui::{Context, Edges, IntoElement, Render, TestAppContext};
use xgpui_luma::controls::tooltip::{Tooltip, TooltipEvent};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
    time::Duration,
};

fn style() -> CartesianPresentation {
    let mut line = LinePresentation::new(gpui::black());
    line.markers = Some(MarkerPresentation::new(gpui::black(), px(4.0)).visibility(MarkerVisibility::Hover));
    CartesianPresentation::default()
        .line("a", line)
        .area("a", AreaPresentation::new(gpui::black()).outline(line))
}

#[test]
fn nearest_targets_original_retained_observations_for_lines_and_areas() {
    let data = Dataset::new()
        .with_series(Series::new(
            "a",
            "A",
            [Sample::new(0.0, 0.0), Sample::new(1.0, 1.0), Sample::new(2.0, f64::NAN), Sample::new(3.0, 0.0)],
        ))
        .expect("dataset");
    let bounds = Bounds::new(point(px(0.0), px(0.0)), size(px(300.0), px(200.0)));
    for area in [false, true] {
        let plot = if area {
            CartesianPlot::new().area(AreaLayer::new("a"))
        } else {
            CartesianPlot::new().line(LineLayer::new("a"))
        };
        let geometry = super::super::geometry::layout(&data, &plot, Edges::default(), bounds);
        let selected = nearest(&geometry, &data, &style(), point(px(100.0), px(1.0)), bounds).expect("observation");
        assert_eq!(selected.area, area);
        assert_eq!(selected.info.sample, Sample::new(1.0, 1.0));
        assert_eq!(selected.info.label, "A");
        assert!(nearest(&geometry, &data, &style(), point(px(200.0), px(100.0)), bounds).is_none());
        let half = Bounds::new(bounds.origin, size(px(90.0), bounds.size.height));
        assert!(nearest(&geometry, &data, &style(), point(px(100.0), px(1.0)), half).is_none());
    }
}

#[test]
fn decimated_point_metadata_matches_drawn_extrema_and_topmost_layer_wins_ties() {
    let mut samples: Vec<_> = (0..100).map(|i| Sample::new(i as f64, 0.0)).collect();
    samples[30].y = 9.0;
    let data = Dataset::new().with_series(Series::new("a", "A", samples)).expect("dataset");
    let plot = CartesianPlot::new()
        .line(LineLayer::new("a").decimation(Decimation::MinMax))
        .area(AreaLayer::new("a").decimation(Decimation::MinMax));
    let bounds = Bounds::new(point(px(0.0), px(0.0)), size(px(10.0), px(200.0)));
    let geometry = super::super::geometry::layout(&data, &plot, Edges::default(), bounds);
    let line = &geometry.lines[0];
    assert!(line.samples.iter().flatten().count() < 100);
    let (position, _) = line
        .runs
        .iter()
        .flatten()
        .zip(line.samples.iter().flatten())
        .find(|(_, sample)| sample.y == 9.0)
        .expect("retained peak");
    let selected = nearest(&geometry, &data, &style(), *position, bounds).expect("peak");
    assert!(!selected.area);
    assert_eq!(selected.info.sample, Sample::new(30.0, 9.0));
}

fn slice_style() -> CartesianPresentation {
    let mut line = LinePresentation::new(gpui::black());
    line.markers = Some(MarkerPresentation::new(gpui::black(), px(4.0)).visibility(MarkerVisibility::HoverSlice));
    CartesianPresentation::default()
        .line("a", line)
        .line("b", line)
        .area("a", AreaPresentation::new(gpui::black()).outline(line))
        .area("b", AreaPresentation::new(gpui::black()).outline(line))
}

#[test]
fn horizontal_slices_ignore_pointer_height_and_group_matching_series() {
    let data = Dataset::new()
        .with_series(Series::new("a", "A", [Sample::new(0.0, 1.0), Sample::new(1.0, 2.0), Sample::new(2.0, 1.0)]))
        .expect("A")
        .with_series(Series::new("b", "B", [Sample::new(0.0, 2.0), Sample::new(1.0, 3.0), Sample::new(2.0, 2.0)]))
        .expect("B");
    let bounds = Bounds::new(point(px(0.0), px(0.0)), size(px(300.0), px(200.0)));
    for area in [false, true] {
        let plot = if area {
            CartesianPlot::new().area(AreaLayer::new("a")).area(AreaLayer::new("b"))
        } else {
            CartesianPlot::new().line(LineLayer::new("a")).line(LineLayer::new("b"))
        };
        let geometry = super::super::geometry::layout(&data, &plot, Edges::default(), bounds);
        for y in [1.0, 100.0, 199.0] {
            let selected = select(&geometry, &data, &slice_style(), point(px(140.0), px(y)), bounds);
            assert_eq!(selected.len(), 2);
            assert_eq!(selected[0].info.sample, Sample::new(1.0, 2.0));
            assert_eq!(selected[1].info.sample, Sample::new(1.0, 3.0));
        }
        assert_eq!(select(&geometry, &data, &slice_style(), point(px(270.0), px(100.0)), bounds)[0].info.sample.x, 2.0);
        assert!(select(&geometry, &data, &slice_style(), point(px(301.0), px(100.0)), bounds).is_empty());
    }
}

#[test]
fn missing_slices_do_not_snap_to_a_neighbor_or_invent_values() {
    let data = Dataset::new()
        .with_series(Series::new("a", "A", [Sample::new(0.0, 1.0), Sample::new(1.0, f64::NAN), Sample::new(2.0, 1.0)]))
        .expect("A");
    let bounds = Bounds::new(point(px(0.0), px(0.0)), size(px(300.0), px(200.0)));
    let plot = CartesianPlot::new().area(AreaLayer::new("a"));
    let geometry = super::super::geometry::layout(&data, &plot, Edges::default(), bounds);
    assert!(select(&geometry, &data, &slice_style(), point(px(150.0), px(100.0)), bounds).is_empty());
    let data = data
        .with_series(Series::new("b", "B", [Sample::new(0.0, 2.0), Sample::new(1.0, 3.0), Sample::new(2.0, 2.0)]))
        .expect("B");
    let geometry = super::super::geometry::layout(&data, &plot.area(AreaLayer::new("b")), Edges::default(), bounds);
    let selected = select(&geometry, &data, &slice_style(), point(px(150.0), px(100.0)), bounds);
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].info.label, "B");
}

struct HoverHarness {
    area: bool,
    slice: bool,
    created: Rc<RefCell<Vec<Sample>>>,
    events: Arc<Mutex<Vec<TooltipEvent>>>,
}
impl Render for HoverHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let mut data = Dataset::new()
            .with_series(Series::new("a", "A", [Sample::new(0.0, 0.0), Sample::new(1.0, 1.0), Sample::new(2.0, 0.0)]))
            .expect("dataset");
        if self.slice {
            data = data
                .with_series(Series::new(
                    "b",
                    "B",
                    [Sample::new(0.0, 0.5), Sample::new(1.0, 1.5), Sample::new(2.0, 0.5)],
                ))
                .expect("B");
        }
        let mut chart = CartesianChartBuilder::new(data)
            .presentation(if self.slice { slice_style() } else { style() })
            .axes([])
            .legend(LegendPresentation { visible: false, text_color: gpui::black() })
            .metrics(ChartMetrics { plot_padding: px(0.0), ..ChartMetrics::default() })
            .x_scale(NumericScale::new().domain(0.0..=2.0))
            .y_scale(NumericScale::new().domain(0.0..=2.0));
        chart = if self.area {
            chart.area(AreaLayer::new("a").interpolation(Interpolation::Monotone))
        } else {
            chart.line(LineLayer::new("a").interpolation(Interpolation::Monotone))
        };
        if self.slice {
            chart = if self.area {
                chart.area(AreaLayer::new("b"))
            } else {
                chart.line(LineLayer::new("b"))
            };
        }
        let created = self.created.clone();
        let events = self.events.clone();
        let chart = if self.slice {
            chart.slice_tooltip(move |slice| {
                created.borrow_mut().extend(slice.points.iter().map(|point| point.sample));
                let events = events.clone();
                Tooltip::new(format!("X {}: {} series", slice.coordinate, slice.points.len()))
                    .delay(Duration::ZERO)
                    .on_event(move |event| events.lock().expect("events").push(event))
            })
        } else {
            chart.point_tooltip(move |point| {
                created.borrow_mut().push(point.sample);
                let events = events.clone();
                Tooltip::new(format!("{} {}", point.label, point.sample.y))
                    .delay(Duration::ZERO)
                    .hide_delay(Duration::ZERO)
                    .on_event(move |event| events.lock().expect("events").push(event))
            })
        };
        div().id("hover-test").w(px(300.0)).h(px(200.0)).child(chart.build())
    }
}

#[gpui::test]
fn sdk_tooltip_is_presented_once_and_retained_across_redraws(cx: &mut TestAppContext) {
    for area in [false, true] {
        let (host, cx) = cx.add_window_view(|_, _| HoverHarness {
            area,
            slice: false,
            created: Rc::default(),
            events: Arc::default(),
        });
        cx.update(|window, _| window.activate_window());
        cx.simulate_mouse_move(point(px(150.0), px(100.0)), None, gpui::Modifiers::default());
        // The mock platform reports a zero pointer during bounds updates. Reapply
        // the simulated pointer before drawing each frame of the tooltip timeline.
        for _ in 0..3 {
            cx.executor().advance_clock(Duration::from_millis(1));
            cx.run_until_parked();
            cx.update(|window, cx| {
                window.simulate_mouse_move(point(px(150.0), px(100.0)), cx);
                window.draw(cx).clear(cx);
            });
        }
        host.read_with(cx, |host, _| {
            assert_eq!(*host.created.borrow(), [Sample::new(1.0, 1.0)]);
            assert!(host.events.lock().expect("events").iter().any(|event| matches!(event, TooltipEvent::Shown)));
        });
        cx.update(|window, _| window.refresh());
        cx.run_until_parked();
        host.read_with(cx, |host, _| assert_eq!(host.created.borrow().len(), 1));
        cx.update(|window, cx| {
            window.simulate_mouse_move(point(px(270.0), px(150.0)), cx);
            window.draw(cx).clear(cx);
        });
        cx.executor().advance_clock(Duration::from_millis(1));
        cx.run_until_parked();
        host.read_with(cx, |host, _| {
            assert!(host.events.lock().expect("events").iter().any(|event| matches!(event, TooltipEvent::Hidden)))
        });
    }
}

#[gpui::test]
fn slice_sdk_tooltip_opens_far_from_the_curve_and_tracks_horizontal_motion(cx: &mut TestAppContext) {
    for area in [false, true] {
        let (host, cx) = cx.add_window_view(|_, _| HoverHarness {
            area,
            slice: true,
            created: Rc::default(),
            events: Arc::default(),
        });
        cx.update(|window, _| window.activate_window());
        for _ in 0..3 {
            cx.executor().advance_clock(Duration::from_millis(1));
            cx.run_until_parked();
            cx.update(|window, cx| {
                window.simulate_mouse_move(point(px(20.0), px(10.0)), cx);
                window.draw(cx).clear(cx);
            });
        }
        host.read_with(cx, |host, _| {
            assert!(host.events.lock().expect("events").iter().any(|event| matches!(event, TooltipEvent::Shown)));
            assert_eq!(&host.created.borrow()[..2], &[Sample::new(0.0, 0.0), Sample::new(0.0, 0.5)]);
        });
        cx.update(|window, cx| {
            window.simulate_mouse_move(point(px(160.0), px(10.0)), cx);
            window.draw(cx).clear(cx);
        });
        host.read_with(cx, |host, _| {
            assert!(host.created.borrow().contains(&Sample::new(1.0, 1.0)));
            assert!(host.created.borrow().contains(&Sample::new(1.0, 1.5)));
        });
    }
}
