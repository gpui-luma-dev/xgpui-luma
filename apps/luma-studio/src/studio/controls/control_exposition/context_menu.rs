use xgpui_luma::controls::context_menu::ContextMenuEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::context_menu::{ContextMenu, ContextMenuEvent};
use xgpui_luma::controls::textfield::TextField;
use xgpui_luma::infra::menu_item::MenuItem;
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::inspector::{ContextMenuInspectorAdapter, CONTEXT_MENU_INSPECTOR_SPEC, context_menu_motion_report};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::standalone_theme_inspectors::ContextMenuThemeInspector;
use super::template::render_control_exposition_card;

pub struct ContextMenuControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<ContextMenuExpositionLeftPane>,
    theme_inspector: Entity<ContextMenuThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct ContextMenuExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    preview: Entity<ContextMenu>,
    textfield: TextField,
    textfield_context_menu: Entity<ContextMenu>,
    event_stream: Entity<ControlEventStream>,
}

impl ContextMenuExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.preview.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for ContextMenuExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(16.0))
                .child(div().min_h(px(160.0)).flex().items_center().justify_center().child(self.preview.clone()))
                .child(div().w(px(200.0)).child(self.textfield_context_menu.clone()))
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-context-menu-left-pane")
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

impl ContextMenuControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("context-menu").expect("context-menu catalog entry");
        let preview = shadcn::ContextMenu::new("controls-doc-context-menu")
            .look(look.as_ref())
            .label("Right-click me")
            .items(context_menu_items())
            .spawn(cx);
        let textfield = shadcn::TextField::new("controls-doc-context-menu-textfield")
            .look(look.as_ref())
            .value("Sample text for context actions")
            .full_width(true)
            .spawn(cx);
        let textfield_target = textfield.clone();
        let textfield_context_menu = shadcn::ContextMenu::new("controls-doc-context-menu-textfield-menu")
            .look(look.as_ref())
            .target_content(move |_| div().w(px(200.0)).child(textfield_target.clone()))
            .items(textfield_context_menu_items())
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-context-menu-event-log",
                "Right-click the target and choose items; ContextMenuEvent variants appear below.",
            )
        });
        let left_pane = cx.new(|_| ContextMenuExpositionLeftPane {
            look: look.clone(),
            entry,
            preview: preview.clone(),
            textfield,
            textfield_context_menu: textfield_context_menu.clone(),
            event_stream: event_stream.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-context-menu-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &CONTEXT_MENU_INSPECTOR_SPEC,
            ContextMenuInspectorAdapter::shared(),
        );
        theme_inspector.update(cx, |inspector, cx| {
            inspector.set_motion_report(
                context_menu_motion_report([
                    ("Right-click target", preview.clone()),
                    ("Text field", textfield_context_menu.clone()),
                ]),
                cx,
            );
        });

        preview.clone().on_event(cx, {
            let event_stream = event_stream.downgrade();
            move |_, event: &ContextMenuEvent, cx| {
                let Some(event_stream) = event_stream.upgrade() else {
                    return;
                };
                let line = format_context_menu_event(event);
                event_stream.update(cx, |stream, cx| {
                    stream.append_line(&line, cx);
                });
            }
        });

        textfield_context_menu.clone().on_event(cx, {
            let event_stream = event_stream.downgrade();
            let textfield = left_pane.read(cx).textfield.clone();
            move |_, event: &ContextMenuEvent, cx| {
                let Some(event_stream) = event_stream.upgrade() else {
                    return;
                };
                let ContextMenuEvent::Select { item_id, .. } = event else {
                    return;
                };
                match item_id.as_ref() {
                    "select-all" => textfield.update(cx, |field, cx| field.select_all(cx)),
                    "copy" => textfield.update(cx, |field, cx| field.copy_selection(cx)),
                    "cut" => textfield.update(cx, |field, cx| field.cut_selection(cx)),
                    "paste" => textfield.update(cx, |field, cx| field.paste(cx)),
                    _ => {}
                }
                event_stream.update(cx, |stream, cx| {
                    stream.append_line(&format!("TextField context action: {item_id}"), cx);
                });
            }
        });

        Self { look, entry, left_pane, theme_inspector, inspector_split, _subscriptions: { Vec::new() } }
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

impl Render for ContextMenuControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-context-menu-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn context_menu_items() -> [MenuItem; 5] {
    [
        MenuItem::new("open").label("Open").icon(LucideIcon::FolderOpen),
        MenuItem::new("copy").label("Copy").icon(LucideIcon::Copy),
        MenuItem::separator("actions-separator"),
        MenuItem::new("inspect").label("Inspect"),
        MenuItem::new("more").label("More").icon(LucideIcon::Ellipsis).submenu([
            MenuItem::new("download").label("Download").icon(LucideIcon::Download),
            MenuItem::separator("submenu-separator"),
            MenuItem::new("external").label("Open externally").icon(LucideIcon::ExternalLink),
        ]),
    ]
}

fn textfield_context_menu_items() -> [MenuItem; 4] {
    [
        MenuItem::new("select-all").label("Select All").icon(LucideIcon::ListChecks),
        MenuItem::new("copy").label("Copy").icon(LucideIcon::Copy),
        MenuItem::new("cut").label("Cut").icon(LucideIcon::Scissors),
        MenuItem::new("paste").label("Paste").icon(LucideIcon::Clipboard),
    ]
}

fn format_context_menu_event(event: &ContextMenuEvent) -> String {
    match event {
        ContextMenuEvent::Select { item_id, label } => {
            format!("ContextMenuEvent::Select {{ item_id: \"{item_id}\", label: \"{label}\" }}")
        }
        ContextMenuEvent::OpenChanged { open } => format!("ContextMenuEvent::OpenChanged {{ open: {open} }}"),
        ContextMenuEvent::Dismiss => "ContextMenuEvent::Dismiss".to_string(),
        ContextMenuEvent::FocusChanged { focused } => {
            format!("ContextMenuEvent::FocusChanged {{ focused: {focused} }}")
        }
        ContextMenuEvent::HoverChanged { hovered } => {
            format!("ContextMenuEvent::HoverChanged {{ hovered: {hovered} }}")
        }
        ContextMenuEvent::EnabledChanged { enabled } => {
            format!("ContextMenuEvent::EnabledChanged {{ enabled: {enabled} }}")
        }
        _ => "ContextMenuEvent::(unknown)".to_string(),
    }
}
