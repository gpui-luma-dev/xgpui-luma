mod templates;
mod gradients;
mod labels;
mod scrolling;

use gpui::{App, Context, Entity, Render, Window, IntoElement, div, prelude::*, px};
use xgpui_luma::vstack;
use crate::theme::AppTheme;
use xgpui_luma_plot::{
    BarAlignment, BarGrouping, BarLayout, BarOrientation, BarLayer, CartesianChartBuilder, CategoryScale, Dataset,
    DatasetError, NumericScale, Sample, Series,
};
use crate::theme::{self, TypographyExt};

fn data() -> Result<Dataset, DatasetError> {
    Dataset::new()
        .with_series(Series::new(
            "desktop",
            "Desktop",
            [Sample::new(0.0, 32.0), Sample::new(1.0, 48.0), Sample::new(2.0, -12.0), Sample::new(3.0, 56.0)],
        ))?
        .with_series(Series::new(
            "mobile",
            "Mobile",
            [Sample::new(0.0, 24.0), Sample::new(1.0, 36.0), Sample::new(2.0, -8.0), Sample::new(3.0, 42.0)],
        ))
}
pub(crate) struct BarExample {
    selected: String,
    entrance: f32,
    scrolling: Entity<scrolling::ScrollingBars>,
    data: Dataset,
    label_data: Dataset,
    look: AppTheme,
}
impl BarExample {
    pub(crate) fn spawn(look: &AppTheme, cx: &mut App) -> Result<Entity<Self>, DatasetError> {
        let data = data()?;
        let label_data = labels::data()?;
        let scrolling = scrolling::ScrollingBars::spawn(look, cx)?;
        let look = look.clone();
        Ok(cx.new(move |_| Self {
            selected: "Click a bar to select its observation".into(),
            entrance: 1.0,
            data,
            label_data,
            look,
            scrolling,
        }))
    }

    fn chart(&self, orientation: BarOrientation, layout: BarLayout) -> CartesianChartBuilder {
        let horizontal = orientation.is_horizontal();
        let stacked = layout.grouping_policy() == BarGrouping::Stacked;
        CartesianChartBuilder::new(self.data.clone())
            .style(theme::SdkCartesianStyle::new(&self.look).blue().series("mobile", 1))
            .x_scale(CategoryScale::new(["Q1", "Q2", "Q3", "Q4"]))
            .y_scale(if stacked {
                NumericScale::new().domain(-25.0..=100.0).tick_values([-25.0, 0.0, 25.0, 50.0, 75.0, 100.0])
            } else {
                NumericScale::new().domain(-20.0..=60.0).tick_values([-20.0, 0.0, 20.0, 40.0, 60.0])
            })
            .bar_orientation(orientation)
            .bar_layout(layout)
            .style_override(move |style| {
                if horizontal {
                    style.grid.horizontal = None;
                } else {
                    style.grid.vertical = None;
                }
                style.metrics.legend_swatch_width = px(10.0);
                style.metrics.legend_swatch_height = px(10.0);
            })
            .bar(BarLayer::new("desktop"))
            .bar(BarLayer::new("mobile"))
    }

    fn tooltip_chart(&self, orientation: BarOrientation) -> CartesianChartBuilder {
        let look = self.look.clone();
        self.chart(orientation, BarLayout::new())
            .bar_template(templates::rounded)
            .bar_tooltip(move |point| {
                theme::points::tooltip(&look, point, format!("Q{}", point.sample.x as usize + 1))
                    .delay(std::time::Duration::from_millis(250))
            })
            .tooltip_motion(std::time::Duration::from_millis(250))
    }

