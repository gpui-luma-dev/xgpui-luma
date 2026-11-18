use gpui::{Context, FocusOutEvent, IntoElement, Render, Window, div, prelude::*};

use super::super::model::ResizablePanelsRenderModel;
use super::{ResizablePanels, ResizablePanelsEvent};
use crate::theme::InteractionState;

impl ResizablePanels {
    pub(super) fn render_model(&self) -> ResizablePanelsRenderModel<'_> {
        ResizablePanelsRenderModel {
            id: &self.model.id,
            orientation: self.model.orientation,
            frame_width: self.model.frame_width,
            frame_height: self.model.frame_height,
            show_border: self.model.show_border,
            enabled: self.model.enabled,
            handle_visibility: self.model.handle_visibility,
            double_click_collapse: self.model.double_click_collapse,
            hovered_handle: self.hovered_handle,
            dragging_handle: self.dragging_handle,
            handle_metrics: self.model.theme.resize_handle_metrics(),
            handle_grip: self.model.handle_grip,
            panel_sizes_px: &self.panel_sizes_px,
            panel_hidden: self.panel_hide_restore.iter().map(Option::is_some).collect(),
            panels: &self.model.panels,
            measured_size: self.measured_size,
        }
    }

    pub(super) fn sync_handle_focus_subscriptions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.handle_focus_subscriptions.len() == self.handle_focuses.len() * 2 {
            return;
        }

        self.handle_focus_subscriptions.clear();
        for (index, focus_handle) in self.handle_focuses.clone().into_iter().enumerate() {
            self.handle_focus_subscriptions.push(cx.on_focus(&focus_handle, window, move |this, window, cx| {
                this.handle_handle_focus_in(index, window, cx);
            }));
            self.handle_focus_subscriptions.push(cx.on_focus_out(
                &focus_handle,
                window,
                move |this, event: FocusOutEvent, window, cx| {
                    this.handle_handle_focus_out(index, event, window, cx);
                },
            ));
        }
    }
}

impl Render for ResizablePanels {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut was_animating = false;
        let mut now_animating = false;

        for transition in &mut self.transitions {
            if transition.is_animating() {
                was_animating = true;
            }
            if transition.sync() {
                now_animating = true;
            }
            transition.schedule_frame(window, cx);
        }

        self.sync_handle_focus_subscriptions(window, cx);
        self.refresh_panel_sizes_px();

        // Never emit from render — subscribers (e.g. studio layout refresh) must not re-enter
        // layout while the element tree is being built. Publish each animation frame
        // in the effect cycle so dependent chrome can follow the visible pane sizes.
        if was_animating {
            let sizes_px = self.panel_sizes_px.clone();
            let entity = cx.entity();
            cx.defer(move |cx| {
                entity.update(cx, |_, cx| {
                    cx.emit(ResizablePanelsEvent::SizesChanged { sizes_px: sizes_px.clone() });
                    if !now_animating {
                        cx.emit(ResizablePanelsEvent::ResizeEnd { sizes_px });
                    }
                });
            });
        }

        let look = self.model.theme.resolve(InteractionState { disabled: !self.model.enabled, ..Default::default() });
        let model = self.render_model();
        let template = self.model.template.clone();

        div().size_full().child(template.render(&model, &look, &self.handle_focuses, window, cx))
    }
}
