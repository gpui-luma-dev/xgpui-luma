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
}

fn fill(slot: &mut Option<f32>, value: f32) {
    slot.get_or_insert(value);
}
