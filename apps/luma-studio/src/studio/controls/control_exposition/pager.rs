//! Pager control exposition — gallery-aligned minimal and numeric pager styles.

use xgpui_luma::controls::pager::PagerEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, FontWeight, Render, SharedString, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::button::{Button, HasPresenter};
use xgpui_luma::controls::pager::{Pager, PagerEvent, PagerStyle};
use xgpui_luma::{vstack, wrappanel};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextSize};

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::inspector::{PagerInspectorAdapter, PAGER_INSPECTOR_SPEC};
use super::shell_theme_inspectors::PagerThemeInspector;
use super::template::render_control_exposition_card;

const PAGE_SIZES: [usize; 3] = [10, 25, 50];

pub struct PagerControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<PagerExpositionLeftPane>,
    theme_inspector: Entity<PagerThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

struct PagerExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    minimal: Pager,
    minimal_edge: Pager,
    numeric: Pager,
    numeric_no_edges: Pager,
    numeric_compact: Pager,
    page_prev_button: Entity<Button>,
    page_next_button: Entity<Button>,
    total_down_button: Entity<Button>,
    total_up_button: Entity<Button>,
    size_down_button: Entity<Button>,
    size_up_button: Entity<Button>,
    enabled_toggle_button: Entity<Button>,
    event_stream: Entity<ControlEventStream>,
    current_page: usize,
    total_items: usize,
    page_size: usize,
    enabled: bool,
}

impl PagerExpositionLeftPane {
    fn pagers(&self) -> [Pager; 5] {
        [
            self.minimal.clone(),
            self.minimal_edge.clone(),
            self.numeric.clone(),
            self.numeric_no_edges.clone(),
            self.numeric_compact.clone(),
        ]
    }

    fn handle_pager_event(&mut self, event: &PagerEvent, cx: &mut Context<Self>) {
        match event {
            PagerEvent::PageChanged { page } => {
                self.current_page = clamp_page(*page, self.page_count());
                self.sync_pagers(cx);
            }
            PagerEvent::PageSizeChanged { page_size } => {
                self.page_size = *page_size;
                self.sync_pagers(cx);
            }
            _ => {}
        }
    }

    fn step_page(&mut self, delta: isize, cx: &mut Context<Self>) {
        self.current_page = offset_usize(self.current_page, delta);
        self.current_page = clamp_page(self.current_page, self.page_count());
        self.sync_pagers(cx);
    }

    fn step_total_items(&mut self, delta: isize, cx: &mut Context<Self>) {
        let step = self.page_size.max(1);
        self.total_items = if delta < 0 {
            self.total_items.saturating_sub(step)
        } else {
            self.total_items.saturating_add(step)
        };
        self.current_page = clamp_page(self.current_page, self.page_count());
        self.sync_pagers(cx);
    }

    fn step_page_size(&mut self, delta: isize, cx: &mut Context<Self>) {
        let index = PAGE_SIZES.iter().position(|size| *size == self.page_size).unwrap_or(1);
        let next = offset_usize(index, delta).min(PAGE_SIZES.len().saturating_sub(1));
        self.page_size = PAGE_SIZES[next];
        self.sync_pagers(cx);
    }

    fn toggle_enabled(&mut self, cx: &mut Context<Self>) {
        self.enabled = !self.enabled;
        self.sync_pagers(cx);
    }

    fn sync_pagers(&mut self, cx: &mut Context<Self>) {
        let page_count = self.page_count();
        self.current_page = clamp_page(self.current_page, page_count);

        for pager in self.pagers() {
            pager.update(cx, |pager, cx| {
                pager.set_page_count(page_count, cx);
                pager.set_page_size(self.page_size, cx);
                pager.set_page(self.current_page, cx);
                pager.set_enabled(self.enabled, cx);
            });
        }

        cx.notify();
    }

    fn page_count(&self) -> usize {
        page_count_for(self.total_items, self.page_size)
    }

    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for pager in self.pagers() {
            pager.update(cx, |_, cx| cx.notify());
        }
        for button in [
            &self.page_prev_button,
            &self.page_next_button,
            &self.total_down_button,
            &self.total_up_button,
            &self.size_down_button,
            &self.size_up_button,
            &self.enabled_toggle_button,
        ] {
            button.update(cx, |_, cx| cx.notify());
        }
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for PagerExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let detail_style = self.look.typography_scale(ShadcnTextSize::Sm);
            let label_style = self.look.typography_scale(ShadcnTextSize::Xs);

