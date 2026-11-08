//! Resizable panels control exposition — horizontal, vertical, and controlled splits.

use xgpui_luma::controls::resizable_panels::ResizablePanelsEventExt;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, Render, Subscription, Window, div, prelude::*, px, transparent_black};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::button::{Button, HasPresenter};
use xgpui_luma::controls::resizable_panels::{
    ResizablePanelSpec, ResizablePanels, ResizablePanelsEvent, ResizablePanelsOrientation, ResizeHandleSize,
};
use xgpui_luma::resizable_panels;
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextSize};

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::inspector::{ResizablePanelsInspectorAdapter, RESIZABLE_PANELS_INSPECTOR_SPEC, resizable_panels_motion_report};
use super::shell_theme_inspectors::ResizablePanelsThemeInspector;
use super::template::render_control_exposition_card;

const DEMO_WIDTH: f32 = 480.0;
const DEMO_HEIGHT: f32 = 200.0;
const PANEL_BG_TRANSPARENT: gpui::Hsla = transparent_black();

pub struct ResizablePanelsControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<ResizablePanelsExpositionLeftPane>,
    theme_inspector: Entity<ResizablePanelsThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct ResizablePanelsExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    horizontal: Entity<ResizablePanels>,
    vertical: Entity<ResizablePanels>,
    controlled: Entity<ResizablePanels>,
    animated_panels: Entity<ResizablePanels>,
    instant_panels: Entity<ResizablePanels>,
    reset_controlled_button: Entity<Button>,
    toggle_animated_button: Entity<Button>,
    toggle_instant_button: Entity<Button>,
    controlled_panel_sizes: Rc<Cell<[f32; 2]>>,
    horizontal_sizes: String,
    vertical_sizes: String,
    controlled_sizes: String,
    animated_sizes: String,
    instant_sizes: String,
    event_stream: Entity<ControlEventStream>,
}

impl ResizablePanelsExpositionLeftPane {
    fn apply_sizes_event(
        &mut self,
        event: &ResizablePanelsEvent,
        assign: impl FnOnce(&mut Self, String),
        cx: &mut Context<Self>,
    ) {
        let sizes = match event {
            ResizablePanelsEvent::SizesChanged { sizes_px } | ResizablePanelsEvent::ResizeEnd { sizes_px } => sizes_px,
            _ => return,
        };
        assign(self, format_sizes(sizes));
        cx.notify();
    }

    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.horizontal.update(cx, |_, cx| cx.notify());
        self.vertical.update(cx, |_, cx| cx.notify());
        self.controlled.update(cx, |_, cx| cx.notify());
        self.animated_panels.update(cx, |_, cx| cx.notify());
        self.instant_panels.update(cx, |_, cx| cx.notify());
        self.toggle_animated_button.update(cx, |_, cx| cx.notify());
        self.toggle_instant_button.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for ResizablePanelsExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let look = &self.look;
            let chrome = look.chrome();
            let label_color = chrome.muted_text;

            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(16.0))
                .child(demo_frame(
                    look,
                    "Horizontal",
                    &self.horizontal_sizes,
                    label_color,
                    chrome.border,
                    self.horizontal.clone(),
                ))
                .child(demo_frame(
                    look,
                    "Vertical",
                    &self.vertical_sizes,
                    label_color,
                    chrome.border,
                    self.vertical.clone(),
                ))
                .child(demo_frame(
                    look,
                    "Controlled min/max",
                    &self.controlled_sizes,
                    label_color,
                    chrome.border,
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(8.0))
                        .child(self.controlled.clone())
                        .child(self.reset_controlled_button.clone()),
                ))
                .child(demo_frame(
                    look,
                    "Animated transition (motion_enabled: true)",
                    &self.animated_sizes,
                    label_color,
                    chrome.border,
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(8.0))
                        .child(self.animated_panels.clone())
                        .child(self.toggle_animated_button.clone()),
                ))
                .child(demo_frame(
                    look,
                    "Instant / Non-animated (motion_enabled: false)",
                    &self.instant_sizes,
                    label_color,
                    chrome.border,
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(8.0))
                        .child(self.instant_panels.clone())
                        .child(self.toggle_instant_button.clone()),
                ))
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-resizable-panels-left-pane")
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

