use xgpui_luma::controls::scrollbar;
use xgpui_luma::controls::scrollbar::ScrollbarEventExt;
use gpui::{App, Context, Entity, FocusHandle, Render, Window, ScrollEvent, MouseButton, div, prelude::*, px};
use xgpui_luma::{
    vstack,
    controls::scrollbar::{Scrollbar, ScrollbarEvent},
    key_handling::{
        ControlKeyProfile, DecreaseValue, IncreaseValue, DecreaseValueLarge, IncreaseValueLarge, MoveToStart, MoveToEnd,
    },
};
use crate::theme::AppTheme;
use xgpui_luma_plot::{BarLayer, BarLayout, CartesianChartBuilder, Dataset, DatasetError, NumericScale, Sample, Series};
use crate::theme::{self, TypographyExt};

const COUNT: usize = 100_000;
const SPAN: f64 = 80.0;
const MIN: f64 = -0.5;
const MAX: f64 = COUNT as f64 - 0.5;

pub(super) struct ScrollingBars {
    entrance: f32,
    data: Dataset,
    look: AppTheme,
    start: f64,
    wheel_position: f64,
    layout: Option<(f32, f32)>,
    retained: usize,
    scrollbar: Entity<Scrollbar>,
    focus: FocusHandle,
}
impl ScrollingBars {
    pub(super) fn spawn(look: &AppTheme, cx: &mut App) -> Result<Entity<Self>, DatasetError> {
        let data = Dataset::new().with_series(Series::new("desktop", "Observations", observations(COUNT, 17)))?;
        let look = look.clone();
        Ok(cx.new(move |cx: &mut Context<Self>| {
            let scrollbar = scrollbar::Scrollbar::new("plot-viz-time-scrollbar")
                .horizontal()
                .step(0.1)
                .page_step(SPAN * 0.8)
                .viewport(MIN, MAX, MIN, MIN + SPAN)
                .template(std::sync::Arc::new(scrollbar::ThemedScrollbarTemplate::new(std::sync::Arc::new(
                    look.clone(),
                ))))
                .spawn(cx);
            scrollbar.clone().on_event(cx, |this, event: &ScrollbarEvent, cx| {
                if let ScrollbarEvent::Change { value } = event {
                    this.start = f64::from(*value);
                    this.wheel_position = this.start;
                    cx.notify();
                }
            });
            Self {
                entrance: 1.0,
                data,
                look,
                start: MIN,
                wheel_position: MIN,
                layout: None,
                retained: 0,
                scrollbar,
                focus: cx.focus_handle().tab_stop(true),
            }
        }))
    }
    fn set_start(&mut self, requested: f64, cx: &mut Context<Self>) {
        let start = self.scrollbar.update(cx, |bar, cx| {
            bar.set_value(requested, cx);
            f64::from(bar.value())
        });
        self.wheel_position = start;
        if self.start != start {
            self.start = start;
            cx.notify();
        }
    }
    fn pan(&mut self, amount: f64, cx: &mut Context<Self>) {
        self.set_start(self.start + amount, cx);
    }
    fn wheel(&mut self, event: &ScrollEvent, window: &mut Window, cx: &mut Context<Self>) {
        let Some((_, width)) = self.layout else {
            return;
        };
        let delta = event.delta.pixel_delta(window.line_height());
        if let Some(amount) = wheel_samples(f32::from(delta.x), f32::from(delta.y), event.modifiers.shift, width) {
            let requested = (self.wheel_position + amount).clamp(MIN, MAX - SPAN);
            self.set_start(requested, cx);
            self.wheel_position = requested;
            cx.stop_propagation();
        }
    }
}
fn observations(count: usize, mut seed: u32) -> Vec<Sample> {
    (0..count)
        .map(|index| {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            Sample::new(index as f64, 80.0 + f64::from((seed >> 8) % 460))
        })
        .collect()
}

fn scrolling_ticks(start: f64) -> Vec<(f64, String)> {
    (0..6)
        .map(|index| {
            // Keep viewport fractions exact; round only the displayed observation number.
            let coordinate = start + SPAN * index as f64 / 5.0;
            let observation = coordinate.round().clamp(0.0, COUNT as f64 - 1.0) as usize + 1;
            (coordinate, observation.to_string())
        })
        .collect()
}

