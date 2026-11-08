use std::sync::Arc;
use gpui::{App, ElementId, IntoElement, RenderOnce, SharedString, Window, div, prelude::*};
use crate::{Dataset, PiePlot, PiePresentation, PieStyleProvider, Sample, LegendItem, LegendView, SwatchPresentation};

/// One accepted source weight and its normalized share. Colors and label names
/// follow the original sample index, even when other samples are omitted.
#[derive(Clone, Debug)]
pub struct PieSlice {
    /// None for the primary pie; zero-based addition order for concentric rings.
    pub ring_index: Option<usize>,
    pub series: crate::SeriesId,
    pub sample: Sample,
    pub sample_index: usize,
    pub label: SharedString,
    pub fraction: f64,
}
/// Original slice observations; the host owns any persistent selection.
#[derive(Clone, Debug)]
pub enum PieSliceEvent {
    HoverChanged { slice: Option<PieSlice> },
    Select { slice: PieSlice },
}
type SliceCallback = dyn Fn(PieSliceEvent, &mut App);
#[derive(Clone)]
pub(crate) struct PieEvents(pub std::rc::Rc<SliceCallback>);
#[derive(Clone)]
pub(crate) struct PieTooltip(pub Arc<dyn Fn(&PieSlice) -> xgpui_luma::controls::tooltip::Tooltip>);
#[derive(Clone, Default)]
pub(crate) struct PieInteraction {
    pub events: Option<PieEvents>,
    pub tooltip: Option<PieTooltip>,
    pub tooltip_motion: std::time::Duration,
    pub active_motion: std::time::Duration,
    pub draw_progress: f32,
}
#[derive(Clone)]
pub(crate) struct PieLabels(pub Arc<dyn Fn(&PieSlice) -> SharedString>);
#[derive(Clone)]
pub struct PieChartBuilder {
    data: Dataset,
    plot: PiePlot,
    legend_interaction: crate::LegendInteraction,
    id: ElementId,
    style: crate::presentation::pie::PieProvider,
    overrides: Vec<Arc<PieOverride>>,
    names: Arc<[SharedString]>,
    labels: Option<PieLabels>,
    active: Option<usize>,
    center: Option<(SharedString, SharedString)>,
    active_rings: std::collections::HashMap<usize, usize>,
    ring_active: std::collections::HashMap<crate::SeriesId, usize>,
    rings: Vec<(PiePlot, crate::PieRingPresentation)>,
    interaction: PieInteraction,
}
type PieOverride = dyn Fn(&mut PiePresentation) + Send + Sync;
impl PieChartBuilder {
    /// Construct a pie from one series. Y stores positive weights, not percentages.
    /// ```
    /// use xgpui_luma_plot::{Dataset, Series, Sample, PieChartBuilder};
    /// let data = Dataset::new().with_series(Series::new("visits", "Visits", [
    ///     Sample::new(0.0, 275.0), Sample::new(1.0, 200.0),
    /// ]))?;
    /// let view = PieChartBuilder::new(data, "visits").id("browser-share")
    ///     .slice_names(["Chrome", "Safari"]).inner_radius(0.6).build();
    /// # Ok::<(), xgpui_luma_plot::DatasetError>(())
    /// ```
    pub fn new(data: Dataset, series: impl Into<crate::SeriesId>) -> Self {
        Self {
            data,
            plot: PiePlot::new(series),
            id: "pie-chart".into(),
            legend_interaction: Default::default(),
            style: Arc::new(PiePresentation::default()),
            overrides: Vec::new(),
            names: Arc::from([]),
            labels: None,
            active: None,
            center: None,
            rings: Vec::new(),
            ring_active: Default::default(),
            active_rings: Default::default(),
            interaction: PieInteraction { draw_progress: 1.0, ..PieInteraction::default() },
        }
    }
    /// Stable identity; sibling pies need distinct IDs.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }
    pub fn start_angle(mut self, degrees: f64) -> Self {
        self.plot = self.plot.start_angle(degrees);
        for (plot, _) in &mut self.rings {
            *plot = plot.clone().start_angle(degrees);
        }
        self
    }
    pub fn clockwise(mut self, clockwise: bool) -> Self {
        self.plot = self.plot.clockwise(clockwise);
        for (plot, _) in &mut self.rings {
            *plot = plot.clone().clockwise(clockwise);
        }
        self
    }
    pub fn plot(mut self, plot: PiePlot) -> Self {
        self.plot = plot;
        self
    }
    /// Bind static or live appearance. Replaces earlier style overrides.
    pub fn style(mut self, style: impl PieStyleProvider) -> Self {
        self.style = Arc::new(style);
        self.overrides.clear();
        self
    }
    pub fn style_override(mut self, customize: impl Fn(&mut PiePresentation) + Send + Sync + 'static) -> Self {
        self.overrides.push(Arc::new(customize));
        self
    }
    /// Ratio of hole radius to outer radius. Finite values clamp to 0..=0.95;
    /// non-finite inputs restore a solid pie. Default zero.
    pub fn inner_radius(self, ratio: f32) -> Self {
        self.style_override(move |style| {
            style.inner_radius = if ratio.is_finite() { ratio.clamp(0.0, 0.95) } else { 0.0 }
        })
    }
    /// Scale the primary pie against the common chart radius; useful for concentric series.
    pub fn outer_radius(self, ratio: f32) -> Self {
        self.style_override(move |style| {
            style.outer_radius = if ratio.is_finite() { ratio.clamp(0.0, 1.0) } else { 1.0 }
        })
    }
    /// Add an independently normalized concentric series. Bands may overlap;
    /// the host chooses their radii. Names and palette follow original sample indices.
    pub fn ring(mut self, series: impl Into<crate::SeriesId>, band: crate::PieRingPresentation) -> Self {
        let plot = PiePlot::new(series)
            .start_angle(self.plot.start_angle_degrees())
            .clockwise(self.plot.is_clockwise());
        self.rings.push((plot, band));
        self
    }
    /// Animate host-controlled active selection using SDK transitions. Default zero.
    pub fn active_motion(mut self, duration: std::time::Duration) -> Self {
        self.interaction.active_motion = duration;
        self
    }
    /// Reveal sectors in angular order from the start angle, without changing weights.
    /// Host owns replay timing. Defaults to one; finite values clamp to 0..=1.
    pub fn draw_progress(mut self, progress: f32) -> Self {
        self.interaction.draw_progress = if progress.is_finite() {
            progress.clamp(0.0, 1.0)
        } else {
            1.0
        };
        self
    }
    /// SDK tooltip for each slice, including added rings. Configure its hover delay
    /// on the returned tooltip; leaving a sector cancels pending display.
    pub fn slice_tooltip(
        mut self,
        tooltip: impl Fn(&PieSlice) -> xgpui_luma::controls::tooltip::Tooltip + 'static,
    ) -> Self {
        self.interaction.tooltip = Some(PieTooltip(Arc::new(tooltip)));
        self
    }
    /// Animate the tooltip anchor between slices. Defaults to immediate movement.
    pub fn tooltip_motion(mut self, duration: std::time::Duration) -> Self {
        self.interaction.tooltip_motion = duration;
        self
    }
    /// Observe hover changes and clicks from primary content and added rings; holes and radial gaps do not hit.
    pub fn on_slice_event(mut self, callback: impl Fn(PieSliceEvent, &mut App) + 'static) -> Self {
        self.interaction.events = Some(PieEvents(std::rc::Rc::new(callback)));
        self
    }

