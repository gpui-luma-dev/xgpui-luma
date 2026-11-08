use gpui::{App, IntoElement, div, prelude::*, px};
use xgpui_luma::vstack;
use crate::theme::AppTheme;
use xgpui_luma_plot::{
    BarLabel, BarLabelPlacement, BarLayer, BarLayout, BarOrientation, CartesianChartBuilder, CategoryScale, Dataset,
    DatasetError, NumericScale, Sample, Series, TextPresentation,
};
use crate::theme::{self, TypographyExt};

const MONTHS: [&str; 6] = ["January", "February", "March", "April", "May", "June"];
pub(super) fn data() -> Result<Dataset, DatasetError> {
    Dataset::new().with_series(Series::new(
        "desktop",
        "Desktop",
        [186.0, 305.0, 237.0, 73.0, 209.0, 214.0]
            .into_iter()
            .enumerate()
            .map(|(index, value)| Sample::new(index as f64, value))
            .collect::<Vec<_>>(),
    ))
}
pub(super) fn card(data: Dataset, look: &AppTheme, progress: f32, cx: &App) -> impl IntoElement {
    let typography = theme::typography();
    let chrome = theme::chrome(look);
    let mut values = TextPresentation::new(chrome.body_text);
    values.font_family = typography.font.sans.family.into();
    values.font_size = px(14.0);
    values.line_height = px(20.0);
    let mut categories = values.clone();
    categories.color = crate::theme::palette::LABEL_BLACK;
    let tooltip_look = look.clone();
    let chart = CartesianChartBuilder::new(data)
        .id("bar-custom-labels-chart")
        .style(theme::SdkCartesianStyle::new(look).blue())
        .x_scale(CategoryScale::new(MONTHS))
        .y_scale(NumericScale::new().domain(0.0..=330.0).tick_values([0.0, 80.0, 160.0, 240.0, 320.0]))
        .bar_orientation(BarOrientation::Right)
        .bar_layout(BarLayout::new().width(0.8))
        .bar(BarLayer::new("desktop"))
        .bar_template(super::templates::rounded)
        .bar_labels(move |bar| {
            vec![
                BarLabel::new(MONTHS[bar.sample_index], categories.clone())
                    .placement(BarLabelPlacement::InsideStart)
                    .gap(px(10.0)),
                BarLabel::new(bar.sample.y.to_string(), values.clone()).gap(px(10.0)),
            ]
        })
        .bar_tooltip(move |point| {
            theme::points::tooltip(&tooltip_look, point, MONTHS[point.sample.x as usize].to_owned())
                .delay(std::time::Duration::from_millis(250))
        })
        .tooltip_motion(std::time::Duration::from_millis(250))
        .axes([])
        .style_override(|style| {
            style.bar.fill = crate::theme::palette::BLUE_BRIGHT.into();
            style.grid.horizontal = None;
            style.legend.visible = false;
            style.metrics.plot_padding = px(8.0);
        });
    theme::panel_frame(look, "bar-custom-labels-frame")
        .p_6()
        .render(cx)
        .child(vstack! { gap=12;
            div().typography_style(typography.text.scale.two_xl).child("Bar Chart – Custom Label"),
            div().typography_style(typography.text.body).text_color(chrome.muted_text).child("January – June 2024"),
            div().h(px(340.0)).mt_4().child(chart.draw_progress(progress).build()),
        })
        .w_full()
}
