use xgpui_luma::controls::pager::PagerEventExt;
use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::button::Button;
use xgpui_luma::controls::pager::{Pager, PagerEvent, PagerStyle};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma::{vstack, wrappanel};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook, ShadcnTextSize};

use crate::studio::style::shared::shell::section_shell_with_width;

const PAGE_SIZES: [usize; 3] = [10, 25, 50];

pub(crate) struct PagerPreview {
    look: Arc<ShadcnLook>,
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
    current_page: usize,
    total_items: usize,
    page_size: usize,
    enabled: bool,
    _subscriptions: Vec<Subscription>,
}

impl PagerPreview {
    pub(crate) fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let current_page = 2;
        let total_items = 281;
        let page_size = 25;
        let enabled = true;
        let page_count = page_count_for(total_items, page_size);

        let minimal = shadcn::Pager::new("luma-studio-pager-minimal")
            .look(look.as_ref())
            .style(PagerStyle::Minimal)
            .page_size(page_size)
            .page_size_options(PAGE_SIZES)
            .page_count(page_count)
            .current_page(current_page)
            .page_indicator_formatter(|current, total| SharedString::from(format!("{current} / {total}")))
            .spawn(cx);
        let minimal_edge = shadcn::Pager::new("luma-studio-pager-minimal-edge")
            .look(look.as_ref())
            .style(PagerStyle::MinimalEdge)
            .page_size(page_size)
            .page_size_options(PAGE_SIZES)
            .page_count(page_count)
            .current_page(current_page)
            .page_indicator_formatter(|current, total| SharedString::from(format!("{current} of {total}")))
            .spawn(cx);
        let numeric = shadcn::Pager::new("luma-studio-pager-numeric")
            .look(look.as_ref())
            .style(PagerStyle::Numeric)
            .page_size(page_size)
            .page_size_options(PAGE_SIZES)
            .page_count(page_count)
            .current_page(current_page)
            .spawn(cx);
        let numeric_no_edges = shadcn::Pager::new("luma-studio-pager-numeric-no-edges")
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
        let numeric_compact = shadcn::Pager::new("luma-studio-pager-numeric-compact")
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
        let page_prev_button = shadcn::Button::new("luma-studio-pager-page-prev")
            .look(look.as_ref())
            .secondary()
            .label("Page -")
            .spawn(cx);
        let page_next_button = shadcn::Button::new("luma-studio-pager-page-next")
            .look(look.as_ref())
            .secondary()
            .label("Page +")
            .spawn(cx);
        let total_down_button = shadcn::Button::new("luma-studio-pager-total-down")
            .look(look.as_ref())
            .secondary()
            .label("Items -")
            .spawn(cx);
        let total_up_button = shadcn::Button::new("luma-studio-pager-total-up")
            .look(look.as_ref())
            .secondary()
            .label("Items +")
            .spawn(cx);
        let size_down_button = shadcn::Button::new("luma-studio-pager-size-down")
            .look(look.as_ref())
            .secondary()
            .label("Per page -")
            .spawn(cx);
        let size_up_button = shadcn::Button::new("luma-studio-pager-size-up")
            .look(look.as_ref())
            .secondary()
            .label("Per page +")
            .spawn(cx);
        let enabled_toggle_button = shadcn::Button::new("luma-studio-pager-enabled-toggle")
            .look(look.as_ref())
            .outline()
            .label("Toggle enabled")
            .spawn(cx);

