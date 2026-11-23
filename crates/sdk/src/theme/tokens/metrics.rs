use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MetricTokens {
    /// Default whole-control disabled opacity; family recipes may override it.
    #[serde(deserialize_with = "crate::theme::stylesheet::required_unit")]
    pub disabled_opacity: f32,
    /// Standard border width in pixels.
    #[serde(deserialize_with = "crate::theme::stylesheet::required_length")]
    pub border_width: f32,
    pub focus: FocusMetricTokens,
    /// The SDK control scale; the only size the SDK defines.
    pub control: ControlMetricTokens,
}

/// Corner radius, in pixels, large enough to fully round any control.
///
/// The SDK has no radius vocabulary; looks own named radii (none, small, medium,
/// large, full) and hand SDK controls plain pixels. Families whose SDK default is
/// fully rounded use this value.
pub const FULL_RADIUS_PX: f32 = 999.0;

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
    /// The SDK's metrics, from `theme/tokens.lss`.
    fn default() -> Self {
        super::from_lss::sdk(super::ThemeMode::Light).metrics
    }
}

impl ControlMetricTokens {
    pub fn new(height: f32, padding_x: f32, padding_y: f32, gap: f32, radius: f32, icon_size: f32) -> Self {
        Self { radius, height, control_height: height, padding_x, padding_y, gap, icon_size }
    }
}

impl MetricTokens {
    pub fn radius(&self) -> f32 {
        self.control.radius
    }

    pub fn control_height(&self) -> f32 {
        self.control.control_height
    }

    pub fn padding_x(&self) -> f32 {
        self.control.padding_x
    }

    pub fn padding_y(&self) -> f32 {
        self.control.padding_y
    }

    pub fn gap(&self) -> f32 {
        self.control.gap
    }

    pub fn icon_size(&self) -> f32 {
        self.control.icon_size
    }
}
