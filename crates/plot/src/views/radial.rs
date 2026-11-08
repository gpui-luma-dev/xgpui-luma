use std::{sync::Arc, time::Duration};
use gpui::{SharedString, ElementId, App, Window, RenderOnce, IntoElement, prelude::*};
use crate::{Dataset, RadialPlot, RadialLayout, RadialPresentation, Sample, SeriesId};
/// Source observation and its displayed share of the configured radial maximum.
#[derive(Clone, Debug, PartialEq)]
pub struct RadialBar {
    pub series: SeriesId,
    pub sample: Sample,
    pub sample_index: usize,
    pub label: SharedString,
    pub fraction: f64,
}
pub(crate) type RadialTooltip = dyn Fn(&RadialBar) -> xgpui_luma::controls::tooltip::Tooltip;
type RadialLabels = dyn Fn(&RadialBar) -> SharedString;
/// Native concentric bars or stacked arcs. Host owns timing and center text.
#[derive(Clone)]
pub struct RadialChartBuilder {
    pub(crate) data: Dataset,
    pub(crate) plot: RadialPlot,
    pub(crate) names: Vec<SharedString>,
    pub(crate) style: Arc<dyn Fn() -> RadialPresentation>,
    pub(crate) labels: Option<Arc<RadialLabels>>,
    pub(crate) center: Option<(SharedString, SharedString)>,
    pub(crate) progress: f32,
    pub(crate) tooltip: Option<Arc<RadialTooltip>>,
    pub(crate) events: Option<super::selection::SelectionEvents<crate::RadialBar>>,
    pub(crate) tooltip_motion: Duration,
    legend_interaction: crate::LegendInteraction,
    id: ElementId,
}
impl RadialChartBuilder {
    /// Finite positive weights preserve source order, names and original indices.
    /// ```
    /// use xgpui_luma_plot::*;
    /// let data = Dataset::new().with_series(Series::new("a","A",[
    ///     Sample::new(0.0,200.0), Sample::new(1.0,120.0),
    /// ]))?;
    /// let view = RadialChartBuilder::new(data,"a").bar_names(["Desktop","Mobile"])
    ///     .maximum(300.0).draw_progress(0.5).build();
    /// # Ok::<(),DatasetError>(())
    /// ```
    pub fn new(data: Dataset, series: impl Into<SeriesId>) -> Self {
        Self {
            data,
            plot: RadialPlot::new(series),
            names: Vec::new(),
            style: Arc::new(RadialPresentation::default),
            labels: None,
            center: None,
            progress: 1.0,
            tooltip: None,
            events: None,
            tooltip_motion: Duration::ZERO,
            id: "radial-chart".into(),
            legend_interaction: Default::default(),
        }
    }
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
    pub fn bar_names(mut self, names: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.names = names.into_iter().map(Into::into).collect();
        self
    }
    pub fn maximum(mut self, value: f64) -> Self {
        self.plot = self.plot.maximum(value);
        self
    }
    pub fn angles(mut self, start: f64, sweep: f64) -> Self {
        self.plot = self.plot.angles(start, sweep);
        self
    }
    pub fn layout(mut self, layout: RadialLayout) -> Self {
        self.plot = self.plot.layout(layout);
        self
    }
    /// Resolve appearance on every render; host requests theme redraws.
    pub fn style(mut self, style: impl Fn() -> RadialPresentation + 'static) -> Self {
        self.style = Arc::new(style);
        self
    }
    /// Optional text positioned inside each visible bar.
    pub fn bar_labels(mut self, labels: impl Fn(&RadialBar) -> SharedString + 'static) -> Self {
        self.labels = Some(Arc::new(labels));
        self
    }
    pub fn center_text(mut self, title: impl Into<SharedString>, caption: impl Into<SharedString>) -> Self {
        self.center = Some((title.into(), caption.into()));
        self
    }
    /// Angular growth; finite progress clamps to 0..=1 and nonfinite restores one.
    pub fn draw_progress(mut self, progress: f32) -> Self {
        self.progress = if progress.is_finite() {
            progress.clamp(0.0, 1.0)
        } else {
            1.0
        };
        self
    }
    /// SDK tooltip for a visible bar; configure delay on the returned tooltip.
    pub fn bar_tooltip(
        mut self,
        tooltip: impl Fn(&RadialBar) -> xgpui_luma::controls::tooltip::Tooltip + 'static,
    ) -> Self {
        self.tooltip = Some(Arc::new(tooltip));
        self
    }
    pub fn tooltip_motion(mut self, duration: Duration) -> Self {
        self.tooltip_motion = duration;
        self
    }
    /// Hover changes and clicks carrying original observations; host owns selection.
    pub fn on_bar_event(
        mut self,
        callback: impl Fn(crate::SelectionEvent<crate::RadialBar>, &mut App) + 'static,
    ) -> Self {
        self.events = Some(super::selection::SelectionEvents::new(callback));
        self
    }
    pub fn legend_interaction(mut self, interaction: crate::LegendInteraction) -> Self {
        self.legend_interaction = interaction;
        self
    }
    pub fn build(self) -> RadialChartView {
        RadialChartView { builder: self }
    }
}
#[derive(IntoElement)]
pub struct RadialChartView {
    builder: RadialChartBuilder,
}
impl RenderOnce for RadialChartView {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let style = (self.builder.style)();
        let targets: Vec<_> = crate::render::radial::observations(&self.builder)
            .iter()
            .map(|bar| crate::LegendTarget::Sample { series: bar.series.clone(), sample_index: bar.sample_index })
            .collect();
        let items = crate::render::radial::observations(&self.builder)
            .iter()
            .map(|bar| crate::LegendItem {
                label: bar.label.clone(),
                swatch: crate::SwatchPresentation::Fill(style.fill_for(bar.sample_index)),
            })
            .collect::<Vec<_>>();
        let interaction = self.builder.legend_interaction.clone();
        gpui::div()
            .id(self.builder.id.clone())
            .size_full()
            .flex()
            .flex_col()
            .gap(style.metrics.legend_gap)
            .child(gpui::div().flex_1().min_h_0().child(crate::render::radial::radial_canvas(self.builder)))
            .when(style.legend.visible, |root| {
                root.child(
                    crate::LegendView::new(items, style.legend, style.metrics)
                        .targets(targets)
                        .interaction(interaction),
                )
            })
    }
}
