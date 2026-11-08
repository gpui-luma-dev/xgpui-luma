//! Slide panel exposition — gallery-aligned drawer prototype with edge triggers.

use xgpui_luma::controls::toggle::ToggleEventExt;
use std::sync::Arc;

use gpui::{
    AnyElement, App, Context, Entity, FocusHandle, Focusable, FontWeight, Hsla, IntoElement, MouseDownEvent,
    MouseUpEvent, Overflow, Render, Window, div, prelude::*, px,
};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::button::{Button, HasPresenter};
use xgpui_luma::controls::slide_panel::{
    SlidePanelEdge, SlidePanelMotion, SlidePanelOverlayHandlers, SlidePanelResizeDrag, SlidePanelResizeHandlers,
    SlidePanelSizeConfig, SlidePanelState, SlidePanelTopAnchor, render_slide_panel_overlay,
};
use xgpui_luma::controls::toggle::{Toggle, ToggleEvent};
use xgpui_luma::{flow, hstack, vstack};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::{ShadcnLook, slide_panel_background, slide_panel_panels_look};
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{ViewportInspectorPane, spawn_viewport_inspector, sync_viewport_inspector};
use super::inspector::{MotionOnlyInspector, SLIDE_PANEL_INSPECTOR_SPEC, slide_panel_motion_report};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::theme_inspector::ThemeInspector;
use super::template::render_control_exposition_card;

const SIDE_PANEL_DEFAULT_WIDTH: f32 = 360.0;
const SIDE_PANEL_MIN_WIDTH: f32 = 280.0;
const SIDE_PANEL_MAX_WIDTH: f32 = 720.0;
const EDGE_PANEL_DEFAULT_HEIGHT: f32 = 420.0;
const EDGE_PANEL_MIN_HEIGHT: f32 = 240.0;
const EDGE_PANEL_MAX_HEIGHT: f32 = 640.0;

const PANEL_SIDE_TOP_PADDING: f32 = 42.0;
const PANEL_VERTICAL_TOP_PADDING: f32 = 22.0;
const PANEL_CONTENT_INSET: f32 = 22.0;
const PANEL_SECTION_GAP: f32 = 18.0;
const PANEL_ACTION_GAP: f32 = 10.0;
const PANEL_HEADER_GAP: f32 = 16.0;
const PANEL_HEADER_TEXT_GAP: f32 = 6.0;

const STAGE_MIN_HEIGHT: f32 = 360.0;
const STAGE_RADIUS: f32 = 12.0;
const STAGE_PADDING: f32 = 20.0;
const STAGE_TRIGGER_GAP: f32 = 12.0;

const STATUS_RADIUS: f32 = 12.0;
const STATUS_PADDING: f32 = 14.0;
const STATUS_BORDER_OPACITY: f32 = 0.78;
const STATUS_BACKGROUND_OPACITY: f32 = 0.36;

const INFO_CARD_RADIUS: f32 = 10.0;
const INFO_CARD_PADDING: f32 = 12.0;
const STATS_BORDER_OPACITY: f32 = 0.88;
const STATS_BACKGROUND_OPACITY: f32 = 0.38;
const NOTE_BORDER_OPACITY: f32 = 0.72;
const NOTE_BACKGROUND_OPACITY: f32 = 0.22;

const PANEL_TITLE_TEXT_SIZE: f32 = 20.0;
const PANEL_TITLE_LINE_HEIGHT: f32 = 26.0;
const BODY_TEXT_SIZE: f32 = 14.0;
const BODY_LINE_HEIGHT: f32 = 20.0;
const DETAIL_TEXT_SIZE: f32 = 13.0;
const DETAIL_LINE_HEIGHT: f32 = 18.0;
const LABEL_TEXT_SIZE: f32 = 12.0;
const LABEL_LINE_HEIGHT: f32 = 16.0;
const NOTE_LINE_HEIGHT: f32 = 18.0;

const PROGRESS_PERCENT_SCALE: f32 = 100.0;

pub struct SlidePanelControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<SlidePanelExpositionLeftPane>,
    theme_inspector: Entity<ThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

struct SlidePanelExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    demo: Entity<SlidePanelDemo>,
    event_stream: Entity<ControlEventStream>,
}

