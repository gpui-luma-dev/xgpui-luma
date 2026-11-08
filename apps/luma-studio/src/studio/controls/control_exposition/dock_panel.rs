//! Dock panel exposition — gallery-aligned dock_panel macro with resizable splitters.

use xgpui_luma::controls::dock_splitter::DockSplitterEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, IntoElement, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::dock_splitter::{
    DockSplitter, DockSplitterEvent, SplitterOrientation, ThemedDockSplitterTemplate,
};
use xgpui_luma::dock_panel;
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook};

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::shell_theme_inspectors::SplitViewThemeInspector;
use super::inspector::{SplitViewInspectorAdapter, SPLIT_VIEW_INSPECTOR_SPEC};
use super::template::render_control_exposition_card;

pub struct DockPanelControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<DockPanelExpositionLeftPane>,
    theme_inspector: Entity<SplitViewThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

struct DockPanelExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    demo: Entity<DockPanelDemo>,
    event_stream: Entity<ControlEventStream>,
}

impl DockPanelExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.demo.update(cx, |demo, cx| demo.sync_look(look.clone(), cx));
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for DockPanelExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let look = &self.look;
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(12.0))
                .child(
                    div()
                        .w_full()
                        .h(px(480.0))
                        .min_w(px(0.0))
                        .px(px(25.0))
                        .child(div().size_full().min_h(px(0.0)).min_w(px(0.0)).child(self.demo.clone())),
                )
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-dock-panel-left-pane")
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

impl DockPanelControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("dock-panel").expect("dock-panel catalog entry");

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-dock-panel-event-log",
                "Drag dock splitters; DockSplitterEvent variants appear below.",
            )
        });

        let left_pane = cx.new(|cx| DockPanelExpositionLeftPane {
            look: look.clone(),
            entry,
            demo: cx.new(|cx| DockPanelDemo::new(look.clone(), event_stream.clone(), cx)),
            event_stream: event_stream.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-dock-panel-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &SPLIT_VIEW_INSPECTOR_SPEC,
            SplitViewInspectorAdapter::shared(),
        );

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

impl Render for DockPanelControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-dock-panel-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

struct DockPanelDemo {
    look: Arc<ShadcnLook>,
    left_width: f32,
    right_width: f32,
    top_height: f32,
    bottom_height: f32,
    drag_start_left_width: f32,
    drag_start_right_width: f32,
    drag_start_top_height: f32,
    drag_start_bottom_height: f32,
    left_splitter: Entity<DockSplitter>,
    top_splitter: Entity<DockSplitter>,
    right_splitter: Entity<DockSplitter>,
    bottom_splitter: Entity<DockSplitter>,
}

impl DockPanelDemo {
    fn new(look: Arc<ShadcnLook>, event_stream: Entity<ControlEventStream>, cx: &mut Context<Self>) -> Self {
        let splitter_theme = look.dock_splitter_theme();
        let splitter_template = Arc::new(ThemedDockSplitterTemplate::new(true));
        let left_splitter = DockSplitter::new("controls-doc-dock-left-splitter", SplitterOrientation::Vertical)
            .template(splitter_template.clone())
            .theme(splitter_theme.clone())
            .spawn(cx);
        let top_splitter = DockSplitter::new("controls-doc-dock-top-splitter", SplitterOrientation::Horizontal)
            .template(splitter_template.clone())
            .theme(splitter_theme.clone())
            .spawn(cx);
        let right_splitter = DockSplitter::new("controls-doc-dock-right-splitter", SplitterOrientation::Vertical)
            .template(splitter_template.clone())
            .theme(splitter_theme.clone())
            .spawn(cx);
        let bottom_splitter = DockSplitter::new("controls-doc-dock-bottom-splitter", SplitterOrientation::Horizontal)
            .template(splitter_template)
            .theme(splitter_theme)
            .spawn(cx);

        let left = left_splitter.clone();
        let top = top_splitter.clone();
        let right = right_splitter.clone();
        let bottom = bottom_splitter.clone();

        let this = Self {
            look,
            left_width: 80.0,
            right_width: 80.0,
            top_height: 50.0,
            bottom_height: 50.0,
            drag_start_left_width: 80.0,
            drag_start_right_width: 80.0,
            drag_start_top_height: 50.0,
            drag_start_bottom_height: 50.0,
            left_splitter,
            top_splitter,
            right_splitter,
            bottom_splitter,
        };

        Self::wire_splitter("left", &left, &event_stream, cx, |this, delta| {
            this.left_width = (this.drag_start_left_width + delta).clamp(40.0, 300.0);
        });
        Self::wire_splitter("top", &top, &event_stream, cx, |this, delta| {
            this.top_height = (this.drag_start_top_height + delta).clamp(30.0, 180.0);
        });
        Self::wire_splitter("right", &right, &event_stream, cx, |this, delta| {
            this.right_width = (this.drag_start_right_width - delta).clamp(40.0, 300.0);
        });
        Self::wire_splitter("bottom", &bottom, &event_stream, cx, |this, delta| {
            this.bottom_height = (this.drag_start_bottom_height - delta).clamp(30.0, 180.0);
        });

        this
    }

    fn wire_splitter(
        name: &'static str,
        splitter: &Entity<DockSplitter>,
        event_stream: &Entity<ControlEventStream>,
        cx: &mut Context<Self>,
        apply_delta: fn(&mut Self, f32),
    ) {
        let event_stream = event_stream.clone();
        splitter.clone().on_event(cx, move |this, event: &DockSplitterEvent, cx| {
            match event {
                DockSplitterEvent::ResizeStart => match name {
                    "left" => this.drag_start_left_width = this.left_width,
                    "right" => this.drag_start_right_width = this.right_width,
                    "top" => this.drag_start_top_height = this.top_height,
                    _ => this.drag_start_bottom_height = this.bottom_height,
                },
                DockSplitterEvent::Resize { total_delta } => {
                    apply_delta(this, *total_delta);
                    cx.notify();
                }
                DockSplitterEvent::ResizeEnd => {}
                _ => {}
            }
            if let Some(line) = format_dock_splitter_event(name, event) {
                event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
            }
        });
    }

    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        for splitter in [&self.left_splitter, &self.top_splitter, &self.right_splitter, &self.bottom_splitter] {
            splitter.update(cx, |_, cx| cx.notify());
        }
        cx.notify();
    }
}