            let preview = vstack! {
                gap=16.0;
                vstack! {
                    gap=12.0;
                    div().typography_style(detail_style).text_color(chrome.body_text).child(
                        format!(
                            "Current page: {} · Page count: {} · Total items: {} · Per page: {} · Enabled: {}",
                            self.current_page + 1,
                            self.page_count(),
                            self.total_items,
                            self.page_size,
                            if self.enabled { "Yes" } else { "No" }
                        ),
                    ),
                    wrappanel! {
                        gap=8.0;
                        self.page_prev_button.clone(),
                        self.page_next_button.clone(),
                        self.total_down_button.clone(),
                        self.total_up_button.clone(),
                        self.size_down_button.clone(),
                        self.size_up_button.clone(),
                        self.enabled_toggle_button.clone(),
                    },
                }
                .w_full(),
                render_sample("Minimal · formatter: \"X / Y\"", self.minimal.clone(), chrome.muted_text, label_style),
                render_sample(
                    "Minimal + first/last · formatter: \"X of Y\"",
                    self.minimal_edge.clone(),
                    chrome.muted_text,
                    label_style,
                ),
                render_sample("Numeric", self.numeric.clone(), chrome.muted_text, label_style),
                render_sample("Numeric · no first/last", self.numeric_no_edges.clone(), chrome.muted_text, label_style),
                render_sample("Numeric · 5 fixed slots", self.numeric_compact.clone(), chrome.muted_text, label_style),
                self.event_stream.clone(),
            }
            .w_full()
            .items_center();

            div()
                .id("controls-doc-pager-left-pane")
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

impl PagerControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("pager").expect("pager catalog entry");
        let current_page = 2;
        let total_items = 281;
        let page_size = 25;
        let enabled = true;
        let page_count = page_count_for(total_items, page_size);

