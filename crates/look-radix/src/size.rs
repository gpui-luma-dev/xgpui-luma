//! Radix control sizes (`1` / `2` / `3`) resolved to pixels.
//!
//! Each Radix control has its own `size` prop enum (for example
//! [`crate::ButtonSize`] or [`crate::CheckboxSize`]). Those map onto [`RadixSize`],
//! which selects the look's control metrics and the `"1"`/`"2"`/`"3"` stylesheet
//! entries. The SDK has no size axis and only receives the resolved pixel values.

use xgpui_luma::theme::ControlMetricTokens;

/// Shared Radix control size used for metrics and stylesheet keys. Default is [`Self::Two`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum RadixSize {
    One,
    #[default]
    Two,
    Three,
}

impl RadixSize {
    pub const ALL: [Self; 3] = [Self::One, Self::Two, Self::Three];

    /// Stylesheet size key (`sizes."1"`, ...).
    pub fn key(self) -> &'static str {
        match self {
            Self::One => "1",
            Self::Two => "2",
            Self::Three => "3",
        }
    }

    pub(crate) fn index(self) -> usize {
        match self {
            Self::One => 0,
            Self::Two => 1,
            Self::Three => 2,
        }
    }
}

/// Radix control metrics for each [`RadixSize`], in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RadixControlMetrics {
    pub one: ControlMetricTokens,
    pub two: ControlMetricTokens,
    pub three: ControlMetricTokens,
}

impl RadixControlMetrics {
    /// Button-box metrics for sizes 1–3, with `two` taken from `md`.
    pub fn from_button_boxes(md: ControlMetricTokens) -> Self {
        let control = |size| {
            let geometry = crate::button_layout::button_box_for(size, crate::Radius::Medium);
            ControlMetricTokens::new(
                geometry.height,
                geometry.padding_x,
                geometry.padding_y,
                geometry.gap,
                geometry.radius,
                geometry.icon_size,
            )
        };
        Self { one: control(crate::ButtonSize::One), two: md, three: control(crate::ButtonSize::Three) }
    }

    pub fn get(&self, size: RadixSize) -> ControlMetricTokens {
        match size {
            RadixSize::One => self.one,
            RadixSize::Two => self.two,
            RadixSize::Three => self.three,
        }
    }

    pub fn for_size(&self, size: RadixSize) -> ControlMetricTokens {
        self.get(size)
    }

    pub fn control_height(&self, size: RadixSize) -> f32 {
        self.get(size).control_height
    }

    pub fn padding_x(&self, size: RadixSize) -> f32 {
        self.get(size).padding_x
    }

    pub fn padding_y(&self, size: RadixSize) -> f32 {
        self.get(size).padding_y
    }

    pub fn gap(&self, size: RadixSize) -> f32 {
        self.get(size).gap
    }

    pub fn radius(&self, size: RadixSize) -> f32 {
        self.get(size).radius
    }