impl SlidePanelControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("slide-panel").expect("slide-panel catalog entry");

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-slide-panel-event-log",
                "Open drawers from each edge; status and dismissal events appear below.",
            )
        });

        let demo = cx.new(|cx| SlidePanelDemo::new(look.clone(), event_stream.clone(), cx));
        let left_pane = cx.new(|_| SlidePanelExpositionLeftPane {
            look: look.clone(),
            entry,
            demo: demo.clone(),
            event_stream: event_stream.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-slide-panel-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &SLIDE_PANEL_INSPECTOR_SPEC,
            MotionOnlyInspector::shared(),
        );
        let demo_for_motion = demo.clone();
        theme_inspector.update(cx, |inspector, cx| {
            inspector.set_motion_report(
                slide_panel_motion_report([(
                    "Drawer",
                    Arc::new(move |cx: &App| {
                        let demo = demo_for_motion.read(cx);
                        (demo.state.is_motion_enabled(), demo.state.motion_profile())
                    }) as Arc<dyn Fn(&App) -> (bool, SlidePanelMotion) + Send + Sync>,
                )]),
                cx,
            );
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

impl SlidePanelExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.demo.update(cx, |demo, cx| demo.sync_look(cx));
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for SlidePanelControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let inspector_split = self.inspector_split.clone();
        with_look(&self.look, || {
            div()
                .id("controls-doc-slide-panel-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(inspector_split)
        })
    }
}

impl Render for SlidePanelExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let look = &self.look;
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(12.0))
                .child(div().w_full().h(px(420.0)).min_w(px(0.0)).child(self.demo.clone()))
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-slide-panel-left-pane")
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

struct SlidePanelDemo {
    look: Arc<ShadcnLook>,
    trigger_left: Entity<Button>,
    trigger_right: Entity<Button>,
    trigger_top: Entity<Button>,
    trigger_bottom: Entity<Button>,
    close_button: Entity<Button>,
    primary_action: Entity<Button>,
    secondary_action: Entity<Button>,
    archive_action: Entity<Button>,
    backdrop_toggle: Toggle,
    state: SlidePanelState,
    last_action: String,
    event_stream: Entity<ControlEventStream>,
}

impl SlidePanelDemo {
    fn new(look: Arc<ShadcnLook>, event_stream: Entity<ControlEventStream>, cx: &mut Context<Self>) -> Self {
        let trigger_left = shadcn::Button::new("controls-doc-slide-open-left")
            .look(look.as_ref())
            .primary()
            .label("Open Left")
            .spawn(cx);
        let trigger_right = shadcn::Button::new("controls-doc-slide-open-right")
            .look(look.as_ref())
            .primary()
            .label("Open Right")
            .spawn(cx);
        let trigger_top = shadcn::Button::new("controls-doc-slide-open-top")
            .look(look.as_ref())
            .primary()
            .label("Open Top")
            .spawn(cx);
        let trigger_bottom = shadcn::Button::new("controls-doc-slide-open-bottom")
            .look(look.as_ref())
            .primary()
            .label("Open Bottom")
            .spawn(cx);
        let close_button = shadcn::Button::icon_button("controls-doc-slide-close", LucideIcon::X)
            .look(look.as_ref())
            .ghost()
            .spawn(cx);
        let primary_action = shadcn::Button::new("controls-doc-slide-primary")
            .look(look.as_ref())
            .primary()
            .label("Apply Changes")
            .spawn(cx);
        let secondary_action = shadcn::Button::new("controls-doc-slide-secondary")
            .look(look.as_ref())
            .secondary()
            .label("Review Draft")
            .spawn(cx);
        let archive_action = shadcn::Button::new("controls-doc-slide-archive")
            .look(look.as_ref())
            .ghost()
            .label("Archive")
            .spawn(cx);
        let backdrop_toggle = shadcn::Toggle::new("controls-doc-slide-backdrop-toggle")
            .look(look.as_ref())
            .secondary()
            .with_data(true)
            .content(|model, _| {
                let label = if model.data.selected {
                    "Backdrop click closes: On"
                } else {
                    "Backdrop click closes: Off"
                };
                div().child(label).into_any_element()
            })
            .spawn(cx);

        let this = Self {
            look: look.clone(),
            trigger_left: trigger_left.clone(),
            trigger_right: trigger_right.clone(),
            trigger_top: trigger_top.clone(),
            trigger_bottom: trigger_bottom.clone(),
            close_button: close_button.clone(),
            primary_action: primary_action.clone(),
            secondary_action: secondary_action.clone(),
            archive_action: archive_action.clone(),
            backdrop_toggle: backdrop_toggle.clone(),
            state: SlidePanelState::new(
                SlidePanelTopAnchor::WindowEdge,
                SlidePanelSizeConfig::new(SIDE_PANEL_DEFAULT_WIDTH, SIDE_PANEL_MIN_WIDTH, SIDE_PANEL_MAX_WIDTH),
            )
            .motion_defaults(look.motion_defaults().slide_panel),
            last_action: "Panel closed. Use one of the edge triggers to open a drawer.".to_string(),
            event_stream: event_stream.clone(),
        };

        this.wire_trigger(&trigger_left, SlidePanelEdge::Left, cx);
        this.wire_trigger(&trigger_right, SlidePanelEdge::Right, cx);
        this.wire_trigger(&trigger_top, SlidePanelEdge::Top, cx);
        this.wire_trigger(&trigger_bottom, SlidePanelEdge::Bottom, cx);

        close_button.clone().on_click(cx, {
            let event_stream = event_stream.downgrade();
            move |this, cx| {
                let Some(event_stream) = event_stream.upgrade() else {
                    return;
                };

                this.set_action("Closed from the panel close button.", &event_stream, cx);
                this.state.request_close();
            }
        });

        for (button, label) in [
            (&primary_action, "Primary action pressed while the panel stayed open."),
            (&secondary_action, "Secondary review action fired."),
            (&archive_action, "Ghost archive action fired."),
        ] {
            let event_stream = event_stream.clone();
            let label = label.to_string();
            button.clone().on_click(cx, move |this, cx| {
                this.set_action(&label, &event_stream, cx);
            });
        }

        backdrop_toggle.clone().on_event(cx, move |this, event: &ToggleEvent, cx| {
            let ToggleEvent::Change { selected } = event else {
                return;
            };
            let enabled = *selected;
            this.state.set_backdrop_click_closes(enabled);
            let message = if enabled {
                "Backdrop click dismissal enabled."
            } else {
                "Backdrop click dismissal disabled."
            };
            this.set_action(message, &event_stream, cx);
        });

        this
    }

    fn wire_trigger(&self, button: &Entity<Button>, edge: SlidePanelEdge, cx: &mut Context<Self>) {
        let event_stream = self.event_stream.clone();
        button.clone().on_click(cx, move |this, cx| {
            this.open_from_trigger(edge, cx);
            this.set_action(&format!("Opened the {} slide panel.", edge.label()), &event_stream, cx);
        });
    }

    fn set_action(&mut self, message: &str, event_stream: &Entity<ControlEventStream>, cx: &mut Context<Self>) {
        self.last_action = message.to_string();
        event_stream.update(cx, |stream, cx| stream.append_line(message, cx));
        cx.notify();
    }

    fn sync_look(&mut self, cx: &mut Context<Self>) {
        for button in [
            &self.trigger_left,
            &self.trigger_right,
            &self.trigger_top,
            &self.trigger_bottom,
            &self.close_button,
            &self.primary_action,
            &self.secondary_action,
            &self.archive_action,
        ] {
            button.update(cx, |_, cx| cx.notify());
        }
        self.backdrop_toggle.update(cx, |_, cx| cx.notify());
        cx.notify();
    }

    fn open_from_trigger(&mut self, edge: SlidePanelEdge, cx: &mut Context<Self>) {
        let opener = match edge {
            SlidePanelEdge::Left => self.trigger_left.read(cx).focus_handle(cx),
            SlidePanelEdge::Right => self.trigger_right.read(cx).focus_handle(cx),
            SlidePanelEdge::Top => self.trigger_top.read(cx).focus_handle(cx),
            SlidePanelEdge::Bottom => self.trigger_bottom.read(cx).focus_handle(cx),
        };
        self.configure_size_for_edge(edge);
        self.state.open(edge, opener);
    }

    fn configure_size_for_edge(&mut self, edge: SlidePanelEdge) {
        let config = match edge {
            SlidePanelEdge::Left | SlidePanelEdge::Right => {
                SlidePanelSizeConfig::new(SIDE_PANEL_DEFAULT_WIDTH, SIDE_PANEL_MIN_WIDTH, SIDE_PANEL_MAX_WIDTH)
            }
            SlidePanelEdge::Top | SlidePanelEdge::Bottom => {
                SlidePanelSizeConfig::new(EDGE_PANEL_DEFAULT_HEIGHT, EDGE_PANEL_MIN_HEIGHT, EDGE_PANEL_MAX_HEIGHT)
            }
        };
        self.state.set_size_config(config);
    }

    fn extra_focus_handles(&self, cx: &App) -> Vec<FocusHandle> {
        vec![
            self.primary_action.read(cx).focus_handle(cx),
            self.secondary_action.read(cx).focus_handle(cx),
            self.archive_action.read(cx).focus_handle(cx),
            self.backdrop_toggle.read(cx).focus_handle(cx),
        ]
    }

    fn handle_overlay_key_down(&mut self, event: &gpui::KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        match event.keystroke.key.as_str() {
            "escape" if self.state.handle_escape(window, cx) => {
                let event_stream = self.event_stream.clone();
                self.set_action("Closed with Escape and restored focus to the trigger.", &event_stream, cx);
            }
            "tab" => {
                let close_focus = self.close_button.read(cx).focus_handle(cx);
                let extra_focuses = self.extra_focus_handles(cx);
                if self.state.handle_tab_navigation(close_focus, &extra_focuses, event, window, cx) {
                    cx.notify();
                }
            }
            _ => {}
        }
    }

    fn handle_backdrop_mouse_down(&mut self, _event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.state.backdrop_click_closes() || self.state.is_resizing() {
            return;
        }

        if self.state.request_close() {
            window.prevent_default();
            cx.stop_propagation();
            let event_stream = self.event_stream.clone();
            self.set_action("Closed by clicking the backdrop.", &event_stream, cx);
        }
    }

    fn handle_resize_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let Some(edge) = self.state.active_edge() else {
            return;
        };

        if self.state.begin_resize(event.position, edge) {
            window.prevent_default();
            cx.stop_propagation();
            cx.notify();
        }
    }

    fn handle_resize_drag_move(
        &mut self,
        event: &gpui::DragMoveEvent<SlidePanelResizeDrag>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(edge) = self.state.active_edge() else {
            return;
        };

        if self.state.update_resize(event.event.position, edge) {
            cx.notify();
        }
    }

    fn handle_resize_mouse_up(&mut self, event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if event.button != gpui::MouseButton::Left {
            return;
        }

        if self.state.finish_resize() {
            cx.notify();
        }
    }

    fn handle_resize_hover(&mut self, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>) {
        if self.state.set_resize_handle_hovered(*hovered) {
            cx.notify();
        }
    }

    fn render_panel_content(&self, edge: SlidePanelEdge, look: &ShadcnLook) -> AnyElement {
        let chrome = look.chrome();
        let panel_foreground = look.token_color("foreground").unwrap_or(chrome.title_text);
        let muted = look.token_color("muted-foreground").unwrap_or(chrome.muted_text);
        let border = look.token_color("border").unwrap_or(chrome.border);
        let content_padding_top = px(match edge {
            SlidePanelEdge::Left | SlidePanelEdge::Right => PANEL_SIDE_TOP_PADDING,
            SlidePanelEdge::Top | SlidePanelEdge::Bottom => PANEL_VERTICAL_TOP_PADDING,
        });

        let mut panel_content = vstack! {
            gap=PANEL_SECTION_GAP justify=between;
            vstack! {
                gap=PANEL_SECTION_GAP;
                render_panel_header(edge, self.close_button.clone().into_any_element(), panel_foreground, muted),
                render_panel_stats_card(
                    edge,
                    self.state.backdrop_click_closes(),
                    panel_foreground,
                    muted,
                    border,
                    chrome.content_background,
                ),
                flow! {
                    gap=PANEL_ACTION_GAP;
                    self.primary_action.clone(),
                    self.secondary_action.clone(),
                    self.archive_action.clone(),
                    self.backdrop_toggle.clone(),
                }
                .items_center(),
            },
            render_panel_focus_note(border, chrome.content_background, muted),
        }
        .size_full()
        .min_h_0()
        .pt(content_padding_top)
        .pr(px(PANEL_CONTENT_INSET))
        .pb(px(PANEL_CONTENT_INSET))
        .pl(px(PANEL_CONTENT_INSET));

        if matches!(edge, SlidePanelEdge::Top | SlidePanelEdge::Bottom) {
            panel_content = panel_content.justify_start().gap(px(PANEL_SECTION_GAP));
            panel_content.style().overflow.y = Some(Overflow::Scroll);
        }

        panel_content.into_any_element()
    }
}