impl Render for DockPanelDemo {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let foreground = self.look.token_color("foreground").unwrap_or(chrome.body_text);
        let muted = self.look.token_color("muted-foreground").unwrap_or(chrome.muted_text);
        let band_background = self.look.token_color("muted").unwrap_or(chrome.content_background);
        let fill_background = self.look.token_color("accent").unwrap_or(chrome.panel_background);
        let fill_foreground = self.look.token_color("accent-foreground").unwrap_or(foreground);

        div()
            .size_full()
            .min_w(px(0.0))
            .overflow_hidden()
            .border_1()
            .border_color(chrome.border)
            .child(dock_panel! {
                left: div()
                    .w(px(self.left_width))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(foreground)
                    .child("Left"),
                left: self.left_splitter.clone(),
                top: div()
                    .h(px(self.top_height))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(foreground)
                    .child("Top"),
                top: self.top_splitter.clone(),
                right: div()
                    .w(px(self.right_width))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(foreground)
                    .child("Right"),
                right: self.right_splitter.clone(),
                bottom: div()
                    .h(px(self.bottom_height))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(foreground)
                    .child("Bottom"),
                bottom: self.bottom_splitter.clone(),
                fill: render_nested_dock_examples(
                    chrome,
                    foreground,
                    muted,
                    band_background,
                    fill_background,
                    fill_foreground,
                )
            })
    }
}

