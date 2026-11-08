//! Toolbar control exposition — gallery-aligned editing toolbar with focus strategy toggles.

use xgpui_luma::controls::toolbar::ToolbarEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::button::ButtonEventExt;
use xgpui_luma::controls::button::{Button, HasPresenter};
use xgpui_luma::controls::control_group::ControlGroupFocusStrategy;
use xgpui_luma::infra::menu_item::MenuItem;
use xgpui_luma::controls::selector::SelectorItem;
use xgpui_luma::controls::toolbar::{Toolbar, ToolbarEvent, ToolbarValue};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::shell_theme_inspectors::ToolbarThemeInspector;
use super::template::render_control_exposition_card;
use super::inspector::{
    ControlGroupMotionReport, ToolbarInspectorAdapter, TOOLBAR_INSPECTOR_SPEC, control_group_motion_source,
};

pub struct ToolbarControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<ToolbarExpositionLeftPane>,
    theme_inspector: Entity<ToolbarThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

struct ToolbarExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    toolbar: Toolbar,
    roving_focus_button: Entity<Button<()>>,
    sequential_focus_button: Entity<Button<()>>,
    event_stream: Entity<ControlEventStream>,
    focus_mode: &'static str,
}

impl ToolbarExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.toolbar.update(cx, |toolbar, cx| toolbar.notify_items(cx));
        self.roving_focus_button.update(cx, |_, cx| cx.notify());
        self.sequential_focus_button.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for ToolbarExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(16.0))
                .child(self.toolbar.clone())
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .justify_center()
                        .gap(px(8.0))
                        .child(self.roving_focus_button.clone())
                        .child(self.sequential_focus_button.clone())
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(16.0))
                                .text_color(chrome.muted_text)
                                .child(format!("mode: {}", self.focus_mode)),
                        ),
                )
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-toolbar-left-pane")
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

