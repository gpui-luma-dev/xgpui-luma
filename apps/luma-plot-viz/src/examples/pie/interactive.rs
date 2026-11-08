use xgpui_luma::controls::button;
use xgpui_luma::controls::selector;
use xgpui_luma::controls::selector_list;
use xgpui_luma::prelude::HasPresenter;
use xgpui_luma::motion::{Transition, Easing};
use xgpui_luma::controls::selector::SelectorEventExt;
use std::time::Duration;
use gpui::{App, Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::{
    DockPanel, vstack, hstack,
    controls::{
        selector::{Selector, SelectorEvent},
        button::{Button},
    },
    motion::VisualTransition,
};
use crate::theme::AppTheme;
use xgpui_luma_plot::{Dataset, DatasetError, Series, Sample, PieChartBuilder, PieRingPresentation, PieSliceEvent};
use crate::theme::{self, TypographyExt};

pub(crate) const MONTHS: [&str; 6] = ["January", "February", "March", "April", "May", "June"];
const DESKTOP: [f64; 6] = [186.0, 305.0, 237.0, 73.0, 209.0, 214.0];
fn data() -> Result<Dataset, DatasetError> {
    let samples =
        |values: [f64; 6]| values.into_iter().enumerate().map(|(i, y)| Sample::new(i as f64, y)).collect::<Vec<_>>();
    Dataset::new()
        .with_series(Series::new("desktop", "Desktop", samples(DESKTOP)))?
        .with_series(Series::new("mobile", "Mobile", samples([80.0, 200.0, 120.0, 190.0, 130.0, 140.0])))
}
fn chart(data: Dataset, look: &AppTheme) -> PieChartBuilder {
    let live = look.clone();
    let tooltip_look = look.clone();
    PieChartBuilder::new(data, "desktop")
        .slice_names(MONTHS)
        .style(move || theme::pie::style(&live))
        .slice_style(5, crate::theme::palette::BLUE_LIGHT_SECONDARY)
        .slice_tooltip(move |slice| {
            xgpui_luma::controls::tooltip::Tooltip::new(format!("{} · {}", slice.label, slice.series))
                .shortcut(format!("{} visitors · {:.1}%", slice.sample.y, slice.fraction * 100.0))
                .theme(std::sync::Arc::new(tooltip_look.clone()))
                .delay(Duration::from_millis(250))
        })
        .tooltip_motion(Duration::from_millis(250))
}
pub(super) struct Stacked {
    selection: Option<xgpui_luma_plot::PieSlice>,
    entrance: f32,
    data: Dataset,
    look: AppTheme,
    replay: Entity<Button<()>>,
    motion: VisualTransition,
}
impl Stacked {
    pub(super) fn spawn(look: &AppTheme, cx: &mut App) -> Result<Entity<Self>, DatasetError> {
        let data = data()?;
        let look = look.clone();
        Ok(cx.new(move |cx: &mut Context<Self>| {
            let replay = button::Button::new("stacked-pie-replay")
                .template(std::sync::Arc::new(button::DefaultButtonTemplate::new(std::sync::Arc::new(look.clone()))))
                .label("Replay")
                .spawn(cx)
                .on_click(cx, |this, cx| {
                    this.motion.snap_to(0.0);
                    this.motion.set_target(1.0);
                    cx.notify();
                });
            Self {
                selection: None,
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
impl Render for Stacked {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        self.motion.sync();
        self.motion.schedule_frame(window, cx);
        let host = cx.weak_entity();
        let chart = chart(self.data.clone(), &self.look)
            .on_slice_event(move |event, cx| {
                if let PieSliceEvent::Select { slice } = event {
                    let _ = host.update(cx, |this, cx| {
                        this.selection = Some(slice);
                        cx.notify();
                    });
                }
            })
            .active_slice(self.selection.as_ref().filter(|s| s.ring_index.is_none()).map(|s| s.sample_index))
            .active_ring_slice(0, self.selection.as_ref().filter(|s| s.ring_index == Some(0)).map(|s| s.sample_index))
            .active_motion(Duration::from_millis(250))
            .id("pie-stacked-interaction")
            .outer_radius(0.65)
            .ring("mobile", PieRingPresentation::new(0.78, 1.0))
            .draw_progress(self.motion.progress() * self.entrance);
        let typography = theme::typography();
        let chrome = theme::chrome(&self.look);
        theme::panel_frame(&self.look, "pie-stacked-card")
            .h(px(460.0))
            .render(cx)
            .child(
                DockPanel::new()
                    .top(
                        hstack! { gap=8;
                            header("Pie Chart – Stacked", &self.look).flex_1(), self.replay.clone(),
                        }
                        .h(px(80.0))
                        .items_center()
                        .px_4()
                        .flex_shrink_0(),
                    )
                    .bottom(
                        div()
                            .h(px(54.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .typography_style(typography.text.caption)
                            .text_color(chrome.muted_text)
                            .child(
                                self.selection
                                    .as_ref()
                                    .map(|s| format!("Selected {} · {}", s.series, s.label))
                                    .unwrap_or_else(|| "Click either ring to select a month".into()),
                            ),
                    )
                    .fill(div().size_full().p_3().child(chart.build())),
            )
            .min_w_0()
    }
}
pub(super) struct Interactive {
    entrance: f32,
    data: Dataset,
    look: AppTheme,
    selected: usize,
    selector: Entity<Selector>,
}
impl Interactive {
    pub(super) fn spawn(look: &AppTheme, cx: &mut App) -> Result<Entity<Self>, DatasetError> {
        let data = data()?;
        let look = look.clone();
        Ok(cx.new(move |cx: &mut Context<Self>| {
            let selector = selector::Selector::new("pie-month-selector")
                .items(
                    MONTHS
                        .into_iter()
                        .enumerate()
                        .map(|(i, label)| selector::SelectorItem::new(i.to_string()).label(label)),
                )
                .selected_id("0")
                .template(std::sync::Arc::new(selector::ThemedSelectorTemplate::new(
                    std::sync::Arc::new(look.clone()),
                    selector_list::default_selector_items_template(),
                )))
                .spawn(cx);
            selector.clone().on_event(cx, |this, event: &SelectorEvent, cx| {
                if let SelectorEvent::Change { item_id, .. } = event
                    && let Ok(index) = item_id.parse::<usize>()
                    && index < MONTHS.len()
                {
                    this.selected = index;
                    cx.notify();
                }
            });
            Self { entrance: 1.0, data, look, selected: 0, selector }
        }))
    }
}
impl Render for Interactive {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let host = cx.weak_entity();
        let chart = chart(self.data.clone(), &self.look)
            .id("pie-interactive-selection")
            .draw_progress(self.entrance)
            .inner_radius(0.52)
            .active_slice(Some(self.selected))
            .active_motion(Duration::from_millis(250))
            .center_text(DESKTOP[self.selected].to_string(), "Visitors")
            .style_override(|style| {
                style.active_expansion = px(24.0);
                style.active_band_gap = Some(px(4.0));
            })
            .on_slice_event(move |event, cx| {
                if let PieSliceEvent::Select { slice } = event {
                    let _ = host.update(cx, |this, cx| {
                        this.selected = slice.sample_index;
                        this.selector.update(cx, |selector, cx| {
                            selector.set_selected_id(slice.sample_index.to_string(), cx);
                        });
                        cx.notify();
                    });
                }
            });
        theme::panel_frame(&self.look, "pie-interactive-card")
            .h(px(460.0))
            .render(cx)
            .child(
                DockPanel::new()
                    .top(header("Pie Chart – Interactive", &self.look).h(px(80.0)).px_4().flex_shrink_0())
                    .top(div().h(px(44.0)).px_4().flex().justify_end().child(self.selector.clone()).flex_shrink_0())
                    .fill(div().size_full().p_3().child(chart.build())),
            )
            .min_w_0()
    }
}
fn header(title: &'static str, look: &AppTheme) -> gpui::Div {
    let typography = theme::typography();
    let chrome = theme::chrome(look);
    vstack! { gap=4;
        div().typography_style(typography.text.label).child(title),
        div().typography_style(typography.text.caption).text_color(chrome.muted_text).child("January – June 2024"),
    }
    .justify_center()
}

crate::examples::motion::entrance!(Stacked);

crate::examples::motion::entrance!(Interactive);
