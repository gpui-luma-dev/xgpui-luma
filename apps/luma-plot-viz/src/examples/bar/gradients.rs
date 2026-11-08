use gpui::{Hsla, Background, linear_gradient, linear_color_stop};
use xgpui_luma_plot::CartesianChartBuilder;

/// Native gradients resolve from the live series palette and fit each bar's bounds.
pub(super) fn apply(chart: CartesianChartBuilder, angle: f32) -> CartesianChartBuilder {
    chart.style_override(move |style| {
        style.bar.fill = fill(style.line.stroke.color, angle);
        for series in style.series.values_mut() {
            if let (Some(line), Some(bar)) = (series.line, &mut series.bar) {
                bar.fill = fill(line.stroke.color, angle);
            }
        }
    })
}
fn fill(color: Hsla, angle: f32) -> Background {
    linear_gradient(angle, linear_color_stop(color.opacity(0.95), 0.0), linear_color_stop(color.opacity(0.2), 1.0))
}