    pub fn icon_size(&self, size: RadixSize) -> f32 {
        self.get(size).icon_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_size_is_two() {
        assert_eq!(RadixSize::default(), RadixSize::Two);
        assert_eq!(RadixSize::default().key(), "2");
    }
}

/// Radix size-1 / size-3 presets for families that start from SDK defaults and have no
/// Radix-authored size entries. Values only fill fields Radix leaves unset.
pub(crate) fn apply_size_presets(common: &mut xgpui_luma::theme::stylesheet::CommonStylesheet) {
    for (size, label_baseline) in [(RadixSize::One, 0.5), (RadixSize::Three, 1.5)] {
        fill(&mut common.shared.sizes.get_or_default(size.key()).label_baseline, label_baseline);
    }
    for (size, thickness, track, thumb, min) in
        [(RadixSize::One, 10.0, 3.0, 6.0, 24.0), (RadixSize::Three, 14.0, 5.0, 10.0, 32.0)]
    {
        let scrollbar = common.scrollbar.sizes.get_or_default(size.key());
        fill(&mut scrollbar.thickness, thickness);
        fill(&mut scrollbar.track_thickness, track);
        fill(&mut scrollbar.thumb_thickness, thumb);
        fill(&mut scrollbar.min_thumb_length, min);
    }
    for (size, badge, track) in [(RadixSize::One, 24.0, 2.0), (RadixSize::Three, 40.0, 3.0)] {
        let stepper = common.stepper.sizes.get_or_default(size.key());
        fill(&mut stepper.step_badge_size, badge);
        fill(&mut stepper.track_thickness, track);
    }
    for (size, min, max, height) in [(RadixSize::One, 320.0, 440.0, 196.0), (RadixSize::Three, 480.0, 600.0, 264.0)] {
        let overlay = common.overlay_window.sizes.get_or_default(size.key());
        fill(&mut overlay.min_width, min);
        fill(&mut overlay.max_width, max);
        fill(&mut overlay.estimated_height, height);
    }
    // The SDK resize handle scale is Radix size 1; sizes 2 and 3 are Radix's.
    for (size, lane, hit, cross, main) in
        [(RadixSize::Two, 12.0, 14.0, 6.0, 40.0), (RadixSize::Three, 18.0, 20.0, 8.0, 44.0)]
    {
        let handle = common.resizable_panels.sizes.get_or_default(size.key());
        fill(&mut handle.lane_width, lane);
        fill(&mut handle.hit_target, hit);
        fill(&mut handle.grip_cross_axis, cross);
        fill(&mut handle.grip_main_axis, main);
    }
}

/// Former SDK size presets for controls Radix sizes itself; used only for fallback scales.
fn apply_choice_presets(common: &mut xgpui_luma::theme::stylesheet::CommonStylesheet) {
    for (size, scale, radius, inset) in [(RadixSize::One, 0.45, 2.0, 2.0), (RadixSize::Three, 0.55, 6.0, 4.0)] {
        let checkbox = common.checkbox.sizes.get_or_default(size.key());
        fill(&mut checkbox.indicator_scale, scale);
        fill(&mut checkbox.indicator_radius, radius);
        fill(&mut checkbox.glyph_inset, inset);
    }
    for (size, scale) in [(RadixSize::One, 0.45), (RadixSize::Three, 0.55)] {
        let radio = common.radio.sizes.get_or_default(size.key());
        fill(&mut radio.indicator_scale, scale);
        fill(&mut radio.dot_scale, 0.24);
    }
    for (size, width, height, thumb) in [
        (RadixSize::One, 32.0 / 28.0, 13.0 / 28.0, 9.0 / 28.0),
        (RadixSize::Three, 48.0 / 44.0, 24.0 / 44.0, 20.0 / 44.0),
    ] {
        let switch = common.switch.sizes.get_or_default(size.key());
        fill(&mut switch.track_width_scale, width);
        fill(&mut switch.track_height_scale, height);
        fill(&mut switch.thumb_scale, thumb);
    }
    for (size, track, diameter, stroke, thumb) in
        [(RadixSize::One, 4.0, 48.0, 5.0, 12.0), (RadixSize::Three, 8.0, 80.0, 8.0, 20.0)]
    {
        let progress = common.progress.sizes.get_or_default(size.key());
        fill(&mut progress.track_height, track);
        fill(&mut progress.size, diameter);
        fill(&mut progress.stroke_width, stroke);
        fill(&mut progress.thumb_size, thumb);
    }
}

fn fill(slot: &mut Option<f32>, value: f32) {
    slot.get_or_insert(value);
}

/// Former SDK baseline control metrics for sizes 1–3, used for Radix fallback scales.
pub(crate) fn scaffold_metrics(size: RadixSize) -> xgpui_luma::theme::MetricTokens {
    let mut metrics = xgpui_luma::theme::MetricTokens::default();
    metrics.control = match size {
        RadixSize::One => ControlMetricTokens::new(28.0, 10.0, 5.0, 6.0, 5.0, 14.0),
        RadixSize::Two => metrics.control,
        RadixSize::Three => ControlMetricTokens::new(44.0, 18.0, 10.0, 10.0, 8.0, 18.0),
    };
    metrics
}

/// SDK baseline stylesheet with the Radix presets for `size`, for fallback scales.
pub(crate) fn scaffold_stylesheet(size: RadixSize) -> &'static xgpui_luma::theme::stylesheet::CommonStylesheet {
    static SHEETS: std::sync::OnceLock<[xgpui_luma::theme::stylesheet::CommonStylesheet; 3]> =
        std::sync::OnceLock::new();
    &SHEETS.get_or_init(|| {
        let mut base = xgpui_luma::theme::stylesheet::CommonStylesheet::sdk_default().clone();
        apply_size_presets(&mut base);
        apply_choice_presets(&mut base);
        RadixSize::ALL.map(|size| base.with_size_key(size.key()))
    })[size.index()]
}

/// Former SDK baseline tokens for `size`, used where Radix starts from SDK defaults.
pub(crate) fn scaffold_tokens(size: RadixSize) -> xgpui_luma::theme::ThemeTokens {
    xgpui_luma::theme::ThemeTokens {
        metrics: scaffold_metrics(size),
        stylesheet: std::sync::Arc::new(scaffold_stylesheet(size).clone()),
        ..Default::default()
    }
}