    /// Names indexed by original sample position. Missing names use the X coordinate.
    pub fn slice_names(mut self, names: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.names = names.into_iter().map(Into::into).collect();
        self
    }
    /// Override the fill of a source sample; does not change angular shares.
    pub fn slice_style(self, index: usize, fill: impl Into<gpui::Background>) -> Self {
        let fill = fill.into();
        self.style_override(move |style| {
            style.slices.insert(index, fill);
        })
    }
    /// Opt-in labels. Placement, text style, gap and leader lines belong to presentation.
    pub fn slice_labels(mut self, labels: impl Fn(&PieSlice) -> SharedString + 'static) -> Self {
        self.labels = Some(PieLabels(Arc::new(labels)));
        self
    }
    pub fn legend(self, visible: bool) -> Self {
        self.style_override(move |style| style.legend.visible = visible)
    }
    /// Expand one source sample outward. The host owns selection; no hidden state.
    pub fn active_slice(mut self, sample_index: Option<usize>) -> Self {
        self.active = sample_index;
        self
    }
    /// Optional center title/caption, normally used inside a donut hole.
    pub fn center_text(mut self, title: impl Into<SharedString>, caption: impl Into<SharedString>) -> Self {
        self.center = Some((title.into(), caption.into()));
        self
    }
    /// Independently activate a source slice in an added ring.
    pub fn ring_active_slice(mut self, series: impl Into<crate::SeriesId>, sample_index: Option<usize>) -> Self {
        let series = series.into();
        if let Some(index) = sample_index {
            self.ring_active.insert(series, index);
        } else {
            self.ring_active.remove(&series);
        }
        self
    }
    pub fn legend_interaction(mut self, interaction: crate::LegendInteraction) -> Self {
        self.legend_interaction = interaction;
        self
    }
    /// Activate one added ring by its zero-based addition index, even when rings reuse a series.
    pub fn active_ring_slice(mut self, ring_index: usize, sample_index: Option<usize>) -> Self {
        if let Some(index) = sample_index {
            self.active_rings.insert(ring_index, index);
        } else {
            self.active_rings.remove(&ring_index);
        }
        self
    }
    pub fn build(self) -> PieChartView {
        PieChartView { builder: self }
    }
}
#[derive(IntoElement)]
pub struct PieChartView {
    builder: PieChartBuilder,
}
impl RenderOnce for PieChartView {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let builder = self.builder;
        let mut style = builder.style.resolve();
        for customize in builder.overrides {
            customize(&mut style);
        }
        let slices = crate::render::pie::slices(&builder.data, &builder.plot, &builder.names);
        let legend = style.legend;
        let metrics = style.metrics;
        let legend_targets: Vec<_> = slices
            .iter()
            .map(|slice| crate::LegendTarget::Sample { series: slice.series.clone(), sample_index: slice.sample_index })
            .collect();
        let items: Vec<_> = slices
            .iter()
            .map(|slice| LegendItem {
                label: slice.label.clone(),
                swatch: SwatchPresentation::Fill(style.fill_for(slice.sample_index)),
            })
            .collect();
        let extra: Vec<_> = builder
            .rings
            .into_iter()
            .enumerate()
            .map(|(index, (plot, band))| {
                let mut ring_style = style.clone();
                let outer = if band.outer_radius.is_finite() {
                    band.outer_radius.clamp(0.0, 1.0)
                } else {
                    1.0
                };
                let inner = if band.inner_radius.is_finite() {
                    band.inner_radius.clamp(0.0, outer)
                } else {
                    0.0
                };
                ring_style.outer_radius = outer;
                ring_style.inner_radius = if outer > 0.0 { (inner / outer).min(0.95) } else { 0.0 };
                ring_style.empty_state = None;
                let mut slices = crate::render::pie::slices(&builder.data, &plot, &builder.names);
                for slice in &mut slices {
                    slice.ring_index = Some(index);
                }
                let interaction = builder.interaction.clone();
                // Each band retains independent hover state and reports its source series.

                let active = builder
                    .active_rings
                    .get(&index)
                    .copied()
                    .or_else(|| builder.ring_active.get(plot.series()).copied());
                div()
                    .id(("pie-ring", index))
                    .absolute()
                    .inset_0()
                    .child(crate::render::pie::pie_canvas(slices, plot, ring_style, None, active, None, interaction))
                    .into_any_element()
            })
            .collect();
        div()
            .id(builder.id)
            .size_full()
            .flex()
            .flex_col()
            .gap(metrics.legend_gap)
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .child(div().id("pie-primary").size_full().child(crate::render::pie::pie_canvas(
                        slices,
                        builder.plot,
                        style,
                        builder.labels,
                        builder.active,
                        builder.center,
                        builder.interaction,
                    )))
                    .children(extra),
            )
            .when(legend.visible, |root| {
                root.child(
                    div().flex().justify_center().child(
                        LegendView::new(items, legend, metrics)
                            .targets(legend_targets)
                            .interaction(builder.legend_interaction),
                    ),
                )
            })
    }
}
