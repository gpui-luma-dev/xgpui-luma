//! Native plotting SDK with separate data, specification, presentation, and view layers.
//!
//! The chart builder composes those responsibilities into one public construction path.
//! Host frames and chart headings remain ordinary SDK content around the built view.
//!
//! # Terminology
//!
//! **Plot** names the library and the rendering-independent specifications:
//! [CartesianPlot], [PiePlot], [RadarPlot], and [RadialPlot]. A **chart**
//! combines a specification, data, and presentation into a GPUI element through
//! its chart builder and chart view. Styling contracts are family-specific:
//! [CartesianStyle] / [CartesianStyleProvider] apply to Cartesian charts.
//! [ChartMetrics] describes shared chart geometry.
//!
//! [PlotStyle] and [PlotStyleProvider] remain compatibility names for
//! [CartesianStyle] and [CartesianStyleProvider].
//!
//! ```
//! use gpui::rgb;
//! use xgpui_luma_plot::{
//!     Axis, AxisSide, AxisPresentation, CartesianChartBuilder, Dataset,
//!     LineLayer, LinePresentation, NumericScale, Sample, Series,
//! };
//!
//! let data = Dataset::new()
//!     .with_series(Series::new("temperature", "Temperature", [Sample::new(0.0, 18.0)]))?
//!     .with_series(Series::new("humidity", "Humidity", [Sample::new(0.0, 42.0)]))?;
//! let axis = AxisPresentation::new(rgb(0x64748b).into());
//! let view = CartesianChartBuilder::new(data)
//!     .x_scale(NumericScale::new().tick_count(6))
//!     .y_scale(NumericScale::new().tick_count(5))
//!     .line(LineLayer::new("temperature"))
//!     .line(LineLayer::new("humidity"))
//!     .line_style("humidity", LinePresentation::new(rgb(0x14b8a6).into()))
//!     .axes([
//!         Axis::new(AxisSide::Bottom, axis.clone()).title("Sample"),
//!         Axis::new(AxisSide::Right, axis).title("Value"),
//!     ])
//!     .build();
//! # Ok::<(), xgpui_luma_plot::DatasetError>(())
//! ```
//!
//! Category coordinates are zero-based indices; numeric ticks can be selected explicitly:
//!
//! ```
//! use xgpui_luma_plot::{CartesianChartBuilder, CategoryScale, Dataset, LineLayer, NumericScale, Sample, Series};
//! let data = Dataset::new().with_series(Series::new("temperature", "Temperature", [
//!     Sample::new(0.0, 18.0), Sample::new(1.0, 21.0), Sample::new(2.0, 27.0),
//! ]))?;
//! let view = CartesianChartBuilder::new(data)
//!     .x_scale(CategoryScale::new(["Mon", "Tue", "Wed"]))
//!     .y_scale(NumericScale::new().tick_values([18.0, 21.0, 24.0, 27.0]))
//!     .line(LineLayer::new("temperature"))
//!     .build();
//! # Ok::<(), xgpui_luma_plot::DatasetError>(())
//! ```

pub mod data;
pub mod plots;
pub mod presentation;
pub mod views;
pub mod view_state;
mod render;

pub use data::{Dataset, DatasetError, Sample, Series, SeriesId};
pub use plots::{
    BarAlignment, BarGrouping, BarLayout, BarOrientation, BarLayer, AreaLayer, CartesianPlot, CategoryScale,
    Decimation, Interpolation, LineLayer, NumericScale, Scale,
};
pub use presentation::{
    BarLabel, BarLabelPlacement, BarPresentation, AreaPresentation, Axis, AxisPresentation, AxisSide,
    CartesianPresentation, ChartMetrics, EmptyStatePresentation, GridPresentation, LegendPresentation,
    LinePresentation, MarkerPresentation, MarkerVisibility, Stroke, SwatchPresentation, TextPresentation,
    CartesianStyle, CartesianStyleProvider, PlotStyle, PlotStyleProvider, SeriesStyle,
};
pub use views::{
    BarRenderModel, BarTemplate, RectangleBarTemplate, PlotPointEvent, PlotSlice, PlotPoint, CartesianChartBuilder,
    CartesianChartView, ChartRenderStats, LegendItem, LegendView, LegendEvent, LegendInteraction, LegendTarget,
};

pub use plots::PiePlot;
pub use presentation::{PiePresentation, PieLabelPlacement, PieStyleProvider, PieRingPresentation};
pub use views::{PieChartBuilder, PieChartView, PieSlice, PieSliceEvent};

pub use plots::RadarPlot;
pub use presentation::{RadarPresentation, RadarSeriesPresentation, RadarGridShape};
pub use views::{RadarChartBuilder, RadarChartView, RadarSpoke, RadarPoint};

pub use plots::{RadialPlot, RadialLayout};
pub use presentation::RadialPresentation;
pub use views::{RadialChartBuilder, RadialChartView, RadialBar};

pub use views::SelectionEvent;

pub use view_state::{Viewport, ViewportChange, ViewportAction, InvalidViewport};

pub use plots::Annotation;
pub use presentation::AnnotationPresentation;
pub use views::ChartAnnotation;
