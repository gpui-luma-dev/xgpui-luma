use xgpui_luma::motion::{Transition, Easing};
use std::time::Duration;
use gpui::{App, Context, Entity, Render, Window, IntoElement, div, px, prelude::*};
use xgpui_luma::{DockPanel, vstack, motion::VisualTransition};
use crate::theme::AppTheme;
use xgpui_luma_plot::{Dataset, DatasetError, Series, Sample, RadialChartBuilder, RadialLayout};
use crate::theme::{self, TypographyExt};
pub(crate) struct RadialGallery {
    selected: String,
    data: Dataset,
    text: Dataset,
    shape: Dataset,
    stacked: Dataset,
    look: AppTheme,
    motion: VisualTransition,
}
impl RadialGallery {
    pub(crate) fn spawn(look: &AppTheme, cx: &mut App) -> Result<Entity<Self>, DatasetError> {
        let data = Dataset::new().with_series(Series::new(
            "browsers",
            "Visitors",
            [275.0, 200.0, 187.0, 173.0, 90.0]
                .into_iter()
                .enumerate()
                .map(|(i, y)| Sample::new(i as f64, y))
                .collect::<Vec<_>>(),
        ))?;
        let text = Dataset::new().with_series(Series::new("visits", "Visitors", [Sample::new(0.0, 200.0)]))?;
        let shape = Dataset::new().with_series(Series::new("visits", "Visitors", [Sample::new(0.0, 1260.0)]))?;
        let stacked = Dataset::new().with_series(Series::new(
            "visits",
            "Visitors",
            [Sample::new(0.0, 1200.0), Sample::new(1.0, 630.0)],
        ))?;
        let look = look.clone();
        Ok(cx.new(|_| Self {
            selected: "Click an arc to select its observation".into(),
            data,
            text,
            shape,
            stacked,
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
impl Render for RadialGallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.motion.sync();
        self.motion.schedule_frame(window, cx);
        let progress = self.motion.progress();
        let typography = theme::typography();
        let chrome = theme::chrome(&self.look);
        div().w_full().max_w(px(1200.0)).grid().grid_cols(2).gap(px(20.0)).children(["Radial Chart","Radial Chart – Label","Radial Chart – Grid","Radial Chart – Text","Radial Chart – Shape","Radial Chart – Stacked"].into_iter().enumerate().map(|(index,title)| {
            let (data,series)=match index {3=>(self.text.clone(),"visits"),4=>(self.shape.clone(),"visits"),5=>(self.stacked.clone(),"visits"),_=>(self.data.clone(),"browsers")};
            let look=self.look.clone();let tooltip_look=self.look.clone();
            let host=cx.weak_entity();
            let mut chart=RadialChartBuilder::new(data,series).id(("radial-example",index)).draw_progress(progress).style(move|| {
                let mut style=theme::radial::style(&look);
                if index==1 {style.label_text.color=crate::theme::palette::LABEL_WHITE;style.label_text.font_size=px(11.0);style.label_text.line_height=px(12.0);}
                if index==2 {style.track=None;style.grid=Some(xgpui_luma_plot::Stroke{color:theme::chrome(&look).border,width:px(1.0)});}
                if index==3 {style.inner_radius=0.88;style.rounded=true;style.palette=vec![crate::theme::palette::BLUE_BRIGHT.into()];}
                if index==4 {style.inner_radius=0.72;style.palette=vec![crate::theme::palette::BLUE_BRIGHT.into()];}
                if index==5 {style.inner_radius=0.77;style.rounded=true;}
                style
            }).bar_tooltip(move|bar|xgpui_luma::controls::tooltip::Tooltip::new(bar.label.clone()).shortcut(format!("{} visitors",bar.sample.y)).theme(std::sync::Arc::new(tooltip_look.clone())).delay(Duration::from_millis(250))).tooltip_motion(Duration::from_millis(250)).on_bar_event(move|event,cx|{
                if let xgpui_luma_plot::SelectionEvent::Select{target}=event {
                    let text=format!("Selected {}",target.label);let _=host.update(cx,|this,cx|{this.selected=text;cx.notify();});
                }
            });
            if index<3 {chart=chart.bar_names(["Chrome","Safari","Firefox","Edge","Other"]);}
            if index==1 {chart=chart.angles(90.0,-360.0).bar_labels(|bar|bar.label.clone());}
            if index==3 {chart=chart.bar_names(["Visitors"]).maximum(300.0).center_text("200","Visitors");}
            if index==4 {chart=chart.bar_names(["Visitors"]).maximum(4000.0).center_text("1,260","Visitors");}
            if index==5 {chart=chart.bar_names(["Desktop","Mobile"]).angles(180.0,180.0).layout(RadialLayout::Stacked).center_text("1,830","Visitors");}
            theme::panel_frame(&self.look,title).h(px(440.0)).render(cx).child(DockPanel::new()
                .top(vstack!{gap=4;div().typography_style(typography.text.label).child(title),div().typography_style(typography.text.caption).text_color(chrome.muted_text).child("January – June 2024"),}.h(px(80.0)).px_4().justify_center().flex_shrink_0())
                .bottom(div().h(px(48.0)).flex().items_center().justify_center().typography_style(typography.text.caption).text_color(chrome.muted_text).child(self.selected.clone()))
                .fill(div().size_full().p_3().child(chart.build()))).min_w_0()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;
    struct Harness {
        gallery: Entity<RadialGallery>,
    }
    impl Render for Harness {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            self.gallery.clone()
        }
    }
    #[gpui::test]
    fn radial_gallery_draws_all_cards_and_restarts_on_entry(cx: &mut TestAppContext) {
        let (host, cx) =
            cx.add_window_view(|_, cx| Harness { gallery: RadialGallery::spawn(&AppTheme::new(), cx).unwrap() });
        host.update(cx, |host, cx| {
            host.gallery.update(cx, |gallery, cx| {
                gallery.enter(cx);
                assert_eq!(gallery.motion.progress(), 0.0);
                assert!(gallery.motion.is_animating());
                gallery.motion.snap_to(1.0);
            })
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        host.update(cx, |host, cx| {
            host.gallery.update(cx, |gallery, cx| {
                gallery.enter(cx);
                assert_eq!(gallery.motion.progress(), 0.0);
                assert!(gallery.motion.is_animating());
            })
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
    }
}