impl Render for SlidePanelDemo {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _ = self.state.sync_animation();
        self.state.schedule_animation_frame(window, cx);

        let close_focus = self.close_button.read(cx).focus_handle(cx);
        self.state.schedule_pending_focus(window, cx, close_focus);

        let chrome = self.look.chrome();
        let body = self.look.token_color("foreground").unwrap_or(chrome.body_text);
        let muted = self.look.token_color("muted-foreground").unwrap_or(chrome.muted_text);
        let border = self.look.token_color("border").unwrap_or(chrome.border);
        let stage_background = self.look.token_color("card").unwrap_or(chrome.panel_background);
        let viewport = window.viewport_size();

        let stage = vstack! {
            gap=PANEL_SECTION_GAP justify=between;
            flow! {
                gap=STAGE_TRIGGER_GAP;
                self.trigger_left.clone(),
                self.trigger_right.clone(),
                self.trigger_top.clone(),
                self.trigger_bottom.clone(),
            }
            .items_center(),
            render_status_card(
                self.last_action.clone(),
                self.state.open_progress(),
                self.state.active_edge(),
                body,
                muted,
                border,
                chrome.content_background,
            ),
        }
        .w_full()
        .min_h(px(STAGE_MIN_HEIGHT))
        .rounded(px(STAGE_RADIUS))
        .border_1()
        .border_color(border)
        .bg(stage_background)
        .p(px(STAGE_PADDING));