fn wheel_samples(x: f32, y: f32, shift: bool, width: f32) -> Option<f64> {
    if ![x, y, width].into_iter().all(f32::is_finite) || width <= 0.0 {
        return None;
    }
    let amount = if shift && x == 0.0 { y } else { x };
    if amount == 0.0 || (!shift && y.abs() > x.abs()) {
        return None;
    }
    Some(-f64::from(amount) * SPAN / f64::from(width))
}
impl Render for ScrollingBars {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let typography = theme::typography();
        let chrome = theme::chrome(&self.look);
        let tooltip_look = self.look.clone();
        let weak = cx.weak_entity();
        let start = self.start;
        let end = start + SPAN;
        let chart = CartesianChartBuilder::new(self.data.clone())
            .id("large-scrolling-bars-chart")
            .style(theme::SdkCartesianStyle::new(&self.look).blue())
            .x_scale(NumericScale::new().domain(start..=end).tick_labels(scrolling_ticks(start)))
            .y_scale(NumericScale::new().domain(0.0..=600.0).tick_count(5))
            .bar_layout(BarLayout::new().width(0.8))
            .bar(BarLayer::new("desktop"))
            .bar_tooltip(move |point| {
                theme::points::tooltip(&tooltip_look, point, format!("Observation {}", point.sample.x as usize + 1))
                    .delay(std::time::Duration::from_millis(250))
            })
            .tooltip_motion(std::time::Duration::from_millis(250))
            .style_override(|style| {
                style.grid.vertical = None;
                style.legend.visible = false;
                // Reserve fixed label slots without changing the theme typography.
                style.metrics.plot_padding = px(34.0);
                for axis in &mut style.axes {
                    axis.presentation.label_width = Some(px(if axis.side == xgpui_luma_plot::AxisSide::Bottom {
                        68.0
                    } else {
                        40.0
                    }));
                    axis.presentation.label_align = gpui::TextAlign::Right;
                    if axis.side == xgpui_luma_plot::AxisSide::Bottom {
                        axis.presentation.inset_end_labels = true;
                        axis.presentation.gutter = px(32.0);
                        axis.presentation.label_gap = px(6.0);
                    }
                }
            })
            .on_render_stats(move |stats, cx| {
                let weak = weak.clone();
                cx.defer(move |cx| {
                    let _ = weak.update(cx, |this, cx| {
                        let layout = Some((stats.plot_offset_x, stats.plot_width));
                        if this.layout != layout {
                            this.layout = layout;
                            this.scrollbar
                                .update(cx, |bar, cx| bar.set_length(f64::from(stats.plot_width.max(1.0)), cx));
                            cx.notify();
                        }
                        if this.retained != stats.retained_points {
                            this.retained = stats.retained_points;
                            cx.notify();
                        }
                    });
                });
            });
        let viewport = div()
            .id("large-bars-viewport")
            .w_full()
            .flex_1()
            .min_h_0()
            .track_focus(&self.focus)
            .key_context(ControlKeyProfile::ScrollOffset.context())
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    this.focus.focus(window, cx);
                    cx.stop_propagation();
                }),
            )
            .on_scroll(cx.listener(Self::wheel))
            .on_action(cx.listener(|this, _: &DecreaseValue, _, cx| this.pan(-1.0, cx)))
            .on_action(cx.listener(|this, _: &IncreaseValue, _, cx| this.pan(1.0, cx)))
            .on_action(cx.listener(|this, _: &DecreaseValueLarge, _, cx| this.pan(-SPAN * 0.8, cx)))
            .on_action(cx.listener(|this, _: &IncreaseValueLarge, _, cx| this.pan(SPAN * 0.8, cx)))
            .on_action(cx.listener(|this, _: &MoveToStart, _, cx| this.set_start(MIN, cx)))
            .on_action(cx.listener(|this, _: &MoveToEnd, _, cx| this.set_start(MAX - SPAN, cx)))
            .child(chart.draw_progress(self.entrance).build());
        vstack! { gap=8;
            div().typography_style(typography.text.label).child("Large dataset · scrolling bars"),
            div().typography_style(typography.text.caption).text_color(chrome.muted_text)
                .child("100,000 stored observations, an 80-observation viewport, and fixed value bounds. Only visible bars are projected."),
            theme::panel_frame(&self.look, "large-bars-frame").h(px(360.0)).p_4().render(cx).child(
                vstack! { gap=8; viewport, div().pl(px(self.layout.map_or(0.0, |layout| layout.0))).child(self.scrollbar.clone()) }.size_full()
            ),
            div().typography_style(typography.text.caption).text_color(chrome.muted_text)
                .child(format!("Stored: 100,000 · Drawing: {} · Swipe horizontally or Shift-wheel; click the chart for arrow, Page Up/Down, and Home/End keys.", self.retained)),
        }.w_full()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scrolling_labels_keep_viewport_fractions_without_text_padding() {
        for start in [MIN, 0.1, 8.9, 9.5, 99.5, 999.5, MAX - SPAN] {
            let ticks = scrolling_ticks(start);
            assert_eq!(ticks.len(), 6);
            for (index, (coordinate, label)) in ticks.iter().enumerate() {
                assert_eq!(*label, label.trim());
                assert!(((coordinate - start) / SPAN - index as f64 / 5.0).abs() < 1e-10);
                let number = label.trim().parse::<usize>().unwrap();
                assert!((1..=COUNT).contains(&number));
            }
        }
    }
    #[test]
    fn wheel_preserves_page_scrolling_and_maps_horizontal_distance() {
        assert_eq!(wheel_samples(0.0, -20.0, false, 800.0), None);
        assert_eq!(wheel_samples(-20.0, 0.0, false, 800.0), Some(2.0));
        assert_eq!(wheel_samples(0.0, -20.0, true, 800.0), Some(2.0));
        assert_eq!(wheel_samples(-20.0, 0.0, false, 0.0), None);
    }
}

#[cfg(test)]
mod state_tests {
    use super::*;
    #[gpui::test]
    fn scrollbar_clamps_at_both_ends_without_replacing_large_dataset(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            let view = ScrollingBars::spawn(&AppTheme::new(), cx).unwrap();
            view.update(cx, |this, cx| {
                let stored = this.data.series()[0].samples().as_ptr();
                this.set_start(MAX * 2.0, cx);
                assert_eq!(this.start, MAX - SPAN);
                assert_eq!(f64::from(this.scrollbar.read(cx).value()), this.start);
                this.set_start(-100.0, cx);
                assert_eq!(this.start, MIN);
                this.pan(10.0, cx);
                assert!((this.start - 9.5).abs() < 1e-4);
                assert_eq!(this.data.series()[0].samples().as_ptr(), stored);
                assert_eq!(this.data.series()[0].samples().len(), COUNT);
            });
        });
    }
}

crate::examples::motion::entrance!(ScrollingBars);
