use super::ControlSize;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MetricTokens {
    /// Default whole-control disabled opacity; family recipes may override it.
    #[serde(deserialize_with = "crate::theme::stylesheet::required_unit")]
    pub disabled_opacity: f32,
    pub spacing: SpacingTokens,
    pub radius: RadiusTokens,
    pub border_width: BorderWidthTokens,
    pub focus: FocusMetricTokens,
    pub control: ControlMetricScale,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SpacingTokens {
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub s0: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub s1: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub s2: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub s3: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub s4: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub s5: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub s6: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RadiusTokens {
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub none: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub sm: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub md: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub lg: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub xl: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub pill: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BorderWidthTokens {
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub hairline: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub default: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub strong: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FocusMetricTokens {
    /// Visual gap between a control and its focus ring, in pixels.
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub gap: f32,
    /// Focus-ring stroke width, independent of the control's face border.
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub width: f32,
    /// Additional outward displacement, added to `gap` and clamped at zero.
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub offset: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ControlMetricScale {
    pub sm: ControlMetricTokens,
    pub md: ControlMetricTokens,
    pub lg: ControlMetricTokens,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ControlMetricTokens {
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub radius: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub height: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub control_height: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub padding_x: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub padding_y: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub gap: f32,
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub icon_size: f32,
}

impl Default for MetricTokens {
    fn default() -> Self {
        Self::native_default()
    }
}

impl ControlMetricTokens {
    pub fn new(height: f32, padding_x: f32, padding_y: f32, gap: f32, radius: f32, icon_size: f32) -> Self {
        Self { radius, height, control_height: height, padding_x, padding_y, gap, icon_size }
    }
}

impl MetricTokens {
    /// Construct the bundled SDK metrics without parsing theme data.
    pub fn native_default() -> Self {
        Self {
            disabled_opacity: 0.56,
            spacing: SpacingTokens { s0: 0.0, s1: 4.0, s2: 6.0, s3: 8.0, s4: 12.0, s5: 16.0, s6: 24.0 },
            radius: RadiusTokens { none: 0.0, sm: 3.0, md: 6.0, lg: 8.0, xl: 12.0, pill: 999.0 },
            border_width: BorderWidthTokens { hairline: 0.5, default: 1.0, strong: 2.0 },
            focus: FocusMetricTokens { gap: 1.0, width: 1.0, offset: 0.0 },
            control: ControlMetricScale {
                sm: ControlMetricTokens::new(28.0, 10.0, 5.0, 6.0, 5.0, 14.0),
                md: ControlMetricTokens::new(36.0, 14.0, 8.0, 8.0, 6.0, 16.0),
                lg: ControlMetricTokens::new(44.0, 18.0, 10.0, 10.0, 8.0, 18.0),
            },
        }
    }

    pub fn for_size(&self, size: ControlSize) -> ControlMetricTokens {
        match size {
            ControlSize::Sm => self.control.sm,
            ControlSize::Md => self.control.md,
            ControlSize::Lg => self.control.lg,
        }
    }

    pub fn radius(&self, size: ControlSize) -> f32 {
        self.for_size(size).radius
    }

    pub fn control_height(&self, size: ControlSize) -> f32 {
        self.for_size(size).control_height
    }

    pub fn padding_x(&self, size: ControlSize) -> f32 {
        self.for_size(size).padding_x
    }

    pub fn padding_y(&self, size: ControlSize) -> f32 {
        self.for_size(size).padding_y
    }

    pub fn gap(&self, size: ControlSize) -> f32 {
        self.for_size(size).gap
    }

    pub fn icon_size(&self, size: ControlSize) -> f32 {
        self.for_size(size).icon_size
    }
}
