//! Supported SplitButton behavior and custom face content.

use xgpui_luma::controls::split_button::SplitButtonEventExt;
use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, FontWeight, Render, Subscription, Window, div, prelude::*, px};
use xgpui_luma::infra::menu_item::MenuItem;
use xgpui_luma::controls::popup_menu::PopupMenuTriggerModel;
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma::{hstack, vstack};
use xgpui_luma::controls::split_button::{SplitButton, SplitButtonEvent};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::{ShadcnLook};
use lucide_svg_static::Icon as LucideIcon;

use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::ControlExpositionLayout;
use super::inspector::{split_button_inspector_resolver, popup_menu_motion_report, SPLIT_BUTTON_INSPECTOR_SPEC};
use super::template::render_control_exposition_card;
use crate::studio::controls::catalog::{catalog_entry, ControlDocEntry};
use super::template::controls_mono_font;

struct SplitButtonPair {
    control: Entity<SplitButton>,
    label: &'static str,
}

pub struct SplitButtonControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    pairs: Vec<SplitButtonPair>,
    event_stream: Entity<ControlEventStream>,
    left_pane: Entity<SplitButtonLeftPane>,
    inspector_split: Entity<InspectorSplitShell>,
    theme_inspector: Entity<super::theme_inspector::ThemeInspector>,
    _subscriptions: Vec<Subscription>,
}

impl SplitButtonControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("split-button").expect("split-button catalog entry");
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-split-button-event-log",
                "Activate the action face or open the menu face; semantic events appear below.",
            )
        });
        let variants = [
            ("Primary", shadcn::SplitButton::new("controls-split-button-primary").look(look.as_ref()).primary()),
            (
                "Secondary",
                shadcn::SplitButton::new("controls-split-button-secondary").look(look.as_ref()).secondary(),
            ),
            ("Outline", shadcn::SplitButton::new("controls-split-button-outline").look(look.as_ref()).outline()),
            ("Ghost", shadcn::SplitButton::new("controls-split-button-ghost").look(look.as_ref()).ghost()),
            (
                "Primary unavailable",
                shadcn::SplitButton::new("controls-split-button-primary-disabled")
                    .look(look.as_ref())
                    .primary_enabled(false),
            ),
            (
                "Disabled",
                shadcn::SplitButton::new("controls-split-button-disabled").look(look.as_ref()).enabled(false),
            ),
        ];
        let mut pairs = Vec::new();
        let subscriptions = Vec::new();

        for (label, builder) in variants {
            let control = builder.label(label).items(split_button_items()).spawn(cx);

            let event_log = event_stream.clone();
            let variant_label = label.to_string();
            control.clone().on_event(cx, move |_, event: &SplitButtonEvent, cx| {
                let line = match event {
                    SplitButtonEvent::ActionClick => format!("{variant_label}: ActionClick"),
                    SplitButtonEvent::Select { item_id, label } => {
                        format!("{variant_label}: Select {{ item_id: \"{item_id}\", label: \"{label}\" }}")
                    }
                    SplitButtonEvent::OpenChanged { open } => {
                        format!("{variant_label}: OpenChanged {{ open: {open} }}")
                    }
                    SplitButtonEvent::Dismiss => format!("{variant_label}: Dismiss"),
                    _ => format!("{variant_label}: other SplitButtonEvent"),
                };
                event_log.update(cx, |stream, cx| stream.append_line(&line, cx));
            });

            pairs.push(SplitButtonPair { control, label });
        }
        let content_example = shadcn::SplitButton::new("controls-split-content-example")
            .look(look.as_ref())
            .primary()
            .label("Label")
            .content(|model: &PopupMenuTriggerModel, _| {
                hstack! {
                    gap=6 align=center;
                    xgpui_luma::infra::icon::lucide_icon(LucideIcon::Pencil, gpui::hsla(0.0, 0.0, 1.0, 1.0), 16.0),
                    div().child(model.label.clone()),
                }
            })
            .items(split_button_items())
            .spawn(cx);
        let content_for_motion = content_example.clone();
        let left_pane = cx.new(|_| SplitButtonLeftPane {
            look: look.clone(),
            entry,
            pairs: pairs
                .iter()
                .map(|pair| SplitButtonPair { control: pair.control.clone(), label: pair.label })
                .collect(),
            content_example,
            event_stream: event_stream.clone(),
        });
        let left_pane_for_inspector = left_pane.clone();
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-split-button-pane",
            move || left_pane_for_inspector.clone().into_any_element(),
            &SPLIT_BUTTON_INSPECTOR_SPEC,
            split_button_inspector_resolver(),
        );
        theme_inspector.update(cx, |inspector, cx| {
            let mut samples: Vec<(&str, _)> =
                pairs.iter().map(|pair| (pair.label, pair.control.read(cx).inner_popup())).collect();
            samples.push(("Icon + Text", content_for_motion.read(cx).inner_popup()));
            inspector.set_motion_report(popup_menu_motion_report(samples), cx);
        });

        Self {
            look,
            entry,
            pairs,
            event_stream,
            left_pane,
            inspector_split,
            theme_inspector,
            _subscriptions: subscriptions,
        }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }
    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for pair in &self.pairs {
            pair.control.update(cx, |_, cx| cx.notify());
        }
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look.clone(), cx));
        sync_viewport_inspector(look, &self.theme_inspector, &self.inspector_split, cx);
        cx.notify();
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
}

