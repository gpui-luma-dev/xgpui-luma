use std::{ops::RangeInclusive, sync::Arc};

type NumericTickLabel = (f64, Arc<str>);

/// Numeric mapping with automatic data bounds and a configurable tick policy.
#[derive(Clone, Debug)]
pub struct NumericScale {
    log_base: Option<f64>,
    tick_count: usize,
    tick_values: Option<Arc<[f64]>>,
    domain: Option<RangeInclusive<f64>>,
    tick_labels: Option<Arc<[NumericTickLabel]>>,
}

impl NumericScale {
    pub fn new() -> Self {
        Self { log_base: None, tick_count: 5, tick_values: None, domain: None, tick_labels: None }
    }

    /// Use an explicit visible numeric domain instead of automatic data bounds.
    /// Non-finite, reversed, or zero-width ranges restore automatic bounds.
    /// An X domain selects visible line segments with their immediate edge neighbors
    /// and bar slots intersecting the viewport; sorted bar series use binary search.
    /// it never changes stored data or the automatic Y domain.
    ///
    /// ```
    /// use xgpui_luma_plot::{CartesianChartBuilder, Dataset, NumericScale};
    /// let chart = CartesianChartBuilder::new(Dataset::new())
    ///     .x_scale(NumericScale::new().domain(200.0..=320.0).tick_count(6))
    ///     .build();
    /// ```
    pub fn domain(mut self, domain: RangeInclusive<f64>) -> Self {
        self.domain = (domain.start().is_finite()
            && domain.end().is_finite()
            && domain.start() < domain.end()
            && (self.log_base.is_none() || *domain.start() > 0.0))
            .then_some(domain);
        self
    }

    /// Positive logarithmic mapping; zero/negative observations and baselines are omitted.
    /// Invalid bases restore base ten. Existing nonpositive domains restore automatic bounds.
    pub fn logarithmic(mut self, base: f64) -> Self {
        self.log_base = Some(if base.is_finite() && base > 1.0 { base } else { 10.0 });
        if self.domain.as_ref().is_some_and(|d| *d.start() <= 0.0) {
            self.domain = None;
        }
        self
    }
    pub fn linear(mut self) -> Self {
        self.log_base = None;
        self
    }
    pub fn log_base(&self) -> Option<f64> {
        self.log_base
    }
    pub fn domain_bounds(&self) -> Option<&RangeInclusive<f64>> {
        self.domain.as_ref()
    }

    /// Request evenly spaced ticks, replacing any explicit tick values.
    /// Counts below two are promoted to two; a constant domain shows one centered tick.
    pub fn tick_count(mut self, count: usize) -> Self {
        self.tick_count = count.max(2);
        self.tick_values = None;
        self.tick_labels = None;
        self
    }

    /// Show selected numeric ticks without changing the data domain or mapping.
    /// Values are sorted and deduplicated; non-finite and out-of-domain ticks are omitted.
    /// An empty collection hides tick marks and labels, preserving the scale and axis line.
    pub fn tick_values(mut self, values: impl IntoIterator<Item = f64>) -> Self {
        let mut values: Vec<_> = values
            .into_iter()
            .filter(|value| value.is_finite())
            .map(|value| if value == 0.0 { 0.0 } else { value })
            .collect();
        values.sort_by(f64::total_cmp);
        values.dedup_by(|a, b| *a == *b);
        self.tick_values = Some(values.into());
        self.tick_labels = None;
        self
    }

    /// Label selected numeric coordinates without changing their spacing or domain.
    /// Non-finite coordinates are omitted; duplicate coordinates keep their first label.
    /// `tick_count` or `tick_values` replaces these labels.
    /// ```
    /// use xgpui_luma_plot::NumericScale;
    /// let dates = NumericScale::new().tick_labels([(0.0, "Apr 1"), (30.0, "May 1")]);
    /// ```
    pub fn tick_labels<S: Into<Arc<str>>>(mut self, labels: impl IntoIterator<Item = (f64, S)>) -> Self {
        let mut labels: Vec<_> = labels
            .into_iter()
            .filter(|(value, _)| value.is_finite())
            .map(|(value, label)| (if value == 0.0 { 0.0 } else { value }, label.into()))
            .collect();
        labels.sort_by(|a, b| a.0.total_cmp(&b.0));
        labels.dedup_by(|a, b| a.0 == b.0);
        self.tick_values = Some(labels.iter().map(|(value, _)| *value).collect());
        self.tick_labels = Some(labels.into());
        self
    }

