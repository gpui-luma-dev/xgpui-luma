use std::sync::Arc;

use gpui::{AnyElement, App, Context, Pixels, ScrollHandle, Size, Window, div, point, prelude::*, px};
use xgpui_luma::infra::ElementExt;
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::components::catalog::first_controls_exposition_id;

use super::control_exposition::ControlExposition;

pub struct ControlsPanel {
    look: Arc<ShadcnLook>,
    expositions: Vec<ControlExposition>,
    scroll_handle: ScrollHandle,
    selected_entry_id: &'static str,
    selected_viewport_size: Option<(&'static str, Size<Pixels>)>,
    /// Workbench content-panel width applied until `on_prepaint` reports the same size.
    /// Prevents one-frame stale bounds (sidebar open/close) from re-widening the inspector split.
    host_content_width: Option<Pixels>,
}

impl ControlsPanel {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let expositions = ControlExposition::spawn_all(look.clone(), cx);

        Self {
            look: look.clone(),
            expositions,
            scroll_handle: ScrollHandle::new(),
            selected_entry_id: first_controls_exposition_id().unwrap_or("button"),
            selected_viewport_size: None,
            host_content_width: None,
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
            self.selected_viewport_size = None;
            self.scroll_handle.set_offset(point(px(0.0), px(0.0)));
        }
        self.request_layout_refresh(cx);
        cx.notify();
    }

    pub fn request_layout_refresh(&mut self, cx: &mut Context<Self>) {
        if let Some(exposition) = ControlExposition::find(&self.expositions, self.selected_entry_id, cx) {
            exposition.request_layout_refresh(cx);
            if let Some((entry_id, size)) = self.selected_viewport_size
                && entry_id == self.selected_entry_id
            {
                exposition.set_viewport_size(size, cx);
            }
        }
    }

    /// Pin inspector-split width to the workbench content panel (source of truth on sidebar toggle).
    pub fn set_host_content_width(&mut self, width: Pixels, cx: &mut Context<Self>) {
        if width.as_f32() < 64.0 {
            return;
        }
        self.host_content_width = Some(width);

        let height = self
            .selected_viewport_size
            .filter(|(entry_id, _)| *entry_id == self.selected_entry_id)
            .map(|(_, size)| size.height);
        let Some(height) = height.filter(|height| height.as_f32() >= 64.0) else {
            return;
        };

        self.apply_viewport_size(Size { width, height }, cx);
    }

    fn update_selected_viewport_size(&mut self, size: Size<Pixels>, _window: &mut Window, cx: &mut Context<Self>) {
        if size.height.as_f32() < 64.0 {
            return;
        }

        let width = if let Some(host_width) = self.host_content_width {
            if (size.width.as_f32() - host_width.as_f32()).abs() < 1.0 {
                self.host_content_width = None;
                size.width
            } else if size.width.as_f32() < 64.0 {
                return;
            } else {
                // Ancestor bounds can lag one frame behind workbench panel sizes after sidebar
                // show/hide; keep the host width until prepaint catches up.
                host_width
            }
        } else if size.width.as_f32() < 64.0 {
            return;
        } else {
            size.width
        };

        self.apply_viewport_size(Size { width, height: size.height }, cx);
    }

    fn apply_viewport_size(&mut self, size: Size<Pixels>, cx: &mut Context<Self>) {
        let entry_id = self.selected_entry_id;
        if self.selected_viewport_size == Some((entry_id, size)) {
            return;
        }

        self.selected_viewport_size = Some((entry_id, size));
        if let Some(exposition) = ControlExposition::find(&self.expositions, entry_id, cx) {
            exposition.set_viewport_size(size, cx);
        }
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
        let entity = cx.entity().clone();
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
                div()
                    .id("controls-content")
                    .size_full()
                    .min_h(px(0.0))
                    .on_prepaint(move |bounds, window, cx| {
                        entity.update(cx, |panel, cx| {
                            panel.update_selected_viewport_size(bounds.size, window, cx);
                        });
                    })
                    .child(page)
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
