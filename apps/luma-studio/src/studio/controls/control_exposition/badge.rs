//! Badge look exposition — gallery-aligned variants, sizes, and icons (non-interactive).

use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::{flow, hstack};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn::{ShadcnLook, ShadcnTextSize};
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::inspector::{BadgeInspectorAdapter, BADGE_INSPECTOR_SPEC};
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::standalone_theme_inspectors::BadgeThemeInspector;
use super::template::render_control_exposition_card;

pub struct BadgeControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<BadgeExpositionLeftPane>,
    theme_inspector: Entity<BadgeThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

struct BadgeExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
}

impl BadgeExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        cx.notify();
    }
}

impl Render for BadgeExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let look = &self.look;
            let chrome = look.chrome();
            let title_color = chrome.muted_text;

            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(20.0))
                .child(demo_section(
                    look,
                    "Reference",
                    title_color,
                    hstack! {
                        gap=10.0 align=center;
                        look.badge("work").variant(BadgeVariant::Default),
                        look.badge("budget").variant(BadgeVariant::Outline),
                    },
                ))
                .child(demo_section(
                    look,
                    "Variants",
                    title_color,
                    flow! {
                        gap=10.0;
                        look.badge("Default").variant(BadgeVariant::Default),
                        look.badge("Secondary").variant(BadgeVariant::Secondary),
                        look.badge("Outline").variant(BadgeVariant::Outline),
                        look.badge("Ghost").variant(BadgeVariant::Ghost),
                    }
                    .justify_center()
                    .items_center(),
                ))
                .child(demo_section(
                    look,
                    "Sizes",
                    title_color,
                    hstack! {
                        gap=10.0 align=center justify=center;
                        look.badge("Small").size(ShadcnSize::Sm).variant(BadgeVariant::Secondary),
                        look.badge("Medium").size(ShadcnSize::Md).variant(BadgeVariant::Secondary),
                        look.badge("Large").size(ShadcnSize::Lg).variant(BadgeVariant::Secondary),
                    },
                ))
                .child(demo_section(
                    look,
                    "Icons",
                    title_color,
                    flow! {
                        gap=10.0;
                        look.badge("Verified").variant(BadgeVariant::Default).start_icon(LucideIcon::BadgeCheck),
                        look.badge("Updated").variant(BadgeVariant::Secondary).end_icon(LucideIcon::ArrowUpRight),
                        look.badge("Draft").variant(BadgeVariant::Outline).start_icon(LucideIcon::Pencil),
                        look.badge("Muted").variant(BadgeVariant::Ghost).end_icon(LucideIcon::Dot),
                    }
                    .justify_center()
                    .items_center(),
                ));

            div()
                .id("controls-doc-badge-left-pane")
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

impl BadgeControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("badge").expect("badge catalog entry");
        let left_pane = cx.new(|_| BadgeExpositionLeftPane { look: look.clone(), entry });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-badge-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &BADGE_INSPECTOR_SPEC,
            BadgeInspectorAdapter::shared(),
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

impl Render for BadgeControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-badge-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn demo_section(
    look: &ShadcnLook,
    title: &'static str,
    title_color: gpui::Hsla,
    content: impl IntoElement,
) -> impl IntoElement {
    let section_title = look.typography_scale(ShadcnTextSize::Sm);

    div()
        .w_full()
        .max_w(px(720.0))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(4.0))
        .child(div().typography_style(section_title).text_color(title_color).child(title))
        .child(
            div()
                .w_full()
                .flex()
                .justify_center()
                .rounded(px(16.0))
                .bg(look.color(ShadcnToken::Card))
                .border_1()
                .border_color(look.color(ShadcnToken::Border))
                .p(px(18.0))
                .child(content),
        )
}
