use xgpui_luma::controls::combobox::ComboBoxEventExt;
use xgpui_luma::controls::toggle::ToggleEventExt;
use xgpui_luma::infra::attachments::TooltipEntityExt;
use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::combobox::{ComboBox, ComboBoxEvent, SelectionItem, TypingPolicy};
use xgpui_luma::controls::toggle::{Toggle};
use xgpui_luma::infra::presenter::HasPresenter;
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::inspector::{combobox_inspector_adapter, COMBOBOX_INSPECTOR_SPEC, combobox_motion_report};
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::input_theme_inspectors::ComboBoxThemeInspector;
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout};
use super::template::render_control_exposition_card;

pub struct ComboBoxControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    preview: ComboBox,
    event_stream: Entity<ControlEventStream>,
    left_pane: Entity<ComboBoxExpositionLeftPane>,
    selection_required: bool,
    has_selection: bool,
    theme_inspector: Entity<ComboBoxThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct ComboBoxExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    preview: ComboBox,
    event_stream: Entity<ControlEventStream>,
    selection_required_toggle: Toggle,
}

impl ComboBoxExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.preview.update(cx, |_, cx| cx.notify());
        self.selection_required_toggle.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for ComboBoxExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
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
                        .gap(px(16.0))
                        .child(
                            div()
                                .text_size(px(11.0))
                                .line_height(px(15.0))
                                .text_color(chrome.muted_text)
                                .child("Strict typing policy + down arrow"),
                        )
                        .child(self.selection_required_toggle.clone())
                        .child(self.preview.clone()),
                )
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-combobox-left-pane")
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

impl ComboBoxControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("combobox").expect("combobox catalog entry");
        let preview = shadcn::ComboBox::new("controls-doc-combobox", combobox_demo_items())
            .look(look.as_ref())
            .placeholder("Strict mode (exact match only)…")
            .full_width(true)
            .clean_on_escape(true)
            .typing_policy(TypingPolicy::Strict)
            .show_down_arrow(true)
            .show_clear_button(true)
            .invalid(false)
            .spawn(cx)
            .help("Type to filter states, or open the list", cx);
        let selection_required_toggle = shadcn::Toggle::new("controls-doc-combobox-selection-required")
            .look(look.as_ref())
            .outline()
            .with_data(false)
            .content(|_, _| div().child("Selection Required").into_any_element())
            .spawn(cx);
        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-combobox-event-log",
                "Edit the combobox and pick items; ComboBoxEvent variants appear below.",
            )
        });

        let left_pane = cx.new(|_| ComboBoxExpositionLeftPane {
            look: look.clone(),
            entry,
            preview: preview.clone(),
            event_stream: event_stream.clone(),
            selection_required_toggle: selection_required_toggle.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-combobox-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &COMBOBOX_INSPECTOR_SPEC,
            combobox_inspector_adapter(),
        );
        theme_inspector.update(cx, |inspector, cx| {
            inspector.set_motion_report(combobox_motion_report([("Strict", preview.clone())]), cx);
        });

        preview.clone().on_event(cx, {
            let event_stream = event_stream.downgrade();
            move |this, event: &ComboBoxEvent, cx| {
                let Some(event_stream) = event_stream.upgrade() else {
                    return;
                };
                let line = format_combobox_event(event);
                event_stream.update(cx, |stream, cx| {
                    stream.append_line(&line, cx);
                });
                match event {
                    ComboBoxEvent::Select { .. } | ComboBoxEvent::Complete { .. } => {
                        this.has_selection = true;
                    }
                    ComboBoxEvent::Clear => {
                        this.has_selection = false;
                    }
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

impl Render for ComboBoxControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-combobox-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn combobox_demo_items() -> Vec<SelectionItem> {
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

pub(crate) fn format_combobox_event(event: &ComboBoxEvent) -> String {
    match event {
        ComboBoxEvent::Change { query } => format!("ComboBoxEvent::Change {{ query: \"{query}\" }}"),
        ComboBoxEvent::Select { item_id, label } => {
            format!("ComboBoxEvent::Select {{ item_id: \"{item_id}\", label: \"{label}\" }}")
        }
        ComboBoxEvent::Complete { item_id, label } => {
            format!("ComboBoxEvent::Complete {{ item_id: \"{item_id}\", label: \"{label}\" }}")
        }
        ComboBoxEvent::Clear => "ComboBoxEvent::Clear".to_string(),
        ComboBoxEvent::OpenChanged { open } => format!("ComboBoxEvent::OpenChanged {{ open: {open} }}"),
        ComboBoxEvent::Dismiss => "ComboBoxEvent::Dismiss".to_string(),
        ComboBoxEvent::FocusChanged { focused } => format!("ComboBoxEvent::FocusChanged {{ focused: {focused} }}"),
        _ => "ComboBoxEvent::(unknown)".to_string(),
    }
}
