use gpui::Hsla;
use xgpui_luma::color::ColorValue;

use super::super::color::interpolate_rgb;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GradientType {
    Linear,
    Radial,
    Conic,
}

impl GradientType {
    pub fn label(self) -> &'static str {
        match self {
            Self::Linear => "linear",
            Self::Radial => "radial",
            Self::Conic => "conic",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreviewRenderer {
    Quads,
    RenderImageSync,
    RenderImageAsync,
    ShaderPaint,
    ShaderBrush,
}

impl PreviewRenderer {
    pub fn is_shader(self) -> bool {
        matches!(self, Self::ShaderPaint | Self::ShaderBrush)
    }
}

/// A mesh/freeform control point. Hosts store `ControlPoint` (a source
/// [`ColorValue`]); renderers take [`PaintPoint`], produced only by
/// [`crate::gradient_builder::encode::encode_points`] for an explicit output.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlPoint<C = ColorValue> {
    pub row: u8,
    pub col: u8,
    pub u: f32,
    pub v: f32,
    pub color: C,
}

/// Paint-layer point: `color` is encoded RGB in the selected output primaries
/// (see [`crate::gradient_builder::encode::OutputSpace`]), carried as
/// `gpui::Hsla` for the existing interpolation math.
pub type PaintPoint = ControlPoint<Hsla>;

impl<C> ControlPoint<C> {
    /// Same geometry with a different color representation.
    pub fn with_color<D>(&self, color: D) -> ControlPoint<D> {
        ControlPoint { row: self.row, col: self.col, u: self.u, v: self.v, color }
    }
}

pub fn color_at_position(stops: &[(f32, Hsla)], position: f32) -> Hsla {
    if stops.is_empty() {
        return gpui::hsla(0.0, 0.0, 0.0, 1.0);
    }
    if stops.len() == 1 {
        return stops[0].1;
    }

    let position = position.clamp(0.0, 1.0);
    if position <= stops[0].0 {
        return stops[0].1;
    }
    if position >= stops[stops.len() - 1].0 {
        return stops[stops.len() - 1].1;
    }

    for window in stops.windows(2) {
        let (left_pos, left_color) = window[0];
        let (right_pos, right_color) = window[1];
        if position >= left_pos && position <= right_pos {
            let span = (right_pos - left_pos).max(f32::EPSILON);
            let t = (position - left_pos) / span;
            return interpolate_rgb(left_color, right_color, t);
        }
    }

    stops[stops.len() - 1].1
}

pub fn sorted_stops<C: Copy>(stops: &[(f32, C)]) -> Vec<(f32, C)> {
    let mut sorted = stops.to_vec();
    sorted.sort_by(|left, right| left.0.total_cmp(&right.0));
    sorted
}
