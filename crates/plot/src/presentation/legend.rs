use gpui::{Background, Hsla};

use super::Stroke;

/// Presentation of the legend's visibility and text color.
#[derive(Clone, Copy, Debug)]
pub struct LegendPresentation {
    pub visible: bool,
    pub text_color: Hsla,
}

impl LegendPresentation {
    pub fn new(text_color: Hsla) -> Self {
        Self { visible: true, text_color }
    }
}

/// Visual sample for a legend entry, independent of its chart family.
/// Filled swatches use the legend's configured height; lines use their stroke width.
/// ```
/// use gpui::{black, white, px, linear_gradient, linear_color_stop};
/// use xgpui_luma_plot::{Stroke, SwatchPresentation};
/// let line = SwatchPresentation::Line(Stroke { color: black(), width: px(3.0) });
/// let solid = SwatchPresentation::Fill(black().into());
/// let gradient = SwatchPresentation::Fill(linear_gradient(
///     180.0, linear_color_stop(black(), 0.0), linear_color_stop(white(), 1.0),
/// ));
/// ```
#[derive(Clone, Copy, Debug)]
pub enum SwatchPresentation {
    /// A horizontal stroke; non-positive or non-finite widths hide it.
    Line(Stroke),
    /// Native solid or gradient paint.
    Fill(Background),
}
