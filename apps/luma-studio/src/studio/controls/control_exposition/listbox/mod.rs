//! Host-composed ListBox examples: rows, cards, transfers, and a large spectrum collection.

mod markup;
mod filtering;
pub(super) mod horizontal;
mod presentation;
mod selection_controls;
mod spectrum;
mod variable_height;
mod transfer;
pub(super) mod vertical;

use std::sync::Arc;

use gpui::{App, Context, Entity, Render, Window, div, prelude::*, px};
use xgpui_luma::vstack;
use xgpui_luma::controls::listbox::ListBoxMotion;
use xgpui_luma_look_shadcn::ShadcnLook;

use horizontal::HorizontalListExample;
use filtering::FilteringExample;
use selection_controls::SelectionControls;
use spectrum::SpectrumListExample;
use variable_height::VariableHeightExample;
use transfer::TransferExample;
use vertical::VerticalListExample;
use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::collection_theme_inspectors::ListBoxThemeInspector;
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector::{ListBoxInspectorAdapter, LISTBOX_INSPECTOR_SPEC, listbox_motion_report};
use super::inspector_split::InspectorSplitShell;
use super::model::ControlExpositionLayout;
use super::template::render_control_exposition_card;

/// Read-only layout values supplied independently by each example to its inspector.
#[derive(Clone, Copy)]
pub(super) struct ListBoxSampleLayout {
    pub item_height: f32,
    pub spacing: f32,
    pub item_padding: f32,
    pub viewport_height: f32,
    pub inset_x: f32,
    pub inset_y: f32,
    pub radius: f32,
    pub border: f32,
}

pub(super) const ITEM_CONTENT_GAP: f32 = 6.0;
pub(super) const VERTICAL_LIST_WIDTH: f32 = 250.0;

pub struct ListBoxControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<ListBoxExpositionLeftPane>,
    theme_inspector: Entity<ListBoxThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
}

struct ListBoxExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    selection_controls: Entity<SelectionControls>,
    vertical: Entity<VerticalListExample>,
    horizontal: Entity<HorizontalListExample>,
    transfer: Entity<TransferExample>,
    spectrum: Entity<SpectrumListExample>,
    variable_height: Entity<VariableHeightExample>,
    filtering: Entity<FilteringExample>,
    event_stream: Entity<ControlEventStream>,
}

impl Render for ListBoxExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let preview = vstack! { gap=24.0;
            self.selection_controls.clone(), self.vertical.clone(), self.horizontal.clone(), self.transfer.clone(), self.spectrum.clone(), self.variable_height.clone(), self.filtering.clone(), self.event_stream.clone(),
        }
        .w_full()
        .min_w(px(0.0));
        div()
            .id("controls-doc-listbox-left-pane")
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
    }
}

