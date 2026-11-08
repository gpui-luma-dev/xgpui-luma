use std::sync::Arc;
use gpui::Hsla;
use xgpui_luma::controls::tooltip::Tooltip;
use crate::{Sample, SeriesId};

/// One original observation selected by pointer hit testing, never a curve estimate.
#[derive(Clone, Debug)]
pub struct PlotPoint {
    pub series: SeriesId,
    pub label: String,
    pub sample: Sample,
    pub color: Hsla,
}

/// Observations sharing one selected coordinate. Missing values are omitted.
/// Currently the selected coordinate is X.
#[derive(Clone, Debug)]
pub struct PlotSlice {
    pub coordinate: f64,
    pub points: Vec<PlotPoint>,
}

#[derive(Clone)]
pub(crate) enum PointTooltipFactory {
    Point(Arc<dyn Fn(&PlotPoint) -> Tooltip>),
    Slice(Arc<dyn Fn(&PlotSlice) -> Tooltip>),
}
#[derive(Clone)]
pub(crate) struct PointTooltip {
    pub(crate) factory: PointTooltipFactory,
    pub(crate) motion: std::time::Duration,
}
impl std::fmt::Debug for PointTooltipFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PointTooltip")
    }
}

/// Input events for the chart's enabled observation markers, independent of tooltips.
#[derive(Clone, Debug)]
pub enum PlotPointEvent {
    /// The hover target changed. Empty points means the target was left or removed.
    /// A slice contains one original observation per matching layer.
    HoverChanged { points: Vec<PlotPoint> },
    /// A click picked the targeted point or slice. Repeated clicks emit again.
    /// This reports activation; the host owns any persistent selection.
    Select { points: Vec<PlotPoint> },
}

type PointEventCallback = dyn Fn(PlotPointEvent, &mut gpui::App);

#[derive(Clone)]
pub(crate) struct PointEvents(std::rc::Rc<PointEventCallback>);

impl PointEvents {
    pub(crate) fn new(callback: impl Fn(PlotPointEvent, &mut gpui::App) + 'static) -> Self {
        Self(std::rc::Rc::new(callback))
    }
    pub(crate) fn report(&self, event: PlotPointEvent, cx: &mut gpui::App) {
        (self.0)(event, cx);
    }
}
impl std::fmt::Debug for PointEvents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PointEvents")
    }
}
