use xgpui_luma::controls::popup_menu::PopupMenuEventExt;
use xgpui_luma::infra::attachments::TooltipEntityExt;
use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::infra::menu_item::MenuItem;
use xgpui_luma::controls::popup_menu::{PopupMenu, PopupMenuEvent, PopupMenuPlacement};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::inspector::{PopupMenuInspectorAdapter, POPUP_MENU_INSPECTOR_SPEC, popup_menu_motion_report};
use super::standalone_theme_inspectors::PopupMenuThemeInspector;
use super::template::render_control_exposition_card;

pub struct PopupMenuControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<PopupMenuExpositionLeftPane>,
    theme_inspector: Entity<PopupMenuThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

struct PopupMenuExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    preview_smart: Entity<PopupMenu>,
    preview_below: Entity<PopupMenu>,
    event_stream: Entity<ControlEventStream>,
}

impl PopupMenuExpositionLeftPane {
    fn dismiss_overlays(&mut self, cx: &mut Context<Self>) {
        self.preview_smart.update(cx, |preview, cx| preview.dismiss(cx));
        self.preview_below.update(cx, |preview, cx| preview.dismiss(cx));
    }

    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.preview_smart.update(cx, |_, cx| cx.notify());
        self.preview_below.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for PopupMenuExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(16.0))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .justify_center()
                        .gap(px(12.0))
                        .child(self.preview_below.clone())
                        .child(self.preview_smart.clone()),
                )
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-popup-menu-left-pane")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .overflow_y_scroll()
                .child(render_control_exposition_card(
                    &self.look,
                    self.entry,
                    preview.into_any_element(),
                    None,
                    ControlExpositionLayout::BORDERLESS,
                ))
        })
    }
}

impl PopupMenuControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("popup-menu").expect("popup-menu catalog entry");
        let preview_smart = shadcn::PopupMenu::new("controls-doc-popup-menu-smart")
            .look(look.as_ref())
            .label("Smart popup")
            .items(popup_menu_items())
            .placement(PopupMenuPlacement::Smart)
            .spawn(cx)
            .help("Open the action menu", cx);
        let preview_below = shadcn::PopupMenu::new("controls-doc-popup-menu-below")
            .look(look.as_ref())
            .label("Below popup")
            .items(popup_menu_items())
            .placement(PopupMenuPlacement::BelowStart)
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-popup-menu-event-log",
                "Open either popup and choose items; PopupMenuEvent variants appear below.",
            )
        });
        let left_pane = cx.new(|_| PopupMenuExpositionLeftPane {
            look: look.clone(),
            entry,
            preview_smart: preview_smart.clone(),
            preview_below: preview_below.clone(),
            event_stream: event_stream.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-popup-menu-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &POPUP_MENU_INSPECTOR_SPEC,
            PopupMenuInspectorAdapter::shared(),
        );
        theme_inspector.update(cx, |inspector, cx| {
            inspector.set_motion_report(
                popup_menu_motion_report([
                    ("Smart popup", preview_smart.clone()),
                    ("Below popup", preview_below.clone()),
                ]),
                cx,
            );
        });

        for preview in [&preview_smart, &preview_below] {
            let event_stream = event_stream.clone();
            preview.clone().on_event(cx, move |_, event: &PopupMenuEvent, cx| {
                let line = format_popup_menu_event(event);
                event_stream.update(cx, |stream, cx| {
                    stream.append_line(&line, cx);
                    cx.notify();
                });
            });
        }

        Self { look, entry, left_pane, theme_inspector, inspector_split }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn fills_viewport(&self) -> bool {
        true
    }

    pub fn dismiss_overlays(&mut self, cx: &mut Context<Self>) {
        self.left_pane.update(cx, |pane, cx| pane.dismiss_overlays(cx));
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

impl Render for PopupMenuControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-popup-menu-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn popup_menu_items() -> [MenuItem; 6] {
    [
        MenuItem::new("new").label("New file").icon(LucideIcon::FilePlus),
        MenuItem::new("rename").label("Rename").icon(LucideIcon::Pencil),
        MenuItem::separator("actions-separator"),
        MenuItem::new("archive").label("Archive"),
        MenuItem::new("share").label("Share").icon(LucideIcon::Share2).submenu([
            MenuItem::new("copy-link").label("Copy link").icon(LucideIcon::Link),
            MenuItem::separator("submenu-separator"),
            MenuItem::new("email").label("Email").icon(LucideIcon::Mail),
        ]),
        MenuItem::new("disabled").label("Unavailable").icon(LucideIcon::ArchiveX).enabled(false),
    ]
}

fn format_popup_menu_event(event: &PopupMenuEvent) -> String {
    match event {
        PopupMenuEvent::Select { item_id, label } => {
            format!("PopupMenuEvent::Select {{ item_id: \"{item_id}\", label: \"{label}\" }}")
        }
        PopupMenuEvent::OpenChanged { open } => format!("PopupMenuEvent::OpenChanged {{ open: {open} }}"),
        PopupMenuEvent::Dismiss => "PopupMenuEvent::Dismiss".to_string(),
        PopupMenuEvent::FocusChanged { focused } => {
            format!("PopupMenuEvent::FocusChanged {{ focused: {focused} }}")
        }
        PopupMenuEvent::HoverChanged { hovered } => {
            format!("PopupMenuEvent::HoverChanged {{ hovered: {hovered} }}")
        }
        PopupMenuEvent::EnabledChanged { enabled } => {
            format!("PopupMenuEvent::EnabledChanged {{ enabled: {enabled} }}")
        }
        _ => "PopupMenuEvent::(unknown)".to_string(),
    }
}
