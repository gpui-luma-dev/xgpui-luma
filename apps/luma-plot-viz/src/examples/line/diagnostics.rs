use gpui::{App, Context, Entity, IntoElement, Render, Window, div, prelude::*};
use xgpui_luma_plot::{CartesianChartBuilder, ChartRenderStats};

/// Keep one sample per plot size/count configuration; timing jitter must not drive redraws.
#[derive(Default)]
pub(super) struct Diagnostics {
    stats: Option<ChartRenderStats>,
}

impl Diagnostics {
    pub(super) fn spawn(cx: &mut App) -> Entity<Self> {
        cx.new(|_| Self::default())
    }

    pub(super) fn plot_width(&self) -> Option<f32> {
        self.stats.map(|stats| stats.plot_width)
    }

    pub(super) fn record(&mut self, stats: ChartRenderStats) -> bool {
        let changed = self.stats.is_none_or(|previous| {
            previous.plot_width != stats.plot_width
                || previous.plot_offset_x != stats.plot_offset_x
                || previous.plot_height != stats.plot_height
                || previous.source_points != stats.source_points
                || previous.valid_points != stats.valid_points
                || previous.retained_points != stats.retained_points
                || previous.runs != stats.runs
                || previous.visible_points != stats.visible_points
                || previous.x_domain != stats.x_domain
        });
        if changed {
            self.stats = Some(stats);
        }
        changed
    }

    pub(super) fn observe(chart: CartesianChartBuilder, entity: &Entity<Self>) -> CartesianChartBuilder {
        let weak = entity.downgrade();
        chart.on_render_stats(move |stats, cx| {
            let weak = weak.clone();
            cx.defer(move |cx| {
                let _ = weak.update(cx, |this, cx| {
                    if this.record(stats) {
                        cx.notify();
                    }
                });
            });
        })
    }
}

impl Render for Diagnostics {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let text = match self.stats {
            None => "Render measurements pending…".to_owned(),
            Some(stats) => {
                let reduction = if stats.visible_points == 0 {
                    0.0
                } else {
                    100.0 * (1.0 - stats.retained_points as f64 / stats.visible_points as f64)
                };
                format!(
                    "Stored: {} · Valid: {} · Visible + neighbors: {} · Retained: {} ({:.1}% reduction) · Runs: {} · Plot: {:.0}px · Prepare CPU: {:.3}ms · Paint CPU: {:.3}ms",
                    stats.source_points,
                    stats.valid_points,
                    stats.visible_points,
                    stats.retained_points,
                    reduction,
                    stats.runs,
                    stats.plot_width,
                    stats.prepare_time.as_secs_f64() * 1000.0,
                    stats.paint_time.as_secs_f64() * 1000.0,
                )
            }
        };
        div().child(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn timing_jitter_does_not_request_another_frame_but_resize_does() {
        let mut diagnostics = Diagnostics::default();
        let mut stats = ChartRenderStats {
            source_points: 20_000,
            valid_points: 19_200,
            visible_points: 19_200,
            x_domain: (0.0, 999.9),
            retained_points: 4_000,
            runs: 6,
            plot_width: 800.0,
            plot_offset_x: 40.0,
            plot_height: 300.0,
            prepare_time: Duration::ZERO,
            paint_time: Duration::ZERO,
        };
        assert!(diagnostics.record(stats));
        stats.prepare_time = Duration::from_millis(2);
        stats.paint_time = Duration::from_millis(1);
        assert!(!diagnostics.record(stats));
        stats.plot_width = 400.0;
        assert!(diagnostics.record(stats));
        assert!(!diagnostics.record(stats));
        stats.retained_points = 2_000;
        assert!(diagnostics.record(stats));
        stats.x_domain = (120.0, 240.0);
        assert!(diagnostics.record(stats));
        assert!(!diagnostics.record(stats));
    }
}