impl Render for SplitButtonControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-split-button-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

struct SplitButtonLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    pairs: Vec<SplitButtonPair>,
    content_example: Entity<SplitButton>,
    event_stream: Entity<ControlEventStream>,
}

impl SplitButtonLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        for pair in &self.pairs {
            pair.control.update(cx, |_, cx| cx.notify());
        }
        self.content_example.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(self.look.clone(), cx));
        cx.notify();
    }
}

impl Render for SplitButtonLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let preview = vstack! {
                gap=18.0;
                div().w_full().flex().flex_wrap().items_center().gap(px(16.0)).children(self.pairs.iter().map(|pair| {
                    div().id(format!("split-button-preview-{}", pair.label)).flex().child(pair.control.clone())
                })),
                self.event_stream.clone(),
            };
            let content_example = vstack! {
                gap=8.0;
                div().text_sm().text_color(self.look.chrome().muted_text).child("Icon + Text"),
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .py(px(12.0))
                    .child(self.content_example.clone()),
                render_split_button_content_code(&self.look),
            }
            .w_full()
            .px(px(16.0));
            div()
                .id("controls-doc-split-button-left-pane")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .overflow_y_scroll()
                .child(vstack! {
                    gap=18.0;
                    render_control_exposition_card(
                        &self.look,
                        self.entry,
                        preview.into_any_element(),
                        None,
                        ControlExpositionLayout::BORDERLESS,
                    ),
                    content_example,
                })
        })
    }
}

fn render_split_button_content_code(look: &ShadcnLook) -> AnyElement {
    let chrome = look.chrome();
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(chrome.title_text)
                .child("Custom construction"),
        )
        .child(
            div()
                .w_full()
                .text_size(px(12.0))
                .line_height(px(18.0))
                .font_family(controls_mono_font())
                .text_color(chrome.muted_text)
                .child(
                    "use xgpui_luma::infra::icon::lucide_icon;\nuse xgpui_luma::controls::popup_menu::PopupMenuTriggerModel;\nuse xgpui_luma::infra::presenter::HasPresenter;\n\nlet open = shadcn::SplitButton::new(\"open\").look(look.as_ref()).primary()\n    .label(\"Label\")\n    .content(|model: &PopupMenuTriggerModel, _| {\n        hstack! {\n            gap = 6.0;\n            align = center;\n            lucide_icon(LucideIcon::Pencil, gpui::hsla(0.0, 0.0, 1.0, 1.0), 16.0),\n            div().child(model.label.clone()),\n        }\n    })\n    .items(items)\n    .spawn(cx);",
                ),
        )
        .into_any_element()
}

fn split_button_items() -> [MenuItem; 3] {
    [
        MenuItem::new("save-as").label("Save as").icon(LucideIcon::Save),
        MenuItem::new("duplicate").label("Duplicate").icon(LucideIcon::Copy),
        MenuItem::new("export").label("Export").icon(LucideIcon::Download),
    ]
}