    pub(crate) fn label_for(&self, value: f64) -> Option<&str> {
        let labels = self.tick_labels.as_ref()?;
        let index = labels.binary_search_by(|(coordinate, _)| coordinate.total_cmp(&value)).ok()?;
        Some(&labels[index].1)
    }

    /// Requested tick count; domain filtering may produce fewer visible ticks.
    pub fn ticks(&self) -> usize {
        self.tick_values.as_ref().map_or(self.tick_count, |values| values.len())
    }

    pub fn explicit_ticks(&self) -> Option<&[f64]> {
        self.tick_values.as_deref()
    }
}

impl Default for NumericScale {
    fn default() -> Self {
        Self::new()
    }
}

/// Ordered, evenly spaced category points, independent of labels' visual treatment.
///
/// Samples use zero-based category indices in the corresponding numeric coordinate.
/// Category order is explicit and remains stable when observations are missing.
/// Non-integer, negative, or unknown indices are gaps in the plotted line.
#[derive(Clone, Debug)]
pub struct CategoryScale {
    labels: Arc<[Arc<str>]>,
}

impl CategoryScale {
    pub fn new(labels: impl IntoIterator<Item = impl Into<Arc<str>>>) -> Self {
        Self { labels: labels.into_iter().map(Into::into).collect() }
    }

    pub fn labels(&self) -> &[Arc<str>] {
        &self.labels
    }
}

/// Coordinate mapping and tick specification. Axis components present either variant.
#[derive(Clone, Debug)]
pub enum Scale {
    Numeric(NumericScale),
    Category(CategoryScale),
}

impl Scale {
    pub fn ticks(&self) -> usize {
        match self {
            Self::Numeric(scale) => scale.ticks(),
            Self::Category(scale) => scale.labels.len(),
        }
    }

    pub(crate) fn accepts(&self, value: f64) -> bool {
        match self {
            Self::Numeric(scale) => value.is_finite() && (scale.log_base.is_none() || value > 0.0),
            Self::Category(scale) => {
                value.is_finite() && value >= 0.0 && value.fract() == 0.0 && value < scale.labels.len() as f64
            }
        }
    }
}

impl From<NumericScale> for Scale {
    fn from(scale: NumericScale) -> Self {
        Self::Numeric(scale)
    }
}

impl From<CategoryScale> for Scale {
    fn from(scale: CategoryScale) -> Self {
        Self::Category(scale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_labels_follow_coordinates_and_existing_tick_replacement_rules() {
        let scale = NumericScale::new().domain(0.0..=90.0).tick_labels([
            (30.0, "May 1"),
            (0.0, "Apr 1"),
            (90.0, "Jun 30"),
            (30.0, "duplicate"),
            (f64::NAN, "invalid"),
        ]);
        assert_eq!(scale.explicit_ticks(), Some([0.0, 30.0, 90.0].as_slice()));
        assert_eq!(scale.label_for(30.0), Some("May 1"));
        assert_eq!(scale.clone().domain(20.0..=90.0).label_for(30.0), Some("May 1"));
        assert!(scale.clone().tick_values([30.0]).label_for(30.0).is_none());
        assert!(scale.tick_count(5).label_for(30.0).is_none());
    }

    #[test]
    fn explicit_domain_survives_tick_changes_and_invalid_ranges_restore_auto() {
        let scale = NumericScale::new().domain(20.0..=140.0).tick_values([20.0, 80.0, 140.0]);
        assert_eq!(scale.domain_bounds(), Some(&(20.0..=140.0)));
        assert_eq!(scale.tick_count(6).domain_bounds(), Some(&(20.0..=140.0)));
        for (start, end) in [(1.0, 1.0), (2.0, 1.0), (f64::NAN, 2.0), (0.0, f64::INFINITY)] {
            assert!(NumericScale::new().domain(start..=end).domain_bounds().is_none());
        }
    }
}
