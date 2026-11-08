use std::sync::Arc;
use gpui::{App, Window, SharedString, ElementId, IntoElement, RenderOnce};
use crate::{Dataset, RadarPlot, RadarPresentation};

/// Original observation at a radar spoke, independent of animation progress.
#[derive(Clone, Debug, PartialEq)]
pub struct RadarPoint {
    pub series: crate::SeriesId,
    pub label: SharedString,
    pub sample: crate::Sample,
    pub sample_index: usize,
}
/// Category and accepted observations under the pointer, in series order.
#[derive(Clone, Debug, PartialEq)]
pub struct RadarSpoke {
    pub category: SharedString,
    pub category_index: usize,
    pub points: Vec<RadarPoint>,
}
pub(crate) type RadarTooltip = dyn Fn(&RadarSpoke) -> xgpui_luma::controls::tooltip::Tooltip;

type CategoryLabels = dyn Fn(usize, &SharedString) -> SharedString;

/// Native radar view builder. Host owns motion timing and supplies draw progress.
#[derive(Clone)]
pub struct RadarChartBuilder {
    pub(crate) data: Dataset,
    pub(crate) plot: RadarPlot,
    pub(crate) categories: Vec<SharedString>,
    pub(crate) style: Arc<dyn Fn() -> RadarPresentation>,
    pub(crate) labels: Option<Arc<CategoryLabels>>,
    pub(crate) progress: f32,
    pub(crate) tooltip: Option<Arc<RadarTooltip>>,
    pub(crate) events: Option<super::selection::SelectionEvents<crate::RadarSpoke>>,
    pub(crate) tooltip_motion: std::time::Duration,
    legend_interaction: crate::LegendInteraction,
    id: ElementId,
}
impl RadarChartBuilder {
    /// At least three categories are needed for a radar polygon.
    /// ```
    /// use xgpui_luma_plot::*;
    /// let data = Dataset::new().with_series(Series::new("a", "A", [
    ///     Sample::new(0.0, 20.0), Sample::new(1.0, 40.0), Sample::new(2.0, 30.0),
    /// ]))?;
    /// let view = RadarChartBuilder::new(data, "a").categories(["One", "Two", "Three"])
    ///     .maximum(50.0).draw_progress(0.5).build();
    /// # Ok::<(), DatasetError>(())
    /// ```
    pub fn new(data: Dataset, series: impl Into<crate::SeriesId>) -> Self {
        Self {
            data,
            plot: RadarPlot::new(series),
            categories: Vec::new(),
            style: Arc::new(RadarPresentation::default),
            labels: None,
            progress: 1.0,
            tooltip: None,
            events: None,
            tooltip_motion: std::time::Duration::ZERO,
            id: "radar-chart".into(),
            legend_interaction: Default::default(),
        }
    }
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
    pub fn categories(mut self, categories: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.categories = categories.into_iter().map(Into::into).collect();
        self
    }
    pub fn series(mut self, series: impl Into<crate::SeriesId>) -> Self {
        self.plot = self.plot.series(series);
        self
    }
    pub fn maximum(mut self, maximum: f64) -> Self {
        self.plot = self.plot.maximum(maximum);
        self
    }
    pub fn start_angle(mut self, degrees: f64) -> Self {
        self.plot = self.plot.start_angle(degrees);
        self
    }
    /// Resolve current theme appearance each render; host requests theme redraws.
    pub fn style(mut self, style: impl Fn() -> RadarPresentation + 'static) -> Self {
        self.style = Arc::new(style);
        self
    }
    /// Custom category text; can include values from the host dataset.
    pub fn category_labels(mut self, labels: impl Fn(usize, &SharedString) -> SharedString + 'static) -> Self {
        self.labels = Some(Arc::new(labels));
        self
    }
    /// Radial growth from the center, clamped to 0..=1; nonfinite restores one.
    pub fn draw_progress(mut self, progress: f32) -> Self {
        self.progress = if progress.is_finite() {
            progress.clamp(0.0, 1.0)
        } else {
            1.0
        };
        self
    }
    /// Tooltip for the nearest angular category inside the radar's outer grid.
    /// Includes all valid series values on that spoke. Configure delay on the SDK tooltip.
    pub fn spoke_tooltip(
        mut self,
        tooltip: impl Fn(&RadarSpoke) -> xgpui_luma::controls::tooltip::Tooltip + 'static,
    ) -> Self {
        self.tooltip = Some(Arc::new(tooltip));
        self
    }
    /// Animate the popup anchor between category spokes; default zero.
    pub fn tooltip_motion(mut self, duration: std::time::Duration) -> Self {
        self.tooltip_motion = duration;
        self
    }
    /// Hover changes and clicks carrying original observations; host owns selection.
    pub fn on_spoke_event(
        mut self,
        callback: impl Fn(crate::SelectionEvent<crate::RadarSpoke>, &mut App) + 'static,
    ) -> Self {
        self.events = Some(super::selection::SelectionEvents::new(callback));
        self
    }
    pub fn legend_interaction(mut self, interaction: crate::LegendInteraction) -> Self {
        self.legend_interaction = interaction;
        self
    }
    pub fn build(self) -> RadarChartView {
        RadarChartView { builder: self }
    }
}
#[derive(IntoElement)]
pub struct RadarChartView {
    builder: RadarChartBuilder,
}
impl RenderOnce for RadarChartView {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        use gpui::prelude::*;
        let style = (self.builder.style)();
        let targets: Vec<_> = self
            .builder
            .plot
            .series
            .iter()
            .filter(|id| self.builder.data.get(id).is_some())
            .map(|id| crate::LegendTarget::Series(id.clone()))
            .collect();
        let items = self
            .builder
            .plot
            .series
            .iter()
            .enumerate()
            .filter_map(|(index, id)| {
                let series = self.builder.data.get(id)?;
                Some(crate::LegendItem {
                    label: series.label().to_owned().into(),
                    swatch: crate::SwatchPresentation::Fill(style.series_style(id, index).dot_color.into()),
                })
            })
            .collect::<Vec<_>>();
        let interaction = self.builder.legend_interaction.clone();
        let legend = style.legend;
        let metrics = style.metrics;
        gpui::div()
            .id(self.builder.id.clone())
            .size_full()
            .flex()
            .flex_col()
            .gap(metrics.legend_gap)
            .child(
                gpui::div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .child(crate::render::radar::radar_canvas(self.builder, style)),
            )
            .when(legend.visible, |root| {
                root.child(
                    gpui::div().flex().justify_center().child(
                        crate::LegendView::new(items, legend, metrics).targets(targets).interaction(interaction),
                    ),
                )
            })
    }
}
