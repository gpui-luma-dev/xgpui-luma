use xgpui_luma::controls::selector::SelectorEventExt;
use xgpui_luma::controls::toggle::ToggleEventExt;
use xgpui_luma::infra::attachments::TooltipEntityExt;
use std::sync::Arc;

use gpui::{Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::controls::selector::{Selector, SelectorEvent, SelectorItem, SelectorPlacement, SelectorOpeningMode};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma::controls::toggle::{Toggle};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::input_theme_inspectors::SelectorThemeInspector;
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::inspector::{SelectorInspectorAdapter, SELECTOR_INSPECTOR_SPEC, selector_motion_report};
use super::template::render_control_exposition_card;

pub struct PopupSelectorControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<SelectorExpositionLeftPane>,
    selection_required: bool,
    has_selection: bool,
    theme_inspector: Entity<SelectorThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

struct SelectorExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    previews: Vec<(&'static str, Entity<Selector>)>,
    event_stream: Entity<ControlEventStream>,
    selection_required_toggle: Toggle,
}

impl SelectorExpositionLeftPane {
    fn dismiss_overlays(&mut self, cx: &mut Context<Self>) {
        for (_, preview) in &self.previews {
            preview.update(cx, |preview, cx| preview.dismiss(cx));
        }
    }

    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        for (_, preview) in &self.previews {
            preview.update(cx, |_, cx| cx.notify());
        }
        self.selection_required_toggle.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for SelectorExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let preview = div()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(16.0))
                .child(div().flex().flex_wrap().items_center().justify_center().gap(px(12.0)).children(
                    self.previews.iter().map(|(label, preview)| {
                        div()
                            .w(px(220.0))
                            .flex()
                            .flex_col()
                            .gap(px(8.0))
                            .child(div().text_color(chrome.muted_text).child(*label))
                            .child(preview.clone())
                    }),
                ))
                .child(self.selection_required_toggle.clone())
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-selector-left-pane")
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

impl PopupSelectorControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("popup-selector").expect("popup-selector catalog entry");
        let preview_below = shadcn::Selector::new("controls-doc-selector-below")
            .look(look.as_ref())
            .label("Below selector")
            .items(selector_items())
            .placement(SelectorPlacement::BelowStart)
            .invalid(false)
            .spawn(cx)
            .help("Choose an item", cx);
        let preview_smart = shadcn::Selector::new("controls-doc-selector-smart")
            .look(look.as_ref())
            .label("Smart selector")
            .items(selector_items())
            .placement(SelectorPlacement::Smart)
            .invalid(false)
            .spawn(cx);
        let preview_centered = shadcn::Selector::new("controls-doc-selector-centered")
            .look(look.as_ref())
            .label("Choose an action")
            .items(selector_items())
            .selected_id("archive")
            .opening_mode(SelectorOpeningMode::Centered)
            .spawn(cx);
        let preview_long = shadcn::Selector::new("controls-doc-selector-centered-long")
            .look(look.as_ref())
            .label("Choose an item")
            .items((0..60).map(|index| {
                SelectorItem::new(format!("item-{index}")).label(format!("Item {}", index + 1)).enabled(index != 2)
            }))
            .selected_id("item-30")
            .opening_mode(SelectorOpeningMode::Centered)
            .spawn(cx);
        let preview_unselected = shadcn::Selector::new("controls-doc-selector-centered-unselected")
            .look(look.as_ref())
            .label("Choose an action")
            .items(selector_items())
            .opening_mode(SelectorOpeningMode::Centered)
            .spawn(cx);
        let preview_empty = shadcn::Selector::new("controls-doc-selector-centered-empty")
            .look(look.as_ref())
            .label("No items")
            .opening_mode(SelectorOpeningMode::Centered)
            .spawn(cx);
        let previews = vec![
            ("Dropdown · below", preview_below),
            ("Dropdown · smart", preview_smart),
            ("Centered · selected", preview_centered),
            ("Centered · long list", preview_long),
            ("Centered · no selection", preview_unselected),
            ("Centered · empty", preview_empty),
        ];
        let selection_required_toggle = shadcn::Toggle::new("controls-doc-selector-selection-required")
            .look(look.as_ref())
            .outline()
            .with_data(false)
            .content(|_, _| div().child("Selection Required").into_any_element())
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-popup-selector-event-log",
                "Open a selector and pick items; SelectorEvent variants appear below.",
            )
        });
        let left_pane = cx.new(|_| SelectorExpositionLeftPane {
            look: look.clone(),
            entry,
            previews: previews.clone(),
            event_stream: event_stream.clone(),
            selection_required_toggle: selection_required_toggle.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-selector-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &SELECTOR_INSPECTOR_SPEC,
            SelectorInspectorAdapter::shared(),
        );
        theme_inspector.update(cx, |inspector, cx| {
            inspector.set_motion_report(selector_motion_report(previews.clone()), cx);
        });

        for (_, preview) in &previews {
            let event_stream = event_stream.clone();
            preview.clone().on_event(cx, move |this, event: &SelectorEvent, cx| {
                let line = format_selector_event(event);
                event_stream.update(cx, |stream, cx| {
                    stream.append_line(&line, cx);
                    cx.notify();
                });
                if matches!(event, SelectorEvent::Change { .. }) {
                    this.has_selection = true;
                }
                this.sync_required_validation(cx);
            });
        }
        selection_required_toggle.clone().on_value_changed(cx, |this, selected, cx| {
            this.selection_required = selected;
            this.sync_required_validation(cx);
        });

        Self {
            look,
            entry,
            left_pane,
            selection_required: false,
            has_selection: false,
            theme_inspector,
            inspector_split,
        }
    }

    fn sync_required_validation(&mut self, cx: &mut Context<Self>) {
        let invalid = self.selection_required && !self.has_selection;
        self.left_pane.update(cx, |pane, cx| {
            for (_, preview) in &pane.previews {
                preview.update(cx, |preview, cx| preview.set_invalid(invalid, cx));
            }
        });
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn fills_viewport(&self) -> bool {
        true
    }

    pub fn dismiss_overlays(&mut self, cx: &mut Context<Self>) {
        self.left_pane.update(cx, |pane, cx| pane.dismiss_overlays(cx));
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

impl Render for PopupSelectorControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-selector-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn selector_items() -> [SelectorItem; 4] {
    [
        SelectorItem::new("new").label("New").icon(LucideIcon::FilePlus),
        SelectorItem::new("open").label("Open").icon(LucideIcon::FolderOpen),
        SelectorItem::new("archive").label("Archive").icon(LucideIcon::Archive),
        SelectorItem::new("export").label("Export").icon(LucideIcon::Share2),
    ]
}

fn format_selector_event(event: &SelectorEvent) -> String {
    match event {
        SelectorEvent::Change { item_id, label } => {
            format!("SelectorEvent::Change {{ item_id: \"{item_id}\", label: \"{label}\" }}")
        }
        SelectorEvent::OpenChanged { open } => format!("SelectorEvent::OpenChanged {{ open: {open} }}"),
        SelectorEvent::Dismiss => "SelectorEvent::Dismiss".to_string(),
        SelectorEvent::FocusChanged { focused } => format!("SelectorEvent::FocusChanged {{ focused: {focused} }}"),
        _ => "SelectorEvent::(unknown)".to_string(),
    }
}
