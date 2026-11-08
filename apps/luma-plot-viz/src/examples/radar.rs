use xgpui_luma::motion::{Transition, Easing};
use std::time::Duration;
use gpui::{App, Context, Entity, Render, Window, IntoElement, div, prelude::*, px};
use xgpui_luma::{DockPanel, vstack, motion::VisualTransition};
use crate::theme::AppTheme;
use xgpui_luma_plot::{Dataset, DatasetError, Series, Sample, RadarChartBuilder, RadarSeriesPresentation};
use crate::theme::{self, TypographyExt};
const MONTHS: [&str; 6] = ["January", "February", "March", "April", "May", "June"];
const DESKTOP: [f64; 6] = [186.0, 305.0, 237.0, 273.0, 209.0, 214.0];
const MOBILE: [f64; 6] = [80.0, 200.0, 120.0, 190.0, 130.0, 140.0];
pub(crate) struct RadarGallery {
    selected: String,
    data: Dataset,
    look: AppTheme,
    motion: VisualTransition,
}
impl RadarGallery {
    pub(crate) fn spawn(look: &AppTheme, cx: &mut App) -> Result<Entity<Self>, DatasetError> {
        let samples = |values: [f64; 6]| {
            values.into_iter().enumerate().map(|(i, y)| Sample::new(i as f64, y)).collect::<Vec<_>>()
        };
        let data = Dataset::new()
            .with_series(Series::new("desktop", "Desktop", samples(DESKTOP)))?
            .with_series(Series::new("mobile", "Mobile", samples(MOBILE)))?;
        let look = look.clone();
        Ok(cx.new(|_| Self {
            selected: "Click a month to select its observations".into(),
            data,
            look,
            motion: VisualTransition::with_transition(
                0.0,
                Transition::timed(Duration::from_millis(900), Easing::OutCubic),
            ),
        }))
    }
    pub(crate) fn enter(&mut self, cx: &mut Context<Self>) {
        self.motion.snap_to(0.0);
        self.motion.set_target(1.0);
        cx.notify();
    }
}
impl Render for RadarGallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.motion.sync();
        self.motion.schedule_frame(window, cx);
        let progress = self.motion.progress();
        let typography = theme::typography();
        let chrome = theme::chrome(&self.look);
        let titles = [
            "Radar Chart",
            "Radar Chart – Dots",
            "Radar Chart – Lines Only",
            "Radar Chart – Custom Label",
            "Radar Chart – Grid Custom",
            "Radar Chart – Grid None",
            "Radar Chart – Grid Circle",
            "Radar Chart – Grid Circle – No Lines",
            "Radar Chart – Grid Circle Filled",
            "Radar Chart – Grid Filled",
            "Radar Chart – Multiple",
            "Radar Chart – Legend",
        ];
        div().w_full().max_w(px(1200.0)).grid().grid_cols(2).gap(px(20.0)).children(titles.into_iter().enumerate().map(|(index,title)| {
            let look = self.look.clone();
            let tooltip_look = self.look.clone();
            let host=cx.weak_entity();
            let mut chart = RadarChartBuilder::new(self.data.clone(),"desktop").id(("radar-example",index)).categories(MONTHS).maximum(340.0).draw_progress(progress).style(move || {
                let mut style = theme::radar::style(&look);
                let mut desktop = RadarSeriesPresentation::new(crate::theme::palette::BLUE_PALE);
                desktop.stroke = None;
                if index == 1 || index == 5 || index == 6 || index == 7 { desktop.dot_radius = px(4.0); }
                if index == 2 {
                    desktop.fill = None; desktop.stroke = Some(xgpui_luma_plot::Stroke { color: desktop.dot_color, width: px(2.0) });
                    let mut mobile = RadarSeriesPresentation::new(crate::theme::palette::BLUE_BRIGHT); mobile.fill = None;
                    style.series.insert("mobile".into(),mobile);
                }
                if index == 3 { style.series.insert("mobile".into(),RadarSeriesPresentation::new(crate::theme::palette::BLUE_BRIGHT)); }
                if index == 4 { style.grid_levels = vec![1.0]; style.spokes = false; }
                if index == 5 { style.grid = None; }
                if (6..=8).contains(&index) { style.grid_shape = xgpui_luma_plot::RadarGridShape::Circle; }
                if index == 7 { style.spokes = false; }
                if index == 8 || index == 9 {
                    style.grid_fill = Some(desktop.dot_color.opacity(0.12).into());
                    style.grid = None; style.spokes = false;
                }
                if index == 10 || index == 11 {
                    style.series.insert("mobile".into(),RadarSeriesPresentation::new(crate::theme::palette::BLUE_BRIGHT));
                }
                if index == 11 { style.legend.visible = true; style.legend.text_color = theme::chrome(&look).body_text; }
                style.series.insert("desktop".into(),desktop); style
            }).spoke_tooltip(move |spoke| {
                xgpui_luma::controls::tooltip::Tooltip::new(spoke.category.clone())
                    .shortcut(spoke.points.iter().map(|point| format!("{}: {}",point.label,point.sample.y)).collect::<Vec<_>>().join(" · "))
                    .theme(std::sync::Arc::new(tooltip_look.clone()))
                    .delay(Duration::from_millis(250))
            }).tooltip_motion(Duration::from_millis(250)).on_spoke_event(move|event,cx|{
                if let xgpui_luma_plot::SelectionEvent::Select{target}=event {
                    let text=format!("Selected {}",target.category);let _=host.update(cx,|this,cx|{this.selected=text;cx.notify();});
                }
            });
            if index == 2 || index == 3 || index == 10 || index == 11 { chart = chart.series("mobile"); }
            if index == 3 { chart = chart.category_labels(|i,name| format!("{} · {}/{}",name,DESKTOP[i],MOBILE[i]).into()); }
            theme::panel_frame(&self.look,title).h(px(440.0)).render(cx).child(DockPanel::new()
                .top(vstack! { gap=4;
                    div().typography_style(typography.text.label).child(title),
                    div().typography_style(typography.text.caption).text_color(chrome.muted_text).child("Visitors by month · shared radial scale"),
                }.h(px(80.0)).px_4().justify_center().flex_shrink_0())
                .bottom(div().h(px(48.0)).flex().items_center().justify_center().typography_style(typography.text.caption).text_color(chrome.muted_text).child(self.selected.clone()))
                .fill(div().size_full().p_3().child(chart.build()))).min_w_0()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;
    #[gpui::test]
    fn gallery_draws_variants_and_restarts_motion_on_entry(cx: &mut TestAppContext) {
        let (host, cx) = cx.add_window_view(|_, _| RadarGallery {
            selected: "Click a month".into(),
            data: Dataset::new()
                .with_series(Series::new(
                    "desktop",
                    "Desktop",
                    DESKTOP.into_iter().enumerate().map(|(i, y)| Sample::new(i as f64, y)).collect::<Vec<_>>(),
                ))
                .unwrap(),
            look: AppTheme::new(),
            motion: VisualTransition::with_transition(
                1.0,
                Transition::timed(Duration::from_millis(900), Easing::OutCubic),
            ),
        });
        host.update(cx, |host, cx| {
            host.enter(cx);
            assert_eq!(host.motion.progress(), 0.0);
            assert!(host.motion.is_animating());
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        host.update(cx, |host, cx| {
            host.motion.snap_to(1.0);
            host.enter(cx);
            assert_eq!(host.motion.progress(), 0.0);
            assert!(host.motion.is_animating());
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
    }
}
