use std::sync::Arc;

use gpui::{AnyElement, App, Context, ScrollHandle, Window, div, point, prelude::*, px};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::components::catalog::first_controls_exposition_id;

use super::control_exposition::ControlExposition;

pub struct ControlsPanel {
    look: Arc<ShadcnLook>,
    expositions: Vec<ControlExposition>,
    scroll_handle: ScrollHandle,
    selected_entry_id: &'static str,
}

impl ControlsPanel {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let expositions = ControlExposition::spawn_all(look.clone(), cx);

        Self {
            look: look.clone(),
            expositions,
            scroll_handle: ScrollHandle::new(),
            selected_entry_id: first_controls_exposition_id().unwrap_or("button"),
        }
    }

    pub fn sync_snapshot(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for exposition in &self.expositions {
            exposition.sync_look(look.clone(), cx);
        }
        cx.notify();
    }

    pub fn dismiss_overlays(&self, cx: &mut Context<Self>) {
        for exposition in &self.expositions {
            exposition.dismiss_overlays(cx);
        }
    }

    pub(crate) fn selected_entry_id(&self) -> &'static str {
        self.selected_entry_id
    }

    pub(crate) fn select_entry(&mut self, entry_id: &'static str, cx: &mut Context<Self>) {
        if self.selected_entry_id != entry_id {
            self.selected_entry_id = entry_id;
            self.scroll_handle.set_offset(point(px(0.0), px(0.0)));
        }
        cx.notify();
    }

    fn render_selected_page(&self, cx: &App) -> (AnyElement, bool) {
        let Some(exposition) = ControlExposition::find(&self.expositions, self.selected_entry_id, cx) else {
            return (div().child("Control documentation not found.").into_any_element(), false);
        };
        (exposition.render(cx), exposition.fills_viewport(cx))
    }
}

impl gpui::Render for ControlsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let (page, fills_viewport) = self.render_selected_page(cx);

            let mut shell = div()
                .id("luma-studio-controls")
                .size_full()
                .min_h_0()
                .relative()
                .flex()
                .flex_col()
                .overflow_hidden()
                .bg(chrome.content_background);

            if !fills_viewport {
                shell = shell.px(px(28.0)).pt(px(12.0)).pb(px(28.0));
            }

            shell.child(div().relative().flex_1().min_h(px(0.0)).child(if fills_viewport {
                div().id("controls-content").size_full().min_h(px(0.0)).child(page)
            } else {
                div()
                    .id("controls-content")
                    .size_full()
                    .overflow_y_scroll()
                    .scrollbar_width(px(0.0))
                    .track_scroll(&self.scroll_handle)
                    .child(div().w_full().flex().justify_start().pb(px(12.0)).child(div().w_full().child(page)))
            }))
        })
    }
}
