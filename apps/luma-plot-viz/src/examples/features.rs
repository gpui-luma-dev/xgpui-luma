//! Focused host examples for viewport actions, annotations, log scales and legends.
use xgpui_luma::controls::button;
use xgpui_luma::prelude::HasPresenter;
use gpui::{App, Context, Entity, Render, Window, IntoElement, div, px, prelude::*};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::{
    vstack, hstack,
    controls::button::{Button},
};
use crate::theme::AppTheme;
use xgpui_luma_plot::*;
use crate::theme::{self, TypographyExt};
pub(crate) struct Features {
    data: Dataset,
    look: AppTheme,
    viewport: Viewport,
    buttons: Vec<Entity<Button<()>>>,
    hidden: Vec<usize>,
    selected: String,
    entrance: f32,
}
impl Features {
    pub(crate) fn spawn(look: &AppTheme, cx: &mut App) -> Result<Entity<Self>, DatasetError> {
        let data = Dataset::new()
            .with_series(Series::new(
                "a",
                "Desktop",
                (0..101)
                    .map(|i| Sample::new(i as f64, 20.0 + ((i as f64 / 10.0).sin() + 1.0) * 20.0))
                    .collect::<Vec<_>>(),
            ))?
            .with_series(Series::new(
                "b",
                "Mobile",
                (0..101)
                    .map(|i| Sample::new(i as f64, 10.0 + ((i as f64 / 8.0).cos() + 1.0) * 10.0))
                    .collect::<Vec<_>>(),
            ))?;
        let look = look.clone();
        Ok(cx.new(move |cx| {
            let mut buttons = Vec::new();

            for (index, (id, label)) in [
                ("viewport-in", "Zoom in"),
                ("viewport-out", "Zoom out"),
                ("viewport-left", "Pan left"),
                ("viewport-right", "Pan right"),
                ("viewport-reset", "Reset"),
            ]
            .into_iter()
            .enumerate()
            {
                let button = button::Button::new(id)
                    .template(std::sync::Arc::new(button::DefaultButtonTemplate::new(std::sync::Arc::new(
                        look.clone(),
                    ))))
                    .label(label)
                    .spawn(cx)
                    .on_click(cx, move |this: &mut Self, cx| {
                        let change = match index {
                            0 => this.viewport.zoom_at(1.5, 0.5, 0.5),
                            1 => this.viewport.zoom_at(1.0 / 1.5, 0.5, 0.5),
                            2 => this.viewport.pan(-10.0, 0.0),
                            3 => this.viewport.pan(10.0, 0.0),
                            _ => this.viewport.reset(),
                        };
                        if change.is_some() {
                            cx.notify();
                        }
                    });
                buttons.push(button);
            }
            // Known finite sample limits; construction does not depend on external input.
            let viewport = Viewport::new(20.0..=80.0, 1.0..=80.0)
                .and_then(|v| v.limits(0.0..=100.0, 1.0..=100.0))
                .expect("constant viewport bounds are finite and ordered");
            Self {
                data,
                look,
                viewport,
                buttons,
                hidden: Vec::new(),
                selected: "Click a marker to select an observation".into(),
                entrance: 1.0,
            }
        }))
    }
}
impl Render for Features {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = theme::chrome(&self.look);
        let typography = theme::typography();
        let host = cx.weak_entity();
        let items = ["Desktop", "Mobile"]
            .into_iter()
            .zip(theme::palette::blue(self.look.mode()))
            .map(|(name, color)| LegendItem { label: name.into(), swatch: SwatchPresentation::Fill(color.into()) });
        let legend = LegendView::new(items, LegendPresentation::new(chrome.body_text), ChartMetrics::default())
            .interaction(LegendInteraction::default().hidden(self.hidden.clone()).on_activate(move |event, cx| {
                let _ = host.update(cx, |this, cx| {
                    if this.hidden.contains(&event.index) {
                        this.hidden.retain(|i| *i != event.index);
                    } else {
                        this.hidden.push(event.index);
                    }
                    cx.notify();
                });
            }));
        let host = cx.weak_entity();
        let mut chart = CartesianChartBuilder::new(self.data.clone())
            .id("viewport-annotation-chart")
            .style(theme::SdkCartesianStyle::new(&self.look).blue().series("b", 1))
            .viewport(&self.viewport)
            .legend(LegendPresentation { visible: false, text_color: chrome.body_text })
            .draw_progress(self.entrance)
            .annotation(ChartAnnotation::new(
                Annotation::Band { from: Sample::new(40.0, 1.0), to: Sample::new(60.0, 80.0) },
                AnnotationPresentation::default(),
            ))
            .annotation(ChartAnnotation::new(
                Annotation::Text(Sample::new(40.0, 70.0)),
                AnnotationPresentation {
                    text: Some("Reference interval".into()),
                    text_style: TextPresentation::new(chrome.body_text),
                    ..Default::default()
                },
            ))
            .on_point_event(move |event, cx| {
                if let PlotPointEvent::Select { points } = event
                    && let Some(point) = points.first()
                {
                    let text = format!("Selected {} · x {} · value {}", point.label, point.sample.x, point.sample.y);
                    let _ = host.update(cx, |this, cx| {
                        this.selected = text;
                        cx.notify();
                    });
                }
            });
        if !self.hidden.contains(&0) {
            chart = chart.line(LineLayer::new("a"));
        }
        if !self.hidden.contains(&1) {
            chart = chart.line(LineLayer::new("b"));
        }
        let logarithmic = CartesianChartBuilder::new(self.data.clone())
            .id("log-scale-chart")
            .style(theme::SdkCartesianStyle::new(&self.look).blue())
            .x_scale(NumericScale::new().domain(0.0..=100.0))
            .y_scale(NumericScale::new().logarithmic(10.0).domain(1.0..=100.0))
            .line(LineLayer::new("a"))
            .draw_progress(self.entrance)
            .annotation(ChartAnnotation::new(
                Annotation::Line { from: Sample::new(0.0, 10.0), to: Sample::new(100.0, 10.0) },
                AnnotationPresentation::default(),
            ));
        vstack!{gap=12;
            div().typography_style(typography.text.label).child("Viewport · annotations · selectable legend"),
            hstack!{gap=8;}.flex_wrap().children(self.buttons.iter().cloned()),
            div().typography_style(typography.text.caption).text_color(chrome.muted_text).child("Pan and zoom keep data unchanged. Click legend items to toggle series."),
            legend,
            theme::panel_frame(&self.look,"viewport-feature-frame").h(px(320.0)).p_4().render(cx).child(chart.build()),
            div().typography_style(typography.text.caption).child(self.selected.clone()),
            div().typography_style(typography.text.label).child("Logarithmic Y · reference line at 10"),
            theme::panel_frame(&self.look,"log-feature-frame").h(px(300.0)).p_4().render(cx).child(logarithmic.build()),
        }.w_full()
    }
}
crate::examples::motion::entrance!(Features);

#[cfg(test)]
mod tests {
    use xgpui_luma::controls::button::ButtonEvent;
    use super::*;
    #[gpui::test]
    fn sdk_zoom_and_reset_preserve_dataset_and_annotations_draw(cx: &mut gpui::TestAppContext) {
        struct Host {
            feature: Entity<Features>,
        }
        impl Render for Host {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                self.feature.clone()
            }
        }
        let (host, cx) = cx.add_window_view(|_, cx| Host { feature: Features::spawn(&AppTheme::new(), cx).unwrap() });
        let feature = host.read_with(cx, |host, _| host.feature.clone());
        feature.update(cx, |f, cx| {
            f.buttons[0].update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        });
        feature.read_with(cx, |f, _| {
            assert!(f.viewport.x().end() - f.viewport.x().start() < 60.0);
            assert_eq!(f.data.series()[0].samples().len(), 101);
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        feature.update(cx, |f, cx| {
            f.buttons[4].update(cx, |_, cx| cx.emit(ButtonEvent::Click));
        });
        feature.read_with(cx, |f, _| assert_eq!(f.viewport.x(), 20.0..=80.0));
    }
}
