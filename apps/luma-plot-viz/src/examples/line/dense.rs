use gpui::px;
use crate::theme::AppTheme;
use xgpui_luma_plot::{CartesianChartBuilder, Dataset, DatasetError, Decimation, LineLayer, NumericScale, Sample, Series};

use super::LineExample;
use crate::theme;

const SAMPLES_PER_SERIES: usize = 10_000;
const SAMPLE_INTERVAL_SECONDS: f64 = 0.1;

pub(super) fn example(look: &AppTheme) -> Result<LineExample, DatasetError> {
    Ok(LineExample {
        title: "Dense time series · 20,000 samples",
        description: "Two signals sampled at 10 Hz over nearly 17 minutes: slow trends, fine variation, isolated spikes, a step, and missing intervals. Full resolution, without point markers.",
        frame: theme::panel_frame(look, "plot-viz-dense-frame"),
        height: px(360.0),
        data: data()?,
        chart,
        diagnostics: None,
    })
}

// Construct once with the gallery; shared immutable samples survive subsequent renders.
fn data() -> Result<Dataset, DatasetError> {
    let mut a = Vec::with_capacity(SAMPLES_PER_SERIES);
    let mut b = Vec::with_capacity(SAMPLES_PER_SERIES);
    for index in 0..SAMPLES_PER_SERIES {
        let time = index as f64 * SAMPLE_INTERVAL_SECONDS;
        if (3_200..3_400).contains(&index) || (6_600..6_800).contains(&index) {
            a.push(Sample::new(time, f64::NAN));
            b.push(Sample::new(time, f64::NAN));
            continue;
        }
        let spike = if index % 1_200 == 400 {
            3.0
        } else if index % 1_700 == 700 {
            -2.5
        } else {
            0.0
        };
        let step = if index >= 5_000 { 0.75 } else { 0.0 };
        a.push(Sample::new(
            time,
            1.5 * (time / 25.0).sin() + 0.3 * (time / 2.3).sin() + 0.08 * (time * 0.9).sin() + spike + step,
        ));
        b.push(Sample::new(
            time,
            1.2 * (time / 30.0 + 0.7).sin() + 0.2 * (time / 1.9).cos() - 0.6 * spike - 0.3 * step,
        ));
    }
    Dataset::new()
        .with_series(Series::new("signal-a", "Signal A", a))?
        .with_series(Series::new("signal-b", "Signal B", b))
}

fn chart(data: Dataset, look: &AppTheme) -> CartesianChartBuilder {
    chart_with_decimation(data, look, Decimation::None)
}

pub(super) fn chart_with_decimation(data: Dataset, look: &AppTheme, decimation: Decimation) -> CartesianChartBuilder {
    CartesianChartBuilder::new(data)
        .style(theme::SdkCartesianStyle::new(look).series("signal-a", 0).series("signal-b", 1))
        .x_scale(NumericScale::new().tick_count(6))
        .y_scale(NumericScale::new().tick_count(5))
        .line(LineLayer::new("signal-a").decimation(decimation))
        .line(LineLayer::new("signal-b").decimation(decimation))
        .style_override(|style| {
            style.line.markers = None;
            for series in style.series.values_mut() {
                if let Some(line) = &mut series.line {
                    line.markers = None;
                }
            }
        })
        .style_override(|style| {
            style.axes[0].title = Some("Time (s)".into());
            style.axes[1].title = Some("Signal".into());
        })
}