impl ToolbarControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("toolbar").expect("toolbar catalog entry");

        let roving_focus_button = shadcn::Button::new("controls-doc-toolbar-focus-roving")
            .look(look.as_ref())
            .ghost()
            .label("Roving focus")
            .spawn(cx);
        let sequential_focus_button = shadcn::Button::new("controls-doc-toolbar-focus-sequential")
            .look(look.as_ref())
            .ghost()
            .label("Sequential focus")
            .spawn(cx);

        let toolbar = shadcn::Toolbar::new("controls-doc-toolbar")
            .look(look.as_ref())
            .roving_item_focus()
            .item(look.toolbar_menu(
                "assistant",
                "Assistant",
                LucideIcon::WandSparkles,
                [
                    MenuItem::new("rewrite").label("Rewrite"),
                    MenuItem::new("summarize").label("Summarize"),
                    MenuItem::new("continue").label("Continue writing"),
                ],
                cx,
            ))
            .separator("assistant-separator")
            .item(look.toolbar_selector(
                "style",
                "Paragraph",
                [
                    SelectorItem::new("paragraph").label("Paragraph").icon(LucideIcon::Pilcrow),
                    SelectorItem::new("heading").label("Heading").icon(LucideIcon::TypeIcon),
                    SelectorItem::new("quote").label("Quote").icon(LucideIcon::BookType),
                ],
                "paragraph",
                cx,
            ))
            .separator("style-separator")
            .item(look.toolbar_toggle("bold", LucideIcon::Bold, cx))
            .item(look.toolbar_toggle("italic", LucideIcon::Italic, cx))
            .item(look.toolbar_toggle("underline", LucideIcon::Underline, cx))
            .item(look.toolbar_label_menu(
                "color",
                "A",
                [
                    MenuItem::new("default").label("Default color"),
                    MenuItem::new("muted").label("Muted"),
                    MenuItem::new("accent").label("Accent"),
                ],
                cx,
            ))
            .separator("format-separator")
            .item(look.toolbar_menu(
                "alignment",
                "Alignment",
                LucideIcon::TextAlignStart,
                [
                    MenuItem::new("left").label("Align left").icon(LucideIcon::TextAlignStart),
                    MenuItem::new("center").label("Align center").icon(LucideIcon::TextAlignCenter),
                    MenuItem::new("right").label("Align right").icon(LucideIcon::TextAlignEnd),
                    MenuItem::new("justify").label("Justify").icon(LucideIcon::TextAlignJustify),
                ],
                cx,
            ))
            .separator("insert-separator")
            .item(look.toolbar_button("link", LucideIcon::Link, cx))
            .item(look.toolbar_button("image", LucideIcon::Image, cx))
            .item(look.toolbar_button("video", LucideIcon::Video, cx))
            .item(look.toolbar_menu(
                "more",
                "More",
                LucideIcon::Ellipsis,
                [
                    MenuItem::new("clear").label("Clear formatting"),
                    MenuItem::new("divider").label("Insert divider"),
                    MenuItem::new("comment").label("Add comment"),
                ],
                cx,
            ))
            .separator("code-separator")
            .item(look.toolbar_textfield("search").placeholder("Search...").spawn(cx).on_change(|_value, _cx| {}))
            .item(look.toolbar_button("code", LucideIcon::Code, cx))
            .spawn(cx);

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-toolbar-event-log",
                "Interact with toolbar items; ToolbarEvent variants appear below.",
            )
        });

        let left_pane = cx.new(|_| ToolbarExpositionLeftPane {
            look: look.clone(),
            entry,
            toolbar: toolbar.clone(),
            roving_focus_button: roving_focus_button.clone(),
            sequential_focus_button: sequential_focus_button.clone(),
            event_stream: event_stream.clone(),
            focus_mode: "roving",
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-toolbar-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &TOOLBAR_INSPECTOR_SPEC,
            ToolbarInspectorAdapter::shared(),
        );
        theme_inspector.update(cx, |inspector, cx| {
            inspector.set_motion_report(
                Arc::new(ControlGroupMotionReport::new([(
                    "Toolbar",
                    control_group_motion_source(toolbar.read(cx).inner_group()),
                )])),
                cx,
            );
        });

        toolbar.clone().on_event(cx, {
            let event_stream = event_stream.downgrade();
            move |_, event: &ToolbarEvent, cx| {
                let Some(event_stream) = event_stream.upgrade() else {
                    return;
                };
                if let Some(line) = format_toolbar_event(event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }
        });
        roving_focus_button.clone().on_click(cx, {
            let left_pane = left_pane.downgrade();
            move |_, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };

                left_pane.update(cx, |pane, cx| {
                    pane.toolbar.update(cx, |toolbar, cx| {
                        toolbar.set_focus_strategy(ControlGroupFocusStrategy::RovingItemFocus, cx);
                    });
                    pane.focus_mode = "roving";
                    cx.notify();
                });
            }
        });
        sequential_focus_button.clone().on_click(cx, {
            let left_pane = left_pane.downgrade();
            move |_, cx| {
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };

                left_pane.update(cx, |pane, cx| {
                    pane.toolbar.update(cx, |toolbar, cx| {
                        toolbar.set_focus_strategy(ControlGroupFocusStrategy::ActiveDescendant, cx);
                    });
                    pane.focus_mode = "sequential";
                    cx.notify();
                });
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

impl Render for ToolbarControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-toolbar-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn format_toolbar_event(event: &ToolbarEvent) -> Option<String> {
    match event {
        ToolbarEvent::Click { id } => Some(format!("ToolbarEvent::Click {{ id: \"{id}\" }}")),
        ToolbarEvent::Change { id, value } => {
            let payload = match value {
                ToolbarValue::Bool(on) => format!("bool({on})"),
                ToolbarValue::String(value) => format!("string(\"{value}\")"),
            };
            Some(format!("ToolbarEvent::Change {{ id: \"{id}\", value: {payload} }}"))
        }
        _ => None,
    }
}