impl ListBoxControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("listbox").expect("listbox catalog entry");
        let left_pane = cx.new(|cx| {
            let event_stream = cx.new(|cx| {
                ControlEventStream::new(
                    cx,
                    look.clone(),
                    "controls-listbox-event-log",
                    "Click or Tab into a list to scroll it; otherwise the page scrolls. All examples follow the selection policy above. By default, clicking a checked item unchecks it; Shift selects a range. Arrows move focus; Cmd/Ctrl+A selects all; Cmd/Ctrl+Shift+A clears; Escape leaves focus; Enter selects and activates.",
                )
            });
            let vertical =
                cx.new(|cx| VerticalListExample::new(look.clone(), event_stream.clone(), cx));
            let horizontal = cx
                .new(|cx| HorizontalListExample::new(look.clone(), event_stream.clone(), cx));
            let transfer = cx.new(|cx| TransferExample::new(look.clone(), event_stream.clone(), cx));
            let spectrum = cx.new(|cx| SpectrumListExample::new(look.clone(), event_stream.clone(), cx));
            let variable_height = cx.new(|cx| VariableHeightExample::new(look.clone(), event_stream.clone(), cx));
            let filtering = cx.new(|cx| FilteringExample::new(look.clone(), event_stream.clone(), cx));
            let selection_controls = cx.new(|cx| {
                let vertical = vertical.clone();
                let horizontal = horizontal.clone();
                let transfer = transfer.clone();
                let spectrum = spectrum.clone();
                let variable_height = variable_height.clone();
                let filtering = filtering.clone();
                SelectionControls::new(look.clone(), move |policy, cx| {
                    vertical.update(cx, |list, cx| list.set_selection_policy(policy, cx));
                    horizontal.update(cx, |list, cx| list.set_selection_policy(policy, cx));
                    transfer.update(cx, |list, cx| list.set_selection_policy(policy, cx));
                    spectrum.update(cx, |list, cx| list.set_selection_policy(policy, cx));
                    variable_height.update(cx, |list, cx| list.set_selection_policy(policy, cx));
                    filtering.update(cx, |list, cx| list.set_selection_policy(policy, cx));
                }, cx)
            });
            ListBoxExpositionLeftPane { look: look.clone(), entry, selection_controls, vertical, horizontal, transfer, spectrum, variable_height, filtering, event_stream }
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-listbox-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &LISTBOX_INSPECTOR_SPEC,
            ListBoxInspectorAdapter::shared(),
        );
        let vertical = left_pane.read(cx).vertical.clone();
        let horizontal = left_pane.read(cx).horizontal.clone();
        let spectrum = left_pane.read(cx).spectrum.clone();
        let filtering = left_pane.read(cx).filtering.clone();
        let variable_height = left_pane.read(cx).variable_height.clone();
        theme_inspector.update(cx, |inspector, cx| {
            inspector.set_motion_report(
                listbox_motion_report([
                    ("Vertical", list_motion(vertical, VerticalListExample::captured_motion)),
                    ("Horizontal", list_motion(horizontal, HorizontalListExample::captured_motion)),
                    ("Spectrum", list_motion(spectrum, SpectrumListExample::captured_motion)),
                    ("Filtering and sorting", list_motion(filtering, FilteringExample::captured_motion)),
                    ("Content · eager", list_motion_index(variable_height.clone(), 0)),
                    ("Content · virtualized", list_motion_index(variable_height, 1)),
                ]),
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
        self.left_pane.update(cx, |pane, cx| {
            pane.look = look.clone();
            pane.selection_controls.update(cx, |controls, cx| controls.sync_look(look.clone(), cx));
            pane.vertical.update(cx, |sample, cx| sample.sync_look(look.clone(), cx));
            pane.horizontal.update(cx, |sample, cx| sample.sync_look(look.clone(), cx));
            pane.transfer.update(cx, |sample, cx| sample.sync_look(look.clone(), cx));
            pane.spectrum.update(cx, |sample, cx| sample.sync_look(look.clone(), cx));
            pane.variable_height.update(cx, |sample, cx| sample.sync_look(look.clone(), cx));
            pane.filtering.update(cx, |sample, cx| sample.sync_look(look.clone(), cx));
            pane.event_stream.update(cx, |stream, cx| stream.sync_look(look.clone(), cx));
            cx.notify();
        });
        sync_viewport_inspector(look, &self.theme_inspector, &self.inspector_split, cx);
        cx.notify();
    }
}

fn list_motion<T: 'static>(
    entity: Entity<T>,
    read: fn(&T) -> (bool, ListBoxMotion),
) -> Arc<dyn Fn(&App) -> (bool, ListBoxMotion) + Send + Sync> {
    Arc::new(move |cx| read(entity.read(cx)))
}

fn list_motion_index(
    example: Entity<VariableHeightExample>,
    index: usize,
) -> Arc<dyn Fn(&App) -> (bool, ListBoxMotion) + Send + Sync> {
    Arc::new(move |cx| example.read(cx).captured_motions(cx)[index].clone())
}

impl Render for ListBoxControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("controls-doc-listbox-exposition")
            .size_full()
            .min_h(px(0.0))
            .min_w(px(0.0))
            .child(self.inspector_split.clone())
    }
}
