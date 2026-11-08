use crate::{
    data::{Dataset, SeriesId},
    plots::{BarLayout, BarOrientation, BarLayer, AreaLayer, CartesianPlot, LineLayer, Scale},
    presentation::{
        BarPresentation, AreaPresentation, Axis, CartesianPresentation, ChartMetrics, EmptyStatePresentation,
        GridPresentation, LegendPresentation, LinePresentation, CartesianStyle, CartesianStyleProvider, SeriesStyle,
    },
};

use super::{
    CartesianChartView, ChartRenderStats, RenderObserver, PlotPoint, PlotSlice, PointTooltip, PlotPointEvent,
    PointEvents, PointTooltipFactory, StyleSource,
};

/// Compose chart content and focused visuals without constructing a separate plot.
///
/// Defaults show bottom/left axes, grid lines, and a legend. Bind one `CartesianStyle`
/// or live provider with `style`, then customize it with `style_override` and
/// `series_style`. Providers resolve each render before local overrides.
/// Frame decoration and chart headings remain ordinary host composition.
#[derive(Clone, Debug)]
pub struct CartesianChartBuilder {
    annotations: Vec<crate::ChartAnnotation>,
    legend_interaction: crate::LegendInteraction,
    id: gpui::ElementId,
    data: Dataset,
    plot: CartesianPlot,
    style: StyleSource,
    observer: Option<RenderObserver>,
    draw_progress: f32,
    point_tooltip: Option<PointTooltipFactory>,
    point_events: Option<PointEvents>,
    tooltip_motion: std::time::Duration,
    bar_options: super::BarViewOptions,
}

impl CartesianChartBuilder {
    pub fn new(data: Dataset) -> Self {
        Self {
            id: "cartesian-chart".into(),
            legend_interaction: Default::default(),
            annotations: Vec::new(),
            data,
            plot: CartesianPlot::new(),
            style: StyleSource::default(),
            observer: None,
            draw_progress: 1.0,
            point_tooltip: None,
            point_events: None,
            tooltip_motion: std::time::Duration::ZERO,
            bar_options: super::BarViewOptions::default(),
        }
    }

