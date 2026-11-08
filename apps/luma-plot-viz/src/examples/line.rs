use super::{points, visitors};
mod dense;
mod decimated;
mod diagnostics;
mod scrolling;
mod interpolation;

use gpui::{App, Entity, IntoElement, Pixels, div, prelude::*, px};
use xgpui_luma::{controls::frame::FrameBuilder, vstack};
use crate::theme::AppTheme;
use xgpui_luma_plot::{
    Axis, AxisSide, CartesianChartBuilder, CategoryScale, Dataset, DatasetError, LineLayer, NumericScale, Sample,
    Series,
};

use crate::theme::{self, TypographyExt};

pub(crate) struct LineExample {
    title: &'static str,
    description: &'static str,
    frame: FrameBuilder,
    height: Pixels,
    data: Dataset,
    chart: fn(Dataset, &AppTheme) -> CartesianChartBuilder,
    diagnostics: Option<Entity<diagnostics::Diagnostics>>,
}

pub(crate) struct LineGallery {
    features: Entity<super::features::Features>,
    examples: Vec<LineExample>,
    scrolling: Entity<scrolling::ScrollingPlot>,
    interpolation: Entity<interpolation::InterpolationExample>,
    visitors: Entity<visitors::VisitorsExample>,
    points: Entity<points::PointExamples>,
}

/// Deterministic datasets that exercise the alpha line-chart API.
pub(crate) fn examples(look: &AppTheme, cx: &mut App) -> Result<LineGallery, DatasetError> {
    let mut raw = dense::example(look)?;
    let mut decimated = decimated::example(raw.data.clone(), look);
    raw.diagnostics = Some(diagnostics::Diagnostics::spawn(cx));
    decimated.diagnostics = Some(diagnostics::Diagnostics::spawn(cx));
    let scrolling = scrolling::ScrollingPlot::spawn(raw.data.clone(), look, cx);
    Ok(LineGallery {
        features: super::features::Features::spawn(look, cx)?,
        examples: vec![two_series(look)?, missing_samples()?, empty_selection(look)?, raw, decimated],
        scrolling,
        interpolation: interpolation::InterpolationExample::spawn(look, cx)?,
        visitors: visitors::VisitorsExample::spawn(look, false, cx)?,
        points: points::PointExamples::spawn(look, false, cx)?,
    })
}

pub(crate) fn gallery(gallery: &LineGallery, look: &AppTheme, progress: f32, cx: &mut App) -> impl IntoElement {
    super::motion::display(&gallery.features, progress, cx);
    super::motion::display(&gallery.scrolling, progress, cx);
    super::motion::display(&gallery.interpolation, progress, cx);
    super::motion::display(&gallery.visitors, progress, cx);
    super::motion::display(&gallery.points, progress, cx);
    vstack! { gap=20; }
        .w_full()
        .max_w(px(960.0))
        .children(gallery.examples.iter().map(|example| example_card(example, look, progress, cx)))
        .child(gallery.scrolling.clone())
        .child(gallery.interpolation.clone())
        .child(gallery.visitors.clone())
        .child(gallery.points.clone())
        .child(gallery.features.clone())
}

fn example_card(example: &LineExample, look: &AppTheme, progress: f32, cx: &App) -> impl IntoElement {
    let chrome = theme::chrome(look);
    let typography = theme::typography();
    let mut chart = (example.chart)(example.data.clone(), look);
    if let Some(diagnostics) = &example.diagnostics {
        chart = diagnostics::Diagnostics::observe(chart, diagnostics);
    }
    vstack! {
        gap=8;
        div().typography_style(typography.text.label).child(example.title),
        div()
            .typography_style(typography.text.caption)
            .text_color(chrome.muted_text)
            .child(example.description),
        example.frame.clone().h(example.height).p_4().render(cx).child(chart.draw_progress(progress).build()),
    }
    .w_full()
    .when_some(example.diagnostics.clone(), |this, diagnostics| {
        this.child(
            vstack! { gap=4;
                diagnostics,
                div().child("CPU sample updates on viewport, size, or count changes; paint measures submission, not GPU execution."),
            }
            .typography_style(typography.text.caption)
            .text_color(chrome.muted_text),
        )
    })
}

