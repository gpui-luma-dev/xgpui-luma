use xgpui_luma::controls::radio_group;
use xgpui_luma::controls::radio_button;
use xgpui_luma::controls::radio_group::RadioGroupEventExt;
use gpui::{App, Context, Entity, IntoElement, Render, Subscription, Window, div, prelude::*, px};
use xgpui_luma::{
    controls::radio_group::{RadioGroup, RadioGroupEvent, RadioGroupItem},
    controls::tooltip::TooltipPlacement,
    hstack, vstack,
};
use crate::theme::AppTheme;
use xgpui_luma_plot::{
    AreaLayer, CartesianChartBuilder, Dataset, DatasetError, Interpolation, LineLayer, MarkerPresentation,
    MarkerVisibility, NumericScale, Sample, Series, PlotPointEvent,
};

use crate::theme::{self, TypographyExt};

pub(crate) struct PointExamples {
    entrance: f32,
    data: Dataset,
    look: AppTheme,
    area: bool,
    selector: RadioGroup<RadioGroupItem>,
    visibility: MarkerVisibility,
    hovered: String,
    picked: String,
    motion_selector: RadioGroup<RadioGroupItem>,
    motion: std::time::Duration,
    placement_selector: RadioGroup<RadioGroupItem>,
    placement: TooltipPlacement,
    _subscriptions: Vec<Subscription>,
}

impl PointExamples {
    pub(crate) fn spawn(look: &AppTheme, area: bool, cx: &mut App) -> Result<Entity<Self>, DatasetError> {
        let data = Dataset::new()
            .with_series(Series::new("desktop", "Desktop", observations([3.0, 7.0, 5.0, 9.0, 4.0, 6.0])))?
            .with_series(Series::new("mobile", "Mobile", observations([1.0, 3.0, 2.0, 5.0, 2.0, 4.0])))?;
        let look = look.clone();
        Ok(cx.new(move |cx: &mut Context<Self>| {
            let selector = radio_group::horizontal("sparse-point-mode")
                .single_required()
                .selection_follows_active(true)
                .items([
                    radio_group::RadioGroupItem::new("hover").label("Hover point"),
                    radio_group::RadioGroupItem::new("slice").label("Hover slice"),
                    radio_group::RadioGroupItem::new("all").label("All points"),
                ])
                .selected("slice")
                .template(radio_group::radio_group_buttons_template(
                    std::sync::Arc::new(radio_button::ThemedRadioButtonTemplate::new(std::sync::Arc::new(
                        look.clone(),
                    ))),
                    radio_group::RadioGroupLayout::Horizontal,
                ))
                .spawn(cx);
            selector.clone().on_event(cx, |this, event: &RadioGroupEvent, cx| {
                if let RadioGroupEvent::Change { value: Some(value) } = event {
                    this.visibility = theme::points::visibility(value.as_ref());
                    cx.notify();
                }
            });
            let motion_selector = radio_group::horizontal("point-tooltip-motion")
                .single_required()
                .selection_follows_active(true)
                .items([
                    radio_group::RadioGroupItem::new("snap").label("Immediate"),
                    radio_group::RadioGroupItem::new("smooth").label("Smooth"),
                ])
                .selected("smooth")
                .template(radio_group::radio_group_buttons_template(
                    std::sync::Arc::new(radio_button::ThemedRadioButtonTemplate::new(std::sync::Arc::new(
                        look.clone(),
                    ))),
                    radio_group::RadioGroupLayout::Horizontal,
                ))
                .spawn(cx);
            motion_selector.clone().on_event(cx, |this, event: &RadioGroupEvent, cx| {
                if let RadioGroupEvent::Change { value: Some(value) } = event {
                    this.motion = theme::points::motion(value.as_ref());
                    cx.notify();
                }
            });
            let placement_selector = radio_group::horizontal("point-tooltip-placement")
                .single_required()
                .selection_follows_active(true)
                .items([
                    radio_group::RadioGroupItem::new("above").label("Above"),
                    radio_group::RadioGroupItem::new("below").label("Below"),
                ])
                .selected("above")
                .template(radio_group::radio_group_buttons_template(
                    std::sync::Arc::new(radio_button::ThemedRadioButtonTemplate::new(std::sync::Arc::new(
                        look.clone(),
                    ))),
                    radio_group::RadioGroupLayout::Horizontal,
                ))
                .spawn(cx);
            placement_selector.clone().on_event(cx, |this, event: &RadioGroupEvent, cx| {
                if let RadioGroupEvent::Change { value: Some(value) } = event {
                    this.placement = theme::points::placement(value.as_ref());
                    cx.notify();
                }
            });
            Self {
                entrance: 1.0,
                data,
                look,
                area,
                selector,
                visibility: MarkerVisibility::HoverSlice,
                hovered: "None".into(),
                picked: "None".into(),
                motion_selector,
                motion: std::time::Duration::from_millis(250),
                placement_selector,
                placement: TooltipPlacement::Above,
                _subscriptions: { Vec::new() },
            }
        }))
    }

