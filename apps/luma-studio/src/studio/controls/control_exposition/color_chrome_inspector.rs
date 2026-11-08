use std::sync::Arc;

use gpui::{Context, Entity, FontWeight, IntoElement, Render, SharedString, Window, div, px};
use gpui::prelude::*;
use xgpui_luma::controls::accordion::{
    AccordionContent, AccordionControl, AccordionItem, AccordionSelectionMode, AccordionTrigger,
};
use xgpui_luma_look_shadcn::ShadcnLook;
use xgpui_luma_look_shadcn as shadcn;
use lucide_svg_static::Icon as LucideIcon;

use super::inspector::color_chrome::{ColorChromeProfile, ColorChromeSection, resolve_color_chrome_sections};
use super::inspector::motion::{SharedMotionReport, render_motion_examples};
use super::inspector::render::{layout, render_color_category, render_property_rows_panel};

pub struct ColorChromeInspector {
    look: Arc<ShadcnLook>,
    id_prefix: SharedString,
    profiles: &'static [ColorChromeProfile],
    motion_report: Option<SharedMotionReport>,
    motion_section: Option<Entity<AccordionControl>>,
}

impl ColorChromeInspector {
    pub fn new(
        look: Arc<ShadcnLook>,
        id_prefix: impl Into<SharedString>,
        profiles: &'static [ColorChromeProfile],
        cx: &mut Context<Self>,
    ) -> Self {
        let _ = cx;
        Self { look, id_prefix: id_prefix.into(), profiles, motion_report: None, motion_section: None }
    }

    pub fn set_motion_report(&mut self, report: SharedMotionReport, cx: &mut Context<Self>) {
        self.motion_report = Some(report);
        let expanded = self.motion_expanded(cx);
        self.rebuild_motion(expanded, cx);
        cx.notify();
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        let expanded = self.motion_expanded(cx);
        self.look = look;
        self.rebuild_motion(expanded, cx);
        cx.notify();
    }

    fn motion_expanded(&self, cx: &gpui::App) -> Option<bool> {
        self.motion_section
            .as_ref()
            .map(|section| section.read(cx).is_expanded(&SharedString::from("motion")))
    }

    fn rebuild_motion(&mut self, expanded: Option<bool>, cx: &mut Context<Self>) {
        let Some(report) = self.motion_report.clone() else {
            self.motion_section = None;
            return;
        };
        let examples = report.examples(&self.look, cx);
        if examples.is_empty() || examples.iter().all(|example| example.rows.is_empty()) {
            self.motion_section = None;
            return;
        }
        let look = self.look.clone();
        let open = expanded.unwrap_or(true);
        self.motion_section = Some(
            shadcn::Accordion::new(format!("{}-motion", self.id_prefix))
                .look(look.as_ref())
                .mode(AccordionSelectionMode::Multiple)
                .item_dividers(false)
                .trigger_min_height(30.0)
                .trigger_padding_y(3.0)
                .content_padding_top(2.0)
                .content_padding_bottom(6.0)
                .items([AccordionItem::new(
                    "motion",
                    AccordionTrigger::new("Motion").icon(LucideIcon::CirclePlay),
                    AccordionContent::custom(move |_, cx| render_motion_examples(&look, &report.examples(&look, cx))),
                )
                .expanded(open)])
                .spawn(cx),
        );
    }
}

impl Render for ColorChromeInspector {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let look = &self.look;
        let chrome = look.chrome();
        let body = &look.mode_tokens().typography.text.body;
        let sections = resolve_color_chrome_sections(look.as_ref(), self.profiles);

        div()
            .id(self.id_prefix.clone())
            .size_full()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(chrome.panel_background)
            .child(
                div()
                    .flex_none()
                    .flex()
                    .flex_col()
                    .gap(px(10.0))
                    .border_b_1()
                    .border_color(chrome.border)
                    .p(px(layout::PANEL_PADDING))
                    .child(
                        div()
                            .text_size(px(body.size + 3.0))
                            .line_height(px(body.line_height + 3.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(chrome.title_text)
                            .child("Color Chrome"),
                    )
                    .child(domain_callout(look)),
            )
            .child({
                let mut scroll = div()
                    .id(SharedString::from(format!("{}-scroll", self.id_prefix)))
                    .flex_1()
                    .min_h(px(0.0))
                    .min_w(px(0.0))
                    .overflow_y_scroll()
                    .p(px(layout::PANEL_PADDING))
                    .flex()
                    .flex_col()
                    .gap(px(12.0))
                    .children(sections.into_iter().map(|section| render_section(look, section)));
                if let Some(section) = &self.motion_section {
                    scroll = scroll.child(div().pt(px(12.0)).child(section.clone()));
                }
                scroll
            })
    }
}

fn domain_callout(look: &ShadcnLook) -> impl IntoElement {
    let chrome = look.chrome();
    let caption = &look.mode_tokens().typography.text.caption;

    div()
        .w_full()
        .rounded(px(6.0))
        .border_1()
        .border_color(chrome.border)
        .bg(chrome.content_background)
        .p(px(10.0))
        .child(
            div()
                .text_size(px(caption.size))
                .line_height(px(caption.line_height))
                .text_color(chrome.muted_text)
                .child(
                    "Inspects theme-derived chrome only. Hue, saturation, value, and alpha gradients are control \
                     domain rendering tied to the selected color — not Shadcn theme tokens.",
                ),
        )
}

fn render_section(look: &Arc<ShadcnLook>, section: ColorChromeSection) -> impl IntoElement {
    let chrome = look.chrome();
    let body = &look.mode_tokens().typography.text.body;
    let caption = &look.mode_tokens().typography.text.caption;

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(layout::DETAIL_GAP))
        .child(
            div()
                .text_size(px(body.size))
                .line_height(px(body.line_height))
                .font_weight(FontWeight::MEDIUM)
                .text_color(chrome.title_text)
                .child(section.title),
        )
        .when_some(section.note, |stack, note| {
            stack.child(
                div()
                    .text_size(px(caption.size))
                    .line_height(px(caption.line_height))
                    .text_color(chrome.muted_text)
                    .child(note),
            )
        })
        .when(!section.color_rows.is_empty(), |stack| stack.child(render_color_category(look, section.color_rows)))
        .when(!section.property_rows.is_empty(), |stack| {
            stack.child(render_property_rows_panel(look, section.property_rows))
        })
}
