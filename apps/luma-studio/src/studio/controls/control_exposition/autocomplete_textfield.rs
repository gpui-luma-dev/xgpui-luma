use xgpui_luma::controls::autocomplete::AutocompleteEventExt;
use xgpui_luma::controls::toggle::ToggleEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::autocomplete::{Autocomplete, AutocompleteEvent, SelectionItem};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma::controls::toggle::{Toggle};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::inspector::{AutocompleteInspectorAdapter, AUTOCOMPLETE_INSPECTOR_SPEC, autocomplete_motion_report};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::input_theme_inspectors::AutocompleteThemeInspector;
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::template::render_control_exposition_card;

pub struct AutocompleteTextFieldControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    preview: Autocomplete,
    event_stream: Entity<ControlEventStream>,
    left_pane: Entity<AutocompleteExpositionLeftPane>,
    selection_required: bool,
    has_selection: bool,
    theme_inspector: Entity<AutocompleteThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct AutocompleteExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    preview: Autocomplete,
    event_stream: Entity<ControlEventStream>,
    selection_required_toggle: Toggle,
}

impl AutocompleteExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.preview.update(cx, |_, cx| cx.notify());
        self.selection_required_toggle.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for AutocompleteExpositionLeftPane {
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
                .id("controls-doc-autocomplete-left-pane")
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

impl AutocompleteTextFieldControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("autocomplete-textfield").expect("autocomplete-textfield catalog entry");
        let preview = shadcn::Autocomplete::new("controls-doc-autocomplete", autocomplete_demo_items())
            .look(look.as_ref())
            .placeholder("Start typing…")
            .full_width(true)
            .clean_on_escape(true)
            .invalid(false)
            .spawn(cx);
        let selection_required_toggle = shadcn::Toggle::new("controls-doc-autocomplete-selection-required")
            .look(look.as_ref())
            .outline()
            .with_data(false)
            .content(|_, _| div().child("Selection Required").into_any_element())
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-autocomplete-event-log",
                "Type in the field and pick items; AutocompleteEvent variants appear below.",
            )
        });
        let left_pane = cx.new(|_| AutocompleteExpositionLeftPane {
            look: look.clone(),
            entry,
            preview: preview.clone(),
            event_stream: event_stream.clone(),
            selection_required_toggle: selection_required_toggle.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-autocomplete-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &AUTOCOMPLETE_INSPECTOR_SPEC,
            AutocompleteInspectorAdapter::shared(),
        );
        theme_inspector.update(cx, |inspector, cx| {
            inspector.set_motion_report(autocomplete_motion_report([("Field", preview.clone())]), cx);
        });

        preview.clone().on_event(cx, {
            let event_stream = event_stream.downgrade();
            move |this, event: &AutocompleteEvent, cx| {
                let Some(event_stream) = event_stream.upgrade() else {
                    return;
                };
                let line = format_autocomplete_event(event);
                event_stream.update(cx, |stream, cx| {
                    stream.append_line(&line, cx);
                });
                match event {
                    AutocompleteEvent::Select { .. } | AutocompleteEvent::Complete { .. } => {
                        this.has_selection = true;
                    }
                    AutocompleteEvent::Clear => this.has_selection = false,
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

impl Render for AutocompleteTextFieldControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-autocomplete-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn autocomplete_demo_items() -> Vec<SelectionItem> {
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

fn format_autocomplete_event(event: &AutocompleteEvent) -> String {
    match event {
        AutocompleteEvent::Change { query } => {
            format!("AutocompleteEvent::Change {{ query: \"{query}\" }}")
        }
        AutocompleteEvent::Select { item_id, label } => {
            format!("AutocompleteEvent::Select {{ item_id: \"{item_id}\", label: \"{label}\" }}")
        }
        AutocompleteEvent::Complete { item_id, label } => {
            format!("AutocompleteEvent::Complete {{ item_id: \"{item_id}\", label: \"{label}\" }}")
        }
        AutocompleteEvent::Clear => "AutocompleteEvent::Clear".to_string(),
        AutocompleteEvent::OpenChanged { open } => {
            format!("AutocompleteEvent::OpenChanged {{ open: {open} }}")
        }
        AutocompleteEvent::Dismiss => "AutocompleteEvent::Dismiss".to_string(),
        AutocompleteEvent::FocusChanged { focused } => {
            format!("AutocompleteEvent::FocusChanged {{ focused: {focused} }}")
        }
        _ => "AutocompleteEvent::(unknown)".to_string(),
    }
}
