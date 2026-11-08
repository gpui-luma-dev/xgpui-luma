use xgpui_luma::controls::button;
use xgpui_luma::controls::radio_group;
use xgpui_luma::controls::radio_button;
use xgpui_luma::prelude::HasPresenter;
use xgpui_luma::motion::{Transition, Easing};
use xgpui_luma::controls::radio_group::RadioGroupEventExt;
use std::time::Duration;

use gpui::{App, Context, Entity, IntoElement, Render, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::{
    controls::button::{Button},
    motion::VisualTransition,
    controls::radio_group::{RadioGroup, RadioGroupEvent, RadioGroupItem},
    hstack, vstack,
};
use crate::theme::AppTheme;
use xgpui_luma_plot::{
    CartesianChartBuilder, Dataset, DatasetError, Interpolation, LineLayer, NumericScale, Sample, Series,
};

use crate::theme::{self, TypographyExt};

pub(super) struct InterpolationExample {
    entrance: f32,
    data: Dataset,
    look: AppTheme,
    interpolation: Interpolation,
    selector: RadioGroup<RadioGroupItem>,
    replay: Entity<Button<()>>,
    motion: VisualTransition,
    _subscriptions: Vec<Subscription>,
}

impl InterpolationExample {
    pub(super) fn spawn(look: &AppTheme, cx: &mut App) -> Result<Entity<Self>, DatasetError> {
        // Sparse, irregular observations: broad changes reveal curvature between markers.
        let data = Dataset::new().with_series(Series::new(
            "level",
            "Reservoir level",
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
            let interpolation = Interpolation::Monotone;
            let selector = radio_group::horizontal("reservoir-interpolation")
                .single_required()
                .selection_follows_active(true)
                .items([
                    radio_group::RadioGroupItem::new("linear").label("Straight"),
                    radio_group::RadioGroupItem::new("monotone").label("Monotone"),
                ])
                .selected(match interpolation {
                    Interpolation::Linear => "linear",
                    Interpolation::Monotone => "monotone",
                })
                .template(radio_group::radio_group_buttons_template(
                    std::sync::Arc::new(radio_button::ThemedRadioButtonTemplate::new(std::sync::Arc::new(
                        look.clone(),
                    ))),
                    radio_group::RadioGroupLayout::Horizontal,
                ))
                .spawn(cx);
            selector.clone().on_event(cx, |this, event: &RadioGroupEvent, cx| {
                if let RadioGroupEvent::Change { value: Some(value) } = event {
                    this.interpolation = if value.as_ref() == "monotone" {
                        Interpolation::Monotone
                    } else {
                        Interpolation::Linear
                    };
                    cx.notify();
                }
            });
            let replay = button::Button::new("interpolation-replay")
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
                interpolation,
                selector,
                replay,
                motion: VisualTransition::with_transition(
                    1.0,
                    Transition::timed(Duration::from_millis(1400), Easing::OutCubic),
                ),
                _subscriptions: { Vec::new() },
            }
        }))
    }
}

impl Render for InterpolationExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.motion.sync();
        self.motion.schedule_frame(window, cx);
        let typography = theme::typography();
        let chrome = theme::chrome(&self.look);
        vstack! { gap=8;
            div().typography_style(typography.text.label).child("Interpolation · reservoir level"),
            div().typography_style(typography.text.caption).text_color(chrome.muted_text)
                .child("One series, ten irregular observations and a missing interval from 6.5–8 hours. Switch Straight / Monotone: markers stay fixed, the plateau stays flat, and the gap stays open. Replay draws the selected line from left to right."),
            hstack! { gap=8;
                div().typography_style(typography.text.caption).text_color(chrome.muted_text).child("Interpolation"),
                self.selector.clone(),
                self.replay.clone(),
            }.items_center().flex_shrink_0(),
            theme::panel_frame(&self.look, "plot-viz-interpolation-frame").h(px(360.0)).p_4().render(cx)
                .child(chart(self.data.clone(), &self.look, self.interpolation).draw_progress(self.motion.progress() * self.entrance).build()),
        }.w_full()
    }
}

fn chart(data: Dataset, look: &AppTheme, interpolation: Interpolation) -> CartesianChartBuilder {
    CartesianChartBuilder::new(data)
        .style(theme::SdkCartesianStyle::new(look))
        .x_scale(NumericScale::new().tick_values([0.0, 2.0, 4.0, 6.0, 8.0, 10.0, 12.0]))
        .y_scale(NumericScale::new().domain(0.0..=10.0).tick_count(6))
        .line(LineLayer::new("level").interpolation(interpolation))
        .style_override(|style| style.line.stroke.width = px(3.0))
        .style_override(|style| {
            style.axes[0].title = Some("Time (hours)".into());
            style.axes[1].title = Some("Level (m)".into());
        })
}

#[cfg(test)]
mod tests;

crate::examples::motion::entrance!(InterpolationExample);