impl ResizablePanelsControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("resizable-panels").expect("resizable-panels catalog entry");
        let demo_theme = look.clone();

        let horizontal = look
            .resizable_panels("controls-doc-resizable-horizontal")
            .orientation(ResizablePanelsOrientation::Horizontal)
            .size(px(DEMO_WIDTH), px(DEMO_HEIGHT))
            .show_handle(true)
            .resize_handle(ResizeHandleSize::Md)
            .handle_grip(true)
            .panels([
                ResizablePanelSpec::new_render({
                    let theme = demo_theme.clone();
                    move || demo_label("Sidebar", &theme)
                })
                .weight(3.0)
                .bg(PANEL_BG_TRANSPARENT),
                ResizablePanelSpec::new_render({
                    let theme = demo_theme.clone();
                    move || demo_label("Content", &theme)
                })
                .weight(7.0)
                .bg(PANEL_BG_TRANSPARENT),
            ])
            .spawn(cx);

        let vertical = resizable_panels! {
            cx,
            theme = look.resizable_panels_theme(),
            id: "controls-doc-resizable-vertical",
            layout: Vertical,
            size: (px(DEMO_WIDTH), px(DEMO_HEIGHT)),
            panels: [
                {
                    let theme = demo_theme.clone();
                    move || demo_label("Header", &theme)
                } => weight(3.0), bg: PANEL_BG_TRANSPARENT;
                |
                {
                    let theme = demo_theme.clone();
                    move || demo_label("Content", &theme)
                } => weight(7.0), bg: PANEL_BG_TRANSPARENT;
            ]
        };

        let controlled_panel_sizes = Rc::new(Cell::new([DEMO_WIDTH * 0.3, DEMO_WIDTH * 0.7]));
        let left_sizes = controlled_panel_sizes.clone();
        let right_sizes = controlled_panel_sizes.clone();
        let controlled_min_left = DEMO_WIDTH * 0.2;
        let controlled_max_left = DEMO_WIDTH * 0.7;
        let controlled_min_right = DEMO_WIDTH * 0.3;
        let controlled_max_right = DEMO_WIDTH * 0.8;

        let controlled = resizable_panels! {
            cx,
            theme = look.resizable_panels_theme(),
            id: "controls-doc-resizable-controlled",
            layout: Horizontal,
            show_handle: true,
            resize_handle: Lg,
            handle_grip: true,
            size: (px(DEMO_WIDTH), px(DEMO_HEIGHT)),
            panels: [
                {
                    let theme = demo_theme.clone();
                    let left_sizes = left_sizes.clone();
                    move || {
                        let sizes = left_sizes.get();
                        div()
                            .size_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_sm()
                            .text_color(label_foreground(&theme))
                            .child(format!("{:.0}%", percent_of_content(sizes[0])))
                            .into_any_element()
                    }
                } => weight(3.0),
                    min: px(controlled_min_left),
                    max: px(controlled_max_left),
                    bg: PANEL_BG_TRANSPARENT;
                |
                {
                    let theme = demo_theme.clone();
                    let right_sizes = right_sizes.clone();
                    move || {
                        let sizes = right_sizes.get();
                        div()
                            .size_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_sm()
                            .text_color(label_foreground(&theme))
                            .child(format!("{:.0}%", percent_of_content(sizes[1])))
                            .into_any_element()
                    }
                } => weight(7.0),
                    min: px(controlled_min_right),
                    max: px(controlled_max_right),
                    bg: PANEL_BG_TRANSPARENT;
            ]
        };

        let reset_controlled_button = shadcn::Button::new("controls-doc-resizable-controlled-reset")
            .look(look.as_ref())
            .secondary()
            .label("Reset (30 / 70)")
            .spawn(cx);

        let animated_panels = resizable_panels! {
            cx,
            theme = look.resizable_panels_theme(),
            id: "controls-doc-resizable-animated",
            layout: Horizontal,
            show_handle: true,
            resize_handle: Sm,
            handle_grip: true,
            size: (px(DEMO_WIDTH), px(DEMO_HEIGHT)),
            panels: [
                {
                    let theme = demo_theme.clone();
                    move || demo_label("Left Sidebar", &theme)
                } => weight(3.0), bg: PANEL_BG_TRANSPARENT;
                |
                {
                    let theme = demo_theme.clone();
                    move || demo_label("Main Area", &theme)
                } => weight(7.0), bg: PANEL_BG_TRANSPARENT;
            ]
        };

        let instant_panels = look
            .resizable_panels("controls-doc-resizable-instant")
            .orientation(ResizablePanelsOrientation::Horizontal)
            .motion_enabled(false)
            .show_handle(true)
            .resize_handle(ResizeHandleSize::Sm)
            .handle_grip(true)
            .size(px(DEMO_WIDTH), px(DEMO_HEIGHT))
            .panels([
                ResizablePanelSpec::new_render({
                    let theme = demo_theme.clone();
                    move || demo_label("Left Sidebar", &theme)
                })
                .weight(3.0)
                .bg(PANEL_BG_TRANSPARENT),
                ResizablePanelSpec::new_render({
                    let theme = demo_theme.clone();
                    move || demo_label("Main Area", &theme)
                })
                .weight(7.0)
                .bg(PANEL_BG_TRANSPARENT),
            ])
            .spawn(cx);

        let toggle_animated_button = shadcn::Button::new("controls-doc-resizable-animated-toggle")
            .look(look.as_ref())
            .secondary()
            .label("Toggle Left Panel (Animated)")
            .spawn(cx);

        let toggle_instant_button = shadcn::Button::new("controls-doc-resizable-instant-toggle")
            .look(look.as_ref())
            .secondary()
            .label("Toggle Left Panel (Instant / Non-Animated)")
            .spawn(cx);

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-resizable-panels-event-log",
                "Drag panel handles; ResizablePanelsEvent variants appear below.",
            )
        });

        let left_pane = cx.new(|_| ResizablePanelsExpositionLeftPane {
            look: look.clone(),
            entry,
            horizontal: horizontal.clone(),
            vertical: vertical.clone(),
            controlled: controlled.clone(),
            animated_panels: animated_panels.clone(),
            instant_panels: instant_panels.clone(),
            reset_controlled_button: reset_controlled_button.clone(),
            toggle_animated_button: toggle_animated_button.clone(),
            toggle_instant_button: toggle_instant_button.clone(),
            controlled_panel_sizes: controlled_panel_sizes.clone(),
            horizontal_sizes: format_sizes(&[DEMO_WIDTH * 0.3, DEMO_WIDTH * 0.7]),
            vertical_sizes: format_sizes(&[DEMO_HEIGHT * 0.3, DEMO_HEIGHT * 0.7]),
            controlled_sizes: format_sizes(&[DEMO_WIDTH * 0.3, DEMO_WIDTH * 0.7]),
            animated_sizes: format_sizes(&[DEMO_WIDTH * 0.3, DEMO_WIDTH * 0.7]),
            instant_sizes: format_sizes(&[DEMO_WIDTH * 0.3, DEMO_WIDTH * 0.7]),
            event_stream: event_stream.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-resizable-panels-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &RESIZABLE_PANELS_INSPECTOR_SPEC,
            ResizablePanelsInspectorAdapter::shared(),
        );
        theme_inspector.update(cx, |inspector, cx| {
            inspector.set_motion_report(
                resizable_panels_motion_report([
                    ("Horizontal", horizontal.clone()),
                    ("Vertical", vertical.clone()),
                    ("Controlled min/max", controlled.clone()),
                    ("Animated transition", animated_panels.clone()),
                    ("Instant", instant_panels.clone()),
                ]),
                cx,
            );
        });

        let subscriptions = Vec::new();
        horizontal.clone().on_event(cx, {
            let event_stream = event_stream.clone();
            let left_pane = left_pane.downgrade();
            move |_, event, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };
                left_pane.update(cx, |pane, cx| {
                    pane.apply_sizes_event(event, |pane, sizes| pane.horizontal_sizes = sizes, cx);
                });
                append_resizable_event(&event_stream, event, cx);
            }
        });
        vertical.clone().on_event(cx, {
            let event_stream = event_stream.clone();
            let left_pane = left_pane.downgrade();
            move |_, event, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };
                left_pane.update(cx, |pane, cx| {
                    pane.apply_sizes_event(event, |pane, sizes| pane.vertical_sizes = sizes, cx);
                });
                append_resizable_event(&event_stream, event, cx);
            }
        });
        controlled.clone().on_event(cx, {
            let event_stream = event_stream.clone();
            let left_pane = left_pane.downgrade();
            move |_, event, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };
                left_pane.update(cx, |pane, cx| {
                    if let ResizablePanelsEvent::SizesChanged { sizes_px }
                    | ResizablePanelsEvent::ResizeEnd { sizes_px } = event
                        && sizes_px.len() >= 2
                    {
                        pane.controlled_panel_sizes.set([sizes_px[0], sizes_px[1]]);
                    }
                    pane.apply_sizes_event(event, |pane, sizes| pane.controlled_sizes = sizes, cx);
                });
                append_resizable_event(&event_stream, event, cx);
            }
        });
        reset_controlled_button.clone().on_click(cx, {
            let left_pane = left_pane.downgrade();
            move |_, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };

                left_pane.update(cx, |pane, cx| {
                    pane.controlled.update(cx, |panels, cx| panels.set_weights(vec![3.0, 7.0], cx));
                    pane.controlled_panel_sizes.set([DEMO_WIDTH * 0.3, DEMO_WIDTH * 0.7]);
                    pane.controlled_sizes = format_sizes(&[DEMO_WIDTH * 0.3, DEMO_WIDTH * 0.7]);
                    cx.notify();
                });
            }
        });
        toggle_animated_button.clone().on_click(cx, {
            let left_pane = left_pane.downgrade();
            move |_, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };

                left_pane.update(cx, |pane, cx| {
                    pane.animated_panels.update(cx, |panels, cx| {
                        panels.toggle_panel_hidden(
                            0,
                            xgpui_luma::controls::resizable_panels::PanelHideMode::Completely,
                            cx,
                        );
                    });
                });
            }
        });
        animated_panels.clone().on_event(cx, {
            let event_stream = event_stream.clone();
            let left_pane = left_pane.downgrade();
            move |_, event, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };
                left_pane.update(cx, |pane, cx| {
                    pane.apply_sizes_event(event, |pane, sizes| pane.animated_sizes = sizes, cx);
                });
                append_resizable_event(&event_stream, event, cx);
            }
        });
        toggle_instant_button.clone().on_click(cx, {
            let left_pane = left_pane.downgrade();
            move |_, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };

                left_pane.update(cx, |pane, cx| {
                    pane.instant_panels.update(cx, |panels, cx| {
                        panels.toggle_panel_hidden(
                            0,
                            xgpui_luma::controls::resizable_panels::PanelHideMode::Completely,
                            cx,
                        );
                    });
                });
            }
        });
        instant_panels.clone().on_event(cx, {
            let event_stream = event_stream.clone();
            let left_pane = left_pane.downgrade();
            move |_, event, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };
                left_pane.update(cx, |pane, cx| {
                    pane.apply_sizes_event(event, |pane, sizes| pane.instant_sizes = sizes, cx);
                });
                append_resizable_event(&event_stream, event, cx);
            }
        });

        Self { look, entry, left_pane, theme_inspector, inspector_split, _subscriptions: subscriptions }
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