        let mut root = div()
            .id("controls-doc-slide-panel-demo")
            .size_full()
            .relative()
            .overflow_hidden()
            .bg(chrome.content_background)
            .child(div().size_full().flex().items_center().justify_center().child(stage));

        if let Some(edge) = self.state.active_edge() {
            let handlers = SlidePanelOverlayHandlers {
                key_down: Box::new(cx.listener(Self::handle_overlay_key_down)),
                backdrop_mouse_down: Box::new(cx.listener(Self::handle_backdrop_mouse_down)),
            };
            let resize_handlers = SlidePanelResizeHandlers {
                mouse_down: Box::new(cx.listener(Self::handle_resize_mouse_down)),
                drag_move: Box::new(cx.listener(Self::handle_resize_drag_move)),
                mouse_up: Box::new(cx.listener(Self::handle_resize_mouse_up)),
                mouse_up_out: Box::new(cx.listener(Self::handle_resize_mouse_up)),
                hover: Box::new(cx.listener(Self::handle_resize_hover)),
            };
            let overlay = render_slide_panel_overlay(
                slide_panel_background(&self.look),
                &slide_panel_panels_look(&self.look),
                &self.state,
                viewport,
                self.render_panel_content(edge, &self.look),
                handlers,
                resize_handlers,
            );
            root = root.child(gpui::deferred(overlay).with_priority(10));
        }

