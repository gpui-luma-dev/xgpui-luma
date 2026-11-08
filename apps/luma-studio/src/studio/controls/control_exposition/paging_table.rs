//! Paging table control exposition — paged task grid with embedded pager.

use xgpui_luma::controls::table::TableEventExt;
use std::sync::Arc;

use gpui::{Context, Entity, Render, Subscription, Window, div, prelude::*, px};
use xgpui_luma::controls::table::{TableSelectionMode, TableEvent, PagingTable};
use xgpui_luma::controls::pager::PagerStyle;
use xgpui_luma::{column, column_emphasis, paging_table};
use xgpui_luma_look_shadcn::prelude::*;
use xgpui_luma_look_shadcn as shadcn;
use xgpui_luma_look_shadcn::ShadcnLook;
use lucide_svg_static::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::collection_theme_inspectors::TableThemeInspector;
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::table_demo::{Task, build_task_rows, email_column, status_cell, tag_pill};
use super::inspector::{TableInspectorAdapter, TABLE_INSPECTOR_SPEC, table_motion_report};
use super::model::{ControlExpositionLayout};
use super::template::render_control_exposition_card;

const DEFAULT_PAGE_SIZE: usize = 10;

pub struct PagingTableControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<PagingTableExpositionLeftPane>,
    theme_inspector: Entity<TableThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct PagingTableExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    table: PagingTable<Task>,
    event_stream: Entity<ControlEventStream>,
}

impl PagingTableExpositionLeftPane {
    fn handle_list_event(&mut self, event: &TableEvent, cx: &mut Context<Self>) {
        if matches!(event, TableEvent::SelectionChanged { .. } | TableEvent::SelectedKeysChanged { .. }) {
            cx.notify();
        }
    }

    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.table.update(cx, |_, cx| cx.notify());
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for PagingTableExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let look = &self.look;
            let chrome = look.chrome();

            let preview = div()
                .w_full()
                .max_w(px(760.0))
                .flex()
                .flex_col()
                .items_start()
                .gap(px(16.0))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_sm()
                                .text_color(chrome.muted_text)
                                .child("Drag rows or a selected group to reorder at a gap on this page. Escape cancels."),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(chrome.body_text)
                                .child(format!("Selected task IDs: {}", self.table.read(cx).list().read(cx).selected_keys().iter().map(|key| key.as_ref()).collect::<Vec<_>>().join(", "))),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(chrome.muted_text)
                                .child("Ctrl/Cmd-click toggles rows; Shift-click extends selection. Dragging does not switch pages."),
                        ),
                )
                .child(self.table.clone())
                .child(self.event_stream.clone());

            div()
                .id("controls-doc-paging-table-left-pane")
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

impl PagingTableControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("paging-table").expect("paging-table catalog entry");

        let tasks = build_task_rows();
        let table = paging_table! {
            table_theme = look.table_theme();
            id = "controls-doc-table-paged";
            items = tasks;
            page_size = DEFAULT_PAGE_SIZE;
            pager = shadcn::Pager::new("controls-doc-table-paged-pager").look(look.as_ref())
                .style(PagerStyle::MinimalEdge)
                .page_size(DEFAULT_PAGE_SIZE)
                .into_sdk_builder(cx);
            selection = TableSelectionMode::Extended;
            selected_index = 1;
            active_index = 1;
            row_label = |row| row.title.clone();
            row_enabled = |row| row.enabled;
            grid_view = {
                column_emphasis!("Task", width = 108 => |row: &Task| row.id.clone()),
                column!("Title" => |row: &Task| {
                    div()
                        .w_full()
                        .min_w(px(0.0))
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(tag_pill(row.tag))
                        .child(
                            div()
                                .flex_1()
                                .min_w(px(0.0))
                                .truncate()
                                .child(row.title.clone()),
                        )
                }),
                column!("Status", width = 132 => |row: &Task| status_cell(row.status)),
                email_column(),
                column!("", width = 44 => |_row: &Task| {
                    div()
                        .w_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(xgpui_luma::infra::icon::lucide_icon(
                            LucideIcon::EllipsisVertical,
                            gpui::hsla(0.0, 0.0, 0.5, 1.0),
                            16.0,
                        ))
                }),
            };
            row_template = |model, cells, _window, _cx| {
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .min_h(px(model.look.min_height))
                    .py(px(model.look.padding_y))
                    .bg(model.look.background)
                    .text_color(model.look.label_color)
                    .text_size(px(model.look.label_typography.size))
                    .line_height(px(model.look.label_typography.line_height))
                    .font_weight(model.look.label_typography.weight)
                    .child(cells)
            };
        }
        .spawn(cx);

        let list = table.read(cx).list().clone();
        list.update(cx, |table, cx| {
            table.set_row_key(|row| row.id.clone(), cx).expect("unique task IDs");
            table.set_row_reordering(true, cx).expect("keyed table");
        });

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-paging-table-event-log",
                "Select rows and change pages; TableEvent variants appear below.",
            )
        });

        let left_pane = cx.new(|_| PagingTableExpositionLeftPane {
            look: look.clone(),
            entry,
            table: table.clone(),
            event_stream: event_stream.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-paging-table-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &TABLE_INSPECTOR_SPEC,
            TableInspectorAdapter::shared(),
        );
        theme_inspector.update(cx, |inspector, cx| {
            inspector.set_motion_report(table_motion_report([("Paging", table.read(cx).list().clone())]), cx);
        });

        table.clone().on_event(cx, {
            let event_stream = event_stream.downgrade();
            let left_pane = left_pane.downgrade();
            move |_, event: &TableEvent, cx| {
                let Some(event_stream) = event_stream.upgrade() else {
                    return;
                };
                let Some(left_pane) = left_pane.upgrade() else {
                    return;
                };
                left_pane.update(cx, |pane, cx| pane.handle_list_event(event, cx));
                if let Some(line) = format_table_event(event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }
        });

        Self { look, entry, left_pane, theme_inspector, inspector_split, _subscriptions: { Vec::new() } }
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

impl Render for PagingTableControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-paging-table-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

fn format_table_event(event: &TableEvent) -> Option<String> {
    match event {
        TableEvent::RowDrag(event) => Some(format!("{event:?}")),
        TableEvent::SelectedKeysChanged { selected_keys } => Some(format!("Selected task IDs: {selected_keys:?}")),
        TableEvent::SelectionChanged { selected_indices } => {
            Some(format!("TableEvent::SelectionChanged {{ selected_indices: {selected_indices:?} }}"))
        }
        TableEvent::ActiveIndexChanged { active_index } => {
            Some(format!("TableEvent::ActiveIndexChanged {{ active_index: {active_index:?} }}"))
        }
        TableEvent::ScrollChanged { top_index } => {
            Some(format!("TableEvent::ScrollChanged {{ top_index: {top_index} }}"))
        }
        TableEvent::PageChanged { page } => Some(format!("TableEvent::PageChanged {{ page: {page} }}")),
        TableEvent::PageSizeChanged { page_size } => {
            Some(format!("TableEvent::PageSizeChanged {{ page_size: {page_size} }}"))
        }
        TableEvent::FocusChanged { focused } => Some(format!("TableEvent::FocusChanged {{ focused: {focused} }}")),
        TableEvent::RowHoverChanged { index, hovered } => {
            Some(format!("TableEvent::RowHoverChanged {{ index: {index}, hovered: {hovered} }}"))
        }
        TableEvent::EnabledChanged { enabled } => Some(format!("TableEvent::EnabledChanged {{ enabled: {enabled} }}")),
        _ => None,
    }
}
