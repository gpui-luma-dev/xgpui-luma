use xgpui_luma::controls::scrollbar;
mod input;

use xgpui_luma::controls::scrollbar::ScrollbarEventExt;
use gpui::{App, Context, Entity, FocusHandle, IntoElement, MouseButton, Render, Window, div, prelude::*, px};
use xgpui_luma::{
    controls::scrollbar::{Scrollbar, ScrollbarEvent},
    key_handling::{
        ControlKeyProfile, DecreaseValue, DecreaseValueLarge, IncreaseValue, IncreaseValueLarge, MoveToStart, MoveToEnd,
    },
    vstack,
};
use crate::theme::AppTheme;
use xgpui_luma_plot::{Dataset, Decimation, NumericScale};

use super::{dense, diagnostics::Diagnostics};
use crate::theme::{self, TypographyExt};

const WINDOW_SECONDS: f64 = 120.0;

pub(super) struct ScrollingPlot {
    entrance: f32,
    data: Dataset,
    look: AppTheme,
    start: f64,
    content: (f64, f64),
    plot_layout: Option<(f32, f32)>,
    wheel_position: f64,
    focus: FocusHandle,
    scrollbar: Entity<Scrollbar>,
    diagnostics: Entity<Diagnostics>,
}

impl ScrollingPlot {
    pub(super) fn spawn(data: Dataset, look: &AppTheme, cx: &mut App) -> Entity<Self> {
        let look = look.clone();
        cx.new(move |cx: &mut Context<Self>| {
            let content = time_domain(&data).unwrap_or((0.0, WINDOW_SECONDS));
            let scrollbar = scrollbar::Scrollbar::new("plot-viz-time-scrollbar")
                .horizontal()
                .step(0.1)
                .page_step(WINDOW_SECONDS * 0.8)
                .viewport(content.0, content.1, content.0, content.0 + WINDOW_SECONDS)
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
                start: content.0,
                content,
                plot_layout: None,
                wheel_position: content.0,
                focus: cx.focus_handle().tab_stop(true),
                scrollbar,
                diagnostics: Diagnostics::spawn(cx),
            }
        })
    }
}

impl Render for ScrollingPlot {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let typography = theme::typography();
        let chrome = theme::chrome(&self.look);
        let end = self.start + WINDOW_SECONDS;
        let chart = dense::chart_with_decimation(self.data.clone(), &self.look, Decimation::MinMax)
            .x_scale(NumericScale::new().domain(self.start..=end).tick_count(6));
        let view = cx.weak_entity();
        let chart = chart.on_render_stats(move |stats, cx| {
            let view = view.clone();
            cx.defer(move |cx| {
                let _ = view.update(cx, |this, cx| {
                    this.diagnostics.update(cx, |diagnostics, cx| {
                        if diagnostics.record(stats) {
                            cx.notify();
                        }
                    });
                    this.scrollbar.update(cx, |bar, cx| {
                        // Use the SDK's length and viewport setters, as ScrollContainer does.
                        bar.set_length(f64::from(stats.plot_width.max(1.0)), cx);
                        bar.set_viewport(this.content.0, this.content.1, this.start, this.start + WINDOW_SECONDS, cx);
                    });
                    let layout = Some((stats.plot_offset_x, stats.plot_width));
                    if this.plot_layout != layout {
                        this.plot_layout = layout;
                        cx.notify();
                    }
                });
            });
        });
        let plot = vstack! { chart.draw_progress(self.entrance).build() }
            .id("plot-viz-scrolling-viewport")
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
            .on_scroll(cx.listener(Self::scroll_wheel))
            .on_action(cx.listener(|this, _: &DecreaseValue, _, cx| this.pan(-6.0, cx)))
            .on_action(cx.listener(|this, _: &IncreaseValue, _, cx| this.pan(6.0, cx)))
            .on_action(cx.listener(|this, _: &DecreaseValueLarge, _, cx| this.pan(-WINDOW_SECONDS * 0.8, cx)))
            .on_action(cx.listener(|this, _: &IncreaseValueLarge, _, cx| this.pan(WINDOW_SECONDS * 0.8, cx)))
            .on_action(cx.listener(|this, _: &MoveToStart, _, cx| {
                let start = f64::from(this.scrollbar.read(cx).range().start);
                this.set_start(start, cx);
            }))
            .on_action(cx.listener(|this, _: &MoveToEnd, _, cx| {
                let end = f64::from(this.scrollbar.read(cx).range().end);
                this.set_start(end, cx);
            }));
        let plot_frame =
            theme::panel_frame(&self.look, "plot-viz-scrolling-frame").h(px(360.0)).p_4().render(cx).child(
                vstack! { gap=8;
                    plot,
                    vstack! { self.scrollbar.clone() }
                        .pl(px(self.plot_layout.map_or(0.0, |layout| layout.0)))
                        .flex_shrink_0(),
                }
                .size_full(),
            );
        vstack! { gap=8;
            div().typography_style(typography.text.label).child("Scrolling time series · 120-second viewport"),
            div().typography_style(typography.text.caption).text_color(chrome.muted_text)
                .child("Move through the same 20,000 samples with the horizontal scrollbar. Y bounds stay fixed; only visible segments and edge neighbors reach decimation."),
            plot_frame,
            div().typography_style(typography.text.caption).text_color(chrome.muted_text)
                .child(format!("Time window: {:.1}–{:.1}s · Swipe horizontally over the plot; click the plot, then use arrow keys.", self.start, end)),
            div().typography_style(typography.text.caption).text_color(chrome.muted_text).child(self.diagnostics.clone()),
        }.w_full()
    }
}

/// Time extent includes finite X coordinates in missing intervals, preserving their space.
fn time_domain(data: &Dataset) -> Option<(f64, f64)> {
    let mut xs = data
        .series()
        .iter()
        .flat_map(|series| series.samples())
        .map(|sample| sample.x)
        .filter(|x| x.is_finite());
    let first = xs.next()?;
    Some(xs.fold((first, first), |(min, max), x| (min.min(x), max.max(x))))
}

#[cfg(test)]
mod tests;

crate::examples::motion::entrance!(ScrollingPlot);
