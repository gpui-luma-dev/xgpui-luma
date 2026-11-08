use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::button::Button;
use xgpui_luma::controls::overlay_window::{OverlayWindow, OverlayWindowMode};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::overlay_demo::render_overlay_dialog_panel_with_footer;
use super::inspector::{OverlayWindowInspectorAdapter, OVERLAY_WINDOW_INSPECTOR_SPEC};
use super::shell_theme_inspectors::OverlayWindowThemeInspector;
use super::template::render_control_exposition_card;

pub struct ModalOverlayControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<ModalOverlayExpositionLeftPane>,
    theme_inspector: Entity<OverlayWindowThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

struct ModalOverlayExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    trigger: Entity<Button>,
    cancel: Entity<Button>,
    confirm: Entity<Button>,
    overlay: OverlayWindow,
}

impl ModalOverlayExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        for button in [&self.trigger, &self.cancel, &self.confirm] {
            button.update(cx, |_, cx| cx.notify());
        }
        self.overlay.update(cx, |_, cx| cx.notify());
        cx.notify();
    }
}

impl Render for ModalOverlayExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(12.0))
                .child(self.trigger.clone())
                .child(self.overlay.clone())
                .into_any_element();

            div()
                .id("controls-doc-modal-overlay-left-pane")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .overflow_y_scroll()
                .child(render_control_exposition_card(
                    &self.look,
                    self.entry,
                    preview,
                    None,
                    ControlExpositionLayout::BORDERLESS,
                ))
        })
    }
}

impl ModalOverlayControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("modal-overlay").expect("modal-overlay catalog entry");
        let cancel = shadcn::Button::new("controls-doc-dialog-cancel")
            .look(look.as_ref())
            .outline()
            .label("Cancel")
            .spawn(cx);
        let confirm = shadcn::Button::new("controls-doc-dialog-confirm")
            .look(look.as_ref())
            .primary()
            .label("Confirm")
            .spawn(cx);
        let overlay = look
            .overlay_window("controls-doc-dialog-overlay")
            .mode(OverlayWindowMode::Modal)
            .content({
                let cancel = cancel.clone();
                let confirm = confirm.clone();
                let look = look.clone();
                move |_, _, _| {
                    render_overlay_dialog_panel_with_footer(
                        &look,
                        "Archive this project?",
                        None::<Entity<Button>>,
                        "This action cannot be undone.",
                        div().w_full().flex().justify_end().gap(px(8.0)).child(cancel.clone()).child(confirm.clone()),
                    )
                }
            })
            .theme_children([cancel.clone(), confirm.clone()])
            .spawn(cx);
        let trigger = shadcn::Button::new("controls-doc-dialog-trigger")
            .look(look.as_ref())
            .primary()
            .label("Open modal")
            .spawn(cx);

        let left_pane = cx.new(|_| ModalOverlayExpositionLeftPane {
            look: look.clone(),
            entry,
            trigger: trigger.clone(),
            cancel: cancel.clone(),
            confirm: confirm.clone(),
            overlay: overlay.clone(),
        });

        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-modal-overlay-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &OVERLAY_WINDOW_INSPECTOR_SPEC,
            OverlayWindowInspectorAdapter::shared(),
        );

        trigger.clone().on_click(cx, {
            let left_pane = left_pane.downgrade();
            move |_, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };

                left_pane.update(cx, |pane, cx| {
                    pane.overlay.update(cx, |overlay, cx| overlay.open(cx));
                });
            }
        });
        cancel.clone().on_click(cx, {
            let left_pane = left_pane.downgrade();
            move |_, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };

                left_pane.update(cx, |pane, cx| {
                    pane.overlay.update(cx, |overlay, cx| overlay.dismiss(cx));
                });
            }
        });
        confirm.clone().on_click(cx, {
            let left_pane = left_pane.downgrade();
            move |_, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };

                left_pane.update(cx, |pane, cx| {
                    pane.overlay.update(cx, |overlay, cx| overlay.dismiss(cx));
                });
            }
        });

        Self { look, entry, left_pane, theme_inspector, inspector_split }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn fills_viewport(&self) -> bool {
        true
    }

    pub fn request_layout_refresh(&mut self, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.request_layout_refresh(cx));
    }

    pub fn set_viewport_size(&mut self, size: gpui::Size<gpui::Pixels>, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.set_viewport_size(size, cx));
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        sync_viewport_inspector(look, &self.theme_inspector, &self.inspector_split, cx);
        cx.notify();
    }
}

impl Render for ModalOverlayControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-modal-overlay-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}
