mod interactive;
use gpui::{App, IntoElement, div, prelude::*, px};
use xgpui_luma::{DockPanel, vstack};
use crate::theme::AppTheme;
use xgpui_luma_plot::{Dataset, DatasetError, Sample, Series, PieChartBuilder, PieLabelPlacement};
use crate::theme::{self, TypographyExt};

const NAMES: [&str; 5] = ["Chrome", "Safari", "Firefox", "Edge", "Other"];
const VALUES: [f64; 5] = [275.0, 200.0, 187.0, 173.0, 90.0];
pub(crate) struct PieGallery {
    data: Dataset,
    stacked: gpui::Entity<interactive::Stacked>,
    interactive: gpui::Entity<interactive::Interactive>,
}
impl PieGallery {
    pub(crate) fn spawn(look: &AppTheme, cx: &mut App) -> Result<Self, DatasetError> {
        let samples: Vec<_> =
            VALUES.into_iter().enumerate().map(|(index, value)| Sample::new(index as f64, value)).collect();
        Ok(Self {
            data: Dataset::new().with_series(Series::new("browsers", "Visitors", samples))?,
            stacked: interactive::Stacked::spawn(look, cx)?,
            interactive: interactive::Interactive::spawn(look, cx)?,
        })
    }
    fn chart(&self, look: &AppTheme) -> PieChartBuilder {
        let live_look = look.clone();
        PieChartBuilder::new(self.data.clone(), "browsers")
            .slice_names(NAMES)
            .style(move || theme::pie::style(&live_look))
    }
    pub(crate) fn render(&self, look: &AppTheme, progress: f32, cx: &mut App) -> impl IntoElement {
        super::motion::display(&self.stacked, progress, cx);
        super::motion::display(&self.interactive, progress, cx);
        let cards = [
            ("pie-basic", "Pie Chart", self.chart(look)),
            ("pie-rotation", "Pie Chart – Start Angle", self.chart(look).start_angle(-90.0).clockwise(true)),
            (
                "pie-labels",
                "Pie Chart – Value Labels",
                self.chart(look)
                    .slice_labels(|slice| slice.sample.y.to_string().into())
                    .style_override(|style| style.label_lines = true),
            ),
            (
                "pie-custom-label",
                "Pie Chart – Custom Label",
                self.chart(look).slice_labels(|slice| format!("{:.0} visits", slice.sample.y).into()),
            ),
            (
                "pie-label-list",
                "Pie Chart – Label List",
                self.chart(look).slice_labels(|slice| slice.label.clone()).style_override(|style| {
                    style.label_placement = PieLabelPlacement::Inside;
                    style.label_text.color = crate::theme::palette::LABEL_BLACK;
                }),
            ),
            ("pie-legend", "Pie Chart – Legend", self.chart(look).legend(true)),
            ("pie-donut", "Pie Chart – Donut", self.chart(look).inner_radius(0.62)),
            (
                "pie-donut-active",
                "Pie Chart – Donut Active",
                self.chart(look).inner_radius(0.62).active_slice(Some(0)),
            ),
            (
                "pie-donut-text",
                "Pie Chart – Donut with Text",
                self.chart(look).inner_radius(0.62).center_text("925", "Visitors"),
            ),
        ];
        div()
            .w_full()
            .max_w(px(960.0))
            .grid()
            .grid_cols(2)
            .gap(px(20.0))
            .children(
                cards.into_iter().map(|(id, title, chart)| card(id, title, chart.draw_progress(progress), look, cx)),
            )
            .child(self.stacked.clone())
            .child(self.interactive.clone())
    }
}
fn card(id: &'static str, title: &'static str, chart: PieChartBuilder, look: &AppTheme, cx: &App) -> impl IntoElement {
    let typography = theme::typography();
    let chrome = theme::chrome(look);
    theme::panel_frame(look, id).h(px(420.0)).render(cx).child(
        DockPanel::new()
            .top(vstack! { gap=4;
                div().typography_style(typography.text.label).child(title),
                div().typography_style(typography.text.caption).text_color(chrome.muted_text).child("January – June 2024"),
            }.h(px(72.0)).flex_shrink_0().justify_center().px_4())
            .bottom(vstack! { gap=4;
                div().typography_style(typography.text.caption).child("925 total visitors"),
                div().typography_style(typography.text.caption).text_color(chrome.muted_text).child("Browser share · deterministic sample data"),
            }.h(px(68.0)).flex_shrink_0().justify_center().items_center())
            .fill(div().size_full().px_3().typography_style(typography.text.caption).child(chart.id(id).build())),
    ).min_w_0()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Context, Render, TestAppContext, Window};
    struct Harness {
        gallery: PieGallery,
        look: AppTheme,
    }
    impl Render for Harness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            self.gallery.render(&self.look, 1.0, cx)
        }
    }
    #[gpui::test]
    fn pie_gallery_draws_all_variants_and_preserves_reference_weights(cx: &mut TestAppContext) {
        let (host, cx) = cx.add_window_view(|_, cx| {
            let look = AppTheme::new();
            Harness { gallery: PieGallery::spawn(&look, cx).unwrap(), look }
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        host.read_with(cx, |host, _| {
            let samples = host.gallery.data.series()[0].samples();
            assert_eq!(samples.iter().map(|sample| sample.y).sum::<f64>(), 925.0);
            assert_eq!(samples.len(), NAMES.len());
        });
    }
}