    fn feature(
        &self,
        id: &'static str,
        title: &'static str,
        description: &'static str,
        charts: impl IntoIterator<Item = (&'static str, CartesianChartBuilder)>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let typography = theme::typography();
        vstack! { gap=8;
            div().typography_style(typography.text.label).child(title),
            div().typography_style(typography.text.caption).text_color(theme::chrome(&self.look).muted_text)
                .child(description),
            theme::panel_frame(&self.look, id).p_4().render(cx).child(
                div().grid().grid_cols(2).gap(px(20.0)).children(charts.into_iter().map(|(label, chart)| {
                    let host=cx.weak_entity();
                    let chart=chart.on_bar_event(move|event,cx|{if let xgpui_luma_plot::PlotPointEvent::Select{points}=event&&let Some(point)=points.first(){let text=format!("Selected {} · category {} · value {}",point.label,point.sample.x,point.sample.y);let _=host.update(cx,|this,cx|{this.selected=text;cx.notify();});}});
                    vstack! { gap=8;
                        div().typography_style(typography.text.caption).child(label),
                        div().h(px(240.0)).child(chart.id(format!("{id}-{label}")).draw_progress(self.entrance).build()),
                    }.min_w_0()
                })),
            ),
        }
        .w_full()
    }
}
impl Render for BarExample {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        super::motion::display(&self.scrolling, self.entrance, cx);
        let base = BarLayout::new();
        let up = BarOrientation::Up;
        vstack! { gap=24;
            self.feature("bar-tooltip-examples", "Motion tooltips", "Pause over a bar for 250 ms to see its category and value. The tooltip anchor slides on entry and smoothly retargets between bars.", [
                ("Vertical bars", self.tooltip_chart(up)),
                ("Horizontal bars", self.tooltip_chart(BarOrientation::Right)),
            ], cx),
            self.feature("bar-orientation-examples", "Orientation", "The same signed values grow up, down, left, or right. Negative values grow in the opposite direction.", [
                ("Up", self.chart(up, base)),
                ("Down", self.chart(BarOrientation::Down, base)),
                ("Left", self.chart(BarOrientation::Left, base)),
                ("Right", self.chart(BarOrientation::Right, base)),
            ], cx),
            self.feature("bar-grouping-examples", "Grouping and stacking", "Grouped series sit beside each other; stacks accumulate positive and negative values separately.", [
                ("Grouped", self.chart(up, base)),
                ("Stacked", self.chart(up, base.grouping(BarGrouping::Stacked))),
            ], cx),
            self.feature("bar-spacing-examples", "Width and spacing", "Group width controls the occupied category band. The series gap controls separation within a group.", [
                ("Narrow groups · width 0.5", self.chart(up, base.width(0.5))),
                ("Wide groups · width 0.95", self.chart(up, base.width(0.95))),
                ("Touching bars · gap 0", self.chart(up, base.gap(0.0))),
                ("Separated bars · gap 0.4", self.chart(up, base.gap(0.4))),
            ], cx),
            self.feature("bar-alignment-examples", "Category alignment", "A narrow group aligns to the start, center, or end of each category band.", [
                ("Start", self.chart(up, base.width(0.5).alignment(BarAlignment::Start))),
                ("Center", self.chart(up, base.width(0.5))),
                ("End", self.chart(up, base.width(0.5).alignment(BarAlignment::End))),
            ], cx),
            self.feature("bar-gradient-examples", "Gradient fills", "Each bar gets a native gradient fitted to its own bounds, using the current theme's series colors.", [
                ("Vertical fade", gradients::apply(self.chart(up, base), 180.0)),
                ("Horizontal fade · rounded bars", gradients::apply(self.chart(BarOrientation::Right, base), 90.0).bar_template(templates::rounded)),
            ], cx),
            self.feature("bar-template-examples", "Custom shape templates", "Templates receive resolved bounds and endpoints. These examples change only the shape, keeping data and layout identical.", [
                ("Rectangle", self.chart(up, base)),
                ("Rounded", self.chart(up, base).bar_template(templates::rounded)),
                ("Tapered", self.chart(up, base).bar_template(templates::tapered)),
            ], cx),
            div().typography_style(theme::typography().text.caption).child(self.selected.clone()),
            self.scrolling.clone(),
            labels::card(self.label_data.clone(), &self.look, self.entrance, cx),
        }.w_full().max_w(px(960.0))
    }
}

crate::examples::motion::entrance!(BarExample);
