use crate::theme::MetricTokens;

/// Rounds a logical length to the nearest physical pixel at the given display scale.
pub fn snap_to_pixel(value: f32, scale_factor: f32) -> f32 {
    (value * scale_factor).round() / scale_factor
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StandardBoxScale {
    pub height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub radius: f32,
    pub icon_size: f32,
}

impl StandardBoxScale {
    pub fn compute(metrics: &MetricTokens, scale_factor: f32) -> Self {
        let control_height = metrics.control_height();

        Self {
            height: snap_to_pixel(control_height, scale_factor),
            padding_x: snap_to_pixel(metrics.padding_x(), scale_factor),
            padding_y: snap_to_pixel(metrics.padding_y(), scale_factor),
            gap: snap_to_pixel(metrics.gap(), scale_factor),
            radius: metrics.radius(),
            icon_size: snap_to_pixel(metrics.icon_size(), scale_factor),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ListRowScale {
    pub min_height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub radius: f32,
    pub gap: f32,
    pub label_baseline_shift: f32,
}

impl ListRowScale {
    pub fn compute(metrics: &MetricTokens, scale_factor: f32) -> Self {
        Self {
            min_height: snap_to_pixel(metrics.control_height(), scale_factor),
            padding_x: snap_to_pixel(metrics.padding_x(), scale_factor),
            padding_y: snap_to_pixel(metrics.padding_y(), scale_factor),
            radius: metrics.radius(),
            gap: snap_to_pixel(metrics.gap(), scale_factor),
            label_baseline_shift: label_baseline_shift(),
        }
    }
}

pub(crate) fn label_baseline_shift() -> f32 {
    crate::theme::stylesheet::SharedGeometry::sdk().label_baseline
}

/// Outer reach of a box shadow layer used to position absolute shadow backing.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ShadowProjectionInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl ShadowProjectionInsets {
    pub fn compute(offset_x: f32, offset_y: f32, blur: f32, spread: f32) -> Self {
        let reach = (blur.max(0.0) + spread).max(0.0);
        Self {
            top: (reach - offset_y).ceil().max(0.0),
            right: (reach + offset_x).ceil().max(0.0),
            bottom: (reach + offset_y).ceil().max(0.0),
            left: (reach - offset_x).ceil().max(0.0),
        }
    }

    pub fn union(self, other: Self) -> Self {
        Self {
            top: self.top.max(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
            left: self.left.max(other.left),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ShadowProjectionInsets, snap_to_pixel};

    #[test]
    fn snap_to_pixel_snaps_at_1x() {
        assert_eq!(snap_to_pixel(10.4, 1.0), 10.0);
        assert_eq!(snap_to_pixel(10.6, 1.0), 11.0);
    }

    #[test]
    fn snap_to_pixel_snaps_at_1_5x() {
        assert_eq!(snap_to_pixel(10.2, 1.5), 10.0);
        assert!((snap_to_pixel(10.4, 1.5) - (16.0 / 1.5)).abs() < 0.000_001);
    }

    #[test]
    fn snap_to_pixel_snaps_at_2x() {
        assert_eq!(snap_to_pixel(10.2, 2.0), 10.0);
        assert_eq!(snap_to_pixel(10.3, 2.0), 10.5);
    }

    #[test]
    fn shadow_projection_insets_accounts_for_offset_and_blur() {
        let insets = ShadowProjectionInsets::compute(0.0, 5.0, 5.0, 0.0);
        assert_eq!(insets.top, 0.0);
        assert_eq!(insets.bottom, 10.0);
        assert_eq!(insets.left, 5.0);
        assert_eq!(insets.right, 5.0);
    }
}