        let minimal = shadcn::Pager::new("controls-doc-pager-minimal")
            .look(look.as_ref())
            .style(PagerStyle::Minimal)
            .page_size(page_size)
            .page_size_options(PAGE_SIZES)
            .page_count(page_count)
            .current_page(current_page)
            .page_indicator_formatter(|current, total| SharedString::from(format!("{current} / {total}")))
            .spawn(cx);
        let minimal_edge = shadcn::Pager::new("controls-doc-pager-minimal-edge")
            .look(look.as_ref())
            .style(PagerStyle::MinimalEdge)
            .page_size(page_size)
            .page_size_options(PAGE_SIZES)
            .page_count(page_count)
            .current_page(current_page)
            .page_indicator_formatter(|current, total| SharedString::from(format!("{current} of {total}")))
            .spawn(cx);
        let numeric = shadcn::Pager::new("controls-doc-pager-numeric")
            .look(look.as_ref())
            .style(PagerStyle::Numeric)
            .page_size(page_size)
            .page_size_options(PAGE_SIZES)
            .page_count(page_count)
            .current_page(current_page)
            .spawn(cx);
        let numeric_no_edges = shadcn::Pager::new("controls-doc-pager-numeric-no-edges")
            .look(look.as_ref())
            .style(PagerStyle::Numeric)
            .page_size(page_size)
            .page_size_options(PAGE_SIZES)
            .page_count(page_count)
            .current_page(current_page)
            .show_first_last(false)
            .previous_label("Back")
            .next_label("Next")
            .spawn(cx);
        let numeric_compact = shadcn::Pager::new("controls-doc-pager-numeric-compact")
            .look(look.as_ref())
            .style(PagerStyle::Numeric)
            .page_size(page_size)
            .page_size_options(PAGE_SIZES)
            .page_count(page_count)
            .current_page(current_page)
            .numeric_slot_count(5)
            .first_label("First")
            .previous_label("Back")
            .next_label("Next")
            .last_label("Last")
            .spawn(cx);

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-pager-event-log",
                "Use the pagers or demo controls; PagerEvent variants appear below.",
            )
        });

        let page_prev_button = shadcn::Button::new("controls-doc-pager-page-prev")
            .look(look.as_ref())
            .secondary()
            .label("Page -")
            .spawn(cx);
        let page_next_button = shadcn::Button::new("controls-doc-pager-page-next")
            .look(look.as_ref())
            .secondary()
            .label("Page +")
            .spawn(cx);
        let total_down_button = shadcn::Button::new("controls-doc-pager-total-down")
            .look(look.as_ref())
            .secondary()
            .label("Items -")
            .spawn(cx);
        let total_up_button = shadcn::Button::new("controls-doc-pager-total-up")
            .look(look.as_ref())
            .secondary()
            .label("Items +")
            .spawn(cx);
        let size_down_button = shadcn::Button::new("controls-doc-pager-size-down")
            .look(look.as_ref())
            .secondary()
            .label("Per page -")
            .spawn(cx);
        let size_up_button = shadcn::Button::new("controls-doc-pager-size-up")
            .look(look.as_ref())
            .secondary()
            .label("Per page +")
            .spawn(cx);
        let enabled_toggle_button = shadcn::Button::new("controls-doc-pager-enabled-toggle")
            .look(look.as_ref())
            .outline()
            .label("Toggle enabled")
            .spawn(cx);

        let left_pane = cx.new(|_| PagerExpositionLeftPane {
            look: look.clone(),
            entry,
            minimal,
            minimal_edge,
            numeric,
            numeric_no_edges,
            numeric_compact,
            page_prev_button,
            page_next_button,
            total_down_button,
            total_up_button,
            size_down_button,
            size_up_button,
            enabled_toggle_button,
            event_stream: event_stream.clone(),
            current_page,
            total_items,
            page_size,
            enabled,
        });

        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-pager-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &PAGER_INSPECTOR_SPEC,
            PagerInspectorAdapter::shared(),
        );

        for pager in left_pane.read(cx).pagers() {
            let event_stream = event_stream.clone();
            let left_pane = left_pane.clone();
            pager.clone().on_event(cx, move |_, event: &PagerEvent, cx| {
                left_pane.update(cx, |pane, cx| pane.handle_pager_event(event, cx));
                if let Some(line) = format_pager_event(event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            });
        }

        let (
            page_prev_button,
            page_next_button,
            total_down_button,
            total_up_button,
            size_down_button,
            size_up_button,
            enabled_toggle_button,
        ) = {
            let pane = left_pane.read(cx);
            (
                pane.page_prev_button.clone(),
                pane.page_next_button.clone(),
                pane.total_down_button.clone(),
                pane.total_up_button.clone(),
                pane.size_down_button.clone(),
                pane.size_up_button.clone(),
                pane.enabled_toggle_button.clone(),
            )
        };

        let left_pane_for_buttons = left_pane.clone();
        page_prev_button.clone().on_click(cx, {
            let left_pane = left_pane_for_buttons.downgrade();
            move |_, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };

                left_pane.update(cx, |pane, cx| pane.step_page(-1, cx));
            }
        });
        page_next_button.clone().on_click(cx, {
            let left_pane = left_pane_for_buttons.downgrade();
            move |_, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };

                left_pane.update(cx, |pane, cx| pane.step_page(1, cx));
            }
        });
        total_down_button.clone().on_click(cx, {
            let left_pane = left_pane_for_buttons.downgrade();
            move |_, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };

                left_pane.update(cx, |pane, cx| pane.step_total_items(-1, cx));
            }
        });
        total_up_button.clone().on_click(cx, {
            let left_pane = left_pane_for_buttons.downgrade();
            move |_, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };

                left_pane.update(cx, |pane, cx| pane.step_total_items(1, cx));
            }
        });
        size_down_button.clone().on_click(cx, {
            let left_pane = left_pane_for_buttons.downgrade();
            move |_, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };

                left_pane.update(cx, |pane, cx| pane.step_page_size(-1, cx));
            }
        });
        size_up_button.clone().on_click(cx, {
            let left_pane = left_pane_for_buttons.downgrade();
            move |_, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };

                left_pane.update(cx, |pane, cx| pane.step_page_size(1, cx));
            }
        });
        enabled_toggle_button.clone().on_click(cx, {
            let left_pane = left_pane_for_buttons;
            move |_, cx| {
                left_pane.update(cx, |pane, cx| pane.toggle_enabled(cx));
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

impl Render for PagerControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-pager-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn render_sample(
    label: &'static str,
    pager: Pager,
    muted_text: gpui::Hsla,
    label_style: xgpui_luma::theme::LumaTextStyle,
) -> impl IntoElement {
    vstack! {
        gap=8.0;
        div()
            .typography_style(label_style)
            .font_weight(FontWeight::MEDIUM)
            .text_color(muted_text)
            .child(label),
        pager,
    }
    .w_full()
}

fn format_pager_event(event: &PagerEvent) -> Option<String> {
    match event {
        PagerEvent::PageChanged { page } => Some(format!("PagerEvent::PageChanged {{ page: {page} }}")),
        PagerEvent::PageSizeChanged { page_size } => {
            Some(format!("PagerEvent::PageSizeChanged {{ page_size: {page_size} }}"))
        }
        PagerEvent::PageSizeOpenChanged { open } => Some(format!("PagerEvent::PageSizeOpenChanged {{ open: {open} }}")),
        PagerEvent::EnabledChanged { enabled } => Some(format!("PagerEvent::EnabledChanged {{ enabled: {enabled} }}")),
        _ => None,
    }
}

fn page_count_for(total_items: usize, page_size: usize) -> usize {
    let page_size = page_size.max(1);
    total_items.div_ceil(page_size).max(1)
}

fn clamp_page(page: usize, page_count: usize) -> usize {
    if page_count == 0 { 0 } else { page.min(page_count - 1) }
}

fn offset_usize(value: usize, delta: isize) -> usize {
    if delta < 0 {
        value.saturating_sub(delta.unsigned_abs())
    } else {
        value.saturating_add(delta as usize)
    }
}