impl Render for ResizablePanelsControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-resizable-panels-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn demo_frame(
    look: &ShadcnLook,
    title: &str,
    sizes: &str,
    label_color: gpui::Hsla,
    border_color: gpui::Hsla,
    content: impl IntoElement,
) -> impl IntoElement {
    let title_style = look.typography_scale(ShadcnTextSize::Sm);
    let size_style = look.typography_scale(ShadcnTextSize::Xs);
    let title = title.to_string();
    let sizes = sizes.to_string();

    div()
        .w_full()
        .max_w(px(520.0))
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(div().typography_style(title_style).text_color(label_color).child(title))
        .child(div().typography_style(size_style).text_color(label_color).child(format!("Sizes: {sizes}")))
        .child(
            div()
                .w_full()
                .border_1()
                .border_dashed()
                .border_color(border_color)
                .rounded(px(10.0))
                .p(px(12.0))
                .child(content),
        )
}

fn append_resizable_event(
    event_stream: &Entity<ControlEventStream>,
    event: &ResizablePanelsEvent,
    cx: &mut Context<ResizablePanelsControlExposition>,
) {
    if let Some(line) = format_resizable_panels_event(event) {
        event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
    }
}

fn format_resizable_panels_event(event: &ResizablePanelsEvent) -> Option<String> {
    match event {
        ResizablePanelsEvent::ResizeStart => Some("ResizablePanelsEvent::ResizeStart".to_string()),
        ResizablePanelsEvent::SizesChanged { sizes_px } => {
            Some(format!("ResizablePanelsEvent::SizesChanged {{ sizes_px: {:?} }}", format_sizes(sizes_px)))
        }
        ResizablePanelsEvent::ResizeEnd { sizes_px } => {
            Some(format!("ResizablePanelsEvent::ResizeEnd {{ sizes_px: {:?} }}", format_sizes(sizes_px)))
        }
        ResizablePanelsEvent::PanelHiddenChanged { panel_index, hidden } => Some(format!(
            "ResizablePanelsEvent::PanelHiddenChanged {{ panel_index: {panel_index}, hidden: {hidden} }}"
        )),
        ResizablePanelsEvent::HandleFocusChanged { handle_index, focused } => Some(format!(
            "ResizablePanelsEvent::HandleFocusChanged {{ handle_index: {handle_index}, focused: {focused} }}"
        )),
        ResizablePanelsEvent::HandleHoverChanged { handle_index, hovered } => Some(format!(
            "ResizablePanelsEvent::HandleHoverChanged {{ handle_index: {handle_index}, hovered: {hovered} }}"
        )),
        _ => None,
    }
}

fn format_sizes(sizes_px: &[f32]) -> String {
    sizes_px.iter().map(|size| format!("{size:.0}px")).collect::<Vec<_>>().join(" / ")
}

fn percent_of_content(size_px: f32) -> f32 {
    size_px / DEMO_WIDTH.max(1.0) * 100.0
}

fn label_foreground(theme: &ShadcnLook) -> gpui::Hsla {
    theme.token_color("foreground").unwrap_or_else(|_| theme.chrome().body_text)
}

fn demo_label(text: &str, theme: &ShadcnLook) -> AnyElement {
    let text = text.to_string();
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .text_sm()
        .text_color(label_foreground(theme))
        .child(text)
        .into_any_element()
}
