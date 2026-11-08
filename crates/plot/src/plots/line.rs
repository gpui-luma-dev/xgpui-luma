use crate::data::SeriesId;

/// Resolution-aware reduction of a line's drawing points. Stored samples are unchanged.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Decimation {
    /// Draw every accepted sample (the default).
    #[default]
    None,
    /// Keep first, last, minimum, and maximum points in each logical pixel column,
    /// in input order. Each contiguous run is reduced separately as the plot resizes.
    /// Runs whose x coordinates reverse direction are drawn at full resolution.
    /// Point markers, when enabled, mark only the retained points.
    MinMax,
}

/// How a line connects its retained observations. Source samples and axis domains are unchanged.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Interpolation {
    #[default]
    Linear,
    /// Shape-preserving cubic Hermite interpolation along X, without overshooting
    /// neighboring values. Each missing-data run is interpolated independently,
    /// after decimation. Markers remain on retained observations.
    /// Runs with repeated/reversing X coordinates, or fewer than three drawing
    /// points, fall back to straight segments.
    Monotone,
}

/// Connect one series in input order; straight segments are the default.
///
/// It contains no visual styling. Non-finite samples break the line.
#[derive(Clone, Debug)]
pub struct LineLayer {
    series: SeriesId,
    decimation: Decimation,
    interpolation: Interpolation,
}

impl LineLayer {
    pub fn new(series: impl Into<SeriesId>) -> Self {
        Self { series: series.into(), decimation: Decimation::None, interpolation: Interpolation::Linear }
    }

    /// Select drawing-point reduction without changing the source data or scale domains.
    pub fn decimation(mut self, decimation: Decimation) -> Self {
        self.decimation = decimation;
        self
    }

    /// Select how retained observations are connected, independently of line styling.
    ///
    /// ```
    /// use xgpui_luma_plot::{Interpolation, LineLayer};
    /// let line = LineLayer::new("signal").interpolation(Interpolation::Monotone);
    /// ```
    pub fn interpolation(mut self, interpolation: Interpolation) -> Self {
        self.interpolation = interpolation;
        self
    }

    pub fn interpolation_policy(&self) -> Interpolation {
        self.interpolation
    }

    pub fn series(&self) -> &SeriesId {
        &self.series
    }

    pub fn decimation_policy(&self) -> Decimation {
        self.decimation
    }
}