    fn observe(chart: CartesianChartBuilder, cx: &Context<Self>) -> CartesianChartBuilder {
        let host = cx.entity().downgrade();
        chart.on_point_event(move |event, cx| {
            let _ = host.update(cx, |this, cx| {
                let (target, points) = match event {
                    PlotPointEvent::HoverChanged { points } => (&mut this.hovered, points),
                    PlotPointEvent::Select { points } => (&mut this.picked, points),
                };
                *target = if points.is_empty() {
                    "None".to_owned()
                } else {
                    points
                        .iter()
                        .map(|point| format!("{} {}: {}", MONTHS[point.sample.x as usize], point.label, point.sample.y))
                        .collect::<Vec<_>>()
                        .join(" · ")
                };
                cx.notify();
            });
        })
    }
}

impl Render for PointExamples {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let typography = theme::typography();
        let chrome = theme::chrome(&self.look);
        let chart = Self::observe(
            chart(self.data.clone(), &self.look, self.visibility, self.area, self.motion, self.placement),
            cx,
        );
        vstack! { gap=8;
            div().typography_style(typography.text.label).child("Point markers and SDK tooltips"),
            div().typography_style(typography.text.caption).text_color(chrome.muted_text)
                .child("Six observations per series. Hover point targets one nearby observation; Hover slice selects matching observations anywhere in the plot. All points keeps every marker visible; hover a marker for its tooltip. Click a point or slice to pick it."),
            hstack! { self.selector.clone() }.flex_shrink_0(),
            hstack! { gap=8;
                div().typography_style(typography.text.caption).child("Tooltip motion"),
                self.motion_selector.clone(),
                div().typography_style(typography.text.caption).child("Placement"),
                self.placement_selector.clone(),
            }.items_center().flex_shrink_0(),
            div().typography_style(typography.text.caption).child(if self.area { "Area" } else { "Line" }),
            theme::panel_frame(&self.look, if self.area { "plot-viz-area-points-frame" } else { "plot-viz-line-points-frame" }).h(px(260.0)).p_4().render(cx)
                .child(chart.draw_progress(self.entrance).build()),
            div().typography_style(typography.text.caption).child(format!("Hover: {} · Picked: {}", self.hovered, self.picked)),
        }.w_full()
    }
}

fn observations(values: [f64; 6]) -> Vec<Sample> {
    values.into_iter().enumerate().map(|(index, value)| Sample::new(index as f64, value)).collect()
}

const MONTHS: [&str; 6] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun"];

fn chart(
    data: Dataset,
    look: &AppTheme,
    visibility: MarkerVisibility,
    area: bool,
    motion: std::time::Duration,
    placement: TooltipPlacement,
) -> CartesianChartBuilder {
    let tooltip_look = look.clone();
    let mut chart = CartesianChartBuilder::new(data)
        .style(theme::SdkCartesianStyle::new(look).blue().series("desktop", 0).series("mobile", 1))
        .tooltip_motion(motion)
        .x_scale(NumericScale::new().tick_labels(MONTHS.into_iter().enumerate().map(|(i, label)| (i as f64, label))))
        .y_scale(NumericScale::new().domain(0.0..=10.0).tick_count(3))
        .slice_tooltip(move |slice| {
            theme::points::slice_tooltip(&tooltip_look, slice, MONTHS[slice.coordinate as usize].to_owned())
                .placement(placement)
        });
    chart = chart.style_override(move |style| {
        for series in style.series.values_mut() {
            if let Some(line) = &mut series.line {
                line.markers = Some(MarkerPresentation::new(line.stroke.color, px(5.0)).visibility(visibility));
            }
            if let Some(area) = &mut series.area
                && let Some(line) = &mut area.outline
            {
                line.markers = Some(MarkerPresentation::new(line.stroke.color, px(5.0)).visibility(visibility));
            }
        }
    });
    for id in ["desktop", "mobile"] {
        chart = if area {
            chart.area(AreaLayer::new(id).interpolation(Interpolation::Monotone))
        } else {
            chart.line(LineLayer::new(id).interpolation(Interpolation::Monotone))
        };
    }
    chart
}

crate::examples::motion::entrance!(PointExamples);
