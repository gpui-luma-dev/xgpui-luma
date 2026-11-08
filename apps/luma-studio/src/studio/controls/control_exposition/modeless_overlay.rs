use xgpui_luma::controls::overlay_window::OverlayWindowEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, Focusable, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::button::{Button, HasPresenter};
use xgpui_luma::controls::overlay_window::{OverlayWindow, OverlayWindowEvent, OverlayWindowMode, OverlayWindowPosition};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::ControlExpositionLayout;
use super::overlay_demo::{overlay_status_for, render_overlay_dialog_panel, render_overlay_status};
use super::inspector::{OverlayWindowInspectorAdapter, OVERLAY_WINDOW_INSPECTOR_SPEC};
use super::shell_theme_inspectors::OverlayWindowThemeInspector;
use super::template::render_control_exposition_card;

pub struct ModelessOverlayControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<ModelessOverlayExpositionLeftPane>,
    theme_inspector: Entity<OverlayWindowThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

struct ModelessOverlayExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    trigger: Entity<Button>,
    background_counter: Entity<Button>,
    close: Entity<Button>,
    overlay: OverlayWindow,
    background_clicks: usize,
    status: String,
}

impl ModelessOverlayExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        for button in [&self.trigger, &self.background_counter, &self.close] {
            button.update(cx, |_, cx| cx.notify());
        }
        self.overlay.update(cx, |_, cx| cx.notify());
        cx.notify();
    }
}

impl Render for ModelessOverlayExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(12.0))
                .child(self.trigger.clone())
                .child(self.background_counter.clone())
                .child(render_overlay_status("Status", &self.status, &self.look))
                .child(self.overlay.clone())
                .into_any_element();

            div()
                .id("controls-doc-modeless-overlay-left-pane")
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

impl ModelessOverlayControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("modeless-overlay").expect("modeless-overlay catalog entry");
        let close = shadcn::Button::icon_button("controls-doc-modeless-close", LucideIcon::X)
            .look(look.as_ref())
            .content_only()
            .spawn(cx);
        let overlay = look
            .overlay_window("controls-doc-modeless-overlay")
            .mode(OverlayWindowMode::Modeless)
            .position(OverlayWindowPosition::TopRight)
            .content({
                let close = close.clone();
                let look = look.clone();
                move |_, _, _| {
                    render_overlay_dialog_panel(
                        &look,
                        "Modeless notes",
                        Some(close.clone()),
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(8.0))
                            .child("The background stays interactive while this surface is open.")
                            .child("Use the counter below to verify clicks reach the workspace."),
                    )
                }
            })
            .theme_children([close.clone()])
            .spawn(cx);
        let trigger = shadcn::Button::new("controls-doc-modeless-trigger")
            .look(look.as_ref())
            .primary()
            .label("Open modeless")
            .spawn(cx);
        let background_counter = shadcn::Button::new("controls-doc-modeless-background")
            .look(look.as_ref())
            .outline()
            .label("Background 0")
            .spawn(cx);

        let left_pane = cx.new(|_| ModelessOverlayExpositionLeftPane {
            look: look.clone(),
            entry,
            trigger: trigger.clone(),
            background_counter: background_counter.clone(),
            close: close.clone(),
            overlay: overlay.clone(),
            background_clicks: 0,
            status: "Closed".into(),
        });

        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-modeless-overlay-pane",
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
                    let focus = pane.trigger.read(cx).focus_handle(cx);
                    pane.overlay.update(cx, |overlay, cx| overlay.open_from(Some(focus), cx));
                });
            }
        });
        background_counter.clone().on_click(cx, {
            let left_pane = left_pane.downgrade();
            move |_, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };

                left_pane.update(cx, |pane, cx| {
                    pane.background_clicks += 1;
                    pane.background_counter.update(cx, |button, cx| {
                        button.set_label(format!("Background {}", pane.background_clicks), cx);
                    });
                    pane.status = format!("Background still interactive: {} clicks.", pane.background_clicks);
                    cx.notify();
                });
            }
        });
        close.clone().on_click(cx, {
            let left_pane = left_pane.downgrade();
            move |_, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };

                left_pane.update(cx, |pane, cx| {
                    pane.status = "Dismissed modeless overlay.".into();
                    pane.overlay.update(cx, |overlay, cx| overlay.dismiss(cx));
                    cx.notify();
                });
            }
        });
        overlay.clone().on_event(cx, {
            let left_pane = left_pane.downgrade();
            move |_, event: &OverlayWindowEvent, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };
                left_pane.update(cx, |pane, cx| {
                    pane.status = overlay_status_for(event, "modeless overlay");
                    cx.notify();
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

impl Render for ModelessOverlayControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-modeless-overlay-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}