        root
    }
}

fn render_status_card(
    last_action: String,
    open_progress: f32,
    active_edge: Option<SlidePanelEdge>,
    body: Hsla,
    muted: Hsla,
    border: Hsla,
    content_background: Hsla,
) -> AnyElement {
    vstack! {
        gap=PANEL_ACTION_GAP;
        div()
            .text_size(px(LABEL_TEXT_SIZE))
            .line_height(px(LABEL_LINE_HEIGHT))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(muted)
            .child("Status"),
        div()
            .text_size(px(BODY_TEXT_SIZE))
            .line_height(px(BODY_LINE_HEIGHT))
            .text_color(body)
            .child(last_action),
        div()
            .text_size(px(LABEL_TEXT_SIZE))
            .line_height(px(NOTE_LINE_HEIGHT))
            .text_color(muted)
            .child(format!(
                "Open progress: {:>3.0}%{}",
                open_progress * PROGRESS_PERCENT_SCALE,
                active_edge
                    .map(|edge| format!(" | active edge: {}", edge.label()))
                    .unwrap_or_else(|| " | panel closed".to_string())
            )),
    }
    .w_full()
    .rounded(px(STATUS_RADIUS))
    .border_1()
    .border_color(border.opacity(STATUS_BORDER_OPACITY))
    .bg(content_background.opacity(STATUS_BACKGROUND_OPACITY))
    .p(px(STATUS_PADDING))
    .into_any_element()
}

