use gpui::px;
use crate::theme::AppTheme;
use xgpui_luma_plot::{CartesianChartBuilder, Dataset, Decimation};

use super::{LineExample, dense};
use crate::theme;

pub(super) fn example(data: Dataset, look: &AppTheme) -> LineExample {
    LineExample {
        title: "Decimated time series · same 20,000 samples",
        description: "The same data and axes as the raw plot above. Min/max decimation retains endpoints and peaks per pixel column; missing intervals stay disconnected. Resize the window to change drawing resolution.",
        frame: theme::panel_frame(look, "plot-viz-decimated-frame"),
        height: px(360.0),
        data,
        chart,
        diagnostics: None,
    }
}

fn chart(data: Dataset, look: &AppTheme) -> CartesianChartBuilder {
    dense::chart_with_decimation(data, look, Decimation::MinMax)
}
