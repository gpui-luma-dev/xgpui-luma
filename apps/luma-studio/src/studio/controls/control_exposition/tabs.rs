//! Tabs navigation control exposition — intrinsic and uniform width examples.

use xgpui_luma::controls::tabs::TabsEventExt;
use xgpui_luma::infra::presenter::HasPresenter;
use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::tabs::{Tabs, TabsEvent, TabsItem, TabsWidthMode, TabsContent};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::shell_theme_inspectors::TabsThemeInspector;
use super::inspector::{TabsInspectorAdapter, TabsMotionReport, TABS_INSPECTOR_SPEC};
use super::template::render_control_exposition_card;

pub struct TabsControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<TabsExpositionLeftPane>,
    theme_inspector: Entity<TabsThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

struct TabsExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    tabs: Entity<Tabs>,
    uniform_tabs: Entity<Tabs>,
    event_stream: Entity<ControlEventStream>,
    active_label: String,
    uniform_active_label: String,
}

impl TabsExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.tabs.update(cx, |_, cx| cx.notify());
        self.uniform_tabs.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for TabsExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let look = &self.look;

            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(12.0))
                .child(
                    div()
                        .text_sm()
                        .text_color(look.chrome().muted_text)
                        .child("Drag tabs to reorder. Hold near an edge to scroll; Escape cancels."),
                )
                .child(render_tabs_example(
                    look,
                    "Intrinsic width",
                    self.tabs.clone(),
                    &self.active_label,
                    "Increment a page’s counter, then reorder its tab; its state stays with the tab.",
                    360.0,
                ))
                .child(render_tabs_example(
                    look,
                    "Uniform width (match widest label)",
                    self.uniform_tabs.clone(),
                    &self.uniform_active_label,
                    "The widest label defines the slot width for every tab.",
                    420.0,
                ))
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-tabs-navigation-left-pane")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .overflow_y_scroll()
                .child(render_control_exposition_card(
                    look,
                    self.entry,
                    preview.into_any_element(),
                    None,
                    ControlExpositionLayout::BORDERLESS,
                ))
        })
    }
}