        let mut preview = Self {
            look,
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
            current_page,
            total_items,
            page_size,
            enabled,
            _subscriptions: Vec::new(),
        };
        preview.subscribe(cx);
        preview
    }

    pub(crate) fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        let pager_theme = look.pager_theme();
        for pager in self.pagers() {
            let pager_theme = pager_theme.clone();
            pager.update(cx, move |pager, cx| {
                pager.set_theme(pager_theme, cx);
            });
        }

        for button in self.buttons() {
            let template = look.button_template(ShadcnButtonStyle::Secondary);
            button.update(cx, move |button, cx| {
                button.set_template(template, cx);
            });
        }
        self.enabled_toggle_button.update(cx, {
            let template = look.button_template(ShadcnButtonStyle::Outline);
            move |button, cx| button.set_template(template, cx)
        });
        cx.notify();
    }

    fn subscribe(&mut self, cx: &mut Context<Self>) {
        for pager in self.pagers() {
            pager.clone().on_event(cx, |preview, event: &PagerEvent, cx| {
                preview.handle_pager_event(event, cx);
            });
        }

        self.page_prev_button.clone().on_click(cx, |preview, cx| {
            preview.step_page(-1, cx);
        });
        self.page_next_button.clone().on_click(cx, |preview, cx| {
            preview.step_page(1, cx);
        });
        self.total_down_button.clone().on_click(cx, |preview, cx| {
            preview.step_total_items(-1, cx);
        });
        self.total_up_button.clone().on_click(cx, |preview, cx| {
            preview.step_total_items(1, cx);
        });
        self.size_down_button.clone().on_click(cx, |preview, cx| {
            preview.step_page_size(-1, cx);
        });
        self.size_up_button.clone().on_click(cx, |preview, cx| {
            preview.step_page_size(1, cx);
        });
        self.enabled_toggle_button.clone().on_click(cx, |preview, cx| {
            preview.toggle_enabled(cx);
        });
    }

    fn pagers(&self) -> [Pager; 5] {
        [
            self.minimal.clone(),
            self.minimal_edge.clone(),
            self.numeric.clone(),
            self.numeric_no_edges.clone(),
            self.numeric_compact.clone(),
        ]
    }

    fn buttons(&self) -> [Entity<Button>; 6] {
        [
            self.page_prev_button.clone(),
            self.page_next_button.clone(),
            self.total_down_button.clone(),
            self.total_up_button.clone(),
            self.size_down_button.clone(),
            self.size_up_button.clone(),
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
}

impl Render for PagerPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let detail_style = self.look.typography_scale(ShadcnTextSize::Sm);
        let label_style = self.look.typography_scale(ShadcnTextSize::Xs);

        vstack! {
            gap=10.0;
            vstack! {
                gap=8.0;
                div().typography_style(detail_style).text_color(chrome.body_text).child(
                    format!(
                        "Current page: {} | Page count: {} | Total items: {} | Per page: {} | Enabled: {}",
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
            render_sample("Minimal | formatter: \"X / Y\"", self.minimal.clone(), chrome.muted_text, label_style),
            render_sample(
                "Minimal + first/last | formatter: \"X of Y\"",
                self.minimal_edge.clone(),
                chrome.muted_text,
                label_style,
            ),
            render_sample("Numeric", self.numeric.clone(), chrome.muted_text, label_style),
            render_sample("Numeric | no first/last", self.numeric_no_edges.clone(), chrome.muted_text, label_style),
            render_sample("Numeric | 5 fixed slots", self.numeric_compact.clone(), chrome.muted_text, label_style),
        }
        .w_full()
    }
}

pub(crate) fn render_pager_template_section(look: Arc<ShadcnLook>, preview: Entity<PagerPreview>) -> AnyElement {
    let chrome = look.chrome();

    section_shell_with_width(
        960.0,
        "Pager",
        "Standalone pager control preview with Minimal, Minimal + first/last, and Numeric styles.",
        chrome.title_text,
        chrome.muted_text,
        chrome.border,
        chrome.panel_background,
        preview.into_any_element(),
    )
}

fn render_sample(
    label: &'static str,
    pager: Pager,
    muted_text: gpui::Hsla,
    label_style: xgpui_luma::theme::LumaTextStyle,
) -> AnyElement {
    div()
        .w_full()
        .min_h(px(42.0))
        .flex()
        .items_center()
        .gap(px(18.0))
        .child(
            div()
                .w(px(210.0))
                .flex_shrink_0()
                .typography_style(label_style)
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(muted_text)
                .child(label),
        )
        .child(div().flex_1().min_w(px(0.0)).flex().items_center().justify_end().child(pager))
        .into_any_element()
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