fn render_panel_header(edge: SlidePanelEdge, close_button: AnyElement, foreground: Hsla, muted: Hsla) -> AnyElement {
    hstack! {
        justify=between align=start gap=PANEL_HEADER_GAP;
        vstack! {
            gap=PANEL_HEADER_TEXT_GAP;
            div()
                .text_size(px(LABEL_TEXT_SIZE))
                .line_height(px(LABEL_LINE_HEIGHT))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(muted)
                .child(format!("{} Edge", edge.label())),
            div()
                .text_size(px(PANEL_TITLE_TEXT_SIZE))
                .line_height(px(PANEL_TITLE_LINE_HEIGHT))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(foreground)
                .child("Slide Panel"),
            div()
                .text_size(px(DETAIL_TEXT_SIZE))
                .line_height(px(DETAIL_LINE_HEIGHT))
                .text_color(muted)
                .child(edge.description()),
        },
        close_button,
    }
    .into_any_element()
}

fn render_panel_stats_card(
    edge: SlidePanelEdge,
    backdrop_click_closes: bool,
    foreground: Hsla,
    muted: Hsla,
    border: Hsla,
    content_background: Hsla,
) -> AnyElement {
    vstack! {
        gap=PANEL_ACTION_GAP;
        render_stat_row("Edge", edge.label(), foreground, muted),
        render_stat_row(
            "Backdrop click",
            if backdrop_click_closes { "Enabled" } else { "Disabled" },
            foreground,
            muted,
        ),
        render_stat_row("Focus", "Trapped inside panel while open", foreground, muted),
        render_stat_row("Close", "Escape, close button, or backdrop", foreground, muted),
    }
    .rounded(px(INFO_CARD_RADIUS))
    .border_1()
    .border_color(border.opacity(STATS_BORDER_OPACITY))
    .bg(content_background.opacity(STATS_BACKGROUND_OPACITY))
    .p(px(INFO_CARD_PADDING))
    .into_any_element()
}

fn render_panel_focus_note(border: Hsla, content_background: Hsla, muted: Hsla) -> AnyElement {
    div()
        .rounded(px(INFO_CARD_RADIUS))
        .border_1()
        .border_color(border.opacity(NOTE_BORDER_OPACITY))
        .bg(content_background.opacity(NOTE_BACKGROUND_OPACITY))
        .p(px(INFO_CARD_PADDING))
        .text_size(px(LABEL_TEXT_SIZE))
        .line_height(px(NOTE_LINE_HEIGHT))
        .text_color(muted)
        .child("Tab cycles through the panel actions and returns to the opener when the panel closes.")
        .into_any_element()
}

fn render_stat_row(label: &'static str, value: &'static str, foreground: Hsla, muted: Hsla) -> AnyElement {
    hstack! {
        justify=between align=center gap=STAGE_TRIGGER_GAP;
        div()
            .text_size(px(LABEL_TEXT_SIZE))
            .line_height(px(LABEL_LINE_HEIGHT))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(muted)
            .child(label),
        div()
            .text_size(px(LABEL_TEXT_SIZE))
            .line_height(px(LABEL_LINE_HEIGHT))
            .text_color(foreground)
            .child(value),
    }
    .into_any_element()
}
