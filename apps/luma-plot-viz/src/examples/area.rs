use xgpui_luma::controls::button;
use xgpui_luma::prelude::HasPresenter;
use xgpui_luma::motion::{Transition, Easing};
use std::time::Duration;

use gpui::{App, Context, Entity, IntoElement, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::{
    controls::button::{Button},
    motion::VisualTransition,
    hstack, vstack,
};
use crate::theme::AppTheme;
use xgpui_luma_plot::{
    AreaLayer, CartesianChartBuilder, Dataset, DatasetError, Interpolation, NumericScale, Sample, Series,
};
use crate::theme::{self, TypographyExt};

pub(crate) struct AreaExample {
    entrance: f32,
    data: Dataset,
    look: AppTheme,
    replay: Entity<Button<()>>,
    motion: VisualTransition,
}

impl AreaExample {
    pub(crate) fn spawn(look: &AppTheme, cx: &mut App) -> Result<Entity<Self>, DatasetError> {
        let data = Dataset::new().with_series(Series::new(
            "inflow",
            "Reservoir inflow",
            [
                Sample::new(0.0, 2.0),
                Sample::new(1.0, 3.0),
                Sample::new(2.5, 8.0),
                Sample::new(4.0, 9.0),
                Sample::new(5.0, 9.0),
                Sample::new(6.5, 4.0),
                Sample::new(7.25, f64::NAN),
                Sample::new(8.0, 1.0),
                Sample::new(9.0, 2.0),
                Sample::new(10.5, 7.0),
                Sample::new(12.0, 8.0),
            ],
        ))?;
        let look = look.clone();
        Ok(cx.new(move |cx: &mut Context<Self>| {
            let replay = button::Button::new("area-replay")
                .template(std::sync::Arc::new(button::DefaultButtonTemplate::new(std::sync::Arc::new(look.clone()))))
                .label("Replay")
                .spawn(cx)
                .on_click(cx, |this, cx| {
                    this.motion.snap_to(0.0);
                    this.motion.set_target(1.0);
                    cx.notify();
                });
            Self {
                entrance: 1.0,
                data,
                look,
                replay,
                motion: VisualTransition::with_transition(
                    1.0,
                    Transition::timed(Duration::from_millis(1400), Easing::OutCubic),
                ),
            }
        }))
    }
}

impl Render for AreaExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.motion.sync();
        self.motion.schedule_frame(window, cx);
        let typography = theme::typography();
        let chrome = theme::chrome(&self.look);
        vstack! { gap=8;
            hstack! { gap=8;
                div().typography_style(typography.text.label).child("Area · gradient fill"),
                self.replay.clone(),
            }.items_center().flex_shrink_0(),
            div().typography_style(typography.text.caption).text_color(chrome.muted_text)
                .child("One smooth series filled to zero, with a gradient fading toward the baseline. Replay reveals the fill and outline together; the missing interval stays open."),
            theme::panel_frame(&self.look, "plot-viz-area-frame").h(px(360.0)).p_4().render(cx)
                .child(chart(self.data.clone(), &self.look).draw_progress(self.motion.progress() * self.entrance).build()),
        }.w_full()
    }
}

fn chart(data: Dataset, look: &AppTheme) -> CartesianChartBuilder {
    CartesianChartBuilder::new(data)
        .style(theme::SdkCartesianStyle::new(look))
        .x_scale(NumericScale::new().tick_count(7))
        .y_scale(NumericScale::new().tick_count(6))
        .area(AreaLayer::new("inflow").interpolation(Interpolation::Monotone))
        .style_override(|style| {
            style.axes[0].title = Some("Time (hours)".into());
            style.axes[1].title = Some("Inflow".into());
        })
}

pub(crate) struct AreaGallery {
    gradient: Entity<AreaExample>,
    visitors: Entity<super::visitors::VisitorsExample>,
    points: Entity<super::points::PointExamples>,
}
impl AreaGallery {
    pub(crate) fn spawn(look: &AppTheme, cx: &mut App) -> Result<Self, DatasetError> {
        Ok(Self {
            gradient: AreaExample::spawn(look, cx)?,
            visitors: super::visitors::VisitorsExample::spawn(look, true, cx)?,
            points: super::points::PointExamples::spawn(look, true, cx)?,
        })
    }
    pub(crate) fn render(&self, progress: f32, cx: &mut App) -> impl IntoElement {
        super::motion::display(&self.gradient, progress, cx);
        super::motion::display(&self.visitors, progress, cx);
        super::motion::display(&self.points, progress, cx);
        vstack! { gap=20; self.gradient.clone(), self.visitors.clone(), self.points.clone() }
            .w_full()
            .max_w(px(960.0))
    }
}

crate::examples::motion::entrance!(AreaExample);