fn two_series(look: &AppTheme) -> Result<LineExample, DatasetError> {
    let data = Dataset::new()
        .with_series(Series::new(
            "temperature",
            "Temperature",
            [
                Sample::new(0.0, 18.0),
                Sample::new(1.0, 21.0),
                Sample::new(2.0, 19.0),
                Sample::new(3.0, 25.0),
                Sample::new(4.0, 22.0),
                Sample::new(5.0, 27.0),
            ],
        ))?
        .with_series(Series::new(
            "humidity",
            "Humidity",
            [
                Sample::new(0.0, 42.0),
                Sample::new(1.0, 46.0),
                Sample::new(2.0, 40.0),
                Sample::new(3.0, 51.0),
                Sample::new(4.0, 48.0),
                Sample::new(5.0, 56.0),
            ],
        ))?;
    Ok(LineExample {
        title: "Shared numeric coordinates",
        description: "Two series, legend entries, per-series presentation, and numeric tick labels.",
        frame: theme::panel_frame(look, "plot-viz-shared-coordinates-frame"),
        height: px(320.0),
        data,
        chart: two_series_chart,
        diagnostics: None,
    })
}

fn missing_samples() -> Result<LineExample, DatasetError> {
    let data = Dataset::new().with_series(Series::new(
        "temperature",
        "Temperature",
        [
            Sample::new(0.0, 18.0),
            Sample::new(1.0, 21.0),
            Sample::new(f64::NAN, f64::NAN),
            Sample::new(3.0, 25.0),
            Sample::new(4.0, 22.0),
            Sample::new(5.0, 27.0),
        ],
    ))?;
    Ok(LineExample {
        title: "Category points · right-side axis",
        description: "Ordered day labels and selected numeric ticks on the right. Wednesday keeps its position despite the missing observation.",
        frame: FrameBuilder::new("plot-viz-missing-samples-frame"),
        height: px(240.0),
        data,
        chart: missing_samples_chart,
        diagnostics: None,
    })
}

fn empty_selection(look: &AppTheme) -> Result<LineExample, DatasetError> {
    let data = Dataset::new().with_series(Series::new("forecast", "Forecast", []))?;
    Ok(LineExample {
        title: "No finite samples",
        description: "The plot keeps its axes and legend while the presentation supplies the empty-state treatment.",
        frame: theme::panel_frame(look, "plot-viz-empty-frame"),
        height: px(190.0),
        data,
        chart: empty_selection_chart,
        diagnostics: None,
    })
}

fn two_series_chart(data: Dataset, look: &AppTheme) -> CartesianChartBuilder {
    CartesianChartBuilder::new(data)
        .style(theme::SdkCartesianStyle::new(look).series("humidity", 1))
        .style_override(|style| {
            if let Some(line) = style.series.get_mut(&"humidity".into()).and_then(|series| series.line.as_mut()) {
                line.markers = None;
            }
        })
        .x_scale(NumericScale::new().tick_count(6))
        .y_scale(NumericScale::new().tick_count(5))
        .line(LineLayer::new("temperature"))
        .line(LineLayer::new("humidity"))
}

fn missing_samples_chart(data: Dataset, look: &AppTheme) -> CartesianChartBuilder {
    CartesianChartBuilder::new(data)
        .style(theme::SdkCartesianStyle::new(look))
        .x_scale(CategoryScale::new(["Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]))
        .y_scale(NumericScale::new().tick_values([18.0, 21.0, 24.0, 27.0]))
        .line(LineLayer::new("temperature"))
        .style_override(|style| {
            let axis = style.axes[0].presentation.clone();
            style.axes = vec![
                Axis::new(AxisSide::Bottom, axis.clone()).title("Day"),
                Axis::new(AxisSide::Right, axis).title("°C"),
            ];
        })
}

fn empty_selection_chart(data: Dataset, look: &AppTheme) -> CartesianChartBuilder {
    CartesianChartBuilder::new(data)
        .style(theme::SdkCartesianStyle::new(look))
        .x_scale(NumericScale::new().tick_count(3))
        .y_scale(NumericScale::new().tick_count(3))
        .line(LineLayer::new("forecast"))
}