    /// Set a stable chart identity for retained interaction and tooltip state.
    /// Give sibling charts distinct IDs when they share a host view or gallery.
    pub fn id(mut self, id: impl Into<gpui::ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// Configure the X mapping and tick policy independently of visible axis edges.
    pub fn x_scale(mut self, scale: impl Into<Scale>) -> Self {
        self.plot = self.plot.x_scale(scale);
        self
    }

    /// Configure the Y mapping and tick policy independently of visible axis edges.
    pub fn y_scale(mut self, scale: impl Into<Scale>) -> Self {
        self.plot = self.plot.y_scale(scale);
        self
    }

    /// Add a line layer selecting a series by stable identity.
    pub fn line(mut self, layer: LineLayer) -> Self {
        self.plot = self.plot.line(layer);
        self
    }

    /// Add a filled area; interpolation and decimation reuse the line boundary rules.
    /// ```
    /// use xgpui_luma_plot::{AreaLayer, CartesianChartBuilder, Dataset, Interpolation};
    /// let chart = CartesianChartBuilder::new(Dataset::new())
    ///     .area(AreaLayer::new("level").baseline(0.0).interpolation(Interpolation::Monotone))
    ///     .build();
    /// ```
    pub fn area(mut self, layer: AreaLayer) -> Self {
        self.plot = self.plot.area(layer);
        self
    }

    /// Add bars selecting one series. X is the category and Y is the value in
    /// every orientation. Category coordinates are zero-based indices.
    pub fn bar(mut self, layer: BarLayer) -> Self {
        self.plot = self.plot.bar(layer);
        self
    }

    /// Set the shared grouping, width, gap, and category alignment.
    /// ```
    /// use xgpui_luma_plot::{BarAlignment, BarGrouping, BarLayout, BarOrientation,
    ///     BarLayer, CartesianChartBuilder, CategoryScale, Dataset, Sample, Series};
    /// let data = Dataset::new()
    ///     .with_series(Series::new("sales", "Sales", [Sample::new(0.0, 12.0)]))?;
    /// let chart = CartesianChartBuilder::new(data)
    ///     .x_scale(CategoryScale::new(["Q1"]))
    ///     .bar_layout(BarLayout::new().width(0.6).gap(0.15)
    ///         .alignment(BarAlignment::Center).grouping(BarGrouping::Grouped))
    ///     .bar_orientation(BarOrientation::Right)
    ///     .bar(BarLayer::new("sales"))
    ///     .build();
    /// # Ok::<(), xgpui_luma_plot::DatasetError>(())
    /// ```
    pub fn bar_layout(mut self, layout: BarLayout) -> Self {
        self.plot = self.plot.bar_layout(layout);
        self
    }
    /// Set increasing-value direction. Samples/scales retain category X and value Y.
    /// Intended for bar-only charts; other layers keep their Cartesian mapping.
    pub fn bar_orientation(mut self, orientation: BarOrientation) -> Self {
        self.plot = self.plot.bar_orientation(orientation);
        self
    }
    /// Supply a GPUI element template for each resolved bar segment.
    /// ```
    /// use gpui::{div, prelude::*};
    /// use xgpui_luma_plot::{BarRenderModel, CartesianChartBuilder, Dataset};
    /// let chart = CartesianChartBuilder::new(Dataset::new())
    ///     .bar_template(|bar: &BarRenderModel, _: &mut gpui::Window, _: &mut gpui::App| {
    ///         div().w(bar.bounds.size.width).h(bar.bounds.size.height)
    ///             .rounded_md().bg(bar.presentation.fill)
    ///     }).build();
    /// ```
    pub fn bar_template(mut self, template: impl super::BarTemplate) -> Self {
        self.bar_options.template = Some(super::BarTemplateSource(std::sync::Arc::new(template)));
        self
    }

    /// Provide optional custom and numeric labels for each resolved bar segment.
    /// Labels are independent of the shape template; defaults are unlabeled.
    /// Inside labels that do not fit are omitted. Outside labels stay within the
    /// chart canvas; reserve endpoint room with the value domain when needed.
    /// ```
    /// use xgpui_luma_plot::{BarLabel, BarLabelPlacement, CartesianChartBuilder, Dataset, TextPresentation};
    /// let chart = CartesianChartBuilder::new(Dataset::new())
    ///     .bar_labels(|bar| vec![
    ///         BarLabel::new(format!("Item {}", bar.sample_index + 1), TextPresentation::new(gpui::black()))
    ///             .placement(BarLabelPlacement::InsideStart),
    ///         BarLabel::new(bar.sample.y.to_string(), TextPresentation::new(gpui::white())),
    ///     ]).build();
    /// ```
    pub fn bar_labels(mut self, labels: impl Fn(&super::BarRenderModel) -> Vec<crate::BarLabel> + 'static) -> Self {
        self.bar_options.labels = Some(super::BarLabelSource(std::sync::Arc::new(labels)));
        self
    }
    /// Show each source value outside its bar endpoint in every orientation.
    pub fn bar_value_labels(self, text_style: crate::TextPresentation) -> Self {
        self.bar_labels(move |bar| vec![crate::BarLabel::new(bar.sample.y.to_string(), text_style.clone())])
    }
    /// Attach an SDK tooltip to a bar's rectangular layout bounds. Gaps and clipped
    /// portions cannot activate it; custom shapes retain their layout hit region.
    /// Uses `tooltip_motion` for the anchor transition. Takes precedence over
    /// point tooltips when both are configured.
    /// ```
    /// use xgpui_luma_plot::{CartesianChartBuilder, Dataset};
    /// use xgpui_luma::controls::tooltip::Tooltip;
    /// let chart = CartesianChartBuilder::new(Dataset::new())
    ///     .bar_tooltip(|bar| Tooltip::new(format!("{}: {}", bar.label, bar.sample.y)))
    ///     .tooltip_motion(std::time::Duration::from_millis(250)).build();
    /// ```
    pub fn bar_tooltip(
        mut self,
        tooltip: impl Fn(&PlotPoint) -> xgpui_luma::controls::tooltip::Tooltip + 'static,
    ) -> Self {
        self.bar_options.tooltip = Some(PointTooltipFactory::Point(std::sync::Arc::new(tooltip)));
        self
    }

    /// Override one bar series' fill.
    pub fn bar_style(self, series: impl Into<SeriesId>, style: BarPresentation) -> Self {
        self.series_style(series, style)
    }

    /// Override fill and optional outline independently of the area's data mapping.
    pub fn area_style(self, series: impl Into<SeriesId>, style: AreaPresentation) -> Self {
        self.series_style(series, style)
    }

    /// Override one series' visual treatment without adding or changing a layer.
    pub fn line_style(self, series: impl Into<SeriesId>, style: LinePresentation) -> Self {
        self.series_style(series, style)
    }

    /// Replace visible axis components; an empty collection hides all axes.
    pub fn axes(mut self, axes: impl IntoIterator<Item = Axis>) -> Self {
        let axes: Vec<_> = axes.into_iter().collect();
        self.style.customize(move |style| {
            let resolved = CartesianPresentation::from(style.clone()).axes(axes.clone());
            style.axes = resolved.axis_components().to_vec();
        });
        self
    }

    pub fn grid(mut self, grid: GridPresentation) -> Self {
        self.style.customize(move |style| style.grid = grid);
        self
    }

    pub fn legend(mut self, legend: LegendPresentation) -> Self {
        self.style.customize(move |style| style.legend = legend);
        self
    }

    pub fn empty_state(mut self, empty_state: Option<EmptyStatePresentation>) -> Self {
        self.style.customize(move |style| style.empty_state = empty_state.clone());
        self
    }

    pub fn metrics(mut self, metrics: ChartMetrics) -> Self {
        self.style.customize(move |style| style.metrics = metrics);
        self
    }

    /// Advanced composition: replace the plot specification, preserving visuals.
    pub fn plot(mut self, plot: CartesianPlot) -> Self {
        self.plot = plot;
        self
    }

    /// Theme/advanced composition: replace all visuals, preserving data and mapping.
    /// Apply this before focused visual overrides; later calls replace earlier values.
    pub fn presentation(mut self, presentation: CartesianPresentation) -> Self {
        self.style = StyleSource::new(CartesianStyle::from(presentation));
        self
    }

    /// Bind a static style or a provider resolved on every render. Replaces the
    /// previous source and its overrides; apply local customization afterward.
    /// ```
    /// use xgpui_luma_plot::{CartesianChartBuilder, Dataset, CartesianStyle};
    /// let chart = CartesianChartBuilder::new(Dataset::new())
    ///     .style(CartesianStyle::default())
    ///     .style_override(|style| style.line.stroke.width = gpui::px(3.0))
    ///     .build();
    /// ```
    pub fn style(mut self, provider: impl CartesianStyleProvider) -> Self {
        self.style = StyleSource::new(provider);
        self
    }

    /// Apply a local override after current theme resolution, in call order.
    /// This runs each render and must not mutate application state.
    pub fn style_override(mut self, customize: impl Fn(&mut CartesianStyle) + Send + Sync + 'static) -> Self {
        self.style.customize(customize);
        self
    }

    /// Override a series' appearance without adding a drawing layer or changing data.
    /// Line/area values override just that component; `SeriesStyle` can supply both.
    /// ```
    /// use xgpui_luma_plot::{CartesianChartBuilder, Dataset, LineLayer, CartesianStyle, SeriesStyle};
    /// let chart = CartesianChartBuilder::new(Dataset::new())
    ///     .style(CartesianStyle::default())
    ///     .series_style("desktop", SeriesStyle::new(gpui::rgb(0x2563eb).into()))
    ///     .line(LineLayer::new("desktop"))
    ///     .build();
    /// ```
    pub fn series_style(mut self, id: impl Into<SeriesId>, series: impl Into<SeriesStyle>) -> Self {
        let id = id.into();
        let series = series.into();
        self.style.customize(move |style| style.set_series_style(id.clone(), series));
        self
    }

    /// Observe CPU render measurements and drawing-point counts. Disabled by default.
    /// Called after canvas painting; defer UI updates and avoid notifying on every report.
    pub fn on_render_stats(mut self, callback: impl Fn(ChartRenderStats, &mut gpui::App) + 'static) -> Self {
        self.observer = Some(RenderObserver::new(callback));
        self
    }

    /// Reveal areas, lines, and their markers from left to right in plot coordinates.
    /// Axes, grid, domains, and stored samples remain fixed. Defaults to `1.0`
    /// (fully drawn); finite values are clamped to `0.0..=1.0`, non-finite values
    /// restore full drawing. The host owns timing and replay state.
    ///
    /// ```
    /// use xgpui_luma_plot::{CartesianChartBuilder, Dataset};
    /// let chart = CartesianChartBuilder::new(Dataset::new()).draw_progress(0.5).build();
    /// ```
    pub fn draw_progress(mut self, progress: f32) -> Self {
        self.draw_progress = if progress.is_finite() {
            progress.clamp(0.0, 1.0)
        } else {
            1.0
        };
        self
    }

    /// Attach SDK tooltips to enabled sample markers. Only the nearest point
    /// within 12 logical pixels is targeted in proximity mode. `HoverSlice`
    /// ignores pointer Y and supplies the first matching series to this callback;
    /// use `slice_tooltip` for grouped values. Hidden points and gaps cannot
    /// activate a tooltip. SDK timing and dismissal remain intact.
    ///
    /// ```
    /// use xgpui_luma_plot::{CartesianChartBuilder, Dataset};
    /// use xgpui_luma::controls::tooltip::Tooltip;
    /// let chart = CartesianChartBuilder::new(Dataset::new())
    ///     .point_tooltip(|point| Tooltip::new(format!("{}: {}", point.label, point.sample.y)))
    ///     .build();
    /// ```
    pub fn point_tooltip(
        mut self,
        tooltip: impl Fn(&PlotPoint) -> xgpui_luma::controls::tooltip::Tooltip + 'static,
    ) -> Self {
        self.point_tooltip = Some(PointTooltipFactory::Point(std::sync::Arc::new(tooltip)));
        self
    }

    /// One SDK tooltip for a shared X slice. Replaces `point_tooltip`; point
    /// proximity mode supplies a single observation through the same callback.
    /// ```
    /// use xgpui_luma_plot::{CartesianChartBuilder, Dataset};
    /// use xgpui_luma::controls::tooltip::Tooltip;
    /// let chart = CartesianChartBuilder::new(Dataset::new())
    ///     .slice_tooltip(|slice| Tooltip::new(format!("{}: {} series", slice.coordinate, slice.points.len())))
    ///     .build();
    /// ```
    pub fn slice_tooltip(
        mut self,
        tooltip: impl Fn(&PlotSlice) -> xgpui_luma::controls::tooltip::Tooltip + 'static,
    ) -> Self {
        self.point_tooltip = Some(PointTooltipFactory::Slice(std::sync::Arc::new(tooltip)));
        self
    }

    /// Animate point and bar tooltip anchors with the SDK's cubic ease-out transition.
    /// Zero (the default) moves immediately. A nonzero duration adds a short
    /// slide on hover entry and smooth retargeting while the pointer moves. SDK popup
    /// placement, edge flipping, delay and dismissal still apply. Markers and
    /// observation events update immediately, independently of tooltip motion.
    /// Use a zero SDK tooltip delay to make the hover-entry slide visible.
    /// ```
    /// use std::time::Duration;
    /// use xgpui_luma_plot::{CartesianChartBuilder, Dataset};
    /// let chart = CartesianChartBuilder::new(Dataset::new())
    ///     .tooltip_motion(Duration::from_millis(250)).build();
    /// ```
    pub fn tooltip_motion(mut self, duration: std::time::Duration) -> Self {
        self.tooltip_motion = duration;
        self.bar_options.motion = duration;
        self
    }

    /// Observe hover changes and click picks for enabled markers on lines and areas.
    /// Slice mode reports all matching observations. Leaving the target reports
    /// an empty hover group. Unchanged targets do not emit during redraws; events
    /// work without tooltips. Hover callbacks run after rendering, so hosts may
    /// update their UI directly. Click picks do not retain a selection in the plot.
    /// ```
    /// use xgpui_luma_plot::{CartesianChartBuilder, Dataset, PlotPointEvent};
    /// let chart = CartesianChartBuilder::new(Dataset::new())
    ///     .on_point_event(|event, _cx| {
    ///         if let PlotPointEvent::Select { points } = event {
    ///             for point in points { println!("{}: {}", point.label, point.sample.y); }
    ///         }
    ///     }).build();
    /// ```
    pub fn on_point_event(mut self, callback: impl Fn(PlotPointEvent, &mut gpui::App) + 'static) -> Self {
        self.point_events = Some(PointEvents::new(callback));
        self
    }

    /// Bar hover changes and clicks use original PlotPoint values. Also applies to line markers in mixed charts.
    pub fn on_bar_event(self, callback: impl Fn(PlotPointEvent, &mut gpui::App) + 'static) -> Self {
        self.on_point_event(callback)
    }
    /// Apply host viewport domains while preserving each numeric scale's mapping and ticks.
    /// Category axes remain unchanged.
    pub fn viewport(mut self, viewport: &crate::Viewport) -> Self {
        if let Scale::Numeric(scale) = self.plot.x_scale_spec() {
            self.plot = self.plot.clone().x_scale(scale.clone().domain(viewport.x()));
        }
        if let Scale::Numeric(scale) = self.plot.y_scale_spec() {
            self.plot = self.plot.clone().y_scale(scale.clone().domain(viewport.y()));
        }
        self
    }
    pub fn legend_interaction(mut self, interaction: crate::LegendInteraction) -> Self {
        self.legend_interaction = interaction;
        self
    }
    /// Add a data-anchored annotation. It clips to the plot, remains stationary during reveal,
    /// and does not expand automatic domains or participate in observation picking.
    pub fn annotation(mut self, annotation: crate::ChartAnnotation) -> Self {
        self.annotations.push(annotation);
        self
    }
    pub fn build(self) -> CartesianChartView {
        CartesianChartView::new(
            self.id,
            self.data,
            self.plot,
            self.style,
            self.observer,
            self.draw_progress,
            self.point_tooltip.map(|factory| PointTooltip { factory, motion: self.tooltip_motion }),
            self.point_events,
            self.bar_options,
            self.legend_interaction,
            self.annotations,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AxisPresentation, AxisSide, NumericScale, Sample, Series};

    #[test]
    fn defaults_and_visual_overrides_preserve_data_and_mapping() {
        let data = Dataset::new()
            .with_series(Series::new("temperature", "Temperature", [Sample::new(1.0, 18.0)]))
            .expect("unique series");
        let base = CartesianChartBuilder::new(data)
            .y_scale(NumericScale::new().tick_count(4))
            .line(LineLayer::new("temperature"));
        assert_eq!(CartesianPresentation::from(base.style.resolve()).axis_components().len(), 2);
        let styled = base
            .clone()
            .axes([Axis::new(AxisSide::Right, AxisPresentation::new(gpui::black()))])
            .line_style("temperature", LinePresentation::new(gpui::white()));
        assert_eq!(
            styled.data.get(&"temperature".into()).expect("series").samples(),
            base.data.get(&"temperature".into()).expect("series").samples()
        );
        assert_eq!(styled.plot.y_scale_spec().ticks(), 4);
        assert_eq!(styled.plot.lines()[0].series(), base.plot.lines()[0].series());
        assert_eq!(
            CartesianPresentation::from(styled.style.resolve()).line_for(&"temperature".into()).stroke.color,
            gpui::white()
        );
        assert_eq!(CartesianPresentation::from(styled.style.resolve()).axis_components()[0].side, AxisSide::Right);
        let _view = styled.build();
    }

    #[test]
    fn area_paint_accepts_native_gradients_without_changing_layer_mapping() {
        let fill = gpui::linear_gradient(
            180.0,
            gpui::linear_color_stop(gpui::white(), 0.0),
            gpui::linear_color_stop(gpui::black(), 1.0),
        );
        let builder = CartesianChartBuilder::new(Dataset::new())
            .area(AreaLayer::new("a").baseline(2.0).interpolation(crate::Interpolation::Monotone))
            .area_style("a", AreaPresentation::new(fill));
        assert!(builder.plot.lines().is_empty());
        assert_eq!(builder.plot.areas().len(), 1);
        assert_eq!(builder.plot.areas()[0].baseline_value(), 2.0);
        assert_eq!(builder.plot.areas()[0].interpolation_policy(), crate::Interpolation::Monotone);
        assert_eq!(CartesianPresentation::from(builder.style.resolve()).area_for(&"a".into()).fill, fill);
        assert!(CartesianPresentation::from(builder.style.resolve()).area_for(&"a".into()).outline.is_none());
        let _ = builder.draw_progress(0.5).build();
    }

    #[test]
    fn draw_progress_is_opt_in_and_never_changes_samples_or_scales() {
        let data = Dataset::new().with_series(Series::new("a", "A", [Sample::new(1.0, 2.0)])).expect("series");
        let base = CartesianChartBuilder::new(data)
            .x_scale(NumericScale::new().domain(0.0..=10.0))
            .line(LineLayer::new("a").interpolation(crate::Interpolation::Monotone));
        assert_eq!(base.draw_progress, 1.0);
        for (requested, expected) in [(-1.0, 0.0), (0.5, 0.5), (2.0, 1.0), (f32::NAN, 1.0), (f32::INFINITY, 1.0)] {
            let revealed = base.clone().draw_progress(requested);
            assert_eq!(revealed.draw_progress, expected);
            assert_eq!(revealed.data.series()[0].samples().as_ptr(), base.data.series()[0].samples().as_ptr());
            assert_eq!(revealed.plot.lines()[0].interpolation_policy(), crate::Interpolation::Monotone);
            assert_eq!(revealed.plot.x_scale_spec().ticks(), base.plot.x_scale_spec().ticks());
        }
    }

    #[test]
    fn theme_defaults_precede_focused_overrides() {
        let themed =
            CartesianPresentation::new(AxisPresentation::new(gpui::white()), LinePresentation::new(gpui::white()));
        let builder = CartesianChartBuilder::new(Dataset::new())
            .presentation(themed.clone())
            .axes([])
            .line_style("a", LinePresentation::new(gpui::black()));
        assert!(CartesianPresentation::from(builder.style.resolve()).axis_components().is_empty());
        assert_eq!(
            CartesianPresentation::from(builder.style.resolve()).line_for(&"a".into()).stroke.color,
            gpui::black()
        );
        let replaced = builder.presentation(themed);
        assert_eq!(CartesianPresentation::from(replaced.style.resolve()).axis_components().len(), 2);
        assert_eq!(
            CartesianPresentation::from(replaced.style.resolve()).line_for(&"a".into()).stroke.color,
            gpui::white()
        );
    }
}