impl TabsControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("tabs-navigation").expect("tabs-navigation catalog entry");

        let mut tabs_builder =
            shadcn::Tabs::new("controls-doc-tabs").look(look.as_ref()).reordering(true).active("activity");
        for item in project_tabs() {
            let label = item.label_text().clone();
            let page = cx.new(|cx| RetainedTabPage::new(label, look.clone(), cx));
            tabs_builder = tabs_builder.tab_content(item, TabsContent::new(move |_, _| page.clone()));
        }
        let tabs = tabs_builder.spawn(cx);
        let uniform_tabs = shadcn::Tabs::new("controls-doc-tabs-uniform")
            .look(look.as_ref())
            .items(uniform_width_tabs())
            .reordering(true)
            .active("recent-activity")
            .width_mode(TabsWidthMode::Uniform)
            .spawn(cx);

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-tabs-navigation-event-log",
                "Interact with the tabs to inspect events.",
            )
        });

        let left_pane = cx.new(|_| TabsExpositionLeftPane {
            look: look.clone(),
            entry,
            tabs: tabs.clone(),
            uniform_tabs: uniform_tabs.clone(),
            event_stream: event_stream.clone(),
            active_label: "Activity".to_string(),
            uniform_active_label: "Recent Activity".to_string(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-tabs-navigation-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &TABS_INSPECTOR_SPEC,
            TabsInspectorAdapter::shared(),
        );
        theme_inspector.update(cx, |inspector, cx| {
            inspector.set_motion_report(
                Arc::new(TabsMotionReport::new([
                    ("Intrinsic width", tabs.clone()),
                    ("Uniform width", uniform_tabs.clone()),
                ])),
                cx,
            );
        });

        for (entity, label_field, left_pane) in
            [(tabs.clone(), "intrinsic", left_pane.clone()), (uniform_tabs.clone(), "uniform", left_pane.clone())]
        {
            let event_stream = event_stream.clone();
            entity.clone().on_event(cx, move |_, event: &TabsEvent, cx| {
                if let TabsEvent::Activate { label, .. } = event {
                    left_pane.update(cx, |pane, cx| {
                        match label_field {
                            "intrinsic" => pane.active_label = label.to_string(),
                            _ => pane.uniform_active_label = label.to_string(),
                        }
                        cx.notify();
                    });
                }
                if let Some(line) = format_tabs_event(event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
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

impl Render for TabsControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-tabs-navigation-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn format_tabs_event(event: &TabsEvent) -> Option<String> {
    match event {
        TabsEvent::DragDrop(event) => Some(format!("TabsEvent::DragDrop {event:?}")),
        TabsEvent::Activate { tab_id, label } => {
            Some(format!("TabsEvent::Activate {{ tab_id: \"{tab_id}\", label: \"{label}\" }}"))
        }
        TabsEvent::Change { tab_id, label } => {
            Some(format!("TabsEvent::Change {{ tab_id: \"{tab_id}\", label: \"{label}\" }}"))
        }
        TabsEvent::FocusChanged { focused } => Some(format!("TabsEvent::FocusChanged {{ focused: {focused} }}")),
        TabsEvent::ItemFocused { tab_id, label } => {
            Some(format!("TabsEvent::ItemFocused {{ tab_id: \"{tab_id}\", label: \"{label}\" }}"))
        }
        TabsEvent::Reactivate { tab_id, label } => {
            Some(format!("TabsEvent::Reactivate {{ tab_id: \"{tab_id}\", label: \"{label}\" }}"))
        }
        _ => None,
    }
}

fn render_tabs_example(
    look: &ShadcnLook,
    label: &'static str,
    tabs: Entity<Tabs>,
    active_label: &str,
    detail: &'static str,
    content_width: f32,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(4.0))
        .child(render_example_label(label, look))
        .w(px(content_width))
        .child(tabs)
        .child(render_tab_content(active_label, detail, content_width, look))
}

fn render_example_label(label: &'static str, look: &ShadcnLook) -> impl IntoElement {
    let chrome = look.chrome();
    div()
        .text_size(px(12.0))
        .line_height(px(16.0))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(chrome.muted_text)
        .child(label)
}

fn render_tab_content(active_label: &str, detail: &'static str, width: f32, look: &ShadcnLook) -> impl IntoElement {
    let chrome = look.chrome();

    div()
        .w(px(width))
        .min_h(px(112.0))
        .flex()
        .flex_col()
        .gap_2()
        .rounded(px(8.0))
        .border_1()
        .border_color(chrome.border)
        .bg(chrome.panel_background)
        .p(px(16.0))
        .text_color(chrome.body_text)
        .child(
            div()
                .text_size(px(14.0))
                .line_height(px(20.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .child(format!("{active_label} tab")),
        )
        .child(div().text_size(px(13.0)).line_height(px(18.0)).child(detail))
}

fn project_tabs() -> [TabsItem; 5] {
    [
        TabsItem::new("overview").label("Overview"),
        TabsItem::new("activity").label("Activity"),
        TabsItem::new("metrics").label("Metrics"),
        TabsItem::new("more").label("More").dropdown_trigger(),
        TabsItem::new("settings").label("Settings").enabled(false),
    ]
}

fn uniform_width_tabs() -> [TabsItem; 4] {
    [
        TabsItem::new("home").label("Home"),
        TabsItem::new("recent-activity").label("Recent Activity"),
        TabsItem::new("api-integrations").label("API & Integrations"),
        TabsItem::new("settings").label("Settings").enabled(false),
    ]
}

/// Retained SDK-backed page: its counter makes state preservation observable.
struct RetainedTabPage {
    look: Arc<ShadcnLook>,
    label: gpui::SharedString,
    count: usize,
    increment: Entity<xgpui_luma::controls::button::Button>,
}
impl RetainedTabPage {
    fn new(label: gpui::SharedString, look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let increment = shadcn::Button::new("tabs-dnd-increment")
            .look(look.as_ref())
            .label("Increment")
            .on_click(cx, |page: &mut Self, cx| {
                page.count += 1;
                cx.notify();
            })
            .spawn(cx);
        Self { look, label, count: 0, increment }
    }
}
impl Render for RetainedTabPage {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .p(px(12.0))
            .text_sm()
            .text_color(self.look.chrome().body_text)
            .child(
                div()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(self.look.chrome().title_text)
                    .child(self.label.clone()),
            )
            .child(format!("Count: {}", self.count))
            .child(self.increment.clone())
    }
}