fn format_dock_splitter_event(name: &str, event: &DockSplitterEvent) -> Option<String> {
    match event {
        DockSplitterEvent::ResizeStart => Some(format!("DockSplitterEvent::ResizeStart ({name})")),
        DockSplitterEvent::Resize { total_delta } => {
            Some(format!("DockSplitterEvent::Resize {{ total_delta: {total_delta:.1} }} ({name})"))
        }
        DockSplitterEvent::ResizeEnd => Some(format!("DockSplitterEvent::ResizeEnd ({name})")),
        DockSplitterEvent::FocusChanged { focused } => {
            Some(format!("DockSplitterEvent::FocusChanged {{ focused: {focused} }} ({name})"))
        }
        DockSplitterEvent::HoverChanged { hovered } => {
            Some(format!("DockSplitterEvent::HoverChanged {{ hovered: {hovered} }} ({name})"))
        }
        DockSplitterEvent::EnabledChanged { enabled } => {
            Some(format!("DockSplitterEvent::EnabledChanged {{ enabled: {enabled} }} ({name})"))
        }
        _ => None,
    }
}

fn render_nested_dock_examples(
    chrome: xgpui_luma::theme::LumaChrome,
    foreground: gpui::Hsla,
    muted: gpui::Hsla,
    band_background: gpui::Hsla,
    fill_background: gpui::Hsla,
    fill_foreground: gpui::Hsla,
) -> gpui::Div {
    let title_style = xgpui_luma::theme::LumaTypography::default().text.role.h4;
    let body_style = xgpui_luma::theme::LumaTypography::default().text.scale.sm;

    div()
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(16.0))
        .p(px(24.0))
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(4.0))
                .child(div().typography_style(title_style).text_color(foreground).child("Nested DockPanel tests"))
                .child(
                    div()
                        .typography_style(body_style)
                        .text_color(muted)
                        .child("Inner examples verify child ordering and last-child-fill behavior."),
                ),
        )
        .child(
            div().flex().gap(px(16.0)).children([
                render_nested_example_card(
                    chrome,
                    foreground,
                    muted,
                    "Default last child fills",
                    dock_panel! {
                        top: demo_band("Top", foreground, band_background),
                        left: demo_rail("Left", foreground, band_background),
                        child: demo_fill("Fill", fill_foreground, fill_background)
                    }
                    .into_any_element(),
                ),
                render_nested_example_card(
                    chrome,
                    foreground,
                    muted,
                    "last_child_fill = false",
                    dock_panel! {
                        last_child_fill = false;
                        top: demo_band("Top", foreground, band_background),
                        child: demo_rail("Child → Left", foreground, band_background),
                        right: demo_rail("Right", foreground, band_background)
                    }
                    .into_any_element(),
                ),
            ]),
        )
}

fn render_nested_example_card(
    chrome: xgpui_luma::theme::LumaChrome,
    foreground: gpui::Hsla,
    muted: gpui::Hsla,
    title: &'static str,
    content: gpui::AnyElement,
) -> gpui::Div {
    let title_style = xgpui_luma::theme::LumaTypography::default().text.scale.sm;
    let caption_style = xgpui_luma::theme::LumaTypography::default().text.scale.xs;
    div()
        .w(px(220.0))
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(
            div()
                .typography_style(title_style)
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(foreground)
                .child(title),
        )
        .child(
            div()
                .h(px(170.0))
                .w_full()
                .overflow_hidden()
                .rounded(px(8.0))
                .border_1()
                .border_color(chrome.border)
                .bg(chrome.panel_background)
                .child(content),
        )
        .child(div().typography_style(caption_style).text_color(muted).child(match title {
            "Default last child fills" => "The final child becomes the fill region.",
            _ => "Without fill, undocked child content falls back to Left docking.",
        }))
}

fn demo_band(label: &'static str, foreground: gpui::Hsla, background: gpui::Hsla) -> gpui::Div {
    div()
        .h(px(28.0))
        .flex()
        .items_center()
        .justify_center()
        .text_xs()
        .text_color(foreground)
        .bg(background)
        .child(label)
}

fn demo_rail(label: &'static str, foreground: gpui::Hsla, background: gpui::Hsla) -> gpui::Div {
    div()
        .w(px(68.0))
        .flex()
        .items_center()
        .justify_center()
        .text_xs()
        .text_color(foreground)
        .bg(background)
        .child(label)
}

fn demo_fill(label: &'static str, foreground: gpui::Hsla, background: gpui::Hsla) -> gpui::Div {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .text_sm()
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(foreground)
        .bg(background)
        .child(label)
}
