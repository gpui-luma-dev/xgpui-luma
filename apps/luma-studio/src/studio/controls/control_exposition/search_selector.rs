use xgpui_luma::controls::search_selector::SearchSelectorEventExt;
use xgpui_luma::controls::toggle::ToggleEventExt;
use xgpui_luma::infra::attachments::TooltipEntityExt;
use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::search_selector::{SearchSelector, SearchSelectorEvent, SelectionItem};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma::controls::toggle::{Toggle};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::input_theme_inspectors::SearchSelectorThemeInspector;
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::inspector::{search_selector_inspector_adapter, SEARCH_SELECTOR_INSPECTOR_SPEC, search_selector_motion_report};
use super::template::render_control_exposition_card;

pub struct SearchSelectorControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    preview: SearchSelector,
    event_stream: Entity<ControlEventStream>,
    left_pane: Entity<SearchSelectorExpositionLeftPane>,
    selection_required: bool,
    has_selection: bool,
    theme_inspector: Entity<SearchSelectorThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct SearchSelectorExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    preview: SearchSelector,
    event_stream: Entity<ControlEventStream>,
    selection_required_toggle: Toggle,
}

impl SearchSelectorExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.preview.update(cx, |_, cx| cx.notify());
        self.selection_required_toggle.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for SearchSelectorExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let preview = div()
                .w_full()
                .max_w(px(760.0))
                .flex()
                .flex_col()
                .items_start()
                .gap(px(16.0))
                .child(
                    div()
                        .w(px(320.0))
                        .flex_none()
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        .child(self.selection_required_toggle.clone())
                        .child(self.preview.clone()),
                )
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-search-selector-left-pane")
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

impl SearchSelectorControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("search-selector").expect("search-selector catalog entry");
        let preview = shadcn::SearchSelector::new("controls-doc-search-selector", search_selector_demo_items())
            .look(look.as_ref())
            .placeholder("Choose a state…")
            .search_placeholder("Search states")
            .full_width(true)
            .clean_on_escape(true)
            .invalid(false)
            .spawn(cx)
            .help("Choose a state or search the available states", cx);
        let selection_required_toggle = shadcn::Toggle::new("controls-doc-search-selector-selection-required")
            .look(look.as_ref())
            .outline()
            .with_data(false)
            .content(|_, _| div().child("Selection Required").into_any_element())
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-search-selector-event-log",
                "Open the selector and search; SearchSelectorEvent variants appear below.",
            )
        });
        let left_pane = cx.new(|_| SearchSelectorExpositionLeftPane {
            look: look.clone(),
            entry,
            preview: preview.clone(),
            event_stream: event_stream.clone(),
            selection_required_toggle: selection_required_toggle.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-search-selector-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &SEARCH_SELECTOR_INSPECTOR_SPEC,
            search_selector_inspector_adapter(),
        );
        theme_inspector.update(cx, |inspector, cx| {
            inspector.set_motion_report(search_selector_motion_report([("States", preview.clone())]), cx);
        });

        preview.clone().on_event(cx, {
            let event_stream = event_stream.downgrade();
            move |this, event: &SearchSelectorEvent, cx| {
                let Some(event_stream) = event_stream.upgrade() else {
                    return;
                };
                let line = format_search_selector_event(event);
                event_stream.update(cx, |stream, cx| {
                    stream.append_line(&line, cx);
                });
                match event {
                    SearchSelectorEvent::Select { .. } | SearchSelectorEvent::Complete { .. } => {
                        this.has_selection = true;
                    }
                    SearchSelectorEvent::Clear => this.has_selection = false,
                    _ => {}
                }
                this.sync_required_validation(cx);
            }
        });
        selection_required_toggle.clone().on_value_changed(cx, |this, selected, cx| {
            this.selection_required = selected;
            this.sync_required_validation(cx);
        });

        Self {
            look,
            entry,
            preview,
            event_stream,
            left_pane,
            selection_required: false,
            has_selection: false,
            theme_inspector,
            inspector_split,
            _subscriptions: { Vec::new() },
        }
    }

    fn sync_required_validation(&mut self, cx: &mut Context<Self>) {
        let invalid = self.selection_required && !self.has_selection;
        self.preview.update(cx, |preview, cx| preview.set_invalid(invalid, cx));
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
        self.preview.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look.clone(), cx));
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        sync_viewport_inspector(look, &self.theme_inspector, &self.inspector_split, cx);
        cx.notify();
    }
}

impl Render for SearchSelectorControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-search-selector-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn search_selector_demo_items() -> Vec<SelectionItem> {
    vec![
        SelectionItem::new("alabama", "Alabama"),
        SelectionItem::new("alaska", "Alaska"),
        SelectionItem::new("arizona", "Arizona"),
        SelectionItem::new("arkansas", "Arkansas"),
        SelectionItem::new("california", "California"),
        SelectionItem::new("colorado", "Colorado"),
        SelectionItem::new("connecticut", "Connecticut"),
        SelectionItem::new("delaware", "Delaware"),
        SelectionItem::new("florida", "Florida"),
        SelectionItem::new("georgia", "Georgia"),
    ]
}

fn format_search_selector_event(event: &SearchSelectorEvent) -> String {
    match event {
        SearchSelectorEvent::Change { query } => format!("SearchSelectorEvent::Change {{ query: \"{query}\" }}"),
        SearchSelectorEvent::Select { item_id, label } => {
            format!("SearchSelectorEvent::Select {{ item_id: \"{item_id}\", label: \"{label}\" }}")
        }
        SearchSelectorEvent::Complete { item_id, label } => {
            format!("SearchSelectorEvent::Complete {{ item_id: \"{item_id}\", label: \"{label}\" }}")
        }
        SearchSelectorEvent::Clear => "SearchSelectorEvent::Clear".to_string(),
        SearchSelectorEvent::OpenChanged { open } => format!("SearchSelectorEvent::OpenChanged {{ open: {open} }}"),
        SearchSelectorEvent::Dismiss => "SearchSelectorEvent::Dismiss".to_string(),
        SearchSelectorEvent::FocusChanged { focused } => {
            format!("SearchSelectorEvent::FocusChanged {{ focused: {focused} }}")
        }
        _ => "SearchSelectorEvent::(unknown)".to_string(),
    }
}
